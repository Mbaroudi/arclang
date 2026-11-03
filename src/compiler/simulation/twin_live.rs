//! A twin on live data: `arclang twin --live` follows the twin's source (the
//! topics of an MQTT broker or of Kafka, the nodes of an OPC UA server, an
//! HTTP API read at intervals), keeps what
//! arrives, and at every `refresh` runs the twin over the latest `window` of
//! data. The window is in the data's own time (a message's `time`, a value's
//! source timestamp, else its arrival), so a replay of past data at full
//! speed is a live run too. With `record:`, every sample received is
//! appended to a CSV that `arclang twin` replays later.
//!
//! An MQTT message is either a number (the column is the topic's last level:
//! `site/042/indoor_c` gives `indoor_c`) or a JSON object (a column per
//! numeric or boolean field, its `time` an ISO 8601 date-time or seconds,
//! or milliseconds, since 1970). The broker's credentials come from
//! `ARCLANG_MQTT_USERNAME` and `ARCLANG_MQTT_PASSWORD`, never from the model;
//! the other sources' are in [`super::twin_opcua`], [`super::twin_kafka`] and
//! [`super::twin_http`].

use super::tool::ToolConfig;
use super::twin::{self, Live, Measurements, Outcome, Source, Spec};
use crate::compiler::ast::Twin;
use crate::compiler::CompilationResult;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

/// Samples by instant (seconds since 1970), the columns of each.
#[derive(Debug, Default, Clone)]
pub struct Buffer {
    samples: BTreeMap<i64, BTreeMap<String, f64>>,
}

/// Instants are kept to the millisecond: two messages of the same instant make one sample.
fn key(t: f64) -> i64 {
    (t * 1000.0).round() as i64
}

impl Buffer {
    pub fn push(&mut self, t: f64, column: &str, value: f64) {
        self.samples.entry(key(t)).or_default().insert(column.to_string(), value);
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// The latest instant received.
    pub fn latest(&self) -> Option<f64> {
        self.samples.keys().next_back().map(|k| *k as f64 / 1000.0)
    }

    /// Forget what is older than `keep` seconds before the latest instant.
    pub fn trim(&mut self, keep: f64) {
        if let Some(latest) = self.latest() {
            let oldest = key(latest - keep);
            self.samples = self.samples.split_off(&oldest);
        }
    }

    /// The instants after `t`, in order, with their columns.
    pub fn after(&self, t: f64) -> Vec<(f64, BTreeMap<String, f64>)> {
        use std::ops::Bound::{Excluded, Unbounded};
        self.samples.range((Excluded(key(t)), Unbounded)).map(|(k, v)| (*k as f64 / 1000.0, v.clone())).collect()
    }

    /// The first instant of the latest `window` seconds.
    pub fn window_start(&self, window: f64) -> Option<f64> {
        let latest = self.latest()?;
        self.samples.range(key(latest - window)..).next().map(|(k, _)| *k as f64 / 1000.0)
    }

    /// The latest `window` seconds as measurements of the given columns; None with fewer than two samples.
    pub fn window(&self, window: f64, columns: &[String]) -> Option<Measurements> {
        let latest = self.latest()?;
        let from = key(latest - window);
        let rows: Vec<(f64, &BTreeMap<String, f64>)> = self.samples.range(from..).map(|(k, v)| (*k as f64 / 1000.0, v)).collect();
        if rows.len() < 2 {
            return None;
        }
        let first = rows[0].0;
        let start = chrono::DateTime::from_timestamp_millis((first * 1000.0) as i64).map(|t| t.to_rfc3339()).unwrap_or_else(|| format!("{first}"));
        Some(Measurements {
            times: rows.iter().map(|(t, _)| t - first).collect(),
            columns: columns.iter().map(|c| (c.clone(), rows.iter().map(|(_, v)| v.get(c).copied()).collect())).collect(),
            start,
        })
    }
}

/// An instant written in a message: an ISO 8601 text, or seconds (milliseconds above 1e11) since 1970.
fn instant(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64().map(|v| if v > 1e11 { v / 1000.0 } else { v }),
        serde_json::Value::String(text) => chrono::DateTime::parse_from_rfc3339(text).ok().map(|t| t.timestamp_millis() as f64 / 1000.0),
        _ => None,
    }
}

/// The samples of a message: (instant, column, value).
pub fn decode(topic: &str, payload: &[u8], arrival: f64) -> Vec<(f64, String, f64)> {
    let Ok(text) = std::str::from_utf8(payload) else { return Vec::new() };
    let text = text.trim();
    let column = topic.rsplit('/').next().unwrap_or(topic);
    if let Ok(value) = text.parse::<f64>() {
        return if value.is_finite() { vec![(arrival, column.to_string(), value)] } else { Vec::new() };
    }
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(value) => samples_of(&value, arrival),
        Err(_) => Vec::new(),
    }
}

