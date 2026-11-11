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
        // Two inputs, two outputs and a timer the model counts in a variable.
        ("door", "door.ino is state_machine \"Door lock\""),
        // Proven for an int of 16 bits, then flashed on a real UNO.
        ("uno_lamp", "uno_lamp.ino is state_machine \"Lamp toggled by a button\""),
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
    // The case that shows it: at 40 ms the model is in High, the sketch still counts.
    assert!(output.contains("with button = 1") && output.contains("`stable == 1 && counting == 0` and `in High`: before 0 and 0, after 0 and 1  <- differ"), "{output}");
    assert!(output.contains("`millis() - since` and `time in Rising`: before 40 and 40, after 50 and -"), "{output}");
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
    assert!(proven && output.contains("an int of 32 bits and a long of 32"), "{output}");
    // ...and wrong on a board whose int has 16, where the product overflows.
    let (proven, output) = proof("thermostat", &[("period: 100 ms", "period: 100 ms\n    int: 16")], &product);
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
    // As written, the sketch is the machine on both.
    let (proven, output) = proof("thermostat", &[("period: 100 ms", "period: 100 ms\n    int: 16")], &[]);
    assert!(proven && output.contains("an int of 16 bits"), "{output}");
    // It reads no clock: on the clock of the board it is proven too.
    assert!(output.contains("on the clock of the board") && output.contains("proven too"), "{output}");
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

/// A proposer that gives, turn after turn, the answers written for it, and
/// keeps each prompt it was given: (its command, the directory of its files).
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
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_proposed_model_is_written_only_once_proven_and_what_is_refuted_goes_back_to_the_proposer() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let right = std::fs::read_to_string(fixtures.join("toggle.arc")).unwrap();
    // A table that says too little: neither how the lamp follows `lit`, nor what the variables can be.
    let poor = right.replacen(",\n        \"lit == 1 is led\"", "", 1).replacen("    invariant: [\"lit == 0 || lit == 1\", \"was == 0 || was == 1\"]\n", "", 1);
    assert_ne!(poor, right);
    let fenced = |model: &str| format!("Here is the model.\n```arclang\n{model}```\n");
    let work = tempfile::tempdir().unwrap();
    let sketch = work.path().join("toggle.ino");
    std::fs::copy(fixtures.join("toggle.ino"), &sketch).unwrap();
    let recovered = work.path().join("toggle.arc");

    // The poor table first, the right one next: proven at the second turn.
    let (command, files) = proposer(&[fenced(&poor), fenced(&right)]);
    let asked = arclang(&["recover", sketch.to_str().unwrap(), "--propose", "--with", &command, "--timeout", "1200"]);
    let said = text(&asked);
    assert!(asked.status.success() && said.contains("✓ Proven") && said.contains("toggle.ino is state_machine \"Lamp\""), "{said}");
    // The second prompt holds the sketch and the lines the proof refuted.
    let again = std::fs::read_to_string(files.path().join("prompt-1")).unwrap();
    assert!(again.contains("int lit = 0;") && again.contains("STEP is refuted") && again.contains("- digitalWrite(LED) is led"), "{again}");
    assert!(again.contains("A case that shows it") && again.contains("<- differ"), "{again}");
    // What is written is the proven model, and `arclang prove` replays its proof with no proposer.
    let written = std::fs::read_to_string(&recovered).unwrap();
    assert!(written.starts_with("// Recovered from toggle.ino: proposed by") && written.contains("\"lit == 1 is led\""), "{written}");
    let replayed = arclang(&["prove", recovered.to_str().unwrap(), "--timeout", "1200"]);
    assert!(replayed.status.success(), "{}", text(&replayed));

    // A proposer that never gets it right: the command fails and writes nothing.
    std::fs::remove_file(&recovered).unwrap();
    let (command, _files) = proposer(&[fenced(&poor), "I cannot.".to_string()]);
    let asked = arclang(&["recover", sketch.to_str().unwrap(), "--propose", "--with", &command, "--turns", "2", "--timeout", "1200"]);
    let said = text(&asked);
    assert!(!asked.status.success() && said.contains("no model of toggle.ino was proven in 2 turn(s); nothing is written"), "{said}");
    assert!(!recovered.exists());
}

