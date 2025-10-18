use super::*;
use crate::compiler::semantic::SemanticModel;
use std::collections::{HashMap, HashSet};

pub struct FTAGenerator {
    config: FTAConfig,
}

impl FTAGenerator {
    pub fn new(config: FTAConfig) -> Self {
        Self { config }
    }
    
    pub fn generate(&self, model: &SemanticModel, hazard: &Hazard) -> Result<FaultTree, SafetyError> {
        let top_event = FaultEvent {
            id: format!("TE-{}", hazard.id),
            description: hazard.description.clone(),
            event_type: FaultEventType::TopEvent,
            probability: None,
        };
        
        let mut gates = Vec::new();
        let mut basic_events = Vec::new();
        let mut intermediate_events = HashMap::new();
        
        self.decompose_hazard(model, hazard, &top_event, &mut gates, &mut basic_events, &mut intermediate_events)?;
        
        self.calculate_probabilities(&top_event, &gates, &mut basic_events);
        
        let minimal_cut_sets = self.compute_minimal_cut_sets(&top_event, &gates, &basic_events);
        
        let top_event_probability = self.calculate_top_event_probability(&minimal_cut_sets, &basic_events);
        
        Ok(FaultTree {
            id: format!("FTA-{}", hazard.id),
            top_event,
            gates,
            basic_events,
            minimal_cut_sets,
            top_event_probability,
        })
    }
    
    fn decompose_hazard(
        &self,
        model: &SemanticModel,
        hazard: &Hazard,
        parent_event: &FaultEvent,
        gates: &mut Vec<FaultGate>,
        basic_events: &mut Vec<BasicEvent>,
        intermediate_events: &mut HashMap<String, FaultEvent>,
    ) -> Result<(), SafetyError> {
        let safety_functions = self.identify_safety_functions(model, hazard);
        
        if safety_functions.is_empty() {
            return Ok(());
        }
        
        let gate_id = format!("G-{}", gates.len() + 1);
        let intermediate_event_id = format!("IE-{}", intermediate_events.len() + 1);
        
        let gate = FaultGate {
            id: gate_id.clone(),
            gate_type: GateType::OR,
            input_events: Vec::new(),
            output_event: parent_event.id.clone(),
        };
        
        let mut input_events = Vec::new();
        
        for safety_func in safety_functions {
            let func_failure_event = format!("BE-{}", safety_func.id);
            
            let basic_event = BasicEvent {
                id: func_failure_event.clone(),
                description: format!("Failure of {}", safety_func.name),
                failure_rate: self.estimate_failure_rate(&safety_func),
                exposure_time: 1.0,
                probability: 0.0,
                diagnostic_coverage: self.estimate_diagnostic_coverage(&safety_func),
            };
            
            basic_events.push(basic_event);
            input_events.push(func_failure_event);
        }
        
        let mut gate_with_inputs = gate;
        gate_with_inputs.input_events = input_events;
        gates.push(gate_with_inputs);
        
        Ok(())
    }
    
    fn identify_safety_functions(&self, model: &SemanticModel, hazard: &Hazard) -> Vec<SafetyFunction> {
        let mut safety_functions = Vec::new();
        
        for sg_id in &hazard.safety_goals {
            if let Some(requirements) = model.get_requirements_for_safety_goal(sg_id) {
                for req in requirements {
                    if let Some(allocated_functions) = model.get_allocated_functions(&req.id) {
                        for func in allocated_functions {
                            safety_functions.push(SafetyFunction {
                                id: func.id.clone(),
                                name: func.name.clone(),
                                integrity_level: req.safety_level.unwrap_or(IntegrityLevel::ASIL_QM),
                                has_diagnostics: func.has_diagnostics,
                            });
                        }
                    }
                }
            }
        }
        
        safety_functions
    }
    
