use super::graph::LinkKind;
use super::*;
use crate::compiler::workbench::{compile_text, EditRequest};

const BATTERY: &str = include_str!("../../../tests/fixtures/impact/battery.arc");
const AEB: &str = include_str!("../../../examples/complete_emergency_braking_simple.arc");

fn compiled(source: &str) -> CompilationResult {
    compile_text(source).expect("the model compiles")
}

fn options() -> Options {
    Options::default()
}

fn only(kinds: &[LinkKind]) -> Options {
    Options { kinds: kinds.to_vec(), ..Options::default() }
}

fn impact(source: &str, seeds: &[&str], options: &Options) -> ImpactReport {
    let result = compiled(source);
    analyze(&result, "model", &seeds.iter().map(|s| s.to_string()).collect::<Vec<_>>(), options).expect("analysis")
}

fn item<'r>(report: &'r ImpactReport, id: &str) -> &'r Impacted {
    report.impacted.iter().find(|i| i.id == id).unwrap_or_else(|| panic!("{id} is not impacted: {:?}", report.impacted.iter().map(|i| &i.id).collect::<Vec<_>>()))
}

fn ids(report: &ImpactReport) -> Vec<&str> {
    report.impacted.iter().map(|i| i.id.as_str()).collect()
}

fn what_if_of(source: &str, requests: Vec<EditRequest>) -> ImpactReport {
    what_if(None, source, requests, &options()).expect("the what-if compiles")
}

fn set(element: &str, key: &str, value: &str) -> EditRequest {
    EditRequest::Set { element: element.to_string(), key: key.to_string(), value: value.to_string() }
}

#[test]
fn a_requirement_then_its_verification_explain_the_path() {
    let report = impact(BATTERY, &["LC-BATT"], &options());
    let test = item(&report, "TC-R");
    assert_eq!(test.why, "LC-BATT --satisfies--> REQ-R --verified by--> TC-R");
    assert_eq!(test.distance, 2);
    assert_eq!(test.category, Category::Test);
    assert_eq!(test.severity, ImpactSeverity::Major);
    assert_eq!(test.path.iter().map(|s| s.kind.as_str()).collect::<Vec<_>>(), ["satisfies", "verifies"]);
}

/// Every kind of link, followed alone, reaches what it links.
#[test]
fn each_kind_of_link_is_followed_on_its_own() {
    let cases: [(LinkKind, &str, &str, &str, &str); 15] = [
        (LinkKind::Realization, BATTERY, "LF-A", "SF-A", "realizes"),
        (LinkKind::Allocation, BATTERY, "LC-BATT", "LF-A", "performs"),
        (LinkKind::Deployment, BATTERY, "LC-BATT", "PN-1", "deployed on"),
        (LinkKind::Exchange, BATTERY, "SF-A", "AtoB", "sends"),
        (LinkKind::Interface, BATTERY, "SF-A", "SF-A.o", "has port"),
        (LinkKind::Chain, BATTERY, "SF-A", "FC-1", "involved in"),
        (LinkKind::Satisfies, BATTERY, "REQ-R", "LC-BATT", "satisfied by"),
        (LinkKind::Verifies, BATTERY, "REQ-R", "TC-R", "verified by"),
        (LinkKind::Safety, BATTERY, "REQ-R", "HAZ-1", "mitigates"),
        (LinkKind::Containment, AEB, "SF-AcquireSensorData", "SF-Read_radar", "contains"),
        (LinkKind::Typing, BATTERY, "Sensor unit", "LC-OTHER", "type of"),
        (LinkKind::Constraint, BATTERY, "FC-1", "CST-1", "constrained by"),
        (LinkKind::View, BATTERY, "LC-BATT", "diagram:Battery view", "shown in"),
        (LinkKind::Behavior, AEB, "LC-RadarSensor", "LC-SensorFusion", "sends 'raw radar targets' to"),
        (LinkKind::Reference, AEB, "CAP-001", "MIS-001", "contributes to"),
    ];
    for (kind, source, seed, reached, relation) in cases {
        let report = impact(source, &[seed], &only(&[kind]));
        let found = item(&report, reached);
        assert_eq!(found.distance, 1, "{kind:?}: {}", found.why);
        assert_eq!(found.path[0].relation, relation, "{kind:?}");
        assert_eq!(found.path[0].kind, kind.as_str());
        assert!(report.impacted.iter().all(|i| i.path.iter().all(|s| s.kind == kind.as_str())), "{kind:?} followed another kind");
    }
}

