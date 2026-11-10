//! Where the credentials of a twin's live data may go.
//!
//! The model names the address of its live data; the user and the password,
//! or the token, come from the environment and belong to one service. A
//! model received from someone else can name another address: the
//! credentials of the environment are sent only to this machine, or to a
//! host named in `ARCLANG_TWIN_HOSTS`. Named nowhere, the host gets nothing:
//! the run stops before it connects and says which variable to set.

use super::twin::Source;

/// The hosts the credentials are for, separated by commas: `plc.plant.example`,
/// `*.plant.example` (any host under it), or `*` (any host).
pub const HOSTS: &str = "ARCLANG_TWIN_HOSTS";

const MQTT: (&str, &str) = ("ARCLANG_MQTT_USERNAME", "ARCLANG_MQTT_PASSWORD");
const OPCUA: (&str, &str) = ("ARCLANG_OPCUA_USERNAME", "ARCLANG_OPCUA_PASSWORD");
const KAFKA: (&str, &str) = ("ARCLANG_KAFKA_USERNAME", "ARCLANG_KAFKA_PASSWORD");
const HTTP: &str = "ARCLANG_HTTP_TOKEN";

fn environment(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

fn both(read: &dyn Fn(&str) -> Option<String>, (user, password): (&str, &str)) -> Option<(String, String)> {
    Some((read(user)?, read(password)?))
}

fn filled(read: &dyn Fn(&str) -> Option<String>, name: &str) -> Option<String> {
    read(name).filter(|value| !value.is_empty())
}

/// The user and the password of the MQTT broker, when both are set.
pub fn mqtt() -> Option<(String, String)> {
    both(&environment, MQTT)
}

/// The user and the password of the OPC UA server, when both are set.
pub fn opcua() -> Option<(String, String)> {
    both(&environment, OPCUA)
}

/// The user and the password of the Kafka brokers, when both are set and not empty.
pub fn kafka() -> Option<(String, String)> {
    kafka_from(&environment)
}

fn kafka_from(read: &dyn Fn(&str) -> Option<String>) -> Option<(String, String)> {
    Some((filled(read, KAFKA.0)?, filled(read, KAFKA.1)?))
}

/// The bearer token of the HTTP API, when it is set and not empty.
pub fn http_token() -> Option<String> {
    filled(&environment, HTTP)
}

/// The variables whose credentials this source would send, as a message names them; None when
/// it connects without credentials.
fn sent(source: &Source, read: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    let pair = |(user, password): (&str, &str)| format!("{user} and {password}");
    match source {
        Source::Mqtt { .. } => both(read, MQTT).map(|_| pair(MQTT)),
        Source::OpcUa { .. } => both(read, OPCUA).map(|_| pair(OPCUA)),
        Source::Kafka { .. } => kafka_from(read).map(|_| pair(KAFKA)),
        Source::Http { .. } => filled(read, HTTP).map(|_| HTTP.to_string()),
    }
}

/// The hosts an address names, in lower case and without their ports: one, or one per broker
/// for Kafka (`kafka://a:9092,b:9092`). An IPv6 address is written without its brackets. An
/// HTTP address is read by the parser that sends the request, so that both see the same host.
pub fn hosts_of(url: &str) -> Vec<String> {
    let (scheme, rest) = url.split_once("://").unwrap_or(("", url));
    if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") {
        if let Some(host) = reqwest::Url::parse(url).ok().and_then(|parsed| parsed.host_str().map(str::to_string)) {
            return vec![host.trim_start_matches('[').trim_end_matches(']').trim_end_matches('.').to_ascii_lowercase()];
        }
    }
    let authority = rest.split(['/', '?', '#', '\\']).next().unwrap_or("");
    authority
        .split(',')
        .map(|one| {
            let one = one.trim();
            let one = one.rsplit_once('@').map(|(_, host)| host).unwrap_or(one);
            let host = match one.strip_prefix('[') {
                Some(bracketed) => bracketed.split(']').next().unwrap_or(""),
                None => match one.rsplit_once(':') {
                    Some((host, port)) if !host.contains(':') && port.chars().all(|c| c.is_ascii_digit()) => host,
                    _ => one,
                },
            };
            host.trim_end_matches('.').to_ascii_lowercase()
        })
        .filter(|host| !host.is_empty())
        .collect()
}

