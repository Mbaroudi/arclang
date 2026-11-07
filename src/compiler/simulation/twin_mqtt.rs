//! The MQTT client of a live twin: MQTT 3.1.1, subscribe only, QoS 0, over
//! TCP or TLS (rustls, see [`super::twin_tls`]). A twin only listens, so this
//! is the few packets it needs (CONNECT, SUBSCRIBE, PUBLISH, PINGREQ and their
//! answers) and nothing else: no session kept on the broker, nothing sent but
//! pings. A lost connection is opened again two seconds later.

use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

/// The broker is pinged when it has said nothing for this long.
pub const KEEP_ALIVE: Duration = Duration::from_secs(30);
/// How long a read waits before the client checks whether a ping is due.
const POLL: Duration = Duration::from_secs(1);
/// The largest packet accepted (the protocol's own limit is 256 MB).
const MAX_PACKET: usize = 16 * 1024 * 1024;

/// Where and as whom the client connects.
#[derive(Debug, Clone)]
pub struct Options {
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub client_id: String,
    pub credentials: Option<(String, String)>,
    pub topics: Vec<String>,
}

/// What a connection reports.
pub enum Event {
    Connected,
    Message { topic: String, payload: Vec<u8> },
}

trait Stream: Read + Write + Send {}
impl<T: Read + Write + Send> Stream for T {}

/// Connects, subscribes and hands every event to `on_event` until the
/// connection fails (the error) or `on_event` answers false (Ok).
pub fn session(options: &Options, on_event: &mut dyn FnMut(Event) -> bool) -> Result<(), String> {
    let mut stream = open(options)?;
    stream.write_all(&connect_packet(options)).map_err(|e| e.to_string())?;
    let mut reader = PacketReader::default();
    let (kind, body) = reader.next_blocking(&mut *stream)?;
    match (kind >> 4, body.as_slice()) {
        (2, [_, 0]) => {}
        (2, [_, code]) => return Err(connack_refusal(*code)),
        _ => return Err("the broker did not answer the connection".into()),
    }
    stream.write_all(&subscribe_packet(1, &options.topics)).map_err(|e| e.to_string())?;
    if !on_event(Event::Connected) {
        return Ok(());
    }
    let mut last_sent = Instant::now();
    let mut ping_sent: Option<Instant> = None;
    loop {
        match reader.next(&mut *stream)? {
            Some((kind, body)) => {
                ping_sent = None;
                match kind >> 4 {
                    3 => {
                        let (topic, payload, id) = publish(kind, &body)?;
                        if let Some(id) = id {
                            stream.write_all(&[0x40, 2, (id >> 8) as u8, id as u8]).map_err(|e| e.to_string())?;
                        }
                        if !on_event(Event::Message { topic, payload }) {
                            return Ok(());
                        }
                    }
                    9 if body.iter().skip(2).any(|&code| code == 0x80) => return Err(format!("the broker refused to subscribe to {}", options.topics.join(", "))),
                    _ => {}
                }
            }
            None => {
                if let Some(sent) = ping_sent {
                    if sent.elapsed() > KEEP_ALIVE {
                        return Err("the broker stopped answering".into());
                    }
                }
            }
        }
        if last_sent.elapsed() >= KEEP_ALIVE / 2 {
            stream.write_all(&[0xC0, 0]).map_err(|e| e.to_string())?;
            last_sent = Instant::now();
            ping_sent.get_or_insert(last_sent);
        }
    }
}

fn open(options: &Options) -> Result<Box<dyn Stream>, String> {
    let tcp = TcpStream::connect((options.host.as_str(), options.port)).map_err(|e| e.to_string())?;
    tcp.set_read_timeout(Some(POLL)).map_err(|e| e.to_string())?;
    tcp.set_nodelay(true).map_err(|e| e.to_string())?;
    if !options.tls {
        return Ok(Box::new(tcp));
    }
    let config = super::twin_tls::client_config("mqtts://")?;
    let name = rustls::pki_types::ServerName::try_from(options.host.clone()).map_err(|e| format!("{}: {e}", options.host))?;
    let connection = rustls::ClientConnection::new(config, name).map_err(|e| e.to_string())?;
    Ok(Box::new(rustls::StreamOwned::new(connection, tcp)))
}

fn connack_refusal(code: u8) -> String {
    let why = match code {
        1 => "it does not speak MQTT 3.1.1",
        2 => "it refused the client id",
        3 => "it is unavailable",
        4 => "the user name or password is wrong (ARCLANG_MQTT_USERNAME, ARCLANG_MQTT_PASSWORD)",
        5 => "this client is not authorized",
        _ => "for a reason it did not name",
    };
    format!("the broker refused the connection: {why}")
}

/// Bytes read so far, cut into packets as they complete. A read that times
/// out keeps what it had, so a packet split across reads is never lost.
#[derive(Default)]
struct PacketReader {
    buffer: Vec<u8>,
}

