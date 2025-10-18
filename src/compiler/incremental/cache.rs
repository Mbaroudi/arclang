use super::*;
use std::path::Path;

pub struct CacheManager {
    config: IncrementalConfig,
}

impl CacheManager {
    pub fn new(config: IncrementalConfig) -> Self {
        Self { config }
    }
    
    pub fn update_cache(&self, cache: &mut CompilationCache, units: &[CompiledUnit]) -> Result<(), IncrementalError> {
        for unit in units {
            let entry = CacheEntry {
                file_path: unit.file_path.clone(),
                content_hash: unit.content_hash.clone(),
                timestamp: Utc::now(),
                compiled_at: Utc::now(),
                artifacts: unit.artifacts.clone(),
                dependencies: unit.dependencies.clone(),
                symbols_exported: unit.symbols_exported.clone(),
                symbols_imported: unit.symbols_imported.clone(),
            };
            
            cache.entries.insert(unit.file_path.clone(), entry);
            
            self.update_dependency_graph(&mut cache.dependency_graph, unit);
        }
        
        self.enforce_cache_size_limit(cache)?;
        
        Ok(())
    }
    
    pub fn get_cached_artifact(&self, cache: &CompilationCache, file: &str, artifact_type: ArtifactType) -> Option<Vec<u8>> {
        cache.entries.get(file)
            .and_then(|entry| {
                entry.artifacts.iter()
                    .find(|a| a.artifact_type == artifact_type)
                    .map(|a| a.data.clone())
            })
    }
    
    pub fn invalidate_file(&self, cache: &mut CompilationCache, file: &str) -> Result<(), IncrementalError> {
        cache.entries.remove(file);
        
        let dependents = cache.dependency_graph.get_dependents(file);
        for dependent in dependents {
            cache.entries.remove(&dependent);
        }
        
        Ok(())
    }
    
    pub fn clear_cache(&self, cache: &mut CompilationCache) -> Result<(), IncrementalError> {
        cache.entries.clear();
        cache.dependency_graph.nodes.clear();
        cache.dependency_graph.edges.clear();
        cache.last_full_build = None;
        
        let cache_dir = &self.config.cache_dir;
        if cache_dir.exists() {
            std::fs::remove_dir_all(cache_dir)
                .map_err(|e| IncrementalError::CacheSaveError(e.to_string()))?;
        }
        
        Ok(())
    }
    
    pub fn get_cache_stats(&self, cache: &CompilationCache) -> CacheStats {
        let total_entries = cache.entries.len();
        let total_size_bytes: usize = cache.entries.values()
            .flat_map(|e| &e.artifacts)
            .map(|a| a.size_bytes)
            .sum();
        
        let total_size_mb = total_size_bytes as f64 / (1024.0 * 1024.0);
        
        let mut artifact_counts = HashMap::new();
        for entry in cache.entries.values() {
            for artifact in &entry.artifacts {
                *artifact_counts.entry(artifact.artifact_type.clone()).or_insert(0) += 1;
            }
        }
        
        let oldest_entry = cache.entries.values()
            .map(|e| e.compiled_at)
            .min();
        
        let newest_entry = cache.entries.values()
            .map(|e| e.compiled_at)
            .max();
        
        CacheStats {
            total_entries,
            total_size_mb,
            artifact_counts,
            oldest_entry,
            newest_entry,
            last_full_build: cache.last_full_build,
        }
    }
    
    fn update_dependency_graph(&self, graph: &mut DependencyGraph, unit: &CompiledUnit) {
        let node = DependencyNode {
            file_path: unit.file_path.clone(),
            content_hash: unit.content_hash.clone(),
            node_type: NodeType::SourceFile,
            elements: unit.symbols_exported.clone(),
        };
        
        graph.nodes.insert(unit.file_path.clone(), node);
        
        graph.edges.retain(|e| e.from != unit.file_path);
        
        for dep in &unit.dependencies {
            graph.edges.push(DependencyEdge {
                from: unit.file_path.clone(),
                to: dep.clone(),
                edge_type: EdgeType::Import,
            });
        }
    }
    
    fn enforce_cache_size_limit(&self, cache: &mut CompilationCache) -> Result<(), IncrementalError> {
        let total_size_bytes: usize = cache.entries.values()
            .flat_map(|e| &e.artifacts)
            .map(|a| a.size_bytes)
            .sum();
        
        let total_size_mb = total_size_bytes / (1024 * 1024);
        
        if total_size_mb > self.config.max_cache_size_mb {
            let mut entries_by_age: Vec<_> = cache.entries.values().collect();
            entries_by_age.sort_by_key(|e| e.compiled_at);
            
            let target_size_bytes = (self.config.max_cache_size_mb * 1024 * 1024) as usize;
            let mut current_size = total_size_bytes;
            
            for entry in entries_by_age {
                if current_size <= target_size_bytes {
                    break;
                }
                
                let entry_size: usize = entry.artifacts.iter().map(|a| a.size_bytes).sum();
                cache.entries.remove(&entry.file_path);
                current_size -= entry_size;
            }
        }
        
        Ok(())
    }
    
