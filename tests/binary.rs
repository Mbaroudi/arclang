//! The proof that a firmware binary, read without its sources, is a machine
//! of the model (`arclang prove`, a correspondence that names a `binary`).
//! The proofs need Ghidra, CBMC and Bitwuzla: they are ignored by default
//! and run in ArcLang's reverse image:
//!   tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored

use std::path::Path;
use std::process::Output;

fn arclang(args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_arclang"))
        .args(args)
        .output()
        .expect("arclang runs")
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The bytes of an Intel HEX file that starts at address 0, and the file of some bytes.
fn bytes(hex: &str) -> Vec<u8> {
    let mut image = Vec::new();
    for line in hex
        .lines()
        .filter(|line| line.len() > 11 && &line[7..9] == "00")
    {
        image.extend(
            (0..(line.len() - 11) / 2)
                .map(|k| u8::from_str_radix(&line[9 + 2 * k..11 + 2 * k], 16).unwrap()),
        );
    }
    image
}

fn hex(image: &[u8]) -> String {
    let mut text = String::new();
    for (rank, chunk) in image.chunks(16).enumerate() {
        let mut record = vec![
            chunk.len() as u8,
            ((rank * 16) >> 8) as u8,
            (rank * 16) as u8,
            0,
        ];
        record.extend(chunk);
        record.push(record.iter().fold(0u8, |sum, byte| sum.wrapping_sub(*byte)));
        text.push_str(&format!(
            ":{}\n",
            record
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<String>()
        ));
    }
    text + ":00000001FF\n"
}

/// The lamp's binary and its model in a fresh directory, after the edits
/// given to the model and the bytes changed in the binary (those that are
/// there, once, and those that replace them); the proof is then asked.
fn proof(in_model: &[(&str, &str)], in_binary: Option<(&[u8], &[u8])>) -> (bool, String) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let dir = tempfile::tempdir().unwrap();
    let mut model = std::fs::read_to_string(fixtures.join("uno_lamp_binary.arc")).unwrap();
    for (from, to) in in_model {
        assert!(model.contains(from), "uno_lamp_binary.arc has `{from}`");
        model = model.replacen(from, to, 1);
    }
    std::fs::write(dir.path().join("uno_lamp_binary.arc"), model).unwrap();
    let mut image = bytes(&std::fs::read_to_string(fixtures.join("uno_lamp.hex")).unwrap());
    if let Some((from, to)) = in_binary {
        let places: Vec<usize> = (0..=image.len() - from.len())
            .filter(|at| image[*at..*at + from.len()] == *from)
            .collect();
        assert_eq!(
            places.len(),
            1,
            "the binary has the bytes {from:02X?} once"
        );
        image[places[0]..places[0] + to.len()].copy_from_slice(to);
    }
    std::fs::write(dir.path().join("uno_lamp.hex"), hex(&image)).unwrap();
    let asked = arclang(&[
        "prove",
        dir.path().join("uno_lamp_binary.arc").to_str().unwrap(),
        "--timeout",
        "1200",
    ]);
    (asked.status.success(), text(&asked))
}

