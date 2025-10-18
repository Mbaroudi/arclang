pub mod registry;
pub mod loader;
pub mod api;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_dir: PathBuf,
    pub enabled_plugins: Vec<String>,
    pub plugin_settings: HashMap<String, serde_json::Value>,
    pub auto_load: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub license: String,
    pub entry_point: String,
    pub dependencies: Vec<PluginDependency>,
    pub capabilities: Vec<PluginCapability>,
    pub hooks: Vec<HookPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginCapability {
    CompilerPass,
    CodeGenerator,
    Linter,
    Formatter,
    LanguageServer,
    ExportFormat,
    ImportFormat,
    CustomAnalysis,
    Visualization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookPoint {
    PreCompile,
    PostCompile,
    PrePass(String),
    PostPass(String),
    OnError,
    OnWarning,
    BeforeCodeGen,
    AfterCodeGen,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), PluginError>;
    fn execute(&self, context: &PluginContext) -> Result<PluginResult, PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
}

pub trait CompilerPassPlugin: Plugin {
    fn pass_name(&self) -> &str;
    fn pass_priority(&self) -> i32;
    fn run_pass(&self, model: &mut api::SemanticModel) -> Result<(), PluginError>;
}

pub trait CodeGeneratorPlugin: Plugin {
    fn target_language(&self) -> &str;
    fn generate_code(&self, model: &api::SemanticModel) -> Result<GeneratedCode, PluginError>;
}

pub trait LinterPlugin: Plugin {
    fn lint(&self, model: &api::SemanticModel) -> Result<Vec<LintDiagnostic>, PluginError>;
}

pub trait FormatterPlugin: Plugin {
    fn format_source(&self, source: &str) -> Result<String, PluginError>;
}

pub trait VisualizationPlugin: Plugin {
    fn visualization_type(&self) -> VisualizationType;
    fn generate_visualization(&self, model: &api::SemanticModel) -> Result<Visualization, PluginError>;
}

#[derive(Debug, Clone)]
pub struct PluginContext {
    pub config: HashMap<String, serde_json::Value>,
    pub workspace_path: PathBuf,
    pub cache_dir: PathBuf,
    pub logger: PluginLogger,
}

impl PluginContext {
    pub fn new(workspace_path: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            config: HashMap::new(),
            workspace_path,
            cache_dir,
            logger: PluginLogger::new(),
        }
    }
    
    pub fn get_config<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.config.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    pub fn set_config(&mut self, key: String, value: serde_json::Value) {
        self.config.insert(key, value);
    }
}

#[derive(Debug, Clone)]
pub struct PluginLogger {
    plugin_name: Option<String>,
}

impl PluginLogger {
    pub fn new() -> Self {
        Self { plugin_name: None }
    }
    
    pub fn set_plugin_name(&mut self, name: String) {
        self.plugin_name = Some(name);
    }
    
    pub fn info(&self, message: &str) {
        let prefix = self.plugin_name.as_ref()
            .map(|n| format!("[{}] ", n))
            .unwrap_or_default();
        println!("INFO: {}{}", prefix, message);
    }
    
    pub fn warn(&self, message: &str) {
        let prefix = self.plugin_name.as_ref()
            .map(|n| format!("[{}] ", n))
            .unwrap_or_default();
        eprintln!("WARN: {}{}", prefix, message);
    }
    
    pub fn error(&self, message: &str) {
        let prefix = self.plugin_name.as_ref()
            .map(|n| format!("[{}] ", n))
            .unwrap_or_default();
        eprintln!("ERROR: {}{}", prefix, message);
    }
}

#[derive(Debug, Clone)]
pub struct PluginResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub diagnostics: Vec<PluginDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub language: String,
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub rule_id: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub location: SourceLocation,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    Graph,
    Diagram,
    Matrix,
    Chart,
}

#[derive(Debug, Clone)]
pub struct Visualization {
    pub viz_type: VisualizationType,
    pub format: VisualizationFormat,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum VisualizationFormat {
    SVG,
    PNG,
    JSON,
    HTML,
    Graphviz,
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
    
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),
    
    #[error("Incompatible plugin version: {0}")]
    IncompatibleVersion(String),
    
    #[error("Missing dependency: {0}")]
    MissingDependency(String),
    
