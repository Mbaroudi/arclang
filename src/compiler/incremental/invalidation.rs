use super::*;
use std::collections::HashSet;

pub struct InvalidationEngine {
    strategy: InvalidationStrategy,
}

#[derive(Debug, Clone)]
pub enum InvalidationStrategy {
    Conservative,
    Aggressive,
    Selective,
}

impl InvalidationEngine {
    pub fn new(strategy: InvalidationStrategy) -> Self {
        Self { strategy }
    }
    
    pub fn compute_invalidation_set(
        &self,
        cache: &CompilationCache,
        changed_files: &[String],
    ) -> Result<HashSet<String>, IncrementalError> {
        match self.strategy {
            InvalidationStrategy::Conservative => self.conservative_invalidation(cache, changed_files),
            InvalidationStrategy::Aggressive => self.aggressive_invalidation(cache, changed_files),
            InvalidationStrategy::Selective => self.selective_invalidation(cache, changed_files),
        }
    }
    
    fn conservative_invalidation(
        &self,
        cache: &CompilationCache,
        changed_files: &[String],
    ) -> Result<HashSet<String>, IncrementalError> {
        let mut invalidated = HashSet::new();
        
        for file in changed_files {
            invalidated.insert(file.clone());
            
            let dependents = self.get_all_dependents(cache, file);
            invalidated.extend(dependents);
        }
        
        Ok(invalidated)
    }
    
    fn aggressive_invalidation(
        &self,
        cache: &CompilationCache,
        changed_files: &[String],
    ) -> Result<HashSet<String>, IncrementalError> {
        let mut invalidated = HashSet::new();
        
        for file in changed_files {
            if let Some(entry) = cache.entries.get(file) {
                if self.has_interface_change(entry)? {
                    invalidated.insert(file.clone());
                    
                    let dependents = self.get_all_dependents(cache, file);
                    invalidated.extend(dependents);
                } else {
                    invalidated.insert(file.clone());
                }
            } else {
                invalidated.insert(file.clone());
            }
        }
        
        Ok(invalidated)
    }
    
    fn selective_invalidation(
        &self,
        cache: &CompilationCache,
        changed_files: &[String],
    ) -> Result<HashSet<String>, IncrementalError> {
        let mut invalidated = HashSet::new();
        
        for file in changed_files {
            invalidated.insert(file.clone());
            
            if let Some(entry) = cache.entries.get(file) {
                let change_impact = self.analyze_change_impact(entry)?;
                
                match change_impact {
                    ChangeImpact::Interface => {
                        let dependents = self.get_all_dependents(cache, file);
                        invalidated.extend(dependents);
                    }
                    ChangeImpact::Implementation => {
                    }
                    ChangeImpact::Documentation => {
                    }
                }
            }
        }
        
        Ok(invalidated)
    }
    
    fn get_all_dependents(&self, cache: &CompilationCache, file: &str) -> HashSet<String> {
        let mut dependents = HashSet::new();
        let mut queue = vec![file.to_string()];
        let mut visited = HashSet::new();
        
        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            
            for edge in &cache.dependency_graph.edges {
                if edge.to == current {
                    dependents.insert(edge.from.clone());
                    queue.push(edge.from.clone());
                }
            }
        }
        
