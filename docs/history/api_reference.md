# ArcLang API Reference

This document provides a complete API reference for programmatically using ArcLang.

## Table of Contents

1. [Rust API](#rust-api)
2. [Plugin API](#plugin-api)
3. [Language Server Protocol](#language-server-protocol)
4. [REST API](#rest-api)

## Rust API

### Compiler Module

```rust
use arclang::compiler::Compiler;
use arclang::config::CompilerConfig;

// Create compiler instance
let config = CompilerConfig::default();
let mut compiler = Compiler::new(config)?;

// Compile a file
let result = compiler.compile_file("src/main.arc")?;

// Compile a string
let source = r#"
operational_analysis "System" {
    actor "User" {}
}
"#;
let result = compiler.compile_string(source)?;

// Get semantic model
let model = result.semantic_model;
```

### Semantic Model API

```rust
use arclang::plugins::api::{SemanticModel, QueryBuilder, ElementType, ArcadiaLevel};

// Query requirements
let requirements = model.requirements.iter()
    .filter(|r| r.priority == RequirementPriority::Critical)
    .collect::<Vec<_>>();

// Query by type and level
let results = QueryBuilder::new()
    .with_type(ElementType::Component)
    .with_level(ArcadiaLevel::LogicalArchitecture)
    .execute(&model);

// Get element by ID
let req = model.get_requirement("REQ-001")?;
let comp = model.get_component("LC-001")?;

// Navigate traces
let downstream = model.get_traces_from("REQ-001");
let upstream = model.get_traces_to("LC-001");

// Validate traceability
let issues = model.validate_traceability();
for issue in issues {
    println!("{:?}: {}", issue.issue_type, issue.description);
}

// Compute metrics
let metrics = model.compute_metrics();
println!("Total elements: {}", metrics.total_elements);
println!("Traceability coverage: {:.1}%", metrics.traceability_coverage);
```

### Incremental Compilation

```rust
use arclang::compiler::incremental::{IncrementalCompiler, IncrementalConfig};

let config = IncrementalConfig {
    cache_dir: PathBuf::from(".arclang/cache"),
    max_cache_size_mb: 1000,
    enable_parallel: true,
    num_threads: 8,
};

let mut compiler = IncrementalCompiler::new(config)?;

// Initial full compilation
let result = compiler.compile_full(&["src/main.arc"])?;

// Incremental compilation after changes
let changed_files = vec!["src/requirements.arc"];
let result = compiler.compile_incremental(&changed_files)?;

println!("Recompiled: {} files", result.recompiled_files.len());
println!("Cached: {} files", result.cached_files.len());
println!("Duration: {:?}", result.compilation_time);
```

### PLM Integration

```rust
use arclang::plm::{PLMConnector, WindchillConnector, PLMConfig};

// Configure PLM connection
let config = PLMConfig {
    url: "https://plm.company.com".to_string(),
    username: "user".to_string(),
    password: "pass".to_string(),
    project_id: "PROJ-001".to_string(),
};

let mut plm = WindchillConnector::new(config)?;

// Pull from PLM
let elements = plm.pull_elements()?;
println!("Pulled {} elements", elements.len());

// Push to PLM
plm.push_model(&model)?;

// Check sync status
let status = plm.check_sync_status()?;
println!("Out of sync: {}", status.out_of_sync_count);
```

### Requirements Management

```rust
use arclang::requirements::{DOORSConnector, RequirementsConfig};

let config = RequirementsConfig {
    url: "https://doors.company.com".to_string(),
    username: "user".to_string(),
    password: "pass".to_string(),
    module_name: "System_Requirements".to_string(),
};

let mut doors = DOORSConnector::new(config)?;

// Import requirements
let requirements = doors.import_requirements()?;
for req in requirements {
    println!("{}: {}", req.id, req.description);
}

// Export traces
doors.export_traces(&model.traces)?;

// Bidirectional sync
doors.sync_bidirectional(&mut model)?;
```

### Safety Analysis

```rust
use arclang::safety::{SafetyAnalyzer, SafetyConfig, SafetyStandard};
use arclang::safety::fmea::FMEAGenerator;
use arclang::safety::fta::FTAGenerator;

// Configure safety analyzer
let config = SafetyConfig {
    standard: SafetyStandard::ISO26262,
    target_asil: Some(ASILLevel::B),
    enable_fmea: true,
    enable_fta: true,
};

let analyzer = SafetyAnalyzer::new(config);

// Perform safety analysis
let result = analyzer.analyze(&model)?;

println!("Compliance: {:.1}%", result.compliance_percentage);
println!("Issues: {}", result.issues.len());

// Generate FMEA
let fmea_gen = FMEAGenerator::new(Default::default());
let fmea_entries = fmea_gen.generate(&model)?;

for entry in fmea_entries {
    println!("Component: {}", entry.component);
    println!("Failure Mode: {}", entry.failure_mode);
    println!("RPN: {}", entry.rpn);
}

// Generate FTA
let fta_gen = FTAGenerator::new(Default::default());
let hazard = model.hazards.first().unwrap();
let fault_tree = fta_gen.generate(&model, hazard)?;

println!("Minimal cut sets: {}", fault_tree.minimal_cut_sets.len());
```

### Collaboration

```rust
use arclang::collaboration::{CollaborationManager, SemanticMerger, ConflictResolver};

let mut collab = CollaborationManager::new(config)?;

// Semantic merge
let merger = SemanticMerger::new(vec![]);
let base = load_snapshot("base")?;
let ours = load_snapshot("ours")?;
let theirs = load_snapshot("theirs")?;

let merge_result = merger.merge(&base, &ours, &theirs)?;

if merge_result.has_conflicts {
    println!("Conflicts: {}", merge_result.conflicts.len());
    
    // Resolve conflicts
    let resolver = ConflictResolver::new(ConflictResolutionPolicy::Interactive);
    let resolution = resolver.auto_resolve(&merge_result.conflicts)?;
}

// Track changes
let change_tracker = collab.change_tracker();
change_tracker.track_change(ModelChange {
    change_type: ChangeType::Modified,
    element_id: "REQ-001".to_string(),
    old_value: "old description".to_string(),
    new_value: "new description".to_string(),
});

// Impact analysis
let impact = change_tracker.analyze_impact("REQ-001")?;
println!("Affected elements: {}", impact.affected_elements.len());
```

## Plugin API

### Creating a Plugin

```rust
use arclang::plugins::{Plugin, PluginContext, PluginResult, PluginError};
use arclang::plugins::api::SemanticModel;

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        context.logger.info("Initializing MyPlugin");
        Ok(())
    }
    
    fn execute(&self, context: &PluginContext) -> Result<PluginResult, PluginError> {
        context.logger.info("Executing MyPlugin");
        
        // Plugin logic here
        
        Ok(PluginResult {
            success: true,
            output: None,
            diagnostics: vec![],
        })
    }
    
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
```

### Compiler Pass Plugin

```rust
use arclang::plugins::{CompilerPassPlugin, Plugin};

pub struct OptimizationPass;

impl Plugin for OptimizationPass {
    // ... base implementation
}

impl CompilerPassPlugin for OptimizationPass {
    fn pass_name(&self) -> &str {
        "optimization"
    }
    
    fn pass_priority(&self) -> i32 {
        100  // Higher runs later
    }
    
    fn run_pass(&self, model: &mut SemanticModel) -> Result<(), PluginError> {
        // Optimize model
        for component in &mut model.components {
            // Optimization logic
        }
        
        Ok(())
    }
}
```

### Linter Plugin

```rust
use arclang::plugins::{LinterPlugin, LintDiagnostic, DiagnosticSeverity};

pub struct StyleLinter;

impl LinterPlugin for StyleLinter {
    fn lint(&self, model: &SemanticModel) -> Result<Vec<LintDiagnostic>, PluginError> {
        let mut diagnostics = vec![];
        
        for req in &model.requirements {
            if req.description.len() > 200 {
                diagnostics.push(LintDiagnostic {
                    rule_id: "req-length".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    message: "Requirement description too long".to_string(),
                    location: SourceLocation {
                        file: "main.arc".to_string(),
                        line: 10,
                        column: 5,
                    },
                    suggestion: Some("Split into multiple requirements".to_string()),
                });
            }
        }
        
        Ok(diagnostics)
    }
}
```

### Code Generator Plugin

```rust
use arclang::plugins::{CodeGeneratorPlugin, GeneratedCode, GeneratedFile};

pub struct PythonGenerator;

impl CodeGeneratorPlugin for PythonGenerator {
    fn target_language(&self) -> &str {
        "Python"
    }
    
    fn generate_code(&self, model: &SemanticModel) -> Result<GeneratedCode, PluginError> {
        let mut files = vec![];
        
        for component in &model.components {
            let content = format!(
                "class {}:\n    def __init__(self):\n        pass\n",
                component.name
            );
            
            files.push(GeneratedFile {
                path: format!("{}.py", component.name.to_lowercase()),
                content,
            });
        }
        
        Ok(GeneratedCode {
            language: "Python".to_string(),
            files,
        })
    }
}
```

### Visualization Plugin

```rust
use arclang::plugins::{VisualizationPlugin, Visualization, VisualizationType, VisualizationFormat};

pub struct GraphVisualizer;

impl VisualizationPlugin for GraphVisualizer {
    fn visualization_type(&self) -> VisualizationType {
        VisualizationType::Graph
    }
    
    fn generate_visualization(&self, model: &SemanticModel) -> Result<Visualization, PluginError> {
        let mut dot = String::from("digraph Model {\n");
        
        for trace in &model.traces {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", trace.from, trace.to));
        }
        
        dot.push_str("}\n");
        
        Ok(Visualization {
            viz_type: VisualizationType::Graph,
            format: VisualizationFormat::Graphviz,
            content: dot,
        })
    }
}
```

### Plugin Registration

```rust
use arclang::plugins::{PluginManager, PluginConfig};

let config = PluginConfig {
    plugin_dir: PathBuf::from("plugins"),
    enabled_plugins: vec!["my-plugin".to_string()],
    plugin_settings: HashMap::new(),
    auto_load: true,
};

let mut manager = PluginManager::new(config)?;

// Load plugins
manager.load_plugins()?;

// Execute plugin
let context = PluginContext::new(workspace_path, cache_dir);
let result = manager.execute_plugin("my-plugin", &context)?;

// Execute hook
let results = manager.execute_hook(&HookPoint::PreCompile, &context)?;

// Get plugins by capability
let linters = manager.get_plugins_by_capability(&PluginCapability::Linter);
```

## Language Server Protocol

### Starting the LSP Server

```rust
use arclang::cli::language_server::{LanguageServer, ServerMode};

let server = LanguageServer::new(ServerMode::Stdio);
server.start()?;
```

### LSP Capabilities

The ArcLang language server supports:

- **Text Document Sync**: Full and incremental
- **Completion**: Context-aware completion for keywords and identifiers
- **Hover**: Show element information on hover
- **Go to Definition**: Navigate to element definitions
- **Find References**: Find all references to an element
- **Document Symbols**: Show document outline
- **Workspace Symbols**: Search symbols across workspace
- **Diagnostics**: Real-time error and warning reporting
- **Code Actions**: Quick fixes and refactorings
- **Rename**: Rename elements with reference updates
- **Semantic Tokens**: Syntax highlighting

### IDE Integration

#### VS Code

```json
{
    "arclang.languageServer.enabled": true,
    "arclang.languageServer.path": "/usr/local/bin/arclang",
    "arclang.languageServer.args": ["lsp", "--stdio"]
}
```

#### Vim/Neovim (with coc.nvim)

```json
{
    "languageserver": {
        "arclang": {
            "command": "arclang",
            "args": ["lsp", "--stdio"],
            "filetypes": ["arclang"],
            "rootPatterns": ["Arclang.toml"]
        }
    }
}
```

## REST API

### Starting the REST Server

```rust
use arclang::server::RestServer;

let server = RestServer::new("0.0.0.0:8080");
server.start()?;
```

### API Endpoints

#### Compile Model

```
POST /api/v1/compile
Content-Type: application/json

{
    "source": "operational_analysis \"System\" { ... }",
    "options": {
        "incremental": true,
        "optimization_level": 2
    }
}

Response 200 OK:
{
    "success": true,
    "diagnostics": [],
    "model": { ... }
}
```

#### Validate Traceability

```
POST /api/v1/trace/validate
Content-Type: application/json

{
    "model_id": "model-123"
}

Response 200 OK:
{
    "valid": true,
    "issues": [],
    "coverage": 95.5
}
```

#### Safety Analysis

```
POST /api/v1/safety/analyze
Content-Type: application/json

{
    "model_id": "model-123",
    "standard": "ISO26262",
    "fmea": true,
    "fta": true
}

Response 200 OK:
{
    "compliance": 92.5,
    "hazards": [...],
    "fmea_entries": [...],
    "recommendations": [...]
}
```

#### Query Model

```
POST /api/v1/query
Content-Type: application/json

{
    "model_id": "model-123",
    "filters": [
        {"type": "Component"},
        {"level": "LogicalArchitecture"}
    ]
}

Response 200 OK:
{
    "results": [
        {
            "id": "LC-001",
            "name": "Component1",
            "type": "Component"
        }
    ]
}
```

## Error Handling

All API functions return `Result<T, Error>` where `Error` is one of:

- `CompilerError`: Compilation errors
- `PluginError`: Plugin execution errors
- `PLMError`: PLM integration errors
- `RequirementsError`: Requirements management errors
- `SafetyError`: Safety analysis errors
- `CollaborationError`: Collaboration errors

```rust
use arclang::error::ArcLangError;

match compiler.compile_file("main.arc") {
    Ok(result) => println!("Success!"),
    Err(ArcLangError::Compiler(e)) => eprintln!("Compilation error: {}", e),
    Err(ArcLangError::IO(e)) => eprintln!("IO error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Examples

See the `examples/` directory in the repository for complete working examples of API usage.
