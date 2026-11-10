//! Digital twins: the physical model of a simulated case, driven by measured
//! data and compared to it.
//!
//! ```text
//! twin "Winter commute, vehicle 042" {
//!     verifies: ["REQ-RANGE"]
//!     simulate: { model: "EVPhysics.CityRange" configuration: "CFG-CITY" sources: ["physics/EVPhysics.mo"] }
//!     data: { file: "data/vehicle-042.csv" time: "timestamp" }
//!     drives: ["\"PN-VCU\".target_speed <- speed_kmh km/h"]
//!     compares: ["\"PN-BATT\".soc ~ bms_soc within 0.03"]
//!     expects: ["min(\"PN-BATT\".soc) >= 0.1"]
//! }
//! ```
//!
//! A drive imposes a measured column on a variable of the model: a wrapper
//! model extends the simulated class, binds the variable to an input of its
//! own, and OpenModelica reads the inputs from the measurements
//! (`-csvInput`), interpolated between the samples. A comparison reads the
//! variable at each measured instant and the deviation from the measurement:
//! its largest, its RMS, the share of samples within the tolerance and the
//! first instant beyond it. The expectations are read on the run, as a
//! simulated case's are. The run lasts as long as the data.

use super::expects::{self, Context};
use super::results::{units_from_init_xml, Series};
use super::tool::{self, ToolConfig};
use crate::compiler::ast::{AttributeValue, Twin};
use crate::compiler::modelica::{self, Export};
use crate::compiler::quantity::{lookup_unit, UnitSpec};
use crate::compiler::CompilationResult;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

const SIMULATE_KEYS: [&str; 4] = ["model", "configuration", "sources", "tolerance"];
const DATA_KEYS: [&str; 19] = ["file", "time", "time_unit", "delimiter", "mqtt", "opcua", "kafka", "http", "topics", "client_id", "nodes", "security", "policy", "sampling", "sasl", "window", "refresh", "record", "advance"];
/// The sources of live data, as `data:` names them.
const LIVE_SOURCES: [&str; 4] = ["mqtt", "opcua", "kafka", "http"];
/// The keys of live data only, whatever the source.
const LIVE_KEYS: [&str; 11] = ["topics", "client_id", "nodes", "security", "policy", "sampling", "sasl", "window", "refresh", "record", "advance"];
/// The keys of one source or a few: the others refuse them.
const SOURCE_KEYS: [(&str, &[&str]); 7] = [
    ("topics", &["mqtt", "kafka"]),
    ("client_id", &["mqtt", "kafka"]),
    ("nodes", &["opcua"]),
    ("security", &["opcua"]),
    ("policy", &["opcua"]),
    ("sampling", &["opcua", "http"]),
    ("sasl", &["kafka"]),
];
/// The SASL mechanisms a twin may ask a Kafka broker for (PLAIN by default, when credentials are set).
pub const KAFKA_SASL: [&str; 3] = ["plain", "scram-sha-256", "scram-sha-512"];
/// The OPC UA security policies a twin may ask for (the None policy goes with `security: "none"`).
pub const OPCUA_POLICIES: [&str; 4] = ["Basic256Sha256", "Aes128Sha256RsaOaep", "Aes256Sha256RsaPss", "Basic128Rsa15"];
const KEYS: [&str; 9] = ["id", "description", "verifies", "simulate", "data", "drives", "compares", "expects", "notes"];
/// Points of each curve kept for the view (the deviations are computed on every sample).
const PLOT_POINTS: usize = 600;
const WRAPPER: &str = "ArcLangTwin";
const RESULT: &str = "arclang_twin";

/// What the measurements impose: `target <- column [unit]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Drive {
    pub target: String,
    pub column: String,
    pub unit: Option<&'static UnitSpec>,
}

/// What the model must reproduce: `target ~ column [unit] within tolerance [unit]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Compare {
    pub target: String,
    pub column: String,
    pub unit: Option<&'static UnitSpec>,
    /// SI: a difference (a tolerance in °C is a tolerance in K).
    pub tolerance: f64,
    pub tolerance_text: String,
    /// The share of the samples that must be within the tolerance (`for 99 %`), 1 by default: real data has outliers.
    pub share: f64,
}

/// Live measurements: where they come from, how much of the latest data
/// the model replays (`window`) and how often (`refresh`); `record` keeps
/// everything received as a CSV the twin replays later.
#[derive(Debug, Clone, PartialEq)]
pub struct Live {
    /// The broker (`mqtt://`, `mqtts://`, `kafka://`, `kafkas://`) or the server (`opc.tcp://`, `http://`, `https://`).
    pub url: String,
    pub source: Source,
    /// Seconds.
    pub window: f64,
    /// Seconds.
    pub refresh: f64,
    pub record: Option<String>,
    /// `advance: "step"`: the model advances with the data, built once as an FMU, instead of
    /// running again over the window at every refresh (`advance: "window"`, the default).
    pub step: bool,
}

/// How OPC UA messages are protected: `security: "none" | "sign" | "sign_and_encrypt"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Security {
    None,
    Sign,
    SignAndEncrypt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// MQTT: the topics subscribed to; a column is a topic's last level or a field of its JSON.
    Mqtt { topics: Vec<String>, client_id: Option<String> },
    /// OPC UA: a column per node (`nodes: { indoor_c: "ns=2;s=Room.T" }`), sampled by the server every `sampling` seconds.
    OpcUa { nodes: Vec<(String, String)>, security: Security, policy: String, sampling: f64 },
    /// Kafka: the topics read from all their partitions, from `window` ago; a column is a field of a
    /// message's JSON, or for a number its key (else the topic's last level).
    Kafka { brokers: Vec<String>, tls: bool, topics: Vec<String>, client_id: Option<String>, sasl: String },
    /// HTTP: a GET every `sampling` seconds, its answer a JSON object or a list of them, as an MQTT message's.
    Http { sampling: f64 },
}

impl Live {
    /// The source as the outcome names it: `opc.tcp://plc:4840 (indoor_c, outdoor_c, last 24 h)`.
    pub fn label(&self) -> String {
        let what = match &self.source {
            Source::Mqtt { topics, .. } | Source::Kafka { topics, .. } => topics.join(" "),
            Source::OpcUa { nodes, .. } => nodes.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>().join(" "),
            Source::Http { sampling } => format!("every {}", window_text(*sampling)),
        };
        format!("{} ({what}, last {}{})", self.url, window_text(self.window), if self.step { ", stepped" } else { "" })
    }
}

/// An OPC UA node id as written in its text form: `ns=2;s=Room.T`, `i=2258`,
/// `ns=3;g=<guid>`, `ns=1;b=<base64>`; also `nsu=<namespace uri>;s=...`.
pub fn is_node_id(text: &str) -> bool {
    let rest = match text.split_once(';') {
        Some((namespace, rest)) => {
            let ok = namespace.strip_prefix("ns=").is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
                || namespace.strip_prefix("nsu=").is_some_and(|u| !u.is_empty());
            if !ok {
                return false;
            }
            rest
        }
        None => text,
    };
    match rest.split_once('=') {
        Some(("i", n)) => !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()),
        Some(("s" | "g" | "b", v)) => !v.is_empty(),
        _ => false,
    }
}

/// The source of live data written in `data:` (`mqtt`, `opcua`, `kafka` or `http`), at `url`.
fn live_of(data: &HashMap<String, AttributeValue>, url: String, source: &str) -> Result<(String, Source), String> {
    let text = |key: &str| data.get(key).and_then(|v| v.as_string()).map(str::to_string);
    if url.split("://").nth(1).unwrap_or(&url).split('/').next().unwrap_or("").contains('@') {
        let hint = match source {
            "mqtt" => "set ARCLANG_MQTT_USERNAME and ARCLANG_MQTT_PASSWORD",
            "opcua" => "set ARCLANG_OPCUA_USERNAME and ARCLANG_OPCUA_PASSWORD",
            "kafka" => "set ARCLANG_KAFKA_USERNAME and ARCLANG_KAFKA_PASSWORD",
            _ => "set ARCLANG_HTTP_TOKEN (a bearer token)",
        };
        return Err(format!("data: the credentials are not written in the model: {hint}"));
    }
    let mut written: Vec<&String> = data.keys().collect();
    written.sort();
    for key in written {
        if let Some((_, owners)) = SOURCE_KEYS.iter().find(|(k, _)| k == key) {
            if !owners.contains(&source) {
                return Err(format!("data: '{key}' is not for {source} (it is {}'s)", owners.join(" and ")));
            }
        }
    }
    let scheme = url.split_once("://").map(|(s, _)| s.to_string()).unwrap_or_default();
    let topics = || -> Result<Vec<String>, String> {
        let topics = strings(data, "topics").map_err(|e| format!("data: {e}"))?;
        if topics.is_empty() {
            return Err(format!("data: topics, what to read: topics: [\"{}\"]", if source == "kafka" { "house.telemetry" } else { "site/042/#" }));
        }
        Ok(topics)
    };
    match source {
        "mqtt" => {
            if !(scheme == "mqtt" || scheme == "mqtts") {
                return Err(format!("data: mqtt '{url}' is not mqtt://host[:port] or mqtts://host[:port]"));
            }
            let topics = topics()?;
            Ok((url, Source::Mqtt { topics, client_id: text("client_id") }))
        }
        "kafka" => {
            let brokers = kafka_brokers(&url)?;
            if text("sasl").is_some_and(|m| !KAFKA_SASL.contains(&m.as_str())) {
                return Err(format!("data: sasl '{}' is one of {}", text("sasl").unwrap_or_default(), KAFKA_SASL.join(", ")));
            }
            let topics = topics()?;
            Ok((url, Source::Kafka { brokers, tls: scheme == "kafkas", topics, client_id: text("client_id"), sasl: text("sasl").unwrap_or_else(|| "plain".to_string()) }))
        }
        "http" => {
            if !(scheme == "http" || scheme == "https") || url.split_once("://").is_some_and(|(_, rest)| rest.is_empty() || rest.starts_with('/')) {
                return Err(format!("data: http '{url}' is not http://host/path or https://host/path"));
            }
            let sampling = super::seconds(data, "sampling").map_err(|e| format!("data: {e}"))?.unwrap_or(10.0);
            if sampling <= 0.0 {
                return Err("data: sampling is positive (10 s by default for http)".to_string());
            }
            Ok((url, Source::Http { sampling }))
        }
        _ => opcua_of(data, url, &scheme),
    }
}

