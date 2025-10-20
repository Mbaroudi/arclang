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
fn test_compile_skips_unknown_tokens() {
    let input = r#"
model Test {
    unknown_stuff here
}
"#;
    let config = CompilerConfig::default();
    let mut compiler = Compiler::new(config);
    let result = compiler.compile_string(input);
    // Parser is lenient and skips unknown tokens
    assert!(result.is_ok(), "Parser should skip unknown tokens");
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