    fn estimate_failure_rate(&self, safety_func: &SafetyFunction) -> f64 {
        let base_rate = match safety_func.integrity_level {
            IntegrityLevel::ASIL_D | IntegrityLevel::DAL_A | IntegrityLevel::SIL_4 => 1e-9,
            IntegrityLevel::ASIL_C | IntegrityLevel::DAL_B | IntegrityLevel::SIL_3 => 1e-8,
            IntegrityLevel::ASIL_B | IntegrityLevel::DAL_C | IntegrityLevel::SIL_2 => 1e-7,
            IntegrityLevel::ASIL_A | IntegrityLevel::DAL_D | IntegrityLevel::SIL_1 => 1e-6,
            _ => 1e-5,
        };
        
        base_rate
    }
    
    fn estimate_diagnostic_coverage(&self, safety_func: &SafetyFunction) -> f64 {
        if safety_func.has_diagnostics {
            match safety_func.integrity_level {
                IntegrityLevel::ASIL_D => 0.99,
                IntegrityLevel::ASIL_C => 0.97,
                IntegrityLevel::ASIL_B => 0.90,
                _ => 0.60,
            }
        } else {
            0.0
        }
    }
    
    fn calculate_probabilities(&self, _top_event: &FaultEvent, _gates: &[FaultGate], basic_events: &mut Vec<BasicEvent>) {
        for be in basic_events.iter_mut() {
            let lambda = be.failure_rate;
            let t = be.exposure_time;
            let dc = be.diagnostic_coverage;
            
            let residual_failure_rate = lambda * (1.0 - dc);
            
            be.probability = 1.0 - (-residual_failure_rate * t).exp();
        }
    }
    
    fn compute_minimal_cut_sets(&self, _top_event: &FaultEvent, gates: &[FaultGate], basic_events: &[BasicEvent]) -> Vec<CutSet> {
        let mut cut_sets = Vec::new();
        
        if gates.is_empty() {
            for be in basic_events {
                cut_sets.push(CutSet {
                    events: vec![be.id.clone()],
                    probability: be.probability,
                    order: 1,
                });
            }
            return cut_sets;
        }
        
        for gate in gates {
            match gate.gate_type {
                GateType::OR => {
                    for input in &gate.input_events {
                        if let Some(be) = basic_events.iter().find(|b| b.id == *input) {
                            cut_sets.push(CutSet {
                                events: vec![be.id.clone()],
                                probability: be.probability,
                                order: 1,
                            });
                        }
                    }
                }
                GateType::AND => {
                    let event_ids: Vec<String> = gate.input_events.clone();
                    let probability = gate.input_events.iter()
                        .filter_map(|id| basic_events.iter().find(|b| b.id == *id))
                        .map(|be| be.probability)
                        .product();
                    
                    cut_sets.push(CutSet {
                        events: event_ids.clone(),
                        probability,
                        order: event_ids.len() as u32,
                    });
                }
                _ => {}
            }
        }
        
        cut_sets.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());
        
        cut_sets.truncate(100);
        
        cut_sets
    }
    
    fn calculate_top_event_probability(&self, cut_sets: &[CutSet], _basic_events: &[BasicEvent]) -> f64 {
        if cut_sets.is_empty() {
            return 0.0;
        }
        
        let single_event_cut_sets: Vec<&CutSet> = cut_sets.iter()
            .filter(|cs| cs.order == 1)
            .collect();
        
        if !single_event_cut_sets.is_empty() {
            let q_top = 1.0 - single_event_cut_sets.iter()
                .map(|cs| 1.0 - cs.probability)
                .product::<f64>();
            
            return q_top;
        }
        
        cut_sets.iter()
            .take(10)
            .map(|cs| cs.probability)
            .sum::<f64>()
            .min(1.0)
    }
}

struct SafetyFunction {
    id: String,
    name: String,
    integrity_level: IntegrityLevel,
    has_diagnostics: bool,
}

