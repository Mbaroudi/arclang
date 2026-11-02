//! Documents filled from templates: every directive, the errors a template
//! can make, and the three formats.

use arclang::compiler::document;
use arclang::{Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Pack" {}

system_analysis "Needs" {
    requirement "Range" { id: "REQ-RANGE" description: "Drives 300 km" priority: High }
    requirement "Charge" { id: "REQ-CHARGE" description: "Charges in 30 min" priority: Low }
}

physical_architecture "Hardware" {
    node "Battery" { id: "PN-BATT" mass: 450 kg voltage: 400 V }
    node "Charger" { id: "PN-CHG" mass: 12 kg }
    node "Cable" { id: "PN-CABLE" mass: 2 kg }
}

test_case "Range test" { id: "TC-RANGE" verifies: ["REQ-RANGE"] method: "test" verdict: "pass" }

constraint "Pack within budget" { id: "CST-MASS" assert: "PN-BATT".mass <= 500 kg }

trace "PN-BATT" satisfies "REQ-RANGE"
trace "PN-CHG" satisfies "REQ-CHARGE"
"#;

fn render(template: &str) -> Result<String, String> {
    let result = Compiler::new(CompilerConfig::default()).compile_string(MODEL).unwrap();
    document::render(template, &result, "2026-10-31")
}

#[test]
fn each_directive_is_filled_from_the_model() {
    let text = render("# {{model}} review, {{date}}\n\nThe pack weighs {{value PN-BATT mass}}.\n\n{{elements PhysicalNode where \"mass > 10 kg\" columns mass, voltage}}\n\n{{verdicts}}\n\n{{constraints}}\n\n{{gate}}\n\n{{suspects}}\n\n{{rules}}\n").unwrap();
    assert!(text.starts_with("# Pack review, 2026-10-31\n\nThe pack weighs 450 kg.\n"), "{text}");
    assert!(text.contains("| Id | Name | mass | voltage |\n|---|---|---|---|\n| PN-BATT | Battery | 450 kg | 400 V |\n| PN-CHG | Charger | 12 kg |  |\n"), "{text}");
    assert!(!text.contains("PN-CABLE"), "the cable is under 10 kg");
    assert!(text.contains("| TC-RANGE | Range test | test | REQ-RANGE | pass |  |"), "{text}");
    assert!(text.contains("| CST-MASS | Pack within budget | `\"PN-BATT\".mass <= 500 kg` | 450 kg | 500 kg | holds |"), "{text}");
    assert!(text.contains("**Production gate (ISO26262): FAIL**, 1/2 requirements verified."), "{text}");
    assert!(text.contains("nothing is watched") && text.contains("The model declares no rule."), "{text}");
}

#[test]
fn a_table_is_computed_or_taken_from_the_model() {
    let text = render("## Coverage\n\n{{table requirements_coverage}}\n").unwrap();
    assert!(text.starts_with("## Coverage\n\n2 rows:"), "the template's heading only: {text}");
    assert!(text.contains("| REQ-RANGE | Range |"), "{text}");
}

#[test]
fn a_template_that_asks_for_what_the_model_does_not_hold_is_refused() {
    assert_eq!(render("ok\n{{unknown}}\n").unwrap_err(), "line 2: unknown directive 'unknown' (one of model, date, value, table, elements, verdicts, constraints, gate, suspects, rules)");
    assert_eq!(render("{{value PN-NONE mass}}").unwrap_err(), "line 1: no element 'PN-NONE' in the model");
    assert_eq!(render("{{value PN-BATT color}}").unwrap_err(), "line 1: 'PN-BATT' has no attribute 'color'");
    assert!(render("{{elements Gadget}}").unwrap_err().contains("'Gadget' is no kind of element"));
    assert!(render("{{table nonsense}}").is_err());
    assert!(render("{{model").unwrap_err().contains("is not closed"));
}

#[test]
fn the_document_is_written_as_html_and_as_word() {
    let text = render(document::DEFAULT_TEMPLATE).unwrap();
    let html = document::to_html(&text, "Pack");
    assert!(html.contains("<title>Pack</title>") && html.contains("<h2>Production gate</h2>") && html.contains("<th>Req id</th>"), "{}", &html[..400.min(html.len())]);
    let docx = document::to_docx(&text, "Pack");
    assert_eq!(&docx[..2], b"PK", "a zip");
    let as_text = String::from_utf8_lossy(&docx);
    assert!(as_text.contains("word/document.xml") && as_text.contains("Production gate"), "the document part is stored, readable");
}