/// The samples of a JSON object (a column per numeric or boolean field, at its `time`, else at
/// `arrival`) or of a list of objects.
pub fn samples_of(value: &serde_json::Value, arrival: f64) -> Vec<(f64, String, f64)> {
    let fields = match value {
        serde_json::Value::Array(items) => return items.iter().flat_map(|item| samples_of(item, arrival)).collect(),
        serde_json::Value::Object(fields) => fields,
        _ => return Vec::new(),
    };
    let t = fields.get("time").and_then(instant).unwrap_or(arrival);
    fields
        .iter()
        .filter(|(name, _)| name.as_str() != "time")
        .filter_map(|(name, value)| {
            let v = match value {
                serde_json::Value::Number(n) => n.as_f64(),
                serde_json::Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                _ => None,
            }?;
            v.is_finite().then(|| (t, name.clone(), v))
        })
        .collect()
}

/// The columns a twin reads: what drives it, what it is compared to.
pub fn columns(spec: &Spec) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for c in spec.drives.iter().map(|d| &d.column).chain(spec.compares.iter().map(|c| &c.column)) {
        if !out.contains(c) {
            out.push(c.clone());
        }
    }
    out
}

/// Appends samples to the record, its header written when it is new.
fn record(path: &Path, time: &str, columns: &[String], rows: &[(f64, BTreeMap<String, f64>)]) -> std::io::Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let new = !path.exists();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    if new {
        writeln!(file, "# Recorded live by arclang twin --live")?;
        writeln!(file, "{time},{}", columns.join(","))?;
    }
    for (t, values) in rows {
        let stamp = chrono::DateTime::from_timestamp_millis((t * 1000.0) as i64).map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)).unwrap_or_default();
        let cells: Vec<String> = columns.iter().map(|c| values.get(c).map(|v| v.to_string()).unwrap_or_default()).collect();
        writeln!(file, "{stamp},{}", cells.join(","))?;
    }
    Ok(())
}

fn broker(url: &str) -> Result<(bool, String, u16), String> {
    let (scheme, rest) = url.split_once("://").ok_or_else(|| format!("'{url}' is not mqtt://host[:port]"))?;
    let tls = scheme == "mqtts";
    let host_port = rest.trim_end_matches('/');
    let (host, port) = match host_port.rsplit_once(':') {
        Some((host, port)) => (host.to_string(), port.parse::<u16>().map_err(|_| format!("'{url}': the port is a number"))?),
        None => (host_port.to_string(), if tls { 8883 } else { 1883 }),
    };
    if host.is_empty() {
        return Err(format!("'{url}' names no host"));
    }
    Ok((tls, host, port))
}

/// What a live run says between runs of the twin.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Status {
    pub status: String,
    pub detail: String,
}

impl Status {
    pub fn new(status: &str, detail: impl Into<String>) -> Status {
        Status { status: status.to_string(), detail: detail.into() }
    }
}

/// What a source sends to the run: its news, or samples (instant, column, value).
pub enum Feed {
    Status(Status),
    Samples(Vec<(f64, String, f64)>),
}

pub fn now() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

/// Follows an MQTT broker in a thread of its own.
fn mqtt(live: &Live, topics: &[String], client_id: Option<&str>, feed: Sender<Feed>) -> Result<(), String> {
    use rumqttc::{Client, Event, MqttOptions, Packet, QoS, Transport};
    let (tls, host, port) = broker(&live.url)?;
    let client_id = client_id.map(str::to_string).unwrap_or_else(|| format!("arclang-twin-{}", std::process::id()));
    let mut options = MqttOptions::new(client_id, host, port);
    options.set_keep_alive(Duration::from_secs(30));
    if tls {
        options.set_transport(Transport::tls_with_config(rumqttc::TlsConfiguration::Native));
    }
    if let (Ok(user), Ok(password)) = (std::env::var("ARCLANG_MQTT_USERNAME"), std::env::var("ARCLANG_MQTT_PASSWORD")) {
        options.set_credentials(user, password);
    }
    let (client, mut connection) = Client::new(options, 64);
    for topic in topics {
        client.subscribe(topic.clone(), QoS::AtMostOnce).map_err(|e| format!("cannot subscribe to {topic}: {e}"))?;
    }
    let (url, topics) = (live.url.clone(), topics.join(", "));
    std::thread::spawn(move || {
        let _client = client;
        let mut connected = false;
        for event in connection.iter() {
            let sent = match event {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    connected = true;
                    feed.send(Feed::Status(Status::new("connected", format!("{url} ({topics})"))))
                }
                Ok(Event::Incoming(Packet::Publish(message))) => feed.send(Feed::Samples(decode(&message.topic, &message.payload, now()))),
                Ok(_) => Ok(()),
                Err(error) => {
                    let status = if connected { Status::new("disconnected", error.to_string()) } else { Status::new("unreachable", format!("{url}: {error}")) };
                    connected = false;
                    let sent = feed.send(Feed::Status(status));
                    std::thread::sleep(Duration::from_secs(2));
                    sent
                }
            };
            if sent.is_err() {
                return;
            }
        }
    });
    Ok(())
}

