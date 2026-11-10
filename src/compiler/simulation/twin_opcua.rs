//! A twin's live data from an OPC UA server: a subscription with a monitored
//! item per node of `nodes:`, sampled by the server every `sampling`. A value's
//! instant is its source timestamp (else the server's, else its arrival);
//! a value whose status is not good is left out, booleans are 0 and 1.
//!
//! Security: `security: "none"` (the default), `"sign"` or
//! `"sign_and_encrypt"`, with a `policy:` (Basic256Sha256 by default). ArcLang's
//! certificate is made on first use in `.arclang/opcua-pki/` beside the model
//! (or `ARCLANG_OPCUA_PKI`): the server must trust it, and the server's own
//! certificate is trusted once it is moved from `rejected/` to `trusted/`.
//! A user and a password come from `ARCLANG_OPCUA_USERNAME` and
//! `ARCLANG_OPCUA_PASSWORD`, never from the model; otherwise the session is
//! anonymous.

use super::twin::Security;
use super::twin_live::{now, Feed, Status};
use opcua::client::{ClientBuilder, DataChangeCallback, IdentityToken, MonitoredItem, Password, SessionPollResult};
use opcua::types::{DataChangeFilter, DataChangeTrigger, DataValue, ExtensionObject, MessageSecurityMode, MonitoredItemCreateRequest, MonitoringMode, MonitoringParameters, NodeId, ReadValueId, TimestampsToReturn, UserTokenPolicy, Variant};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::time::Duration;

/// Values kept by the server between two publications: a fast replay is not thinned out.
const QUEUE: u32 = 1000;

fn pki(model_dir: &Path) -> PathBuf {
    match std::env::var_os("ARCLANG_OPCUA_PKI") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => model_dir.join(".arclang").join("opcua-pki"),
    }
}

/// A value as a number: numbers as they are, booleans 0 and 1.
pub fn number(value: &Variant) -> Option<f64> {
    match value {
        Variant::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
        other => other.as_f64(),
    }
    .filter(|v| v.is_finite())
}

/// The sample of a data change: (instant, value), None when the value is missing or not good.
pub fn sample(value: &DataValue, arrival: f64) -> Option<(f64, f64)> {
    if value.status.is_some_and(|s| !s.is_good()) {
        return None;
    }
    let v = number(value.value.as_ref()?)?;
    let at = value.source_timestamp.or(value.server_timestamp).map(|t| t.as_chrono().timestamp_millis() as f64 / 1000.0).unwrap_or(arrival);
    Some((at, v))
}