/// This machine: what is sent there does not leave it.
fn is_this_machine(host: &str) -> bool {
    host == "localhost" || host.parse::<std::net::IpAddr>().is_ok_and(|address| address.is_loopback())
}

fn is_named(host: &str, named: &str) -> bool {
    named.split(',').map(|pattern| pattern.trim().trim_end_matches('.').to_ascii_lowercase()).filter(|pattern| !pattern.is_empty()).any(|pattern| match pattern.strip_prefix("*.") {
        _ if pattern == "*" => true,
        Some(domain) => host.strip_suffix(domain).is_some_and(|under| under.ends_with('.') && under.len() > 1),
        None => host == pattern,
    })
}

/// Refuses, before anything is connected, to send the credentials of the environment to a
/// host they were not said to be for.
pub fn guard(url: &str, source: &Source) -> Result<(), String> {
    guard_with(url, source, &environment)
}

fn guard_with(url: &str, source: &Source, read: &dyn Fn(&str) -> Option<String>) -> Result<(), String> {
    let Some(variables) = sent(source, read) else { return Ok(()) };
    let named = read(HOSTS).unwrap_or_default();
    let hosts = hosts_of(url);
    let Some(stranger) = hosts.iter().find(|host| !is_this_machine(host) && !is_named(host, &named)) else { return Ok(()) };
    let wanted = hosts.iter().filter(|host| !is_this_machine(host)).cloned().collect::<Vec<_>>().join(",");
    Err(format!(
        "{url}: {variables} {} set, and '{stranger}' is not a host {} for: the credentials of the environment are sent only to the hosts named in {HOSTS} ({HOSTS}={wanted}); unset them to connect without credentials",
        if variables.contains(" and ") { "are" } else { "is" },
        if variables.contains(" and ") { "they are" } else { "it is" },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |name| map.get(name).cloned()
    }

    fn http() -> Source {
        Source::Http { sampling: 10.0 }
    }

    fn kafka_source() -> Source {
        Source::Kafka { brokers: vec![], tls: false, topics: vec![], client_id: None, sasl: "plain".to_string() }
    }

    #[test]
    fn the_hosts_of_an_address() {
        assert_eq!(hosts_of("https://API.Example.com:8443/houses/042?x=1"), ["api.example.com"]);
        assert_eq!(hosts_of("mqtt://broker"), ["broker"]);
        assert_eq!(hosts_of("opc.tcp://plc.plant.example.:4840/server"), ["plc.plant.example"]);
        assert_eq!(hosts_of("kafka://a.example:9092, b.example:9093"), ["a.example", "b.example"]);
        assert_eq!(hosts_of("http://[::1]:8080/x"), ["::1"]);
        assert_eq!(hosts_of("http://[2001:db8::7]/x"), ["2001:db8::7"]);
        // What is after an @ is where the request goes, whatever is written before it.
        assert_eq!(hosts_of("http://localhost@collector.example/x"), ["collector.example"]);
        assert_eq!(hosts_of("http://collector.example#@localhost/"), ["collector.example"]);
        // As the HTTP client reads it: a backslash ends the host, a number is an address.
        assert_eq!(hosts_of("http://collector.example\\@localhost/"), ["collector.example"]);
        assert_eq!(hosts_of("mqtt://collector.example\\@localhost"), ["collector.example"]);
        assert_eq!(hosts_of("http://2130706433/x"), ["127.0.0.1"]);
    }

    #[test]
    fn without_credentials_any_host_is_followed() {
        assert_eq!(guard_with("https://collector.example/latest", &http(), &env(&[])), Ok(()));
        // An empty token is no token; a user without a password is not sent.
        assert_eq!(guard_with("https://collector.example/latest", &http(), &env(&[(HTTP, "")])), Ok(()));
        assert_eq!(guard_with("mqtt://broker.example", &Source::Mqtt { topics: vec![], client_id: None }, &env(&[(MQTT.0, "u")])), Ok(()));
        // The credentials of another source are not this one's.
        assert_eq!(guard_with("https://collector.example/latest", &http(), &env(&[(MQTT.0, "u"), (MQTT.1, "p")])), Ok(()));
    }

    #[test]
    fn credentials_are_not_sent_to_a_host_that_is_not_named() {
        let error = guard_with("https://collector.example/latest", &http(), &env(&[(HTTP, "secret")])).unwrap_err();
        assert!(error.contains("ARCLANG_HTTP_TOKEN is set, and 'collector.example' is not a host it is for"), "{error}");
        assert!(error.contains("ARCLANG_TWIN_HOSTS=collector.example"), "{error}");
        assert!(!error.contains("secret"), "the message never repeats the credentials: {error}");

        let error = guard_with("opc.tcp://plc.other.example:4840", &Source::OpcUa { nodes: vec![], security: super::super::twin::Security::None, policy: String::new(), sampling: 1.0 }, &env(&[(OPCUA.0, "u"), (OPCUA.1, "p"), (HOSTS, "plc.plant.example")])).unwrap_err();
        assert!(error.contains("ARCLANG_OPCUA_USERNAME and ARCLANG_OPCUA_PASSWORD are set, and 'plc.other.example' is not a host they are for"), "{error}");
    }

    #[test]
    fn credentials_go_to_a_named_host_and_to_this_machine() {
        let secret = [(HTTP, "secret")];
        for local in ["http://localhost:8080/x", "http://127.0.0.1/x", "http://[::1]:8080/x", "mqtt://localhost", "opc.tcp://127.0.0.1:4840"] {
            assert_eq!(guard_with(local, &http(), &env(&secret)), Ok(()), "{local}");
        }
        let named = |hosts: &'static str| env(&[(HTTP, "secret"), (HOSTS, hosts)]);
        assert_eq!(guard_with("https://api.example.com/x", &http(), &named("other.example, API.example.com")), Ok(()));
        assert_eq!(guard_with("https://api.plant.example/x", &http(), &named("*.plant.example")), Ok(()));
        assert_eq!(guard_with("https://anywhere.example/x", &http(), &named("*")), Ok(()));
        // A domain names what is under it, not itself, and not a host that only ends like it.
        assert!(guard_with("https://plant.example/x", &http(), &named("*.plant.example")).is_err());
        assert!(guard_with("https://evilplant.example/x", &http(), &named("*.plant.example")).is_err());
        assert!(guard_with("https://api.example.com.evil.example/x", &http(), &named("api.example.com")).is_err());
        // A name that looks like this machine's is not this machine.
        assert!(guard_with("https://localhost.evil.example/x", &http(), &env(&secret)).is_err());
        assert!(guard_with("http://127.0.0.1.evil.example/x", &http(), &env(&secret)).is_err());
        assert!(guard_with("http://api.localhost.evil.example/x", &http(), &env(&secret)).is_err());
    }

    #[test]
    fn every_broker_of_a_kafka_address_is_named() {
        let credentials = |hosts: &'static str| env(&[(KAFKA.0, "u"), (KAFKA.1, "p"), (HOSTS, hosts)]);
        assert_eq!(guard_with("kafkas://a.example:9093,b.example:9093", &kafka_source(), &credentials("a.example,b.example")), Ok(()));
        let error = guard_with("kafkas://a.example:9093,b.example:9093", &kafka_source(), &credentials("a.example")).unwrap_err();
        assert!(error.contains("'b.example' is not a host they are for"), "{error}");
        assert!(error.contains("ARCLANG_TWIN_HOSTS=a.example,b.example"), "{error}");
    }
}
