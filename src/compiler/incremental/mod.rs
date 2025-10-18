pub mod cache;
pub mod dependency_graph;
pub mod invalidation;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalConfig {
    pub cache_dir: PathBuf,
    pub max_cache_size_mb: usize,
    pub enable_parallel: bool,
    pub num_threads: usize,
    pub cache_strategy: CacheStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    ContentBased,
    TimestampBased,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationCache {
    pub version: String,
    pub entries: HashMap<String, CacheEntry>,
    pub dependency_graph: DependencyGraph,
    pub last_full_build: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub file_path: String,
    pub content_hash: String,
    pub timestamp: DateTime<Utc>,
    pub compiled_at: DateTime<Utc>,
    pub artifacts: Vec<CacheArtifact>,
    pub dependencies: Vec<String>,
    pub symbols_exported: Vec<String>,
    pub symbols_imported: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheArtifact {
    pub artifact_type: ArtifactType,
    pub content_hash: String,
    pub size_bytes: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArtifactType {
    AST,
    SemanticModel,
    TypeInfo,
    TraceabilityGraph,
    Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub file_path: String,
    pub content_hash: String,
    pub node_type: NodeType,
    pub elements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    SourceFile,
    GeneratedFile,
    ExternalDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    Import,
    Traces,
    Includes,
    Derives,
}

#[derive(Debug, Clone)]
pub struct IncrementalCompiler {
    config: IncrementalConfig,
    cache: CompilationCache,
    cache_manager: cache::CacheManager,
    dep_graph: dependency_graph::DependencyGraphBuilder,
}

impl IncrementalCompiler {
    pub fn new(config: IncrementalConfig) -> Result<Self, IncrementalError> {
        let cache = CompilationCache::load_or_create(&config.cache_dir)?;
        let cache_manager = cache::CacheManager::new(config.clone());
        let dep_graph = dependency_graph::DependencyGraphBuilder::new();
        
        Ok(Self {
            config,
            cache,
            cache_manager,
            dep_graph,
        })
    }
    
    pub fn compile_incremental(&mut self, changed_files: &[String]) -> Result<IncrementalCompileResult, IncrementalError> {
        let start_time = std::time::Instant::now();
        
        let invalidated = self.compute_invalidation_set(changed_files)?;
        
        if invalidated.is_empty() && changed_files.is_empty() {
            return Ok(IncrementalCompileResult {
                success: true,
                compiled_files: Vec::new(),
                cached_files: self.cache.entries.keys().cloned().collect(),
                invalidated_files: Vec::new(),
                compilation_time_ms: start_time.elapsed().as_millis() as u64,
                cache_hit_ratio: 1.0,
            });
        }
        
        let files_to_compile = self.order_by_dependencies(&invalidated)?;
        
        let compiled = if self.config.enable_parallel {
            self.compile_parallel(&files_to_compile)?
        } else {
            self.compile_sequential(&files_to_compile)?
        };
        
        self.cache_manager.update_cache(&mut self.cache, &compiled)?;
        
        self.cache.save(&self.config.cache_dir)?;
        
        let total_files = self.cache.entries.len();
        let cached_count = total_files - compiled.len();
        let cache_hit_ratio = if total_files > 0 {
            cached_count as f64 / total_files as f64
        } else {
            0.0
        };
        
        Ok(IncrementalCompileResult {
            success: true,
            compiled_files: compiled.iter().map(|c| c.file_path.clone()).collect(),
            cached_files: self.cache.entries.keys()
                .filter(|k| !compiled.iter().any(|c| &c.file_path == *k))
                .cloned()
                .collect(),
            invalidated_files: invalidated,
            compilation_time_ms: start_time.elapsed().as_millis() as u64,
            cache_hit_ratio,
        })
    }
    
    fn compute_invalidation_set(&self, changed_files: &[String]) -> Result<Vec<String>, IncrementalError> {
        let mut invalidated = HashSet::new();
        
        for file in changed_files {
            invalidated.insert(file.clone());
            
            if let Some(entry) = self.cache.entries.get(file) {
                let current_hash = self.compute_file_hash(file)?;
                
                if entry.content_hash != current_hash {
                    let dependents = self.cache.dependency_graph.get_dependents(file);
                    invalidated.extend(dependents);
                }
            } else {
                invalidated.insert(file.clone());
            }
        }
        
        Ok(invalidated.into_iter().collect())
    }
    
    fn order_by_dependencies(&self, files: &[String]) -> Result<Vec<String>, IncrementalError> {
        let mut ordered = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        
        for file in files {
            if !visited.contains(file) {
                self.topological_sort(file, &mut visited, &mut visiting, &mut ordered)?;
            }
        }
        
        Ok(ordered)
    }
    
    fn topological_sort(
        &self,
        file: &str,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        ordered: &mut Vec<String>,
    ) -> Result<(), IncrementalError> {
        if visiting.contains(file) {
            return Err(IncrementalError::CyclicDependency(file.to_string()));
        }
        
        if visited.contains(file) {
            return Ok(());
        }
        
        visiting.insert(file.to_string());
        
        if let Some(entry) = self.cache.entries.get(file) {
            for dep in &entry.dependencies {
                self.topological_sort(dep, visited, visiting, ordered)?;
            }
        }
        
        visiting.remove(file);
        visited.insert(file.to_string());
        ordered.push(file.to_string());
        
        Ok(())
    }
    
    fn compile_sequential(&self, files: &[String]) -> Result<Vec<CompiledUnit>, IncrementalError> {
        let mut compiled = Vec::new();
        
        for file in files {
            let unit = self.compile_single_file(file)?;
            compiled.push(unit);
        }
        
        Ok(compiled)
    }
    
    fn compile_parallel(&self, files: &[String]) -> Result<Vec<CompiledUnit>, IncrementalError> {
        use rayon::prelude::*;
        
        let compiled: Result<Vec<_>, _> = files.par_iter()
            .map(|file| self.compile_single_file(file))
            .collect();
        
        compiled
    }
    
    fn compile_single_file(&self, file: &str) -> Result<CompiledUnit, IncrementalError> {
        let content = std::fs::read_to_string(file)
            .map_err(|e| IncrementalError::FileReadError(file.to_string(), e.to_string()))?;
        
        let content_hash = self.compute_content_hash(&content);
        
        let ast = self.parse_file(&content)?;
        let semantic_model = self.analyze_semantics(&ast)?;
        let dependencies = self.extract_dependencies(&ast);
        
        let artifacts = vec![
            CacheArtifact {
                artifact_type: ArtifactType::AST,
                content_hash: content_hash.clone(),
                size_bytes: content.len(),
                data: bincode::serialize(&ast)
                    .map_err(|e| IncrementalError::SerializationError(e.to_string()))?,
            },
            CacheArtifact {
                artifact_type: ArtifactType::SemanticModel,
                content_hash: content_hash.clone(),
                size_bytes: 0,
                data: bincode::serialize(&semantic_model)
                    .map_err(|e| IncrementalError::SerializationError(e.to_string()))?,
            },
        ];
        
        Ok(CompiledUnit {
            file_path: file.to_string(),
            content_hash,
            artifacts,
            dependencies,
            symbols_exported: semantic_model.exported_symbols,
            symbols_imported: semantic_model.imported_symbols,
        })
    }
    
    fn compute_file_hash(&self, file_path: &str) -> Result<String, IncrementalError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| IncrementalError::FileReadError(file_path.to_string(), e.to_string()))?;
        
        Ok(self.compute_content_hash(&content))
    }
    
    fn compute_content_hash(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    fn parse_file(&self, content: &str) -> Result<AST, IncrementalError> {
        Ok(AST { nodes: Vec::new() })
    }
    
    fn analyze_semantics(&self, ast: &AST) -> Result<SemanticInfo, IncrementalError> {
        Ok(SemanticInfo {
            exported_symbols: Vec::new(),
            imported_symbols: Vec::new(),
        })
    }
    
    fn extract_dependencies(&self, ast: &AST) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
struct CompiledUnit {
    file_path: String,
    content_hash: String,
    artifacts: Vec<CacheArtifact>,
    dependencies: Vec<String>,
    symbols_exported: Vec<String>,
    symbols_imported: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AST {
    nodes: Vec<String>,
}

#[derive(Debug, Clone)]
struct SemanticInfo {
    exported_symbols: Vec<String>,
    imported_symbols: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IncrementalCompileResult {
    pub success: bool,
    pub compiled_files: Vec<String>,
    pub cached_files: Vec<String>,
    pub invalidated_files: Vec<String>,
    pub compilation_time_ms: u64,
    pub cache_hit_ratio: f64,
}

impl CompilationCache {
    fn load_or_create(cache_dir: &PathBuf) -> Result<Self, IncrementalError> {
        let cache_file = cache_dir.join("compilation_cache.bin");
        
        if cache_file.exists() {
            let data = std::fs::read(&cache_file)
                .map_err(|e| IncrementalError::CacheLoadError(e.to_string()))?;
            
            bincode::deserialize(&data)
                .map_err(|e| IncrementalError::CacheLoadError(e.to_string()))
        } else {
            Ok(Self {
                version: env!("CARGO_PKG_VERSION").to_string(),
                entries: HashMap::new(),
                dependency_graph: DependencyGraph {
                    nodes: HashMap::new(),
                    edges: Vec::new(),
                },
                last_full_build: None,
            })
        }
    }
    
    fn save(&self, cache_dir: &PathBuf) -> Result<(), IncrementalError> {
        std::fs::create_dir_all(cache_dir)
            .map_err(|e| IncrementalError::CacheSaveError(e.to_string()))?;
        
        let cache_file = cache_dir.join("compilation_cache.bin");
        
        let data = bincode::serialize(self)
            .map_err(|e| IncrementalError::CacheSaveError(e.to_string()))?;
        
        std::fs::write(&cache_file, data)
            .map_err(|e| IncrementalError::CacheSaveError(e.to_string()))?;
        
        Ok(())
    }
}

impl DependencyGraph {
    fn get_dependents(&self, file: &str) -> Vec<String> {
        self.edges.iter()
            .filter(|e| e.to == file)
            .map(|e| e.from.clone())
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IncrementalError {
    #[error("File read error: {0} - {1}")]
    FileReadError(String, String),
    
    #[error("Cache load error: {0}")]
    CacheLoadError(String),
    
    #[error("Cache save error: {0}")]
    CacheSaveError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Cyclic dependency detected: {0}")]
    CyclicDependency(String),
    
    #[error("Invalid cache entry: {0}")]
    InvalidCacheEntry(String),
}
