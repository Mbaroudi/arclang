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
/// given to the model and the bytes changed in the binary (the two bytes
/// that are there, the two that replace them); the proof is then asked.
fn proof(in_model: &[(&str, &str)], in_binary: Option<([u8; 2], [u8; 2])>) -> (bool, String) {
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
        let places: Vec<usize> = (0..image.len() - 1)
            .step_by(2)
            .filter(|at| image[*at..*at + 2] == from)
            .collect();
        assert_eq!(
            places.len(),
            1,
            "the binary has the instruction {from:02X?} once"
        );
        image[places[0]..places[0] + 2].copy_from_slice(&to);
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
    assert!(
        output.contains("taken on trust: these are the Arduino functions they are said to be")
            && output.contains("digitalRead at 0x0514"),
        "{output}"
    );
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
    let (proven, output) = proof(&[], Some(([0xC2, 0x97], [0xC3, 0x97])));
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
