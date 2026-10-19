//! Product lines: a component `variation: true` holds the components that
//! are its variants (`variant: true`), and a `configuration` selects one
//! variant of every variation. The compiler checks the selection, and a
//! declared diagram draws the architecture of one configuration.

use arclang::compiler::diagram::build_diagrams;
use arclang::compiler::{Compiler, CompilerConfig};

const LINE: &str = r#"
logical_architecture "Car line" {
    component "Car" {
        id: "LC-CAR"
        component "Engine" {
            id: "LC-ENGINE"
            variation: true
            component "Four cylinders" { id: "LC-4CYL" variant: true }
            component "Six cylinders" { id: "LC-6CYL" variant: true }
        }
        component "Gearbox" {
            id: "LC-GEARBOX"
            variation: true
            component "Manual" { id: "LC-MANUAL" variant: true }
            component "Automatic" { id: "LC-AUTO" variant: true }
        }
    }
}

configuration "City" {
    id: "CFG-CITY"
    selects: ["LC-4CYL", "LC-MANUAL"]
}

configuration "Sport" {
    id: "CFG-SPORT"
    selects: ["LC-6CYL", "LC-AUTO"]
}

diagram "City car" { view: lab configuration: "CFG-CITY" }
"#;

fn compile(source: &str) -> Result<arclang::compiler::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(source).map_err(|e| e.to_string())
}

#[test]
fn a_configuration_selects_one_variant_of_every_variation() {
    let result = compile(LINE).unwrap();
    assert_eq!(result.ast.configurations.len(), 2);
    assert_eq!(result.ast.configurations[0].selects, vec!["LC-4CYL", "LC-MANUAL"]);
    assert!(result.warnings.iter().all(|w| !w.contains("variation")), "{:?}", result.warnings);
}

#[test]
fn two_variants_of_one_variation_are_an_error() {
    let error = compile(&LINE.replace("[\"LC-4CYL\", \"LC-MANUAL\"]", "[\"LC-4CYL\", \"LC-6CYL\", \"LC-MANUAL\"]")).err().unwrap_or_default();
    assert!(error.contains("configuration 'City' selects 'LC-4CYL' and 'LC-6CYL', two variants of variation 'LC-ENGINE'"), "{error}");
}

#[test]
fn selecting_what_is_not_a_variant_is_an_error() {
    let error = compile(&LINE.replace("[\"LC-4CYL\", \"LC-MANUAL\"]", "[\"LC-CAR\", \"LC-MANUAL\"]")).err().unwrap_or_default();
    assert!(error.contains("configuration 'City': 'LC-CAR' is not a variant"), "{error}");
}

#[test]
fn a_variation_left_open_and_a_variant_outside_a_variation_are_reported() {
    let open = compile(&LINE.replace("[\"LC-4CYL\", \"LC-MANUAL\"]", "[\"LC-4CYL\"]")).unwrap();
    assert!(open.warnings.iter().any(|w| w.contains("configuration 'City' leaves variation 'LC-GEARBOX' open")), "{:?}", open.warnings);
    let error = compile(&LINE.replace("component \"Car\" {\n        id: \"LC-CAR\"", "component \"Car\" {\n        id: \"LC-CAR\"\n        component \"Roof\" { id: \"LC-ROOF\" variant: true }")).err().unwrap_or_default();
    assert!(error.contains("'LC-ROOF' is a variant outside a variation"), "{error}");
}

#[test]
fn the_diagram_of_a_configuration_draws_only_what_it_selects() {
    let result = compile(LINE).unwrap();
    let set = build_diagrams(&result.ast);
    let city = set.diagrams.iter().find(|d| d.title == "City car").unwrap_or_else(|| panic!("{:?}", set.diagnostics));
    fn names(nodes: &[arclang::compiler::diagram::Node], out: &mut Vec<String>) {
        for n in nodes {
            out.push(n.name.clone());
            names(&n.children, out);
        }
    }
    let mut drawn = Vec::new();
    names(&city.nodes, &mut drawn);
    assert!(drawn.contains(&"Four cylinders".to_string()) && drawn.contains(&"Manual".to_string()), "{drawn:?}");
    assert!(!drawn.contains(&"Six cylinders".to_string()) && !drawn.contains(&"Automatic".to_string()), "{drawn:?}");
}

#[test]
fn variants_go_through_sysml_v2_and_back() {
    let result = compile(LINE).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("variation part <'LC-ENGINE'> Engine"), "{text}");
    assert!(text.contains("variant part <'LC-4CYL'> 'Four cylinders'"), "{text}");
    let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
    assert!(back.contains("variation: true"), "{back}");
    assert!(back.contains("variant: true"), "{back}");
    assert!(back.contains("configuration \"City\""), "{back}");
    assert!(back.contains("selects: [\"LC-4CYL\", \"LC-MANUAL\"]"), "{back}");
}