#[test]
fn a_table_lists_and_a_diagram_shows_but_neither_propagates() {
    let report = impact(BATTERY, &["LC-BATT"], &options());
    assert_eq!(item(&report, "Coverage").category, Category::View);
    assert_eq!(item(&report, "diagram:Battery view").severity, ImpactSeverity::Minor);
    // The table lists LC-LOG too: not reached through the table.
    assert!(!ids(&report).contains(&"LC-LOG"), "{:?}", ids(&report));
}

#[test]
fn the_depth_bounds_the_walk() {
    let one = impact(BATTERY, &["LC-BATT"], &Options { depth: 1, ..options() });
    assert!(one.impacted.iter().all(|i| i.distance == 1));
    assert!(ids(&one).contains(&"REQ-R") && !ids(&one).contains(&"TC-R"));
    let four = impact(BATTERY, &["LC-BATT"], &options());
    assert_eq!(item(&four, "CST-1").distance, 4);
    assert_eq!(four.summary.max_distance, 4);
}

#[test]
fn a_cycle_is_walked_once() {
    let source = r#"model C { name: "C" }
logical_architecture "LA" {
    component "A" { id: "LC-A" port out "o" port in "i" }
    component "B" { id: "LC-B" port out "o" port in "i" }
    component_exchange "AB" { from_port: "LC-A.o" to_port: "LC-B.i" }
    component_exchange "BA" { from_port: "LC-B.o" to_port: "LC-A.i" }
}
"#;
    let report = impact(source, &["LC-A"], &Options { depth: 12, ..options() });
    let mut seen = ids(&report);
    let count = seen.len();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), count, "an element listed twice");
    assert!(!seen.contains(&"LC-A"), "the seed is not its own impact");
    assert_eq!(item(&report, "LC-B").distance, 2);
}

#[test]
fn an_exchange_reaches_its_consumer_and_stops_there() {
    let report = impact(AEB, &["LC-RadarSensor"], &options());
    let fusion = report.impacted.iter().find(|i| i.id == "LC-SensorFusion.RadarDataConsumer").unwrap();
    assert_eq!(fusion.why, "LC-RadarSensor --sends--> Radar Data --received by--> LC-SensorFusion.RadarDataConsumer");
    // Not the consumer's other interfaces.
    assert!(!ids(&report).contains(&"Fused Data"), "{:?}", ids(&report));
    // Upstream (the node it is deployed on) is re-checked, not walked through: no physical link.
    assert!(!ids(&report).contains(&"CANBus"));
}

#[test]
fn categories_and_severities() {
    let report = impact(BATTERY, &["LC-BATT"], &options());
    let requirement = item(&report, "REQ-R");
    assert_eq!((requirement.category, requirement.severity), (Category::Requirement, ImpactSeverity::Blocker));
    assert!(requirement.reason.contains("ASIL-D"), "{}", requirement.reason);
    assert_eq!(item(&report, "HAZ-1").category, Category::Safety);
    assert_eq!(item(&report, "FC-1").category, Category::Budget);
    assert_eq!(item(&report, "LF-A").severity, ImpactSeverity::Minor);
    let ports = impact(BATTERY, &["SF-A"], &options());
    let port = item(&ports, "SF-A.o");
    assert_eq!((port.category, port.severity), (Category::Interface, ImpactSeverity::Major));
    let types = impact(BATTERY, &["Sensor unit"], &options());
    assert_eq!(item(&types, "LC-OTHER").category, Category::TypeUser);
    assert_eq!(report.summary.blockers, report.impacted.iter().filter(|i| i.severity == ImpactSeverity::Blocker).count());
}

#[test]
fn a_gate_blocker_naming_an_element_makes_it_a_blocker() {
    // TC-2 fails in the tables' fixture: REQ-2 is a gate blocker.
    let source = include_str!("../../../tests/fixtures/tables/braking.arc");
    let report = impact(source, &["LC-BRAKE"], &options());
    let requirement = item(&report, "REQ-2");
    assert_eq!(requirement.severity, ImpactSeverity::Blocker);
    assert!(requirement.reason.starts_with("gate blocker"), "{}", requirement.reason);
}

