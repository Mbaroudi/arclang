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
pub mod spatial;
pub mod allocations;
pub mod identity_sites;
pub mod verification;
pub mod execution;
pub mod fta;
pub mod notes;
pub mod occurrences;
pub mod variability;
pub mod rules;
pub mod document;
pub mod suspect;
pub mod user_rules;
pub mod transition;
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
pub mod refine;
pub mod capella_xmi;
pub mod capella_export;
pub mod sysmlv2_generator;
pub mod sysml_syntax;
pub mod sysml_import;
pub mod sysml_parser;
pub mod sysml_export;
pub mod sysml_records;
pub mod sysml_library;
pub mod simulink_connector;
pub mod simulink_generator;
mod simulink_stateflow;
pub mod fmi_generator;
pub mod modelica;
pub mod embedded;
pub mod simulation;
pub mod partition;
pub mod realization;
pub mod schedule;
pub mod reqif;
pub mod graph_diff;
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
pub mod tables; // tables hook
pub mod impact; // impact hook
pub mod semgit; // semgit hook
pub mod cicd; // cicd hook
pub mod arcviz_explorer;
pub mod workbench;
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
    /// The HW/SW measurements `arclang partition` kept beside the model, each with whether it still holds.
    pub partition: Vec<(partition::Entry, bool)>,
    /// The tasks run together on the target (`arclang schedule --prove`), each with whether it still holds.
    pub schedule: Vec<(embedded::tasks::TaskRun, bool)>,
}

impl CompilationResult {
    /// A model compiled from a copy (an edit beside it, a revision in a
    /// scratch directory) speaks of the copy: name `shown` instead, so that
    /// no scratch path reaches a warning, a verdict or the gate.
    pub fn name_file_as(&mut self, copy: &Path, shown: &str) {
        let copy = copy.display().to_string();
        if copy.is_empty() || copy == shown {
            return;
        }
        for warning in &mut self.warnings {
            *warning = warning.replace(&copy, shown);
        }
        for status in self.semantic_model.simulations.values_mut() {
            status.reason = status.reason.replace(&copy, shown);
            status.evidence = status.evidence.replace(&copy, shown);
        }
    }
}

/// The file a compiled input stands for, when it is a copy of it (`--as`):
/// imports and cached verdicts resolve next to that file, not the copy.
static ORIGIN: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Declare the file every later compiled input stands for (set once).
pub fn set_origin(path: std::path::PathBuf) {
    let _ = ORIGIN.set(path);
}

