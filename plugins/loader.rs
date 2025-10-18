use super::*;
use std::path::{Path, PathBuf};
use std::fs;

pub struct PluginLoader {
    plugin_dir: PathBuf,
}

impl PluginLoader {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self { plugin_dir }
    }
    
    pub fn discover_plugins(&self) -> Result<Vec<PluginManifest>, PluginError> {
        if !self.plugin_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut manifests = Vec::new();
        
        for entry in fs::read_dir(&self.plugin_dir)
            .map_err(|e| PluginError::APIError(format!("Failed to read plugin directory: {}", e)))? 
        {
            let entry = entry.map_err(|e| PluginError::APIError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Ok(manifest) = self.load_manifest(&path) {
                    manifests.push(manifest);
                }
            }
        }
        
        Ok(manifests)
    }
    
    pub fn load_manifest(&self, plugin_path: &Path) -> Result<PluginManifest, PluginError> {
        let manifest_path = plugin_path.join("plugin.toml");
        
        if !manifest_path.exists() {
            return Err(PluginError::InvalidManifest("plugin.toml not found".to_string()));
        }
        
        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::InvalidManifest(format!("Failed to read manifest: {}", e)))?;
        
        toml::from_str(&content)
            .map_err(|e| PluginError::InvalidManifest(format!("Failed to parse manifest: {}", e)))
    }
    
    pub fn load_plugin(&self, manifest: &PluginManifest) -> Result<Box<dyn Plugin>, PluginError> {
        match manifest.name.as_str() {
            "traceability-matrix" => Ok(Box::new(builtin_plugins::TraceabilityMatrixPlugin)),
            "architecture-diagram" => Ok(Box::new(builtin_plugins::ArchitectureDiagramPlugin)),
            _ => Err(PluginError::PluginNotFound(manifest.name.clone())),
        }
    }
    
    pub fn validate_plugin(&self, manifest: &PluginManifest) -> Result<(), PluginError> {
        if manifest.name.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin name cannot be empty".to_string()));
        }
        
        if manifest.version.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin version cannot be empty".to_string()));
        }
        
        if manifest.entry_point.is_empty() {
            return Err(PluginError::InvalidManifest("Entry point cannot be empty".to_string()));
        }
        
        Ok(())
    }
    
    pub fn check_compatibility(&self, manifest: &PluginManifest) -> Result<(), PluginError> {
        let current_version = env!("CARGO_PKG_VERSION");
        
        for dep in &manifest.dependencies {
            if dep.name == "arclang" {
                if !self.version_matches(current_version, &dep.version_requirement) {
                    return Err(PluginError::IncompatibleVersion(
                        format!("Plugin requires ArcLang {}, but running {}", 
                            dep.version_requirement, current_version)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    fn version_matches(&self, version: &str, requirement: &str) -> bool {
        true
    }
}

pub struct DynamicPluginLoader {
    loader: PluginLoader,
}

impl DynamicPluginLoader {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            loader: PluginLoader::new(plugin_dir),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn load_dynamic_plugin(&self, plugin_path: &Path) -> Result<Box<dyn Plugin>, PluginError> {
        use libloading::{Library, Symbol};
        
        let lib_path = plugin_path.join("libplugin.so");
        
        if !lib_path.exists() {
            return Err(PluginError::PluginNotFound(
                format!("Library not found at {:?}", lib_path)
            ));
        }
        
        unsafe {
            let lib = Library::new(&lib_path)
                .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;
            
            type PluginCreate = unsafe fn() -> *mut dyn Plugin;
            
            let create_fn: Symbol<PluginCreate> = lib.get(b"create_plugin")
                .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;
            
            let plugin_ptr = create_fn();
            
            Ok(Box::from_raw(plugin_ptr))
        }
    }
    
    #[cfg(target_os = "windows")]
    pub fn load_dynamic_plugin(&self, plugin_path: &Path) -> Result<Box<dyn Plugin>, PluginError> {
        Err(PluginError::APIError("Dynamic plugin loading not yet implemented for Windows".to_string()))
    }
}

pub fn create_plugin_manifest(
    name: &str,
    version: &str,
    description: &str,
    capabilities: Vec<PluginCapability>,
) -> PluginManifest {
    PluginManifest {
        name: name.to_string(),
        version: version.to_string(),
        description: description.to_string(),
        author: "Unknown".to_string(),
        homepage: None,
        license: "MIT".to_string(),
        entry_point: "main".to_string(),
        dependencies: Vec::new(),
        capabilities,
        hooks: Vec::new(),
    }
}

pub fn save_plugin_manifest(manifest: &PluginManifest, path: &Path) -> Result<(), PluginError> {
    let content = toml::to_string_pretty(manifest)
        .map_err(|e| PluginError::ConfigError(format!("Failed to serialize manifest: {}", e)))?;
    
    fs::write(path, content)
        .map_err(|e| PluginError::ConfigError(format!("Failed to write manifest: {}", e)))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_plugin_manifest() {
        let manifest = create_plugin_manifest(
            "test-plugin",
            "1.0.0",
            "A test plugin",
            vec![PluginCapability::Linter],
        );
        
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.capabilities.len(), 1);
    }
}
