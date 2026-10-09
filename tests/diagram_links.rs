//! Links between elements across views: what lets a reader go from an
//! operational activity to the function that realizes it, then to the
//! component it is allocated to. A link is a trace the author wrote; the
//! diagram model resolves its ends to drawn elements and never adds one.

use arclang::compiler::diagram::{build_diagrams, DiagramSet, Link};
use arclang::compiler::{Compiler, CompilerConfig};

fn diagrams_of(source: &str) -> DiagramSet {
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(source)
        .expect("test model compiles");
    build_diagrams(&result.ast)
}

fn link(kind: &str, source: &str, target: &str) -> Link {
    Link {
        kind: kind.to_string(),
        source: source.to_string(),
        target: target.to_string(),
    }
}

const LAYERS: &str = r#"
operational_analysis "OA" {
    operational_entity "Vehicle" {
        id: "OE-1"
        operational_activity "Monitor Road" { id: "OA-1" }
    }
}
system_analysis "SA" {
    requirement "REQ-1" { title: "Detect obstacles" }
    function "Detect Obstacle" { id: "SF-1" }
    function "Scan" { id: "SF-2" }
    function "Scan" { id: "SF-3" }
}
trace "Detect Obstacle" realizes "Monitor Road" { rationale: "by name" }
trace "SF-1" satisfies "REQ-1"
trace "SF-2" realizes "OA-1"
"#;

#[test]
fn trace_between_drawn_elements_links_their_ids() {
    let set = diagrams_of(LAYERS);

    assert!(
        set.links.contains(&link("realizes", "SF-1", "OA-1")),
        "names are resolved to the ids the diagrams use: {:?}",
        set.links
    );
    assert!(set.links.contains(&link("realizes", "SF-2", "OA-1")));
}

#[test]
fn trace_to_an_element_no_view_draws_is_kept_as_written() {
    let set = diagrams_of(LAYERS);

    // Requirements have no diagram: the link is still the author's, the
    // viewer shows it as text.
    assert!(
        set.links.contains(&link("satisfies", "SF-1", "REQ-1")),
        "got {:?}",
        set.links
    );
}

#[test]
fn links_keep_the_declared_order_and_are_not_repeated() {
    let set = diagrams_of(
        r#"
system_analysis "SA" {
    function "A" { id: "SF-A" }
    function "B" { id: "SF-B" }
}
trace "SF-B" refines "SF-A"
trace "SF-A" refines "SF-B"
trace "B" refines "A"
"#,
    );

    assert_eq!(
        set.links,
        vec![
            link("refines", "SF-B", "SF-A"),
            link("refines", "SF-A", "SF-B")
        ]
    );
}

#[test]
fn model_without_traces_exports_no_links_field() {
    let set = diagrams_of(r#"system_analysis "SA" { function "A" { id: "SF-A" } }"#);

    assert!(set.links.is_empty());
    let json = serde_json::to_value(&set).expect("serializes");
    assert!(json.get("links").is_none(), "an empty list is not exported");
}

#[test]
fn viewer_payload_carries_the_links() {
    let set = diagrams_of(LAYERS);
    let html = arclang::compiler::diagram::html::standalone_html(&set, "Layers");

    assert!(html.contains(r#""links":[{"kind":"realizes","source":"SF-1","target":"OA-1"}"#));
}

#[test]
fn trace_to_an_undrawn_element_is_not_attached_to_a_drawn_namesake() {
    // The requirement is called like a function. The trace names the
    // requirement by its id; a second one by the shared name must not be
    // pinned on the function just because the function is the one drawn.
    let set = diagrams_of(
        r#"
system_analysis "SA" {
    requirement "Braking" { title: "Brake in time" }
    function "Braking" { id: "SF-1" }
    function "Sense" { id: "SF-2" }
}
trace "SF-2" satisfies "Braking"
"#,
    );

    assert_eq!(set.links, vec![link("satisfies", "SF-2", "Braking")]);
}
