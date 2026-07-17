pub mod repl;
pub mod language_server;

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(name = "arclang")]
#[clap(author = "ArcLang Contributors")]
#[clap(version = "1.0.0")]
#[clap(about = "Industrial-grade Arcadia-as-Code compiler", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    
    #[clap(short, long, global = true)]
    pub verbose: bool,
    
    #[clap(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    Build {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,
        
        #[clap(long)]
        incremental: bool,
        
        #[clap(long)]
        release: bool,
        
        #[clap(long)]
        target: Option<String>,
    },
    
    Check {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(long)]
        lint: bool,
        
        #[clap(long)]
        safety: bool,
    },
    
    Format {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(long)]
        check: bool,
        
        #[clap(long)]
        write: bool,
    },
    
    New {
        #[clap(value_parser)]
        name: String,
        
        #[clap(long)]
        template: Option<String>,
    },
    
    Sync {
        #[clap(subcommand)]
        sync_command: SyncCommands,
    },
    
    Plugin {
        #[clap(subcommand)]
        plugin_command: PluginCommands,
    },
    
    Trace {
        #[clap(value_parser)]
        input: PathBuf,

        #[clap(long)]
        from: Option<String>,

        #[clap(long)]
        to: Option<String>,

        #[clap(long)]
        validate: bool,

        #[clap(long)]
        matrix: bool,
    },

    /// Production-readiness gate: PASS/FAIL verdict against what an
    /// industrial design review requires (requirements coverage, HARA/ASIL
    /// consistency, timing budgets, ICD completeness)
    Gate {
        #[clap(value_parser)]
        input: PathBuf,

        #[clap(long, default_value = "iso26262")]
        standard: SafetyStandard,
    },

    /// Change-impact analysis: everything transitively affected when an
    /// element (requirement, component, function) changes
    Impact {
        #[clap(value_parser)]
        input: PathBuf,

        /// Element id or unambiguous name (e.g. "REQ-AEB-004")
        #[clap(value_parser)]
        element: String,
    },
    
    Export {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long, value_parser)]
        output: PathBuf,
        
        #[clap(short, long)]
        format: ExportFormat,
    },
    
    Import {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long)]
        format: ImportFormat,
        
        #[clap(short, long, value_parser)]
        output: PathBuf,
    },
    
    Safety {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(long)]
        standard: SafetyStandard,
        
        #[clap(long)]
        fmea: bool,
        
        #[clap(long)]
        fta: bool,
        
        #[clap(long)]
        report: bool,
    },
    
    Serve {
        #[clap(long, default_value = "5001")]
        port: u16,
    },
    
    Lsp {
        #[clap(long)]
        stdio: bool,
        
        #[clap(long)]
        port: Option<u16>,
    },
    
    Explorer {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,
        
        #[clap(long, help = "Open in browser after generation")]
        open: bool,
    },
    
    Repl {
        #[clap(value_parser)]
        project: Option<PathBuf>,
    },
    
    Clean {
        #[clap(value_parser)]
        project: PathBuf,
        
        #[clap(long)]
        cache: bool,
    },
    
    Info {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(long)]
        metrics: bool,
        
        #[clap(long)]
        dependencies: bool,
    },
    
    Diagram {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long, value_parser)]
        output: PathBuf,
        
        #[clap(short, long, default_value = "mermaid")]
        format: DiagramFormat,
        
        #[clap(long, default_value = "System Architecture")]
        title: String,
        
        #[clap(long)]
        open: bool,
    },
}

#[derive(Subcommand)]
pub enum SyncCommands {
    Pull {
        #[clap(long)]
        plm: Option<String>,
        
        #[clap(long)]
        requirements: Option<String>,
        
        #[clap(long)]
        dry_run: bool,
    },
    
    Push {
        #[clap(long)]
        plm: Option<String>,
        
        #[clap(long)]
        requirements: Option<String>,
        
        #[clap(long)]
        dry_run: bool,
    },
    
    Status {
        #[clap(value_parser)]
        project: PathBuf,
    },
    
