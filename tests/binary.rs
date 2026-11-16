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
        "the interrupt of timer 0 (at 0x0208) adds 1.024 to the count of milliseconds (the four bytes at 0x010b, which millis returns)",
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

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_binary_that_times_with_micros_is_proven_and_so_is_its_micros() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = arclang(&["prove", fixtures.join("uno_lamp_micros_binary.arc").to_str().unwrap(), "--timeout", "1200"]);
    let output = text(&asked);
    assert!(asked.status.success() && output.contains("✓ Proven"), "{output}");
    for said in [
        "micros at 0x029e returns the time in microseconds of a 16 MHz clock: 4 a count of timer 0, 1024 a turn (the four bytes of the memory at 0x010f)",
        // The clock is proven of it too: the turns micros reads are those the interrupt counts.
        "and 1 to its turns (the four bytes at 0x010f, which micros reads) at each overflow of the timer",
        "the start leaves timer 0 overflowing every 256 x 64 cycles of the clock",
        // As for its sketch: timed with micros, it is never a loop late on the clock of the board.
        "on the clock of the board (an AVR's millis() counts ticks of 1.024 ms: a time read can be 1 ms short): proven too",
    ] {
        assert!(output.contains(said), "{said}: {output}");
    }
}

#[test]
#[ignore = "needs Ghidra: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_binary_built_with_link_time_optimisation_has_its_cycle_placed_and_no_function_named() {
    // The lamp built as the Arduino IDE builds it: setup() and loop() are folded into main.
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = arclang(&["recover", fixtures.join("uno_lamp_lto.hex").to_str().unwrap()]);
    let output = text(&asked);
    assert!(asked.status.success(), "{output}");
    // The head the proven correspondence places, the state it keeps without the clock, the step of its time.
    let proven = std::fs::read_to_string(fixtures.join("uno_lamp_lto_binary.arc")).unwrap();
    assert!(proven.contains("functions: [\"cycle at 0x0390\"]"));
    for said in [
        "functions: [\"cycle at 0x0390\"]",
        "tick: 1000 us",
        "0x0100 to 0x0109 (10 bytes) (the clock, kept by the interrupt of timer 0, is left out)",
        "found: main (0x0272) has one loop it never leaves (134 instructions), entered at 0x0390 only: that is the head of the cycle",
        "the pins and the period are not found",
    ] {
        assert!(output.contains(said), "{said}: {output}");
    }
    assert!(!output.contains("loop at 0x") && !output.contains("digitalRead at 0x"), "no function is named: {output}");
}