pub fn export_fta_to_graphviz(fault_tree: &FaultTree) -> String {
    let mut dot = String::new();
    
    dot.push_str("digraph FaultTree {\n");
    dot.push_str("    rankdir=TB;\n");
    dot.push_str("    node [shape=box];\n\n");
    
    dot.push_str(&format!("    \"{}\" [label=\"{}\\nP={:.2e}\", shape=ellipse, style=filled, fillcolor=red];\n",
        fault_tree.top_event.id,
        fault_tree.top_event.description,
        fault_tree.top_event_probability
    ));
    
    for gate in &fault_tree.gates {
        let gate_shape = match gate.gate_type {
            GateType::AND => "invtrapezium",
            GateType::OR => "trapezium",
            _ => "diamond",
        };
        
        dot.push_str(&format!("    \"{}\" [label=\"{:?}\", shape={}];\n",
            gate.id,
            gate.gate_type,
            gate_shape
        ));
        
        dot.push_str(&format!("    \"{}\" -> \"{}\";\n", gate.id, gate.output_event));
        
        for input in &gate.input_events {
            dot.push_str(&format!("    \"{}\" -> \"{}\";\n", input, gate.id));
        }
    }
    
    for be in &fault_tree.basic_events {
        dot.push_str(&format!("    \"{}\" [label=\"{}\\nλ={:.2e}\\nP={:.2e}\", shape=circle, style=filled, fillcolor=lightblue];\n",
            be.id,
            be.description,
            be.failure_rate,
            be.probability
        ));
    }
    
    dot.push_str("}\n");
    
    dot
}

pub fn generate_fta_report(fault_tree: &FaultTree) -> String {
    let mut report = String::new();
    
    report.push_str("Fault Tree Analysis Report\n");
    report.push_str("==========================\n\n");
    
    report.push_str(&format!("Top Event: {}\n", fault_tree.top_event.description));
    report.push_str(&format!("Top Event Probability: {:.2e}\n\n", fault_tree.top_event_probability));
    
    report.push_str("Minimal Cut Sets:\n");
    report.push_str("-----------------\n");
    
    for (i, cut_set) in fault_tree.minimal_cut_sets.iter().take(10).enumerate() {
        report.push_str(&format!("{}. Order {}: {} (P={:.2e})\n",
            i + 1,
            cut_set.order,
            cut_set.events.join(" AND "),
            cut_set.probability
        ));
    }
    
    report.push_str("\n");
    
    report.push_str("Basic Events:\n");
    report.push_str("-------------\n");
    
    for be in &fault_tree.basic_events {
        report.push_str(&format!("{}: {} (λ={:.2e}, DC={:.1}%, P={:.2e})\n",
            be.id,
            be.description,
            be.failure_rate,
            be.diagnostic_coverage * 100.0,
            be.probability
        ));
    }
    
    report
}

pub fn compute_importance_measures(fault_tree: &FaultTree) -> HashMap<String, ImportanceMeasures> {
    let mut importance = HashMap::new();
    
    for be in &fault_tree.basic_events {
        let q_top = fault_tree.top_event_probability;
        
        let fussell_vesely = be.probability / q_top;
        
        let birnbaum = fault_tree.minimal_cut_sets.iter()
            .filter(|cs| cs.events.contains(&be.id))
            .map(|cs| cs.probability / be.probability)
            .sum::<f64>();
        
        let risk_achievement_worth = q_top / (q_top - fussell_vesely * q_top);
        
        let risk_reduction_worth = q_top / (q_top - be.probability);
        
        importance.insert(be.id.clone(), ImportanceMeasures {
            fussell_vesely,
            birnbaum,
            risk_achievement_worth,
            risk_reduction_worth,
        });
    }
    
    importance
}

#[derive(Debug, Clone)]
pub struct ImportanceMeasures {
    pub fussell_vesely: f64,
    pub birnbaum: f64,
    pub risk_achievement_worth: f64,
    pub risk_reduction_worth: f64,
}