#[test]
fn the_direction_and_the_kinds_filter_the_walk() {
    let down = impact(BATTERY, &["REQ-R"], &Options { direction: Direction::Downstream, ..options() });
    assert!(ids(&down).contains(&"LC-BATT") && ids(&down).contains(&"TC-R"));
    let up = impact(BATTERY, &["LC-BATT"], &Options { direction: Direction::Upstream, ..options() });
    assert!(ids(&up).contains(&"REQ-R") && !ids(&up).contains(&"LF-A"));
    assert_eq!(Options::parse_kinds(&["satisfies, verification".to_string()]).unwrap(), vec![LinkKind::Satisfies, LinkKind::Verifies]);
    assert!(Options::parse_kinds(&["gravity".to_string()]).is_err());
}

#[test]
fn an_unknown_seed_is_refused() {
    let result = compiled(BATTERY);
    let error = analyze(&result, "m", &["LC-NOPE".to_string()], &options()).unwrap_err();
    assert!(error.contains("LC-NOPE"), "{error}");
}

#[test]
fn the_same_model_gives_the_same_report() {
    let a = serde_json::to_string(&impact(AEB, &["LC-RadarSensor"], &options())).unwrap();
    let b = serde_json::to_string(&impact(AEB, &["LC-RadarSensor"], &options())).unwrap();
    assert_eq!(a, b);
}

#[test]
fn a_latency_what_if_flips_the_constraint_and_breaks_the_budget() {
    let report = what_if_of(BATTERY, vec![set("SF-A", "latency", "40 ms")]);
    let w = report.what_if.as_ref().unwrap();
    assert!(w.gate_before.passed && !w.gate_after.passed);
    assert!(w.new_blockers.iter().any(|b| b.contains("exceeds the 50.0 ms budget")), "{:?}", w.new_blockers);
    let constraint = w.constraints.iter().find(|c| c.id == "CST-1").unwrap();
    assert!(constraint.flipped && constraint.before == Some(true) && constraint.after == Some(false));
    assert_eq!((constraint.left_before.as_str(), constraint.left_after.as_str()), ("40 ms", "60 ms"));
    let budget = w.budgets.iter().find(|b| b.chain == "FC-1").unwrap();
    assert_eq!((budget.before_ms, budget.after_ms, budget.ok_after), (Some(40.0), Some(60.0), false));
    let cst = item(&report, "CST-1");
    assert_eq!(cst.severity, ImpactSeverity::Blocker);
    assert!(cst.why.ends_with("--constrained by--> CST-1"), "{}", cst.why);
    assert_eq!(report.seeds.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["SF-A"]);
    assert_eq!(w.diff.modified.len(), 2);
}

#[test]
fn a_delete_what_if_leaves_a_requirement_unsatisfied() {
    let result = compiled(BATTERY);
    let requests = cascade_delete(&result, "LC-LOG").unwrap();
    assert!(matches!(requests.first(), Some(EditRequest::RemoveTrace { .. })));
    assert!(matches!(requests.last(), Some(EditRequest::Remove { .. })));
    let report = what_if_of(BATTERY, requests);
    let w = report.what_if.as_ref().unwrap();
    let delta = w.coverage.iter().find(|c| c.requirement == "REQ-L").unwrap();
    assert_eq!((delta.before.status.as_str(), delta.after.status.as_str()), ("covered", "unsatisfied"));
    assert!(w.new_blockers.iter().any(|b| b.contains("'REQ-L' is satisfied by no architecture element")));
    let requirement = item(&report, "REQ-L");
    assert_eq!(requirement.severity, ImpactSeverity::Blocker);
    assert_eq!(w.coverage_before.satisfied, 2);
    assert_eq!(w.coverage_after.satisfied, 1);
    assert_eq!(report.seeds[0].id, "LC-LOG");
}

#[test]
fn a_change_makes_a_stated_verdict_stale() {
    let report = what_if_of(BATTERY, vec![set("LC-BATT", "description", "\"A bigger pack\"")]);
    let verdict = report.what_if.as_ref().unwrap().verdicts.iter().find(|v| v.test == "TC-R").expect("TC-R is listed");
    assert!(verdict.stale);
    assert_eq!((verdict.before.as_str(), verdict.after.as_str()), ("pass", "pass"));
    assert!(verdict.why.contains("--verified by--> TC-R"), "{}", verdict.why);
}

#[test]
fn an_edit_that_does_not_compile_is_refused() {
    let result = compiled(AEB);
    let requests = cascade_delete(&result, "RadarSensor").unwrap();
    let error = what_if(None, AEB, requests, &options()).unwrap_err();
    assert!(error.contains("refused") && error.contains("scenario 'EmergencyStop'"), "{error}");
    let error = what_if(None, BATTERY, vec![set("LC-NOPE", "x", "1")], &options()).unwrap_err();
    assert!(error.contains("refused"), "{error}");
}

