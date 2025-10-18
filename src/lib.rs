pub mod cli;
pub mod compiler;
pub mod parser;
pub mod semantic;
pub mod plm;
pub mod requirements;
pub mod safety;
pub mod collaboration;
pub mod plugins;

// Re-export for convenience
pub use compiler::{Compiler, CompilerConfig, CompilerError, CompilationResult};
pub use cli::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compile_simple_model() {
        let source = r#"
        operational_analysis "Test System" {
            actor "User" {
                id: "ACT-001"
                description: "System user"
            }
        }
        
        system_analysis "Test" {
            requirement "REQ-001" {
                description: "System shall work"
                priority: "High"
            }
        }
        "#;
        
        let mut compiler = Compiler::new(CompilerConfig {
            optimization_level: 0,
            target: "json".to_string(),
        });
        
        let result = compiler.compile_string(source);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_traceability() {
        let source = r#"
        system_analysis "Test" {
            requirement "REQ-001" {
                description: "Test requirement"
                priority: "High"
            }
        }
        
        logical_architecture "Test Arch" {
            component "Component1" {
                id: "COMP-001"
                type: "Logical"
            }
        }
        
        trace "COMP-001" satisfies "REQ-001" {
            rationale: "Component implements requirement"
        }
        "#;
        
        let mut compiler = Compiler::new(CompilerConfig::default());
        let result = compiler.compile_string(source).unwrap();
        
        assert_eq!(result.semantic_model.requirements.len(), 1);
        assert_eq!(result.semantic_model.components.len(), 1);
        assert_eq!(result.semantic_model.traces.len(), 1);
    }
}
