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

#[test]
fn a_tagged_view_of_a_whole_level_and_a_single_tag_go_through_sysml_v2() {
    let source = MODEL.replace("diagram \"Body safety\" {", "diagram \"All safety\" { view: lab tagged: \"Safety\" }\ndiagram \"Body safety\" {");
    let result = Compiler::new(CompilerConfig::default()).compile_string(&source).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("view 'All safety' {\n        attribute 'view' : String = \"lab\";\n        filter @Safety;"), "{text}");
    let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
    assert!(back.contains("diagram \"All safety\" {\n    view: lab\n    tagged: [\"Safety\"]\n}"), "{back}");
}

#[test]
fn every_tag_written_is_declared() {
    let source = format!("{MODEL}\noperational_analysis \"Need\" {{ entity \"Driver\" {{ tags: [\"Human\"] }} }}\n");
    let result = Compiler::new(CompilerConfig::default()).compile_string(&source).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("@Human;"), "{text}");
    assert!(text.contains("metadata def Human;"), "{text}");
}

#[test]
fn an_untagged_element_named_like_a_tagged_one_is_not_drawn() {
    let source = MODEL.replace("component \"Crash recorder\" {", "component \"Airbag controller\" { id: \"LC-AIRBAG-2\" }\n        component \"Crash recorder\" {");
    let result = Compiler::new(CompilerConfig::default()).compile_string(&source.replace("under: \"LC-BODY\"\n", "")).unwrap();
    let set = build_diagrams(&result.ast);
    let diagram = set.diagrams.iter().find(|d| d.title == "Body safety").unwrap();
    fn ids(nodes: &[Node], out: &mut Vec<String>) {
        for n in nodes {
            out.push(n.id.clone());
            ids(&n.children, out);
        }
    }
    let mut drawn = Vec::new();
    ids(&diagram.nodes, &mut drawn);
    assert!(drawn.contains(&"LC-AIRBAG".to_string()) && !drawn.contains(&"LC-AIRBAG-2".to_string()), "{drawn:?}");
}

#[test]
fn a_view_under_no_exported_element_is_said_not_widened() {
    let source = MODEL.replace("under: \"LC-BODY\"", "under: \"LC-NOWHERE\"");
    let result = Compiler::new(CompilerConfig::default()).compile_string(&source).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("// view 'Body safety' left out: under 'LC-NOWHERE' is no exported element"), "{text}");
    assert!(!text.contains("view 'Body safety' {"), "{text}");
}