impl PacketReader {
    /// The next whole packet (its first byte and its body), or None when
    /// nothing completed one before the read timed out.
    fn next(&mut self, stream: &mut dyn Read) -> Result<Option<(u8, Vec<u8>)>, String> {
        loop {
            if let Some(packet) = self.take()? {
                return Ok(Some(packet));
            }
            let mut chunk = [0u8; 8192];
            match stream.read(&mut chunk) {
                Ok(0) => return Err("the broker closed the connection".into()),
                Ok(n) => self.buffer.extend_from_slice(&chunk[..n]),
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => return Ok(None),
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    fn next_blocking(&mut self, stream: &mut dyn Read) -> Result<(u8, Vec<u8>), String> {
        let started = Instant::now();
        loop {
            if let Some(packet) = self.next(stream)? {
                return Ok(packet);
            }
            if started.elapsed() > KEEP_ALIVE {
                return Err("the broker did not answer the connection".into());
            }
        }
    }

    fn take(&mut self) -> Result<Option<(u8, Vec<u8>)>, String> {
        let Some(&kind) = self.buffer.first() else { return Ok(None) };
        let Some((length, used)) = remaining_length(&self.buffer[1..])? else { return Ok(None) };
        if length > MAX_PACKET {
            return Err(format!("the broker sent a packet of {length} bytes"));
        }
        let start = 1 + used;
        if self.buffer.len() < start + length {
            return Ok(None);
        }
        let body = self.buffer[start..start + length].to_vec();
        self.buffer.drain(..start + length);
        Ok(Some((kind, body)))
    }
}

/// The remaining length at the start of `bytes` and how many bytes it took,
/// None while incomplete.
fn remaining_length(bytes: &[u8]) -> Result<Option<(usize, usize)>, String> {
    let mut value = 0usize;
    for (i, &byte) in bytes.iter().enumerate().take(4) {
        value |= usize::from(byte & 0x7F) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok(Some((value, i + 1)));
        }
    }
    if bytes.len() >= 4 {
        return Err("the broker sent a malformed packet length".into());
    }
    Ok(None)
}

fn encode_length(mut length: usize, out: &mut Vec<u8>) {
    loop {
        let mut byte = (length % 128) as u8;
        length /= 128;
        if length > 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if length == 0 {
            return;
        }
    }
}

fn string(text: &str, out: &mut Vec<u8>) {
    out.extend_from_slice(&(text.len() as u16).to_be_bytes());
    out.extend_from_slice(text.as_bytes());
}

fn packet(kind: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![kind];
    encode_length(body.len(), &mut out);
    out.extend_from_slice(body);
    out
}

fn connect_packet(options: &Options) -> Vec<u8> {
    let mut body = Vec::new();
    string("MQTT", &mut body);
    body.push(4);
    // Clean session; user name and password when given.
    body.push(if options.credentials.is_some() { 0xC2 } else { 0x02 });
    body.extend_from_slice(&(KEEP_ALIVE.as_secs() as u16).to_be_bytes());
    string(&options.client_id, &mut body);
    if let Some((user, password)) = &options.credentials {
        string(user, &mut body);
        string(password, &mut body);
    }
    packet(0x10, &body)
}

fn subscribe_packet(id: u16, topics: &[String]) -> Vec<u8> {
    let mut body = id.to_be_bytes().to_vec();
    for topic in topics {
        string(topic, &mut body);
        body.push(0);
    }
    packet(0x82, &body)
}

/// A PUBLISH's topic, payload and, above QoS 0, its packet id to acknowledge.
fn publish(kind: u8, body: &[u8]) -> Result<(String, Vec<u8>, Option<u16>), String> {
    let malformed = || "the broker sent a malformed message".to_string();
    let length = usize::from(u16::from_be_bytes([*body.first().ok_or_else(malformed)?, *body.get(1).ok_or_else(malformed)?]));
    let topic = std::str::from_utf8(body.get(2..2 + length).ok_or_else(malformed)?).map_err(|_| malformed())?.to_string();
    let mut at = 2 + length;
    let id = if (kind >> 1) & 3 > 0 {
        let id = u16::from_be_bytes([*body.get(at).ok_or_else(malformed)?, *body.get(at + 1).ok_or_else(malformed)?]);
        at += 2;
        Some(id)
    } else {
        None
    };
    Ok((topic, body[at..].to_vec(), id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn lengths_take_one_to_four_bytes() {
        for length in [0usize, 127, 128, 16_383, 16_384, 2_097_151, 2_097_152] {
            let mut bytes = Vec::new();
            encode_length(length, &mut bytes);
            assert_eq!(remaining_length(&bytes).unwrap(), Some((length, bytes.len())));
            assert_eq!(remaining_length(&bytes[..bytes.len() - 1]).unwrap(), None);
        }
        assert!(remaining_length(&[0xFF, 0xFF, 0xFF, 0xFF]).is_err());
    }

    #[test]
    fn a_connect_names_the_protocol_the_client_and_its_credentials() {
        let mut options = Options { host: "h".into(), port: 1, tls: false, client_id: "twin".into(), credentials: None, topics: vec![] };
        assert_eq!(connect_packet(&options), [0x10, 16, 0, 4, b'M', b'Q', b'T', b'T', 4, 0x02, 0, 30, 0, 4, b't', b'w', b'i', b'n']);
        options.credentials = Some(("u".into(), "p".into()));
        let with = connect_packet(&options);
        assert_eq!(with[9], 0xC2);
        assert!(with.ends_with(&[0, 1, b'u', 0, 1, b'p']));
    }

    #[test]
    fn a_packet_split_across_reads_is_kept_whole() {
        let whole = packet(0x30, &[0, 1, b't', b'4', b'2']);
        let mut reader = PacketReader::default();
        let mut first = &whole[..3];
        assert!(reader.next(&mut TimesOut(&mut first)).unwrap().is_none());
        let mut rest = &whole[3..];
        let (kind, body) = reader.next(&mut rest).unwrap().unwrap();
        assert_eq!(publish(kind, &body).unwrap(), ("t".to_string(), b"42".to_vec(), None));
        let qos1 = [0, 1, b't', 0, 7, b'1'];
        assert_eq!(publish(0x32, &qos1).unwrap(), ("t".to_string(), b"1".to_vec(), Some(7)));
        assert!(publish(0x30, &[0, 9, b't']).is_err());
    }

    /// Reads what it holds, then times out instead of reaching the end.
    struct TimesOut<'a, 'b>(&'a mut &'b [u8]);
    impl Read for TimesOut<'_, '_> {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            if self.0.is_empty() {
                return Err(ErrorKind::WouldBlock.into());
            }
            self.0.read(out)
        }
    }

    #[test]
    fn a_session_with_a_broker_delivers_its_messages() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let broker = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut reader = PacketReader::default();
            let (connect, _) = reader.next_blocking(&mut socket).unwrap();
            socket.write_all(&[0x20, 2, 0, 0]).unwrap();
            let (subscribe, body) = reader.next_blocking(&mut socket).unwrap();
            socket.write_all(&[0x90, 3, 0, 1, 0]).unwrap();
            socket.write_all(&packet(0x30, &[&[0, 9][..], b"site/temp", b"21.5"].concat())).unwrap();
            (connect, subscribe, body)
        });
        let options = Options { host: "127.0.0.1".into(), port, tls: false, client_id: "t".into(), credentials: None, topics: vec!["site/#".into()] };
        let mut events = Vec::new();
        session(&options, &mut |event| {
            events.push(match event {
                Event::Connected => "connected".to_string(),
                Event::Message { topic, payload } => format!("{topic}={}", String::from_utf8_lossy(&payload)),
            });
            events.len() < 2
        })
        .unwrap();
        assert_eq!(events, ["connected", "site/temp=21.5"]);
        let (connect, subscribe, body) = broker.join().unwrap();
        assert_eq!((connect, subscribe), (0x10, 0x82));
        assert_eq!(body, [&[0, 1, 0, 6][..], b"site/#", &[0]].concat());
    }

