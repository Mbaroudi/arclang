//! A twin on live data: `arclang twin --live` subscribes to the twin's MQTT
//! topics, keeps what arrives, and at every `refresh` runs the twin over the
//! latest `window` of data. The window is in the data's own time (a
//! message's `time`, else its arrival), so a replay of past data at full
//! speed is a live run too. With `record:`, every sample received is
//! appended to a CSV that `arclang twin` replays later.
//!
//! A message is either a number (the column is the topic's last level:
//! `site/042/indoor_c` gives `indoor_c`) or a JSON object (a column per
//! numeric or boolean field, its `time` an ISO 8601 date-time or seconds,
//! or milliseconds, since 1970). The broker's credentials come from
//! `ARCLANG_MQTT_USERNAME` and `ARCLANG_MQTT_PASSWORD`, never from the model.

use super::tool::ToolConfig;
use super::twin::{self, Live, Measurements, Outcome, Spec};
use crate::compiler::ast::Twin;
use crate::compiler::CompilationResult;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
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
    let column = topic.rsplit('/').next().unwrap_or(topic).to_string();
    if let Ok(value) = text.parse::<f64>() {
        return if value.is_finite() { vec![(arrival, column, value)] } else { Vec::new() };
    }
    let Ok(serde_json::Value::Object(fields)) = serde_json::from_str::<serde_json::Value>(text) else { return Vec::new() };
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

/// Follows the broker until `until` (or forever): `emit` receives each run of the twin, `say` the connection's news.
pub fn live(result: &CompilationResult, model_path: &Path, twin: &Twin, config: &ToolConfig, until: Option<Duration>, mut emit: impl FnMut(&Outcome), mut say: impl FnMut(&Status)) -> Result<(), String> {
    use rumqttc::{Client, Event, MqttOptions, Packet, QoS, Transport};
    let spec = twin::spec_of(&twin.attributes)?;
    let live: Live = spec.data.live.clone().ok_or_else(|| format!("twin '{}' reads a file, not live data: data: {{ mqtt: \"mqtt://broker:1883\" topics: [...] }}", twin.name))?;
    let (tls, host, port) = broker(&live.url)?;
    let client_id = live.client_id.clone().unwrap_or_else(|| format!("arclang-twin-{}", std::process::id()));
    let mut options = MqttOptions::new(client_id, host.clone(), port);
    options.set_keep_alive(Duration::from_secs(30));
    if tls {
        options.set_transport(Transport::tls_with_config(rumqttc::TlsConfiguration::Native));
    }
    if let (Ok(user), Ok(password)) = (std::env::var("ARCLANG_MQTT_USERNAME"), std::env::var("ARCLANG_MQTT_PASSWORD")) {
        options.set_credentials(user, password);
    }
    let (client, mut connection) = Client::new(options, 64);
    for topic in &live.topics {
        client.subscribe(topic.clone(), QoS::AtMostOnce).map_err(|e| format!("cannot subscribe to {topic}: {e}"))?;
    }
    let base = super::cache::model_dir(model_path);
    let record_path = live.record.as_ref().map(|r| base.join(r));
    let wanted = columns(&spec);
    let mut buffer = Buffer::default();
    let started = Instant::now();
    let mut last_run = Instant::now();
    let mut new_samples = 0usize;
    let mut connected = false;
    let mut pending: Vec<(f64, BTreeMap<String, f64>)> = Vec::new();
    let refresh = Duration::from_secs_f64(live.refresh);
    loop {
        let done = until.is_some_and(|u| started.elapsed() >= u);
        if done || last_run.elapsed() >= refresh {
            if let Some(path) = &record_path {
                if let Err(error) = record(path, &spec.data.time, &wanted, &pending) {
                    say(&Status { status: "record".into(), detail: format!("{} cannot be written: {error}", path.display()) });
                }
            }
            pending.clear();
            if new_samples > 0 {
                match buffer.window(live.window, &wanted) {
                    Some(measurements) => {
                        let label = format!("{} ({}, last {})", live.url, live.topics.join(" "), super::twin::window_text(live.window));
                        emit(&twin::run_on(result, model_path, twin, &spec, measurements, &label, config, Instant::now()));
                        new_samples = 0;
                    }
                    None => say(&Status { status: "waiting".into(), detail: format!("{} sample(s) so far: the twin runs from two", buffer.len()) }),
                }
            }
            last_run = Instant::now();
            buffer.trim(live.window * 2.0);
            if done {
                return Ok(());
            }
        }
        match connection.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(Event::Incoming(Packet::ConnAck(_)))) => {
                connected = true;
                say(&Status { status: "connected".into(), detail: format!("{} ({})", live.url, live.topics.join(", ")) });
            }
            Ok(Ok(Event::Incoming(Packet::Publish(message)))) => {
                let arrival = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
                let mut row: BTreeMap<String, f64> = BTreeMap::new();
                let mut at = arrival;
                for (t, column, value) in decode(&message.topic, &message.payload, arrival) {
                    if wanted.contains(&column) {
                        buffer.push(t, &column, value);
                        row.insert(column, value);
                        at = t;
                        new_samples += 1;
                    }
                }
                if !row.is_empty() {
                    pending.push((at, row));
                }
            }
            Ok(Ok(_)) => {}
            Ok(Err(error)) => {
                if connected {
                    say(&Status { status: "disconnected".into(), detail: error.to_string() });
                } else {
                    say(&Status { status: "unreachable".into(), detail: format!("{}: {error}", live.url) });
                }
                connected = false;
                std::thread::sleep(Duration::from_secs(2));
            }
            Err(rumqttc::RecvTimeoutError::Timeout) => {}
            Err(rumqttc::RecvTimeoutError::Disconnected) => return Err("the MQTT connection closed".to_string()),
        }
    }
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
