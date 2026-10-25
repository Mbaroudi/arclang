//! CI/CD end to end, on real git repositories built by the tests: `ci init`
//! for each provider, `ci run`, the pull-request report (with goldens in
//! tests/fixtures/cicd/) and the SARIF output (checked against the SARIF
//! 2.1.0 schema).
//!
//! UPDATE_GOLDEN=1 rewrites the goldens; read the diff before committing it.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MODEL: &str = r#"// Demo model for CI/CD.
model Demo {
    name: "Demo"
}

requirements {
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
    functional_chain "Reaction" {
        id: "FC-1"
        involves: ["SF-SENSE", "SF-PLAN"]
        latency_budget: 100 ms
    }
}

logical_architecture "Logical" {
    component "Controller" {
        id: "LC-001"
        description: "Computes the plan"
        function "Compute" { id: "LF-001" }
    }
}

trace "LC-001" satisfies "REQ-001" { rationale: "plans" }
trace "LC-001" satisfies "REQ-002" { rationale: "reacts" }

test_case "TC-1" { verifies: ["REQ-001"] method: "test" description: "range" }
test_case "TC-2" { verifies: ["REQ-002"] method: "test" description: "timing" }
"#;

fn arclang() -> &'static str {
    env!("CARGO_BIN_EXE_arclang")
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

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new() -> Repo {
        let dir = tempfile::tempdir().unwrap();
        git_in(dir.path(), &["init", "-q", "-b", "main"]);
        git_in(dir.path(), &["config", "commit.gpgsign", "false"]);
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
        if let Some(parent) = self.file(name).parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
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

    /// A branch from main with the change committed; the repository is left on it.
    fn branch(&self, name: &str, change: impl Fn(&Repo)) {
        git_in(self.path(), &["checkout", "-q", "-b", name, "main"]);
        change(self);
        self.commit(name);
    }

    fn arclang(&self, args: &[&str]) -> Output {
        Command::new(arclang()).current_dir(self.path()).args(args).output().expect("arclang runs")
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.arclang(args);
        assert!(out.status.success(), "arclang {args:?}: {}{}", text(&out), String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).to_string()
    }
}

fn text(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cicd").join(name)
}

/// Compare with a golden (or write it with UPDATE_GOLDEN=1).
fn golden(name: &str, actual: &str) {
    let path = fixture(name);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} is missing: run with UPDATE_GOLDEN=1", path.display()));
    assert_eq!(expected, actual, "{name} differs from its golden (UPDATE_GOLDEN=1 rewrites it)");
}

/// The text of a report without what changes from one run to the next: the commit hashes and the version.
fn stable(report: &str) -> String {
    let hash = regex::Regex::new(r"\b[0-9a-f]{8}\b").unwrap();
    let version = regex::Regex::new(&format!(r"ArcLang {}", regex::escape(env!("CARGO_PKG_VERSION")))).unwrap();
    let out = hash.replace_all(report, "<commit>");
    version.replace_all(&out, "ArcLang <version>").to_string()
}

fn stable_json(report: &str) -> String {
    let mut value: Value = serde_json::from_str(report).unwrap();
    value["tool_version"] = Value::from("<version>");
    let pretty = serde_json::to_string_pretty(&value).unwrap();
    format!("{}\n", regex::Regex::new(r"\b[0-9a-f]{40}\b").unwrap().replace_all(&pretty, "<commit>"))
}

// ------------------------------------------------------------------ ci init

const PROVIDER_FILES: [(&str, &[&str]); 4] = [
    ("github", &[".github/workflows/arclang.yml"]),
    ("gitlab", &[".gitlab/arclang.gitlab-ci.yml", ".gitlab-ci.yml"]),
    ("azure", &["azure-pipelines.yml"]),
    ("jenkins", &["Jenkinsfile"]),
];

fn yaml(text: &str) -> serde_yaml::Value {
    serde_yaml::from_str(text).unwrap_or_else(|e| panic!("not valid YAML: {e}\n{text}"))
}