        dependents
    }
    
    fn has_interface_change(&self, entry: &CacheEntry) -> Result<bool, IncrementalError> {
        Ok(true)
    }
    
    fn analyze_change_impact(&self, entry: &CacheEntry) -> Result<ChangeImpact, IncrementalError> {
        Ok(ChangeImpact::Implementation)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeImpact {
    Interface,
    Implementation,
    Documentation,
}

pub struct ContentHasher;

impl ContentHasher {
    pub fn hash_file(file_path: &str) -> Result<String, IncrementalError> {
        use sha2::{Sha256, Digest};
        
        let content = std::fs::read(file_path)
            .map_err(|e| IncrementalError::FileReadError(file_path.to_string(), e.to_string()))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    pub fn hash_content(content: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }
    
    pub fn hash_ast(ast: &super::AST) -> Result<String, IncrementalError> {
        let serialized = bincode::serialize(ast)
            .map_err(|e| IncrementalError::SerializationError(e.to_string()))?;
        
        Ok(Self::hash_content(&serialized))
    }
}

pub struct IncrementalOptimizer;

impl IncrementalOptimizer {
    pub fn optimize_build_order(files: &[String], cache: &CompilationCache) -> Vec<String> {
        let mut ordered = files.to_vec();
        
        ordered.sort_by_key(|file| {
            cache.entries.get(file)
                .map(|e| e.dependencies.len())
                .unwrap_or(0)
        });
        
        ordered
    }
    
    pub fn identify_parallel_batches(files: &[String], cache: &CompilationCache) -> Vec<Vec<String>> {
        let mut batches = Vec::new();
        let mut remaining: HashSet<String> = files.iter().cloned().collect();
        
        while !remaining.is_empty() {
            let mut current_batch = Vec::new();
            let mut to_remove = Vec::new();
            
            for file in &remaining {
                if let Some(entry) = cache.entries.get(file) {
                    let deps_satisfied = entry.dependencies.iter()
                        .all(|dep| !remaining.contains(dep));
                    
                    if deps_satisfied {
                        current_batch.push(file.clone());
                        to_remove.push(file.clone());
                    }
                } else {
                    current_batch.push(file.clone());
                    to_remove.push(file.clone());
                }
            }
            
            for file in to_remove {
                remaining.remove(&file);
            }
            
            if !current_batch.is_empty() {
                batches.push(current_batch);
            } else {
                break;
            }
        }
        
        batches
    }
    
    pub fn estimate_compilation_time(files: &[String], cache: &CompilationCache) -> std::time::Duration {
        let mut total_ms = 0u64;
        
        for file in files {
            let estimated_ms = if cache.entries.contains_key(file) {
                100
            } else {
                500
            };
            
            total_ms += estimated_ms;
        }
        
        std::time::Duration::from_millis(total_ms)
    }
}

pub fn generate_invalidation_report(
    changed_files: &[String],
    invalidated_files: &[String],
    cache: &CompilationCache,
) -> String {
    let mut report = String::new();
    
    report.push_str("Incremental Compilation Invalidation Report\n");
    report.push_str("============================================\n\n");
    
    report.push_str(&format!("Changed Files: {}\n", changed_files.len()));
    report.push_str(&format!("Invalidated Files: {}\n", invalidated_files.len()));
    report.push_str(&format!("Invalidation Ratio: {:.2}x\n\n", 
        invalidated_files.len() as f64 / changed_files.len().max(1) as f64));
    
    report.push_str("Changed Files:\n");
    for file in changed_files {
        report.push_str(&format!("  - {}\n", file));
    }
    report.push_str("\n");
    
    report.push_str("Invalidated Files (by dependency chain):\n");
    for file in invalidated_files {
        if !changed_files.contains(file) {
            let reason = if let Some(entry) = cache.entries.get(file) {
                let deps_changed: Vec<_> = entry.dependencies.iter()
                    .filter(|d| changed_files.contains(d))
                    .collect();
                
                if !deps_changed.is_empty() {
                    format!("depends on: {}", deps_changed.iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "))
                } else {
                    "transitive dependency".to_string()
                }
            } else {
                "not in cache".to_string()
            };
            
            report.push_str(&format!("  - {} ({})\n", file, reason));
        }
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_hasher() {
        let content = b"test content";
        let hash1 = ContentHasher::hash_content(content);
        let hash2 = ContentHasher::hash_content(content);
        
        assert_eq!(hash1, hash2);
        
        let different_content = b"different content";
        let hash3 = ContentHasher::hash_content(different_content);
        
        assert_ne!(hash1, hash3);
    }
}