#[test]
fn a_cycle_says_the_step_of_its_time_and_is_checked_before_any_proof() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let model = std::fs::read_to_string(fixtures.join("uno_lamp_lto_binary.arc")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let check = |edited: String| {
        let path = dir.path().join("m.arc");
        std::fs::write(&path, edited).unwrap();
        let asked = arclang(&["check", path.to_str().unwrap()]);
        (asked.status.success(), text(&asked))
    };
    assert!(check(model.clone()).0);
    let (ok, output) = check(model.replacen("    tick: 1000 us\n", "", 1));
    assert!(!ok && output.contains("a cycle says the step of its time"), "{output}");
    let (ok, output) = check(model.replacen("1000 us", "250 us", 1));
    assert!(!ok && output.contains("a multiple of 4 us"), "{output}");
    let (ok, output) = check(model.replacen("\"cycle at 0x0390\"", "\"cycle at 0x0390\", \"millis at 0x029c\"", 1));
    assert!(!ok && output.contains("only the cycle is placed"), "{output}");
    let (ok, output) = check(model.replacen("digitalRead(2) is button", "analogRead(2) is button", 1));
    assert!(!ok || output.contains("button"), "{output}");
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla (about 10 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_binary_built_as_the_arduino_ide_builds_it_is_proven_with_no_function_named() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = arclang(&["prove", fixtures.join("uno_lamp_lto_binary.arc").to_str().unwrap(), "--timeout", "1800"]);
    let output = text(&asked);
    assert!(asked.status.success() && output.contains("✓ Proven"), "{output}");
    for said in [
        "the whole of it is run, from its reset: no function is stood in for",
        "1000 us pass each time the code has read the timer",
        "the interrupt of timer 0 (at 0x01de) adds 1.024 to the count of milliseconds",
    ] {
        assert!(output.contains(said), "{said}: {output}");
    }
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla (about 10 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_pwm_pin_of_an_optimised_binary_is_proven_by_the_registers_of_its_timer_and_a_wrong_duty_refuted() {
    // analogWrite(9, 128) on a press, analogWrite(9, 0) otherwise: the value of the pin is what timer 1 gives.
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = arclang(&["prove", fixtures.join("uno_pwm_lto_binary.arc").to_str().unwrap(), "--timeout", "1800"]);
    let output = text(&asked);
    assert!(asked.status.success() && output.contains("✓ Proven"), "{output}");
    for said in [
        "the whole of it is run, from its reset: no function is stood in for",
        "taken on trust: pin 9 is driven by the timer when it is connected to it, by the level of the pin otherwise",
    ] {
        assert!(output.contains(said), "{said}: {output}");
    }
    // The machine has one state and no line names it: it is in it, and a model that says another duty is refuted.
    let dir = tempfile::tempdir().unwrap();
    let model = std::fs::read_to_string(fixtures.join("uno_pwm_lto_binary.arc")).unwrap();
    std::fs::write(dir.path().join("m.arc"), model.replacen("duty := 128", "duty := 127", 1)).unwrap();
    std::fs::copy(fixtures.join("uno_pwm_lto.hex"), dir.path().join("uno_pwm_lto.hex")).unwrap();
    let asked = arclang(&["prove", dir.path().join("m.arc").to_str().unwrap(), "--timeout", "1800"]);
    let output = text(&asked);
    assert!(!asked.status.success() && output.contains("✗ Refuted") && output.contains("`the value written to 9` and `duty`: after 128 and 127"), "{output}");
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla (about 10 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn an_optimised_binary_that_prints_a_text_on_the_serial_port_is_proven_and_a_wrong_line_refuted() {
    // A lamp toggled by a button, that says "on" or "off" through Print::write and the vtable of the serial port.
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = arclang(&["prove", fixtures.join("uno_serial_text_lto_binary.arc").to_str().unwrap(), "--timeout", "1800"]);
    let output = text(&asked);
    assert!(asked.status.success() && output.contains("✓ Proven"), "{output}");
    for said in [
        "the whole of it is run, from its reset: no function is stood in for",
        "taken on trust: the serial port is not looked at, what the code prints included; its transmit buffer is empty at the head of each turn",
    ] {
        assert!(output.contains(said), "{said}: {output}");
    }
    let dir = tempfile::tempdir().unwrap();
    let model = std::fs::read_to_string(fixtures.join("uno_serial_text_lto_binary.arc")).unwrap();
    std::fs::write(dir.path().join("m.arc"), model.replacen("word(0x01b9) == 1 is led", "word(0x01b9) == 0 is led", 1)).unwrap();
    std::fs::copy(fixtures.join("uno_serial_text_lto.hex"), dir.path().join("uno_serial_text_lto.hex")).unwrap();
    let asked = arclang(&["prove", dir.path().join("m.arc").to_str().unwrap(), "--timeout", "1800"]);
    let output = text(&asked);
    assert!(!asked.status.success() && output.contains("✗ Refuted") && output.contains("does not hold after it: word(0x01b9) == 0 is led"), "{output}");
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla (about 20 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn an_optimised_dimmer_with_a_pwm_pin_and_a_serial_port_is_proven_and_a_wrong_step_refuted() {
    // More than a quarter of an hour of proof: run when asked (ARCLANG_LONG_PROOFS=1), not at every change.
    if std::env::var_os("ARCLANG_LONG_PROOFS").is_none() {
        eprintln!("skipped: set ARCLANG_LONG_PROOFS=1 to run this proof");
        return;
    }
    // digitalRead(3) disconnects timer 2, which could drive that pin: a write of its setting that changes nothing,
    // the settings of the timers being at the head of each turn as the start leaves them.
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = arclang(&["prove", fixtures.join("uno_dimmer_lto_binary.arc").to_str().unwrap(), "--timeout", "2400"]);
    let output = text(&asked);
    assert!(asked.status.success() && output.contains("✓ Proven"), "{output}");
    let dir = tempfile::tempdir().unwrap();
    let model = std::fs::read_to_string(fixtures.join("uno_dimmer_lto_binary.arc")).unwrap();
    std::fs::write(dir.path().join("m.arc"), model.replacen("level := min(level + 85, 255)", "level := min(level + 84, 255)", 1)).unwrap();
    std::fs::copy(fixtures.join("uno_dimmer_lto.hex"), dir.path().join("uno_dimmer_lto.hex")).unwrap();
    let asked = arclang(&["prove", dir.path().join("m.arc").to_str().unwrap(), "--timeout", "2400"]);
    let output = text(&asked);
    assert!(!asked.status.success() && output.contains("✗ Refuted") && output.contains("`the value written to 9` and `level`: after 85 and 84"), "{output}");
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla (about 25 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn an_optimised_binary_that_prints_a_number_on_the_serial_port_is_proven() {
    // More than a quarter of an hour of proof: run when asked (ARCLANG_LONG_PROOFS=1), not at every change.
    if std::env::var_os("ARCLANG_LONG_PROOFS").is_none() {
        eprintln!("skipped: set ARCLANG_LONG_PROOFS=1 to run this proof");
        return;
    }
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = arclang(&["prove", fixtures.join("uno_serial_lto_binary.arc").to_str().unwrap(), "--timeout", "2400"]);
    let output = text(&asked);
    assert!(asked.status.success() && output.contains("✓ Proven"), "{output}");
}

#[test]
#[ignore = "needs Ghidra, CBMC and Bitwuzla (about 10 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_correspondence_that_reads_the_wrong_bytes_of_an_optimised_binary_is_refuted() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let dir = tempfile::tempdir().unwrap();
    let model = std::fs::read_to_string(fixtures.join("uno_lamp_lto_binary.arc")).unwrap();
    std::fs::write(dir.path().join("m.arc"), model.replacen("word(0x0100) == 1 is led", "word(0x010a) == 1 is led", 1)).unwrap();
    std::fs::copy(fixtures.join("uno_lamp_lto.hex"), dir.path().join("uno_lamp_lto.hex")).unwrap();
    let asked = arclang(&["prove", dir.path().join("m.arc").to_str().unwrap(), "--timeout", "1800"]);
    let output = text(&asked);
    // The lines of a cycle are not asked one by one (each question takes minutes): the case says which do not hold.
    assert!(!asked.status.success() && output.contains("✗ Refuted"), "{output}");
    assert!(output.contains("does not hold after it: word(0x010a) == 1 is led"), "the line that fails is named: {output}");
    assert!(!output.contains("does not hold after it: word(0x0108) == 0 && word(0x0106) == 0 is in Up"), "a line that holds is not named: {output}");
}

#[test]
#[ignore = "needs Ghidra: tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_proposal_for_a_cycle_cannot_move_the_head_or_change_the_step_and_is_refused_before_any_proof() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let right = std::fs::read_to_string(fixtures.join("uno_lamp_lto_binary.arc")).unwrap();
    let moved = right.replacen("cycle at 0x0390", "cycle at 0x0498", 1);
    let finer = right.replacen("tick: 1000 us", "tick: 200 us", 1);
    assert!(moved != right && finer != right);
    let fence = |model: &str| format!("```arclang\n{model}```\n");
    let (command, kept) = proposer(&[fence(&moved), fence(&finer), fence(&moved)]);
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixtures.join("uno_lamp_lto.hex"), dir.path().join("uno_lamp_lto.hex")).unwrap();
    let written = dir.path().join("recovered.arc");
    let asked = arclang(&["recover", dir.path().join("uno_lamp_lto.hex").to_str().unwrap(), "--propose", "--with", &command, "--turns", "3", "-o", written.to_str().unwrap()]);
    let output = text(&asked);
    assert!(!asked.status.success() && !written.exists() && output.contains("no model of uno_lamp_lto.hex was proven"), "{output}");
    // The proposer was given the cycle, the step and the listing of the code, and no source.
    let first = std::fs::read_to_string(kept.path().join("prompt-0")).unwrap();
    assert!(first.contains("functions: [\"cycle at 0x0390\"]") && first.contains("tick: 1000 us") && first.contains("0x0100 to 0x0109"), "{first}");
    assert!(first.contains("; a function they call at 0x0272") || first.contains("0272"), "{first}");
    assert!(first.contains("built with link-time optimisation"), "{first}");
    let second = std::fs::read_to_string(kept.path().join("prompt-1")).unwrap();
    assert!(second.contains("must be exactly `functions: [\"cycle at 0x0390\"]`"), "{second}");
    let third = std::fs::read_to_string(kept.path().join("prompt-2")).unwrap();
    assert!(third.contains("it must be `tick: 1000 us`"), "{third}");
}

#[test]
fn a_step_that_is_a_block_of_a_binary_is_checked_before_any_proof() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let model = std::fs::read_to_string(fixtures.join("uno_lock_block.arc")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let check = |edited: String| {
        let path = dir.path().join("m.arc");
        std::fs::write(&path, edited).unwrap();
        let asked = arclang(&["check", path.to_str().unwrap()]);
        (asked.status.success(), text(&asked))
    };
    assert!(check(model.clone()).0, "{}", check(model.clone()).1);
    // The block is of the cycle, and of every machine: no machine, no pin and no time are named with it.
    let (ok, output) = check(model.replacen("\"cycle at 0x0fc6\", ", "", 1));
    assert!(!ok && output.contains("the functions say where the cycle is too"), "{output}");
    let (ok, output) = check(model.replacen("    binary: \"uno_lock.hex\"\n", "    binary: \"uno_lock.hex\"\n    machine: \"Lock modes\"\n", 1));
    assert!(!ok && output.contains("not 'machine'"), "{output}");
    let (ok, output) = check(model.replacen("\"step from 0x1026 to 0x1294\"", "\"step at 0x1026\"", 1));
    assert!(!ok && output.contains("does not say where the step is"), "{output}");
    // The structures are those the model generates, where the correspondence places them.
    let (ok, output) = check(model.replacen(", \"outputs at 0x0409\"", "", 1));
    assert!(!ok && output.contains("where the outputs of the step are"), "{output}");
    let (ok, output) = check(model.replacen("\"state at 0x0439\"", "\"state at 0x08f0\"", 1));
    assert!(!ok && output.contains("the state (43 bytes at 0x08f0) is not within the memory of the chip"), "{output}");
    let (ok, output) = check(model.replacen("\"inputs at 0x0411\"", "\"inputs at 0x0430\"", 1));
    assert!(!ok && output.contains("share bytes of the memory"), "{output}");
}

/// The lock's binary and its model in a fresh directory, after an edit of the model; the proof is then asked.
fn block_proof(from: &str, to: &str) -> (bool, String) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let dir = tempfile::tempdir().unwrap();
    let model = std::fs::read_to_string(fixtures.join("uno_lock_block.arc")).unwrap();
    assert!(from.is_empty() || model.contains(from), "uno_lock_block.arc has `{from}`");
    std::fs::write(dir.path().join("uno_lock_block.arc"), model.replacen(from, to, 1)).unwrap();
    std::fs::copy(fixtures.join("uno_lock.hex"), dir.path().join("uno_lock.hex")).unwrap();
    let asked = arclang(&["prove", dir.path().join("uno_lock_block.arc").to_str().unwrap(), "--timeout", "900"]);
    (asked.status.success(), text(&asked))
}

#[test]
#[ignore = "needs Ghidra and CBMC (about 2 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn the_step_of_three_machines_folded_into_main_is_proven_on_the_memory_of_the_binary() {
    // The firmware read back from an Arduino UNO: the step the model generates is a run of instructions of main.
    let (ok, output) = block_proof("", "");
    assert!(ok && output.contains("✓ Proven"), "{output}");
    for said in [
        "the block from 0x1026 to 0x1294 of uno_lock.hex is one step of the machines of the model",
        "the state (43 bytes at 0x0439), the inputs (40 bytes at 0x0411), the outputs (8 bytes at 0x0409)",
        "writes no byte of the memory outside the state, the outputs and the 64 bytes under the stack pointer",
        "taken, not proven here: at the first instruction of the block the stack pointer (2301) and the registers the cycle keeps (r1 = 0,",
        "NOT said: when the block runs",
    ] {
        assert!(output.contains(said), "{said}: {output}");
    }
    // What a proof of a cycle says of its time and its pins is not said of a block.
    assert!(!output.contains("for one loop() a step of"), "{output}");
}

#[test]
#[ignore = "needs Ghidra and CBMC (about 2 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_model_whose_guard_is_not_the_one_of_the_binary_is_refuted_on_the_machine_it_changes() {
    let (ok, output) = block_proof("trigger: \"SF-2\" guard: \"door closed\"", "trigger: \"SF-2\" guard: \"credential accepted\"");
    assert!(!ok && output.contains("✗ Refuted"), "{output}");
    assert!(output.contains("from some memory, the block and one step of the machines of the model do not leave the same"), "{output}");
    assert!(output.contains("does not hold after it: state: lock_modes.active[0] (0x044a) is what the step of the model leaves"), "{output}");
    // The machine the edit does not touch is still the one of the binary.
    assert!(!output.contains("state: lock_lifecycle."), "{output}");
}

#[test]
#[ignore = "needs Ghidra and CBMC (about 2 minutes): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn a_block_that_runs_on_past_the_step_is_refuted_by_what_it_writes_besides() {
    // One instruction more: the sketch empties the queue of events after the step, which a step does not do.
    let (ok, output) = block_proof("step from 0x1026 to 0x1294", "step from 0x1026 to 0x1298");
    assert!(!ok && output.contains("✗ Refuted"), "{output}");
    assert!(output.contains("does not hold after it: the block writes no byte of the memory outside the state, the outputs and the 64 bytes under the stack pointer"), "{output}");
    assert!(!output.contains("does not hold after it: state:") && !output.contains("does not hold after it: outputs:"), "{output}");
    // A block that is not of the function of the cycle is refused, with the address.
    let (ok, output) = block_proof("step from 0x1026 to 0x1294", "step from 0x1026 to 0x1d00");
    assert!(!ok && output.contains("0x1d00, the second address of the step, is not an instruction of the function of the cycle (0x0d5c)"), "{output}");
}

#[test]
#[ignore = "needs Ghidra (about 1 minute): tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored"]
fn the_step_of_a_model_and_its_structures_are_found_in_a_binary_without_its_symbols() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let model = fixtures.join("uno_lock_block.arc");
    let asked = arclang(&["recover", fixtures.join("uno_lock.hex").to_str().unwrap(), "--model", model.to_str().unwrap()]);
    let output = text(&asked);
    assert!(asked.status.success(), "{output}");
    // What is proposed is, line for line, the correspondence the fixture holds and the proof proves.
    let proven = std::fs::read_to_string(&model).unwrap();
    for line in ["    binary: \"uno_lock.hex\"\n", "    functions: [\"cycle at 0x0fc6\", \"step from 0x1026 to 0x1294\"]\n", "    memory: [\"state at 0x0439\", \"inputs at 0x0411\", \"outputs at 0x0409\"]\n"] {
        assert!(proven.contains(line) && output.contains(line), "{line}: {output}");
    }
    for said in [
        "the state of the step is at 0x0439 (43 bytes): the only place where the function of the cycle (0x0d5c) reads and writes, by address, the flags of the 3 machine(s) and reads those of their 8 state(s)",
        "the step starts at 0x1026, the first read of the flag of its first machine, and ends before 0x1294, after the last write of its outputs (at 0x1290)",
        "the inputs are at 0x0411 (40 bytes): 8 of their 8 scalars are read there by address",
        "a proposal, not a proof",
    ] {
        assert!(output.contains(said), "{said}: {output}");
    }
    // A model the binary was not built from: its step is said not to be found, and nothing is proposed.
    let other = fixtures.join("uno_lamp_lto_binary.arc");
    let asked = arclang(&["recover", fixtures.join("uno_lock.hex").to_str().unwrap(), "--model", other.to_str().unwrap()]);
    let output = text(&asked);
    assert!(!asked.status.success() && output.contains("the step of the model is not found") && !output.contains("correspondence \""), "{output}");
}