/// `kafka://host:port[,host:port...]` (or `kafkas://`, over TLS): the brokers to start from.
pub fn kafka_brokers(url: &str) -> Result<Vec<String>, String> {
    let wrong = || format!("data: kafka '{url}' is not kafka://host:port[,host:port] or kafkas://host:port");
    let (scheme, rest) = url.split_once("://").ok_or_else(wrong)?;
    if scheme != "kafka" && scheme != "kafkas" {
        return Err(wrong());
    }
    let brokers: Vec<String> = rest
        .trim_end_matches('/')
        .split(',')
        .map(|broker| {
            let broker = broker.trim();
            match broker.rsplit_once(':') {
                Some((host, port)) if !host.is_empty() && port.parse::<u16>().is_ok() => Ok(broker.to_string()),
                None if !broker.is_empty() && !broker.contains('/') => Ok(format!("{broker}:9092")),
                _ => Err(wrong()),
            }
        })
        .collect::<Result<_, _>>()?;
    Ok(brokers)
}

fn opcua_of(data: &HashMap<String, AttributeValue>, url: String, scheme: &str) -> Result<(String, Source), String> {
    let text = |key: &str| data.get(key).and_then(|v| v.as_string()).map(str::to_string);
    if scheme != "opc.tcp" {
        return Err(format!("data: opcua '{url}' is not opc.tcp://host[:port]"));
    }
    let Some(AttributeValue::Map(written)) = data.get("nodes") else {
        return Err("data: nodes, a column per node: nodes: { indoor_c: \"ns=2;s=Room.T\" }".to_string());
    };
    let mut nodes = Vec::new();
    for (column, node) in written {
        let node = node.as_string().ok_or_else(|| format!("data: nodes: {column} is a node id written as text (\"ns=2;s=Room.T\")"))?;
        if !is_node_id(node) {
            return Err(format!("data: nodes: '{node}' is not an OPC UA node id (ns=2;s=Room.T, ns=2;i=1002, i=2258)"));
        }
        nodes.push((column.clone(), node.to_string()));
    }
    if nodes.is_empty() {
        return Err("data: nodes, a column per node: nodes: { indoor_c: \"ns=2;s=Room.T\" }".to_string());
    }
    nodes.sort();
    let security = match text("security").as_deref() {
        None | Some("none") => Security::None,
        Some("sign") => Security::Sign,
        Some("sign_and_encrypt") => Security::SignAndEncrypt,
        Some(other) => return Err(format!("data: security '{other}' is \"none\", \"sign\" or \"sign_and_encrypt\"")),
    };
    let policy = match (security, text("policy")) {
        (Security::None, None) => "None".to_string(),
        (Security::None, Some(_)) => return Err("data: a policy is for messages signed or encrypted: security: \"sign_and_encrypt\"".to_string()),
        (_, None) => "Basic256Sha256".to_string(),
        (_, Some(policy)) if OPCUA_POLICIES.contains(&policy.as_str()) => policy,
        (_, Some(policy)) => return Err(format!("data: policy '{policy}' is one of {}", OPCUA_POLICIES.join(", "))),
    };
    let sampling = super::seconds(data, "sampling").map_err(|e| format!("data: {e}"))?.unwrap_or(1.0);
    if sampling <= 0.0 {
        return Err("data: sampling is positive (1 s by default)".to_string());
    }
    Ok((url, Source::OpcUa { nodes, security, policy, sampling }))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    /// The measurements: a file, or for a live twin its `record` (empty when it keeps none).
    pub file: String,
    pub live: Option<Live>,
    /// The column of the instants: numbers (in `time_unit`, seconds by default) or ISO 8601 date-times.
    pub time: String,
    pub time_unit: &'static UnitSpec,
    pub delimiter: Option<char>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    pub model: String,
    pub configuration: Option<String>,
    pub sources: Vec<String>,
    pub tolerance: Option<f64>,
    pub data: Data,
    pub drives: Vec<Drive>,
    pub compares: Vec<Compare>,
    pub expects: Vec<String>,
}

fn strings(attributes: &HashMap<String, AttributeValue>, key: &str) -> Result<Vec<String>, String> {
    match attributes.get(key) {
        None => Ok(Vec::new()),
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).ok_or_else(|| format!("{key}: a list of texts"))).collect(),
        Some(AttributeValue::String(one)) => Ok(vec![one.clone()]),
        Some(_) => Err(format!("{key}: a list of texts")),
    }
}

fn block<'a>(attributes: &'a HashMap<String, AttributeValue>, key: &str, keys: &[&str], example: &str) -> Result<&'a HashMap<String, AttributeValue>, String> {
    let Some(AttributeValue::Map(map)) = attributes.get(key) else {
        return Err(format!("{key} is required: {key}: {example}"));
    };
    let mut written: Vec<&String> = map.keys().collect();
    written.sort();
    if let Some(unknown) = written.iter().find(|k| !keys.contains(&k.as_str())) {
        return Err(format!("{key}: '{unknown}' is not one of {}", keys.join(", ")));
    }
    Ok(map)
}