/// Follows the twin's source until `until` (or forever), or until `stop` is set: then what is
/// waiting is recorded and the run ends without a last run of the twin. `emit` receives each run
/// of the twin, `say` the connection's news.
#[allow(clippy::too_many_arguments)]
pub fn live(result: &CompilationResult, model_path: &Path, twin: &Twin, config: &ToolConfig, until: Option<Duration>, stop: &AtomicBool, mut emit: impl FnMut(&Outcome), mut say: impl FnMut(&Status)) -> Result<(), String> {
    let spec = twin::spec_of(&twin.attributes)?;
    let live: Live = spec.data.live.clone().ok_or_else(|| format!("twin '{}' reads a file, not live data: data: {{ mqtt: \"mqtt://broker:1883\" topics: [...] }} (or opcua:, kafka:, http:)", twin.name))?;
    let base = super::cache::model_dir(model_path);
    let (sender, feed): (Sender<Feed>, Receiver<Feed>) = std::sync::mpsc::channel();
    match &live.source {
        Source::Mqtt { topics, client_id } => mqtt(&live, topics, client_id.as_deref(), sender)?,
        Source::OpcUa { nodes, security, policy, sampling } => super::twin_opcua::follow(&live.url, nodes, *security, policy, *sampling, &base, sender)?,
        Source::Kafka { brokers, tls, topics, client_id, sasl } => super::twin_kafka::follow(&live.url, brokers, *tls, topics, client_id.as_deref(), sasl, live.window, sender)?,
        Source::Http { sampling } => super::twin_http::follow(&live.url, *sampling, sender)?,
    }
    // A model that advances with the data is built once, while the source is already followed.
    let mut stepping = if live.step {
        say(&Status::new("building", format!("{}: the model as an FMU, to advance with the data", twin.name)));
        let model = super::twin_step::Model::build(result, model_path, &spec, config)?;
        say(&Status::new("ready", format!("the model advances with the data ({})", model.tool_text)));
        Some(super::twin_step::Stepping::new(model, &spec))
    } else {
        None
    };
    let record_path = live.record.as_ref().map(|r| base.join(r));
    let wanted = columns(&spec);
    let label = live.label();
    let mut buffer = Buffer::default();
    let started = Instant::now();
    let mut last_run = Instant::now();
    let mut new_samples = 0usize;
    // What arrived since the last record, by instant: the values of one instant make one row,
    // whether they came in one message (MQTT JSON) or one notification per node (OPC UA).
    let mut pending: BTreeMap<i64, BTreeMap<String, f64>> = BTreeMap::new();
    let refresh = Duration::from_secs_f64(live.refresh);
    loop {
        let stopped = stop.load(Ordering::Relaxed);
        let done = stopped || until.is_some_and(|u| started.elapsed() >= u);
        if done || last_run.elapsed() >= refresh {
            if let Some(path) = &record_path {
                // The latest instant may still be arriving (one node after the other): it waits for the next record.
                let held = if done { BTreeMap::new() } else { pending.last_key_value().map(|(k, _)| *k).map(|k| pending.split_off(&k)).unwrap_or_default() };
                let rows: Vec<(f64, BTreeMap<String, f64>)> = std::mem::replace(&mut pending, held).into_iter().map(|(k, row)| (k as f64 / 1000.0, row)).collect();
                if let Err(error) = record(path, &spec.data.time, &wanted, &rows) {
                    say(&Status::new("record", format!("{} cannot be written: {error}", path.display())));
                }
            }
            if stopped {
                return Ok(());
            }
            if let Some(stepping) = stepping.as_mut() {
                advance(stepping, &buffer, done, &mut say)?;
            }
            if new_samples > 0 {
                let judged = match (buffer.window(live.window, &wanted), &stepping) {
                    (Some(measurements), Some(stepping)) => stepping.outcome(twin, &spec, measurements, buffer.window_start(live.window).unwrap_or(0.0), &label, Instant::now()),
                    (Some(measurements), None) => Some(twin::run_on(result, model_path, twin, &spec, measurements, &label, config, Instant::now())),
                    (None, _) => None,
                };
                match judged {
                    Some(outcome) => {
                        emit(&outcome);
                        new_samples = 0;
                    }
                    None => say(&Status::new("waiting", format!("{} sample(s) so far: the twin runs from two", buffer.len()))),
                }
            }
            last_run = Instant::now();
            buffer.trim(live.window * 2.0);
            if let Some(stepping) = stepping.as_mut() {
                stepping.trim(live.window * 2.0);
            }
            if done {
                return Ok(());
            }
        }
        match feed.recv_timeout(Duration::from_millis(200)) {
            Ok(Feed::Status(status)) => say(&status),
            Ok(Feed::Samples(samples)) => {
                for (t, column, value) in samples {
                    if wanted.contains(&column) {
                        buffer.push(t, &column, value);
                        if record_path.is_some() {
                            pending.entry(key(t)).or_default().insert(column, value);
                        }
                        new_samples += 1;
                    }
                }
                if let Some(stepping) = stepping.as_mut() {
                    advance(stepping, &buffer, false, &mut say)?;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Err(format!("the connection to {} closed", live.url)),
        }
    }
}

/// Steps the model to what arrived; a failure is said and the model started again from the next instant.
fn advance(stepping: &mut super::twin_step::Stepping, buffer: &Buffer, complete: bool, say: &mut impl FnMut(&Status)) -> Result<(), String> {
    if let Err(error) = stepping.advance(buffer, complete) {
        say(&Status::new("model", format!("{error}; the model starts again from the next sample")));
        stepping.restart()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_is_a_number_or_a_json_object() {
        assert_eq!(decode("site/042/indoor_c", b" 21.5 ", 100.0), vec![(100.0, "indoor_c".to_string(), 21.5)]);
        let mut fields = decode("site/042", br#"{"time":"2026-01-14T00:10:00Z","outdoor_c":-9.5,"pump":true,"label":"x"}"#, 1.0);
        fields.sort_by(|a, b| a.1.cmp(&b.1));
        assert_eq!(fields, vec![(1768349400.0, "outdoor_c".to_string(), -9.5), (1768349400.0, "pump".to_string(), 1.0)]);
        // Milliseconds since 1970 are told from seconds by their size.
        assert_eq!(decode("t", br#"{"time":1768349400000,"x":1}"#, 0.0)[0].0, 1768349400.0);
        assert!(decode("t", b"not a value", 0.0).is_empty());
    }

    #[test]
    fn the_window_keeps_the_latest_data_in_its_own_time() {
        let mut buffer = Buffer::default();
        for (i, t) in [0.0, 600.0, 1200.0, 1800.0].iter().enumerate() {
            buffer.push(1_768_348_800.0 + t, "x", i as f64);
            if i % 2 == 0 {
                buffer.push(1_768_348_800.0 + t, "y", 10.0 + i as f64);
            }
        }
        let m = buffer.window(1200.0, &["x".to_string(), "y".to_string()]).expect("three samples");
        assert_eq!(m.times, vec![0.0, 600.0, 1200.0]);
        assert_eq!(m.columns["x"], vec![Some(1.0), Some(2.0), Some(3.0)]);
        assert_eq!(m.columns["y"], vec![None, Some(12.0), None]);
        assert_eq!(m.start, "2026-01-14T00:10:00+00:00");
        buffer.trim(600.0);
        assert_eq!(buffer.len(), 2);
        assert!(Buffer::default().window(60.0, &[]).is_none());
    }

    #[test]
    fn the_broker_is_read_with_its_default_ports() {
        assert_eq!(broker("mqtt://localhost").unwrap(), (false, "localhost".into(), 1883));
        assert_eq!(broker("mqtts://broker.example:8884").unwrap(), (true, "broker.example".into(), 8884));
        assert!(broker("mqtt://:1883").is_err());
    }

    #[test]
    fn the_record_is_a_csv_the_twin_replays() {
        let dir = std::env::temp_dir().join(format!("arclang-record-{}", std::process::id()));
        let path = dir.join("live.csv");
        let _ = std::fs::remove_dir_all(&dir);
        let columns = vec!["a".to_string(), "b".to_string()];
        record(&path, "time", &columns, &[(1_768_348_800.0, BTreeMap::from([("a".to_string(), 1.5)]))]).unwrap();
        record(&path, "time", &columns, &[(1_768_348_860.0, BTreeMap::from([("a".to_string(), 2.0), ("b".to_string(), 3.0)]))]).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let data = twin::Data { file: "live.csv".into(), live: None, time: "time".into(), time_unit: crate::compiler::quantity::lookup_unit("s").unwrap(), delimiter: None };
        let m = twin::read_measurements(&text, &data).unwrap();
        assert_eq!(m.times, vec![0.0, 60.0]);
        assert_eq!(m.columns["b"], vec![None, Some(3.0)]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
