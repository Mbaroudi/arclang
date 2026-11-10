//! The bench of `arclang soc`: the console of the SoC, over TCP to the
//! simulator (LiteX's `serial2tcp`) or through a board's serial port, and
//! the step() API protocol of `pil_direct` on it.
//!
//! The target says `ARCLANG READY` once booted; a bench that comes later
//! (the simulator boots before the bench connects, a board before its port
//! is opened) asks with a line `?`, answered by `ARCLANG READY` again. Then
//! one line of inputs a step, one report `R <step>,...` back.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

/// How often a bench asks a target that has not said it is ready.
const PROBE: Duration = Duration::from_millis(1000);

/// The bytes a link receives, read by a thread so that every wait has a deadline.
struct Link {
    writer: Box<dyn Write + Send>,
    bytes: Receiver<Vec<u8>>,
    /// Everything received, for the log.
    log: Vec<u8>,
    /// Received and not yet taken as lines.
    pending: Vec<u8>,
}

impl Link {
    fn new(reader: Box<dyn Read + Send>, writer: Box<dyn Write + Send>) -> Link {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Link { writer, bytes: rx, log: Vec::new(), pending: Vec::new() }
    }

    fn send(&mut self, line: &str) -> Result<(), String> {
        self.writer.write_all(line.as_bytes()).and_then(|_| self.writer.write_all(b"\n")).and_then(|_| self.writer.flush()).map_err(|e| format!("cannot write to the target: {e}"))
    }

    /// The next complete line, or None at the deadline.
    fn line(&mut self, deadline: Instant) -> Result<Option<String>, String> {
        loop {
            if let Some(end) = self.pending.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = self.pending.drain(..=end).collect();
                return Ok(Some(String::from_utf8_lossy(&line).trim_end_matches(['\r', '\n']).to_string()));
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }
            match self.bytes.recv_timeout(deadline - now) {
                Ok(bytes) => {
                    self.log.extend_from_slice(&bytes);
                    self.pending.extend_from_slice(&bytes);
                }
                Err(RecvTimeoutError::Timeout) => return Ok(None),
                Err(RecvTimeoutError::Disconnected) => return Err("the target closed its console".to_string()),
            }
        }
    }

    fn log(&self) -> String {
        String::from_utf8_lossy(&self.log).into_owned()
    }
}

/// Wait for `ARCLANG READY`, asking with `?` meanwhile.
fn ready(link: &mut Link, seconds: u64) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        link.send("?")?;
        let probe = (Instant::now() + PROBE).min(deadline);
        while let Some(line) = link.line(probe)? {
            if line.starts_with("ARCLANG READY") {
                return Ok(());
            }
            if let Some(e) = line.strip_prefix("ARCLANG ERROR ") {
                return Err(format!("on the target: {e}"));
            }
        }
        if Instant::now() >= deadline {
            return Err(format!("the target did not say it was ready within {seconds} s; its console:\n{}", tail(&link.log())));
        }
    }
}

fn tail(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(15)..].join("\n")
}

/// The board's serial port opened raw at 115200 bauds, 8N1.
fn open_port(port: &str) -> Result<Link, String> {
    let flag = if cfg!(target_os = "macos") { "-f" } else { "-F" };
    let stty = Command::new("stty").args([flag, port, "115200", "raw", "-echo", "cs8", "-cstopb", "-parenb"]).output().map_err(|e| format!("cannot run stty: {e}"))?;
    if !stty.status.success() {
        return Err(format!("cannot set up {port}: {}", String::from_utf8_lossy(&stty.stderr).trim()));
    }
    let file = std::fs::OpenOptions::new().read(true).write(true).open(port).map_err(|e| format!("cannot open {port}: {e}"))?;
    let reader = file.try_clone().map_err(|e| e.to_string())?;
    Ok(Link::new(Box::new(reader), Box::new(file)))
}

/// One step's line sent, its report `R <k>,...` awaited and kept.
fn step(link: &mut Link, k: usize, input: &str, seconds: u64, reports: &mut Vec<String>) -> Result<(), String> {
    link.send(input)?;
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let prefix = format!("R {k},");
    loop {
        match link.line(deadline)? {
            Some(line) if line.starts_with(&prefix) => {
                reports.push(line);
                return Ok(());
            }
            Some(line) if line.starts_with("ARCLANG ERROR ") => return Err(format!("on the target, at step {k}: {}", &line["ARCLANG ERROR ".len()..])),
            Some(_) => {}
            None => return Err(format!("no report of step {k} within {seconds} s; the console:\n{}", tail(&link.log()))),
        }
    }
}

/// Each step's line, each step's report: the reports, as `pil` reads them.
fn steps(link: &mut Link, lines: &[String], seconds: u64) -> Result<String, String> {
    let mut reports = Vec::new();
    for (k, input) in lines.iter().enumerate() {
        step(link, k, input, seconds, &mut reports)?;
    }
    Ok(reports.iter().map(|r| format!("{r}\n")).collect())
}

