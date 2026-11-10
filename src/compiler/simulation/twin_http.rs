//! A twin's live data from an HTTP API: a GET every `sampling` seconds (10 s
//! by default), its answer a JSON object, a column per numeric or boolean
//! field at its `time` (else at the answer's arrival), or a list of such
//! objects; only what is newer than what was already read is kept, so an API
//! that answers with its latest hour at every call does not repeat samples.
//!
//! A bearer token comes from `ARCLANG_HTTP_TOKEN`, never from the model, and
//! goes only to a host it was said to be for (see [`super::twin_credentials`]).

use super::twin_live::{now, samples_of, Feed, Status};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

/// The samples newer, column by column, than the latest already kept; `latest` moves on.
pub fn fresh(samples: Vec<(f64, String, f64)>, latest: &mut HashMap<String, f64>) -> Vec<(f64, String, f64)> {
    let mut kept: Vec<(f64, String, f64)> = samples.into_iter().filter(|(t, column, _)| latest.get(column).is_none_or(|l| t > l)).collect();
    kept.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (t, column, _) in &kept {
        let l = latest.entry(column.clone()).or_insert(*t);
        *l = l.max(*t);
    }
    kept
}

/// A request's error with its causes ("error sending request" alone says little).
fn failure(error: &reqwest::Error) -> String {
    let mut text = error.to_string();
    let mut cause = std::error::Error::source(error);
    while let Some(c) = cause {
        text = format!("{text}: {c}");
        cause = c.source();
    }
    text.replace(&format!(" for url ({})", error.url().map(|u| u.as_str()).unwrap_or("")), "")
}

/// What an answer that is not a success means for the twin: `refused` for a missing or wrong
/// token, else the API is not reached.
fn refused(code: reqwest::StatusCode) -> (&'static str, String) {
    match code.as_u16() {
        401 | 403 => ("refused", format!("{code} (a bearer token comes from ARCLANG_HTTP_TOKEN)")),
        _ => ("unreachable", code.to_string()),
    }
}

/// Reads the API in a thread of its own.
pub fn follow(url: &str, sampling: f64, feed: Sender<Feed>) -> Result<(), String> {
    let period = Duration::from_secs_f64(sampling);
    let client = reqwest::blocking::Client::builder()
        .timeout(period.clamp(Duration::from_secs(5), Duration::from_secs(30)))
        .user_agent(concat!("arclang/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("HTTP: {e}"))?;
    let token = super::twin_credentials::http_token();
    let url = url.to_string();
    std::thread::spawn(move || {
        let mut connected = false;
        let mut latest: HashMap<String, f64> = HashMap::new();
        loop {
            let asked = Instant::now();
            let mut request = client.get(&url).header("Accept", "application/json");
            if let Some(token) = &token {
                request = request.bearer_auth(token);
            }
            let answer = request.send().map_err(|e| ("unreachable", failure(&e))).and_then(|response| {
                if !response.status().is_success() {
                    return Err(refused(response.status()));
                }
                response.json::<serde_json::Value>().map_err(|e| ("unreachable", format!("the answer is not JSON: {}", failure(&e))))
            });
            let sent = match answer {
                Ok(value) => {
                    let first = !connected;
                    connected = true;
                    let samples = fresh(samples_of(&value, now()), &mut latest);
                    let news = if first { feed.send(Feed::Status(Status::new("connected", url.clone()))) } else { Ok(()) };
                    news.and_then(|_| if samples.is_empty() { Ok(()) } else { feed.send(Feed::Samples(samples)) })
                }
                Err((kind, error)) => {
                    let status = if connected && kind == "unreachable" { Status::new("disconnected", error) } else { Status::new(kind, format!("{url}: {error}")) };
                    connected = false;
                    feed.send(Feed::Status(status))
                }
            };
            if sent.is_err() {
                return;
            }
            std::thread::sleep(period.saturating_sub(asked.elapsed()));
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_that_repeats_the_last_hour_adds_only_what_is_new() {
        let mut latest = HashMap::new();
        let first = samples_of(&serde_json::json!([{"time": 100, "x": 1}, {"time": 160, "x": 2, "y": 5}]), 0.0);
        assert_eq!(fresh(first, &mut latest).len(), 3);
        let again = samples_of(&serde_json::json!([{"time": 160, "x": 2, "y": 5}, {"time": 220, "x": 3}]), 0.0);
        assert_eq!(fresh(again, &mut latest), vec![(220.0, "x".to_string(), 3.0)]);
        // Without a time, an answer is at its arrival: always new.
        assert_eq!(fresh(samples_of(&serde_json::json!({"y": 6}), 300.0), &mut latest), vec![(300.0, "y".to_string(), 6.0)]);
        assert!(matches!(refused(reqwest::StatusCode::UNAUTHORIZED), ("refused", hint) if hint.contains("ARCLANG_HTTP_TOKEN")));
        assert_eq!(refused(reqwest::StatusCode::BAD_GATEWAY).0, "unreachable");
    }
}
