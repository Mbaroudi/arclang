//! Semantic git end to end, on real git repositories built by the tests:
//! the diff by revision, the history of an element, the merge driver,
//! the layout merge and the baselines.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MODEL: &str = r#"// Demo model for semantic git.
model Demo {
    name: "Demo"
}

requirements {
    // The range requirement.
    requirement "REQ-001" {
        description: "Detect at 150 m"
        priority: "High"
    }
    requirement "REQ-002" {
        description: "React in 100 ms"
        priority: "High"
    }
}

system_analysis "Sys" {
    function "Sense" { id: "SF-SENSE" latency: 20 ms }
    function "Plan" { id: "SF-PLAN" latency: 30 ms }
}

logical_architecture "Logical" {
    component "Controller" {
        id: "LC-001"
        safety_level: "ASIL-B"
        description: "Computes the plan"
        function "Compute" { id: "LF-001" }
    }
    component "Sensor" {
        id: "LC-002"
        description: "Senses"
    }
}

trace "LC-001" satisfies "REQ-001" { rationale: "plans" }
"#;

fn arclang() -> &'static str {
    env!("CARGO_BIN_EXE_arclang")
}

struct Repo {
    dir: tempfile::TempDir,
}

fn git_in(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "Ada")
        .env("GIT_AUTHOR_EMAIL", "ada@example.com")
        .env("GIT_COMMITTER_NAME", "Ada")
        .env("GIT_COMMITTER_EMAIL", "ada@example.com")
        .output()
        .expect("git runs");
    assert!(out.status.success(), "git {args:?}: {}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).to_string()
}

impl Repo {
    fn new() -> Repo {
        let dir = tempfile::tempdir().unwrap();
        git_in(dir.path(), &["init", "-q", "-b", "main"]);
        git_in(dir.path(), &["config", "commit.gpgsign", "false"]);
        git_in(dir.path(), &["config", "tag.gpgsign", "false"]);
        Repo { dir }
    }

    fn with_model() -> Repo {
        let repo = Repo::new();
        repo.write("model.arc", MODEL);
        repo.commit("initial model");
        repo
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn file(&self, name: &str) -> PathBuf {
        self.path().join(name)
    }

    fn write(&self, name: &str, text: &str) {
        std::fs::write(self.file(name), text).unwrap();
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.file(name)).unwrap()
    }

    fn edit(&self, name: &str, from: &str, to: &str) {
        let text = self.read(name);
        assert!(text.contains(from), "{from:?} is not in {name}");
        self.write(name, &text.replacen(from, to, 1));
    }

    fn commit(&self, message: &str) {
        git_in(self.path(), &["add", "-A"]);
        git_in(self.path(), &["commit", "-q", "-m", message]);
    }

    fn git(&self, args: &[&str]) -> String {
        git_in(self.path(), args)
    }

