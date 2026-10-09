pub mod lexer;
pub mod format;
pub mod source_edit;
pub mod parser;
pub mod ast;
pub mod identity;
pub mod quantity;
pub mod multiplicity;
pub mod constraint;
pub mod types;
pub mod elements;
pub mod metamodel;
pub mod metamodel_check;
pub mod production_gate;
pub mod semantic;
pub mod capability_relations;
mod capability_semantics;
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
pub mod sysml_syntax;
pub mod sysml_records;
pub mod sysml_library;
pub mod simulink_generator;
pub mod fmi_generator;
pub mod reqif;
pub mod semantic_diff;
pub mod c_header_generator;
pub mod proto_generator;
pub mod mermaid_generator;
pub mod mermaid_importer;
pub mod plantuml_generator;
pub mod plantuml_importer;
pub mod arcadia_7d_intelligent_generator;
pub mod capella_compliant_generator;

// v2.0.0 Active Generators (RECOMMENDED)
pub mod graph_model;
pub mod diagram;
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
        mut ast: ast::Model,
        mut warnings: Vec<String>,
    ) -> Result<CompilationResult, CompilerError> {
        // User-defined types: fold inherited attributes and ports into every
        // typed element FIRST, so all later stages see effective values.
        let type_uses = types::resolve(&mut ast).map_err(CompilerError::Semantic)?;

        // Semantic analysis (dangling traces are errors; unresolved exchange
        // endpoints are warnings until ports become first-class)
        let (semantic_model, semantic_warnings) = semantic::SemanticAnalyzer::new()
            .analyze_with_warnings(&ast)
            .map_err(CompilerError::Semantic)?;
        warnings.extend(semantic_warnings);

        // Constraints: an ill-formed constraint (unknown element, dimension
        // mismatch) is an ERROR; a violated one is a warning here and a
        // blocker in the production gate.
        let mut semantic_model = semantic_model;
        // Already validated by `types::resolve` above.
        let effective_types = types::effective_types(&ast.types).unwrap_or_default();
        for declared in &ast.types {
            semantic_model.types.push(semantic::TypeInfo {
                name: declared.name.clone(),
                extends: declared.extends.clone(),
                required: effective_types
                    .get(&declared.name)
                    .map(|effective| effective.required.clone())
                    .unwrap_or_default(),
                instances: type_uses
                    .iter()
                    .filter(|usage| usage.type_name == declared.name)
                    .map(|usage| usage.element.clone())
                    .collect(),
            });
        }
        let scope = constraint::Scope::from_model(&ast);
        for declared in &ast.constraints {
            let outcome = constraint::check(&declared.expression, &scope)
                .map_err(|reason| CompilerError::Semantic(format!("constraint '{}': {}", declared.name, reason)))?;
            let expression = declared.expression.to_string();
            if !outcome.satisfied {
                warnings.push(format!(
                    "constraint: constraint '{}' is violated: `{}` — left side is {}, right side is {}",
                    declared.name, expression, outcome.left, outcome.right
                ));
            }
            if let Some(existing) = semantic_model.all_elements.get(&declared.id) {
                warnings.push(format!(
                    "duplicate element id '{}': {} '{}' and Constraint '{}' share the same identity — give one an explicit unique id",
                    declared.id, existing.element_type, existing.name, declared.name
                ));
            } else {
                semantic_model.all_elements.insert(
                    declared.id.clone(),
                    semantic::ElementInfo::new(declared.id.clone(), declared.name.clone(), "Constraint"),
                );
            }
            semantic_model.constraints.push(semantic::ConstraintInfo {
                id: declared.id.clone(),
                name: declared.name.clone(),
                expression,
                satisfied: outcome.satisfied,
                left: outcome.left.to_string(),
                right: outcome.right.to_string(),
            });
        }

        // Typed metamodel: attribute type violations are reported, never
        // fatal (semver MINOR); the production gate turns errors into blockers.
        warnings.extend(
            metamodel_check::check_model(&ast, &semantic_model)
                .iter()
                .map(|d| d.to_string()),
        );

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
