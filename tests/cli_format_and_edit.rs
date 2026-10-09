//! `arclang fmt`, `arclang set` and `arclang unset` through the real binary.

use std::path::Path;
use std::process::{Command, Output};

const MODEL: &str = r#"// Lane keeping
system_analysis "LKA" {
  // The requirement everything traces to.
  requirement "REQ-1" {
    description: "Correct within 80 ms"   // budget
    priority: "Critical"
  }
}
"#;

fn arclang(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn model_in(directory: &Path) -> String {
    let path = directory.join("model.arc");
    std::fs::write(&path, MODEL).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn fmt_prints_the_formatted_model_and_leaves_the_file_alone() {
    let directory = tempfile::tempdir().unwrap();
    let path = model_in(directory.path());

    let output = arclang(&["fmt", &path]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("    requirement \"REQ-1\" {\n        description: \"Correct within 80 ms\" // budget\n"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), MODEL);
}

#[test]
fn fmt_check_fails_until_the_file_is_written_formatted() {
    let directory = tempfile::tempdir().unwrap();
    let path = model_in(directory.path());
    let folder = directory.path().to_string_lossy().into_owned();

    let unformatted = arclang(&["fmt", &folder, "--check"]);
    assert_eq!(unformatted.status.code(), Some(1));
    assert!(stdout(&unformatted).contains("model.arc"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), MODEL, "--check must not write");

    assert!(arclang(&["fmt", &folder, "--write"]).status.success());
    assert!(arclang(&["fmt", &folder, "--check"]).status.success());
    let formatted = std::fs::read_to_string(&path).unwrap();
    assert!(formatted.contains("// The requirement everything traces to."));
    assert!(formatted.contains("// budget"));
}

#[test]
fn fmt_reports_a_file_that_does_not_lex_and_still_handles_the_others() {
    let directory = tempfile::tempdir().unwrap();
    model_in(directory.path());
    std::fs::write(directory.path().join("broken.arc"), "actor \"never closed").unwrap();
    let folder = directory.path().to_string_lossy().into_owned();

    let output = arclang(&["fmt", &folder, "--check"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("broken.arc"));
    assert!(stdout(&output).contains("model.arc"));
}

#[test]
fn set_rewrites_one_value_and_keeps_the_rest_byte_for_byte() {
    let directory = tempfile::tempdir().unwrap();
    let path = model_in(directory.path());

    let output = arclang(&["set", &path, "REQ-1", "priority", "\"High\"", "--write"]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("REQ-1.priority: Critical -> High"));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        MODEL.replace("priority: \"Critical\"", "priority: \"High\"")
    );
}

#[test]
fn set_without_write_prints_and_leaves_the_file_alone() {
    let directory = tempfile::tempdir().unwrap();
    let path = model_in(directory.path());

    let output = arclang(&["set", &path, "REQ-1", "owner", "\"Chassis\""]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("    priority: \"Critical\"\n    owner: \"Chassis\"\n  }"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), MODEL);
}

#[test]
fn unset_removes_the_attribute_and_keeps_its_comment() {
    let directory = tempfile::tempdir().unwrap();
    let path = model_in(directory.path());

    let output = arclang(&["unset", &path, "REQ-1", "description", "--write"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let edited = std::fs::read_to_string(&path).unwrap();
    assert!(!edited.contains("description"));
    assert!(edited.contains("// budget"));
}

#[test]
fn a_refused_edit_writes_nothing_and_leaves_no_scratch_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = model_in(directory.path());

    for arguments in [
        vec!["set", &path, "REQ-404", "priority", "\"High\"", "--write"],
        vec!["set", &path, "REQ-1", "budget", "80 parsecs", "--write"],
        vec!["set", &path, "REQ-1", "priority", "High // note", "--write"],
        vec!["unset", &path, "REQ-1", "owner", "--write"],
    ] {
        let output = arclang(&arguments);
        assert_eq!(output.status.code(), Some(1), "{arguments:?} should be refused");
    }

    assert_eq!(std::fs::read_to_string(&path).unwrap(), MODEL);
    let files: Vec<_> = std::fs::read_dir(directory.path()).unwrap().collect();
    assert_eq!(files.len(), 1, "only the model itself remains");
}

#[test]
fn an_element_of_an_imported_file_is_edited_in_that_file() {
    let directory = tempfile::tempdir().unwrap();
    let requirements = model_in(directory.path());
    let main = directory.path().join("main.arc");
    std::fs::write(&main, "import \"model.arc\"\nmodel Car {}\n").unwrap();
    let main = main.to_string_lossy().into_owned();

    let through_main = arclang(&["set", &main, "REQ-1", "priority", "\"High\"", "--write"]);
    assert_eq!(through_main.status.code(), Some(1));
    assert!(stderr(&through_main).contains("edit the imported file"));

    let direct = arclang(&["set", &requirements, "REQ-1", "priority", "\"High\"", "--write"]);
    assert!(direct.status.success(), "{}", stderr(&direct));
}

#[test]
fn rename_changes_the_name_on_the_declaration_line_when_identity_survives() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("model.arc");
    let source = "logical_architecture \"L\" {\n    // main channel\n    component \"Controller\" { id: \"LC-1\" }\n    component \"Monitor\" {}\n}\n";
    std::fs::write(&path, source).unwrap();
    let path = path.to_string_lossy().into_owned();

    let renamed = arclang(&["rename", &path, "LC-1", "Brake controller", "--write"]);
    assert!(renamed.status.success(), "{}", stderr(&renamed));
    assert!(stdout(&renamed).contains("LC-1: 'Controller' -> 'Brake controller'"));
    let expected = source.replace("\"Controller\"", "\"Brake controller\"");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), expected);

    // "Monitor" has no id: renaming it would make it another element.
    let refused = arclang(&["rename", &path, "Monitor", "Watchdog", "--write"]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(stderr(&refused).contains("identity"), "{}", stderr(&refused));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), expected);
}