fn is_class_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('.').all(|part| {
            let mut chars = part.chars();
            chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

fn unit_of(symbol: &str, what: &str) -> Result<&'static UnitSpec, String> {
    lookup_unit(symbol).ok_or_else(|| format!("{what}: '{symbol}' is not a unit ArcLang knows"))
}

/// `"PN-VCU".target_speed <- speed_kmh km/h`
pub fn parse_drive(text: &str) -> Result<Drive, String> {
    let (target, rest) = text.split_once("<-").ok_or_else(|| format!("drives: '{text}' is not 'variable <- column [unit]'"))?;
    let mut words = rest.split_whitespace();
    let column = words.next().ok_or_else(|| format!("drives: '{text}' names no column"))?.to_string();
    let unit = words.next().map(|u| unit_of(u, "drives")).transpose()?;
    if words.next().is_some() {
        return Err(format!("drives: '{text}': after the column, only its unit"));
    }
    let target = target.trim().to_string();
    if target.is_empty() {
        return Err(format!("drives: '{text}' names no variable"));
    }
    Ok(Drive { target, column, unit })
}

/// `"PN-BATT".temperature ~ pack_temp °C within 2 K`
pub fn parse_compare(text: &str) -> Result<Compare, String> {
    let (target, rest) = text.split_once('~').ok_or_else(|| format!("compares: '{text}' is not 'variable ~ column [unit] within tolerance [unit]'"))?;
    let (measured, tolerance) = rest.split_once(" within ").ok_or_else(|| format!("compares: '{text}' needs 'within <tolerance>', how close the model must stay"))?;
    let (tolerance, share) = match tolerance.split_once(" for ") {
        None => (tolerance, 1.0),
        Some((tolerance, share)) => {
            let percent: f64 = share.trim().trim_end_matches('%').trim().parse().ok().filter(|p: &f64| *p > 0.0 && *p <= 100.0).ok_or_else(|| format!("compares: '{text}': 'for' gives the share of samples within, 0 to 100 %"))?;
            (tolerance, percent / 100.0)
        }
    };
    let mut words = measured.split_whitespace();
    let column = words.next().ok_or_else(|| format!("compares: '{text}' names no column"))?.to_string();
    let unit = words.next().map(|u| unit_of(u, "compares")).transpose()?;
    if words.next().is_some() {
        return Err(format!("compares: '{text}': after the column, only its unit"));
    }
    let mut parts = tolerance.split_whitespace();
    let amount: f64 = parts.next().and_then(|v| v.parse().ok()).filter(|v: &f64| *v >= 0.0).ok_or_else(|| format!("compares: '{text}': the tolerance is a number"))?;
    let tolerance_unit = parts.next().map(|u| unit_of(u, "compares")).transpose()?;
    if let (Some(a), Some(b)) = (tolerance_unit, unit) {
        if a.dimension != b.dimension {
            return Err(format!("compares: '{text}': a tolerance in {} for a column in {}", a.symbol, b.symbol));
        }
    }
    // A tolerance is a difference: its unit's factor, never its offset.
    let factor = tolerance_unit.or(unit).map_or(1.0, |u| u.factor);
    let target = target.trim().to_string();
    if target.is_empty() {
        return Err(format!("compares: '{text}' names no variable"));
    }
    Ok(Compare { target, column, unit, tolerance: amount * factor, tolerance_text: tolerance.trim().to_string(), share })
}

/// The twin's declaration, read and checked (nothing is read from disk here).
pub fn spec_of(attributes: &HashMap<String, AttributeValue>) -> Result<Spec, String> {
    let mut keys: Vec<&String> = attributes.keys().collect();
    keys.sort();
    if let Some(unknown) = keys.iter().find(|k| !KEYS.contains(&k.as_str())) {
        return Err(format!("'{unknown}' is not one of {}", KEYS.join(", ")));
    }
    let simulate = block(attributes, "simulate", &SIMULATE_KEYS, "{ model: \"Package.Model\" sources: [\"physics/Model.mo\"] }")?;
    let model = simulate.get("model").and_then(|v| v.as_string()).ok_or("simulate: model is required (the Modelica class the data drives)")?.to_string();
    if !is_class_name(&model) {
        return Err(format!("simulate: model '{model}' is not a Modelica class name (Package.Model)"));
    }
    let sources = match simulate.get("sources") {
        None => Vec::new(),
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).ok_or("simulate: sources are file paths")).collect::<Result<_, _>>()?,
        Some(AttributeValue::String(one)) => vec![one.clone()],
        Some(_) => return Err("simulate: sources are file paths".to_string()),
    };
    let tolerance = match simulate.get("tolerance") {
        None => None,
        Some(v) => Some(v.as_number().filter(|t| *t > 0.0).ok_or("simulate: tolerance is a positive number")?),
    };
    let configuration = simulate.get("configuration").and_then(|v| v.as_string()).map(str::to_string);

    let data = block(attributes, "data", &DATA_KEYS, "{ file: \"data/measured.csv\" time: \"time\" } or { mqtt: \"mqtt://broker:1883\" topics: [\"site/#\"] } (or opcua:, kafka:, http:)")?;
    let text = |key: &str| data.get(key).and_then(|v| v.as_string()).map(str::to_string);
    let inside = |file: &str| -> Result<(), String> {
        if Path::new(file).is_absolute() || file.split(['/', '\\']).any(|p| p == "..") {
            return Err(format!("data: '{file}' must be a path inside the model's folder"));
        }
        Ok(())
    };
    let given: Vec<&str> = std::iter::once("file").chain(LIVE_SOURCES).filter(|k| data.contains_key(*k)).collect();
    let live = match given.as_slice() {
        [] => return Err("data: file (the measurements, CSV or Parquet) or live data (mqtt, opcua, kafka or http) is required".to_string()),
        ["file"] => {
            if let Some(key) = LIVE_KEYS.iter().find(|k| data.contains_key(**k)) {
                return Err(format!("data: '{key}' is for live data (mqtt, opcua, kafka or http), not for a file"));
            }
            None
        }
        [source] => {
            let url = text(source).ok_or_else(|| format!("data: {source} is a URL written as text"))?;
            let (url, source) = live_of(data, url, source)?;
            let window = super::seconds(data, "window").map_err(|e| format!("data: {e}"))?.unwrap_or(3600.0);
            let refresh = super::seconds(data, "refresh").map_err(|e| format!("data: {e}"))?.unwrap_or(60.0);
            if window <= 0.0 || refresh < 1.0 {
                return Err("data: window is positive and refresh at least 1 s".to_string());
            }
            let record = text("record");
            if let Some(record) = &record {
                inside(record)?;
                if !record.to_ascii_lowercase().ends_with(".csv") {
                    return Err(format!("data: record '{record}' is a CSV file (.csv)"));
                }
            }
            let step = match text("advance").as_deref() {
                None | Some("window") => false,
                Some("step") => true,
                Some(other) => return Err(format!("data: advance '{other}' is \"window\" (the model runs again over the window) or \"step\" (it advances with the data)")),
            };
            Some(Live { url, source, window, refresh, record, step })
        }
        _ => return Err(format!("data: one source, not {}", given.join(" and "))),
    };
    let file = match &live {
        Some(live) => live.record.clone().unwrap_or_default(),
        None => text("file").unwrap_or_default(),
    };
    if live.is_none() {
        let lower = file.to_ascii_lowercase();
        if !(lower.ends_with(".csv") || lower.ends_with(".tsv") || lower.ends_with(".txt") || lower.ends_with(".parquet")) {
            return Err(format!("data: '{file}' is neither a CSV file (.csv, .tsv) nor a Parquet file (.parquet)"));
        }
        inside(&file)?;
    }
    let time = data.get("time").and_then(|v| v.as_string()).unwrap_or("time").to_string();
    let time_unit = unit_of(data.get("time_unit").and_then(|v| v.as_string()).unwrap_or("s"), "data: time_unit")?;
    if time_unit.dimension != crate::compiler::quantity::Dimension::Time {
        return Err(format!("data: time_unit '{}' is not a unit of time", time_unit.symbol));
    }
    let delimiter = match data.get("delimiter").and_then(|v| v.as_string()) {
        None => None,
        Some(d) if d.chars().count() == 1 => d.chars().next(),
        Some("tab") => Some('\t'),
        Some(other) => return Err(format!("data: delimiter '{other}' is one character (or \"tab\")")),
    };

    let drives = strings(attributes, "drives")?.iter().map(|d| parse_drive(d)).collect::<Result<Vec<_>, _>>()?;
    let compares = strings(attributes, "compares")?.iter().map(|c| parse_compare(c)).collect::<Result<Vec<_>, _>>()?;
    let expects = strings(attributes, "expects")?;
    if drives.is_empty() {
        return Err("drives: what the measurements impose on the model, [\"variable <- column [unit]\", ...]".to_string());
    }
    if compares.is_empty() && expects.is_empty() {
        return Err("compares: [\"variable ~ column within tolerance\"] or expects: [...]: what the twin checks".to_string());
    }
    for e in &expects {
        expects::check_syntax(e)?;
    }
    let mut seen = std::collections::HashSet::new();
    if let Some(twice) = drives.iter().find(|d| !seen.insert(d.target.clone())) {
        return Err(format!("drives: '{}' is driven twice", twice.target));
    }
    Ok(Spec { model, configuration, sources, tolerance, data: Data { file, live, time, time_unit, delimiter }, drives, compares, expects })
}

pub fn validate(attributes: &HashMap<String, AttributeValue>) -> Result<(), String> {
    spec_of(attributes).map(|_| ())
}

/// The compile-time check: the requirements and the configuration it names exist.
pub fn check(model: &crate::compiler::ast::Model) -> Vec<String> {
    let mut findings = Vec::new();
    for twin in &model.twins {
        let Ok(spec) = spec_of(&twin.attributes) else { continue };
        if let Some(configuration) = &spec.configuration {
            if !model.configurations.iter().any(|c| &c.id == configuration || &c.name == configuration) {
                findings.push(format!("twin '{}' simulates configuration '{configuration}', which is none", twin.name));
            }
        }
        for requirement in &twin.verifies {
            // A reference that names no requirement comes back unchanged and unknown.
            let key = crate::compiler::verification::requirement_key(model, requirement);
            let known = model.system_analysis.iter().flat_map(|sa| sa.requirements.iter()).any(|r| r.id == key || r.attributes.get("id").and_then(|v| v.as_string()) == Some(key.as_str()));
            if !known {
                findings.push(format!("twin '{}' verifies '{requirement}', which is not a requirement", twin.name));
            }
        }
    }
    findings
}

// ---- The measurements ----------------------------------------------------------

/// The columns of the measurement file, the instants in seconds from the first.
#[derive(Debug, Clone)]
pub struct Measurements {
    pub times: Vec<f64>,
    /// By column: one value per instant, `None` where the cell is empty.
    pub columns: BTreeMap<String, Vec<Option<f64>>>,
    /// The first instant as written (a date-time, or a number).
    pub start: String,
}

fn split(line: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    for c in line.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c == delimiter && !quoted => out.push(std::mem::take(&mut field)),
            c => field.push(c),
        }
    }
    out.push(field);
    out.into_iter().map(|f| f.trim().to_string()).collect()
}

