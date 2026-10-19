//! Ports that hold ports, and exchanges between more than two ends: the
//! ArcLang language beyond Arcadia's binary exchanges between flat ports,
//! as SysML v2 has them (`port def` with ports, `connect (a, b, c)`).

use arclang::compiler::diagram::{build_diagrams, ViewKind};
use arclang::compiler::{Compiler, CompilerConfig};

const WHEELS: &str = r#"
logical_architecture "LA" {
    component "wheel" {
        port inout "nuts" {
            port inout "lugNut" { multiplicity: 5 }
        }
    }
    component "hub" {
        port inout "shanks" {
            port inout "shank" { multiplicity: 5 }
        }
    }
    component_exchange "fastening" {
        from_port: "wheel.nuts.lugNut"
        to_port: "hub.shanks.shank"
    }
}
"#;

fn compiled(source: &str) -> arclang::compiler::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn a_port_holds_ports_and_an_exchange_names_them() {
    let result = compiled(WHEELS);
    let wheel = &result.ast.logical_architecture[0].components[0];
    assert_eq!(wheel.ports[0].ports[0].name, "lugNut");
    assert!(result.warnings.iter().all(|w| !w.contains("declares no")), "{:?}", result.warnings);
}

#[test]
fn a_port_that_is_not_there_is_reported_however_deep() {
    let source = WHEELS.replace("hub.shanks.shank", "hub.shanks.bolt");
    let error = Compiler::new(CompilerConfig::default()).compile_string(&source).map(|r| r.warnings.join("\n")).unwrap_or_else(|e| e.to_string());
    assert!(error.contains("'hub.shanks' declares no port"), "{error}");
}

#[test]
fn the_logical_view_draws_an_exchange_on_a_sub_port_at_the_outer_port() {
    let result = compiled(WHEELS);
    let set = build_diagrams(&result.ast);
    let lab = set.diagrams.iter().find(|d| d.kind == ViewKind::Lab).expect("a LAB");
    let edge = lab.edges.iter().find(|e| e.label == "fastening").unwrap_or_else(|| panic!("{:?}", set.diagnostics));
    let wheel = lab.nodes.iter().find(|n| n.name == "wheel").unwrap();
    let nuts = wheel.ports.iter().find(|p| p.name == "nuts").expect("the outer port is drawn");
    assert_eq!(edge.source_port.as_deref(), Some(nuts.id.as_str()), "{edge:?}");
    assert!(set.diagnostics.is_empty(), "{:?}", set.diagnostics);
}

const BUS: &str = r#"
logical_architecture "LA" {
    component "a" { port inout "p" }
    component "b" { port inout "q" }
    component "c" { port inout "r" }
    component_exchange "bus" {
        ends: ["a.p", "b.q", "c.r"]
    }
}
"#;

#[test]
fn an_exchange_joins_more_than_two_ends_through_a_junction() {
    let result = compiled(BUS);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    let set = build_diagrams(&result.ast);
    let lab = set.diagrams.iter().find(|d| d.kind == ViewKind::Lab).unwrap();
    let junction = lab.nodes.iter().find(|n| format!("{:?}", n.kind) == "Junction").unwrap_or_else(|| panic!("{:?}", lab.nodes));
    let legs: Vec<_> = lab.edges.iter().filter(|e| e.target == junction.id || e.source == junction.id).collect();
    assert_eq!(legs.len(), 3, "{:?}", lab.edges);
    assert!(set.diagnostics.is_empty(), "{:?}", set.diagnostics);
}

#[test]
fn an_n_ary_exchange_names_ends_that_exist() {
    let source = BUS.replace("\"c.r\"", "\"c.nowhere\"");
    let error = Compiler::new(CompilerConfig::default()).compile_string(&source).map(|r| r.warnings.join("\n")).unwrap_or_else(|e| e.to_string());
    assert!(error.contains("'c' declares no port"), "{error}");
}

#[test]
fn both_come_back_from_their_sysml_v2_export() {
    for source in [WHEELS, BUS] {
        let result = compiled(source);
        let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
        let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
        if source == WHEELS {
            assert!(back.contains("port inout \"nuts\" {"), "{back}");
            assert!(back.contains("from_port: \"LC-wheel.nuts.lugNut\""), "{back}");
            assert!(text.contains("inout port lugNut : 'Data port' [5];"), "{text}");
        } else {
            assert!(text.contains("connect ('LC-a'.p, 'LC-b'.q, 'LC-c'.r)"), "{text}");
            assert!(back.contains("ends: [\"LC-a.p\", \"LC-b.q\", \"LC-c.r\"]"), "{back}");
        }
    }
}
