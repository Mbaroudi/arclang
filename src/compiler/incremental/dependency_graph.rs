use super::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct DependencyGraphBuilder {
    graph: DependencyGraph,
}

impl DependencyGraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph {
                nodes: HashMap::new(),
                edges: Vec::new(),
            },
        }
    }
    
    pub fn add_node(&mut self, node: DependencyNode) {
        self.graph.nodes.insert(node.file_path.clone(), node);
    }
    
    pub fn add_edge(&mut self, edge: DependencyEdge) {
        self.graph.edges.push(edge);
    }
    
    pub fn build(self) -> DependencyGraph {
        self.graph
    }
    
    pub fn get_transitive_dependencies(&self, file: &str) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(file.to_string());
        
        while let Some(current) = queue.pop_front() {
            for edge in &self.graph.edges {
                if edge.from == current && !dependencies.contains(&edge.to) {
                    dependencies.insert(edge.to.clone());
                    queue.push_back(edge.to.clone());
                }
            }
        }
        
        dependencies
    }
    
    pub fn get_transitive_dependents(&self, file: &str) -> HashSet<String> {
        let mut dependents = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(file.to_string());
        
        while let Some(current) = queue.pop_front() {
            for edge in &self.graph.edges {
                if edge.to == current && !dependents.contains(&edge.from) {
                    dependents.insert(edge.from.clone());
                    queue.push_back(edge.from.clone());
                }
            }
        }
        
        dependents
    }
    
    pub fn compute_compilation_order(&self) -> Result<Vec<String>, String> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        
        for node_id in self.graph.nodes.keys() {
            if !visited.contains(node_id) {
                self.topological_sort_dfs(node_id, &mut visited, &mut visiting, &mut order)?;
            }
        }
        
        Ok(order)
    }
    
    fn topological_sort_dfs(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if visiting.contains(node) {
            return Err(format!("Cyclic dependency detected at {}", node));
        }
        
        if visited.contains(node) {
            return Ok(());
        }
        
        visiting.insert(node.to_string());
        
        for edge in &self.graph.edges {
            if edge.from == node {
                self.topological_sort_dfs(&edge.to, visited, visiting, order)?;
            }
        }
        
        visiting.remove(node);
        visited.insert(node.to_string());
        order.push(node.to_string());
        
        Ok(())
    }
    
    pub fn find_strongly_connected_components(&self) -> Vec<Vec<String>> {
        let mut index = 0;
        let mut stack = Vec::new();
        let mut indices = HashMap::new();
        let mut lowlinks = HashMap::new();
        let mut on_stack = HashSet::new();
        let mut components = Vec::new();
        
        for node_id in self.graph.nodes.keys() {
            if !indices.contains_key(node_id) {
                self.tarjan_scc(
                    node_id,
                    &mut index,
                    &mut stack,
                    &mut indices,
                    &mut lowlinks,
                    &mut on_stack,
                    &mut components,
                );
            }
        }
        
        components
    }
    
    fn tarjan_scc(
        &self,
        node: &str,
        index: &mut usize,
        stack: &mut Vec<String>,
        indices: &mut HashMap<String, usize>,
        lowlinks: &mut HashMap<String, usize>,
        on_stack: &mut HashSet<String>,
        components: &mut Vec<Vec<String>>,
    ) {
        indices.insert(node.to_string(), *index);
        lowlinks.insert(node.to_string(), *index);
        *index += 1;
        stack.push(node.to_string());
        on_stack.insert(node.to_string());
        
        for edge in &self.graph.edges {
            if edge.from == node {
                let successor = &edge.to;
                
                if !indices.contains_key(successor) {
                    self.tarjan_scc(successor, index, stack, indices, lowlinks, on_stack, components);
                    
                    let successor_lowlink = *lowlinks.get(successor).unwrap();
                    let node_lowlink = *lowlinks.get(node).unwrap();
                    lowlinks.insert(node.to_string(), node_lowlink.min(successor_lowlink));
                } else if on_stack.contains(successor) {
                    let successor_index = *indices.get(successor).unwrap();
                    let node_lowlink = *lowlinks.get(node).unwrap();
                    lowlinks.insert(node.to_string(), node_lowlink.min(successor_index));
                }
            }
        }
        
        if lowlinks.get(node) == indices.get(node) {
            let mut component = Vec::new();
            
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                component.push(w.clone());
                
                if w == node {
                    break;
                }
            }
            
            if component.len() > 1 || self.has_self_loop(node) {
                components.push(component);
            }
        }
    }
    
    fn has_self_loop(&self, node: &str) -> bool {
        self.graph.edges.iter().any(|e| e.from == node && e.to == node)
    }
}

pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    pub fn analyze_impact(graph: &DependencyGraph, changed_files: &[String]) -> ImpactAnalysisResult {
        let builder = DependencyGraphBuilder { graph: graph.clone() };
        
        let mut directly_affected = HashSet::new();
        let mut transitively_affected = HashSet::new();
        
        for file in changed_files {
            directly_affected.insert(file.clone());
            
            let dependents = builder.get_transitive_dependents(file);
            transitively_affected.extend(dependents);
        }
        
        transitively_affected.retain(|f| !directly_affected.contains(f));
        
        let total_files = graph.nodes.len();
        let affected_count = directly_affected.len() + transitively_affected.len();
        let impact_percentage = if total_files > 0 {
            (affected_count as f64 / total_files as f64) * 100.0
        } else {
            0.0
        };
        
        ImpactAnalysisResult {
            directly_affected: directly_affected.into_iter().collect(),
            transitively_affected: transitively_affected.into_iter().collect(),
            total_affected: affected_count,
            impact_percentage,
        }
    }
    
    pub fn find_critical_files(graph: &DependencyGraph) -> Vec<CriticalFile> {
        let mut critical_files = Vec::new();
        
        for (file_path, _node) in &graph.nodes {
            let dependent_count = graph.edges.iter()
                .filter(|e| e.to == *file_path)
                .count();
            
            if dependent_count > 0 {
                critical_files.push(CriticalFile {
                    file_path: file_path.clone(),
                    dependent_count,
                    criticality_score: Self::calculate_criticality_score(graph, file_path),
                });
            }
        }
        
        critical_files.sort_by(|a, b| b.criticality_score.partial_cmp(&a.criticality_score).unwrap());
        
        critical_files
    }
    
    fn calculate_criticality_score(graph: &DependencyGraph, file: &str) -> f64 {
        let builder = DependencyGraphBuilder { graph: graph.clone() };
        
        let direct_dependents = graph.edges.iter()
            .filter(|e| e.to == file)
            .count();
        
        let transitive_dependents = builder.get_transitive_dependents(file).len();
        
        (direct_dependents as f64 * 2.0) + (transitive_dependents as f64)
    }
    
    pub fn export_to_dot(graph: &DependencyGraph) -> String {
        let mut dot = String::new();
        
        dot.push_str("digraph Dependencies {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box];\n\n");
        
        for (file_path, node) in &graph.nodes {
            let color = match node.node_type {
                NodeType::SourceFile => "lightblue",
                NodeType::GeneratedFile => "lightgreen",
                NodeType::ExternalDependency => "lightgray",
            };
            
            let label = file_path.split('/').last().unwrap_or(file_path);
            dot.push_str(&format!("    \"{}\" [label=\"{}\", style=filled, fillcolor={}];\n",
                file_path, label, color));
        }
        
        dot.push_str("\n");
        
        for edge in &graph.edges {
            let color = match edge.edge_type {
                EdgeType::Import => "black",
                EdgeType::Traces => "blue",
                EdgeType::Includes => "green",
                EdgeType::Derives => "red",
            };
            
            dot.push_str(&format!("    \"{}\" -> \"{}\" [color={}];\n",
                edge.from, edge.to, color));
        }
        
        dot.push_str("}\n");
        
        dot
    }
}

#[derive(Debug, Clone)]
pub struct ImpactAnalysisResult {
    pub directly_affected: Vec<String>,
    pub transitively_affected: Vec<String>,
    pub total_affected: usize,
    pub impact_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct CriticalFile {
    pub file_path: String,
    pub dependent_count: usize,
    pub criticality_score: f64,
}