#[test]
fn the_fixture_is_read_and_written_back_byte_for_byte() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let image = bytes(&std::fs::read_to_string(fixtures.join("uno_lamp.hex")).unwrap());
    assert_eq!(image.len(), 1388);
    assert_eq!(bytes(&hex(&image)), image);
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_binary_is_proven_to_be_the_machine_its_correspondence_says_without_its_sources() {
    let (proven, output) = proof(&[], None);
    assert!(
        proven
            && output.contains("✓ Proven")
            && output.contains("uno_lamp.hex is state_machine \"Lamp toggled by a button\""),
        "{output}"
    );
    // What the proof is of, and what it takes on trust, are said with it.
    assert!(
        output.contains("of the machine code of uno_lamp.hex itself")
            && output.contains("no source is read"),
        "{output}"
    );
    // The functions of the pins and millis are proven to be what they are said to be; what is left on trust is said.
    for proven_of in [
        "digitalRead at 0x0514 returns the level of the pin, for each of the pins 0 to 19",
        "digitalWrite at 0x04b6 sets the level of the pin and of no other",
        "pinMode at 0x043e makes the pin an input",
        "millis at 0x029c returns the four bytes of the memory at 0x010b",
        "taken on trust: these are the Arduino functions they are said to be, and are not run (delay at 0x02fe)",
        // The clock: the interrupt that keeps the time, the start that sets the timer, and no other interrupt.
        "the interrupt of timer 0 (at 0x0208) adds 1.024 to the four bytes millis returns at each overflow of the timer",
        "the start leaves timer 0 overflowing every 256 x 64 cycles of the clock (1.024 ms at 16 MHz)",
        "it is the only interrupt of the binary, and the code of loop() reads none of its bytes",
        "taken on trust: the clock of the board is 16 MHz",
    ] {
        assert!(output.contains(proven_of), "{proven_of}: {output}");
    }
    // The same verdict as the proof of its sketch, which the board confirmed: one loop late some of the time.
    assert!(
        output.contains("on the clock of the board") && output.contains("NOT proven"),
        "{output}"
    );
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_binary_changed_by_one_instruction_is_refuted() {
    // `sbiw r24, 0x32` (the 50 ms of the press) made `sbiw r24, 0x33`: the binary waits 51 ms.
    let (proven, output) = proof(&[], Some((&[0xC2, 0x97], &[0xC3, 0x97])));
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_correspondence_that_reads_the_wrong_bytes_of_the_memory_is_refuted() {
    let (proven, output) = proof(
        &[("word(0x0108) == 1 is led", "word(0x010a) == 1 is led")],
        None,
    );
    assert!(
        !proven && output.contains("✗ Refuted") && output.contains("word(0x010a) == 1 is led"),
        "{output}"
    );
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_pin_the_binary_writes_itself_is_never_passed_over() {
    // pinMode is not named: its machine code runs, and writes the direction of the port itself.
    let (proven, output) = proof(&[("        \"pinMode at 0x043e\",\n", "")], None);
    assert!(
        !proven
            && output.contains("✗ Refuted")
            && output.contains("writes no port of the chip itself"),
        "{output}"
    );
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn two_functions_taken_for_each_other_are_refuted() {
    let (proven, output) = proof(
        &[
            ("\"digitalRead at 0x0514\"", "\"digitalRead at 0x04b6\""),
            ("\"digitalWrite at 0x04b6\"", "\"digitalWrite at 0x0514\""),
        ],
        None,
    );
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
    // Before any proof of the correspondence: neither is the function it is said to be.
    assert!(
        output.contains("is not the Arduino function it is said to be")
            && output.contains("digitalRead at 0x04b6: it returns the bit of the pin in the input register of its port"),
        "{output}"
    );
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_function_whose_pin_table_is_not_an_unos_is_not_the_function_it_is_said_to_be() {
    // The table that gives the port of each pin, pins 12 to 19: pin 13 moved from port B to port C.
    let (proven, output) = proof(&[], Some((&[2, 2, 3, 3, 3, 3, 3, 3], &[2, 3, 3, 3, 3, 3, 3, 3])));
    assert!(
        !proven && output.contains("✗ Refuted") && output.contains("is not the Arduino function it is said to be"),
        "{output}"
    );
    for not_done in [
        "digitalWrite at 0x04b6: it sets the bit of the pin in the output register of its port to the level, and no other bit",
        "digitalRead at 0x0514: it returns the bit of the pin in the input register of its port",
    ] {
        assert!(output.contains(not_done), "{not_done}: {output}");
    }
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_binary_that_does_not_keep_the_time_as_the_arduino_core_does_is_refuted() {
    // In the interrupt of timer 0, `cpi r18, 125` made `cpi r18, 126`: the milliseconds drift.
    let (proven, output) = proof(&[], Some((&[0x23, 0x0f, 0x2d, 0x37], &[0x23, 0x0f, 0x2e, 0x37])));
    assert!(
        !proven && output.contains("✗ Refuted") && output.contains("does not keep the time as the Arduino core does"),
        "{output}"
    );
    assert!(output.contains("the interrupt of timer 0, at 0x0208: it adds 1 to the four bytes at 0x010b"), "{output}");
    // In the start, the second bit of the prescaler of timer 0 is not set: it counts one cycle in 8, not in 64.
    let (proven, output) = proof(&[], Some((&[0x85, 0xb5, 0x81, 0x60, 0x85, 0xbd], &[0x85, 0xb5, 0x80, 0x60, 0x85, 0xbd])));
    assert!(
        !proven && output.contains("✗ Refuted") && output.contains("the start of the binary: it leaves timer 0 counting one in 64 cycles of the clock"),
        "{output}"
    );
}

#[test]
fn a_binary_without_ghidra_says_what_is_missing() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = std::process::Command::new(env!("CARGO_BIN_EXE_arclang"))
        .args([
            "prove",
            fixtures.join("uno_lamp_binary.arc").to_str().unwrap(),
        ])
        .env("ARCLANG_GHIDRA", "/nowhere/analyzeHeadless")
        .output()
        .unwrap();
    let output = text(&asked);
    assert!(
        !asked.status.success()
            && output.contains("Ghidra could not be run")
            && output.contains("ARCLANG_GHIDRA"),
        "{output}"
    );
}

#[test]
fn the_compiler_checks_a_correspondence_that_names_a_binary() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let model = std::fs::read_to_string(fixtures.join("uno_lamp_binary.arc")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let check = |edited: String| {
        let path = dir.path().join("m.arc");
        std::fs::write(&path, edited).unwrap();
        let asked = arclang(&["check", path.to_str().unwrap()]);
        (asked.status.success(), text(&asked))
    };
    assert!(check(model.clone()).0, "{}", check(model.clone()).1);
    let (ok, output) = check(model.replacen("\"loop at 0x0102\",", "", 1));
    assert!(
        !ok && output.contains("does not say where loop is in the binary"),
        "{output}"
    );
    let (ok, output) = check(model.replacen(
        "binary: \"uno_lamp.hex\"",
        "binary: \"uno_lamp.hex\"\n    source: \"uno_lamp.ino\"",
        1,
    ));
    assert!(!ok && output.contains("a source and a binary"), "{output}");
    let (ok, output) = check(model.replacen(
        "\"word(0x0108) == 1 is led\"",
        "\"word(0x0108) == 1 is in Nowhere\"",
        1,
    ));
    assert!(!ok && output.contains("has no state 'Nowhere'"), "{output}");
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn the_functions_the_pins_and_the_period_of_a_binary_are_recovered_from_its_machine_code() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = arclang(&["recover", fixtures.join("uno_lamp.hex").to_str().unwrap()]);
    let output = text(&asked);
    assert!(asked.status.success(), "{output}");
    // Every function the proven correspondence names is proposed at the same address.
    let proven = std::fs::read_to_string(fixtures.join("uno_lamp_binary.arc")).unwrap();
    let placed: Vec<&str> = proven.lines().map(str::trim).filter(|line| line.starts_with('"') && line.contains(" at 0x")).map(|line| line.trim_end_matches(',')).collect();
    assert_eq!(placed.len(), 6, "{proven}");
    for function in placed {
        assert!(output.contains(function), "{function} is proposed: {output}");
    }
    assert!(output.contains("period: 10 ms") && output.contains("reads: [\"digitalRead(2) is ...\"]") && output.contains("writes: [\"digitalWrite(13) is ...\"]"), "{output}");
    assert!(output.contains("// setup() calls: pinMode(2, 2), pinMode(13, 1), digitalWrite(13, 0)"), "{output}");
    assert!(output.contains("0x0100 to 0x0109 (10 bytes)") && output.contains("a proposal, not a proof"), "{output}");
}

#[test]
fn recovering_a_binary_without_ghidra_says_what_is_missing() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = std::process::Command::new(env!("CARGO_BIN_EXE_arclang")).args(["recover", fixtures.join("uno_lamp.hex").to_str().unwrap()]).env("ARCLANG_GHIDRA", "/nowhere/analyzeHeadless").output().unwrap();
    let output = text(&asked);
    assert!(!asked.status.success() && output.contains("Ghidra could not be run"), "{output}");
}

/// A proposer that answers from files, one a turn, and keeps the prompts it was given.
fn proposer(answers: &[String]) -> (String, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    for (turn, answer) in answers.iter().enumerate() {
        std::fs::write(dir.path().join(format!("answer-{turn}")), answer).unwrap();
    }
    let at = dir.path().display();
    let command = format!("n=$(cat '{at}/count' 2>/dev/null || echo 0); echo $((n+1)) > '{at}/count'; cat > '{at}/prompt-'$n; cat '{at}/answer-'$n");
    (command, dir)
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_model_proposed_for_a_binary_is_written_only_once_proven_against_its_machine_code() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let right = std::fs::read_to_string(fixtures.join("uno_lamp_binary.arc")).unwrap();
    // First a machine that waits 40 ms where the binary waits 50, then one that names a function of its own, then the right one.
    let wrong = right.replacen("transition Arming -> Down { guard: \"not button\" timing: 50 ms", "transition Arming -> Down { guard: \"not button\" timing: 40 ms", 1);
    let moved = right.replacen("\"millis at 0x029c\"", "\"millis at 0x02b4\"", 1);
    assert!(wrong != right && moved != right);
    let fence = |model: &str| format!("Here is the model.\n```arclang\n{model}```\n");
    let (command, kept) = proposer(&[fence(&wrong), fence(&moved), fence(&right)]);
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixtures.join("uno_lamp.hex"), dir.path().join("uno_lamp.hex")).unwrap();
    let written = dir.path().join("recovered.arc");
    let asked = arclang(&["recover", dir.path().join("uno_lamp.hex").to_str().unwrap(), "--propose", "--with", &command, "--turns", "3", "--timeout", "1200", "-o", written.to_str().unwrap()]);
    let output = text(&asked);
    assert!(asked.status.success() && output.contains("✓ Proven") && output.contains("of the machine code of uno_lamp.hex itself"), "{output}");
    assert!(std::fs::read_to_string(&written).unwrap().contains("timing: 50 ms"));
    // The proposer was given what was found in the machine code and the listing of loop(), no source.
    let first = std::fs::read_to_string(kept.path().join("prompt-0")).unwrap();
    assert!(first.contains("\"loop at 0x0102\"") && first.contains("; loop at 0x0102") && first.contains("call 0x514    ; digitalRead"), "{first}");
    assert!(!first.contains("void loop()"), "{first}");
    // What was refuted went back to it, and so did the function it moved.
    let second = std::fs::read_to_string(kept.path().join("prompt-1")).unwrap();
    assert!(second.contains("STEP is refuted") && second.contains("timing: 40 ms"), "{second}");
    let third = std::fs::read_to_string(kept.path().join("prompt-2")).unwrap();
    assert!(third.contains("they must be exactly those found in the machine code"), "{third}");
}
