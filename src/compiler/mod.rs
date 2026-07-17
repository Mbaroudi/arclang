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
pub mod simulink_generator;
pub mod fmi_generator;
pub mod reqif;
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

#[derive(Debug)]
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
        let path = path.as_ref();
        let mut import_stack = Vec::new();
        let (ast, warnings) = Self::parse_file_with_imports(path, &mut import_stack)?;
        self.finish(ast, warnings)
    }

    pub fn compile_string(&mut self, source: &str) -> Result<CompilationResult, CompilerError> {
        let (ast, warnings) = Self::parse_source(source)?;
        if !ast.imports.is_empty() {
            return Err(CompilerError::Parser(format!(
                "this model imports {} file(s) — compile it from its file so \
                 relative import paths can be resolved",
                ast.imports.len()
            )));
        }
        self.finish(ast, warnings)
    }

    /// Lex + parse one source text. No filesystem access.
    fn parse_source(source: &str) -> Result<(ast::Model, Vec<String>), CompilerError> {
        let (tokens, spans) = lexer::Lexer::new(source).tokenize_spanned()
            .map_err(CompilerError::Lexer)?;
        let parser::ParseOutcome { model, warnings } =
            parser::Parser::with_spans(tokens, spans)
                .parse_with_warnings()
                .map_err(CompilerError::Parser)?;
        Ok((model, warnings))
    }

    /// Parse a file and recursively merge its `import "..."` declarations,
    /// resolved relative to the importing file. `import_stack` holds the
    /// canonical paths currently being parsed: re-entering one is a cycle
    /// and fails with the full chain.
    fn parse_file_with_imports(
        path: &Path,
        import_stack: &mut Vec<std::path::PathBuf>,
    ) -> Result<(ast::Model, Vec<String>), CompilerError> {
        let canonical = path.canonicalize().map_err(|e| {
            CompilerError::Io(std::io::Error::new(
                e.kind(),
                format!("cannot resolve {}: {e}", path.display()),
            ))
        })?;
        if import_stack.contains(&canonical) {
            let chain: Vec<String> = import_stack
                .iter()
                .chain(std::iter::once(&canonical))
                .map(|p| p.display().to_string())
                .collect();
            return Err(CompilerError::Parser(format!(
                "circular import: {}",
                chain.join(" -> ")
            )));
        }
        import_stack.push(canonical.clone());

        let source = std::fs::read_to_string(&canonical)?;
        let (mut root, mut warnings) = Self::parse_source(&source).map_err(|e| match e {
            // Localize parse errors to the file they came from.
            CompilerError::Parser(msg) => {
                CompilerError::Parser(format!("{}: {msg}", path.display()))
            }
            CompilerError::Lexer(msg) => {
                CompilerError::Lexer(format!("{}: {msg}", path.display()))
            }
            other => other,
        })?;

        let base_dir = canonical.parent().map(Path::to_path_buf).unwrap_or_default();
        for import in std::mem::take(&mut root.imports) {
            let target = base_dir.join(&import);
            if !target.exists() {
                import_stack.pop();
                return Err(CompilerError::Parser(format!(
                    "{}: imported file not found: {} (resolved to {})",
                    path.display(),
                    import,
                    target.display()
                )));
            }
            let (fragment, fragment_warnings) =
                Self::parse_file_with_imports(&target, import_stack)?;
            root.merge(fragment);
            warnings.extend(fragment_warnings);
        }

        import_stack.pop();
        Ok((root, warnings))
    }

    /// Semantic analysis + code generation on a fully-merged AST.
    fn finish(
        &mut self,
        ast: ast::Model,
        mut warnings: Vec<String>,
    ) -> Result<CompilationResult, CompilerError> {
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
