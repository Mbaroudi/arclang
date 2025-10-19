# 🔌 Plugin Development Guide

**Complete guide for developing ArcLang compiler plugins**

---

## Overview

The ArcLang plugin system allows extending the compiler with custom functionality including:
- Custom code generators
- Additional validation rules
- Custom diagram formats
- Integration with external tools
- Domain-specific analysis

---

## Plugin Architecture

```
┌──────────────┐
│   Compiler   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│Plugin Manager│
└──────┬───────┘
       │
       ├──────────────┐
       ▼              ▼
┌──────────┐   ┌──────────┐
│ Plugin A │   │ Plugin B │
└──────────┘   └──────────┘
```

---

## Creating a Plugin

### 1. Plugin Trait

```rust
use arclang::compiler::plugin::{CompilerPlugin, PluginError};
use arclang::compiler::{Token, Ast, SemanticModel, GeneratedCode};

pub struct MyPlugin {
    name: String,
    config: PluginConfig,
}

impl CompilerPlugin for MyPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn on_lexer_complete(&self, tokens: &[Token]) -> Result<(), PluginError> {
        println\!("Lexer produced {} tokens", tokens.len());
        Ok(())
    }
    
    fn on_parser_complete(&self, ast: &Ast) -> Result<(), PluginError> {
        println\!("Parser produced {} nodes", ast.nodes.len());
        Ok(())
    }
    
    fn on_semantic_complete(&self, model: &SemanticModel) -> Result<(), PluginError> {
        println\!("Semantic analysis found:");
        println\!("  - {} requirements", model.requirements.len());
        println\!("  - {} components", model.components.len());
        Ok(())
    }
    
    fn on_codegen_complete(&self, code: &GeneratedCode) -> Result<(), PluginError> {
        println\!("Code generation produced {} bytes", code.len());
        Ok(())
    }
}
```

### 2. Plugin Registration

```rust
use arclang::compiler::PluginManager;

fn main() {
    let mut manager = PluginManager::new();
    
    // Register plugin
    manager.register(Box::new(MyPlugin {
        name: "my-plugin".to_string(),
        config: PluginConfig::default(),
    }));
    
    // Use compiler with plugins
    let compiler = Compiler::with_plugins(manager);
}
```

---

## Example Plugins

### Custom Validator Plugin

```rust
pub struct SafetyValidatorPlugin;

impl CompilerPlugin for SafetyValidatorPlugin {
    fn name(&self) -> &str {
        "safety-validator"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn on_semantic_complete(&self, model: &SemanticModel) -> Result<(), PluginError> {
        let mut errors = Vec::new();
        
        // Rule 1: All ASIL-D requirements must have independent verification
        for req in &model.requirements {
            if req.safety_level == Some("ASIL_D".to_string()) {
                if \!has_independent_verification(model, &req.id) {
                    errors.push(format\!(
                        "ASIL-D requirement {} missing independent verification",
                        req.id
                    ));
                }
            }
        }
        
        // Rule 2: Safety components must have monitoring
        for comp in &model.components {
            if comp.safety_level.is_some() {
                if \!has_safety_monitor(model, &comp.id) {
                    errors.push(format\!(
                        "Safety component {} missing monitor",
                        comp.id
                    ));
                }
            }
        }
        
        if \!errors.is_empty() {
            return Err(PluginError::ValidationFailed(errors));
        }
        
        Ok(())
    }
}

fn has_independent_verification(model: &SemanticModel, req_id: &str) -> bool {
    model.traces.iter().any(|t| 
        t.target == req_id && 
        t.trace_type == "verifies" && 
        t.attributes.get("independence") == Some(&"true".to_string())
    )
}
```

### Custom Code Generator Plugin

```rust
pub struct PythonGeneratorPlugin;

impl CompilerPlugin for PythonGeneratorPlugin {
    fn name(&self) -> &str {
        "python-generator"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn on_codegen_complete(&self, code: &GeneratedCode) -> Result<(), PluginError> {
        // Parse semantic model from generated code
        let model = extract_semantic_model(code)?;
        
        // Generate Python classes
        let python_code = generate_python(&model)?;
        
        // Write to file
        std::fs::write("generated/model.py", python_code)?;
        
        Ok(())
    }
}

fn generate_python(model: &SemanticModel) -> Result<String, PluginError> {
    let mut code = String::from("# Auto-generated from ArcLang\n\n");
    
    for component in &model.components {
        code.push_str(&format\!("class {}:\n", component.name));
        code.push_str("    \"\"\"{}\"\"\"\ n\n", component.description.as_deref().unwrap_or(""));
        
        for function in &component.functions {
            code.push_str(&format\!("    def {}(self):\n", function.name.to_lowercase().replace(" ", "_")));
            code.push_str("        pass\n\n");
        }
    }
    
    Ok(code)
}
```