#[test]
fn proposing_without_a_proposer_says_what_is_missing() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let asked = std::process::Command::new(env!("CARGO_BIN_EXE_arclang")).args(["recover", fixtures.join("toggle.ino").to_str().unwrap(), "--propose", "-o", "/dev/null"]).env_remove("ARCLANG_PROPOSER").output().unwrap();
    assert!(!asked.status.success() && text(&asked).contains("--propose needs a proposer"), "{}", text(&asked));
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn the_machine_steps_at_the_period_of_the_correspondence() {
    // A loop() of 20 ms and a time of 500 ms: 25 steps, whatever period the model's C takes by default.
    let (proven, output) = proof("blinker", &[], &[]);
    assert!(proven && output.contains("for one loop() a step of 20 ms"), "{output}");
    // The sketch changes after 400 ms: refuted.
    let (proven, output) = proof("blinker", &[], &[("millis() - changed >= 500", "millis() - changed >= 400")]);
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
}

#[test]
fn the_compiler_checks_what_a_correspondence_names() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sketches");
    let dir = tempfile::tempdir().unwrap();
    let model = std::fs::read_to_string(fixtures.join("toggle.arc")).unwrap();
    let check = |from: &str, to: &str| {
        assert!(model.contains(from), "toggle.arc has `{from}`");
        let path = dir.path().join("toggle.arc");
        std::fs::write(&path, model.replacen(from, to, 1)).unwrap();
        let checked = arclang(&["check", path.to_str().unwrap()]);
        (checked.status.success(), text(&checked))
    };
    // As written, with no sketch beside it and no prover: the model compiles.
    let (compiles, said) = check("machine: \"Lamp\"", "machine: \"Lamp\"");
    assert!(compiles, "{said}");
    // A machine, a state, a variable the model does not have; a line that says nothing.
    let (compiles, said) = check("machine: \"Lamp\"", "machine: \"Lantern\"");
    assert!(!compiles && said.contains("the model has no state_machine \"Lantern\""), "{said}");
    let (compiles, said) = check("is in OnHeld\"", "is in Held\"");
    assert!(!compiles && said.contains("has no state 'Held'"), "{said}");
    let (compiles, said) = check("\"lit == 1 is led\"", "\"lit == 1 is lamp\"");
    assert!(!compiles && said.contains("has no variable 'lamp'"), "{said}");
    // A source outside the directory of the model is never read.
    for outside in ["/etc/passwd", "../toggle.ino", "sub/../../toggle.ino"] {
        let (compiles, said) = check("source: \"toggle.ino\"", &format!("source: \"{outside}\""));
        assert!(!compiles && said.contains("is not a file beside the model or below it"), "{outside}: {said}");
    }
    let (compiles, said) = check("\"digitalRead(BUTTON) is button\"", "\"digitalRead(BUTTON)\"");
    assert!(!compiles && said.contains("does not say what it is in the model"), "{said}");
}

