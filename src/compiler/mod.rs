pub mod lexer;
pub mod parser;
pub mod ast;
pub mod identity;
pub mod production_gate;
pub mod semantic;
pub mod semantic_analyzer;
pub mod layout_strategy;
pub mod post_processor;
pub mod quality_metrics_v2;
pub mod arcadia_rules_engine;
pub mod professional_styler;
pub mod elk_complete_v2_generator;
pub mod semantic_enhanced;
pub mod semantic_adapter;
pub mod capella_metamodel;
pub mod codegen;
pub mod capella_importer;
pub mod sysmlv2_generator;
pub mod mermaid_generator;
pub mod mermaid_importer;
pub mod plantuml_generator;
pub mod plantuml_importer;
pub mod arcadia_7d_intelligent_generator;
pub mod capella_compliant_generator;

// v2.0.0 Active Generators (RECOMMENDED)
pub mod graph_model;
pub mod arcviz_elk_static;
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
    
    #[error("{0}")]
    Other(String),
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
    /// Non-fatal diagnostics (e.g. constructs accepted syntactically but not
    /// yet represented in the compiled model). Never silently empty a model.
    pub warnings: Vec<String>,
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
        // Lexical analysis (tokens with source positions)
        let (tokens, spans) = lexer::Lexer::new(source).tokenize_spanned()
            .map_err(CompilerError::Lexer)?;

        // Parsing (strict: unknown constructs are errors, skipped ones are warnings)
        let parser::ParseOutcome { model: ast, mut warnings } =
            parser::Parser::with_spans(tokens, spans)
                .parse_with_warnings()
                .map_err(CompilerError::Parser)?;

        // Semantic analysis (dangling traces are errors; unresolved exchange
        // endpoints are warnings until ports become first-class)
        let (semantic_model, semantic_warnings) = semantic::SemanticAnalyzer::new()
            .analyze_with_warnings(&ast)
            .map_err(CompilerError::Semantic)?;
        warnings.extend(semantic_warnings);

        // Code generation
        let output = codegen::CodeGenerator::new(&self.config).generate(&semantic_model)?;

        Ok(CompilationResult {
            ast,
            semantic_model,
            output,
            warnings,
        })
    }
}
