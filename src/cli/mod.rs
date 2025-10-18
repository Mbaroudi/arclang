pub mod commands;
pub mod repl;
pub mod language_server;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    
    Lsp {
        #[clap(long)]
        stdio: bool,
        
        #[clap(long)]
        port: Option<u16>,
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
    ArcViz,
    ArcVizSmart,
    ArcVizChannel,
    ArcVizPerfect,
    ArcVizUltimate,
    HTML,
    PDF,
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
            Commands::Export { input, output, format } => {
                self.run_export(input, output, format)
            }
            Commands::Import { input, format, output } => {
                self.run_import(input, format, output)
            }
            Commands::Safety { input, standard, fmea, fta, report } => {
                self.run_safety(input, standard, fmea, fta, report)
            }
            Commands::Lsp { stdio, port } => {
                self.run_lsp(stdio, port)
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
                
                let warnings = result.semantic_model.validate_traceability();
                if !warnings.is_empty() {
                    println!("\n⚠ Traceability warnings:");
                    for warning in &warnings {
                        println!("  {}", warning);
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
    
    fn run_format(&self, input: PathBuf, check: bool, write: bool) -> Result<(), CliError> {
        if check {
            self.log("Checking formatting...");
        } else if write {
            self.log("Formatting files...");
        }
        
        Ok(())
    }
    
    fn run_new(&self, name: String, template: Option<String>) -> Result<(), CliError> {
        self.log(&format!("Creating new project: {}", name));
        
        if let Some(tmpl) = template {
            self.log(&format!("Using template: {}", tmpl));
        }
        
        Ok(())
    }
    
    fn run_sync(&self, sync_command: SyncCommands) -> Result<(), CliError> {
        match sync_command {
            SyncCommands::Pull { plm, requirements, dry_run } => {
                self.log("Pulling from external systems...");
                if dry_run {
                    self.log("(Dry run mode)");
                }
            }
            SyncCommands::Push { plm, requirements, dry_run } => {
                self.log("Pushing to external systems...");
                if dry_run {
                    self.log("(Dry run mode)");
                }
            }
            SyncCommands::Status { project } => {
                self.log("Checking sync status...");
            }
            SyncCommands::Configure { plm_type, url, credentials } => {
                self.log(&format!("Configuring {} at {}", plm_type, url));
            }
        }
        
        Ok(())
    }
    
    fn run_plugin(&self, plugin_command: PluginCommands) -> Result<(), CliError> {
        match plugin_command {
            PluginCommands::List => {
                self.log("Installed plugins:");
            }
            PluginCommands::Install { name, version } => {
                self.log(&format!("Installing plugin: {}", name));
            }
            PluginCommands::Uninstall { name } => {
                self.log(&format!("Uninstalling plugin: {}", name));
            }
            PluginCommands::Info { name } => {
                self.log(&format!("Plugin info: {}", name));
            }
            PluginCommands::Enable { name } => {
                self.log(&format!("Enabling plugin: {}", name));
            }
            PluginCommands::Disable { name } => {
                self.log(&format!("Disabling plugin: {}", name));
            }
        }
        
        Ok(())
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
            ExportFormat::ArcViz => "json".to_string(),
            ExportFormat::ArcVizSmart => "json".to_string(),
            ExportFormat::ArcVizChannel => "json".to_string(),
            ExportFormat::ArcVizPerfect => "json".to_string(),
            ExportFormat::ArcVizUltimate => "json".to_string(),
            _ => {
                println!("⚠ Format {:?} not yet implemented", format);
                return Err(CliError::Config(format!("Export format {:?} not supported yet", format)));
            }
        };
        
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                let output_content = match format {
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
                    ExportFormat::ArcViz => {
                        use crate::compiler::arcviz_generator::generate_arcviz_html;
                        generate_arcviz_html(&result.semantic_model, "System Requirements")
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    ExportFormat::ArcVizSmart => {
                        use crate::compiler::arcviz_smart_routing::{generate_smart_arcviz, wrap_smart_arcviz_html};
                        let svg = generate_smart_arcviz(&result.semantic_model, "System Architecture")
                            .map_err(|e| CliError::Compilation(e.to_string()))?;
                        wrap_smart_arcviz_html("System Architecture", &svg)
                    }
                    ExportFormat::ArcVizChannel => {
                        use crate::compiler::arcviz_channel_routing::{generate_channel_routed_arcviz, wrap_channel_routed_html};
                        let svg = generate_channel_routed_arcviz(&result.semantic_model, "System Architecture")
                            .map_err(|e| CliError::Compilation(e.to_string()))?;
                        wrap_channel_routed_html("System Architecture", &svg)
                    }
                    ExportFormat::ArcVizPerfect => {
                        use crate::compiler::arcviz_perfect_routing::{generate_perfect_arcviz, wrap_perfect_html};
                        let svg = generate_perfect_arcviz(&result.semantic_model, "System Architecture")
                            .map_err(|e| CliError::Compilation(e.to_string()))?;
                        wrap_perfect_html("System Architecture", &svg)
                    }
                    ExportFormat::ArcVizUltimate => {
                        use crate::compiler::arcviz_ultimate_routing::{generate_ultimate_arcviz, wrap_ultimate_html};
                        let svg = generate_ultimate_arcviz(&result.semantic_model, "System Architecture")
                            .map_err(|e| CliError::Compilation(e.to_string()))?;
                        wrap_ultimate_html("System Architecture", &svg)
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
        self.log(&format!("Running safety analysis ({:?})...", standard));
        
        if fmea {
            self.log("Generating FMEA...");
        }
        
        if fta {
            self.log("Generating FTA...");
        }
        
        if report {
            self.log("Generating safety report...");
        }
        
        Ok(())
    }
    
    fn run_lsp(&self, stdio: bool, port: Option<u16>) -> Result<(), CliError> {
        if stdio {
            self.log("Starting language server (stdio)...");
        } else if let Some(p) = port {
            self.log(&format!("Starting language server on port {}...", p));
        }
        
        Ok(())
    }
    
    fn run_repl(&self, project: Option<PathBuf>) -> Result<(), CliError> {
        self.log("Starting REPL...");
        Ok(())
    }
    
    fn run_clean(&self, project: PathBuf, cache: bool) -> Result<(), CliError> {
        self.log("Cleaning project...");
        
        if cache {
            self.log("Clearing cache...");
        }
        
        Ok(())
    }
    
    fn run_info(
        &self,
        input: PathBuf,
        metrics: bool,
        dependencies: bool,
    ) -> Result<(), CliError> {
        self.log("Project information:");
        
        if metrics {
            self.log("Computing metrics...");
        }
        
        if dependencies {
            self.log("Analyzing dependencies...");
        }
        
        Ok(())
    }
    
    fn run_diagram(
        &self,
        input: PathBuf,
        output: PathBuf,
        format: DiagramFormat,
        title: String,
        open: bool,
    ) -> Result<(), CliError> {
        println!("Generating {:?} diagram from {}...", format, input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                use crate::compiler::mermaid_generator::generate_mermaid_flowchart;
                
                let diagram = match format {
                    DiagramFormat::Mermaid => {
                        generate_mermaid_flowchart(&result.semantic_model, &title, "elk")
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    _ => {
                        return Err(CliError::Config(format!("Diagram format {:?} not yet supported", format)));
                    }
                };
                
                std::fs::write(&output, &diagram)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Diagram generated successfully");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                println!("  Format: {:?}", format);
                println!("  Title: {}", title);
                
                if open {
                    println!("\n📊 Opening diagram in browser...");
                    self.open_mermaid_diagram(&output)?;
                }
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Diagram generation failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
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
}