#[test]
fn since_a_revision_of_a_git_repository() {
    let dir = std::env::temp_dir().join(format!("arclang-impact-since-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let git = |args: &[&str]| {
        let status = std::process::Command::new("git").arg("-C").arg(&dir).args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"]).args(args).output().expect("git runs");
        assert!(status.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&status.stderr));
    };
    git(&["init", "-q"]);
    let file = dir.join("battery.arc");
    std::fs::write(&file, BATTERY).unwrap();
    git(&["add", "battery.arc"]);
    git(&["commit", "-q", "-m", "model"]);
    let changed = BATTERY.replace("function \"Acquire\" { id: \"SF-A\" latency: 20 ms", "function \"Acquire\" { id: \"SF-A\" latency: 45 ms");
    assert_ne!(changed, BATTERY);
    std::fs::write(&file, &changed).unwrap();
    let report = since::since(&file, &changed, "HEAD", &options()).expect("since HEAD");
    assert_eq!(report.since.as_deref(), Some("HEAD"));
    assert_eq!(report.seeds.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["SF-A"]);
    let w = report.what_if.as_ref().unwrap();
    assert!(w.constraints.iter().any(|c| c.id == "CST-1" && c.flipped));
    assert_eq!(item(&report, "CST-1").severity, ImpactSeverity::Blocker);
    let error = since::since(&file, &changed, "no-such-branch", &options()).unwrap_err();
    assert!(error.contains("unknown revision"), "{error}");
    assert!(since::since(&file, &changed, "--output=x", &options()).is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_reports_render() {
    let report = what_if_of(BATTERY, vec![set("SF-A", "latency", "40 ms")]);
    let text = report::text(&report);
    assert!(text.starts_with("Impact analysis for 'the proposed change (SF-A)':"), "{text}");
    assert!(text.contains("gate ISO26262 PASS → FAIL"));
    let markdown = report::markdown(&report);
    assert!(markdown.contains("## Constraints") && markdown.contains("| CST-1 |"), "{markdown}");
    let html = report::html(&report);
    assert!(html.contains("class=\"chip blocker\"") && html.contains("prefers-color-scheme: dark"));
    let xlsx = report::xlsx(&report);
    assert_eq!(&xlsx[..2], b"PK");
    let names = String::from_utf8_lossy(&xlsx);
    for sheet in ["Summary", "Impact", "Gate", "Constraints", "Budgets", "Coverage", "Changes"] {
        assert!(names.contains(&format!("name=\"{sheet}\"")), "no sheet {sheet}");
    }
    assert_eq!(report::xlsx(&report), xlsx, "the same report gives the same bytes");
    let tsv = report::tsv(&report);
    assert!(tsv.starts_with("Element\tName\tKind\tLayer\tCategory\tSeverity\tDistance\tReason\tWhy\n"));
}

#[test]
fn no_scratch_path_reaches_a_verdict_or_the_gate() {
    let dir = std::env::temp_dir().join(format!("arclang-impact-scratch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("physics")).unwrap();
    let examples = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/automotive");
    let model = std::fs::read_to_string(examples.join("ev_range_simulation.arc")).unwrap();
    std::fs::copy(examples.join("physics/EVPhysics.mo"), dir.join("physics/EVPhysics.mo")).unwrap();
    let file = dir.join("ev.arc");
    std::fs::write(&file, &model).unwrap();
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git").arg("-C").arg(&dir).args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"]).args(args).output().expect("git runs");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    };
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "model"]);
    // Never simulated here: every verdict says which file to verify.
    let base = crate::compiler::semgit::rich::load(&file, &crate::compiler::semgit::rich::Source::Git("HEAD".into())).expect("loads at HEAD");
    let reasons: Vec<&String> = base.result.semantic_model.simulations.values().map(|s| &s.reason).collect();
    assert!(!reasons.is_empty() && reasons.iter().all(|r| r.contains(&file.display().to_string())), "{reasons:?}");
    let changed = model.replacen("stop_time: 3.2 h", "stop_time: 3.3 h", 1);
    assert_ne!(changed, model);
    std::fs::write(&file, &changed).unwrap();
    let report = since::since(&file, &changed, "HEAD", &options()).expect("since HEAD");
    for text in [report::text(&report), report::markdown(&report), report::tsv(&report)] {
        assert!(!text.contains("arclang-rev-") && !text.contains(".edit.arc"), "{text}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
