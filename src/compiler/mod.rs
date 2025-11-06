pub mod lexer;
pub mod parser;
pub mod ast;
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
pub mod constraint_engine;
pub mod constraint_solver;
pub mod quality_metrics;
pub mod multi_objective_optimizer;
pub mod routing_intelligence;
pub mod hierarchy_intelligence;
pub mod safety_intelligence;
pub mod aesthetic_intelligence;
pub mod codegen;
pub mod capella_importer;
pub mod mermaid_generator;
pub mod mermaid_importer;
pub mod plantuml_generator;
pub mod plantuml_importer;
pub mod arcadia_7d_intelligent_generator;
pub mod capella_compliant_generator;

// v2.0.0 Active Generators (RECOMMENDED)
pub mod arcviz_elk_static;
pub mod arcviz_explorer;

// v1 Legacy Generators - OBSOLETE (archived in src/compiler/archive/)
// DO NOT USE - These are kept for backward compatibility only
// Will be removed in v3.0.0
#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator instead. This module is obsolete and will be removed in v3.0.0")]
#[path = "archive/v1_obsolete/arcviz_generator.rs"]
pub mod arcviz_generator;

#[deprecated(since = "2.0.0", note = "Use arcviz_elk_static instead. Quality: 2.5/10 (v1) vs 9.0/10 (v2)")]
#[path = "archive/v1_obsolete/arcviz_smart_routing.rs"]
pub mod arcviz_smart_routing;

#[deprecated(since = "2.0.0", note = "Use arcviz_elk_static instead. Quality: 2.5/10 (v1) vs 9.0/10 (v2)")]
#[path = "archive/v1_obsolete/arcviz_channel_routing.rs"]
pub mod arcviz_channel_routing;

#[deprecated(since = "2.0.0", note = "Use arcviz_elk_static instead. Quality: 2.5/10 (v1) vs 9.0/10 (v2)")]
#[path = "archive/v1_obsolete/arcviz_perfect_routing.rs"]
pub mod arcviz_perfect_routing;

#[deprecated(since = "2.0.0", note = "Use arcviz_elk_static instead. Quality: 2.5/10 (v1) vs 9.0/10 (v2)")]
#[path = "archive/v1_obsolete/arcviz_ultimate_routing.rs"]
pub mod arcviz_ultimate_routing;

#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator instead. Experimental code, never production-ready")]
#[path = "archive/v1_obsolete/arcviz_enhanced.rs"]
pub mod arcviz_enhanced;

#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator instead. This was an early experiment")]
#[path = "archive/v1_obsolete/arcviz_capella_routing.rs"]
pub mod arcviz_capella_routing;

#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator instead. Early ELK attempt, abandoned")]
#[path = "archive/v1_obsolete/arcviz_elk.rs"]
pub mod arcviz_elk;

#[deprecated(since = "2.0.0", note = "D3.js experiment never completed. Use elk_complete_v2_generator")]
#[path = "archive/v1_obsolete/arcviz_d3.rs"]
pub mod arcviz_d3;

#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator for integrated pipeline")]
#[path = "archive/v1_deprecated/elk_json_generator.rs"]
pub mod elk_json_generator;

#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator for integrated pipeline")]
#[path = "archive/v1_deprecated/elk_html_template.rs"]
pub mod elk_html_template;

#[deprecated(since = "2.0.0", note = "Fallback only - Use elk_complete_v2_generator with ELK as primary")]
#[path = "archive/v1_deprecated/dagre_json_generator.rs"]
pub mod dagre_json_generator;

#[deprecated(since = "2.0.0", note = "Fallback only - Use elk_complete_v2_generator with ELK as primary")]
#[path = "archive/v1_deprecated/dagre_html_template.rs"]
pub mod dagre_html_template;

#[deprecated(since = "2.0.0", note = "Strategy selection now in layout_strategy.rs module")]
#[path = "archive/v1_deprecated/elk_dagre_hybrid.rs"]
pub mod elk_dagre_hybrid;

#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator templates instead")]
#[path = "archive/v1_deprecated/elk_dagre_hybrid_template.rs"]
pub mod elk_dagre_hybrid_template;

#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator instead. v1 architecture superseded")]
#[path = "archive/v1_deprecated/elk_complete_generator.rs"]
pub mod elk_complete_generator;

#[deprecated(since = "2.0.0", note = "Use elk_complete_v2_generator templates instead")]
#[path = "archive/v1_deprecated/elk_complete_template.rs"]
pub mod elk_complete_template;
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
