//! The proof that code written by hand is a machine of the model, as a
//! `correspondence` says (`arclang prove`). The proofs need CBMC and
//! Bitwuzla: they are ignored by default and run in ArcLang's formal image:
//!   tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored

use std::path::Path;
use std::process::Output;

fn arclang(args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs")
}

fn text(output: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

/// A sketch and its model in a fresh directory, each after the edits given
/// (what is written, what replaces it); the proof is then asked.
fn proof(name: &str, in_model: &[(&str, &str)], in_sketch: &[(&str, &str)]) -> (bool, String) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let dir = tempfile::tempdir().unwrap();
    for (extension, edits) in [("arc", in_model), ("ino", in_sketch)] {
        let mut content = std::fs::read_to_string(fixtures.join(format!("{name}.{extension}"))).unwrap();
        for (from, to) in edits {
            assert!(content.contains(from), "{name}.{extension} has `{from}`");
            content = content.replacen(from, to, 1);
        }
        std::fs::write(dir.path().join(format!("{name}.{extension}")), content).unwrap();
    }
    let asked = arclang(&["prove", dir.path().join(format!("{name}.arc")).to_str().unwrap(), "--timeout", "1200"]);
    (asked.status.success(), text(&asked))
}

const NEEDS: &str = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored";

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_sketch_is_proven_to_be_the_machine_its_correspondence_says() {
    let _ = NEEDS;
    for (name, said) in [
        ("thermostat", "thermostat.ino is state_machine \"Thermostat\""),
        ("toggle", "toggle.ino is state_machine \"Lamp\""),
        ("debounce", "debounce.ino is state_machine \"Debounced button\""),
    ] {
        let (proven, output) = proof(name, &[], &[]);
        assert!(proven && output.contains("✓ Proven") && output.contains(said), "{name}: {output}");
    }
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_model_that_is_not_what_the_sketch_does_is_refuted() {
    // The model lights the lamp when the button is released, the sketch when it is pressed.
    let (proven, output) = proof("toggle", &[("transition OffReleased -> OnHeld { guard: \"button\" }", "transition OffReleased -> OnHeld { guard: \"not button\" }")], &[]);
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
    // The model waits 40 ms, the sketch 50.
    let (proven, output) = proof("debounce", &[("transition Rising -> High { guard: \"button\" timing: 50 ms }", "transition Rising -> High { guard: \"button\" timing: 40 ms }")], &[]);
    assert!(!proven && output.contains("✗ Refuted") && output.contains("one loop() and one step can leave two that do not"), "{output}");
    assert!(output.contains("does not hold after it: stable == 1 && counting == 0 is in High"), "{output}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_sketch_changed_after_the_model_is_refuted() {
    // The sketch now lights the lamp on release.
    let (proven, output) = proof("toggle", &[], &[("if (now == 1 && was == 0) {", "if (now == 0 && was == 1) {")]);
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
    // The sketch starts with the lamp lit; the model with it dark: refuted at the start.
    let (proven, output) = proof("toggle", &[], &[("digitalWrite(LED, LOW);", "digitalWrite(LED, HIGH);")]);
    assert!(!proven && output.contains("after setup() and one loop(), and after init and one step, the code and the model do not correspond"), "{output}");
    assert!(output.contains("does not hold after it: digitalWrite(LED) is led"), "{output}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_correspondence_that_does_not_hold_is_refuted() {
    // Two states of the table exchanged.
    let (proven, output) = proof("toggle", &[("\"lit == 1 && was == 1 is in OnHeld\"", "\"lit == 1 && was == 1 is in OnReleased\""), ("\"lit == 1 && was == 0 is in OnReleased\"", "\"lit == 1 && was == 0 is in OnHeld\"")], &[]);
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_pin_the_correspondence_does_not_name_is_never_passed_over() {
    // The sketch drives a second pin the correspondence says nothing of.
    let (proven, output) = proof("toggle", &[], &[("    digitalWrite(LED, lit);", "    digitalWrite(LED, lit);\n    digitalWrite(7, lit);")]);
    assert!(!proven && output.contains("does not hold after it: the code reads and writes only the pins the correspondence names") && !output.contains("is in OnHeld"), "{output}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_sketch_is_proven_for_the_int_of_its_board() {
    // The same comparison written with a product: right with an int of 32 bits...
    let product = [("if (mode == 0 && t < 400) {", "if (mode == 0 && t * 100 < 40000) {")];
    let (proven, output) = proof("thermostat", &[], &product);
    assert!(proven && output.contains("an int of 32 bits"), "{output}");
    // ...and wrong on a board whose int has 16, where the product overflows.
    let (proven, output) = proof("thermostat", &[("period: 100 ms", "period: 100 ms\n    int: 16")], &product);
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
    // As written, the sketch is the machine on both.
    let (proven, output) = proof("thermostat", &[("period: 100 ms", "period: 100 ms\n    int: 16")], &[]);
    assert!(proven && output.contains("an int of 16 bits"), "{output}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_loop_that_waits_another_time_than_the_period_is_refuted() {
    let (proven, output) = proof("thermostat", &[], &[("delay(100);", "delay(250);")]);
    assert!(!proven && output.contains("does not hold after it: the delay()s of one loop(), when it has some, add up to the period"), "{output}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_variable_no_line_speaks_of_is_said_with_the_proof() {
    // The lamp is no longer related: what is left is proven, and the proof says what it leaves out.
    let (proven, output) = proof("thermostat", &[("writes: [\"digitalWrite(HEATER) is heater\"]", "writes: [\"digitalWrite(HEATER) is in Heating\"]")], &[]);
    assert!(proven && output.contains("the proof says nothing of this variable of the machine: no line of the correspondence speaks of heater"), "{output}");
}