/// Follows the server in a thread of its own; returns once the nodes are read and the thread started.
pub fn follow(url: &str, nodes: &[(String, String)], security: Security, policy: &str, sampling: f64, model_dir: &Path, feed: Sender<Feed>) -> Result<(), String> {
    let mut columns: HashMap<NodeId, String> = HashMap::new();
    for (column, text) in nodes {
        let node = NodeId::from_str(text).map_err(|_| format!("data: nodes: '{text}' is not an OPC UA node id"))?;
        columns.insert(node, column.clone());
    }
    let (url, policy, pki) = (url.to_string(), policy.to_string(), pki(model_dir));
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| format!("OPC UA: {e}"))?;
    std::thread::spawn(move || {
        runtime.block_on(async move {
            loop {
                let error = session(&url, &columns, security, &policy, sampling, &pki, &feed).await;
                if feed.send(Feed::Status(Status::new("unreachable", format!("{url}: {error}")))).is_err() {
                    return;
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        })
    });
    Ok(())
}

/// The URL with the host's first address that accepts a connection: the client tries only the
/// first one, and `localhost` is often `::1` first while the server listens on 127.0.0.1 alone.
/// Unchanged when the first address answers, or none does.
async fn reachable(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else { return url.to_string() };
    let (authority, path) = rest.split_once('/').map(|(a, p)| (a, format!("/{p}"))).unwrap_or((rest, String::new()));
    let Some((host, port)) = authority.rsplit_once(':').filter(|(h, _)| !h.ends_with(']') || h.starts_with('[')) else { return url.to_string() };
    let Ok(addresses) = tokio::net::lookup_host(format!("{host}:{port}")).await else { return url.to_string() };
    for (i, address) in addresses.enumerate() {
        if tokio::time::timeout(Duration::from_secs(2), tokio::net::TcpStream::connect(address)).await.is_ok_and(|c| c.is_ok()) {
            if i == 0 {
                return url.to_string();
            }
            let ip = match address.ip() {
                std::net::IpAddr::V6(ip) => format!("[{ip}]"),
                ip => ip.to_string(),
            };
            return format!("{scheme}://{ip}:{port}{path}");
        }
    }
    url.to_string()
}

/// One session: connects, subscribes, forwards the data changes; returns why it ended.
async fn session(url: &str, columns: &HashMap<NodeId, String>, security: Security, policy: &str, sampling: f64, pki: &Path, feed: &Sender<Feed>) -> String {
    use futures::StreamExt;
    let mode = match security {
        Security::None => MessageSecurityMode::None,
        Security::Sign => MessageSecurityMode::Sign,
        Security::SignAndEncrypt => MessageSecurityMode::SignAndEncrypt,
    };
    let built = ClientBuilder::new()
        .application_name("ArcLang twin")
        .application_uri("urn:arclang:twin")
        .product_uri("urn:arclang")
        .pki_dir(pki)
        .create_sample_keypair(true)
        .trust_server_certs(false)
        .verify_server_certs(true)
        // No retry inside the library (it would retry in silence): a lost session ends, and
        // `follow` says so and makes a new one, its subscription with it, 2 s later.
        .session_retry_limit(0)
        .client();
    let mut client = match built {
        Ok(client) => client,
        Err(errors) => return errors.join("; "),
    };
    let identity = match super::twin_credentials::opcua() {
        Some((user, password)) => IdentityToken::UserName(user, Password::new(password)),
        None => IdentityToken::Anonymous,
    };
    let target = reachable(url).await;
    let endpoint = (target.as_str(), policy, mode, UserTokenPolicy::anonymous());
    let (session, event_loop) = match client.connect_to_matching_endpoint(endpoint, identity).await {
        Ok(connected) => connected,
        Err(error) => return explain(&error.to_string(), pki),
    };
    let mut events = Box::pin(event_loop.enter());
    let mut subscribed = false;
    let label: Vec<&str> = {
        let mut names: Vec<&str> = columns.values().map(String::as_str).collect();
        names.sort();
        names
    };
    while let Some(event) = events.next().await {
        match event {
            Err(code) => return explain(&code.to_string(), pki),
            Ok(SessionPollResult::ConnectionLost(code)) => {
                if feed.send(Feed::Status(Status::new("disconnected", format!("{url}: {code}")))).is_err() {
                    return "stopped".to_string();
                }
            }
            Ok(SessionPollResult::ReconnectFailed(code)) => {
                if feed.send(Feed::Status(Status::new("unreachable", explain(&format!("{url}: {code}"), pki)))).is_err() {
                    return "stopped".to_string();
                }
            }
            Ok(SessionPollResult::Reconnected(_)) => {
                // The requests need the event loop polled meanwhile: they run beside it.
                if !subscribed {
                    subscribed = true;
                    tokio::spawn(subscribe(session.clone(), columns.clone(), sampling, feed.clone()));
                }
                let via = if target == url { String::new() } else { format!(", via {target}") };
                if feed.send(Feed::Status(Status::new("connected", format!("{url} ({}{via})", label.join(", "))))).is_err() {
                    return "stopped".to_string();
                }
            }
            Ok(_) => {}
        }
    }
    "the session ended".to_string()
}

/// A subscription with a monitored item per node; a failure is said, not hidden.
async fn subscribe(session: std::sync::Arc<opcua::client::Session>, columns: HashMap<NodeId, String>, sampling: f64, feed: Sender<Feed>) {
    let forward = feed.clone();
    let by_node = columns.clone();
    let callback = DataChangeCallback::new(move |value: DataValue, item: &MonitoredItem| {
        if let (Some(column), Some((at, v))) = (by_node.get(&item.item_to_monitor().node_id), sample(&value, now())) {
            let _ = forward.send(Feed::Samples(vec![(at, column.clone(), v)]));
        }
    });
    let publishing = Duration::from_secs_f64(sampling.max(0.05));
    let id = match session.create_subscription(publishing, 60, 10, 0, 0, true, callback).await {
        Ok(id) => id,
        Err(error) => {
            let _ = feed.send(Feed::Status(Status::new("error", format!("cannot subscribe: {error}"))));
            return;
        }
    };
    let nodes: Vec<NodeId> = columns.keys().cloned().collect();
    let items: Vec<MonitoredItemCreateRequest> = nodes
        .iter()
        .map(|node| {
            // A new timestamp is a new sample, even with the same value: a steady reading is data too.
            let filter = ExtensionObject::from_message(DataChangeFilter { trigger: DataChangeTrigger::StatusValueTimestamp, deadband_type: 0, deadband_value: 0.0 });
            let parameters = MonitoringParameters { sampling_interval: sampling * 1000.0, queue_size: QUEUE, discard_oldest: true, filter, ..Default::default() };
            MonitoredItemCreateRequest::new(ReadValueId::from(node.clone()), MonitoringMode::Reporting, parameters)
        })
        .collect();
    match session.create_monitored_items(id, TimestampsToReturn::Both, items).await {
        Ok(created) => {
            for (item, node) in created.iter().zip(&nodes) {
                if !item.result.status_code.is_good() {
                    let _ = feed.send(Feed::Status(Status::new("error", format!("{} ({node}) cannot be read: {}", columns[node], item.result.status_code))));
                }
            }
        }
        Err(error) => {
            let _ = feed.send(Feed::Status(Status::new("error", format!("cannot monitor the nodes: {error}"))));
        }
    }
}

/// What to do about a refusal: of a certificate, of an identity or a policy.
fn explain(error: &str, pki: &Path) -> String {
    if error.contains("Certificate") || error.contains("certificate") || error.contains("BadSecurityChecksFailed") {
        format!("{error}: the server's certificate is trusted once moved from {} to {}, and ArcLang's ({}) once the server trusts it", pki.join("rejected").display(), pki.join("trusted").display(), pki.join("own").display())
    } else if ["BadUserAccessDenied", "BadIdentityToken", "BadSecurityPolicyRejected", "BadSecurityModeRejected"].iter().any(|code| error.contains(code)) {
        format!("{error}: the server refuses this identity or this protection; see data: security and policy, and ARCLANG_OPCUA_USERNAME and ARCLANG_OPCUA_PASSWORD (anonymous when they are not set)")
    } else {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opcua::types::{DateTime, StatusCode};

    #[test]
    fn a_value_is_a_number_at_its_source_timestamp() {
        let at = DateTime::from(chrono::DateTime::parse_from_rfc3339("2026-01-14T00:10:00Z").unwrap().with_timezone(&chrono::Utc));
        let value = DataValue { value: Some(Variant::Double(21.5)), source_timestamp: Some(at), ..Default::default() };
        assert_eq!(sample(&value, 5.0), Some((1768349400.0, 21.5)));
        // Without a timestamp, its arrival; a boolean is 0 or 1.
        assert_eq!(sample(&DataValue { value: Some(Variant::Boolean(true)), ..Default::default() }, 5.0), Some((5.0, 1.0)));
        assert_eq!(sample(&DataValue { value: Some(Variant::Int32(-3)), ..Default::default() }, 5.0), Some((5.0, -3.0)));
        // A value that is not good, or not a number, is left out.
        assert_eq!(sample(&DataValue { value: Some(Variant::Double(1.0)), status: Some(StatusCode::BadSensorFailure), ..Default::default() }, 5.0), None);
        assert_eq!(sample(&DataValue { value: Some(Variant::from("text")), ..Default::default() }, 5.0), None);
        assert_eq!(sample(&DataValue { value: Some(Variant::Double(f64::NAN)), ..Default::default() }, 5.0), None);
    }
}
