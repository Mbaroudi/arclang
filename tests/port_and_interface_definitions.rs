//! Reusable port and interface definitions: a `type` declaring ports types
//! a port (its ports become the ports the port holds, its attributes the
//! port's), and a `type` types an exchange (an interface definition: its
//! protocol, bandwidth and items flow to every exchange of that type).

use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"
type "Lug nuts" {
    interface: "M12"
    port inout "lug nut" { multiplicity: 5 }
}
type "Wheel fastening" {
    protocol: "Torque 120 Nm"
    bandwidth: 1 kbps
}

logical_architecture "Wheels" {
    component "Wheel" {
        id: "LC-WHEEL"
        port inout "nuts" { is: "Lug nuts" }
    }
    component "Hub" {
        id: "LC-HUB"
        port inout "shanks" { is: "Lug nuts" }
    }
    component_exchange "Fastening" {
        is: "Wheel fastening"
        from_port: "LC-WHEEL.nuts.lug nut"
        to_port: "LC-HUB.shanks.lug nut"
    }
}
"#;

fn compiled(source: &str) -> arclang::compiler::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn a_port_typed_by_a_definition_holds_its_ports_and_carries_its_attributes() {
    let result = compiled(MODEL);
    let wheel = &result.ast.logical_architecture[0].components[0];
    let nuts = &wheel.ports[0];
    assert_eq!(nuts.ports.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), vec!["lug nut"]);
    assert_eq!(nuts.interface_type, "M12");
    assert!(result.warnings.iter().all(|w| !w.contains("declares no")), "{:?}", result.warnings);
}

#[test]
fn an_exchange_typed_by_an_interface_definition_carries_its_attributes() {
    let result = compiled(MODEL);
    let exchange = &result.ast.logical_architecture[0].component_exchanges[0];
    assert_eq!(exchange.attributes.get("protocol").and_then(|v| v.as_string()), Some("Torque 120 Nm"));
    assert!(exchange.attributes.contains_key("bandwidth"));
}

#[test]
fn a_port_type_that_does_not_exist_is_an_error() {
    let error = Compiler::new(CompilerConfig::default()).compile_string(&MODEL.replace("port inout \"nuts\" { is: \"Lug nuts\" }", "port inout \"nuts\" { is: \"Bolts\" }")).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("unknown type 'Bolts'"), "{error}");
}

#[test]
fn both_go_through_sysml_v2_and_back() {
    let result = compiled(MODEL);
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("port def 'Lug nuts'"), "{text}");
    assert!(text.contains("inout port nuts : Types::'Lug nuts'"), "{text}");
    assert!(text.contains("interface def 'Wheel fastening'") || text.contains("connection def 'Wheel fastening'"), "{text}");
    let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
    assert!(back.contains("type \"Lug nuts\""), "{back}");
    assert!(back.contains("is: \"Lug nuts\""), "{back}");
    assert!(back.contains("is: \"Wheel fastening\""), "{back}");
}