#[test]
fn a_board_is_recovered_from_its_device_tree_and_the_model_compiles() {
    let dir = tempfile::tempdir().unwrap();
    let tree = dir.path().join("board.dts");
    std::fs::write(&tree, "/dts-v1/;\n/ {\n    model = \"Acme Board\";\n    compatible = \"acme,board\\0acme,soc\";\n    #address-cells = <1>;\n    #size-cells = <1>;\n    cpus { cpu@0 { compatible = \"arm,cortex-m7\"; }; };\n    memory@20000000 { device_type = \"memory\"; reg = <0x20000000 0x100000>; };\n    soc {\n        compatible = \"simple-bus\";\n        gpio0: gpio@40020000 { compatible = \"acme,gpio\"; #gpio-cells = <2>; };\n        i2c@40005400 {\n            compatible = \"acme,i2c\";\n            sensor@48 { compatible = \"acme,temp\"; alert-gpios = <&gpio0 5 0>; };\n        };\n        can@40006400 { compatible = \"acme,can\"; status = \"disabled\"; };\n    };\n};\n").unwrap();
    let model = dir.path().join("board.arc");
    let recovered = arclang(&["recover", tree.to_str().unwrap(), "-o", model.to_str().unwrap()]);
    let said = text(&recovered);
    assert!(recovered.status.success() && said.contains("Recovered board \"Acme Board\"") && said.contains("4 device(s), 5 link(s), read in the tree"), "{said}");
    assert!(said.contains("1 node(s) the tree marks as not enabled") && said.contains("does not hold the schematic"), "{said}");
    let written = std::fs::read_to_string(&model).unwrap();
    assert!(written.contains("processor: \"1 x arm,cortex-m7\"") && written.contains("ram: 1 MiB") && written.contains("type: \"acme,board\""), "{written}");
    assert!(written.contains("protocol: \"I2C, address 0x48\"") && written.contains("protocol: \"GPIO 5 (alert)\""), "{written}");
    assert!(!written.contains("acme,can"), "{written}");
    assert!(arclang(&["check", model.to_str().unwrap()]).status.success());
    // A tree that is not flattened is refused, with how to flatten it.
    std::fs::write(&tree, "#include \"soc.dtsi\"\n/ { };\n").unwrap();
    let refused = arclang(&["recover", tree.to_str().unwrap()]);
    assert!(!refused.status.success() && text(&refused).contains("give the flattened tree"), "{}", text(&refused));
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_proof_says_whether_it_holds_on_the_clock_of_the_board() {
    // Proven on the clock the proof states...
    let (proven, output) = proof("uno_lamp", &[], &[]);
    assert!(proven && output.contains("✓ Proven"), "{output}");
    // ...and not on the clock of an AVR, where a time read can be 1 ms short:
    // the sketch reads 49 where the machine has 50, and waits one loop more.
    assert!(output.contains("on the clock of the board") && output.contains("NOT proven"), "{output}");
    assert!(output.contains("`millis() - since` and `time in") && output.contains("before 49 and 50"), "{output}");
    assert!(output.contains("the code can act one loop later than the machine"), "{output}");
    // A board that is not an AVR: the question is not asked.
    let (proven, output) = proof("debounce", &[], &[]);
    assert!(proven && !output.contains("on the clock of the board"), "{output}");
}

/// A run recorded on a real Arduino UNO by a second board: 400 changes of its button, 99 of its lamp.
const UNO_RUN: &str = "tests/fixtures/observations/uno_lamp_400.json";

#[test]
fn a_board_recorded_is_compared_with_the_model_change_by_change() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let observed = arclang(&["observe", root.join("tests/fixtures/sketches/uno_lamp.arc").to_str().unwrap(), "--recorded", root.join(UNO_RUN).to_str().unwrap()]);
    let said = text(&observed);
    // The loop of the board, measured from the moments its lamp changed.
    assert!(said.contains("the board's loop takes 10.024 ms (the model steps every 10 ms)"), "{said}");
    // The same changes in the same order; some one loop later, as its clock makes them.
    assert!(observed.status.success() && said.contains("✓ Agrees on this run: the same 99 change(s) in the same order; 90 at the model's step, 9 one loop later"), "{said}");
    assert!(said.contains("a measure of what was played, not a proof"), "{said}");
}

#[test]
fn the_same_board_timing_with_micros_is_never_a_loop_late() {
    // The same lamp, flashed with the sketch that reads the finer clock: 400 changes of the button, 103 of the lamp.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let observed = arclang(&["observe", root.join("tests/fixtures/sketches/uno_lamp_micros.arc").to_str().unwrap(), "--recorded", root.join("tests/fixtures/observations/uno_lamp_micros_400.json").to_str().unwrap()]);
    let said = text(&observed);
    assert!(observed.status.success() && said.contains("the same 103 change(s) in the same order; 103 at the model's step, 0 one loop later"), "{said}");
}

