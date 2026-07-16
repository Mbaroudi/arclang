use super::quality_metrics::*;
use super::constraint_engine::*;
use super::constraint_solver::*;
use super::semantic_enhanced::*;
use super::capella_metamodel::*;
use std::collections::HashMap;

pub struct MultiObjectiveOptimizer {
    evaluator: QualityEvaluator,
    solver: ConstraintSolver,
    config: OptimizerConfig,
}

#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    pub max_iterations: usize,
    pub convergence_threshold: f64,
    pub cooling_rate: f64,
    pub initial_temperature: f64,
    pub adaptive_weights: bool,
    pub multi_pass: bool,
}

impl OptimizerConfig {
    pub fn default() -> Self {
        Self {
            max_iterations: 1000,
            convergence_threshold: 0.001,
            cooling_rate: 0.95,
            initial_temperature: 1.0,
            adaptive_weights: true,
            multi_pass: true,
        }
    }
    
    pub fn fast() -> Self {
        Self {
            max_iterations: 200,
            convergence_threshold: 0.01,
            cooling_rate: 0.90,
            initial_temperature: 0.5,
            adaptive_weights: false,
            multi_pass: false,
        }
    }
    
    pub fn precision() -> Self {
        Self {
            max_iterations: 2000,
            convergence_threshold: 0.0001,
            cooling_rate: 0.98,
            initial_temperature: 2.0,
            adaptive_weights: true,
            multi_pass: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub layout: HashMap<String, ElementBounds>,
    pub metrics: QualityMetrics,
    pub iterations: usize,
    pub phases: Vec<OptimizationPhase>,
    pub convergence_achieved: bool,
    pub improvement_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationPhase {
    pub name: String,
    pub iterations: usize,
    pub initial_score: f64,
    pub final_score: f64,
    pub improvement: f64,
}

impl MultiObjectiveOptimizer {
    pub fn new(metamodel: CapellaMetamodel) -> Self {
        Self {
            evaluator: QualityEvaluator::new(metamodel.clone()),
            solver: ConstraintSolver::new(metamodel),
            config: OptimizerConfig::default(),
        }
    }
    
    pub fn with_config(mut self, config: OptimizerConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn optimize(
        &mut self,
        model: &EnhancedSemanticModel,
        initial_layout: HashMap<String, ElementBounds>,
    ) -> OptimizationResult {
        let mut layout = initial_layout.clone();
        let edges = self.extract_edges(model);
        
        let initial_metrics = self.evaluator.evaluate(model, &layout, &edges);
        let initial_score = initial_metrics.overall_score;
        
        let mut phases = Vec::new();
        
        if self.config.multi_pass {
            layout = self.run_rough_layout_phase(model, layout, &edges, &mut phases);
            
            layout = self.run_refinement_phase(model, layout, &edges, &mut phases);
            
            layout = self.run_fine_tuning_phase(model, layout, &edges, &mut phases);
        } else {
            layout = self.run_single_pass(model, layout, &edges, &mut phases);
        }
        
        let final_metrics = self.evaluator.evaluate(model, &layout, &edges);
        let final_score = final_metrics.overall_score;
        
        let total_iterations: usize = phases.iter().map(|p| p.iterations).sum();
        let convergence_achieved = total_iterations < self.config.max_iterations;
        let improvement_percentage = ((final_score - initial_score) / initial_score.max(0.01)) * 100.0;
        
        OptimizationResult {
            layout,
            metrics: final_metrics,
            iterations: total_iterations,
            phases,
            convergence_achieved,
            improvement_percentage,
        }
    }
    
    fn run_rough_layout_phase(
        &mut self,
        model: &EnhancedSemanticModel,
        layout: HashMap<String, ElementBounds>,
        edges: &[(String, String)],
        phases: &mut Vec<OptimizationPhase>,
    ) -> HashMap<String, ElementBounds> {
        let phase_name = "Rough Layout".to_string();
        let max_iterations = self.config.max_iterations / 3;
        
        let initial_metrics = self.evaluator.evaluate(model, &layout, edges);
        let initial_score = initial_metrics.overall_score;
        
        let mut current_layout = layout;
        let mut temperature = self.config.initial_temperature;
        let mut iteration = 0;
        
        while iteration < max_iterations {
            let candidate = self.generate_rough_candidate(&current_layout, temperature);
            
            let violations = self.solver.engine.validate_hard_constraints(model, &candidate);
            
            if violations.is_empty() {
                let candidate_metrics = self.evaluator.evaluate(model, &candidate, edges);
                let current_metrics = self.evaluator.evaluate(model, &current_layout, edges);
                
                if self.accept_candidate(
                    current_metrics.overall_score,
                    candidate_metrics.overall_score,
                    temperature,
                ) {
                    current_layout = candidate;
                }
            }
            
            temperature *= self.config.cooling_rate;
            iteration += 1;
        }
        
        let final_metrics = self.evaluator.evaluate(model, &current_layout, edges);
        let final_score = final_metrics.overall_score;
        
        phases.push(OptimizationPhase {
            name: phase_name,
            iterations: iteration,
            initial_score,
            final_score,
            improvement: ((final_score - initial_score) / initial_score.max(0.01)) * 100.0,
        });
        
        current_layout
    }
    
    fn run_refinement_phase(
        &mut self,
        model: &EnhancedSemanticModel,
        layout: HashMap<String, ElementBounds>,
        edges: &[(String, String)],
        phases: &mut Vec<OptimizationPhase>,
    ) -> HashMap<String, ElementBounds> {
        let phase_name = "Refinement".to_string();
        let max_iterations = self.config.max_iterations / 3;
        
        let initial_metrics = self.evaluator.evaluate(model, &layout, edges);
        let initial_score = initial_metrics.overall_score;
        
        let mut current_layout = layout;
        let mut iteration = 0;
        
        if self.config.adaptive_weights {
            self.adjust_weights_for_weaknesses(&initial_metrics);
        }
        
        while iteration < max_iterations {
            current_layout = self.optimize_alignment(&current_layout);
            current_layout = self.optimize_spacing(&current_layout);
            current_layout = self.optimize_edge_lengths(model, &current_layout, edges);
            
            iteration += 1;
            
            if iteration % 10 == 0 {
                let current_metrics = self.evaluator.evaluate(model, &current_layout, edges);
                
                if (current_metrics.overall_score - initial_score).abs() < self.config.convergence_threshold {
                    break;
                }
            }
        }
        
        let final_metrics = self.evaluator.evaluate(model, &current_layout, edges);
        let final_score = final_metrics.overall_score;
        
        phases.push(OptimizationPhase {
            name: phase_name,
            iterations: iteration,
            initial_score,
            final_score,
            improvement: ((final_score - initial_score) / initial_score.max(0.01)) * 100.0,
        });
        
        current_layout
    }
    
    fn run_fine_tuning_phase(
        &mut self,
        model: &EnhancedSemanticModel,
        layout: HashMap<String, ElementBounds>,
        edges: &[(String, String)],
        phases: &mut Vec<OptimizationPhase>,
    ) -> HashMap<String, ElementBounds> {
        let phase_name = "Fine Tuning".to_string();
        let max_iterations = self.config.max_iterations / 3;
        
        let initial_metrics = self.evaluator.evaluate(model, &layout, edges);
        let initial_score = initial_metrics.overall_score;
        
        let mut current_layout = layout;
        let mut iteration = 0;
        let mut no_improvement_count = 0;
        
        while iteration < max_iterations && no_improvement_count < 20 {
            let before_score = self.evaluator.evaluate(model, &current_layout, edges).overall_score;
            
            current_layout = self.fine_tune_positions(&current_layout, 2.0);
            
            let after_score = self.evaluator.evaluate(model, &current_layout, edges).overall_score;
            
            if after_score <= before_score + self.config.convergence_threshold {
                no_improvement_count += 1;
            } else {
                no_improvement_count = 0;
            }
            
            iteration += 1;
        }
        
        let final_metrics = self.evaluator.evaluate(model, &current_layout, edges);
        let final_score = final_metrics.overall_score;
        
        phases.push(OptimizationPhase {
            name: phase_name,
            iterations: iteration,
            initial_score,
            final_score,
            improvement: ((final_score - initial_score) / initial_score.max(0.01)) * 100.0,
        });
        
        current_layout
    }
    
    fn run_single_pass(
        &mut self,
        model: &EnhancedSemanticModel,
        layout: HashMap<String, ElementBounds>,
        edges: &[(String, String)],
        phases: &mut Vec<OptimizationPhase>,
    ) -> HashMap<String, ElementBounds> {
        let phase_name = "Single Pass Optimization".to_string();
        
        let initial_metrics = self.evaluator.evaluate(model, &layout, edges);
        let initial_score = initial_metrics.overall_score;
        
        let mut current_layout = layout;
        let mut temperature = self.config.initial_temperature;
        let mut iteration = 0;
        
        while iteration < self.config.max_iterations {
            let candidate = self.generate_candidate(&current_layout, temperature);
            
            let violations = self.solver.engine.validate_hard_constraints(model, &candidate);
            
            if violations.is_empty() {
                let candidate_metrics = self.evaluator.evaluate(model, &candidate, edges);
                let current_metrics = self.evaluator.evaluate(model, &current_layout, edges);
                
                if self.accept_candidate(
                    current_metrics.overall_score,
                    candidate_metrics.overall_score,
                    temperature,
                ) {
                    current_layout = candidate;
                }
            }
            
            temperature *= self.config.cooling_rate;
            iteration += 1;
        }
        
        let final_metrics = self.evaluator.evaluate(model, &current_layout, edges);
        let final_score = final_metrics.overall_score;
        
        phases.push(OptimizationPhase {
            name: phase_name,
            iterations: iteration,
            initial_score,
            final_score,
            improvement: ((final_score - initial_score) / initial_score.max(0.01)) * 100.0,
        });
        
        current_layout
    }
    
    fn generate_rough_candidate(
        &self,
        layout: &HashMap<String, ElementBounds>,
        temperature: f64,
    ) -> HashMap<String, ElementBounds> {
        let mut candidate = layout.clone();
        let max_move = 50.0 * temperature;
        
        let element_ids: Vec<_> = candidate.keys().cloned().collect();
        
        if !element_ids.is_empty() {
            let idx = rand::random::<usize>() % element_ids.len();
            let id = &element_ids[idx];
            
            if let Some(bounds) = candidate.get_mut(id) {
                let dx = (rand::random::<f64>() - 0.5) * 2.0 * max_move;
                let dy = (rand::random::<f64>() - 0.5) * 2.0 * max_move;
                
                bounds.x += dx;
                bounds.y += dy;
                
                bounds.x = bounds.x.max(0.0);
                bounds.y = bounds.y.max(0.0);
            }
        }
        
        candidate
    }
    
    fn generate_candidate(
        &self,
        layout: &HashMap<String, ElementBounds>,
        temperature: f64,
    ) -> HashMap<String, ElementBounds> {
        let mut candidate = layout.clone();
        let max_move = 20.0 * temperature;
        
        let element_ids: Vec<_> = candidate.keys().cloned().collect();
        
        if !element_ids.is_empty() {
            let idx = rand::random::<usize>() % element_ids.len();
            let id = &element_ids[idx];
            
            if let Some(bounds) = candidate.get_mut(id) {
                let dx = (rand::random::<f64>() - 0.5) * 2.0 * max_move;
                let dy = (rand::random::<f64>() - 0.5) * 2.0 * max_move;
                
                bounds.x += dx;
                bounds.y += dy;
                
                bounds.x = bounds.x.max(0.0);
                bounds.y = bounds.y.max(0.0);
            }
        }
        
        candidate
    }
    
    fn accept_candidate(&self, current_score: f64, candidate_score: f64, temperature: f64) -> bool {
        if candidate_score > current_score {
            return true;
        }
        
        let delta = candidate_score - current_score;
        let probability = (delta / temperature).exp();
        
        rand::random::<f64>() < probability
    }
    
    fn adjust_weights_for_weaknesses(&mut self, metrics: &QualityMetrics) {
        let mut weights = self.evaluator.weights.clone();
        
        if metrics.edge_crossing_score < 0.7 {
            weights.edge_crossings *= 1.5;
        }
        
        if metrics.alignment_score < 0.7 {
            weights.alignment *= 1.3;
        }
        
        if metrics.arcadia_compliance_score < 0.8 {
            weights.arcadia_compliance *= 1.4;
        }
        
        if metrics.safety_emphasis_score < 0.9 {
            weights.safety_emphasis *= 1.5;
        }
        
        weights.normalize();
        self.evaluator.weights = weights;
    }
    
    fn optimize_alignment(&self, layout: &HashMap<String, ElementBounds>) -> HashMap<String, ElementBounds> {
        let mut optimized = layout.clone();
        let tolerance = 10.0;
        
        let elements: Vec<_> = layout.iter()
            .map(|(id, bounds)| (id.clone(), bounds.clone()))
            .collect();
        
        for i in 0..elements.len() {
            for j in i+1..elements.len() {
                let (_, bounds1) = &elements[i];
                let (id2, bounds2) = &elements[j];
                
                if (bounds1.y - bounds2.y).abs() < tolerance {
                    if let Some(b2) = optimized.get_mut(id2) {
                        b2.y = bounds1.y;
                    }
                }
                
                if (bounds1.x - bounds2.x).abs() < tolerance {
                    if let Some(b2) = optimized.get_mut(id2) {
                        b2.x = bounds1.x;
                    }
                }
            }
        }
        
        optimized
    }
    
    fn optimize_spacing(&self, layout: &HashMap<String, ElementBounds>) -> HashMap<String, ElementBounds> {
        layout.clone()
    }
    
    fn optimize_edge_lengths(
        &self,
        _model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
        _edges: &[(String, String)],
    ) -> HashMap<String, ElementBounds> {
        layout.clone()
    }
    
    fn fine_tune_positions(
        &self,
        layout: &HashMap<String, ElementBounds>,
        max_adjustment: f64,
    ) -> HashMap<String, ElementBounds> {
        let mut optimized = layout.clone();
        
        for bounds in optimized.values_mut() {
            let dx = (rand::random::<f64>() - 0.5) * 2.0 * max_adjustment;
            let dy = (rand::random::<f64>() - 0.5) * 2.0 * max_adjustment;
            
            bounds.x = (bounds.x + dx).max(0.0);
            bounds.y = (bounds.y + dy).max(0.0);
        }
        
        optimized
    }
    
    fn extract_edges(&self, model: &EnhancedSemanticModel) -> Vec<(String, String)> {
        model.relationships.iter()
            .map(|r| (r.source_id.clone(), r.target_id.clone()))
            .collect()
    }
}

impl Default for MultiObjectiveOptimizer {
    fn default() -> Self {
        Self::new(CapellaMetamodel::new())
    }
}

#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    Fast,
    Balanced,
    Precision,
    Custom(OptimizerConfig),
}

impl OptimizationStrategy {
    pub fn to_config(&self) -> OptimizerConfig {
        match self {
            OptimizationStrategy::Fast => OptimizerConfig::fast(),
            OptimizationStrategy::Balanced => OptimizerConfig::default(),
            OptimizationStrategy::Precision => OptimizerConfig::precision(),
            OptimizationStrategy::Custom(config) => config.clone(),
        }
    }
}