    fn arclang(&self, args: &[&str]) -> Output {
        Command::new(arclang()).current_dir(self.path()).envs([("GIT_AUTHOR_NAME", "ArcLang Test"), ("GIT_AUTHOR_EMAIL", "test@arclang.invalid"), ("GIT_COMMITTER_NAME", "ArcLang Test"), ("GIT_COMMITTER_EMAIL", "test@arclang.invalid")]).args(args).output().expect("arclang runs")
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.arclang(args);
        assert!(out.status.success(), "arclang {args:?}: {}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn install_driver(&self) {
        let out = self.ok(&["merge", "--install", "--yes", "--binary", arclang()]);
        assert!(out.contains("Installed"), "{out}");
    }

    /// Branch `name` from main, with `change` applied and committed.
    fn branch(&self, name: &str, change: impl Fn(&Repo)) {
        self.git(&["checkout", "-q", "-b", name, "main"]);
        change(self);
        self.commit(name);
        self.git(&["checkout", "-q", "main"]);
    }
}

fn text_of(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

// ---- diff by revision ------------------------------------------------------

fn normalize(text: &str, repo: &Repo) -> String {
    text.replace(&repo.path().canonicalize().unwrap().display().to_string(), "<repo>").replace(&repo.path().display().to_string(), "<repo>")
}

fn golden(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/semgit").join(name);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(&path, actual).unwrap();
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("no golden {name}: run with UPDATE_GOLDEN=1"));
    assert_eq!(expected, actual, "{name} differs: UPDATE_GOLDEN=1 regenerates it");
}

fn changed_repo() -> Repo {
    let repo = Repo::with_model();
    repo.edit("model.arc", "description: \"Detect at 150 m\"", "description: \"Detect at 200 m\"");
    repo.edit("model.arc", "component \"Sensor\" {", "component \"Radar Sensor\" {");
    repo.edit("model.arc", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n    function \"Act\" { id: \"SF-ACT\" latency: 10 ms }\n");
    repo.edit("model.arc", "    requirement \"REQ-002\" {\n        description: \"React in 100 ms\"\n        priority: \"High\"\n    }\n", "");
    repo.commit("rework");
    repo
}

#[test]
fn diff_by_revision_groups_changes_by_layer_and_kind() {
    let repo = changed_repo();
    let out = repo.arclang(&["diff", "model.arc", "--from", "HEAD~1", "--to", "HEAD"]);
    assert_eq!(out.status.code(), Some(1), "a diff exits 1: {}", text_of(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("+ added") && text.contains("SF-ACT"), "{text}");
    assert!(text.contains("- removed") && text.contains("REQ-002"), "{text}");
    assert!(text.contains("renamed: \"Sensor\" -> \"Radar Sensor\""), "{text}");
    assert!(text.contains("description: \"Detect at 150 m\" -> \"Detect at 200 m\""), "{text}");
    assert!(text.contains("Logical Architecture (LA)") && text.contains("System Analysis (SA)"), "{text}");
    golden("diff.txt", &normalize(&text, &repo));
}

#[test]
fn diff_formats_json_md_html_have_the_same_content() {
    let repo = changed_repo();
    let run = |f: &str| String::from_utf8_lossy(&repo.arclang(&["diff", "model.arc", "--from", "HEAD~1", "--to", "HEAD", "-f", f]).stdout).to_string();
    let json = run("json");
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["summary"]["elements_added"], 1);
    assert_eq!(value["summary"]["renamed"], 1);
    assert_eq!(value["model_changed"], true);
    assert!(value["layers"].as_array().unwrap().iter().any(|l| l["layer"] == "LA"));
    assert!(value["consequences"]["gate_before"]["standard"].is_string());
    golden("diff.md", &normalize(&run("md"), &repo));
    let html = run("html");
    assert!(html.starts_with("<!doctype html>") && html.contains("SF-ACT") && html.contains("class=\"removed\""), "{html}");
    golden("diff.html", &normalize(&html, &repo));
    let mut v = value;
    v["model"] = "<repo>".into();
    v["from_commit"] = "<c>".into();
    v["to_commit"] = "<c>".into();
    golden("diff.json", &format!("{}\n", serde_json::to_string_pretty(&v).unwrap()));
}

#[test]
fn diff_reports_the_consequences_on_gate_and_coverage() {
    let repo = Repo::with_model();
    repo.edit("model.arc", "trace \"LC-001\" satisfies \"REQ-001\" { rationale: \"plans\" }\n", "");
    repo.commit("drop the trace");
    let json: serde_json::Value = serde_json::from_str(&repo.ok_or_diff(&["diff", "model.arc", "--from", "HEAD~1", "-f", "json"])).unwrap();
    let c = &json["consequences"];
    assert!(c["coverage"].as_array().unwrap().iter().any(|d| d["requirement"] == "REQ-001"), "{c}");
    assert!(c["newly_unsatisfied"].as_array().unwrap().iter().any(|d| d["requirement"] == "REQ-001"), "{c}");
    let rel = json["relationships"].as_array().unwrap();
    assert!(rel.iter().any(|r| r["status"] == "removed" && r["category"] == "trace"), "{rel:?}");
}

impl Repo {
    /// Run a command that may exit 1 (a diff) and return its output.
    fn ok_or_diff(&self, args: &[&str]) -> String {
        let out = self.arclang(args);
        assert!(matches!(out.status.code(), Some(0) | Some(1)), "arclang {args:?}: {}", text_of(&out));
        String::from_utf8_lossy(&out.stdout).to_string()
    }
}

#[test]
fn a_retargeted_trace_is_one_change() {
    let repo = Repo::with_model();
    repo.edit("model.arc", "trace \"LC-001\" satisfies \"REQ-001\"", "trace \"LC-001\" satisfies \"REQ-002\"");
    repo.commit("retarget");
    let json: serde_json::Value = serde_json::from_str(&repo.ok_or_diff(&["diff", "model.arc", "--from", "HEAD~1", "-f", "json"])).unwrap();
    let rel = json["relationships"].as_array().unwrap();
    assert!(rel.iter().any(|r| r["status"] == "retargeted" && r["was"]["from"] == "REQ-001" && r["was"]["to"] == "REQ-002"), "{rel:?}");
}

#[test]
fn a_moved_element_is_reported_as_moved() {
    let repo = Repo::with_model();
    let text = repo.read("model.arc");
    let moved = text.replace("        function \"Compute\" { id: \"LF-001\" }\n", "").replace("        description: \"Senses\"\n", "        description: \"Senses\"\n        function \"Compute\" { id: \"LF-001\" }\n");
    repo.write("model.arc", &moved);
    repo.commit("move the function");
    let json: serde_json::Value = serde_json::from_str(&repo.ok_or_diff(&["diff", "model.arc", "--from", "HEAD~1", "-f", "json"])).unwrap();
    let text = json.to_string();
    assert!(text.contains("\"moved\""), "{text}");
}

#[test]
fn layout_only_changes_are_not_model_changes() {
    let repo = Repo::with_model();
    repo.write("model.layout.json", "{\n  \"arclang_layout\": \"1\",\n  \"views\": {},\n  \"arrangements\": []\n}\n");
    repo.commit("layout");
    repo.write("model.layout.json", "{\n  \"arclang_layout\": \"1\",\n  \"views\": {},\n  \"arrangements\": [{\"view\": \"lab\", \"open\": null, \"folded\": [], \"places\": {\"LC-001\": {\"dx\": 10.0, \"dy\": 5.0}}}]\n}\n");
    repo.commit("move a box");
    let out = repo.arclang(&["diff", "model.arc", "--from", "HEAD~1", "-f", "json"]);
    assert_eq!(out.status.code(), Some(0), "layout alone is not a difference: {}", text_of(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["model_changed"], false);
    assert_eq!(json["layout_only"], true);
    assert_eq!(json["layout"][0]["what"], "places");
    assert_eq!(json["layout"][0]["element"], "LC-001");
    let text = String::from_utf8_lossy(&repo.arclang(&["diff", "model.arc", "--from", "HEAD~1"]).stdout).to_string();
    assert!(text.contains("No semantic changes.") && text.contains("not a model change"), "{text}");
}

#[test]
fn diff_of_two_files_keeps_its_old_shape() {
    let repo = changed_repo();
    repo.write("old.arc", MODEL);
    let out = repo.arclang(&["diff", "old.arc", "model.arc", "--json"]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["added"].is_array() && json["modified"].is_array());
}

#[test]
fn diff_resolves_imports_at_the_revision() {
    let repo = Repo::new();
    repo.write("main.arc", "import \"reqs.arc\"\n\nmodel M {\n    name: \"M\"\n}\n");
    repo.write("reqs.arc", "requirements {\n    requirement \"R-1\" {\n        description: \"old\"\n        priority: \"High\"\n    }\n}\n");
    repo.commit("one");
    repo.edit("reqs.arc", "\"old\"", "\"new\"");
    repo.commit("two");
    let text = repo.ok_or_diff(&["diff", "main.arc", "--from", "HEAD~1", "--to", "HEAD"]);
    assert!(text.contains("description: \"old\" -> \"new\""), "{text}");
}

#[test]
fn an_unknown_revision_is_named() {
    let repo = Repo::with_model();
    let out = repo.arclang(&["diff", "model.arc", "--from", "nope"]);
    assert!(text_of(&out).contains("unknown revision 'nope'"), "{}", text_of(&out));
}

// ---- history ---------------------------------------------------------------

#[test]
fn history_lists_the_commits_that_changed_an_element() {
    let repo = Repo::with_model();
    repo.edit("model.arc", "description: \"Detect at 150 m\"", "description: \"Detect at 200 m\"");
    repo.commit("raise the range");
    repo.edit("model.arc", "latency: 20 ms", "latency: 25 ms");
    repo.commit("unrelated change");
    repo.edit("model.arc", "requirement \"REQ-001\" {\n        description: \"Detect at 200 m\"\n        priority: \"High\"", "requirement \"REQ-001\" {\n        description: \"Detect at 200 m\"\n        priority: \"Critical\"");
    repo.commit("make it critical");
    let json: serde_json::Value = serde_json::from_str(&repo.ok_or_diff(&["history", "model.arc", "REQ-001", "--json"])).unwrap();
    let entries = json["entries"].as_array().unwrap();
    let subjects: Vec<&str> = entries.iter().map(|e| e["subject"].as_str().unwrap()).collect();
    assert_eq!(subjects, ["make it critical", "raise the range", "initial model"], "{json}");
    assert_eq!(entries[0]["changes"][0]["field"], "priority");
    assert_eq!(entries[0]["author"], "Ada");
    assert_eq!(entries[1]["changes"][0]["new"], "Detect at 200 m");
    assert_eq!(entries[2]["status"], "added");
    let limited: serde_json::Value = serde_json::from_str(&repo.ok_or_diff(&["history", "model.arc", "REQ-001", "--json", "--limit", "1"])).unwrap();
    assert_eq!(limited["entries"].as_array().unwrap().len(), 1);
    let text = repo.ok(&["history", "model.arc", "REQ-001"]);
    assert!(text.contains("History of 'REQ-001'") && text.contains("priority:"), "{text}");
}

#[test]
fn history_follows_a_rename() {
    let repo = Repo::with_model();
    repo.edit("model.arc", "component \"Sensor\" {", "component \"Radar\" {");
    repo.commit("rename the sensor");
    let json: serde_json::Value = serde_json::from_str(&repo.ok_or_diff(&["history", "model.arc", "LC-002", "--json"])).unwrap();
    assert_eq!(json["entries"][0]["renamed"]["to"], "Radar");
}

// ---- merge -----------------------------------------------------------------

fn merge_branches(repo: &Repo, ours: &str, theirs: &str) -> Output {
    repo.git(&["checkout", "-q", ours]);
    Command::new("git").arg("-C").arg(repo.path()).args(["merge", "--no-edit", theirs]).env("GIT_AUTHOR_NAME", "Ada").env("GIT_AUTHOR_EMAIL", "a@e.c").env("GIT_COMMITTER_NAME", "Ada").env("GIT_COMMITTER_EMAIL", "a@e.c").output().unwrap()
}

#[test]
fn different_elements_merge_wherever_they_are() {
    let repo = Repo::with_model();
    repo.install_driver();
    repo.commit("driver");
    repo.branch("ours", |r| r.edit("model.arc", "description: \"Detect at 150 m\"", "description: \"Detect at 200 m\""));
    repo.branch("theirs", |r| r.edit("model.arc", "latency: 30 ms", "latency: 35 ms"));
    let out = merge_branches(&repo, "ours", "theirs");
    assert!(out.status.success(), "{}", text_of(&out));
    let merged = repo.read("model.arc");
    assert!(merged.contains("Detect at 200 m") && merged.contains("latency: 35 ms"), "{merged}");
    assert!(merged.contains("// The range requirement."), "comments survive");
}

#[test]
fn different_attributes_of_one_element_merge() {
    let repo = Repo::with_model();
    repo.install_driver();
    repo.commit("driver");
    repo.branch("ours", |r| r.edit("model.arc", "safety_level: \"ASIL-B\"", "safety_level: \"ASIL-C\""));
    repo.branch("theirs", |r| r.edit("model.arc", "description: \"Computes the plan\"", "description: \"Computes the trajectory\""));
    let out = merge_branches(&repo, "ours", "theirs");
    assert!(out.status.success(), "adjacent attributes must merge: {}", text_of(&out));
    let merged = repo.read("model.arc");
    assert!(merged.contains("ASIL-C") && merged.contains("Computes the trajectory"), "{merged}");
}

#[test]
fn two_attributes_on_one_line_merge() {
    let repo = Repo::with_model();
    repo.install_driver();
    repo.commit("driver");
    repo.branch("ours", |r| r.edit("model.arc", "function \"Sense\" { id: \"SF-SENSE\" latency: 20 ms }", "function \"Sense\" { id: \"SF-SENSE\" latency: 25 ms }"));
    repo.branch("theirs", |r| r.edit("model.arc", "function \"Sense\" { id: \"SF-SENSE\" latency: 20 ms }", "function \"Sense\" { id: \"SF-SENSE\" latency: 20 ms description: \"x\" }"));
    let out = merge_branches(&repo, "ours", "theirs");
    assert!(out.status.success(), "{}", text_of(&out));
    assert!(repo.read("model.arc").contains("function \"Sense\" { id: \"SF-SENSE\" latency: 25 ms description: \"x\" }"));
}

#[test]
fn additions_to_the_same_container_merge_in_both_orders() {
    for (a, b) in [("ours", "theirs"), ("theirs", "ours")] {
        let repo = Repo::with_model();
        repo.install_driver();
        repo.commit("driver");
        repo.branch("ours", |r| r.edit("model.arc", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n    function \"Act\" { id: \"SF-ACT\" latency: 10 ms }\n"));
        repo.branch("theirs", |r| r.edit("model.arc", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n    function \"Monitor\" { id: \"SF-MON\" latency: 5 ms }\n"));
        let out = merge_branches(&repo, a, b);
        assert!(out.status.success(), "{a} <- {b}: {}", text_of(&out));
        let merged = repo.read("model.arc");
        assert!(merged.contains("SF-ACT") && merged.contains("SF-MON"), "{merged}");
        assert!(merged.find("SF-ACT").unwrap() < merged.find("SF-MON").unwrap(), "additions on both sides are ordered by key, whichever the order of the merge");
        let compiled = repo.arclang(&["check", "model.arc"]);
        assert!(compiled.status.success(), "{}", text_of(&compiled));
    }
}

#[test]
fn the_same_attribute_changed_differently_is_an_element_conflict() {
    let repo = Repo::with_model();
    repo.install_driver();
    repo.commit("driver");
    repo.branch("ours", |r| r.edit("model.arc", "description: \"Computes the plan\"", "description: \"Computes A\""));
    repo.branch("theirs", |r| r.edit("model.arc", "description: \"Computes the plan\"", "description: \"Computes B\""));
    let out = merge_branches(&repo, "ours", "theirs");
    assert!(!out.status.success(), "{}", text_of(&out));
    let merged = repo.read("model.arc");
    assert!(merged.contains("<<<<<<< ours component 'Controller' [LC-001]: description: \"Computes A\" (ours) vs description: \"Computes B\" (theirs)"), "{merged}");
    assert!(merged.contains("\n=======\n") && merged.contains(">>>>>>> theirs component 'Controller' [LC-001]"), "{merged}");
    // Both sides are whole elements: braces balance on each side.
    let count = |s: &str, c: char| s.matches(c).count();
    assert_eq!(count(&merged, '{'), count(&merged, '}') , "balanced braces: {merged}");
    assert!(merged.contains("Computes A") && merged.contains("Computes B"));
    // The rest of the file merged normally and is untouched.
    assert!(merged.contains("component \"Sensor\""));
}

#[test]
fn changed_on_one_side_removed_on_the_other_is_a_conflict() {
    let repo = Repo::with_model();
    repo.install_driver();
    repo.commit("driver");
    repo.branch("ours", |r| r.edit("model.arc", "description: \"Senses\"", "description: \"Senses far\""));
    repo.branch("theirs", |r| r.edit("model.arc", "    component \"Sensor\" {\n        id: \"LC-002\"\n        description: \"Senses\"\n    }\n", ""));
    let out = merge_branches(&repo, "ours", "theirs");
    assert!(!out.status.success());
    let merged = repo.read("model.arc");
    assert!(merged.contains("changed on ours, removed on theirs"), "{merged}");
    assert!(merged.contains("Senses far"));
}

#[test]
fn an_element_removed_on_one_side_and_untouched_on_the_other_is_removed() {
    let repo = Repo::with_model();
    repo.install_driver();
    repo.commit("driver");
    repo.branch("ours", |r| r.edit("model.arc", "latency: 20 ms", "latency: 21 ms"));
    repo.branch("theirs", |r| r.edit("model.arc", "    component \"Sensor\" {\n        id: \"LC-002\"\n        description: \"Senses\"\n    }\n", ""));
    let out = merge_branches(&repo, "ours", "theirs");
    assert!(out.status.success(), "{}", text_of(&out));
    let merged = repo.read("model.arc");
    assert!(!merged.contains("LC-002") && merged.contains("latency: 21 ms"));
}

#[test]
fn the_driver_says_why_a_clean_merge_does_not_compile() {
    let repo = Repo::with_model();
    repo.install_driver();
    repo.commit("driver");
    // Both add the same id with different content: a conflict, not silent corruption.
    repo.branch("ours", |r| r.edit("model.arc", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n    function \"New\" { id: \"SF-NEW\" latency: 1 ms }\n"));
    repo.branch("theirs", |r| r.edit("model.arc", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n", "    function \"Plan\" { id: \"SF-PLAN\" latency: 30 ms }\n    function \"New\" { id: \"SF-NEW\" latency: 2 ms }\n"));
    let out = merge_branches(&repo, "ours", "theirs");
    assert!(!out.status.success());
    assert!(repo.read("model.arc").contains("added on both sides with different content"));
}

#[test]
fn a_merge_driver_run_by_hand_reports_json() {
    let repo = Repo::with_model();
    let base = repo.read("model.arc");
    repo.write("ours.arc", &base.replace("Computes the plan", "Computes A"));
    repo.write("theirs.arc", &base.replace("Computes the plan", "Computes B"));
    repo.write("base.arc", &base);
    let out = repo.arclang(&["merge", "base.arc", "ours.arc", "theirs.arc", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["clean"], false);
    assert_eq!(json["conflicts"][0]["element"], "LC-001");
    assert!(json["conflicts"][0]["ours"].as_str().unwrap().contains("Computes A"));
    assert!(json["conflicts"][0]["theirs"].as_str().unwrap().contains("Computes B"));
}

#[test]
fn merge_preserves_crlf_and_untouched_order() {
    let repo = Repo::with_model();
    let base = repo.read("model.arc").replace('\n', "\r\n");
    let ours = base.replace("Detect at 150 m", "Detect at 200 m");
    let theirs = base.replace("latency: 30 ms", "latency: 35 ms");
    repo.write("b.arc", &base);
    repo.write("o.arc", &ours);
    repo.write("t.arc", &theirs);
    let out = repo.ok(&["merge", "b.arc", "o.arc", "t.arc"]);
    assert!(out.contains("\r\n") && !out.replace("\r\n", "").contains('\n'));
    assert!(out.find("REQ-001").unwrap() < out.find("REQ-002").unwrap());
}

// ---- layout merge ----------------------------------------------------------

fn layout(places: &[(&str, f64)]) -> String {
    let body: Vec<String> = places.iter().map(|(id, dx)| format!("\"{id}\": {{\"dx\": {dx}, \"dy\": 0.0}}")).collect();
    format!("{{\"arclang_layout\": \"1\", \"views\": {{}}, \"arrangements\": [{{\"view\": \"lab\", \"open\": null, \"folded\": [], \"places\": {{{}}}}}]}}", body.join(", "))
}

#[test]
fn layouts_of_different_elements_merge_and_the_same_entry_takes_theirs() {
    let repo = Repo::with_model();
    repo.write("b.layout.json", &layout(&[("LC-001", 0.0), ("LC-002", 0.0)]));
    repo.write("o.layout.json", &layout(&[("LC-001", 10.0), ("LC-002", 0.0)]));
    repo.write("t.layout.json", &layout(&[("LC-001", 0.0), ("LC-002", 20.0)]));
    let out = repo.ok(&["merge", "b.layout.json", "o.layout.json", "t.layout.json", "--path", "m.layout.json"]);
    let merged: serde_json::Value = serde_json::from_str(&out).unwrap();
    let places = &merged["arrangements"][0]["places"];
    assert_eq!(places["LC-001"]["dx"], 10.0);
    assert_eq!(places["LC-002"]["dx"], 20.0);
    // The same entry changed on both sides: theirs, said on standard error.
    repo.write("t2.layout.json", &layout(&[("LC-001", 99.0), ("LC-002", 0.0)]));
    let out = repo.arclang(&["merge", "b.layout.json", "o.layout.json", "t2.layout.json", "--path", "m.layout.json"]);
    assert!(out.status.success());
    let merged: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(merged["arrangements"][0]["places"]["LC-001"]["dx"], 99.0);
    assert!(text_of(&out).contains("changed on both sides: took theirs"));
}

#[test]
fn a_git_merge_of_layout_files_goes_through_the_driver() {
    let repo = Repo::with_model();
    repo.write("model.layout.json", &layout(&[("LC-001", 0.0), ("LC-002", 0.0)]));
    repo.install_driver();
    repo.commit("layout and driver");
    repo.branch("ours", |r| r.write("model.layout.json", &layout(&[("LC-001", 10.0), ("LC-002", 0.0)])));
    repo.branch("theirs", |r| r.write("model.layout.json", &layout(&[("LC-001", 0.0), ("LC-002", 20.0)])));
    let out = merge_branches(&repo, "ours", "theirs");
    assert!(out.status.success(), "{}", text_of(&out));
    let merged: serde_json::Value = serde_json::from_str(&repo.read("model.layout.json")).unwrap();
    assert_eq!(merged["arrangements"][0]["places"]["LC-001"]["dx"], 10.0);
    assert_eq!(merged["arrangements"][0]["places"]["LC-002"]["dx"], 20.0);
}

// ---- install ---------------------------------------------------------------

#[test]
fn install_shows_the_change_and_asks_first() {
    let repo = Repo::with_model();
    let mut child = Command::new(arclang()).current_dir(repo.path()).args(["merge", "--install"]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"n\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("+ merge.arclang.driver") && text.contains("+ *.arc merge=arclang") && text.contains("Apply these changes? [y/N]"), "{text}");
    assert!(text.contains("Nothing changed."));
    assert!(!repo.file(".gitattributes").exists());
    repo.install_driver();
    let attrs = repo.read(".gitattributes");
    assert!(attrs.contains("*.arc merge=arclang") && attrs.contains("*.layout.json merge=arclang"));
    assert!(repo.git(&["config", "--local", "merge.arclang.driver"]).contains("merge %O %A %B -o %A --path %P"));
    let again = repo.ok(&["merge", "--install", "--yes", "--binary", arclang()]);
    assert!(again.contains("already installed"), "{again}");
}

// ---- baselines -------------------------------------------------------------

const GATED: &str = r#"model Gated {
    name: "Gated"
}

requirements {
    requirement "R-1" {
        description: "Stop"
        priority: "High"
    }
}

logical_architecture "L" {
    component "Brake" { id: "LC-1" }
}

trace "LC-1" satisfies "R-1" { rationale: "stops" }
"#;

fn baseline_repo() -> Repo {
    let repo = Repo::new();
    repo.write("model.arc", GATED);
    repo.commit("model");
    repo
}

#[test]
fn a_baseline_is_created_listed_verified_and_tampering_is_caught() {
    let repo = baseline_repo();
    let out = repo.ok(&["baseline", "create", "v1", "--standard", "iso26262"]);
    assert!(out.contains("Baseline 'v1' created") && out.contains("baseline/v1"), "{out}");
    for f in ["manifest.json", "gate_report.json", "gate_report.md", "requirements_coverage.csv", "model.sysml", "model.json"] {
        assert!(repo.file(&format!("baselines/v1/{f}")).exists(), "{f}");
    }
    assert_eq!(repo.git(&["cat-file", "-t", "refs/tags/baseline/v1"]).trim(), "tag");
    let manifest: serde_json::Value = serde_json::from_str(&repo.read("baselines/v1/manifest.json")).unwrap();
    assert_eq!(manifest["git"]["commit"].as_str().unwrap(), repo.git(&["rev-parse", "HEAD"]).trim());
    assert_eq!(manifest["compiler_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(manifest["files"].as_array().unwrap().len(), 5);
    assert_eq!(manifest["files"][0]["sha256"].as_str().unwrap().len(), 64);
    assert!(manifest["identities"].as_array().unwrap().iter().any(|i| i["id"] == "LC-1"));
    let list = repo.ok(&["baseline", "list"]);
    assert!(list.contains("v1") && list.contains("iso26262"), "{list}");
    let verified = repo.ok(&["baseline", "verify", "v1"]);
    assert!(verified.contains("VERIFIED") && !verified.contains("NOT VERIFIED"), "{verified}");
    // Baselines/ may stay uncommitted without making the tree dirty for the next one.
    repo.ok(&["baseline", "create", "v2"]);

    // Tamper with a file of the baseline.
    let csv = repo.read("baselines/v1/requirements_coverage.csv");
    repo.write("baselines/v1/requirements_coverage.csv", &csv.replace("R-1", "R-9"));
    let out = repo.arclang(&["baseline", "verify", "v1"]);
    assert!(!out.status.success());
    assert!(text_of(&out).contains("FAIL file requirements_coverage.csv") && text_of(&out).contains("NOT VERIFIED"), "{}", text_of(&out));
    repo.write("baselines/v1/requirements_coverage.csv", &csv);
    assert!(repo.arclang(&["baseline", "verify", "v1"]).status.success());
    // Tamper with the manifest itself.
    let m = repo.read("baselines/v1/manifest.json");
    repo.write("baselines/v1/manifest.json", &m.replace("\"standard\": \"iso26262\"", "\"standard\": \"do178c\""));
    let out = repo.arclang(&["baseline", "verify", "v1"]);
    assert!(text_of(&out).contains("the manifest is not the one the tag was made with"), "{}", text_of(&out));
    // A deleted tag.
    repo.write("baselines/v1/manifest.json", &m);
    repo.git(&["tag", "-d", "baseline/v1"]);
    assert!(text_of(&repo.arclang(&["baseline", "verify", "v1"])).contains("does not exist"));
}

#[test]
fn a_baseline_needs_a_clean_tree_and_a_model_that_compiles() {
    let repo = baseline_repo();
    repo.edit("model.arc", "Stop", "Stop fast");
    let out = repo.arclang(&["baseline", "create", "dirty"]);
    assert!(text_of(&out).contains("working tree is not clean") && text_of(&out).contains("model.arc"), "{}", text_of(&out));
    assert!(!repo.file("baselines/dirty").exists());
    repo.write("model.arc", "this is not a model {");
    repo.commit("broken");
    let out = repo.arclang(&["baseline", "create", "broken"]);
    assert!(!out.status.success() && text_of(&out).contains("does not compile"), "{}", text_of(&out));
    assert!(repo.arclang(&["baseline", "create", "bad name!"]).status.code() == Some(1));
}

#[test]
fn a_baseline_is_never_overwritten() {
    let repo = baseline_repo();
    repo.ok(&["baseline", "create", "v1"]);
    let out = repo.arclang(&["baseline", "create", "v1"]);
    assert!(text_of(&out).contains("already exists"), "{}", text_of(&out));
}

#[test]
fn baseline_diff_compares_models_and_gates() {
    let repo = baseline_repo();
    repo.ok(&["baseline", "create", "v1"]);
    repo.edit("model.arc", "trace \"LC-1\" satisfies \"R-1\" { rationale: \"stops\" }\n", "");
    repo.commit("drop the trace");
    repo.ok(&["baseline", "create", "v2"]);
    let out = repo.ok(&["baseline", "diff", "v1", "v2"]);
    assert!(out.contains("Gate reports") && out.contains("v1: ") && out.contains("v2: "), "{out}");
    assert!(out.contains("trace"), "{out}");
    let json: serde_json::Value = serde_json::from_str(&repo.ok(&["baseline", "diff", "v1", "v2", "-f", "json"])).unwrap();
    assert_eq!(json["a"], "v1");
    assert!(json["diff"]["relationships"].as_array().unwrap().iter().any(|r| r["status"] == "removed"));
}
