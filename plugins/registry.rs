use super::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    manifests: Arc<RwLock<HashMap<String, PluginManifest>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            manifests: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn register(&mut self, name: String, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write()
            .map_err(|e| PluginError::APIError(format!("Lock error: {}", e)))?;
        
        if plugins.contains_key(&name) {
            return Err(PluginError::APIError(format!("Plugin {} already registered", name)));
        }
        
        plugins.insert(name, plugin);
        
        Ok(())
    }
    
    pub fn register_with_manifest(&mut self, name: String, plugin: Box<dyn Plugin>, manifest: PluginManifest) -> Result<(), PluginError> {
        self.register(name.clone(), plugin)?;
        
        let mut manifests = self.manifests.write()
            .map_err(|e| PluginError::APIError(format!("Lock error: {}", e)))?;
        
        manifests.insert(name, manifest);
        
        Ok(())
    }
    
    pub fn unregister(&mut self, name: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write()
            .map_err(|e| PluginError::APIError(format!("Lock error: {}", e)))?;
        
        if let Some(mut plugin) = plugins.remove(name) {
            plugin.shutdown()?;
        }
        
        let mut manifests = self.manifests.write()
            .map_err(|e| PluginError::APIError(format!("Lock error: {}", e)))?;
        
        manifests.remove(name);
        
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Result<&dyn Plugin, PluginError> {
        let plugins = self.plugins.read()
            .map_err(|e| PluginError::APIError(format!("Lock error: {}", e)))?;
        
        plugins.get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| PluginError::PluginNotFound(name.to_string()))
    }
    
    pub fn has_plugin(&self, name: &str) -> bool {
        if let Ok(plugins) = self.plugins.read() {
            plugins.contains_key(name)
        } else {
            false
        }
    }
    
    pub fn list_plugins(&self) -> Vec<String> {
        if let Ok(plugins) = self.plugins.read() {
            plugins.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn get_by_capability(&self, capability: &PluginCapability) -> Vec<&dyn Plugin> {
        let manifests = match self.manifests.read() {
            Ok(m) => m,
            Err(_) => return Vec::new(),
        };
        
        let plugins = match self.plugins.read() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };
        
        let mut result = Vec::new();
        
        for (name, manifest) in manifests.iter() {
            if manifest.capabilities.contains(capability) {
                if let Some(plugin) = plugins.get(name) {
                    result.push(plugin.as_ref());
                }
            }
        }
        
        result
    }
    
    pub fn get_plugins_for_hook(&self, hook: &HookPoint) -> Vec<&dyn Plugin> {
        let manifests = match self.manifests.read() {
            Ok(m) => m,
            Err(_) => return Vec::new(),
        };
        
        let plugins = match self.plugins.read() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };
        
        let mut result = Vec::new();
        
        for (name, manifest) in manifests.iter() {
            if Self::hook_matches(&manifest.hooks, hook) {
                if let Some(plugin) = plugins.get(name) {
                    result.push(plugin.as_ref());
                }
            }
        }
        
        result
    }
    
    fn hook_matches(hooks: &[HookPoint], target: &HookPoint) -> bool {
        hooks.iter().any(|h| std::mem::discriminant(h) == std::mem::discriminant(target))
    }
    
    pub fn get_info(&self, name: &str) -> Option<PluginInfo> {
        let plugins = self.plugins.read().ok()?;
        let manifests = self.manifests.read().ok()?;
        
        let plugin = plugins.get(name)?;
        let manifest = manifests.get(name)?;
        
        Some(PluginInfo {
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            description: manifest.description.clone(),
            capabilities: manifest.capabilities.clone(),
            status: PluginStatus::Active,
        })
    }
}

impl Clone for PluginRegistry {
    fn clone(&self) -> Self {
        Self {
            plugins: Arc::clone(&self.plugins),
            manifests: Arc::clone(&self.manifests),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestPlugin {
        name: String,
    }
    
    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
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
    
    #[test]
    fn test_register_and_get() {
        let mut registry = PluginRegistry::new();
        
        let plugin = Box::new(TestPlugin {
            name: "test-plugin".to_string(),
        });
        
        registry.register("test-plugin".to_string(), plugin).unwrap();
        
        assert!(registry.has_plugin("test-plugin"));
        assert!(!registry.has_plugin("non-existent"));
    }
    
    #[test]
    fn test_list_plugins() {
        let mut registry = PluginRegistry::new();
        
        let plugin1 = Box::new(TestPlugin {
            name: "plugin1".to_string(),
        });
        let plugin2 = Box::new(TestPlugin {
            name: "plugin2".to_string(),
        });
        
        registry.register("plugin1".to_string(), plugin1).unwrap();
        registry.register("plugin2".to_string(), plugin2).unwrap();
        
        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 2);
        assert!(plugins.contains(&"plugin1".to_string()));
        assert!(plugins.contains(&"plugin2".to_string()));
    }
}