    Configure {
        #[clap(long)]
        plm_type: String,
        
        #[clap(long)]
        url: String,
        
        #[clap(long)]
        credentials: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum PluginCommands {
    List,
    
    Install {
        #[clap(value_parser)]
        name: String,
        
        #[clap(long)]
        version: Option<String>,
    },
    
    Uninstall {
        #[clap(value_parser)]
        name: String,
    },
    
    Info {
        #[clap(value_parser)]
        name: String,
    },
    
    Enable {
        #[clap(value_parser)]
        name: String,
    },
    
    Disable {
        #[clap(value_parser)]
        name: String,
    },
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum ExportFormat {
    Capella,
    JSON,
    YAML,
    XML,
    Markdown,
    Mermaid,
    PlantUML,
    HTML,
    PDF,
    Terraform,
    SysML,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum ImportFormat {
    Capella,
    Mermaid,
    PlantUML,
    JSON,
    YAML,
    XML,
    DOORS,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum SafetyStandard {
    ISO26262,
    DO178C,
    IEC61508,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum DiagramFormat {
    Mermaid,
    PlantUML,
    Graphviz,
    SVG,
    Operational,
    Functional,
    Sequence,
    StateMachine,
    Component,
    Physical,
    Class,
    Tree,
    Capability,
    FunctionalChain,
    All,
}

pub struct CliRunner {
    verbose: bool,
    config_path: Option<PathBuf>,
}

impl CliRunner {
    pub fn new(cli: &Cli) -> Self {
        Self {
            verbose: cli.verbose,
            config_path: cli.config.clone(),
        }
    }
    
    pub fn run(&self, command: Commands) -> Result<(), CliError> {
        match command {
            Commands::Build { input, output, incremental, release, target } => {
                self.run_build(input, output, incremental, release, target)
            }
            Commands::Check { input, lint, safety } => {
                self.run_check(input, lint, safety)
            }
            Commands::Format { input, check, write } => {
                self.run_format(input, check, write)
            }
            Commands::New { name, template } => {
                self.run_new(name, template)
            }
            Commands::Sync { sync_command } => {
                self.run_sync(sync_command)
            }
            Commands::Plugin { plugin_command } => {
                self.run_plugin(plugin_command)
            }
            Commands::Trace { input, from, to, validate, matrix } => {
                self.run_trace(input, from, to, validate, matrix)
            }
            Commands::Impact { input, element } => {
                self.run_impact(input, element)
            }
            Commands::Gate { input, standard } => {
                self.run_gate(input, standard)
            }
            Commands::Export { input, output, format } => {
                self.run_export(input, output, format)
            }
            Commands::Import { input, format, output } => {
                self.run_import(input, format, output)
            }
            Commands::Safety { input, standard, fmea, fta, report } => {
                self.run_safety(input, standard, fmea, fta, report)
            }
            Commands::Serve { port } => {
                self.run_serve(port)
            }
            Commands::Lsp { stdio, port } => {
                self.run_lsp(stdio, port)
            }
            Commands::Explorer { input, output, open } => {
                self.run_explorer(input, output, open)
            }
            Commands::Repl { project } => {
                self.run_repl(project)
            }
            Commands::Clean { project, cache } => {
                self.run_clean(project, cache)
            }
            Commands::Info { input, metrics, dependencies } => {
                self.run_info(input, metrics, dependencies)
            }
            Commands::Diagram { input, output, format, title, open } => {
                self.run_diagram(input, output, format, title, open)
            }
        }
    }
    
    fn run_build(
        &self,
        input: PathBuf,
        output: Option<PathBuf>,
        incremental: bool,
        release: bool,
        target: Option<String>,
    ) -> Result<(), CliError> {
        println!("Building {}...", input.display());
        
        let mut config = crate::CompilerConfig::default();
        config.optimization_level = if release { 3 } else { 0 };
        
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                let output_path = output.unwrap_or_else(|| {
                    input.with_extension("json")
                });
                
                if let Err(e) = std::fs::write(&output_path, &result.output) {
                    return Err(CliError::Io(e));
                }
                
                if !result.warnings.is_empty() {
                    eprintln!("⚠ {} warning(s):", result.warnings.len());
                    for warning in &result.warnings {
                        eprintln!("  warning: {}", warning);
                    }
                }

                println!("✓ Compilation successful");
                println!("  Output: {}", output_path.display());
                println!("  Requirements: {}", result.semantic_model.requirements.len());
                println!("  Components: {}", result.semantic_model.components.len());
                println!("  Functions: {}", result.semantic_model.functions.len());
                println!("  Traces: {}", result.semantic_model.traces.len());

                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Compilation failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn run_check(&self, input: PathBuf, lint: bool, safety: bool) -> Result<(), CliError> {
        println!("Checking {}...", input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                println!("✓ No compilation errors");

                if !result.warnings.is_empty() {
                    println!("\n⚠ Compilation warnings:");
                    for warning in &result.warnings {
                        println!("  {}", warning);
                    }
                }

                let warnings = result.semantic_model.validate_traceability();
                if !warnings.is_empty() {
                    println!("\n⚠ Traceability warnings:");
                    for warning in &warnings {
                        println!("  {}", warning);
                    }
                }
                
                if lint {
                    let lints = crate::compiler::semantic::arcadia_methodology_lints(&result.ast);
                    if lints.is_empty() {
                        println!("\n✓ Arcadia methodology: no advisories");
                    } else {
                        println!("\nℹ Arcadia methodology advisories:");
                        for advisory in &lints {
                            println!("  {}", advisory);
                        }
                    }
                }

                if lint || safety {
                    println!("\nModel metrics:");
                    let metrics = result.semantic_model.compute_metrics();
                    println!("  Total elements: {}", metrics.total_elements);
                    println!("  Requirements: {}", metrics.requirements_count);
                    println!("  Components: {}", metrics.components_count);
                    println!("  Traces: {}", metrics.traces_count);
                    println!("  Traceability: {:.1}%", metrics.traceability_coverage);
                }
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Check failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn run_format(&self, _input: PathBuf, _check: bool, _write: bool) -> Result<(), CliError> {
        Err(CliError::NotImplemented("the formatter is not implemented yet".to_string()))
    }

    fn run_new(&self, _name: String, _template: Option<String>) -> Result<(), CliError> {
        Err(CliError::NotImplemented(
            "project scaffolding ('new') is not implemented yet".to_string(),
        ))
    }

    fn run_sync(&self, _sync_command: SyncCommands) -> Result<(), CliError> {
        Err(CliError::NotImplemented(
            "PLM synchronization is not implemented yet".to_string(),
        ))
    }
    
    fn run_plugin(&self, _plugin_command: PluginCommands) -> Result<(), CliError> {
        Err(CliError::NotImplemented(
            "the plugin system is not implemented yet".to_string(),
        ))
    }
    
    fn run_trace(
        &self,
        input: PathBuf,
        from: Option<String>,
        to: Option<String>,
        validate: bool,
        matrix: bool,
    ) -> Result<(), CliError> {
        println!("Analyzing traceability in {}...", input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                if validate {
                    let warnings = result.semantic_model.validate_traceability();
                    if warnings.is_empty() {
                        println!("✓ All elements properly traced");
                    } else {
                        println!("⚠ Traceability issues found:");
                        for warning in &warnings {
                            println!("  {}", warning);
                        }
                    }
                }
                
                if matrix {
                    println!("\nTraceability Matrix:");
                    println!("═══════════════════════════════════════");
                    for trace in &result.semantic_model.traces {
                        println!("  {} → {}", trace.from, trace.to);
                        if let Some(ref rationale) = trace.rationale {
                            println!("    Rationale: {}", rationale);
                        }
                    }
                }
                
                let metrics = result.semantic_model.compute_metrics();
                println!("\nTraceability Coverage: {:.1}%", metrics.traceability_coverage);
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Trace analysis failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn run_gate(&self, input: PathBuf, standard: SafetyStandard) -> Result<(), CliError> {
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        let result = compiler
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;

        let standard_name = match standard {
            SafetyStandard::ISO26262 => "ISO26262",
            SafetyStandard::DO178C => "DO178C",
            SafetyStandard::IEC61508 => "IEC61508",
        };
        let report = crate::compiler::production_gate::run_gate(
            &result.ast,
            &result.semantic_model,
            standard_name,
        );

        println!("Production-readiness gate ({}) — {}", standard_name, input.display());
        println!(
            "  Requirements: {} total, {} satisfied, {} verified",
            report.requirements_total, report.requirements_satisfied, report.requirements_verified
        );

        let blockers: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.severity == crate::compiler::production_gate::Severity::Blocker)
            .collect();
        let gate_warnings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.severity == crate::compiler::production_gate::Severity::Warning)
            .collect();

        if !blockers.is_empty() {
            println!("\n✗ {} blocker(s):", blockers.len());
            for finding in &blockers {
                println!("  [{}] {}", finding.check, finding.message);
            }
        }
        if !gate_warnings.is_empty() {
            println!("\n⚠ {} warning(s):", gate_warnings.len());
            for finding in &gate_warnings {
                println!("  [{}] {}", finding.check, finding.message);
            }
        }

        if report.passed {
            println!("\nPRODUCTION GATE: PASS ✓");
            Ok(())
        } else {
            println!("\nPRODUCTION GATE: FAIL ✗ ({} blockers)", blockers.len());
            Err(CliError::Compilation(format!(
                "production gate failed with {} blocker(s)",
                blockers.len()
            )))
        }
    }

    fn run_impact(&self, input: PathBuf, element: String) -> Result<(), CliError> {
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        let result = compiler
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;

        let entries = result
            .semantic_model
            .impact_of(&element)
            .ok_or_else(|| {
                CliError::Compilation(format!(
                    "element '{}' not found (or its name is ambiguous — use an id)",
                    element
                ))
            })?;

        println!("Impact analysis for '{}':", element);
        if entries.is_empty() {
            println!("  No connected elements — changing it affects nothing traced.");
            return Ok(());
        }

        let mut current_depth = 0;
        for entry in &entries {
            if entry.depth != current_depth {
                current_depth = entry.depth;
                let label = if current_depth == 1 { "directly affected" } else { "transitively affected" };
                println!("\n  Distance {} ({}):", current_depth, label);
            }
            println!(
                "    {} '{}' [{}] — via {} from '{}'",
                entry.element_type, entry.name, entry.id, entry.via, entry.via_element
            );
        }
        println!("\n  Total affected elements: {}", entries.len());
        Ok(())
    }

    fn run_export(
        &self,
        input: PathBuf,
        output: PathBuf,
        format: ExportFormat,
    ) -> Result<(), CliError> {
        println!("Exporting {} to {:?} format...", input.display(), format);
        
        let mut config = crate::CompilerConfig::default();
        
        // Set target based on format (Mermaid uses default, others specify target)
        config.target = match format {
            ExportFormat::JSON => "json".to_string(),
            ExportFormat::Capella => "capella".to_string(),
            ExportFormat::XML => "capella".to_string(),
            ExportFormat::Markdown => "markdown".to_string(),
            ExportFormat::Mermaid => "json".to_string(),
            ExportFormat::PlantUML => "json".to_string(),
            ExportFormat::HTML => "json".to_string(),
            ExportFormat::PDF => "json".to_string(),
            ExportFormat::YAML => "json".to_string(),
            ExportFormat::Terraform => "terraform".to_string(),
            ExportFormat::SysML => "json".to_string(),
        };
        
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                let output_content = match format {
                    ExportFormat::JSON => {
                        // Export the raw AST model as JSON for diagram rendering
                        result.ast.to_json()
                            .map_err(|e| CliError::Compilation(format!("JSON export failed: {}", e)))?
                    }
                    ExportFormat::Mermaid => {
                        use crate::compiler::mermaid_generator::generate_mermaid_flowchart;
                        generate_mermaid_flowchart(&result.semantic_model, "System Requirements", "elk")
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    ExportFormat::PlantUML => {
                        use crate::compiler::plantuml_generator::generate_plantuml_component;
                        generate_plantuml_component(&result.semantic_model)
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    ExportFormat::HTML => {
                        // The explorer is the renderer that actually draws the
                        // diagram (layers, components, ports, labeled edges).
                        // The ELK v2 pipeline's to_html only dumps layout JSON.
                        use crate::compiler::arcviz_explorer::generate_explorer_html;

                        let mut semantic_model = result.semantic_model.clone();
                        if semantic_model.name.is_none() {
                            // No `model` header: fall back to the file name.
                            semantic_model.name = input
                                .file_stem()
                                .map(|stem| stem.to_string_lossy().to_string());
                        }
                        let (html, _json) = generate_explorer_html(&semantic_model, &result.ast)
                            .map_err(|e| CliError::Compilation(format!("HTML generation failed: {}", e)))?;
                        html
                    }
                    ExportFormat::SysML => {
                        // SysML v2 textual notation (interoperability subset)
                        crate::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model)
                    }
                    ExportFormat::PDF => {
                        return Err(CliError::NotImplemented(
                            "PDF export not implemented in v3 — use --format html and print to PDF".to_string()
                        ));
                    }
                    ExportFormat::Terraform => {
                        use crate::compiler::terraform_databricks_generator::{generate_terraform_databricks, TerraformConfig};
                        let config = TerraformConfig::default();
                        generate_terraform_databricks(&result.semantic_model, &config)
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    _ => result.output
                };
                
                std::fs::write(&output, &output_content)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Export successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                println!("  Format: {:?}", format);
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Export failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn run_import(
        &self,
        input: PathBuf,
        format: ImportFormat,
        output: PathBuf,
    ) -> Result<(), CliError> {
        println!("Importing from {:?}: {}...", format, input.display());
        
        match format {
            ImportFormat::Capella => {
                use crate::compiler::capella_importer::{CapellaImporter, ArcCodeGenerator};
                
                let importer = CapellaImporter::new();
                let model = importer.import_file(&input)
                    .map_err(|e| CliError::Compilation(e.to_string()))?;
                
                let generator = ArcCodeGenerator::new();
                let arc_code = generator.generate(&model)
                    .map_err(|e| CliError::Compilation(e.to_string()))?;
                
                std::fs::write(&output, arc_code)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Import successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                
                Ok(())
            }
            ImportFormat::Mermaid => {
                use crate::compiler::mermaid_importer::import_mermaid;
                
                let content = std::fs::read_to_string(&input)
                    .map_err(|e| CliError::Io(e))?;
                
                let arc_code = import_mermaid(&content)
                    .map_err(|e| CliError::Compilation(e.to_string()))?;
                
                std::fs::write(&output, arc_code)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Import successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                println!("  Format: Mermaid -> ArcLang");
                
                Ok(())
            }
            ImportFormat::PlantUML => {
                use crate::compiler::plantuml_importer::import_plantuml;
                
                let content = std::fs::read_to_string(&input)
                    .map_err(|e| CliError::Io(e))?;
                
                let arc_code = import_plantuml(&content)
                    .map_err(|e| CliError::Compilation(e.to_string()))?;
                
                std::fs::write(&output, arc_code)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Import successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                println!("  Format: PlantUML -> ArcLang");
                
                Ok(())
            }
            _ => {
                println!("⚠ Format {:?} not yet implemented", format);
                Err(CliError::Config(format!("Import format {:?} not supported yet", format)))
            }
        }
    }
    
    fn run_safety(
        &self,
        input: PathBuf,
        standard: SafetyStandard,
        fmea: bool,
        fta: bool,
        report: bool,
    ) -> Result<(), CliError> {
        if fta {
            return Err(CliError::NotImplemented(
                "FTA generation is not implemented yet".to_string(),
            ));
        }
        if report {
            return Err(CliError::NotImplemented(
                "safety report generation is not implemented yet".to_string(),
            ));
        }

        println!("Safety analysis ({:?}) of {}...", standard, input.display());

        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        let result = compiler
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;

        let safety_blocks = &result.ast.safety_analysis;
        let hazard_count: usize = safety_blocks.iter().map(|s| s.hazards.len()).sum();
        let fmea_count: usize = safety_blocks.iter().map(|s| s.fmea.len()).sum();

        println!("  Safety analysis blocks: {}", safety_blocks.len());
        println!("  Hazards: {}", hazard_count);
        println!("  FMEA entries: {}", fmea_count);

        if hazard_count == 0 && fmea_count == 0 {
            println!("⚠ No safety analysis content found in the model.");
            println!("  Add a 'safety_analysis' block with 'hazard' and 'fmea' entries.");
        }

        if fmea {
            if fmea_count == 0 {
                return Err(CliError::Compilation(
                    "cannot report FMEA: the model contains no fmea entries".to_string(),
                ));
            }
            println!("\nFMEA entries:");
            for block in safety_blocks {
                for entry in &block.fmea {
                    println!("  - {}", entry.name);
                    for (key, value) in &entry.attributes {
                        println!("      {}: {:?}", key, value);
                    }
                }
            }
        }

        Ok(())
    }
    
    fn run_serve(&self, port: u16) -> Result<(), CliError> {
        use colored::Colorize;
        
        println!("{}", "🚀 Starting ArcLang Rust Backend Server".bright_cyan().bold());
        println!("{}", format!("   Port: {}", port).bright_white());
        println!("{}", format!("   Professional 7D Arcadia Diagrams").bright_green());
        println!();
        
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::Compilation(format!("Failed to create runtime: {}", e)))?;
        
        runtime.block_on(async {
            crate::web_server::serve(port).await
        }).map_err(|e| CliError::Compilation(format!("Server error: {}", e)))?;
        
        Ok(())
    }
    
    fn run_lsp(&self, _stdio: bool, port: Option<u16>) -> Result<(), CliError> {
        if port.is_some() {
            return Err(CliError::NotImplemented(
                "TCP mode is not implemented; use stdio (arclang lsp --stdio)".to_string(),
            ));
        }
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::Compilation(format!("Failed to create runtime: {}", e)))?;
        runtime.block_on(language_server::run_stdio());
        Ok(())
    }

    fn run_repl(&self, _project: Option<PathBuf>) -> Result<(), CliError> {
        Err(CliError::NotImplemented("the REPL is not implemented yet".to_string()))
    }

    fn run_clean(&self, _project: PathBuf, _cache: bool) -> Result<(), CliError> {
        Err(CliError::NotImplemented("'clean' is not implemented yet".to_string()))
    }
    
    fn run_info(
        &self,
        input: PathBuf,
        metrics: bool,
        dependencies: bool,
    ) -> Result<(), CliError> {
        if dependencies {
            return Err(CliError::NotImplemented(
                "dependency analysis is not implemented yet".to_string(),
            ));
        }

        println!("Model information: {}", input.display());

        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        let result = compiler
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;

        if let Some(name) = result.ast.attributes.get("name") {
            println!("  Name: {:?}", name);
        }
        if let Some(version) = result.ast.attributes.get("version") {
            println!("  Version: {:?}", version);
        }

        let model_metrics = result.semantic_model.compute_metrics();
        println!("  Total elements: {}", model_metrics.total_elements);
        println!("  Requirements: {}", model_metrics.requirements_count);
        println!("  Components: {}", model_metrics.components_count);
        println!("  Functions: {}", result.semantic_model.functions.len());
        println!("  Traces: {}", model_metrics.traces_count);

        if metrics {
            println!("  Traceability coverage: {:.1}%", model_metrics.traceability_coverage);
        }

        Ok(())
    }
    
    fn run_explorer(
        &self,
        input: PathBuf,
        output: Option<PathBuf>,
        open: bool,
    ) -> Result<(), CliError> {
        println!("🔍 Generating Architecture Explorer from {}...", input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                use crate::compiler::arcviz_explorer::generate_explorer_html;

                let mut semantic_model = result.semantic_model.clone();
                if semantic_model.name.is_none() {
                    // No `model` header: fall back to the file name.
                    semantic_model.name = input
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().to_string());
                }
                let (html, json) = generate_explorer_html(&semantic_model, &result.ast)
                    .map_err(|e| CliError::Compilation(e.to_string()))?;
                
                let output_html = output.unwrap_or_else(|| {
                    input.with_file_name(format!("{}_explorer.html", 
                        input.file_stem().unwrap().to_str().unwrap()))
                });
                
                let output_json = output_html.with_extension("json");
                
                std::fs::write(&output_html, &html)
                    .map_err(|e| CliError::Io(e))?;
                std::fs::write(&output_json, &json)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Architecture Explorer generated successfully");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output_html.display());
                println!("  Data: {}", output_json.display());
                println!("  Requirements: {}", result.semantic_model.requirements.len());
                println!("  Components: {}", result.semantic_model.components.len());
                println!("  Interfaces: {}", result.semantic_model.interfaces.len());
                println!("  Functions: {}", result.semantic_model.functions.len());
                
                println!("\n📋 Features:");
                println!("  • Interactive architecture diagram");
                println!("  • Expandable requirements & components");
                println!("  • Complete traceability matrix");
                println!("  • Floating table of contents");
                println!("  • PDF & HTML export");
                
                if open {
                    println!("\n🌐 Opening explorer in browser...");
                    opener::open(&output_html)
                        .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                }
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Explorer generation failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn run_diagram(
        &self,
        input: PathBuf,
        output: PathBuf,
        format: DiagramFormat,
        title: String,
        open: bool,
    ) -> Result<(), CliError> {
        println!("🎨 Generating {:?} diagram from {}...", format, input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                match format {
                    DiagramFormat::Mermaid => {
                        use crate::compiler::mermaid_generator::generate_mermaid_flowchart;
                        let diagram = generate_mermaid_flowchart(&result.semantic_model, &title, "elk")
                            .map_err(|e| CliError::Compilation(e.to_string()))?;
                        
                        std::fs::write(&output, &diagram)
                            .map_err(|e| CliError::Io(e))?;
                        
                        println!("✓ Mermaid diagram generated");
                        println!("  Output: {}", output.display());
                        
                        if open {
                            self.open_mermaid_diagram(&output)?;
                        }
                    }
                    
                    DiagramFormat::All => {
                        self.generate_all_capella_diagrams(&input, &result, &output)?;
                    }
                    
                    _ => {
                        self.generate_capella_diagram(&input, &result, &output, format)?;
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Diagram generation failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn generate_capella_diagram(
        &self,
        input: &PathBuf,
        result: &crate::CompilationResult,
        output: &PathBuf,
        format: DiagramFormat,
    ) -> Result<(), CliError> {
        // Step 1: Export AST to JSON
        let json_data = result.ast.to_json()
            .map_err(|e| CliError::Compilation(format!("JSON export failed: {}", e)))?;
        
        let temp_json = std::env::temp_dir().join(format!("arclang_model_{}.json", std::process::id()));
        std::fs::write(&temp_json, &json_data)
            .map_err(|e| CliError::Io(e))?;
        
        // Step 2: Determine diagram service path
        let diagram_service_dir = PathBuf::from("/Users/malek/Arclang/arcviz-web/apps/diagram-service");
        
        if !diagram_service_dir.exists() {
            return Err(CliError::Config(
                "Diagram service not found. Please ensure arcviz-web/apps/diagram-service is installed.".to_string()
            ));
        }
        
        // Step 3: Call Node.js diagram renderer
        let diagram_type = match format {
            DiagramFormat::Operational => "operational",
            DiagramFormat::Functional => "functional",
            DiagramFormat::Sequence => "sequence",
            DiagramFormat::StateMachine => "statemachine",
            DiagramFormat::Component => "component",
            DiagramFormat::Physical => "physical",
            DiagramFormat::Class => "class",
            DiagramFormat::Tree => "tree",
            DiagramFormat::Capability => "capability",
            DiagramFormat::FunctionalChain => "functional-chain",
            _ => return Err(CliError::Config(format!("Unsupported format: {:?}", format)))
        };
        
        let test_script = format!("test-{}.js", diagram_type);
        let script_path = diagram_service_dir.join(&test_script);
        
        if !script_path.exists() {
            return Err(CliError::Config(
                format!("Diagram renderer script not found: {}", test_script)
            ));
        }
        
        println!("  📊 Rendering {} diagram...", diagram_type);
        
        // Convert output to absolute path
        let abs_output = std::fs::canonicalize(output.parent().unwrap_or_else(|| Path::new(".")))
            .map_err(|e| CliError::Io(e))?
            .join(output.file_name().unwrap_or_else(|| std::ffi::OsStr::new("output.svg")));
        
        let node_output = std::process::Command::new("node")
            .current_dir(&diagram_service_dir)
            .arg(&test_script)
            .arg(&temp_json)
            .arg(&abs_output)
            .output()
            .map_err(|e| CliError::Io(e))?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_json);
        
        if !node_output.status.success() {
            let stderr = String::from_utf8_lossy(&node_output.stderr);
            return Err(CliError::Compilation(format!("Diagram rendering failed: {}", stderr)));
        }
        
        println!("✓ {} diagram generated successfully", diagram_type.to_uppercase());
        println!("  Input: {}", input.display());
        println!("  Output: {}", output.display());
        
        Ok(())
    }
    
    fn generate_all_capella_diagrams(
        &self,
        input: &PathBuf,
        result: &crate::CompilationResult,
        base_output: &PathBuf,
    ) -> Result<(), CliError> {
        println!("📦 Generating all Capella diagrams...\n");
        
        let base_name = base_output.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("diagram");
        
        let output_dir = base_output.parent().unwrap_or_else(|| Path::new("."));
        
        let diagram_types = vec![
            (DiagramFormat::Operational, "operational"),
            (DiagramFormat::Functional, "functional"),
            (DiagramFormat::Sequence, "sequence"),
            (DiagramFormat::StateMachine, "statemachine"),
            (DiagramFormat::Component, "component"),
            (DiagramFormat::Physical, "physical"),
            (DiagramFormat::Class, "class"),
            (DiagramFormat::Tree, "tree"),
            (DiagramFormat::Capability, "capability"),
            (DiagramFormat::FunctionalChain, "functional-chain"),
        ];
        
        let mut success_count = 0;
        let mut total = diagram_types.len();
        
        for (format, name) in diagram_types {
            let output_path = output_dir.join(format!("{}_{}.svg", base_name, name));
            
            match self.generate_capella_diagram(input, result, &output_path, format) {
                Ok(_) => {
                    success_count += 1;
                    println!();
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to generate {} diagram: {}\n", name, e);
                }
            }
        }
        
        println!("═══════════════════════════════════════");
        println!("✓ Generated {}/{} diagrams successfully", success_count, total);
        println!("  Output directory: {}", output_dir.display());
        
        Ok(())
    }
    
    fn open_mermaid_diagram(&self, mermaid_file: &PathBuf) -> Result<(), CliError> {
        let content = std::fs::read_to_string(mermaid_file)
            .map_err(|e| CliError::Io(e))?;
        
        // Create HTML wrapper
        let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>ArcLang Diagram</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
    <script>
        mermaid.initialize({{ startOnLoad: true, theme: 'default' }});
    </script>
    <style>
        body {{
            margin: 0;
            padding: 20px;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: #f5f5f5;
        }}
        .container {{
            max-width: 100%;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-top: 0;
        }}
        .mermaid {{
            display: flex;
            justify-content: center;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 ArcLang Diagram</h1>
        <div class="mermaid">
{}
        </div>
    </div>
</body>
</html>"#, content);
        
        let html_file = mermaid_file.with_extension("html");
        std::fs::write(&html_file, html)
            .map_err(|e| CliError::Io(e))?;
        
        // Open in browser
        #[cfg(target_os = "macos")]
        std::process::Command::new("open")
            .arg(&html_file)
            .spawn()
            .map_err(|e| CliError::Io(e))?;
        
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open")
            .arg(&html_file)
            .spawn()
            .map_err(|e| CliError::Io(e))?;
        
        #[cfg(target_os = "windows")]
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", html_file.to_str().unwrap()])
            .spawn()
            .map_err(|e| CliError::Io(e))?;
        
        Ok(())
    }
    
    fn log(&self, message: &str) {
        if self.verbose {
            println!("[INFO] {}", message);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Compilation error: {0}")]
    Compilation(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
