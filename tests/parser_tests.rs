use arclang::compiler::lexer::Lexer;
use arclang::compiler::parser::Parser;

#[test]
fn test_parse_minimal_model() {
    let input = r#"
model Test {
    metadata {
        version: "1.0"
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    assert!(ast.is_ok(), "Failed to parse minimal model: {:?}", ast.err());
}

#[test]
fn test_parse_requirements_block() {
    let input = r#"
model Test {
}

requirements stakeholder {
    req "REQ-001" "Test Requirement" {
        description: "A test requirement"
        priority: High
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    assert!(ast.is_ok(), "Failed to parse requirements block: {:?}", ast.err());
    let model = ast.unwrap();
    assert_eq!(model.system_analysis.len(), 1, "Should have 1 system analysis block");
    assert_eq!(model.system_analysis[0].requirements.len(), 1, "Should have 1 requirement");
}

#[test]
fn test_parse_architecture_logical() {
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
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    assert!(ast.is_ok(), "Failed to parse logical architecture: {:?}", ast.err());
    let model = ast.unwrap();
    assert_eq!(model.logical_architecture.len(), 1, "Should have 1 logical architecture");
    assert_eq!(model.logical_architecture[0].components.len(), 1, "Should have 1 component");
}

#[test]
fn test_parse_connections() {
    let input = r#"
model Test {
}

architecture logical {
    component "ComponentA" {
        id: "COMP-001"
    }
    
    component "ComponentB" {
        id: "COMP-002"
    }
    
    connection "ConnAB" {
        from: "COMP-001"
        to: "COMP-002"
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    assert!(ast.is_ok(), "Failed to parse connections: {:?}", ast.err());
    let model = ast.unwrap();
    assert_eq!(model.logical_architecture.len(), 1, "Should have 1 logical architecture");
    assert_eq!(model.logical_architecture[0].components.len(), 2, "Should have 2 components");
    assert_eq!(model.logical_architecture[0].component_exchanges.len(), 1, "Should have 1 connection");
}

#[test]
fn test_parse_multiple_requirement_types() {
    let input = r#"
model Test {
}

requirements stakeholder {
    req "STK-001" "Stakeholder Requirement" {
        description: "User needs"
    }
}

requirements system {
    req "SYS-001" "System Requirement" {
        description: "System shall"
    }
}

requirements safety {
    req "SAF-001" "Safety Requirement" {
        description: "Safety critical"
        safety_level: ASIL_B
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    assert!(ast.is_ok(), "Failed to parse multiple requirement types: {:?}", ast.err());
    let model = ast.unwrap();
    assert_eq!(model.system_analysis.len(), 3, "Should have 3 requirement blocks");
}

#[test]
fn test_parse_component_with_interfaces() {
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
            description: "Sensor input"
        }
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    assert!(ast.is_ok(), "Failed to parse component with interfaces: {:?}", ast.err());
    let model = ast.unwrap();
    assert_eq!(model.logical_architecture[0].components.len(), 1, "Should have 1 component");
}

#[test]
fn test_parse_req_with_string_id() {
    let input = r#"
model Test {
}

requirements system {
    req "SYS-001" "Title" {
        description: "Test"
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    assert!(ast.is_ok(), "Failed to parse req with string ID: {:?}", ast.err());
}

#[test]
fn test_parse_architecture_operational_skip() {
    let input = r#"
model Test {
}

architecture operational {
    scenario "Test" {
        steps: ["A", "B"]
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    // Should succeed by skipping unknown architecture types
    assert!(ast.is_ok(), "Failed to skip operational architecture: {:?}", ast.err());
}

#[test]
fn test_parse_from_to_keywords() {
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
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse();
    
    assert!(ast.is_ok(), "Failed to parse from/to keywords: {:?}", ast.err());
    let model = ast.unwrap();
    let exchange = &model.logical_architecture[0].component_exchanges[0];
    assert_eq!(exchange.from_port, "A1", "from field should be A1");
    assert_eq!(exchange.to_port, "B1", "to field should be B1");
}

// ---- Typed quantities (metamodel M6) ------------------------------------

fn parse_ok(input: &str) -> arclang::compiler::ast::Model {
    let tokens = Lexer::new(input).tokenize().unwrap();
    Parser::new(tokens).parse().unwrap_or_else(|e| panic!("parse failed: {e}"))
}

#[test]
fn number_followed_by_unit_parses_as_typed_quantity() {
    use arclang::compiler::ast::{AttributeValue, Quantity};
    let model = parse_ok(
        r#"
model Test {}
system_analysis "SA" {
    function "Fuse" {
        id: "SF-001"
        latency: 25 ms
        bandwidth: 100Mbps
        period: 0.5 s
    }
}
"#,
    );
    let function = &model.system_analysis[0].functions[0];
    assert_eq!(
        function.attributes.get("latency"),
        Some(&AttributeValue::Quantity(Quantity::new(25.0, "ms").unwrap()))
    );
    assert_eq!(
        function.attributes.get("bandwidth"),
        Some(&AttributeValue::Quantity(Quantity::new(100.0, "Mbps").unwrap()))
    );
    assert_eq!(
        function.attributes.get("period"),
        Some(&AttributeValue::Quantity(Quantity::new(0.5, "s").unwrap()))
    );
}

#[test]
fn bare_number_before_next_attribute_stays_a_number() {
    use arclang::compiler::ast::AttributeValue;
    let model = parse_ok(
        r#"
model Test {}
system_analysis "SA" {
    function "Fuse" {
        id: "SF-001"
        wcet: 40
        name: "Fuse data"
        count: 3
    }
}
"#,
    );
    let function = &model.system_analysis[0].functions[0];
    assert!(matches!(function.attributes.get("wcet"), Some(AttributeValue::Number(n)) if *n == 40.0));
    assert!(matches!(function.attributes.get("count"), Some(AttributeValue::Number(n)) if *n == 3.0));
}

#[test]
fn unknown_unit_is_a_localized_compile_error() {
    let input = "model Test {}\nsystem_analysis \"SA\" {\n    function \"Fuse\" {\n        id: \"SF-001\"\n        latency: 25 furlongs\n    }\n}\n";
    // Go through the compiler entry point: that is the path that carries spans.
    let err = arclang::compiler::Compiler::new(arclang::compiler::CompilerConfig::default())
        .compile_string(input)
        .err()
        .map(|e| e.to_string())
        .expect("unknown unit must not compile");
    assert!(err.contains("unknown unit 'furlongs'"), "message was: {err}");
    assert!(err.contains("line 5"), "error must carry a source position, got: {err}");
}

#[test]
fn quantities_are_allowed_inside_lists_and_legacy_strings_stay_strings() {
    use arclang::compiler::ast::{AttributeValue, Quantity};
    let model = parse_ok(
        r#"
model Test {}
system_analysis "SA" {
    function "Fuse" {
        id: "SF-001"
        latency: "25 ms"
        budgets: [10 ms, 20 ms]
    }
}
"#,
    );
    let function = &model.system_analysis[0].functions[0];
    assert_eq!(function.attributes.get("latency"), Some(&AttributeValue::String("25 ms".to_string())));
    assert_eq!(
        function.attributes.get("budgets"),
        Some(&AttributeValue::List(vec![
            AttributeValue::Quantity(Quantity::new(10.0, "ms").unwrap()),
            AttributeValue::Quantity(Quantity::new(20.0, "ms").unwrap()),
        ]))
    );
}

#[test]
fn compound_unit_symbols_parse_as_one_quantity() {
    use arclang::compiler::ast::{AttributeValue, Quantity};
    let model = parse_ok(
        "model T {}\nsystem_analysis \"SA\" {\n    function \"Cruise\" { id: \"SF-1\" max_speed: 130 km/h rate: 2 Mbit/s }\n}\n",
    );
    let function = &model.system_analysis[0].functions[0];
    assert_eq!(function.attributes.get("max_speed"), Some(&AttributeValue::Quantity(Quantity::new(130.0, "km/h").unwrap())));
    assert_eq!(function.attributes.get("rate"), Some(&AttributeValue::Quantity(Quantity::new(2.0, "Mbit/s").unwrap())));
}

#[test]
fn constraint_requires_exactly_one_assert() {
    let compile = |source: &str| {
        arclang::compiler::Compiler::new(arclang::compiler::CompilerConfig::default())
            .compile_string(source)
            .map(|_| ())
            .map_err(|e| e.to_string())
    };
    let missing = compile("model T {}\nconstraint \"C\" { description: \"nothing asserted\" }").unwrap_err();
    assert!(missing.contains("declares no `assert: <expression>`"), "{missing}");
    let twice = compile("model T {}\nconstraint \"C\" { assert: 1 < 2 assert: 2 < 3 }").unwrap_err();
    assert!(twice.contains("more than one `assert:`"), "{twice}");
    let lone = compile("model T {}\nconstraint \"C\" { assert: 1 = 2 }").unwrap_err();
    assert!(lone.contains("did you mean '=='"), "{lone}");
    assert!(compile("model T {}\nconstraint \"C\" { assert: 1 + 1 == 2 }").is_ok());
}
