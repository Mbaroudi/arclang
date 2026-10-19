//! Views filtered by metadata: any element may carry `tags` (SysML v2
//! metadata, `#Safety`), and a declared diagram shows the elements of a
//! view that carry a tag (`tagged`), under an element when it says so
//! (`under`). In SysML v2, a `view` exposing and filtering.

use arclang::compiler::diagram::{build_diagrams, Node};
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"
logical_architecture "Car" {
    component "Body" {
        id: "LC-BODY"
        component "Airbag controller" { id: "LC-AIRBAG" tags: ["Safety"] }
        component "Seat heater" { id: "LC-SEAT" }
        component "Belt tensioner" { id: "LC-BELT" tags: ["Safety", "Comfort"] }
    }
    component "Infotainment" { id: "LC-INFO" component "Crash recorder" { id: "LC-EDR" tags: ["Safety"] } }
}

diagram "Body safety" { view: lab under: "LC-BODY" tagged: ["Safety"] }
"#;

fn names(nodes: &[Node], out: &mut Vec<String>) {
    for n in nodes {
        out.push(n.name.clone());
        names(&n.children, out);
    }
}

#[test]
fn a_view_draws_what_carries_its_tags_under_what_it_names() {
    let result = Compiler::new(CompilerConfig::default()).compile_string(MODEL).unwrap_or_else(|e| panic!("{e}"));
    assert!(result.warnings.iter().all(|w| !w.contains("tags")), "{:?}", result.warnings);
    let set = build_diagrams(&result.ast);
    let view = set.diagrams.iter().find(|d| d.title == "Body safety").unwrap_or_else(|| panic!("{:?}", set.diagnostics));
    let mut drawn = Vec::new();
    names(&view.nodes, &mut drawn);
    assert!(drawn.contains(&"Airbag controller".to_string()) && drawn.contains(&"Belt tensioner".to_string()), "{drawn:?}");
    assert!(!drawn.contains(&"Seat heater".to_string()), "{drawn:?}");
    assert!(!drawn.contains(&"Crash recorder".to_string()), "not under the body: {drawn:?}");
}

#[test]
fn tags_and_tagged_views_go_through_sysml_v2_and_back() {
    let result = Compiler::new(CompilerConfig::default()).compile_string(MODEL).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("metadata def Safety;"), "{text}");
    assert!(text.contains("@Safety;"), "{text}");
    assert!(text.contains("view 'Body safety'"), "{text}");
    assert!(text.contains("filter @Safety;"), "{text}");
    let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
    assert!(back.contains("tags: [\"Safety\", \"Comfort\"]") || back.contains("tags: [\"Comfort\", \"Safety\"]"), "{back}");
    assert!(back.contains("diagram \"Body safety\" {\n    view: lab\n    under: \"LC-BODY\"\n    tagged: [\"Safety\"]\n}"), "{back}");
}