### Metrics Collection Plugin

```rust
pub struct MetricsPlugin {
    metrics: Arc<Mutex<Metrics>>,
}

impl CompilerPlugin for MetricsPlugin {
    fn name(&self) -> &str {
        "metrics-collector"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn on_semantic_complete(&self, model: &SemanticModel) -> Result<(), PluginError> {
        let mut metrics = self.metrics.lock().unwrap();
        
        metrics.total_requirements = model.requirements.len();
        metrics.total_components = model.components.len();
        metrics.total_traces = model.traces.len();
        
        // Calculate complexity
        metrics.complexity = calculate_complexity(model);
        
        // Calculate coverage
        metrics.traceability_coverage = calculate_coverage(model);
        
        // Safety metrics
        metrics.asil_d_count = model.requirements.iter()
            .filter(|r| r.safety_level == Some("ASIL_D".to_string()))
            .count();
        
        Ok(())
    }
    
    fn on_codegen_complete(&self, _code: &GeneratedCode) -> Result<(), PluginError> {
        let metrics = self.metrics.lock().unwrap();
        
        // Export metrics
        let json = serde_json::to_string_pretty(&*metrics)?;
        std::fs::write("metrics.json", json)?;
        
        println\!("Metrics exported to metrics.json");
        
        Ok(())
    }
}
```

---

## Plugin Configuration

### Config File

```toml
# .arclang-plugins.toml

[[plugins]]
name = "safety-validator"
enabled = true

[plugins.safety-validator.config]
strict_asil_d = true
require_monitoring = true

[[plugins]]
name = "python-generator"
enabled = true

[plugins.python-generator.config]
output_dir = "generated"
include_docstrings = true
```

### Loading Config

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PluginConfig {
    plugins: Vec<PluginEntry>,
}

#[derive(Deserialize)]
struct PluginEntry {
    name: String,
    enabled: bool,
    config: toml::Value,
}

fn load_plugins() -> Result<PluginManager, Box<dyn std::error::Error>> {
    let config_str = std::fs::read_to_string(".arclang-plugins.toml")?;
    let config: PluginConfig = toml::from_str(&config_str)?;
    
    let mut manager = PluginManager::new();
    
    for entry in config.plugins {
        if entry.enabled {
            let plugin = create_plugin(&entry.name, entry.config)?;
            manager.register(plugin);
        }
    }
    
    Ok(manager)
}
```

---

## Testing Plugins

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safety_validator() {
        let plugin = SafetyValidatorPlugin;
        
        let model = create_test_model_with_asil_d();
        
        let result = plugin.on_semantic_complete(&model);
        
        assert\!(result.is_err());
        if let Err(PluginError::ValidationFailed(errors)) = result {
            assert_eq\!(errors.len(), 1);
            assert\!(errors[0].contains("missing independent verification"));
        }
    }
    
    #[test]
    fn test_python_generator() {
        let plugin = PythonGeneratorPlugin;
        
        let code = create_test_generated_code();
        
        let result = plugin.on_codegen_complete(&code);
        
        assert\!(result.is_ok());
        assert\!(std::path::Path::new("generated/model.py").exists());
    }
}
```

---

## Best Practices

### 1. Error Handling

```rust
// ✅ Good: Detailed error messages
if validation_fails {
    return Err(PluginError::ValidationFailed(vec\![
        format\!("Component {} violates rule XYZ at line {}", comp.id, line)
    ]));
}

// ❌ Bad: Generic errors
return Err(PluginError::Generic("Validation failed".to_string()));
```

### 2. Performance

```rust
// ✅ Good: Parallel processing
use rayon::prelude::*;

components.par_iter()
    .map(|comp| validate_component(comp))
    .collect::<Result<Vec<_>, _>>()?;

// ❌ Bad: Sequential processing
for comp in components {
    validate_component(comp)?;
}
```

### 3. Logging

```rust
use log::{info, warn, error};

impl CompilerPlugin for MyPlugin {
    fn on_semantic_complete(&self, model: &SemanticModel) -> Result<(), PluginError> {
        info\!("Processing {} components", model.components.len());
        
        for comp in &model.components {
            if comp.safety_level.is_none() {
                warn\!("Component {} has no safety level", comp.id);
            }
        }
        
        Ok(())
    }
}
```

---

## API Reference

```rust
pub trait CompilerPlugin: Send + Sync {
    // Required methods
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    // Optional lifecycle hooks
    fn on_lexer_complete(&self, _tokens: &[Token]) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_parser_complete(&self, _ast: &Ast) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_semantic_complete(&self, _model: &SemanticModel) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_codegen_complete(&self, _code: &GeneratedCode) -> Result<(), PluginError> {
        Ok(())
    }
}
```

---

**Status**: Production Ready ✅  
**Version**: 1.0.0  
**Authors**: Malek Baroudi & Bilel Laasami