    pub fn export_cache_report(&self, cache: &CompilationCache, output_path: &Path) -> Result<(), IncrementalError> {
        let stats = self.get_cache_stats(cache);
        
        let report = format!(
            "ArcLang Compilation Cache Report\n\
             =================================\n\n\
             Total Entries: {}\n\
             Total Size: {:.2} MB\n\
             Max Cache Size: {} MB\n\
             Cache Utilization: {:.1}%\n\n\
             Artifact Breakdown:\n\
             {}\n\n\
             Oldest Entry: {}\n\
             Newest Entry: {}\n\
             Last Full Build: {}\n",
            stats.total_entries,
            stats.total_size_mb,
            self.config.max_cache_size_mb,
            (stats.total_size_mb / self.config.max_cache_size_mb as f64) * 100.0,
            self.format_artifact_counts(&stats.artifact_counts),
            stats.oldest_entry.map(|t| t.to_rfc3339()).unwrap_or_else(|| "N/A".to_string()),
            stats.newest_entry.map(|t| t.to_rfc3339()).unwrap_or_else(|| "N/A".to_string()),
            stats.last_full_build.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Never".to_string())
        );
        
        std::fs::write(output_path, report)
            .map_err(|e| IncrementalError::CacheSaveError(e.to_string()))?;
        
        Ok(())
    }
    
    fn format_artifact_counts(&self, counts: &HashMap<ArtifactType, usize>) -> String {
        let mut result = String::new();
        for (artifact_type, count) in counts {
            result.push_str(&format!("  {:?}: {}\n", artifact_type, count));
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_mb: f64,
    pub artifact_counts: HashMap<ArtifactType, usize>,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
    pub last_full_build: Option<DateTime<Utc>>,
}

pub struct CacheValidator {
    config: IncrementalConfig,
}

impl CacheValidator {
    pub fn new(config: IncrementalConfig) -> Self {
        Self { config }
    }
    
    pub fn validate_cache(&self, cache: &CompilationCache) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        
        for (file_path, entry) in &cache.entries {
            if !Path::new(file_path).exists() {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    message: format!("Cached file no longer exists: {}", file_path),
                    file_path: Some(file_path.clone()),
                });
            }
            
            if let Ok(metadata) = std::fs::metadata(file_path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_datetime: DateTime<Utc> = modified.into();
                    
                    if modified_datetime > entry.compiled_at {
                        warnings.push(ValidationIssue {
                            severity: IssueSeverity::Warning,
                            message: format!("File modified after compilation: {}", file_path),
                            file_path: Some(file_path.clone()),
                        });
                    }
                }
            }
            
            for dep in &entry.dependencies {
                if !cache.entries.contains_key(dep) && !Path::new(dep).exists() {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        message: format!("Missing dependency: {} (required by {})", dep, file_path),
                        file_path: Some(file_path.clone()),
                    });
                }
            }
        }
        
        self.validate_dependency_graph(&cache.dependency_graph, &mut issues);
        
        ValidationResult {
            valid: issues.is_empty(),
            issues,
            warnings,
        }
    }
    
    fn validate_dependency_graph(&self, graph: &DependencyGraph, issues: &mut Vec<ValidationIssue>) {
        let cycles = self.detect_cycles(graph);
        
        for cycle in cycles {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: format!("Cyclic dependency detected: {}", cycle.join(" -> ")),
                file_path: None,
            });
        }
    }
    
    fn detect_cycles(&self, graph: &DependencyGraph) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_cycle_detection(node_id, graph, &mut visited, &mut path, &mut cycles);
            }
        }
        
        cycles
    }
    
    fn dfs_cycle_detection(
        &self,
        node_id: &str,
        graph: &DependencyGraph,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if path.contains(&node_id.to_string()) {
            let cycle_start = path.iter().position(|n| n == node_id).unwrap();
            cycles.push(path[cycle_start..].to_vec());
            return;
        }
        
        if visited.contains(node_id) {
            return;
        }
        
        path.push(node_id.to_string());
        
        let neighbors: Vec<String> = graph.edges.iter()
            .filter(|e| e.from == node_id)
            .map(|e| e.to.clone())
            .collect();
        
        for neighbor in neighbors {
            self.dfs_cycle_detection(&neighbor, graph, visited, path, cycles);
        }
        
        path.pop();
        visited.insert(node_id.to_string());
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}
