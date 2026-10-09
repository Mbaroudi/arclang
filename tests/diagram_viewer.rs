//! The diagram viewer as shipped: in the standalone page and spliced into
//! the architecture explorer. Layout and drawing themselves run in
//! JavaScript and are checked by `tools/diagram_render/check.sh` (CI job
//! `examples`); these tests pin what the Rust side guarantees.

use arclang::compiler::arcviz_explorer::generate_explorer_html;
use arclang::compiler::diagram::{build_diagrams, html};
use arclang::compiler::{CompilationResult, Compiler, CompilerConfig};
use std::path::Path;

fn flagship() -> CompilationResult {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/complete_emergency_braking_simple.arc");
    Compiler::new(CompilerConfig::default())
        .compile_file(source)
        .expect("flagship compiles")
}

/// The JSON the viewer is mounted with.
fn embedded_payload(html: &str) -> serde_json::Value {
    let marker = "id=\"arcviz-data\">";
    let start = html.find(marker).expect("viewer data element") + marker.len();
    let end = start
        + html[start..]
            .find("</script>")
            .expect("data element closes");
    serde_json::from_str(&html[start..end]).expect("viewer data is valid JSON")
}

#[test]
fn explorer_embeds_the_viewer_with_every_diagram_of_the_model() {
    let result = flagship();
    let (page, _) =
        generate_explorer_html(&result.semantic_model, &result.ast).expect("explorer renders");

    assert!(
        page.contains("<div id=\"arcviz-viewer\"></div>"),
        "viewer mount point"
    );
    assert!(
        !page.contains("<!--ARCVIZ_"),
        "every placeholder was replaced"
    );
    assert!(
        !page.contains("ARCH_DATA_PLACEHOLDER"),
        "document data was embedded"
    );

    let payload = embedded_payload(&page);
    let views: Vec<&str> = payload["graphs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        views,
        [
            "oab",
            "sab",
            "lab",
            "pab",
            "cap",
            "cdb",
            "msm:AEBOperatingModes",
            "es:EmergencyStop"
        ]
    );
}

#[cfg(feature = "native")]
#[test]
fn explorer_fetches_nothing_from_the_network() {
    let result = flagship();
    let (page, _) =
        generate_explorer_html(&result.semantic_model, &result.ast).expect("explorer renders");

    for needle in [
        "<script src=",
        "<link rel=\"stylesheet\" href=\"http",
        "d3js.org",
        "dagre",
    ] {
        assert!(
            !page.contains(needle),
            "explorer must be self-contained, found `{needle}`"
        );
    }
}

#[test]
fn standalone_viewer_carries_the_same_graphs_as_the_elk_export() {
    let result = flagship();
    let set = build_diagrams(&result.ast);
    let page = html::standalone_html(&set, "AEB");

    let payload = embedded_payload(&page);
    let exported: Vec<serde_json::Value> = set
        .diagrams
        .iter()
        .map(arclang::compiler::diagram::elk::to_elk)
        .collect();
    assert_eq!(payload["graphs"], serde_json::Value::Array(exported));
    assert_eq!(
        payload["diagnostics"].as_array().unwrap().len(),
        set.diagnostics.len()
    );
}

#[test]
fn groups_without_exchanges_are_packed_and_wired_groups_are_layered() {
    let source = r#"
system_analysis SA {
  function Parent {
    function A {
      description: "a"
    }
    function B {
      description: "b"
    }
  }
}
logical_architecture LA {
  component Left {
    interface_out Out {
      name: "I"
    }
  }
  component Right {
    interface_in In {
      name: "I"
    }
  }
  component_exchange "Link" {
    from_port: "Left.Out"
    to_port: "Right.In"
  }
}
"#;
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(source)
        .expect("compiles");
    let set = build_diagrams(&result.ast);
    let graph_of = |id: &str| {
        let diagram = set
            .diagrams
            .iter()
            .find(|d| d.id == id)
            .expect("view exists");
        arclang::compiler::diagram::elk::to_elk(diagram)
    };

    // No exchange anywhere in the system view: boxes are packed in rows.
    let sab = graph_of("sab");
    assert_eq!(sab["layoutOptions"]["elk.algorithm"], "rectpacking");
    assert_eq!(
        sab["children"][0]["layoutOptions"]["elk.algorithm"],
        "rectpacking"
    );

    // An exchange to route: layered, and no node on its path is packed.
    let lab = graph_of("lab");
    assert_eq!(lab["layoutOptions"]["elk.algorithm"], "layered");
    assert_eq!(lab["edges"][0]["sources"][0], "Left::Out");
    assert_eq!(lab["edges"][0]["targets"][0], "Right::In");
}

#[test]
fn self_loop_and_container_exchanges_are_both_laid_out() {
    let source = r#"
logical_architecture LA {
  component Loop {
    interface_out O {
      name: "I"
    }
    interface_in I {
      name: "I"
    }
  }
  component_exchange "Feedback" {
    from_port: "Loop.O"
    to_port: "Loop.I"
  }
}
operational_analysis OA {
  entity Vehicle {
    activity Brake {
      description: "brake"
    }
  }
  interaction Force {
    from: Vehicle.Brake
    to: Vehicle
  }
}
"#;
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(source)
        .expect("compiles");
    let set = build_diagrams(&result.ast);
    let graph_of = |id: &str| {
        let diagram = set
            .diagrams
            .iter()
            .find(|d| d.id == id)
            .expect("view exists");
        arclang::compiler::diagram::elk::to_elk(diagram)
    };

    let lab = graph_of("lab");
    assert_eq!(
        lab["edges"].as_array().unwrap().len(),
        1,
        "a self-loop is routable"
    );

    // An exchange between an element and its own container is laid out
    // like any other: nothing the author wrote is left off the drawing.
    let oab = graph_of("oab");
    assert_eq!(oab["edges"].as_array().unwrap().len(), 1);
    assert_eq!(oab["edges"][0]["labels"][0]["text"], "Force");
    assert!(oab["arc"].get("omitted_edges").is_none());
}

#[test]
fn explorer_carries_the_traces_and_the_navigation_that_follows_them() {
    let result = flagship();
    let (page, _) =
        generate_explorer_html(&result.semantic_model, &result.ast).expect("explorer renders");
    let payload = embedded_payload(&page);

    let links = payload["links"].as_array().expect("links are embedded");
    assert!(
        links.iter().any(|link| link["kind"] == "realizes"
            && link["source"] == "SF-AssessThreat"
            && link["target"] == "OA-DetectThreat"),
        "the function-to-activity realization is navigable: {links:?}"
    );
    assert!(page.contains("ArcVizLinks"), "navigation index is inlined");
    assert!(
        page.contains("navigation: ArcVizNavigation"),
        "navigation chrome is mounted"
    );
    assert!(
        page.contains("collapse: ArcVizCollapse"),
        "container folding is mounted"
    );
    assert!(
        page.contains("scope: ArcVizScope"),
        "fold and focus state is mounted"
    );
    assert!(
        page.contains("place: ArcVizPlace") && page.contains("stage: ArcVizStage"),
        "manual placement is mounted"
    );
}
