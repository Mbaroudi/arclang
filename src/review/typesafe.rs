//! The TypeSafe judgment API (model Jev) as a [`Judge`]: one yes/no question
//! per pair, answered by a probability.
//!
//! The questions and their criteria are the ones measured on the example
//! models of this repository (see the README for the figures); changing
//! their wording changes what the probabilities mean. A trace that states
//! its rationale is judged with it: on swapped traces the stated reason
//! contradicts the wrong target, which separates them better.

use super::{Judge, Pair};
use serde_json::{json, Value};
use std::time::Duration;

pub const ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
pub const KEY_VARIABLE: &str = "TYPESAFE_API_KEY";
const MODEL: &str = "jev-latest";
/// Requests retried when the service asks to slow down (429, 529).
const ATTEMPTS: u32 = 4;

const CONTEXT: &str = "In a systems engineering model, `source` is declared to have the relation `relation` towards `target` (satisfies: the element contributes to meeting the requirement; realizes: the element implements the higher-level element). ";

/// The request sent for one pair: its two ends, their relation and the
/// stated rationale as state, one question about them.
pub fn request_body(pair: &Pair) -> Value {
    let mut state = json!({ "source": pair.source, "target": pair.target, "relation": pair.relation });
    let question = match &pair.rationale {
        Some(rationale) => {
            state["rationale"] = json!(rationale);
            "The author's stated reason for the trace is `rationale`. Judging from what the two elements say about themselves and from that reason, is this trace plausible?"
        }
        None => "Judging only from what the two elements say about themselves, is this trace plausible?",
    };
    json!({
        "model": MODEL,
        "state": state,
        "questions": {
            "plausible": {
                "type": "noul",
                "instructions": format!("{}{}", CONTEXT, question),
                "criteria": {
                    "true": "What `source` is or does clearly relates to what `target` asks for or describes.",
                    "false": "`source` has no evident connection with `target`: the trace looks like a mistake."
                }
            }
        }
    })
}

/// The probability and the answering model of a response.
pub fn read_response(response: &Value) -> Result<(f64, Option<String>), String> {
    let probability = response["answers"]["plausible"]["noul"]
        .as_f64()
        .filter(|p| (0.0..=1.0).contains(p))
        .ok_or_else(|| "the judgment service answered without a probability".to_string())?;
    Ok((probability, response["model"].as_str().map(str::to_string)))
}

/// The value of `KEY_VARIABLE` in the text of a `.env` file. Quotes around
/// the value are dropped, however many layers of them.
pub fn key_from_env_file(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        if name.trim().trim_start_matches("export ").trim() != KEY_VARIABLE {
            return None;
        }
        let mut value = value.trim();
        while value.len() >= 2 && (value.starts_with('"') && value.ends_with('"') || value.starts_with('\'') && value.ends_with('\'')) {
            value = value[1..value.len() - 1].trim();
        }
        Some(value.to_string()).filter(|value| !value.is_empty())
    })
}

/// The API key: the environment variable, else the `.env` file of the
/// current directory.
pub fn find_key() -> Result<String, String> {
    if let Ok(key) = std::env::var(KEY_VARIABLE) {
        if !key.trim().is_empty() {
            return Ok(key.trim().to_string());
        }
    }
    std::fs::read_to_string(".env")
        .ok()
        .and_then(|text| key_from_env_file(&text))
        .ok_or_else(|| format!("no API key: set {} or put it in ./.env", KEY_VARIABLE))
}

pub struct TypeSafe {
    client: reqwest::blocking::Client,
    endpoint: String,
    key: String,
    model: Option<String>,
    /// Pause before the first retry; doubled at each further one.
    backoff: Duration,
}

impl TypeSafe {
    pub fn new(key: String) -> Result<TypeSafe, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent(concat!("arclang/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| format!("cannot set up the HTTP client: {}", e))?;
        Ok(TypeSafe { client, endpoint: ENDPOINT.to_string(), key, model: None, backoff: Duration::from_secs(1) })
    }

    /// Use another endpoint and retry pace (tests, a proxy).
    pub fn at(mut self, endpoint: &str, backoff: Duration) -> TypeSafe {
        self.endpoint = endpoint.to_string();
        self.backoff = backoff;
        self
    }
}

impl Judge for TypeSafe {
    fn plausibility(&mut self, pair: &Pair) -> Result<f64, String> {
        let body = request_body(pair);
        for attempt in 0..ATTEMPTS {
            // Errors name the service and the status, never the key.
            let response = self
                .client
                .post(&self.endpoint)
                .bearer_auth(&self.key)
                .json(&body)
                .send()
                .map_err(|e| format!("the judgment service cannot be reached: {}", e.without_url()))?;
            let status = response.status().as_u16();
            match status {
                200 => {
                    let answer: Value = response.json().map_err(|e| format!("unreadable answer from the judgment service: {}", e))?;
                    let (probability, model) = read_response(&answer)?;
                    self.model = model.or(self.model.take());
                    return Ok(probability);
                }
                429 | 529 if attempt + 1 < ATTEMPTS => std::thread::sleep(self.backoff * 2u32.pow(attempt)),
                429 | 529 => break,
                401 => return Err(format!("the judgment service refused the API key ({})", KEY_VARIABLE)),
                _ => return Err(format!("the judgment service answered with status {}", status)),
            }
        }
        Err("the judgment service kept asking to slow down".to_string())
    }

    fn model(&self) -> Option<String> {
        self.model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_is_read_from_an_env_file_whatever_its_quoting() {
        for text in [
            "TYPESAFE_API_KEY=abc_123",
            "OTHER=1\nTYPESAFE_API_KEY=\"abc_123\"\n",
            "TYPESAFE_API_KEY='\"abc_123\"'",
            "export TYPESAFE_API_KEY = \"'abc_123'\"  ",
        ] {
            assert_eq!(key_from_env_file(text).as_deref(), Some("abc_123"), "{text}");
        }
        assert_eq!(key_from_env_file("TYPESAFE_API_KEY=\"\""), None);
        assert_eq!(key_from_env_file("NOT_TYPESAFE_API_KEY=abc"), None);
    }

    #[test]
    fn the_rationale_is_sent_and_asked_about_only_when_the_trace_states_one() {
        let mut pair = Pair {
            source_id: "LC-1".to_string(),
            target_id: "REQ-1".to_string(),
            relation: "satisfies".to_string(),
            source: json!({ "name": "Radar" }),
            target: json!({ "name": "REQ-1" }),
            rationale: None,
        };
        let without = request_body(&pair);
        assert_eq!(without["state"].get("rationale"), None);
        assert!(!without["questions"]["plausible"]["instructions"].as_str().unwrap().contains("`rationale`"));

        pair.rationale = Some("Long-range detection".to_string());
        let with = request_body(&pair);
        assert_eq!(with["state"]["rationale"], "Long-range detection");
        assert!(with["questions"]["plausible"]["instructions"].as_str().unwrap().contains("stated reason for the trace is `rationale`"));
    }

    #[test]
    fn a_response_without_a_valid_probability_is_an_error() {
        let good = json!({ "model": "jev-1.13.0", "answers": { "plausible": { "type": "noul", "noul": 0.42 } } });
        assert_eq!(read_response(&good), Ok((0.42, Some("jev-1.13.0".to_string()))));
        assert!(read_response(&json!({ "answers": {} })).is_err());
        assert!(read_response(&json!({ "answers": { "plausible": { "noul": 1.7 } } })).is_err());
    }
}