/// An instant: a number in `unit`, or an ISO 8601 date-time (seconds since 1970).
fn instant(text: &str, unit: &UnitSpec) -> Option<f64> {
    if let Ok(value) = text.parse::<f64>() {
        return Some(unit.to_canonical(value));
    }
    if let Ok(t) = chrono::DateTime::parse_from_rfc3339(text) {
        return Some(t.timestamp_millis() as f64 / 1000.0);
    }
    for format in ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M", "%Y-%m-%dT%H:%M"] {
        if let Ok(t) = chrono::NaiveDateTime::parse_from_str(text, format) {
            return Some(t.and_utc().timestamp_millis() as f64 / 1000.0);
        }
    }
    None
}

pub fn read_measurements(text: &str, data: &Data) -> Result<Measurements, String> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'));
    let header = lines.next().ok_or("the data file is empty")?;
    let delimiter = data.delimiter.unwrap_or_else(|| if header.contains('\t') { '\t' } else if header.matches(';').count() > header.matches(',').count() { ';' } else { ',' });
    let names = split(header, delimiter);
    let time_column = names.iter().position(|n| n == &data.time).ok_or_else(|| format!("the data has no time column '{}' (its columns: {})", data.time, names.join(", ")))?;
    let mut times = Vec::new();
    let mut cells: Vec<Vec<Option<f64>>> = vec![Vec::new(); names.len()];
    let mut start = String::new();
    for (number, line) in lines.enumerate() {
        let fields = split(line, delimiter);
        let Some(raw) = fields.get(time_column) else { continue };
        let t = instant(raw, data.time_unit).ok_or_else(|| format!("line {}: '{raw}' is not an instant (a number, or a date-time like 2026-10-28T08:15:00Z)", number + 2))?;
        if start.is_empty() {
            start = raw.clone();
        }
        times.push(t);
        for (i, column) in cells.iter_mut().enumerate() {
            column.push(fields.get(i).and_then(|v| v.replace(',', ".").parse::<f64>().ok()).filter(|v| v.is_finite()));
        }
    }
    if times.len() < 2 {
        return Err("the data has fewer than two samples".to_string());
    }
    if times.windows(2).any(|w| w[1] < w[0]) {
        return Err("the instants of the data go back in time: sort the file by its time column".to_string());
    }
    let first = times[0];
    let times: Vec<f64> = times.iter().map(|t| t - first).collect();
    let columns = names.into_iter().zip(cells).enumerate().filter(|(i, _)| *i != time_column).map(|(_, (n, c))| (n, c)).collect();
    Ok(Measurements { times, columns, start })
}

/// A window as people read it: 45 s, 15 min, 6 h, 2 d.
pub fn window_text(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0} ms", seconds * 1000.0)
    } else if seconds < 120.0 {
        format!("{seconds:.0} s")
    } else if seconds < 7200.0 {
        format!("{:.0} min", seconds / 60.0)
    } else if seconds < 172_800.0 {
        format!("{} h", ((seconds / 3600.0) * 10.0).round() / 10.0)
    } else {
        format!("{} d", ((seconds / 86400.0) * 10.0).round() / 10.0)
    }
}