    /// Against a real broker: `ARCLANG_TEST_MQTT=mqtt://host:1883/topic` (or
    /// `mqtts://`), something publishing on the topic. Not run by default.
    #[test]
    #[ignore]
    fn a_real_broker_delivers_a_message() {
        let url = std::env::var("ARCLANG_TEST_MQTT").expect("ARCLANG_TEST_MQTT");
        let (scheme, rest) = url.split_once("://").unwrap();
        let (address, topic) = rest.split_once('/').unwrap();
        let (host, port) = address.rsplit_once(':').unwrap();
        let options = Options { host: host.into(), port: port.parse().unwrap(), tls: scheme == "mqtts", client_id: format!("arclang-test-{}", std::process::id()), credentials: None, topics: vec![topic.into()] };
        let mut received = None;
        session(&options, &mut |event| {
            if let Event::Message { topic, payload } = event {
                received = Some((topic, payload));
                return false;
            }
            true
        })
        .unwrap();
        let (topic, payload) = received.unwrap();
        eprintln!("{topic} = {}", String::from_utf8_lossy(&payload));
    }

    #[test]
    fn a_refused_connection_says_why() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let _ = PacketReader::default().next_blocking(&mut socket);
            socket.write_all(&[0x20, 2, 0, 4]).unwrap();
        });
        let options = Options { host: "127.0.0.1".into(), port, tls: false, client_id: "t".into(), credentials: None, topics: vec!["x".into()] };
        let error = session(&options, &mut |_| true).err().unwrap();
        assert!(error.contains("user name or password"), "{error}");
    }
}
