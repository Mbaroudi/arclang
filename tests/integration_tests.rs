use arclang::compiler::{Compiler, CompilerConfig};

#[test]
fn test_compile_minimal_model() {
    let input = r#"
model Test {
    metadata {
        version: "1.0"
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_ok(), "Failed to compile minimal model: {:?}", result.err());
}

#[test]
fn test_compile_with_requirements() {
    let input = r#"
model Test {
}

requirements stakeholder {
    req "REQ-001" "Test Requirement" {
        description: "Test"
        priority: High
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_ok(), "Failed to compile with requirements: {:?}", result.err());
    
    let output = result.unwrap();
    assert_eq!(output.semantic_model.requirements.len(), 1);
}

#[test]
fn test_compile_with_architecture() {
    let input = r#"
model Test {
}

architecture logical {
    component "TestComponent" {
        id: "COMP-001"
        description: "Test component"
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_ok(), "Failed to compile with architecture: {:?}", result.err());
    
    let output = result.unwrap();
    assert_eq!(output.semantic_model.components.len(), 1);
}

#[test]
fn test_compile_with_connections() {
    let input = r#"
model Test {
}

architecture logical {
    component "A" { id: "A1" }
    component "B" { id: "B1" }
    
    connection "AB" {
        from: "A1"
        to: "B1"
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_ok(), "Failed to compile with connections: {:?}", result.err());
    
    let output = result.unwrap();
    assert_eq!(output.semantic_model.components.len(), 2);
    assert_eq!(output.semantic_model.interfaces.len(), 1);
}

#[test]
fn test_compile_full_system() {
    let input = r#"
model AdaptiveCruiseControl {
    metadata {
        version: "1.0"
        description: "ACC System"
    }
}

requirements stakeholder {
    req "STK-001" "User Control" {
        description: "Driver shall control speed"
        priority: High
    }
}

requirements system {
    req "SYS-001" "Speed Range" {
        description: "System operates 30-180 km/h"
        priority: High
    }
}

architecture logical {
    component "ACCController" {
        id: "LC-001"
        description: "Main controller"
    }
    
    component "RadarSensor" {
        id: "LC-002"
        description: "Forward radar"
    }
    
    connection "SensorToController" {
        from: "LC-002"
        to: "LC-001"
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_ok(), "Failed to compile full system: {:?}", result.err());
    
    let output = result.unwrap();
    assert_eq!(output.semantic_model.requirements.len(), 2);
    assert_eq!(output.semantic_model.components.len(), 2);
    assert_eq!(output.semantic_model.interfaces.len(), 1);
}

#[test]
fn test_compile_rejects_unknown_tokens_with_location() {
    let input = r#"
model Test {
    unknown_stuff here
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    // The parser is strict: unknown constructs must fail loudly with a source
    // location, never be silently dropped from the model.
    assert!(result.is_err(), "Parser must reject unknown tokens instead of skipping them");
    let message = result.err().unwrap().to_string();
    assert!(
        message.contains("line 3"),
        "Error must carry the source location, got: {message}"
    );
}

#[test]
fn test_compile_multiple_architectures() {
    let input = r#"
model Test {
}

architecture logical {
    component "LogicalComp" { id: "LC-001" }
}

architecture physical {
    component "PhysicalComp" { id: "PC-001" }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_ok(), "Failed to compile multiple architectures: {:?}", result.err());
}

#[test]
fn test_compile_requirement_id_with_quotes() {
    let input = r#"
model Test {
}

requirements system {
    req "SYS-001" "Title" {
        description: "Test"
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_ok(), "Failed with quoted requirement ID: {:?}", result.err());
    
    let output = result.unwrap();
    assert_eq!(output.semantic_model.requirements[0].id, "SYS-001");
}

#[test]
fn test_compile_component_with_nested_interfaces() {
    let input = r#"
model Test {
}

architecture logical {
    component "Controller" {
        id: "CTRL-001"
        
        provides interface IControl {
            description: "Control interface"
        }
        
        requires interface ISensor {
            description: "Sensor data"
        }
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_ok(), "Failed with nested interfaces: {:?}", result.err());
    
    let output = result.unwrap();
    assert_eq!(output.semantic_model.components.len(), 1);
}

#[test]
fn test_dangling_trace_is_a_compile_error() {
    let input = r#"
model Test {
}

architecture logical {
    component "Controller" { id: "LC-001" }
}

trace "LC-001" satisfies "REQ-DOES-NOT-EXIST" { rationale: "test" }
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_err(), "A trace to a nonexistent element must fail compilation");
    let message = result.err().unwrap().to_string();
    assert!(
        message.contains("REQ-DOES-NOT-EXIST"),
        "Error must name the unresolved reference, got: {message}"
    );
}

#[test]
fn test_trace_by_name_is_normalized_to_id() {
    let input = r#"
model Test {
}

requirements safety {
    req "REQ-001" "Braking" { description: "Brake on demand" }
}

architecture logical {
    component "Brake Controller" { id: "LC-001" }
}

trace "Brake Controller" satisfies "REQ-001" { rationale: "by-name reference" }
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("name-based trace must resolve");
    assert_eq!(result.semantic_model.traces.len(), 1);
    // The endpoint written by name must be normalized to the element id.
    assert_eq!(result.semantic_model.traces[0].from, "LC-001");
}

#[test]
fn test_elements_have_stable_deterministic_uuids() {
    let input = r#"
model Test {
}

architecture logical {
    component "Controller" { id: "LC-001" }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("must compile");

    let element = result
        .semantic_model
        .all_elements
        .get("LC-001")
        .expect("LC-001 must be registered");
    // Deterministic v5 UUID: same id -> same uuid on every machine, forever.
    // Cross-checked against Python: uuid5(ARCLANG_NAMESPACE, "element:LC-001").
    assert_eq!(element.uuid, "8006ab91-390c-5908-8464-b353219dfc1f");
}

#[test]
fn test_unresolved_exchange_endpoint_produces_warning() {
    let input = r#"
model Test {
}

system_analysis SA {
    function Compute { outputs: ["x"] }
    functional_exchange Flow1 {
        from: Compute
        to: DoesNotExist
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("must compile (warning, not error)");
    assert!(
        result.warnings.iter().any(|w| w.contains("DoesNotExist")),
        "Expected a warning naming the unresolved endpoint, got: {:?}",
        result.warnings
    );
}

#[test]
fn test_duplicate_element_id_produces_warning() {
    let input = r#"
model Test {
}

architecture logical {
    component "Controller" { id: "LC-001" }
    component "OtherThing" { id: "LC-001" }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("duplicate ids warn, not fail");
    assert!(
        result.warnings.iter().any(|w| w.contains("duplicate element id 'LC-001'")),
        "Expected duplicate-id warning, got: {:?}",
        result.warnings
    );
}

#[test]
fn test_deployment_to_unknown_component_produces_warning() {
    let input = r#"
model Test {
}

physical_architecture "PA" {
    node "ECU" {
        id: "PN-001"
        deploys "GhostComponent"
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("unknown deployment warns");
    assert!(
        result.warnings.iter().any(|w| w.contains("GhostComponent")),
        "Expected deployment warning, got: {:?}",
        result.warnings
    );
}

#[test]
fn test_traceability_coverage_counts_satisfies_direction() {
    let input = r#"
model Test {
}

requirements safety {
    req "REQ-001" "Covered" { description: "traced" }
    req "REQ-002" "Uncovered" { description: "not traced" }
}

architecture logical {
    component "Controller" { id: "LC-001" }
}

trace "LC-001" satisfies "REQ-001" { rationale: "direct" }
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("compiles");
    let metrics = result.semantic_model.compute_metrics();
    // REQ-001 is the TARGET of the satisfies trace and must count as covered.
    assert!((metrics.traceability_coverage - 50.0).abs() < f64::EPSILON,
        "expected 50% coverage, got {}", metrics.traceability_coverage);
}

#[test]
fn test_impact_analysis_traverses_traces_and_exchanges() {
    let input = r#"
model Test {
}

requirements safety {
    req "REQ-001" "Braking" { description: "Brake on demand" }
}

system_analysis SA {
    functional_exchange Flow1 {
        from: "LC-001"
        to: "LC-002"
        exchange_item: "command"
    }
}

architecture logical {
    component "Controller" { id: "LC-001" }
    component "Actuator" { id: "LC-002" }
    component "Unrelated" { id: "LC-999" }
}

trace "LC-001" satisfies "REQ-001" { rationale: "direct" }
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("compiles");
    let entries = result.semantic_model.impact_of("REQ-001").expect("element resolves");

    let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
    assert!(ids.contains(&"LC-001"), "trace target must be impacted: {ids:?}");
    assert!(ids.contains(&"LC-002"), "exchange neighbor must be transitively impacted: {ids:?}");
    assert!(!ids.contains(&"LC-999"), "unconnected element must NOT be impacted: {ids:?}");

    let controller = entries.iter().find(|e| e.id == "LC-001").unwrap();
    let actuator = entries.iter().find(|e| e.id == "LC-002").unwrap();
    assert_eq!(controller.depth, 1);
    assert_eq!(actuator.depth, 2);
}

#[test]
fn test_missions_capabilities_and_chains_are_first_class() {
    let input = r#"
model Test {
}

operational_analysis "OA" {
    operational_capability "Avoid Collisions" { id: "OC-001" }
}

system_analysis SA {
    mission SafeBraking { id: "MIS-001" }

    capability EmergencyBraking {
        id: "CAP-001"
        mission: "MIS-001"
        realizes: "OC-001"
        involves: ["Detect"]
    }

    functional_chain BrakeChain {
        id: "FC-001"
        involves: ["Detect", "Brake"]
    }

    function Detect { outputs: ["threat"] }
    function Brake { inputs: ["threat"] }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("must compile");
    let model = &result.semantic_model;

    assert_eq!(model.missions.len(), 1);
    assert_eq!(model.capabilities.len(), 1);
    assert_eq!(model.functional_chains.len(), 1);

    let cap = &model.capabilities[0];
    assert_eq!(cap.realizes.as_deref(), Some("OC-001"));
    assert_eq!(cap.mission.as_deref(), Some("MIS-001"));
    // involves resolved from name to id
    assert_eq!(cap.involves, vec!["SF-Det"]);

    // All three registered with stable identity
    assert!(model.all_elements.contains_key("MIS-001"));
    assert!(model.all_elements.contains_key("CAP-001"));
    assert!(model.all_elements.contains_key("FC-001"));
}

#[test]
fn test_dangling_capability_reference_is_an_error() {
    let input = r#"
model Test {
}

system_analysis SA {
    capability Broken {
        id: "CAP-001"
        realizes: "OC-DOES-NOT-EXIST"
    }
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    assert!(result.is_err(), "dangling realizes must fail compilation");
    assert!(result.err().unwrap().to_string().contains("OC-DOES-NOT-EXIST"));
}

#[test]
fn test_realizes_trace_type_parses_and_resolves() {
    let input = r#"
model Test {
}

operational_analysis "OA" {
    operational_activity "Watch" { id: "OA-001" }
}

system_analysis SA {
    function Monitor { outputs: ["data"] }
}

trace "Monitor" realizes "OA-001" { rationale: "vertical realization" }
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input).expect("must compile");
    let trace = &result.semantic_model.traces[0];
    assert_eq!(trace.trace_type, "realizes");
    assert_eq!(trace.to, "OA-001");
}
