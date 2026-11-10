//! The address of a twin's live data is written in the model; the token it
//! is read with comes from the environment. A model received from someone
//! else must not be able to send that token to an address of its choosing:
//! `arclang twin --live` sends it to this machine, or to a host named in
//! `ARCLANG_TWIN_HOSTS`, and to no other.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

const HEAT_PUMP: &str = "examples/building/heat-pump";
const TOKEN: &str = "token-of-the-test";

/// The heat pump with its twin reading `url` instead of its file.
fn model_reading(name: &str, url: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-twin-credentials-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("physics")).unwrap();
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join(HEAT_PUMP);
    std::fs::copy(from.join("physics/HeatingPhysics.mo"), dir.join("physics/HeatingPhysics.mo")).unwrap();
    let text = std::fs::read_to_string(from.join("heat_pump_heating.arc")).unwrap();
    let file = "data: { file: \"data/house-2026-01-14.csv\" time: \"timestamp\" }";
    assert_eq!(text.matches(file).count(), 1, "the twin of the example reads its file");
    let model = dir.join("heat_pump_heating.arc");
    std::fs::write(&model, text.replace(file, &format!("data: {{ http: \"{url}\" sampling: 1 s window: 1 h refresh: 1 min }}"))).unwrap();
    model
}

/// `arclang twin --live --for 1` on the model, with this environment.
fn follow(model: &Path, environment: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_arclang"));
    command.args(["twin", model.to_str().unwrap(), "--live", "--for", "1"]).stdin(Stdio::null());
    for name in ["ARCLANG_HTTP_TOKEN", "ARCLANG_TWIN_HOSTS"] {
        command.env_remove(name);
    }
    command.envs(environment.iter().copied()).output().expect("arclang runs")
}

fn said(output: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

/// An API on this machine that answers one request with `{}`, and gives back what it was asked.
fn api() -> (String, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://127.0.0.1:{}/latest", listener.local_addr().unwrap().port());
    listener.set_nonblocking(true).unwrap();
    let asked = std::thread::spawn(move || {
        let waited = Instant::now();
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(_) if waited.elapsed() < Duration::from_secs(20) => std::thread::sleep(Duration::from_millis(20)),
                Err(error) => return format!("nothing asked: {error}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut request = Vec::new();
        let mut chunk = [0u8; 1024];
        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
            match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => request.extend_from_slice(&chunk[..n]),
            }
        }
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}");
        String::from_utf8_lossy(&request).to_ascii_lowercase()
    });
    (url, asked)
}

#[test]
fn the_token_is_not_sent_to_a_host_the_model_chose() {
    let model = model_reading("stranger", "https://collector.example.invalid/latest");
    let started = Instant::now();
    let output = follow(&model, &[("ARCLANG_HTTP_TOKEN", TOKEN)]);
    let text = said(&output);
    assert!(!output.status.success(), "{text}");
    assert!(text.contains("ARCLANG_HTTP_TOKEN is set, and 'collector.example.invalid' is not a host it is for"), "{text}");
    assert!(text.contains("ARCLANG_TWIN_HOSTS=collector.example.invalid"), "{text}");
    assert!(!text.contains(TOKEN), "the token is never printed: {text}");
    assert!(started.elapsed() < Duration::from_secs(20), "it stops before it connects");
}

#[test]
fn a_named_host_is_followed() {
    // The name resolves nowhere: the run says it cannot reach it, which is past the refusal.
    let model = model_reading("named", "https://collector.example.invalid/latest");
    let output = follow(&model, &[("ARCLANG_HTTP_TOKEN", TOKEN), ("ARCLANG_TWIN_HOSTS", "other.example, collector.example.invalid")]);
    let text = said(&output);
    assert!(output.status.success(), "{text}");
    assert!(!text.contains("is not a host"), "{text}");
    assert!(text.contains("unreachable"), "{text}");
}

#[test]
fn without_a_token_any_host_is_followed() {
    let model = model_reading("anonymous", "https://collector.example.invalid/latest");
    let output = follow(&model, &[]);
    let text = said(&output);
    assert!(output.status.success(), "{text}");
    assert!(text.contains("unreachable"), "{text}");
}

#[test]
fn the_token_goes_to_this_machine() {
    let (url, asked) = api();
    let model = model_reading("local", &url);
    let output = follow(&model, &[("ARCLANG_HTTP_TOKEN", TOKEN)]);
    let request = asked.join().unwrap();
    assert!(output.status.success(), "{}", said(&output));
    assert!(request.starts_with("get /latest "), "{request}");
    assert!(request.contains(&format!("authorization: bearer {TOKEN}")), "{request}");
}
