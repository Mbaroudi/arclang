//! A twin's live data from Kafka: the topics of `topics:`, every partition,
//! read from `window` ago (Kafka keeps what was sent, so the first run of the
//! twin needs no wait) and then as messages arrive. A message is decoded as
//! an MQTT message is: a number, its column the message's key (else the
//! topic's last level, `house.indoor_c` gives `indoor_c`), or a JSON object;
//! its instant is its `time`, else the message's own timestamp.
//!
//! `kafkas://` connects over TLS, the broker's certificate checked against
//! the system's trusted roots. A user and a password come from
//! `ARCLANG_KAFKA_USERNAME` and `ARCLANG_KAFKA_PASSWORD`, never from the
//! model, with the SASL mechanism of `sasl:` (PLAIN by default).
//!
//! The twin reads without a consumer group: it commits nothing and moves no
//! other consumer's offsets.

use super::twin_live::{decode, now, Feed, Status};
use rskafka::client::consumer::{StartOffset, StreamConsumerBuilder};
use rskafka::client::partition::{OffsetAt, UnknownTopicHandling};
use rskafka::client::{Client, ClientBuilder, Credentials, SaslConfig};
use rskafka::BackoffConfig;
use futures::StreamExt;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::time::Duration;

/// How long a request retries before the twin says what is wrong.
const PATIENCE: Duration = Duration::from_secs(5);
const RETRY: Duration = Duration::from_secs(2);
/// How often the broker is asked whether it is still there: a reader waiting for messages
/// retries on its own and would not tell a broker gone from a quiet topic.
const HEARTBEAT: Duration = Duration::from_secs(5);

/// Where each partition is read next: a reconnection takes up where the last one stopped.
type Positions = Arc<Mutex<HashMap<(String, i32), i64>>>;

/// A partition's reader, or the heartbeat: Ok when the run ends, Err when it fails.
type Reader<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + 'a>>;

/// The column of a message that is a bare number: its key, else the topic's last level.
pub fn column(topic: &str, key: Option<&[u8]>) -> String {
    match key.and_then(|k| std::str::from_utf8(k).ok()).map(str::trim).filter(|k| !k.is_empty()) {
        Some(key) => key.to_string(),
        None => topic.rsplit(['.', '/']).next().unwrap_or(topic).to_string(),
    }
}

fn sasl(mechanism: &str) -> Option<SaslConfig> {
    let (user, password) = super::twin_credentials::kafka()?;
    let credentials = Credentials::new(user, password);
    Some(match mechanism {
        "scram-sha-256" => SaslConfig::ScramSha256(credentials),
        "scram-sha-512" => SaslConfig::ScramSha512(credentials),
        _ => SaslConfig::Plain(credentials),
    })
}

fn tls() -> Result<Arc<rustls::ClientConfig>, String> {
    super::twin_tls::client_config("kafkas://")
}

/// Follows the topics in a thread of its own.
#[allow(clippy::too_many_arguments)]
pub fn follow(url: &str, brokers: &[String], tls_on: bool, topics: &[String], client_id: Option<&str>, mechanism: &str, window: f64, feed: Sender<Feed>) -> Result<(), String> {
    let tls = if tls_on { Some(tls()?) } else { None };
    let sasl = sasl(mechanism);
    let client_id = client_id.map(str::to_string).unwrap_or_else(|| format!("arclang-twin-{}", std::process::id()));
    let (url, brokers, topics) = (url.to_string(), brokers.to_vec(), topics.to_vec());
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| format!("Kafka: {e}"))?;
    let positions: Positions = Arc::default();
    std::thread::spawn(move || {
        runtime.block_on(async move {
            let mut connected = false;
            loop {
                let mut builder = ClientBuilder::new(brokers.clone())
                    .client_id(client_id.as_str())
                    .backoff_config(BackoffConfig { deadline: Some(PATIENCE), ..BackoffConfig::default() });
                if let Some(tls) = &tls {
                    builder = builder.tls_config(tls.clone());
                }
                if let Some(sasl) = &sasl {
                    builder = builder.sasl_config(sasl.clone());
                }
                let outcome = match builder.build().await {
                    Ok(client) => read(&client, &url, &topics, window, &positions, &feed, &mut connected).await,
                    Err(error) => Err(explain(&error.to_string())),
                };
                let Err(error) = outcome else { return };
                let status = if refused(&error) {
                    Status::new("refused", format!("{url}: {error}"))
                } else if connected {
                    Status::new("disconnected", error)
                } else {
                    Status::new("unreachable", format!("{url}: {error}"))
                };
                connected = false;
                if feed.send(Feed::Status(status)).is_err() {
                    return;
                }
                tokio::time::sleep(RETRY).await;
            }
        })
    });
    Ok(())
}