/// Through the I/O: 'G' once, then the report of each of `steps` cycles
/// (the inputs are the simulated bench's, played in step with the cycles).
fn cycles(link: &mut Link, steps: usize, seconds: u64) -> Result<String, String> {
    link.send("G")?;
    let mut reports = String::new();
    for k in 0..steps {
        let deadline = Instant::now() + Duration::from_secs(seconds);
        let prefix = format!("R {k},");
        loop {
            match link.line(deadline)? {
                Some(line) if line.starts_with(&prefix) => {
                    reports.push_str(&line);
                    reports.push('\n');
                    break;
                }
                Some(line) if line.starts_with("ARCLANG ERROR ") => return Err(format!("on the target, at cycle {k}: {}", &line["ARCLANG ERROR ".len()..])),
                Some(_) => {}
                None => return Err(format!("no report of cycle {k} within {seconds} s; the console:\n{}", tail(&link.log()))),
            }
        }
    }
    Ok(reports)
}

/// What the bench sends: a line a step (the step() API), or 'G' and the
/// cycles through the I/O.
pub enum Drive<'a> {
    Lines(&'a [String]),
    Cycles(usize),
}

/// The simulator, killed when the bench is done with it.
struct Simulator(Child);

impl Drop for Simulator {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A target ready for its steps: the simulator (killed when the session
/// ends) or the board, its console linked; a loop sends one step at a time.
pub struct Session {
    // Held so the simulator dies with the session.
    _sim: Option<Simulator>,
    link: Link,
    step_seconds: u64,
}

impl Session {
    /// The simulator built in `gateware` started (a fresh reset), its console on TCP `port`, booted.
    pub fn simulate(gateware: &Path, port: u16, boot_seconds: u64, step_seconds: u64) -> Result<Session, String> {
        let (sim, link) = start(gateware, port)?;
        let mut session = Session { _sim: Some(sim), link, step_seconds };
        ready(&mut session.link, boot_seconds)?;
        Ok(session)
    }

    /// The board through its serial port, booted.
    pub fn board(port: &str, step_seconds: u64) -> Result<Session, String> {
        let link = open_port(port)?;
        let mut session = Session { _sim: None, link, step_seconds };
        ready(&mut session.link, 20)?;
        Ok(session)
    }

    /// One step: its line of inputs, its report `R <k>,...`.
    pub fn step(&mut self, k: usize, input: &str) -> Result<String, String> {
        let mut one = Vec::new();
        step(&mut self.link, k, input, self.step_seconds, &mut one)?;
        Ok(one.join("\n"))
    }

    /// Everything the console said.
    pub fn log(&self) -> String {
        self.link.log()
    }
}

/// The simulator of `gateware` started, its console reached.
fn start(gateware: &Path, port: u16) -> Result<(Simulator, Link), String> {
    let log = std::fs::File::create(gateware.join("vsim.log")).map_err(|e| format!("cannot write the simulator's log: {e}"))?;
    let child = Command::new(gateware.join("obj_dir").join("Vsim"))
        .current_dir(gateware)
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log)
        .spawn()
        .map_err(|e| format!("cannot start the simulator {}: {e}", gateware.join("obj_dir/Vsim").display()))?;
    let mut sim = Simulator(child);
    let deadline = Instant::now() + Duration::from_secs(30);
    let stream = loop {
        match std::net::TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => break s,
            Err(e) => {
                if let Ok(Some(status)) = sim.0.try_wait() {
                    return Err(format!("the simulator stopped ({status}): {}", tail(&std::fs::read_to_string(gateware.join("vsim.log")).unwrap_or_default())));
                }
                if Instant::now() >= deadline {
                    return Err(format!("cannot reach the simulator's console on port {port}: {e}"));
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    };
    let _ = stream.set_nodelay(true);
    let reader = stream.try_clone().map_err(|e| e.to_string())?;
    Ok((sim, Link::new(Box::new(reader), Box::new(stream))))
}

/// Run the steps in the simulator built in `gateware` (a fresh reset), its
/// console on TCP `port`: the reports, and everything the console said.
pub fn simulate(gateware: &Path, port: u16, drive: Drive, boot_seconds: u64, step_seconds: u64) -> Result<(String, String), String> {
    let (sim, mut link) = start(gateware, port)?;
    let sim = sim;
    let wanted = match drive {
        Drive::Cycles(n) => n,
        Drive::Lines(_) => 0,
    };
    let reports = ready(&mut link, boot_seconds).and_then(|_| match drive {
        Drive::Lines(lines) => steps(&mut link, lines, step_seconds),
        Drive::Cycles(n) => cycles(&mut link, n, step_seconds),
    });
    if reports.is_ok() && wanted > 0 {
        // The bench records a cycle's channels just after its report: wait
        // for the last one before the simulator stops.
        let deadline = Instant::now() + Duration::from_secs(step_seconds);
        let recorded = || std::fs::read_to_string(gateware.join("recorder.txt")).map(|t| t.lines().filter(|l| l.starts_with("S ")).count()).unwrap_or(0);
        while recorded() < wanted && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    drop(sim);
    reports.map(|r| (r, link.log()))
}

/// Run the steps on a board through its serial port (115200 bauds, 8N1,
/// raw), the board holding the SoC's bitstream.
pub fn board(port: &str, drive: Drive, step_seconds: u64) -> Result<(String, String), String> {
    let mut link = open_port(port)?;
    ready(&mut link, 20)?;
    let reports = match drive {
        Drive::Lines(lines) => steps(&mut link, lines, step_seconds)?,
        Drive::Cycles(n) => cycles(&mut link, n, step_seconds)?,
    };
    Ok((reports, link.log()))
}
