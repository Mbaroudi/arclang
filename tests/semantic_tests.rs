use arclang::compiler::lexer::Lexer;
use arclang::compiler::parser::Parser;
use arclang::compiler::semantic::SemanticAnalyzer;

#[test]
fn test_semantic_analysis_basic() {
    let input = r#"
model Test {
}

requirements stakeholder {
    req "REQ-001" "Test" {
        description: "Test requirement"
    }
}

architecture logical {
    component "Component1" {
        id: "COMP-001"
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse().unwrap();
    let result = SemanticAnalyzer::new().analyze(&ast);
    
    assert!(result.is_ok(), "Semantic analysis failed: {:?}", result.err());
    let model = result.unwrap();
    assert_eq!(model.requirements.len(), 1, "Should have 1 requirement");
    assert_eq!(model.components.len(), 1, "Should have 1 component");
}

#[test]
fn test_semantic_interfaces_collected() {
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
    let ast = Parser::new(tokens).parse().unwrap();
    let result = SemanticAnalyzer::new().analyze(&ast);
    
    assert!(result.is_ok(), "Semantic analysis failed: {:?}", result.err());
    let model = result.unwrap();
    assert_eq!(model.interfaces.len(), 1, "Should have 1 interface");
    assert_eq!(model.interfaces[0].from, "A1");
    assert_eq!(model.interfaces[0].to, "B1");
}

#[test]
fn test_semantic_multiple_requirements() {
    let input = r#"
model Test {
}

requirements stakeholder {
    req "STK-001" "Req1" { description: "Test" }
    req "STK-002" "Req2" { description: "Test" }
}

requirements system {
    req "SYS-001" "Req3" { description: "Test" }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse().unwrap();
    let result = SemanticAnalyzer::new().analyze(&ast);
    
    assert!(result.is_ok());
    let model = result.unwrap();
    assert_eq!(model.requirements.len(), 3, "Should have 3 requirements total");
}

#[test]
fn test_semantic_components_with_levels() {
    let input = r#"
model Test {
}

architecture logical {
    component "LogicalComp" {
        id: "LC-001"
        type: "Logical"
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse().unwrap();
    let result = SemanticAnalyzer::new().analyze(&ast);
    
    assert!(result.is_ok());
    let model = result.unwrap();
    assert_eq!(model.components.len(), 1);
    assert_eq!(model.components[0].level, "Logical");
}

#[test]
fn test_semantic_requirement_attributes() {
    let input = r#"
model Test {
}

requirements system {
    req "SYS-001" "Title" {
        description: "System requirement"
        priority: Critical
        safety_level: ASIL_B
    }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse().unwrap();
    let result = SemanticAnalyzer::new().analyze(&ast);
    
    assert!(result.is_ok());
    let model = result.unwrap();
    let req = &model.requirements[0];
    assert_eq!(req.id, "SYS-001");
    assert_eq!(req.description, "System requirement");
    assert_eq!(req.priority, "Critical");
    assert_eq!(req.safety_level, Some("ASIL_B".to_string()));
}

#[test]
fn test_semantic_empty_model() {
    let input = r#"
model Test {
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse().unwrap();
    let result = SemanticAnalyzer::new().analyze(&ast);
    
    assert!(result.is_ok());
    let model = result.unwrap();
    assert_eq!(model.requirements.len(), 0);
    assert_eq!(model.components.len(), 0);
    assert_eq!(model.interfaces.len(), 0);
}

#[test]
fn test_semantic_metrics() {
    let input = r#"
model Test {
}

requirements stakeholder {
    req "STK-001" "Req" { description: "Test" }
    req "STK-002" "Req" { description: "Test" }
}

architecture logical {
    component "Comp1" { id: "C1" }
    component "Comp2" { id: "C2" }
    component "Comp3" { id: "C3" }
}
"#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    let ast = Parser::new(tokens).parse().unwrap();
    let model = SemanticAnalyzer::new().analyze(&ast).unwrap();
    let metrics = model.compute_metrics();
    
    assert_eq!(metrics.requirements_count, 2);
    assert_eq!(metrics.components_count, 3);
    assert_eq!(metrics.total_elements, 5);
}
