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
