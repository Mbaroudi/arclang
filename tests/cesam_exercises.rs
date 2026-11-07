//! The CESAM tutorial (`docs/cesam-tutorial.md`) and its exercises
//! (`examples/cesam/exercises/`) are executed, not trusted: the starter
//! model misses exactly what the exercises teach, each cumulative solution
//! answers its exercise and nothing less, and every `console` block of the
//! tutorial prints what the page shows when its commands run against the
//! real compiler. The page cannot drift from the compiler.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn arclang() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_arclang"))
}

/// Runs a shell line in the repository root with `arclang` on the PATH; the
/// output is stdout then stderr, as a terminal shows them.
fn sh(line: &str) -> (i32, String) {
    let bin_dir = arclang().parent().expect("binary dir").to_path_buf();
    let path = format!("{}:{}", bin_dir.display(), std::env::var("PATH").unwrap_or_default());
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(line)
        .current_dir(root())
        .env("PATH", path)
        .env("NO_COLOR", "1")
        .output()
        .unwrap_or_else(|e| panic!("cannot run `{line}`: {e}"));
    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), text)
}

/// The CESAM rules the lint of a file reports, with the element each names.
fn advisories(file: &str) -> BTreeSet<String> {
    let (_, out) = sh(&format!("arclang check {file} --lint --method cesam"));
    out.lines()
        .filter(|l| l.contains("[cesam."))
        .map(|l| {
            let rule = l.split('[').nth(1).and_then(|r| r.split(']').next()).unwrap_or_default();
            let subject = l.split('\'').nth(1).unwrap_or_default();
            format!("{rule} {subject}")
        })
        .collect()
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

const START: &str = "examples/cesam/exercises/01_start.arc";
const AFTER: [&str; 4] = [
    "examples/cesam/exercises/solutions/after_ex2.arc",
    "examples/cesam/exercises/solutions/after_ex3.arc",
    "examples/cesam/exercises/solutions/after_ex4.arc",
    "examples/cesam/exercises/solutions/after_ex5.arc",
];

#[test]
fn the_starter_compiles_and_misses_exactly_what_the_exercises_teach() {
    let (code, out) = sh(&format!("arclang check {START}"));
    assert_eq!(code, 0, "the starter must compile:\n{out}");
    assert_eq!(
        advisories(START),
        set(&[
            "cesam.stakeholder-need Installer",
            "cesam.stakeholder-need Insurer",
            "cesam.need-stakeholder NEED-3",
            "cesam.need-requirement NEED-2",
            "cesam.mission-use-case Keep everyone else out",
            "cesam.use-case-scenario Enter with a guest code",
            "cesam.lifecycle-context Lock lifecycle.Maintenance",
            "cesam.mode-configuration Lock modes.Pairing",
            "cesam.requirement-vision FUN-2",
        ])
    );
    assert!(
        out.contains("cesam: state 'Lock configurations.Radio only': realizes 'Lock lifecycle.Installation' is a lifecycle phase, expected a functional mode"),
        "the wrong-role relation is a warning of the compilation:\n{out}"
    );
}

#[test]
fn each_solution_answers_its_exercise_and_nothing_less() {
    let after2 = advisories(AFTER[0]);
    assert!(!after2.iter().any(|a| a.contains("need")), "exercise 2 closes the needs: {after2:?}");
    assert_eq!(after2.len(), 5, "{after2:?}");
    let after3 = advisories(AFTER[1]);
    assert!(!after3.iter().any(|a| a.contains("use-case")), "exercise 3 closes the use cases: {after3:?}");
    assert_eq!(after3.len(), 3, "{after3:?}");
    let after4 = advisories(AFTER[2]);
    assert_eq!(after4, set(&["cesam.requirement-vision FUN-2"]), "exercise 4 closes the states");
    assert!(advisories(AFTER[3]).is_empty(), "exercise 5 leaves nothing: {:?}", advisories(AFTER[3]));
}

#[test]
fn the_finished_model_passes_the_gate_like_the_example() {
    for file in [AFTER[3], "examples/cesam/smart-lock/smart_lock.arc"] {
        let (code, out) = sh(&format!("arclang gate {file}"));
        assert_eq!(code, 0, "{file} passes the gate:\n{out}");
        assert!(out.contains("PASS"), "{file}:\n{out}");
    }
}

/// Every `console` block of the tutorial: `$ ` lines are commands run in the
/// repository root, the lines under each are what it must print.
#[test]
fn the_tutorial_prints_what_its_commands_print() {
    let page = std::fs::read_to_string(root().join("docs/cesam-tutorial.md")).expect("docs/cesam-tutorial.md");
    let mut checked = 0;
    let mut failures = Vec::new();
    for block in page.split("```console\n").skip(1) {
        let body = block.split("\n```").next().unwrap_or_default();
        let mut command: Option<String> = None;
        let mut expected: Vec<&str> = Vec::new();
        let mut flush = |command: &Option<String>, expected: &[&str]| {
            if let Some(cmd) = command {
                let (_, out) = sh(cmd);
                let shown = expected.join("\n").trim_end().to_string();
                let got = out.trim_end().to_string();
                checked += 1;
                if shown != got {
                    failures.push(format!("$ {cmd}\n--- the page shows\n{shown}\n--- the compiler prints\n{got}\n"));
                }
            }
        };
        for line in body.lines() {
            if let Some(cmd) = line.strip_prefix("$ ") {
                flush(&command, &expected);
                command = Some(cmd.to_string());
                expected.clear();
            } else {
                expected.push(line);
            }
        }
        flush(&command, &expected);
    }
    assert!(checked >= 10, "the tutorial has at least ten executed commands, {checked} found");
    assert!(failures.is_empty(), "{} command(s) of docs/cesam-tutorial.md no longer print what the page shows:\n\n{}", failures.len(), failures.join("\n"));
}

/// Every figure of the tutorial and of the method's page is a file of
/// `docs/images/` (regenerated by `tools/cesam_screenshots.mjs` and
/// `tools/cesam_visuals.mjs`).
#[test]
fn every_image_of_the_cesam_pages_exists() {
    for (file, at_least) in [("docs/cesam-tutorial.md", 6), ("docs/cesam.md", 4)] {
        let page = std::fs::read_to_string(root().join(file)).unwrap_or_else(|_| panic!("{file}"));
        let mut images = 0;
        for (i, _) in page.match_indices("](images/") {
            let rest = &page[i + 2..];
            let path = rest.split(')').next().unwrap_or_default();
            assert!(Path::new(&root().join("docs").join(path)).is_file(), "{file}: docs/{path} is missing");
            images += 1;
        }
        assert!(images >= at_least, "{file} shows at least {at_least} figures, {images} found");
    }
}