fn assert_clean(name: &str, content: &str) {
    assert!(!content.contains("@@"), "{name}: a placeholder is left");
    for secret in ["ghp_", "gho_", "glpat-", "AKIA", "BEGIN PRIVATE KEY", "password", "passwd", "secret:"] {
        assert!(!content.contains(secret), "{name} holds something like a secret: {secret}");
    }
    // Every token comes from the provider's environment, none is written in the file.
    for line in content.lines().filter(|l| l.to_ascii_lowercase().contains("token")) {
        let known = ["github.token", "GH_TOKEN", "System.AccessToken", "SYSTEM_ACCESSTOKEN", "ARCLANG_GITLAB_TOKEN", "PRIVATE-TOKEN", "task.", "token", "Token"];
        assert!(known.iter().any(|k| line.contains(k)), "{name}: an unexpected token line: {line}");
        assert!(!regex::Regex::new(r#"(?i)token\s*[:=]\s*["']?[A-Za-z0-9_\-]{20,}"#).unwrap().is_match(line), "{name}: a literal token: {line}");
    }
    // Actions are pinned to a major version at least: never a branch.
    for line in content.lines().filter(|l| l.trim_start().starts_with("- uses:") || l.trim_start().starts_with("uses:")) {
        let version = line.rsplit('@').next().unwrap_or("");
        assert!(version.starts_with('v') && version.chars().nth(1).is_some_and(|c| c.is_ascii_digit()), "{name}: an unpinned action: {line}");
    }
}

#[test]
fn init_writes_each_providers_pipeline_as_valid_yaml_without_secrets() {
    for (provider, files) in PROVIDER_FILES {
        let repo = Repo::with_model();
        let out = repo.ok(&["ci", "init", "--provider", provider]);
        assert!(out.contains("created"), "{provider}: {out}");
        assert!(out.contains("not run against the provider"), "the output says the files are untested: {out}");
        for file in files {
            let content = repo.read(file);
            assert_clean(file, &content);
            if file.ends_with(".yml") {
                yaml(&content);
            }
        }
    }
}

#[test]
fn the_github_workflow_has_the_stable_job_name_and_the_pinned_version() {
    let repo = Repo::with_model();
    repo.ok(&["ci", "init", "--provider", "github", "--standard", "do178c"]);
    let content = repo.read(".github/workflows/arclang.yml");
    let doc = yaml(&content);
    assert_eq!(doc["jobs"]["gate"]["name"].as_str(), Some("ArcLang gate"));
    assert_eq!(doc["env"]["ARCLANG_VERSION"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert!(content.contains("arclang ci run --standard do178c"), "{content}");
    assert!(content.contains("sha256sum -c"), "the binary is checksum verified");
    assert!(content.contains("upload-sarif"));
    assert!(content.contains("<!-- arclang-pr-report -->"), "the hidden marker finds the comment again");
    assert!(content.contains("--method PATCH") && content.contains("--method POST"), "the comment is updated, else created");
    assert!(content.contains("fetch-depth: 0"));
    let permissions = &doc["permissions"];
    assert_eq!(permissions["security-events"].as_str(), Some("write"));
    assert_eq!(permissions["pull-requests"].as_str(), Some("write"));
}

#[test]
fn init_all_writes_the_four_pipelines_and_models_can_be_named() {
    let repo = Repo::with_model();
    repo.ok(&["ci", "init", "--provider", "all", "--model", "model.arc"]);
    for (_, files) in PROVIDER_FILES {
        for file in files {
            assert!(repo.file(file).is_file(), "{file}");
        }
    }
    assert!(repo.read("Jenkinsfile").contains("arclang ci run --standard iso26262 --sarif arclang-results/arclang.sarif --summary arclang-results/summary.md model.arc"));
    let azure = yaml(&repo.read("azure-pipelines.yml"));
    assert_eq!(azure["jobs"][0]["displayName"].as_str(), Some("ArcLang gate"));
    let gitlab = yaml(&repo.read(".gitlab/arclang.gitlab-ci.yml"));
    assert!(gitlab["arclang-gate"]["script"].is_sequence());
    let root = yaml(&repo.read(".gitlab-ci.yml"));
    assert_eq!(root["include"][0]["local"].as_str(), Some(".gitlab/arclang.gitlab-ci.yml"));
    // The Jenkinsfile is Groovy: its braces balance.
    let jenkins = repo.read("Jenkinsfile");
    assert_eq!(jenkins.matches('{').count(), jenkins.matches('}').count());
}

#[test]
fn dry_run_prints_the_files_and_writes_nothing() {
    let repo = Repo::with_model();
    let out = repo.ok(&["ci", "init", "--provider", "all", "--dry-run"]);
    for path in [".github/workflows/arclang.yml", ".gitlab/arclang.gitlab-ci.yml", "azure-pipelines.yml", "Jenkinsfile"] {
        assert!(out.contains(&format!("--- {path}")), "{path} is shown: {out}");
        assert!(!repo.file(path).exists(), "{path} must not be written");
    }
    assert!(out.contains("nothing was written"));
}

#[test]
fn an_existing_file_is_never_overwritten_without_force() {
    let repo = Repo::with_model();
    repo.write("Jenkinsfile", "pipeline { agent any }\n");
    let out = repo.arclang(&["ci", "init", "--provider", "all"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("Jenkinsfile"), "{}", text(&out));
    assert_eq!(repo.read("Jenkinsfile"), "pipeline { agent any }\n", "the file is untouched");
    assert!(!repo.file(".github/workflows/arclang.yml").exists(), "nothing at all is written when one file is refused");

    let forced = repo.ok(&["ci", "init", "--provider", "jenkins", "--force"]);
    assert!(forced.contains("overwrote Jenkinsfile"), "{forced}");
    assert!(repo.read("Jenkinsfile").contains("ArcLang gate"));

    // Dry run with an existing file says it would be refused.
    let dry = repo.ok(&["ci", "init", "--provider", "jenkins", "--dry-run"]);
    assert!(dry.contains("would be refused without --force"), "{dry}");
}

#[test]
fn the_provider_is_detected_from_the_git_remote() {
    for (url, expected) in [
        ("https://github.com/acme/rocket.git", ".github/workflows/arclang.yml"),
        ("git@github.com:acme/rocket.git", ".github/workflows/arclang.yml"),
        ("https://gitlab.com/acme/rocket.git", ".gitlab/arclang.gitlab-ci.yml"),
        ("git@gitlab.example.org:acme/rocket.git", ".gitlab/arclang.gitlab-ci.yml"),
        ("https://dev.azure.com/acme/rocket/_git/rocket", "azure-pipelines.yml"),
        ("https://acme.visualstudio.com/rocket/_git/rocket", "azure-pipelines.yml"),
    ] {
        let repo = Repo::with_model();
        git_in(repo.path(), &["remote", "add", "origin", url]);
        let out = repo.ok(&["ci", "init"]);
        assert!(repo.file(expected).is_file(), "{url} -> {expected}: {out}");
    }
}

#[test]
fn without_a_recognisable_remote_the_provider_must_be_named() {
    let repo = Repo::with_model();
    let out = repo.arclang(&["ci", "init"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("--provider"), "{}", text(&out));
    git_in(repo.path(), &["remote", "add", "origin", "https://git.example.org/acme/rocket.git"]);
    assert!(!repo.arclang(&["ci", "init"]).status.success(), "an unknown host is not guessed");
}

#[test]
fn an_existing_gitlab_ci_file_is_left_alone() {
    let repo = Repo::with_model();
    repo.write(".gitlab-ci.yml", "stages: [build]\n");
    let out = repo.ok(&["ci", "init", "--provider", "gitlab"]);
    assert_eq!(repo.read(".gitlab-ci.yml"), "stages: [build]\n");
    assert!(out.contains("include:") && out.contains(".gitlab/arclang.gitlab-ci.yml"), "{out}");
}

#[test]
fn init_refuses_values_that_do_not_belong_in_a_pipeline() {
    let repo = Repo::with_model();
    for bad in ["a b.arc", "x;rm -rf.arc", "$(id).arc", "../up.arc"] {
        repo.write("a b.arc", MODEL);
        let out = repo.arclang(&["ci", "init", "--provider", "github", "--model", bad]);
        assert!(!out.status.success(), "{bad}: {}", text(&out));
    }
    assert!(!repo.arclang(&["ci", "init", "--provider", "github", "--model", "missing.arc"]).status.success());
    assert!(!repo.arclang(&["ci", "init", "--provider", "github", "--branch", "main; curl evil"]).status.success());
    assert!(!repo.file(".github/workflows/arclang.yml").exists());
}

// ------------------------------------------------------------------- ci run

#[test]
fn ci_models_lists_entry_points_not_fragments() {
    let repo = Repo::new();
    repo.write("main.arc", "model M {}\nimport \"parts/left.arc\"\n");
    repo.write("parts/left.arc", "logical_architecture \"L\" { component \"A\" { id: \"LC-1\" } }\n");
    repo.write("other/solo.arc", MODEL);
    repo.commit("models");
    let listed = repo.ok(&["ci", "models"]);
    assert_eq!(listed.lines().collect::<Vec<_>>(), vec!["main.arc", "other/solo.arc"]);
}

#[test]
fn ci_run_passes_a_good_model_and_fails_a_bad_one() {
    let repo = Repo::with_model();
    let out = repo.ok(&["ci", "run", "--sarif", "out/a.sarif", "--summary", "out/s.md"]);
    assert!(out.contains("PASS  model.arc") && out.contains("ArcLang gate: PASS"), "{out}");
    assert!(repo.file("out/a.sarif").is_file());
    assert!(repo.read("out/s.md").contains("**PASS**"));

    repo.edit("model.arc", "latency: 20 ms", "latency: 200 ms");
    let bad = repo.arclang(&["ci", "run", "--sarif", "out/a.sarif", "--summary", "out/s.md"]);
    assert!(!bad.status.success(), "a failing gate fails the run: {}", text(&bad));
    assert!(text(&bad).contains("FAIL  model.arc"), "{}", text(&bad));
    assert!(repo.read("out/s.md").contains("**FAIL**"));
    let sarif: Value = serde_json::from_str(&repo.read("out/a.sarif")).unwrap();
    assert!(!sarif["runs"][0]["results"].as_array().unwrap().is_empty(), "the failure is in the SARIF file as well");
}

#[test]
fn ci_run_with_no_model_is_a_failure_not_a_pass() {
    let repo = Repo::new();
    repo.write("README.md", "nothing\n");
    repo.commit("empty");
    let out = repo.arclang(&["ci", "run"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("nothing was checked"));
    assert!(repo.arclang(&["ci", "run", "--allow-empty"]).status.success());
}

#[test]
fn ci_run_does_not_pass_a_model_that_does_not_compile() {
    let repo = Repo::with_model();
    repo.edit("model.arc", "model Demo {", "model Demo {{");
    let out = repo.arclang(&["ci", "run", "--sarif", "out/a.sarif"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("FAIL  model.arc") && text(&out).contains("does not compile"), "{}", text(&out));
}

// -------------------------------------------------------------------- SARIF

fn schema_validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(&std::fs::read_to_string(fixture("sarif-schema-2.1.0.json")).unwrap()).unwrap();
    jsonschema::draft4::new(&schema).expect("the SARIF schema compiles")
}

fn assert_valid_sarif(log: &Value) {
    let validator = schema_validator();
    let errors: Vec<String> = validator.iter_errors(log).map(|e| format!("{} at {}", e, e.instance_path())).collect();
    assert!(errors.is_empty(), "not valid SARIF 2.1.0:\n{}", errors.join("\n"));
}

fn sarif_of(repo: &Repo, args: &[&str]) -> (Value, bool) {
    let out = repo.arclang(args);
    let log: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{args:?} printed no JSON ({e}): {}", text(&out)));
    (log, out.status.success())
}

#[test]
fn sarif_of_a_clean_model_is_valid_and_has_nothing_that_fails() {
    let repo = Repo::with_model();
    let (log, ok) = sarif_of(&repo, &["sarif", "model.arc"]);
    assert!(ok);
    assert_valid_sarif(&log);
    assert_eq!(log["version"], "2.1.0");
    assert_eq!(log["runs"][0]["tool"]["driver"]["name"], "ArcLang");
    // Only the methodology advisory (a note) remains: nothing that fails.
    assert!(log["runs"][0]["results"].as_array().unwrap().iter().all(|r| r["level"] == "note"), "{}", log["runs"][0]["results"]);
    assert_eq!(log["runs"][0]["artifacts"][0]["location"]["uri"], "model.arc", "a clean run still says what it covered");
}

#[test]
fn sarif_carries_gate_blockers_with_rules_locations_and_fix_hints() {
    let repo = Repo::with_model();
    repo.edit("model.arc", "latency: 20 ms", "latency: 200 ms");
    let (log, ok) = sarif_of(&repo, &["sarif", "model.arc"]);
    assert!(!ok, "the gate fails, so does the command");
    assert_valid_sarif(&log);
    let run = &log["runs"][0];
    let results = run["results"].as_array().unwrap();
    let budget = results.iter().find(|r| r["ruleId"] == "gate.timing.budget").expect("the budget blocker is a result");
    assert_eq!(budget["level"], "error");
    let at = &budget["locations"][0]["physicalLocation"];
    assert_eq!(at["artifactLocation"]["uri"], "model.arc");
    let line = at["region"]["startLine"].as_u64().unwrap() as usize;
    assert!(MODEL_LINE_OF_CHAIN.iter().any(|needle| repo.read("model.arc").lines().nth(line - 1).unwrap_or("").contains(needle)), "line {line} is where the chain is declared");
    assert!(budget["message"]["text"].as_str().unwrap().contains("Fix:"), "the message says how to answer");
    assert!(budget["partialFingerprints"]["arclang/v1"].as_str().unwrap().len() == 64);
    // The rule is described, and ruleIndex points at it.
    let index = budget["ruleIndex"].as_u64().unwrap() as usize;
    let rule = &run["tool"]["driver"]["rules"][index];
    assert_eq!(rule["id"], "gate.timing.budget");
    assert!(rule["help"]["text"].as_str().unwrap().len() > 10);
    assert_eq!(rule["defaultConfiguration"]["level"], "error");
    // Stable between runs: the fingerprints do not change.
    let (again, _) = sarif_of(&repo, &["sarif", "model.arc"]);
    assert_eq!(log["runs"][0]["results"], again["runs"][0]["results"]);
}

const MODEL_LINE_OF_CHAIN: [&str; 2] = ["FC-1", "Reaction"];

#[test]
fn sarif_of_a_syntax_error_has_the_line_and_column() {
    let repo = Repo::with_model();
    repo.write("broken.arc", "model B {\n    name: \"B\"\n\nrequirements {\n");
    let (log, ok) = sarif_of(&repo, &["sarif", "broken.arc"]);
    assert!(!ok);
    assert_valid_sarif(&log);
    let result = &log["runs"][0]["results"][0];
    assert_eq!(result["ruleId"], "compile.error");
    assert_eq!(result["level"], "error");
    let region = &result["locations"][0]["physicalLocation"]["region"];
    assert!(region["startLine"].as_u64().unwrap() >= 4, "{region}");
    assert!(region["startColumn"].as_u64().is_some(), "{region}");
    assert_eq!(result["properties"]["locationPrecision"], "exact");
}

#[test]
fn check_and_gate_take_format_sarif() {
    let repo = Repo::with_model();
    let (check, ok) = sarif_of(&repo, &["check", "model.arc", "--format", "sarif"]);
    assert!(ok);
    assert_valid_sarif(&check);
    let (gate, ok) = sarif_of(&repo, &["gate", "model.arc", "--format", "sarif"]);
    assert!(ok);
    assert_valid_sarif(&gate);

    repo.edit("model.arc", "latency: 20 ms", "latency: 200 ms");
    let (gate, ok) = sarif_of(&repo, &["gate", "model.arc", "--format", "sarif"]);
    assert!(!ok, "gate --format sarif exits like gate");
    assert_valid_sarif(&gate);
    assert!(gate["runs"][0]["results"].as_array().unwrap().iter().all(|r| r["ruleId"].as_str().unwrap().starts_with("gate.") || r["properties"]["gateCheck"].is_string()), "the gate's SARIF holds the gate's findings");
    let (check, ok) = sarif_of(&repo, &["check", "model.arc", "--format", "sarif"]);
    assert!(ok, "the model compiles: check passes");
    assert_valid_sarif(&check);
}

#[test]
fn sarif_fail_on_never_always_exits_zero() {
    let repo = Repo::with_model();
    repo.edit("model.arc", "latency: 20 ms", "latency: 200 ms");
    assert!(repo.arclang(&["sarif", "model.arc", "--fail-on", "never"]).status.success());
    assert!(!repo.arclang(&["sarif", "model.arc"]).status.success());
}

// ------------------------------------------------------------- report --pr

fn report(repo: &Repo, extra: &[&str]) -> (String, Output) {
    let mut args = vec!["report", "--pr", "--from", "main"];
    args.extend_from_slice(extra);
    let out = repo.arclang(&args);
    (String::from_utf8_lossy(&out.stdout).to_string(), out)
}

fn slow(repo: &Repo) {
    repo.edit("model.arc", "latency: 20 ms", "latency: 200 ms");
}

#[test]
fn a_new_blocker_fails_the_report_and_matches_its_goldens() {
    let repo = Repo::with_model();
    repo.branch("slow", slow);
    let (md, out) = report(&repo, &["--out-dir", "out", "--output", "out/report.md", "--note", "Pictures: see the artifact."]);
    assert_eq!(out.status.code(), Some(1), "a new blocker is a non-zero exit: {}", text(&out));
    let md = std::fs::read_to_string(repo.file("out/report.md")).unwrap();
    assert!(md.starts_with("<!-- arclang-pr-report -->\n"), "the hidden marker is first");
    assert!(md.contains("## ArcLang gate: **FAIL**"));
    assert!(!md.contains("ArcLang gate: **PASS**"), "a failing gate is never reported as a pass");
    assert!(md.contains("New blockers:") && md.contains("timing.budget"));
    assert!(md.contains("Production gate: **PASS** -> **FAIL (2 blockers)**") || md.contains("Production gate: **PASS** -> **FAIL"), "{md}");
    assert!(md.contains("![before](diagrams/model-arc-before.svg)") && md.contains("![after](diagrams/model-arc-after.svg)"), "the pictures are linked by relative path");
    assert!(md.contains("Pictures: see the artifact."));
    golden("pr_report_blocker.md", &stable(&md));
    golden("pr_report_blocker_after.svg", &std::fs::read_to_string(repo.file("out/diagrams/model-arc-after.svg")).unwrap());
    golden("pr_report_blocker_before.svg", &std::fs::read_to_string(repo.file("out/diagrams/model-arc-before.svg")).unwrap());
    drop(md);

    let (json, _) = report(&repo, &["--format", "json"]);
    golden("pr_report_blocker.json", &stable_json(&json));
    let (html, _) = report(&repo, &["--format", "html"]);
    golden("pr_report_blocker.html", &stable(&html));
    let _ = out;
}

#[test]
fn fail_on_never_changes_the_exit_status_not_the_verdict() {
    let repo = Repo::with_model();
    repo.branch("slow", slow);
    let (md, out) = report(&repo, &["--fail-on", "never"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(md.contains("## ArcLang gate: **FAIL**"), "the gate fails, the report says so even when the exit is zero");
    let (json, _) = report(&repo, &["--format", "json", "--fail-on", "never"]);
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["status"], "fail");
    assert_eq!(value["fails_the_check"], false);
    assert!(!report(&repo, &["--fail-on", "major"]).1.status.success());
    assert!(!report(&repo, &["--fail-on", "sometimes"]).1.status.success(), "an unknown level is refused");
}

#[test]
fn an_unchanged_model_passes() {
    let repo = Repo::with_model();
    repo.branch("docs", |r| r.write("NOTES.md", "notes\n"));
    let (md, out) = report(&repo, &["--out-dir", "out"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(md.contains("## ArcLang gate: **PASS**"));
    assert!(md.contains("No semantic change to this model."));
    assert!(!repo.file("out/diagrams").exists(), "no change, no pictures");
    golden("pr_report_unchanged.md", &stable(&md));
}

#[test]
fn a_blocker_that_was_already_there_is_not_new_but_the_gate_still_fails() {
    let repo = Repo::new();
    repo.write("model.arc", MODEL.replacen("latency: 20 ms", "latency: 200 ms", 1).as_str());
    repo.commit("already slow");
    repo.branch("docs", |r| r.edit("model.arc", "description: \"Computes the plan\"", "description: \"Computes the plan, better\""));
    let (md, out) = report(&repo, &[]);
    assert!(out.status.success(), "no new blocker: the exit is zero under --fail-on blocker: {}", text(&out));
    assert!(md.contains("## ArcLang gate: **FAIL**"), "but the gate fails, and the report says so");
    assert!(md.contains("Blockers already there at the base"));
    assert!(!md.contains("New blockers:"));
    golden("pr_report_preexisting.md", &stable(&md));
}

#[test]
fn a_resolved_blocker_is_listed_and_the_gate_passes() {
    let repo = Repo::new();
    repo.write("model.arc", MODEL.replacen("latency: 20 ms", "latency: 200 ms", 1).as_str());
    repo.commit("slow");
    repo.branch("fix", |r| r.edit("model.arc", "latency: 200 ms", "latency: 20 ms"));
    let (md, out) = report(&repo, &[]);
    assert!(out.status.success());
    assert!(md.contains("## ArcLang gate: **PASS**"));
    assert!(md.contains("Resolved by this change:"));
    golden("pr_report_resolved.md", &stable(&md));
}

#[test]
fn a_new_model_is_reported_as_new() {
    let repo = Repo::with_model();
    repo.branch("second", |r| r.write("second.arc", &MODEL.replace("Demo", "Second")));
    let (md, out) = report(&repo, &["--fail-on", "never", "--out-dir", "out"]);
    assert!(out.status.success());
    assert!(md.contains("new model"), "{md}");
    assert!(md.contains("This model is new in the pull request"));
}

#[test]
fn a_head_that_does_not_compile_fails_and_says_why() {
    let repo = Repo::with_model();
    repo.branch("broken", |r| r.edit("model.arc", "model Demo {", "model Demo {{"));
    let (md, out) = report(&repo, &[]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(md.contains("## ArcLang gate: **FAIL**"));
    assert!(md.contains("the model does not compile at the head"));
    assert!(!md.contains("ArcLang gate: **PASS**"));
}

#[test]
fn no_model_is_neither_a_pass_nor_silent() {
    let repo = Repo::new();
    repo.write("README.md", "x\n");
    repo.commit("init");
    repo.branch("more", |r| r.write("NOTES.md", "y\n"));
    let (md, out) = report(&repo, &[]);
    assert!(!out.status.success());
    assert!(md.contains("no model found") && md.contains("This is not a pass"));
    assert!(report(&repo, &["--allow-empty"]).1.status.success());
}

#[test]
fn a_removed_model_file_is_named() {
    let repo = Repo::with_model();
    repo.write("old.arc", &MODEL.replace("Demo", "Old"));
    repo.commit("a second model");
    repo.branch("drop", |r| {
        std::fs::remove_file(r.file("old.arc")).unwrap();
    });
    let (md, _) = report(&repo, &["--fail-on", "never"]);
    assert!(md.contains("Removed from the base: `old.arc`"), "{md}");
}

#[test]
fn a_long_report_stays_under_the_comment_limit() {
    let repo = Repo::with_model();
    let mut big = String::new();
    for i in 0..1500 {
        big.push_str(&format!("logical_architecture \"L{i}\" {{ component \"C{i}\" {{ id: \"LC-X{i}\" description: \"{}\" }} }}\n", "word ".repeat(40)));
    }
    repo.branch("big", |r| {
        let model = r.read("model.arc");
        r.write("model.arc", &format!("{model}\n{big}"));
    });
    let (md, _) = report(&repo, &["--fail-on", "never"]);
    assert!(md.len() < 65_000, "the comment is {} characters", md.len());
    assert!(md.contains("<!-- arclang-pr-report -->"));
}

#[test]
fn the_report_needs_a_git_repository_and_a_known_base() {
    let repo = Repo::with_model();
    let (_, out) = report(&repo, &["--from", "nope"]);
    let all = Command::new(arclang()).current_dir(repo.path()).args(["report", "--pr", "--from", "nope"]).output().unwrap();
    assert!(!all.status.success() && text(&all).contains("unknown revision"), "{}", text(&all));
    let _ = out;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("m.arc"), MODEL).unwrap();
    let outside = Command::new(arclang()).current_dir(dir.path()).args(["report", "--pr", "--from", "main"]).output().unwrap();
    assert!(!outside.status.success() && text(&outside).contains("git repository"), "{}", text(&outside));
}

// ------------------------------------------------------------- simulations

#[test]
fn ci_run_without_openmodelica_reports_the_simulations_as_inconclusive_never_as_passed() {
    let repo = Repo::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    repo.write("ev.arc", &std::fs::read_to_string(root.join("examples/automotive/ev_range_simulation.arc")).unwrap());
    repo.write("physics/EVPhysics.mo", &std::fs::read_to_string(root.join("examples/automotive/physics/EVPhysics.mo")).unwrap());
    repo.commit("ev");
    let out = Command::new(arclang())
        .current_dir(repo.path())
        .env("ARCLANG_OMC", "/nonexistent/omc")
        .args(["ci", "run", "--sarif", "out/a.sarif"])
        .output()
        .unwrap();
    let said = text(&out);
    assert!(said.contains("verification.inconclusive"), "{said}");
    let log: Value = serde_json::from_str(&repo.read("out/a.sarif")).unwrap();
    assert_valid_sarif(&log);
    let inconclusive: Vec<&Value> = log["runs"][0]["results"].as_array().unwrap().iter().filter(|r| r["ruleId"] == "verification.inconclusive").collect();
    assert!(!inconclusive.is_empty(), "an unrun simulation is a result, not silence");
    assert!(inconclusive.iter().all(|r| r["level"] == "warning"));
    assert!(inconclusive[0]["message"]["text"].as_str().unwrap().contains("OpenModelica not available"));
    // With --verify never nothing is simulated and nothing is claimed about it.
    let never = Command::new(arclang()).current_dir(repo.path()).env("ARCLANG_OMC", "/nonexistent/omc").args(["ci", "run", "--verify", "never"]).output().unwrap();
    assert!(!text(&never).contains("verification.inconclusive: "));
}