/// The measurements of a file: Parquet by its extension, CSV otherwise.
pub fn load_measurements(path: &Path, data: &Data) -> Result<Measurements, String> {
    if data.file.to_ascii_lowercase().ends_with(".parquet") {
        return read_parquet(path, data);
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot be read: {e}"))?;
    read_measurements(&text, data)
}

/// A Parquet file of measurements: a row per instant, the time column a
/// timestamp, a number (in `time_unit`) or an ISO 8601 text; the other
/// columns numbers (null where there is no value).
#[cfg(feature = "native")]
pub fn read_parquet(path: &Path, data: &Data) -> Result<Measurements, String> {
    use parquet::file::reader::{FileReader, SerializedFileReader};
    use parquet::record::Field;
    let file = std::fs::File::open(path).map_err(|e| format!("cannot be read: {e}"))?;
    let reader = SerializedFileReader::new(file).map_err(|e| format!("is not a Parquet file: {e}"))?;
    let schema = reader.metadata().file_metadata().schema_descr();
    let names: Vec<String> = schema.columns().iter().map(|c| c.name().to_string()).collect();
    if !names.iter().any(|n| n == &data.time) {
        return Err(format!("the data has no time column '{}' (its columns: {})", data.time, names.join(", ")));
    }
    // A timestamp in nanoseconds (pandas' default) reaches the rows as a bare integer: its unit is in the schema.
    let nanoseconds = schema.columns().iter().find(|c| c.name() == data.time).and_then(|c| c.logical_type_ref()).is_some_and(|t| {
        matches!(t, parquet::basic::LogicalType::Timestamp(t) if matches!(t.unit, parquet::basic::TimeUnit::NANOS))
    });
    let number = |field: &Field| -> Option<f64> {
        match field {
            Field::Double(v) => Some(*v),
            Field::Float(v) => Some(f64::from(*v)),
            Field::Long(v) => Some(*v as f64),
            Field::Int(v) => Some(f64::from(*v)),
            Field::Short(v) => Some(f64::from(*v)),
            Field::Byte(v) => Some(f64::from(*v)),
            Field::ULong(v) => Some(*v as f64),
            Field::UInt(v) => Some(f64::from(*v)),
            Field::UShort(v) => Some(f64::from(*v)),
            Field::UByte(v) => Some(f64::from(*v)),
            Field::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
        .filter(|v| v.is_finite())
    };
    let mut times = Vec::new();
    let mut columns: BTreeMap<String, Vec<Option<f64>>> = names.iter().filter(|n| *n != &data.time).map(|n| (n.clone(), Vec::new())).collect();
    let mut start = String::new();
    let rows = reader.get_row_iter(None).map_err(|e| format!("cannot be read: {e}"))?;
    for (number_of_row, row) in rows.enumerate() {
        let row = row.map_err(|e| format!("row {}: {e}", number_of_row + 1))?;
        let mut time = None;
        let mut values: BTreeMap<&str, Option<f64>> = BTreeMap::new();
        for (name, field) in row.get_column_iter() {
            if name == &data.time {
                let (seconds, shown) = match field {
                    Field::TimestampMillis(ms) => (Some(*ms as f64 / 1e3), ms.to_string()),
                    Field::TimestampMicros(us) => (Some(*us as f64 / 1e6), us.to_string()),
                    Field::Long(ns) if nanoseconds => (Some(*ns as f64 / 1e9), ns.to_string()),
                    Field::Str(text) => (instant(text, data.time_unit), text.clone()),
                    other => (number(other).map(|v| data.time_unit.to_canonical(v)), other.to_string()),
                };
                let seconds = seconds.ok_or_else(|| format!("row {}: the time '{shown}' is not an instant", number_of_row + 1))?;
                if start.is_empty() {
                    start = match field {
                        Field::TimestampMillis(_) | Field::TimestampMicros(_) => chrono::DateTime::from_timestamp_millis((seconds * 1e3) as i64).map(|t| t.to_rfc3339()).unwrap_or(shown),
                        Field::Long(_) if nanoseconds => chrono::DateTime::from_timestamp_millis((seconds * 1e3) as i64).map(|t| t.to_rfc3339()).unwrap_or(shown),
                        _ => shown,
                    };
                }
                time = Some(seconds);
            } else {
                values.insert(name.as_str(), number(field));
            }
        }
        let Some(time) = time else { continue };
        times.push(time);
        for (name, column) in columns.iter_mut() {
            column.push(values.get(name.as_str()).copied().flatten());
        }
    }
    if times.len() < 2 {
        return Err("the data has fewer than two samples".to_string());
    }
    if times.windows(2).any(|w| w[1] < w[0]) {
        return Err("the instants of the data go back in time: sort the file by its time column".to_string());
    }
    let first = times[0];
    Ok(Measurements { times: times.iter().map(|t| t - first).collect(), columns, start })
}

#[cfg(not(feature = "native"))]
pub fn read_parquet(_path: &Path, _data: &Data) -> Result<Measurements, String> {
    Err("Parquet files are read by the arclang binary, not by this build".to_string())
}

/// A measured column in SI at each instant, empty cells left out.
fn channel(m: &Measurements, column: &str, unit: Option<&UnitSpec>) -> Result<Vec<(f64, f64)>, String> {
    let values = m.columns.get(column).ok_or_else(|| format!("the data has no column '{column}' (its columns: {})", m.columns.keys().cloned().collect::<Vec<_>>().join(", ")))?;
    let points: Vec<(f64, f64)> = m.times.iter().zip(values).filter_map(|(t, v)| v.map(|v| (*t, unit.map_or(v, |u| u.to_canonical(v))))).collect();
    if points.is_empty() {
        return Err(format!("the column '{column}' has no value"));
    }
    Ok(points)
}

fn interpolate(points: &[(f64, f64)], t: f64) -> f64 {
    match points.iter().position(|(time, _)| *time >= t) {
        None => points[points.len() - 1].1,
        Some(0) => points[0].1,
        Some(i) => {
            let (t0, v0) = points[i - 1];
            let (t1, v1) = points[i];
            if t1 > t0 { v0 + (v1 - v0) * (t - t0) / (t1 - t0) } else { v1 }
        }
    }
}

/// The inputs of the run, as OpenModelica's `-csvInput` reads them: the time, then one column per drive.
pub fn input_csv(m: &Measurements, drives: &[Drive]) -> Result<String, String> {
    let channels: Vec<Vec<(f64, f64)>> = drives.iter().map(|d| channel(m, &d.column, d.unit)).collect::<Result<_, _>>()?;
    let mut out = String::from("time");
    for i in 0..drives.len() {
        out.push_str(&format!(",twin_in_{i}"));
    }
    out.push('\n');
    let mut previous = f64::NEG_INFINITY;
    for &t in &m.times {
        // An instant written twice keeps its first row: the inputs are a function of time.
        if t <= previous {
            continue;
        }
        previous = t;
        out.push_str(&modelica::names::real(t));
        for c in &channels {
            out.push(',');
            out.push_str(&format!("{}", interpolate(c, t)));
        }
        out.push('\n');
    }
    Ok(out)
}

// ---- The run ---------------------------------------------------------------------

/// A variable as the twin names it (`"PN-BATT".soc`) as a path of the model (`batteryPack.soc`).
pub fn resolve(target: &str, aliases: &BTreeMap<String, String>) -> String {
    let target = target.trim();
    if let Some(rest) = target.strip_prefix('"') {
        if let Some((id, tail)) = rest.split_once('"') {
            if let Some(path) = aliases.get(id) {
                return format!("{path}{tail}");
            }
        }
    }
    target.to_string()
}

fn modelica_path(path: &str) -> bool {
    !path.is_empty() && path.split('.').all(|p| is_class_name(p) && !p.contains('.'))
}

/// The wrapper the run simulates: the model, each driven variable bound to an input of its own.
pub fn wrapper(model: &str, targets: &[String]) -> Result<String, String> {
    if let Some(bad) = targets.iter().find(|t| !modelica_path(t)) {
        return Err(format!("drives: '{bad}' is not a variable of the model (a.b.c)"));
    }
    let bindings: Vec<String> = targets.iter().enumerate().map(|(i, t)| format!("{t} = twin_in_{i}")).collect();
    let inputs: String = (0..targets.len()).map(|i| format!("  input Real twin_in_{i};\n")).collect();
    Ok(format!("model {WRAPPER} \"The twin: {model} driven by the measurements\"\n  extends {model}({});\n{inputs}end {WRAPPER};\n", bindings.join(", ")))
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Curve {
    pub time: Vec<f64>,
    pub measured: Vec<f64>,
    pub simulated: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Comparison {
    pub target: String,
    pub variable: String,
    pub column: String,
    pub tolerance: String,
    /// SI, for the view.
    pub tolerance_si: f64,
    pub unit: String,
    /// The column's unit from SI, for the view: `value = (si - offset) / factor`.
    pub unit_factor: f64,
    pub unit_offset: f64,
    pub samples: usize,
    pub max_deviation: f64,
    pub rms_deviation: f64,
    /// The share of the samples within the tolerance, 0 to 1.
    pub within: f64,
    /// The share required (`for 99 %`), 0 to 1.
    pub required: f64,
    /// The first instant (seconds from the start of the data) beyond the tolerance.
    pub first_exceedance: Option<f64>,
    pub holds: bool,
    pub curve: Curve,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Outcome {
    pub twin: String,
    pub name: String,
    pub verifies: Vec<String>,
    pub verdict: String,
    pub reason: String,
    pub data: String,
    pub data_start: String,
    pub samples: usize,
    pub duration: f64,
    pub comparisons: Vec<Comparison>,
    pub values: Vec<expects::Evaluated>,
    pub tool: String,
    pub omc_version: String,
    pub seconds: f64,
    pub log: String,
}

/// The hash of what a twin's verdict depends on: the generated package,
/// every source, the measurements (path and content), and the twin as
/// written (its model, configuration, data, drives, compares, expects).
pub fn cache_key(result: &CompilationResult, model_path: &Path, twin: &Twin) -> Result<String, String> {
    let spec = spec_of(&twin.attributes)?;
    let export = modelica::export(&result.ast)?;
    let base = super::cache::model_dir(model_path);
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("twin", b"2");
    field("package", export.text.as_bytes());
    for source in &spec.sources {
        field("source", source.as_bytes());
        match std::fs::read(base.join(source)) {
            Ok(content) => field("content", &content),
            Err(_) => field("missing", b""),
        }
    }
    field("data", spec.data.file.as_bytes());
    match std::fs::read(base.join(&spec.data.file)) {
        Ok(content) => field("content", &content),
        Err(_) => field("missing", b""),
    }
    field("model", spec.model.as_bytes());
    field("configuration", spec.configuration.as_deref().unwrap_or("").as_bytes());
    field("tolerance", format!("{:?}", spec.tolerance).as_bytes());
    let mut written: Vec<(&String, String)> = twin.attributes.iter().map(|(k, v)| (k, v.display())).collect();
    written.sort();
    for (key, value) in written {
        field(key, value.as_bytes());
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Keep a twin's verdict in the model's cache (`<twin>.twin.json`), where
/// the compiler reads it as the twin rung of the requirements it verifies.
pub fn cache(result: &CompilationResult, model_path: &Path, twin: &Twin, outcome: &Outcome) -> Result<PathBuf, String> {
    let key = cache_key(result, model_path, twin)?;
    let spec = spec_of(&twin.attributes)?;
    let comparisons: Vec<String> = outcome
        .comparisons
        .iter()
        .map(|c| format!("{} ~ {}: max deviation {:.4} {}, RMS {:.4} {}, {:.1} % within {} ({:.1} % required)", c.target, c.column, c.max_deviation, c.unit, c.rms_deviation, c.unit, c.within * 100.0, c.tolerance, c.required * 100.0))
        .collect();
    let cached = super::Outcome {
        test_case: twin.id.clone(),
        kind: "twin".to_string(),
        name: twin.name.clone(),
        verifies: twin.verifies.clone(),
        verdict: outcome.verdict.clone(),
        reason: outcome.reason.clone(),
        values: outcome.values.clone(),
        key,
        model: spec.model.clone(),
        configuration: spec.configuration.clone(),
        tool: outcome.tool.clone(),
        target: outcome.data.clone(),
        omc_version: outcome.omc_version.clone(),
        log: comparisons.join("\n"),
        seconds: outcome.seconds,
        ..Default::default()
    };
    super::cache::write_kind(&super::cache::dir(model_path), "twin", &cached)
}

pub(crate) fn work_dir() -> Result<PathBuf, String> {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("arclang-twin-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(dir.join("sources")).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir)
}

/// The lines of a script that load the library, the model, the physics and the wrapper.
pub fn load_lines(export: &Export, staged: &[String]) -> Vec<String> {
    let msl = modelica::MSL_VERSION;
    let mut lines = vec![
        "print(\"@@version \" + getVersion() + \"\\n\");".to_string(),
        format!("if not loadModel(Modelica, {{\"{msl}\"}}) then installPackage(Modelica, \"{msl}+maint.om\", exactMatch=true); end if;"),
        format!("print(\"@@msl \" + String(loadModel(Modelica, {{\"{msl}\"}})) + \"\\n\");"),
        "print(getErrorString());".to_string(),
    ];
    for file in std::iter::once(&export.file_name).chain(staged).chain(std::iter::once(&"twin.mo".to_string())) {
        lines.push(format!("print(\"@@load {file} \" + String(loadFile(\"{file}\")) + \"\\n\");"));
        lines.push("print(getErrorString());".to_string());
    }
    lines
}

fn script(export: &Export, staged: &[String], spec: &Spec, stop: f64, intervals: usize) -> String {
    let mut lines = load_lines(export, staged);
    lines.push(format!(
        "simulate({WRAPPER}, startTime=0, stopTime={}, numberOfIntervals={intervals}, tolerance={}, outputFormat=\"csv\", fileNamePrefix=\"{RESULT}\", simflags=\"-csvInput=twin_inputs.csv\");",
        modelica::names::real(stop),
        modelica::names::real(spec.tolerance.unwrap_or(1e-6))
    ));
    lines.push("print(getErrorString());".to_string());
    lines.join("\n") + "\n"
}

fn downsample(points: Vec<(f64, f64, f64)>) -> Curve {
    let step = (points.len() / PLOT_POINTS).max(1);
    let mut curve = Curve::default();
    for (i, (t, m, s)) in points.iter().enumerate() {
        if i % step == 0 || i + 1 == points.len() {
            curve.time.push(*t);
            curve.measured.push(*m);
            curve.simulated.push(*s);
        }
    }
    curve
}

/// The deviation of the model from a measured column, at every measured instant.
pub fn compare(series: &Series, column: usize, measured: &[(f64, f64)], tolerance: f64) -> (f64, f64, f64, Option<f64>, Curve) {
    let mut max: f64 = 0.0;
    let mut squares = 0.0;
    let mut within = 0usize;
    let mut first = None;
    let mut points = Vec::new();
    for &(t, m) in measured {
        let Some(s) = series.at(column, t) else { continue };
        let deviation = (s - m).abs();
        max = max.max(deviation);
        squares += deviation * deviation;
        if deviation <= tolerance + 1e-12 * tolerance.abs().max(1.0) {
            within += 1;
        } else if first.is_none() {
            first = Some(t);
        }
        points.push((t, m, s));
    }
    let n = points.len().max(1) as f64;
    (max, (squares / n).sqrt(), within as f64 / n, first, downsample(points))
}

pub fn shown(value: f64, unit: Option<&UnitSpec>) -> String {
    match unit {
        // A deviation in its column's unit: the factor, never the offset.
        Some(u) => format!("{:.4} {}", value / u.factor, u.symbol),
        None => format!("{value:.4}"),
    }
}

/// A working directory with the model's Modelica, the wrapper and the physics: the files to load.
pub fn stage(export: &Export, model: &str, sources: &[String], targets: &[String], base: &Path) -> Result<(PathBuf, Vec<String>), String> {
    let wrapper = wrapper(model, targets)?;
    let dir = work_dir()?;
    let written = (|| -> Result<Vec<String>, String> {
        std::fs::write(dir.join(&export.file_name), &export.text).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("twin.mo"), wrapper).map_err(|e| e.to_string())?;
        let mut staged = Vec::new();
        for (index, source) in sources.iter().enumerate() {
            let content = std::fs::read(base.join(source)).map_err(|e| format!("source '{source}' cannot be read: {e}"))?;
            let stem: String = Path::new(source).file_stem().and_then(|s| s.to_str()).unwrap_or("source").chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
            let name = format!("sources/{index}_{stem}.mo");
            std::fs::write(dir.join(&name), content).map_err(|e| e.to_string())?;
            staged.push(name);
        }
        Ok(staged)
    })();
    match written {
        Ok(staged) => Ok((dir, staged)),
        Err(error) => {
            let _ = std::fs::remove_dir_all(&dir);
            Err(error)
        }
    }
}

/// Why OpenModelica could not run the wrapper, from what it printed.
pub fn failure(output: &str, what: &str) -> String {
    if let Some(failed) = output.lines().filter_map(|l| l.strip_prefix("@@load ")).find_map(|l| l.strip_suffix(" false")) {
        return format!("{failed} does not load in OpenModelica (see the log)");
    }
    // A measurement can only drive a variable: OpenModelica refuses to bind a parameter to an input.
    if output.contains("of variability parameter has binding") {
        let name = output.lines().find(|l| l.contains("of variability parameter")).and_then(|l| l.split("Component ").nth(1)).and_then(|l| l.split_whitespace().next()).unwrap_or("?");
        return format!("drives: '{name}' is a parameter of the physics: a measurement can only drive a variable; declare it as a variable (or an input) in the Modelica source");
    }
    let errors: Vec<&str> = output.lines().filter(|l| l.contains("Error")).take(4).collect();
    format!("the twin could not be {what}{}", if errors.is_empty() { " (see the log)".to_string() } else { format!(": {}", errors.join(" ")) })
}

/// The variables of the model the twin reads: what it is compared on, what its expectations read.
pub fn read_variables(spec: &Spec, instances: &BTreeMap<String, String>) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = spec.compares.iter().map(|c| resolve(&c.target, instances)).collect();
    for text in &spec.expects {
        let expect = expects::parse(text, instances).map_err(|e| format!("expects '{text}': {e}"))?;
        for variable in expects::variables(&expect) {
            if !out.contains(&variable) {
                out.push(variable);
            }
        }
    }
    Ok(out)
}

/// Runs a twin: the data read, the wrapper simulated, the deviations and the expectations.
pub fn run(result: &CompilationResult, model_path: &Path, twin: &Twin, config: &ToolConfig) -> Outcome {
    let started = std::time::Instant::now();
    let mut outcome = Outcome { twin: twin.id.clone(), name: twin.name.clone(), verifies: twin.verifies.clone(), verdict: "inconclusive".to_string(), ..Outcome::default() };
    let finish = |mut outcome: Outcome, reason: String| {
        outcome.reason = reason;
        outcome.seconds = started.elapsed().as_secs_f64();
        outcome
    };
    let spec = match spec_of(&twin.attributes) {
        Ok(spec) => spec,
        Err(error) => return finish(Outcome { verdict: "error".to_string(), ..outcome }, error),
    };
    let base = super::cache::model_dir(model_path);
    outcome.data = spec.data.file.clone();
    // A live twin replays what it recorded; arclang twin --live follows the broker.
    if let Some(live) = &spec.data.live {
        if spec.data.file.is_empty() || !base.join(&spec.data.file).exists() {
            let why = if spec.data.file.is_empty() { "it records nothing (record: \"data/live.csv\")".to_string() } else { format!("{} is not recorded yet", spec.data.file) };
            return finish(outcome, format!("this twin reads live data from {}: run arclang twin --live; {why}", live.url));
        }
    }
    let measurements = match load_measurements(&base.join(&spec.data.file), &spec.data) {
        Ok(m) => m,
        Err(error) => return finish(Outcome { verdict: "error".to_string(), ..outcome }, format!("{}: {error}", spec.data.file)),
    };
    let label = spec.data.file.clone();
    run_on(result, model_path, twin, &spec, measurements, &label, config, started)
}

/// A twin on measurements already read (a file, or the window of a live stream).
#[allow(clippy::too_many_arguments)]
pub fn run_on(result: &CompilationResult, model_path: &Path, twin: &Twin, spec: &Spec, measurements: Measurements, label: &str, config: &ToolConfig, started: std::time::Instant) -> Outcome {
    let mut outcome = Outcome { twin: twin.id.clone(), name: twin.name.clone(), verifies: twin.verifies.clone(), verdict: "inconclusive".to_string(), data: label.to_string(), ..Outcome::default() };
    let finish = |mut outcome: Outcome, reason: String| {
        outcome.reason = reason;
        outcome.seconds = started.elapsed().as_secs_f64();
        outcome
    };
    let base = super::cache::model_dir(model_path);
    outcome.samples = measurements.times.len();
    outcome.duration = *measurements.times.last().unwrap_or(&0.0);
    outcome.data_start = measurements.start.clone();
    let export = match modelica::export(&result.ast) {
        Ok(export) => export,
        Err(error) => return finish(Outcome { verdict: "error".to_string(), ..outcome }, format!("the model does not export to Modelica: {error}")),
    };
    let prepared = (|| -> Result<(PathBuf, Vec<String>), String> {
        let inputs = input_csv(&measurements, &spec.drives)?;
        let targets: Vec<String> = spec.drives.iter().map(|d| resolve(&d.target, &export.instances)).collect();
        let (dir, staged) = stage(&export, &spec.model, &spec.sources, &targets, &base)?;
        std::fs::write(dir.join("twin_inputs.csv"), inputs).map_err(|e| e.to_string())?;
        Ok((dir, staged))
    })();
    let (dir, staged) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => return finish(Outcome { verdict: "error".to_string(), ..outcome }, error),
    };
    let tool = match tool::locate(config) {
        Ok(tool) => tool,
        Err(reason) => {
            let _ = std::fs::remove_dir_all(&dir);
            return finish(outcome, reason);
        }
    };
    outcome.tool = tool.describe();
    let intervals = (measurements.times.len() * 2).clamp(100, 20_000);
    let _ = std::fs::write(dir.join("twin.mos"), script(&export, &staged, spec, outcome.duration.max(1e-6), intervals));
    let run = tool::run(&tool, &dir, "twin.mos", config.timeout);
    let csv = std::fs::read_to_string(dir.join(format!("{RESULT}_res.csv"))).ok().filter(|c| !c.trim().is_empty());
    let xml = std::fs::read_to_string(dir.join(format!("{RESULT}_init.xml"))).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    let run = match run {
        Ok(run) => run,
        Err(error) => return finish(outcome, error),
    };
    outcome.omc_version = run.output.lines().find_map(|l| l.strip_prefix("@@version ")).unwrap_or("").trim().to_string();
    outcome.log = run.output.chars().rev().take(12_000).collect::<Vec<_>>().into_iter().rev().collect();
    let loaded = !run.output.lines().any(|l| l.starts_with("@@load ") && l.ends_with(" false"));
    let Some(csv) = csv.filter(|_| loaded) else {
        return finish(Outcome { verdict: "error".to_string(), ..outcome }, failure(&run.output, "simulated"));
    };
    let mut series = match Series::from_csv(&csv) {
        Ok(series) => series,
        Err(error) => return finish(Outcome { verdict: "error".to_string(), ..outcome }, error),
    };
    series.set_units(units_from_init_xml(&xml));

    let judged = judge(outcome, spec, &export.instances, &series, &measurements);
    let reason = judged.reason.clone();
    finish(judged, reason)
}

/// The verdict of a run of the twin: each comparison over the measurements, each expectation
/// over the run; the reason says what fails, or what is wrong.
pub fn judge(mut outcome: Outcome, spec: &Spec, instances: &BTreeMap<String, String>, series: &Series, measurements: &Measurements) -> Outcome {
    let finish = |mut outcome: Outcome, reason: String| {
        outcome.reason = reason;
        outcome
    };
    let mut problems = Vec::new();
    for c in &spec.compares {
        let variable = resolve(&c.target, instances);
        let comparison = (|| -> Result<Comparison, String> {
            let column = series.column(&variable).ok_or_else(|| format!("compares: '{}' is not a variable of the run ({variable})", c.target))?;
            let measured = channel(measurements, &c.column, c.unit)?;
            let (max, rms, within, first, curve) = compare(series, column, &measured, c.tolerance);
            Ok(Comparison {
                target: c.target.clone(),
                variable: variable.clone(),
                column: c.column.clone(),
                tolerance: c.tolerance_text.clone(),
                tolerance_si: c.tolerance,
                unit: c.unit.map(|u| u.symbol.to_string()).unwrap_or_default(),
                unit_factor: c.unit.map_or(1.0, |u| u.factor),
                unit_offset: c.unit.map_or(0.0, |u| u.offset),
                samples: measured.len(),
                max_deviation: max,
                rms_deviation: rms,
                within,
                required: c.share,
                first_exceedance: first,
                holds: within + 1e-12 >= c.share,
                curve,
            })
        })();
        match comparison {
            Ok(comparison) => {
                if !comparison.holds {
                    problems.push(format!(
                        "{} departs from {} by {} at most (tolerance {}), first at {:.0} s; {:.1} % of the samples within, {:.1} % required",
                        c.target,
                        c.column,
                        shown(comparison.max_deviation, c.unit),
                        c.tolerance_text,
                        comparison.first_exceedance.unwrap_or(0.0),
                        comparison.within * 100.0,
                        c.share * 100.0
                    ));
                }
                outcome.comparisons.push(comparison);
            }
            Err(error) => return finish(Outcome { verdict: "error".to_string(), ..outcome }, error),
        }
    }
    let mut notes = Vec::new();
    for text in &spec.expects {
        let evaluated = expects::parse(text, instances).and_then(|e| expects::evaluate(&e, &mut Context { series, notes: &mut notes }));
        match evaluated {
            Ok(value) => {
                if !value.holds {
                    problems.push(format!("{}: {}", value.expect, value.value));
                }
                outcome.values.push(value);
            }
            Err(error) => return finish(Outcome { verdict: "error".to_string(), ..outcome }, format!("expects '{text}': {error}")),
        }
    }
    outcome.verdict = if problems.is_empty() { "pass" } else { "fail" }.to_string();
    let reason = problems.join("; ");
    finish(outcome, reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attributes(text: &str) -> HashMap<String, AttributeValue> {
        let source = format!("model T {{ name: \"t\" }}\ntwin \"W\" {}\n", text);
        let ast = crate::compiler::parser::Parser::new(crate::compiler::lexer::Lexer::new(&source).tokenize().unwrap()).parse().unwrap();
        ast.twins[0].attributes.clone()
    }

    const TWIN: &str = r#"{
        verifies: ["REQ-1"]
        simulate: { model: "P.Room" sources: ["physics/P.mo"] }
        data: { file: "data/room.csv" time: "timestamp" }
        drives: ["heater.power <- heater_kw kW", "\"PN-AMB\".T <- outside °C"]
        compares: ["T ~ room °C within 0.5 K"]
        expects: ["max(T) <= 30 °C"]
    }"#;

    #[test]
    fn a_twin_reads_its_drives_comparisons_and_data() {
        let spec = spec_of(&attributes(TWIN)).expect("valid");
        assert_eq!(spec.model, "P.Room");
        assert_eq!(spec.data.time, "timestamp");
        assert_eq!(spec.drives[0], Drive { target: "heater.power".into(), column: "heater_kw".into(), unit: lookup_unit("kW") });
        assert_eq!(spec.drives[1].target, "\"PN-AMB\".T");
        let c = &spec.compares[0];
        assert_eq!((c.target.as_str(), c.column.as_str(), c.tolerance), ("T", "room", 0.5));
        assert_eq!(c.unit.map(|u| u.symbol), Some("°C"));
    }

    /// The parser refuses a twin that is not well formed: its message.
    fn refused(text: &str) -> String {
        let source = format!("model T {{ name: \"t\" }}\ntwin \"W\" {}\n", text);
        crate::compiler::parser::Parser::new(crate::compiler::lexer::Lexer::new(&source).tokenize().unwrap()).parse().map(|_| String::new()).unwrap_or_else(|e| e)
    }

    #[test]
    fn a_twin_says_what_is_wrong() {
        let without = |key: &str| TWIN.lines().filter(|l| !l.trim_start().starts_with(key)).collect::<Vec<_>>().join("\n");
        assert!(refused(&without("drives")).contains("drives"));
        assert!(refused(&TWIN.replace("room.csv", "room.xlsx")).contains("Parquet file"));
        assert_eq!(refused(&TWIN.replace("room.csv", "room.parquet")), "");
        assert!(refused(&TWIN.replace("data/room.csv", "../room.csv")).contains("inside the model's folder"));
        assert!(refused(&TWIN.replace("verifies", "verifys")).contains("'verifys' is not one of"));
        assert!(parse_compare("T ~ room °C within 2 kg").unwrap_err().contains("kg"));
        assert!(parse_compare("T ~ room").unwrap_err().contains("within"));
        assert!(parse_drive("heater.power = x").unwrap_err().contains("<-"));
    }

    #[test]
    fn live_data_comes_from_an_mqtt_broker_or_an_opcua_server() {
        let with = |data: &str| TWIN.replace(r#"data: { file: "data/room.csv" time: "timestamp" }"#, data);
        let mqtt = spec_of(&attributes(&with(r#"data: { mqtt: "mqtt://broker:1883" topics: ["site/#"] window: 24 h refresh: 1 min }"#))).unwrap();
        let live = mqtt.data.live.unwrap();
        assert_eq!(live.source, Source::Mqtt { topics: vec!["site/#".into()], client_id: None });
        assert_eq!((live.window, live.refresh), (86400.0, 60.0));
        assert_eq!(live.label(), "mqtt://broker:1883 (site/#, last 24 h)");

        let opcua = spec_of(&attributes(&with(r#"data: { opcua: "opc.tcp://plc:4840" nodes: { room: "ns=2;s=Room.T" outside: "ns=2;i=1002" } security: "sign_and_encrypt" sampling: 500 ms record: "data/live.csv" }"#))).unwrap();
        let live = opcua.data.live.unwrap();
        let Source::OpcUa { nodes, security, policy, sampling } = &live.source else { panic!("OPC UA") };
        assert_eq!(nodes, &vec![("outside".to_string(), "ns=2;i=1002".to_string()), ("room".to_string(), "ns=2;s=Room.T".to_string())]);
        assert_eq!((*security, policy.as_str(), *sampling), (Security::SignAndEncrypt, "Basic256Sha256", 0.5));
        assert_eq!(opcua.data.file, "data/live.csv");
        let anonymous = spec_of(&attributes(&with(r#"data: { opcua: "opc.tcp://plc:4840" nodes: { room: "i=2258" } }"#))).unwrap();
        assert!(matches!(anonymous.data.live.unwrap().source, Source::OpcUa { security: Security::None, ref policy, .. } if policy == "None"));

        let refused_data = |data: &str| refused(&with(data));
        assert!(refused_data(r#"data: { opcua: "opc.tcp://plc:4840" }"#).contains("nodes"));
        assert!(refused_data(r#"data: { opcua: "opc.tcp://plc:4840" nodes: { room: "Room.T" } }"#).contains("not an OPC UA node id"));
        assert!(refused_data(r#"data: { opcua: "opc.tcp://user:pw@plc:4840" nodes: { room: "i=1" } }"#).contains("ARCLANG_OPCUA_USERNAME"));
        assert!(refused_data(r#"data: { opcua: "http://plc:4840" nodes: { room: "i=1" } }"#).contains("opc.tcp://"));
        assert!(refused_data(r#"data: { opcua: "opc.tcp://plc:4840" nodes: { room: "i=1" } policy: "Basic256Sha256" }"#).contains("signed or encrypted"));
        assert!(refused_data(r#"data: { opcua: "opc.tcp://plc:4840" nodes: { room: "i=1" } security: "sign" policy: "Basic256" }"#).contains("is one of"));
        assert!(refused_data(r#"data: { opcua: "opc.tcp://plc:4840" nodes: { room: "i=1" } topics: ["x"] }"#).contains("it is mqtt and kafka's"));
        assert!(refused_data(r#"data: { mqtt: "mqtt://broker" topics: ["x"] nodes: { room: "i=1" } }"#).contains("it is opcua's"));
        assert!(refused_data(r#"data: { mqtt: "mqtt://broker" opcua: "opc.tcp://plc:4840" topics: ["x"] }"#).contains("one source, not mqtt and opcua"));
    }

    #[test]
    fn live_data_comes_from_kafka_or_an_http_api() {
        let with = |data: &str| TWIN.replace(r#"data: { file: "data/room.csv" time: "timestamp" }"#, data);
        let refused_data = |data: &str| refused(&with(data));
        let kafka = spec_of(&attributes(&with(r#"data: { kafka: "kafkas://k1:9093,k2" topics: ["house.telemetry"] sasl: "scram-sha-512" window: 6 h }"#))).unwrap();
        let live = kafka.data.live.unwrap();
        assert_eq!(live.source, Source::Kafka { brokers: vec!["k1:9093".into(), "k2:9092".into()], tls: true, topics: vec!["house.telemetry".into()], client_id: None, sasl: "scram-sha-512".into() });
        assert_eq!(live.label(), "kafkas://k1:9093,k2 (house.telemetry, last 6 h)");
        let http = spec_of(&attributes(&with(r#"data: { http: "https://api.example/house/42/latest" record: "data/live.csv" }"#))).unwrap();
        let live = http.data.live.unwrap();
        assert_eq!(live.source, Source::Http { sampling: 10.0 });
        assert_eq!(live.label(), "https://api.example/house/42/latest (every 10 s, last 60 min)");
        assert!(refused_data(r#"data: { kafka: "kafka://k1:9092" }"#).contains("house.telemetry"));
        assert!(refused_data(r#"data: { kafka: "kafka://u:p@k1:9092" topics: ["t"] }"#).contains("ARCLANG_KAFKA_USERNAME"));
        assert!(refused_data(r#"data: { kafka: "http://k1:9092" topics: ["t"] }"#).contains("kafka://host:port"));
        assert!(refused_data(r#"data: { kafka: "kafka://k1:port" topics: ["t"] }"#).contains("kafka://host:port"));
        assert!(refused_data(r#"data: { kafka: "kafka://k1" topics: ["t"] sasl: "gssapi" }"#).contains("is one of plain"));
        assert!(refused_data(r#"data: { kafka: "kafka://k1" topics: ["t"] sampling: 1 s }"#).contains("it is opcua and http's"));
        assert!(refused_data(r#"data: { http: "https://token@api.example/x" }"#).contains("ARCLANG_HTTP_TOKEN"));
        assert!(refused_data(r#"data: { http: "ftp://api.example/x" }"#).contains("http://host/path"));
        assert!(refused_data(r#"data: { http: "https://api.example/x" topics: ["t"] }"#).contains("it is mqtt and kafka's"));
        assert!(refused_data(r#"data: { http: "https://api.example/x" sasl: "plain" }"#).contains("it is kafka's"));
        assert!(refused_data(r#"data: { file: "a.csv" sasl: "plain" }"#).contains("not for a file"));
        let stepped = spec_of(&attributes(&with(r#"data: { http: "https://api.example/x" advance: "step" }"#))).unwrap().data.live.unwrap();
        assert!(stepped.step && stepped.label().ends_with(", stepped)"));
        assert_eq!(window_text(0.5), "500 ms");
        assert!(refused_data(r#"data: { http: "https://api.example/x" advance: "fmu" }"#).contains("\"step\""));
        assert!(refused_data(r#"data: { file: "a.csv" advance: "step" }"#).contains("not for a file"));
        assert!(refused_data(r#"data: { file: "data/room.csv" sampling: 1 s }"#).contains("live data"));
    }

    #[test]
    fn an_opcua_node_id_is_written_in_its_text_form() {
        for good in ["ns=2;s=Room.T", "ns=2;i=1002", "i=2258", "ns=3;g=09087e75-8e5e-499b-954f-f2a9603db28a", "ns=1;b=M/RbKBsRVkePCePcx24oRA==", "nsu=urn:plant;s=T"] {
            assert!(is_node_id(good), "{good}");
        }
        for bad in ["Room.T", "ns=x;s=T", "ns=2;i=12a", "ns=2;s=", "s"] {
            assert!(!is_node_id(bad), "{bad}");
        }
    }

    #[test]
    fn a_comparison_may_allow_a_share_of_outliers() {
        assert_eq!(parse_compare("T ~ room °C within 0.5 K for 99 %").unwrap().share, 0.99);
        assert_eq!(parse_compare("T ~ room °C within 0.5 K").unwrap().share, 1.0);
        assert!(parse_compare("T ~ room within 0.5 for 120 %").unwrap_err().contains("0 to 100"));
    }

    #[test]
    fn a_tolerance_in_celsius_is_a_difference() {
        assert_eq!(parse_compare("T ~ room °C within 2").unwrap().tolerance, 2.0);
        assert_eq!(parse_compare("v ~ speed km/h within 3.6").unwrap().tolerance, 1.0);
    }

    #[test]
    fn the_measurements_read_dates_and_units_and_drive_the_inputs() {
        let data = Data { file: "x.csv".into(), live: None, time: "timestamp".into(), time_unit: lookup_unit("s").unwrap(), delimiter: None };
        let m = read_measurements("timestamp;heater_kw;outside\n2026-10-28T08:00:00Z;0;5\n2026-10-28T08:10:00Z;2,5;\n2026-10-28T08:20:00Z;2;7\n", &data).unwrap();
        assert_eq!(m.times, vec![0.0, 600.0, 1200.0]);
        assert_eq!(m.start, "2026-10-28T08:00:00Z");
        let drives = vec![parse_drive("heater.power <- heater_kw kW").unwrap(), parse_drive("amb.T <- outside °C").unwrap()];
        let csv = input_csv(&m, &drives).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "time,twin_in_0,twin_in_1");
        assert_eq!(lines[1], "0,0,278.15");
        // An empty cell is interpolated between its neighbors.
        assert_eq!(lines[2], "600,2500,279.15");
        assert!(read_measurements("t,x\n2,1\n1,2\n", &Data { time: "t".into(), ..data.clone() }).unwrap_err().contains("back in time"));
    }

    #[cfg(feature = "native")]
    #[test]
    fn a_parquet_file_reads_its_timestamps_whatever_their_unit() {
        // Written by pandas: nanoseconds by default (snappy), microseconds on request (zstd); a null, a boolean, an int32.
        for name in ["ns", "us"] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/twin/{name}.parquet"));
            let data = Data { file: format!("{name}.parquet"), live: None, time: "timestamp".into(), time_unit: lookup_unit("s").unwrap(), delimiter: None };
            let m = load_measurements(&path, &data).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(m.times, vec![0.0, 600.0, 1200.0], "{name}");
            assert_eq!(m.start, "2026-01-14T00:00:00+00:00", "{name}");
            assert_eq!(m.columns["outdoor_c"], vec![Some(-9.5), None, Some(-8.5)], "{name}");
            assert_eq!(m.columns["pump_on"], vec![Some(1.0), Some(0.0), Some(1.0)], "{name}");
            assert_eq!(m.columns["count"], vec![Some(1.0), Some(2.0), Some(3.0)], "{name}");
        }
    }

    #[test]
    fn the_wrapper_binds_each_driven_variable_to_an_input() {
        let text = wrapper("P.Room", &["heater.power".into(), "amb.T".into()]).unwrap();
        assert!(text.contains("extends P.Room(heater.power = twin_in_0, amb.T = twin_in_1);"));
        assert!(text.contains("input Real twin_in_1;"));
        assert!(wrapper("P.Room", &["a.b = 1); bad(".into()]).is_err());
        let aliases = BTreeMap::from([("PN-AMB".to_string(), "ambient".to_string())]);
        assert_eq!(resolve("\"PN-AMB\".T", &aliases), "ambient.T");
    }

    #[test]
    fn a_comparison_finds_the_largest_deviation_and_the_first_instant_beyond() {
        let series = Series::from_csv("\"time\",\"T\"\n0,10\n10,12\n20,14\n").unwrap();
        let (max, rms, within, first, curve) = compare(&series, 1, &[(0.0, 10.0), (10.0, 12.4), (20.0, 15.0)], 0.5);
        assert!((max - 1.0).abs() < 1e-9);
        assert!((rms - ((0.0f64 + 0.16 + 1.0) / 3.0).sqrt()).abs() < 1e-9);
        assert!((within - 2.0 / 3.0).abs() < 1e-9);
        assert_eq!(first, Some(20.0));
        assert_eq!(curve.time, vec![0.0, 10.0, 20.0]);
    }
}
