# 🔧 API Reference

**Complete API documentation for ArcLang compiler integration**

---

## Table of Contents

1. [Introduction](#introduction)
2. [Rust API](#rust-api)
3. [CLI API](#cli-api)
4. [Programmatic Usage](#programmatic-usage)
5. [Code Generation API](#code-generation-api)
6. [Custom Tools](#custom-tools)
7. [Extension Points](#extension-points)

---

## Introduction

ArcLang provides multiple APIs for integration:

| API Type | Use Case | Language |
|----------|----------|----------|
| **Rust Library API** | Embed compiler in Rust applications | Rust |
| **CLI API** | Command-line tools and scripts | Shell/Any |
| **Programmatic API** | Build custom tools and extensions | Rust |
| **Code Generation API** | Generate custom outputs | Rust |

---

## Rust API

### Core Compiler API

#### Compile a Model

```rust
use arclang::compiler::{compile, CompilerOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("model.arc")?;
    
    let options = CompilerOptions {
        optimize: true,
        validate: true,
        trace_analysis: true,
    };
    
    let result = compile(&source, options)?;
    
    println!("Compiled successfully!");
    println!("Requirements: {}", result.semantic_model.requirements.len());
    println!("Components: {}", result.semantic_model.components.len());
    
    Ok(())
}
```

#### Lexer API

```rust
use arclang::compiler::lexer::{Lexer, Token, TokenKind};

fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token()?;
        if token.kind == TokenKind::Eof {
            break;
        }
        tokens.push(token);
    }
    
    Ok(tokens)
}

fn main() {
    let source = r#"
        system_analysis "Example" {
            requirement "REQ-001" {
                description: "Test requirement"
            }
        }
    "#;
    
    match tokenize(source) {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(e) => eprintln!("Lexer error: {}", e),
    }
}
```

#### Parser API

```rust
use arclang::compiler::parser::{Parser, Ast};
use arclang::compiler::lexer::Lexer;

fn parse(source: &str) -> Result<Ast, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    Ok(ast)
}

fn main() {
    let source = std::fs::read_to_string("model.arc").unwrap();
    
    match parse(&source) {
        Ok(ast) => {
            println!("Parse successful!");
            println!("Nodes: {}", ast.nodes.len());
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
```

#### Semantic Analysis API

```rust
use arclang::compiler::semantic::{SemanticAnalyzer, SemanticModel};

fn analyze(ast: Ast) -> Result<SemanticModel, String> {
    let mut analyzer = SemanticAnalyzer::new();
    let model = analyzer.analyze(ast)?;
    
    Ok(model)
}

fn main() {
    let source = std::fs::read_to_string("model.arc").unwrap();
    let ast = parse(&source).unwrap();
    
    match analyze(ast) {
        Ok(model) => {
            println!("Analysis successful!");
            println!("Requirements: {}", model.requirements.len());
            println!("Components: {}", model.components.len());
            println!("Traces: {}", model.traces.len());
        }
        Err(e) => eprintln!("Semantic error: {}", e),
    }
}
```

---

## CLI API

### Command-Line Interface

#### Build Command

```bash
arclang build model.arc [OPTIONS]

Options:
  -o, --output <FILE>      Output file path (default: model.json)
  --optimize               Enable optimizations
  --validate               Validate semantic model
  --verbose                Verbose output
  -h, --help               Print help
```

**Example:**
```bash
arclang build model.arc -o output.json --validate
```

#### Export Command

```bash
arclang export model.arc -o <OUTPUT> -f <FORMAT>

Options:
  -o, --output <FILE>      Output file path
  -f, --format <FORMAT>    Export format
  --title <TITLE>          Diagram title
  -h, --help               Print help

Formats:
  arc-viz-ultimate         Professional zero-crossing diagrams
  mermaid                  Mermaid flowchart format
  plant-uml                PlantUML format
  capella-xml              Capella XML format
```

**Example:**
```bash
arclang export model.arc -o diagram.html -f arc-viz-ultimate
```

#### Check Command

```bash
arclang check model.arc [OPTIONS]

Options:
  --lint                   Enable linting checks
  --strict                 Strict validation mode
  -h, --help               Print help
```

**Example:**
```bash
arclang check model.arc --lint --strict
```

#### Trace Command

```bash
arclang trace model.arc [OPTIONS]

Options:
  --validate               Validate traceability
  --matrix                 Show traceability matrix
  --coverage               Show coverage metrics
  --orphans                Find orphan elements
  --gaps                   Find traceability gaps
  --output <FILE>          Output file for reports
  -h, --help               Print help
```

**Example:**
```bash
arclang trace model.arc --validate --matrix --output trace_report.html
```

---

## Programmatic Usage

### Embedding ArcLang Compiler

#### Add Dependency

```toml
[dependencies]
arclang = { path = "../arclang" }
```

#### Basic Integration

```rust
use arclang::compiler::{compile, CompilerOptions};

pub struct MyTool {
    compiler_options: CompilerOptions,
}

impl MyTool {
    pub fn new() -> Self {
        Self {
            compiler_options: CompilerOptions {
                optimize: true,
                validate: true,
                trace_analysis: true,
            },
        }
    }
    
    pub fn compile_model(&self, source: &str) -> Result<String, String> {
        let result = compile(source, self.compiler_options.clone())?;
        
        let json = serde_json::to_string_pretty(&result.semantic_model)
            .map_err(|e| e.to_string())?;
        
        Ok(json)
    }
    
    pub fn generate_diagram(&self, source: &str) -> Result<String, String> {
        let result = compile(source, self.compiler_options.clone())?;
        
        use arclang::compiler::arcviz_ultimate_routing::{
            generate_ultimate_arcviz, wrap_ultimate_html
        };
        
        let svg = generate_ultimate_arcviz(&result.semantic_model, "Diagram")?;
        let html = wrap_ultimate_html("My Diagram", &svg);
        
        Ok(html)
    }
}

fn main() {
    let tool = MyTool::new();
    
    let source = std::fs::read_to_string("model.arc").unwrap();
    
    match tool.compile_model(&source) {
        Ok(json) => {
            std::fs::write("output.json", json).unwrap();
            println!("Compilation successful!");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    match tool.generate_diagram(&source) {
        Ok(html) => {
            std::fs::write("diagram.html", html).unwrap();
            println!("Diagram generated!");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Validation API

```rust
use arclang::compiler::semantic::SemanticModel;

pub fn validate_model(model: &SemanticModel) -> Vec<String> {
    let mut warnings = Vec::new();
    
    for requirement in &model.requirements {
        if !has_trace(&model, &requirement.id) {
            warnings.push(format!(
                "Requirement {} has no trace", 
                requirement.id
            ));
        }
    }
    
    for component in &model.components {
        if !satisfies_requirement(&model, &component.id) {
            warnings.push(format!(
                "Component {} doesn't satisfy any requirement", 
                component.id
            ));
        }
    }
    
    warnings
}

fn has_trace(model: &SemanticModel, id: &str) -> bool {
    model.traces.iter().any(|t| t.source == id || t.target == id)
}

fn satisfies_requirement(model: &SemanticModel, component_id: &str) -> bool {
    model.traces.iter().any(|t| 
        t.source == component_id && t.trace_type == "satisfies"
    )
}
```

---

## Code Generation API

### Custom Output Formats

#### Create Custom Generator

```rust
use arclang::compiler::semantic::SemanticModel;

pub trait OutputGenerator {
    fn generate(&self, model: &SemanticModel) -> Result<String, String>;
}

pub struct MarkdownGenerator;

impl OutputGenerator for MarkdownGenerator {
    fn generate(&self, model: &SemanticModel) -> Result<String, String> {
        let mut output = String::new();
        
        output.push_str("# System Architecture\n\n");
        
        output.push_str("## Requirements\n\n");
        for req in &model.requirements {
            output.push_str(&format!("- **{}**: {}\n", req.id, req.description));
        }
        
        output.push_str("\n## Components\n\n");
        for comp in &model.components {
            output.push_str(&format!("### {}\n", comp.name));
            output.push_str(&format!("ID: {}\n\n", comp.id));
            
            for func in &comp.functions {
                output.push_str(&format!("- Function: {}\n", func.name));
            }
            output.push_str("\n");
        }
        
        Ok(output)
    }
}

fn main() {
    let source = std::fs::read_to_string("model.arc").unwrap();
    let result = arclang::compiler::compile(&source, Default::default()).unwrap();
    
    let generator = MarkdownGenerator;
    let markdown = generator.generate(&result.semantic_model).unwrap();
    
    std::fs::write("output.md", markdown).unwrap();
}
```

#### Diagram Generator API

```rust
use arclang::compiler::arcviz_ultimate_routing::{
    generate_ultimate_arcviz, 
    wrap_ultimate_html,
    ArcVizConfig
};

pub fn generate_custom_diagram(
    model: &SemanticModel,
    config: ArcVizConfig,
) -> Result<String, String> {
    let svg = generate_ultimate_arcviz_with_config(model, config)?;
    let html = wrap_ultimate_html("Custom Diagram", &svg);
    Ok(html)
}

pub struct ArcVizConfig {
    pub title: String,
    pub component_width: u32,
    pub component_height: u32,
    pub arrow_width: f32,
    pub arrow_color: String,
}

impl Default for ArcVizConfig {
    fn default() -> Self {
        Self {
            title: "Architecture".to_string(),
            component_width: 200,
            component_height: 150,
            arrow_width: 1.5,
            arrow_color: "#666".to_string(),
        }
    }
}
```

---

## Custom Tools

### Example: Traceability Analyzer

```rust
use arclang::compiler::semantic::SemanticModel;
use std::collections::HashMap;

pub struct TraceabilityAnalyzer {
    model: SemanticModel,
}

impl TraceabilityAnalyzer {
    pub fn new(model: SemanticModel) -> Self {
        Self { model }
    }
    
    pub fn calculate_coverage(&self) -> f32 {
        let total_requirements = self.model.requirements.len();
        if total_requirements == 0 {
            return 100.0;
        }
        
        let traced_requirements = self.model.requirements.iter()
            .filter(|req| self.has_trace(&req.id))
            .count();
        
        (traced_requirements as f32 / total_requirements as f32) * 100.0
    }
    
    pub fn find_orphan_requirements(&self) -> Vec<String> {
        self.model.requirements.iter()
            .filter(|req| !self.has_trace(&req.id))
            .map(|req| req.id.clone())
            .collect()
    }
    
    pub fn generate_traceability_matrix(&self) -> HashMap<String, Vec<String>> {
        let mut matrix = HashMap::new();
        
        for trace in &self.model.traces {
            matrix.entry(trace.target.clone())
                .or_insert_with(Vec::new)
                .push(trace.source.clone());
        }
        
        matrix
    }
    
    fn has_trace(&self, id: &str) -> bool {
        self.model.traces.iter().any(|t| t.target == id)
    }
}

fn main() {
    let source = std::fs::read_to_string("model.arc").unwrap();
    let result = arclang::compiler::compile(&source, Default::default()).unwrap();
    
    let analyzer = TraceabilityAnalyzer::new(result.semantic_model);
    
    println!("Coverage: {:.1}%", analyzer.calculate_coverage());
    
    let orphans = analyzer.find_orphan_requirements();
    if !orphans.is_empty() {
        println!("\nOrphan Requirements:");
        for id in orphans {
            println!("  - {}", id);
        }
    }
    
    let matrix = analyzer.generate_traceability_matrix();
    println!("\nTraceability Matrix:");
    for (target, sources) in matrix {
        println!("  {} ← {:?}", target, sources);
    }
}
```

### Example: Safety Report Generator

```rust
use arclang::compiler::semantic::{SemanticModel, Requirement};

pub struct SafetyReportGenerator {
    model: SemanticModel,
}

impl SafetyReportGenerator {
    pub fn new(model: SemanticModel) -> Self {
        Self { model }
    }
    
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Safety Analysis Report\n\n");
        
        report.push_str(&self.asil_summary());
        report.push_str(&self.hazard_analysis());
        report.push_str(&self.fmea_summary());
        
        report
    }
    
    fn asil_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("## ASIL Summary\n\n");
        
        let mut asil_counts = std::collections::HashMap::new();
        
        for req in &self.model.requirements {
            if let Some(asil) = &req.safety_level {
                *asil_counts.entry(asil.clone()).or_insert(0) += 1;
            }
        }
        
        for (asil, count) in asil_counts {
            summary.push_str(&format!("- {}: {} requirements\n", asil, count));
        }
        
        summary.push_str("\n");
        summary
    }
    
    fn hazard_analysis(&self) -> String {
        let mut analysis = String::new();
        analysis.push_str("## Hazard Analysis\n\n");
        
        for hazard in &self.model.hazards {
            analysis.push_str(&format!("### {}\n", hazard.id));
            analysis.push_str(&format!("{}\n\n", hazard.description));
            analysis.push_str(&format!("- ASIL: {:?}\n", hazard.asil));
            analysis.push_str(&format!("- Mitigation: {:?}\n\n", hazard.mitigation));
        }
        
        analysis
    }
    
    fn fmea_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("## FMEA Summary\n\n");
        
        for fmea in &self.model.fmea {
            summary.push_str(&format!("- {}: RPN = {}\n", fmea.id, fmea.rpn));
        }
        
        summary.push_str("\n");
        summary
    }
}
```

---

## Extension Points

### Custom Trace Types

```rust
pub enum TraceType {
    Satisfies,
    Implements,
    Deploys,
    DerivesFrom,
    Refines,
    Mitigates,
    Verifies,
    Custom(String),
}

pub fn register_custom_trace_type(name: &str) {
    CUSTOM_TRACE_TYPES.lock().unwrap().insert(name.to_string());
}
```

### Custom Validators

```rust
pub trait Validator {
    fn validate(&self, model: &SemanticModel) -> Vec<ValidationError>;
}

pub struct NamingConventionValidator;

impl Validator for NamingConventionValidator {
    fn validate(&self, model: &SemanticModel) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        
        for req in &model.requirements {
            if !req.id.starts_with("REQ-") {
                errors.push(ValidationError {
                    element: req.id.clone(),
                    message: "Requirement ID must start with 'REQ-'".to_string(),
                });
            }
        }
        
        errors
    }
}
```

### Plugin System

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self);
    fn process(&self, model: &SemanticModel) -> Result<(), String>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }
    
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    pub fn initialize_all(&mut self) {
        for plugin in &mut self.plugins {
            plugin.initialize();
        }
    }
    
    pub fn process_all(&self, model: &SemanticModel) -> Result<(), String> {
        for plugin in &self.plugins {
            plugin.process(model)?;
        }
        Ok(())
    }
}
```

---

## Data Structures

### SemanticModel

```rust
pub struct SemanticModel {
    pub requirements: Vec<Requirement>,
    pub components: Vec<Component>,
    pub functions: Vec<Function>,
    pub traces: Vec<Trace>,
    pub hazards: Vec<Hazard>,
    pub fmea: Vec<Fmea>,
    pub nodes: Vec<Node>,
}
```

### Requirement

```rust
pub struct Requirement {
    pub id: String,
    pub description: String,
    pub priority: Option<String>,
    pub requirement_type: Option<String>,
    pub verification_method: Option<String>,
    pub safety_level: Option<String>,
    pub dal: Option<String>,
    pub sil: Option<String>,
    pub derived_from: Vec<String>,
}
```

### Component

```rust
pub struct Component {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub description: Option<String>,
    pub safety_level: Option<String>,
    pub functions: Vec<Function>,
    pub sub_components: Vec<Component>,
}
```

### Trace

```rust
pub struct Trace {
    pub source: String,
    pub target: String,
    pub trace_type: String,
    pub rationale: String,
    pub coverage: Option<String>,
    pub verification: Option<String>,
}
```

---

## Error Handling

### Compiler Errors

```rust
pub enum CompilerError {
    LexerError(String),
    ParserError(String),
    SemanticError(String),
    CodegenError(String),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompilerError::LexerError(msg) => write!(f, "Lexer error: {}", msg),
            CompilerError::ParserError(msg) => write!(f, "Parser error: {}", msg),
            CompilerError::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            CompilerError::CodegenError(msg) => write!(f, "Codegen error: {}", msg),
        }
    }
}

impl std::error::Error for CompilerError {}
```

---

## Testing API

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compile_simple_model() {
        let source = r#"
            system_analysis "Test" {
                requirement "REQ-001" {
                    description: "Test requirement"
                }
            }
        "#;
        
        let result = compile(source, Default::default());
        assert!(result.is_ok());
        
        let model = result.unwrap().semantic_model;
        assert_eq!(model.requirements.len(), 1);
        assert_eq!(model.requirements[0].id, "REQ-001");
    }
}
```

---

## Best Practices

### Performance

```rust
use std::sync::Arc;

pub fn compile_multiple_models(sources: Vec<String>) -> Vec<Result<SemanticModel, String>> {
    use rayon::prelude::*;
    
    sources.par_iter()
        .map(|source| {
            let result = compile(source, Default::default())?;
            Ok(result.semantic_model)
        })
        .collect()
}
```

### Memory Management

```rust
pub fn process_large_model(source: &str) -> Result<(), String> {
    let result = compile(source, CompilerOptions {
        optimize: true,
        validate: false,
        trace_analysis: false,
    })?;
    
    drop(result);
    
    Ok(())
}
```

---

## Version Information

**Version**: 1.0.0  
**API Stability**: Stable  
**Language**: Rust 1.70+  
**License**: MIT

---

## Next Steps

- [Language Reference](LANGUAGE_REFERENCE.md) - Complete syntax guide
- [Best Practices](BEST_PRACTICES.md) - Production recommendations
- [Examples](../examples/) - Real-world usage examples