/// Reads every partition of the topics until one fails (Err) or the run ends (Ok).
async fn read(client: &Client, url: &str, topics: &[String], window: f64, positions: &Positions, feed: &Sender<Feed>, connected: &mut bool) -> Result<(), String> {
    let known = client.list_topics().await.map_err(|e| explain(&e.to_string()))?;
    let mut partitions = Vec::new();
    for topic in topics {
        let found = known.iter().find(|t| &t.name == topic).ok_or_else(|| format!("the topic '{topic}' does not exist"))?;
        partitions.extend(found.partitions.iter().map(|p| (topic.clone(), *p)));
    }
    let mut readers: Vec<Reader> = vec![Box::pin(heartbeat(client))];
    for (topic, partition) in &partitions {
        let client = client.partition_client(topic.clone(), *partition, UnknownTopicHandling::Error).await.map_err(|e| explain(&e.to_string()))?;
        let next = positions.lock().ok().and_then(|p| p.get(&(topic.clone(), *partition)).copied());
        let start = match next {
            Some(offset) => offset,
            None => from(&client, window).await?,
        };
        readers.push(Box::pin(partition_reader(Arc::new(client), topic.clone(), *partition, start, positions.clone(), feed.clone())));
    }
    *connected = true;
    if feed.send(Feed::Status(Status::new("connected", format!("{url} ({}, {} partition(s))", topics.join(", "), partitions.len())))).is_err() {
        return Ok(());
    }
    // The first partition to stop (or the heartbeat) says why; the others stop with it.
    let (outcome, _, _) = futures::future::select_all(readers).await;
    outcome
}

/// Fails when the broker no longer answers.
async fn heartbeat(client: &Client) -> Result<(), String> {
    loop {
        tokio::time::sleep(HEARTBEAT).await;
        match tokio::time::timeout(HEARTBEAT, client.list_topics()).await {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => return Err(explain(&error.to_string())),
            Err(_) => return Err(format!("the broker has not answered for {} s", HEARTBEAT.as_secs())),
        }
    }
}

/// The offset `window` ago: the first message since then, else the next one to come.
async fn from(client: &rskafka::client::partition::PartitionClient, window: f64) -> Result<i64, String> {
    let since = chrono::DateTime::from_timestamp_millis(((now() - window) * 1000.0) as i64).unwrap_or_default();
    match client.get_offset(OffsetAt::Timestamp(since)).await {
        Ok(offset) if offset >= 0 => Ok(offset),
        _ => client.get_offset(OffsetAt::Latest).await.map_err(|e| explain(&e.to_string())),
    }
}

async fn partition_reader(client: Arc<rskafka::client::partition::PartitionClient>, topic: String, partition: i32, start: i64, positions: Positions, feed: Sender<Feed>) -> Result<(), String> {
    let mut stream = StreamConsumerBuilder::new(client, StartOffset::At(start)).with_max_wait_ms(500).build();
    while let Some(item) = stream.next().await {
        let (message, _) = item.map_err(|e| explain(&e.to_string()))?;
        let record = message.record;
        let arrival = record.timestamp.timestamp_millis() as f64 / 1000.0;
        let samples = decode(&column(&topic, record.key.as_deref()), record.value.as_deref().unwrap_or_default(), arrival);
        if let Ok(mut positions) = positions.lock() {
            positions.insert((topic.clone(), partition), message.offset + 1);
        }
        if !samples.is_empty() && feed.send(Feed::Samples(samples)).is_err() {
            return Ok(());
        }
    }
    Err(format!("the partition {partition} of '{topic}' stopped"))
}

const CREDENTIALS: &str = "the user and password come from ARCLANG_KAFKA_USERNAME and ARCLANG_KAFKA_PASSWORD, the mechanism from sasl:";

/// Whether the broker turned the twin's user down (rather than not being there).
fn refused(error: &str) -> bool {
    error.contains(CREDENTIALS)
}

/// The library's error with what to do about it, when it says enough to tell.
pub fn explain(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("sasl") || lower.contains("authentication") || lower.contains("authorization") {
        return format!("{error} ({CREDENTIALS})");
    }
    if lower.contains("early eof") {
        return format!("{error} (the broker closed the connection: does it ask for a user and a password, ARCLANG_KAFKA_USERNAME and ARCLANG_KAFKA_PASSWORD, or for TLS, kafkas://?)");
    }
    if lower.contains("certificate") || lower.contains("invalidcertificate") {
        return format!("{error} (kafkas:// checks the broker's certificate against the system's trusted roots)");
    }
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_is_named_by_its_key_else_by_its_topic() {
        assert_eq!(column("house.telemetry", Some(b"indoor_c")), "indoor_c");
        assert_eq!(column("house.indoor_c", None), "indoor_c");
        assert_eq!(column("site/042/outdoor_c", Some(b" ")), "outdoor_c");
        assert_eq!(decode(&column("house.indoor_c", None), b"21.5", 10.0), vec![(10.0, "indoor_c".to_string(), 21.5)]);
        assert!(refused(&explain("Sasl handshake failed: API error: SaslAuthenticationFailed")));
        assert!(!refused(&explain("Cannot read data: early eof")));
        assert!(explain("Cannot read data: early eof").contains("kafkas://"));
    }
}