/// The file the inputs stand for, when one was declared.
pub fn origin() -> Option<&'static Path> {
    ORIGIN.get().map(|p| p.as_path())
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }
    
    pub fn compile_file<P: AsRef<Path>>(&mut self, path: P) -> Result<CompilationResult, CompilerError> {
        let path = path.as_ref();
        // A copy of another file (an editor's buffer): read here, resolved there.
        if let Some(origin) = origin() {
            let same = path.canonicalize().ok().zip(origin.canonicalize().ok()).is_some_and(|(a, b)| a == b);
            if !same {
                let source = std::fs::read_to_string(path)?;
                return self.compile_string_at(&source, origin);
            }
        }
        let mut import_stack = Vec::new();
        let mut files = Vec::new();
        let (ast, warnings) = Self::parse_file_with_imports(path, &mut import_stack, &mut files)?;
        let mut result = self.finish_located(ast, warnings, &files)?;
        // The cached verdicts of the simulated cases: nothing is simulated.
        simulation::run::attach(&mut result, path);
        result.partition = partition::fresh(&result.ast, path);
        result.schedule = embedded::tasks::fresh(&result.ast, path);
        Ok(result)
    }

    /// The files a model is read from: the file itself, then every file it
    /// imports (transitively), each once, as canonical paths.
    pub fn files_of<P: AsRef<Path>>(path: P) -> Result<Vec<std::path::PathBuf>, CompilerError> {
        let mut files = Vec::new();
        Self::parse_file_with_imports(path.as_ref(), &mut Vec::new(), &mut files)?;
        files.iter().map(|file| Path::new(&file.label).canonicalize().map_err(CompilerError::Io)).collect()
    }

    /// Finish, with the places of duplicate identities named.
    fn finish_located(
        &mut self,
        ast: ast::Model,
        warnings: Vec<String>,
        files: &[identity_sites::SourceFile],
    ) -> Result<CompilationResult, CompilerError> {
        identity_sites::check_requirements(&ast, files).map_err(CompilerError::Semantic)?;
        let mut result = self.finish(ast, warnings)?;
        result.warnings = std::mem::take(&mut result.warnings).into_iter().map(|w| identity_sites::locate_duplicate(w, files)).collect();
        Ok(result)
    }

    pub fn compile_string(&mut self, source: &str) -> Result<CompilationResult, CompilerError> {
        // A text that stands for a file (`--as`): its imports resolve next to that file.
        if let Some(origin) = origin() {
            return self.compile_string_at(source, origin);
        }
        let (ast, warnings) = Self::parse_source(source)?;
        if !ast.imports.is_empty() {
            return Err(CompilerError::Parser(format!(
                "this model imports {} file(s) — compile it from its file so \
                 relative import paths can be resolved",
                ast.imports.len()
            )));
        }
        let files = [identity_sites::SourceFile { label: String::new(), text: source.to_string() }];
        self.finish_located(ast, warnings, &files)
    }

    /// Compile a source text that belongs at `path` (an editor's buffer, saved
    /// or not): its imports resolve relative to that file, as `compile_file`
    /// would resolve them.
    pub fn compile_string_at(&mut self, source: &str, path: &Path) -> Result<CompilationResult, CompilerError> {
        let (mut root, mut warnings) = Self::parse_source(source)?;
        let mut files = vec![identity_sites::SourceFile { label: path.display().to_string(), text: source.to_string() }];
        if !root.imports.is_empty() {
            let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            let base_dir = canonical.parent().map(Path::to_path_buf).unwrap_or_default();
            let mut import_stack = vec![canonical];
            for import in std::mem::take(&mut root.imports) {
                let target = base_dir.join(&import);
                if !target.exists() {
                    return Err(CompilerError::Parser(format!("imported file not found: {import} (resolved to {})", target.display())));
                }
                let (fragment, fragment_warnings) = Self::parse_file_with_imports(&target, &mut import_stack, &mut files)?;
                root.merge(fragment);
                warnings.extend(fragment_warnings);
            }
        }
        let mut result = self.finish_located(root, warnings, &files)?;
        simulation::run::attach(&mut result, path);
        Ok(result)
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
        files: &mut Vec<identity_sites::SourceFile>,
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
        // A file imported along two paths (a diamond) is read once.
        if files.iter().any(|file| Path::new(&file.label).canonicalize().ok().as_ref() == Some(&canonical)) {
            return Ok((ast::Model::new(), Vec::new()));
        }
        import_stack.push(canonical.clone());

        let source = std::fs::read_to_string(&canonical)?;
        files.push(identity_sites::SourceFile { label: path.display().to_string(), text: source.clone() });
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
                Self::parse_file_with_imports(&target, import_stack, files)?;
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
        // What the model adds to elements declared elsewhere (an imported
        // file): applied first, so every later stage sees the whole element.
        refine::apply(&mut ast).map_err(CompilerError::Semantic)?;

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
        // A twin names requirements and a configuration of the model.
        for finding in simulation::twin::check(&ast) {
            warnings.push(format!("twin: {finding}"));
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

        // Typed allocations: their ends are of the kinds their definition says.
        allocations::check(&ast, &semantic_model).map_err(CompilerError::Semantic)?;

        // Verification cases: the verdict of each, from the constraints it asserts.
        warnings.extend(verification::check(&ast, &semantic_model).map_err(CompilerError::Semantic)?);

        // Product lines: variations, variants and the configurations selecting them.
        warnings.extend(variability::check(&ast).map_err(CompilerError::Semantic)?);

        // Notes on the diagrams: about elements the model declares.
        warnings.extend(notes::check(&ast, &mut semantic_model).map_err(CompilerError::Semantic)?);

        // Rules the model declares, and traces whose end changed since they were reviewed.
        if let Some(found) = user_rules::violations(&ast, &semantic_model).map_err(CompilerError::Semantic)? {
            warnings.extend(found.iter().map(|v| v.line()));
        }
        if suspect::watches(&ast) {
            let graph = elements::build(&ast, &semantic_model);
            warnings.extend(suspect::suspects(&ast, &graph).map_err(CompilerError::Semantic)?.iter().map(|s| s.line()));
        }

        // Fault trees: failure modes leading to hazards, held to their budgets.
        warnings.extend(fta::check(&ast, &semantic_model).map_err(CompilerError::Semantic)?);

        // Ports that declare physics are connected to ports of the same physics.
        modelica::check(&ast).map_err(CompilerError::Semantic)?;

        // Embedded C: the I/O bindings and the cases executed at fixed step.
        warnings.extend(embedded::check(&ast).map_err(CompilerError::Semantic)?);
        for config in &ast.configurations {
            semantic_model
                .all_elements
                .entry(config.id.clone())
                .or_insert_with(|| semantic::ElementInfo::new(config.id.clone(), config.name.clone(), "Configuration"));
        }

        // Code generation
        let output = codegen::CodeGenerator::new(&self.config).generate(&semantic_model)?;

        Ok(CompilationResult {
            ast,
            semantic_model,
            output,
            warnings,
            partition: Vec::new(),
            schedule: Vec::new(),
        })
    }
}