    #[error("Plugin API error: {0}")]
    APIError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub struct PluginManager {
    registry: registry::PluginRegistry,
    loader: loader::PluginLoader,
    config: PluginConfig,
}

impl PluginManager {
    pub fn new(config: PluginConfig) -> Result<Self, PluginError> {
        let registry = registry::PluginRegistry::new();
        let loader = loader::PluginLoader::new(config.plugin_dir.clone());
        
        Ok(Self {
            registry,
            loader,
            config,
        })
    }
    
    pub fn load_plugins(&mut self) -> Result<(), PluginError> {
        let manifests = self.loader.discover_plugins()?;
        
        for manifest in manifests {
            if self.config.enabled_plugins.contains(&manifest.name) || self.config.auto_load {
                self.load_plugin(&manifest)?;
            }
        }
        
        Ok(())
    }
    
    pub fn load_plugin(&mut self, manifest: &PluginManifest) -> Result<(), PluginError> {
        self.validate_dependencies(manifest)?;
        
        let plugin = self.loader.load_plugin(manifest)?;
        
        self.registry.register(manifest.name.clone(), plugin)?;
        
        Ok(())
    }
    
    pub fn execute_plugin(&self, plugin_name: &str, context: &PluginContext) -> Result<PluginResult, PluginError> {
        let plugin = self.registry.get(plugin_name)?;
        plugin.execute(context)
    }
    
    pub fn execute_hook(&self, hook: &HookPoint, context: &PluginContext) -> Result<Vec<PluginResult>, PluginError> {
        let plugins = self.registry.get_plugins_for_hook(hook);
        
        let mut results = Vec::new();
        for plugin in plugins {
            let result = plugin.execute(context)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    pub fn get_plugins_by_capability(&self, capability: &PluginCapability) -> Vec<&dyn Plugin> {
        self.registry.get_by_capability(capability)
    }
    
    fn validate_dependencies(&self, manifest: &PluginManifest) -> Result<(), PluginError> {
        for dep in &manifest.dependencies {
            if !self.registry.has_plugin(&dep.name) {
                return Err(PluginError::MissingDependency(dep.name.clone()));
            }
        }
        
        Ok(())
    }
    
    pub fn unload_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError> {
        self.registry.unregister(plugin_name)
    }
    
    pub fn list_plugins(&self) -> Vec<String> {
        self.registry.list_plugins()
    }
    
    pub fn get_plugin_info(&self, plugin_name: &str) -> Option<PluginInfo> {
        self.registry.get_info(plugin_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<PluginCapability>,
    pub status: PluginStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Loaded,
    Active,
    Error(String),
}

pub mod builtin_plugins {
    use super::*;
    
    pub struct TraceabilityMatrixPlugin;
    
    impl Plugin for TraceabilityMatrixPlugin {
        fn name(&self) -> &str {
            "traceability-matrix"
        }
        
        fn version(&self) -> &str {
            "1.0.0"
        }
        
        fn initialize(&mut self, _context: &mut PluginContext) -> Result<(), PluginError> {
            Ok(())
        }
        
        fn execute(&self, context: &PluginContext) -> Result<PluginResult, PluginError> {
            context.logger.info("Generating traceability matrix");
            
            Ok(PluginResult {
                success: true,
                output: Some(serde_json::json!({
                    "matrix_type": "requirements_to_components"
                })),
                diagnostics: Vec::new(),
            })
        }
        
        fn shutdown(&mut self) -> Result<(), PluginError> {
            Ok(())
        }
    }
    
    pub struct ArchitectureDiagramPlugin;
    
    impl Plugin for ArchitectureDiagramPlugin {
        fn name(&self) -> &str {
            "architecture-diagram"
        }
        
        fn version(&self) -> &str {
            "1.0.0"
        }
        
        fn initialize(&mut self, _context: &mut PluginContext) -> Result<(), PluginError> {
            Ok(())
        }
        
        fn execute(&self, _context: &PluginContext) -> Result<PluginResult, PluginError> {
            Ok(PluginResult {
                success: true,
                output: None,
                diagnostics: Vec::new(),
            })
        }
        
        fn shutdown(&mut self) -> Result<(), PluginError> {
            Ok(())
        }
    }
    
    impl VisualizationPlugin for ArchitectureDiagramPlugin {
        fn visualization_type(&self) -> VisualizationType {
            VisualizationType::Diagram
        }
        
        fn generate_visualization(&self, _model: &api::SemanticModel) -> Result<Visualization, PluginError> {
            Ok(Visualization {
                viz_type: VisualizationType::Diagram,
                format: VisualizationFormat::SVG,
                content: "<svg></svg>".to_string(),
            })
        }
    }
}
