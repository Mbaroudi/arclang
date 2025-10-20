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
    assert_eq!(model.logical_architecture[0].interfaces.len(), 1, "Should have 1 connection");
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
    let interface = &model.logical_architecture[0].interfaces[0];
    assert_eq!(interface.from, "A1", "from field should be A1");
    assert_eq!(interface.to, "B1", "to field should be B1");
}