#[test]
fn a_variation_holds_only_its_variants() {
    let error = compile(&LINE.replace("component \"Manual\" { id: \"LC-MANUAL\" variant: true }", "component \"Manual\" { id: \"LC-MANUAL\" variant: true }\n            component \"Oil pump\" { id: \"LC-PUMP\" }")).err().unwrap_or_default();
    assert!(error.contains("'LC-PUMP' is in a variation but is not a variant"), "{error}");
}

/// SysML v2 lets a variation own nothing but its variants: what it has of
/// its own is a definition typing it, and no feature chain reaches a variant.
#[test]
fn what_a_variation_has_of_its_own_is_its_definition() {
    let source = LINE
        .replace("            variation: true\n            component \"Four", "            variation: true\n            tags: [\"Powertrain\"]\n            port out \"torque\"\n            component \"Four")
        .replace("component \"Four cylinders\" { id: \"LC-4CYL\" variant: true }", "component \"Four cylinders\" { id: \"LC-4CYL\" variant: true power: 90 kW }")
        + "constraint \"Enough power\" { id: \"CST-P\" assert: \"LC-4CYL\".power >= 80 kW }\n";
    let result = compile(&source).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("part def 'Engine variation' {\n                @Powertrain;\n                out port torque"), "{text}");
    assert!(text.contains("variation part <'LC-ENGINE'> Engine : 'Engine variation' {\n                variant part"), "{text}");
    assert!(text.contains("// not an expression of the export: 'LC-4CYL' is a variant or in one"), "{text}");
    let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
    assert!(back.contains("port out \"torque\""), "{back}");
    assert!(back.contains("tags: [\"Powertrain\"]"), "{back}");
    assert!(back.contains("assert: \"LC-4CYL\".power >= 80 kW"), "{back}");
    assert!(!back.contains("type \"Engine variation\""), "{back}");
    compile(&back).unwrap_or_else(|e| panic!("{e}\n{back}"));
    // The reader of the API abstract syntax knows variations too.
    arclang::compiler::sysml_records::from_text(&text).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_variation_inside_a_variant_left_out_is_not_open() {
    let source = LINE.replace(
        "component \"Six cylinders\" { id: \"LC-6CYL\" variant: true }",
        "component \"Six cylinders\" {\n                id: \"LC-6CYL\"\n                variant: true\n                component \"Turbo\" {\n                    id: \"LC-TURBO\"\n                    variation: true\n                    component \"Single\" { id: \"LC-T1\" variant: true }\n                    component \"Twin\" { id: \"LC-T2\" variant: true }\n                }\n            }",
    ).replace("selects: [\"LC-6CYL\", \"LC-AUTO\"]", "selects: [\"LC-6CYL\", \"LC-AUTO\", \"LC-T2\"]")
        .replace("selects: [\"LC-4CYL\", \"LC-MANUAL\"]", "selects: [\"LC-4CYL\", \"LC-MANUAL\", \"LC-MANUAL\"]");
    let result = compile(&source).unwrap();
    assert!(result.warnings.iter().all(|w| !w.contains("LC-TURBO")), "{:?}", result.warnings);
}

#[test]
fn a_variant_selected_by_a_name_two_variations_share_is_an_error() {
    let source = LINE.replace("component \"Manual\" { id: \"LC-MANUAL\"", "component \"Four cylinders\" { id: \"LC-MANUAL\"");
    let error = compile(&source.replace("[\"LC-4CYL\", \"LC-MANUAL\"]", "[\"Four cylinders\"]")).err().unwrap_or_default();
    assert!(error.contains("configuration 'City': 'Four cylinders' names variants of several variations (LC-ENGINE, LC-GEARBOX): select it by id"), "{error}");
}

#[test]
fn a_configuration_leaves_out_variants_by_id_not_by_name() {
    // A function named like a left-out variant stays drawn.
    let source = LINE.replace("component \"Car\" {\n        id: \"LC-CAR\"", "component \"Car\" {\n        id: \"LC-CAR\"\n        component \"Six cylinders\" { id: \"LC-SIX-LABEL\" }");
    let result = compile(&source.replace("component \"Six cylinders\" { id: \"LC-6CYL\" variant: true }", "component \"Six\" { id: \"LC-6CYL\" variant: true }")).unwrap();
    let set = build_diagrams(&result.ast);
    let city = set.diagrams.iter().find(|d| d.title == "City car").unwrap();
    fn ids(nodes: &[arclang::compiler::diagram::Node], out: &mut Vec<String>) {
        for n in nodes {
            out.push(n.id.clone());
            ids(&n.children, out);
        }
    }
    let mut drawn = Vec::new();
    ids(&city.nodes, &mut drawn);
    assert!(drawn.contains(&"LC-SIX-LABEL".to_string()) && !drawn.contains(&"LC-6CYL".to_string()), "{drawn:?}");
}
