pub mod lexer;
pub mod parser;
pub mod ast;
pub mod semantic;
pub mod codegen;
pub mod capella_importer;
pub mod mermaid_generator;
pub mod mermaid_importer;
pub mod plantuml_generator;
pub mod plantuml_importer;
pub mod arcviz_generator;
pub mod arcviz_smart_routing;
pub mod arcviz_channel_routing;
pub mod arcviz_perfect_routing;
pub mod arcviz_ultimate_routing;
pub mod arcviz_enhanced;
pub mod arcviz_capella_routing;
pub mod arcviz_elk;
pub mod arcviz_d3;
pub mod arcviz_explorer;
pub mod terraform_databricks_generator;
pub mod terraform_aws_complete_generator;
pub mod terraform_azure_generator;
pub mod terraform_gcp_generator;
pub mod kubernetes_helm_generator;
pub mod github_actions_generator;
pub mod gitlab_ci_generator;
pub mod opa_policy_generator;

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Lexer error: {0}")]
    Lexer(String),
    
    #[error("Parser error: {0}")]
    Parser(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Semantic error: {0}")]
    Semantic(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Compiler {
    config: CompilerConfig,
}

#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub optimization_level: u8,
    pub target: String,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization_level: 2,
            target: "capella".to_string(),
        }
    }
}

pub struct CompilationResult {
    pub ast: ast::Model,
    pub semantic_model: semantic::SemanticModel,
    pub output: String,
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }
    
    pub fn compile_file<P: AsRef<Path>>(&mut self, path: P) -> Result<CompilationResult, CompilerError> {
        let source = std::fs::read_to_string(path)?;
        self.compile_string(&source)
    }
    
    pub fn compile_string(&mut self, source: &str) -> Result<CompilationResult, CompilerError> {
        // Lexical analysis
        let tokens = lexer::Lexer::new(source).tokenize()
            .map_err(|e| CompilerError::Lexer(e))?;
        
        // Parsing
        let ast = parser::Parser::new(tokens).parse()
            .map_err(|e| CompilerError::Parser(e))?;
        
        // Semantic analysis
        let semantic_model = semantic::SemanticAnalyzer::new().analyze(&ast)
            .map_err(|e| CompilerError::Semantic(e))?;
        
        // Code generation
        let output = codegen::CodeGenerator::new(&self.config).generate(&semantic_model)?;
        
        Ok(CompilationResult {
            ast,
            semantic_model,
            output,
        })
    }
}