#[test]
fn a_model_the_board_does_not_follow_disagrees_with_its_recording() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = std::fs::read_to_string(root.join("tests/fixtures/sketches/uno_lamp.arc")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    // A model that waits 100 ms where the board waits 50; a model whose lamp only ever turns on.
    for (from, to) in [("guard: \"not button\" timing: 50 ms action", "guard: \"not button\" timing: 100 ms action"), ("action: \"led := not led\"", "action: \"led := true\"")] {
        assert!(model.contains(from), "uno_lamp.arc has `{from}`");
        let wrong = dir.path().join("uno_lamp.arc");
        std::fs::write(&wrong, model.replacen(from, to, 1)).unwrap();
        let observed = arclang(&["observe", wrong.to_str().unwrap(), "--recorded", root.join(UNO_RUN).to_str().unwrap()]);
        let said = text(&observed);
        assert!(!observed.status.success() && said.contains("✗ Disagrees") && said.contains("where the model does not change it"), "{to}: {said}");
    }
    // Without an instrument nor a recording, the command says what it needs.
    let alone = arclang(&["observe", root.join("tests/fixtures/sketches/uno_lamp.arc").to_str().unwrap()]);
    assert!(!alone.status.success() && text(&alone).contains("--instrument names the serial port of the instrument"), "{}", text(&alone));
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_sketch_with_analog_writes_a_serial_port_and_a_pin_read_back_is_proven() {
    // analogWrite as a variable of the machine, constrain, Serial.print, a pin the sketch writes read back.
    let (proven, output) = proof("dimmer", &[], &[]);
    assert!(proven && output.contains("dimmer.ino is state_machine \"Dimmer\"") && output.contains("proven too"), "{output}");
    // The value written is compared: a sketch that writes another is refuted.
    let (proven, output) = proof("dimmer", &[], &[("    analogWrite(LAMP, level);", "    analogWrite(LAMP, level / 2);")]);
    assert!(!proven && output.contains("does not hold after it: analogWrite(LAMP) is duty"), "{output}");
    // The pin read back is the level last written: a heartbeat that no longer changes is refuted.
    let (proven, output) = proof("dimmer", &[], &[("digitalWrite(BEAT, !digitalRead(BEAT));", "digitalWrite(BEAT, digitalRead(BEAT));")]);
    assert!(!proven && output.contains("does not hold after it: digitalWrite(BEAT) is beat"), "{output}");
    // What is printed is not looked at; what would be read from the port is an input nobody named.
    let (proven, output) = proof("dimmer", &[], &[("  int up = digitalRead(UP);", "  int up = digitalRead(UP);\n  if (Serial.available() > 0) { level = Serial.read(); }")]);
    assert!(!proven && output.contains("does not hold after it: the code reads and writes only the pins the correspondence names"), "{output}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test correspondence -- --include-ignored"]
fn a_sketch_that_times_with_micros_is_proven_on_the_clock_of_the_board_too() {
    // The same lamp as uno_lamp, timed on the finer clock: measured on the board, never late.
    let (proven, output) = proof("uno_lamp_micros", &[], &[]);
    assert!(proven && output.contains("on the clock of the board") && output.contains("proven too"), "{output}");
    // A model that waits another time is still refuted.
    let (proven, output) = proof("uno_lamp_micros", &[("timing: 50 ms action", "timing: 60 ms action")], &[]);
    assert!(!proven && output.contains("✗ Refuted"), "{output}");
}

#[test]
fn an_action_written_as_code_that_is_not_code_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("m.arc");
    // `%` is not an operator of the action language: taken for a name, the action would do nothing.
    std::fs::write(&model, "model \"M\" { version: \"1.0.0\" }\nstate_machine \"S\" {\n    variables: [\"in go: Boolean\", \"n: Integer = 0\"]\n    initial: \"A\"\n    state A {}\n    state B {}\n    transition A -> B { guard: \"go\" action: \"n := (n + 1) % 4\" }\n}\n").unwrap();
    let checked = arclang(&["check", model.to_str().unwrap()]);
    let said = text(&checked);
    assert!(!checked.status.success() && said.contains("is written as code (`:=`) and is not understood as statements"), "{said}");
    // An action that is a name stays one.
    std::fs::write(&model, "model \"M\" { version: \"1.0.0\" }\nstate_machine \"S\" {\n    variables: [\"in go: Boolean\"]\n    initial: \"A\"\n    state A {}\n    state B {}\n    transition A -> B { guard: \"go\" action: \"open the valve\" }\n}\n").unwrap();
    assert!(arclang(&["check", model.to_str().unwrap()]).status.success());
}
