use super::*;
use crate::compiler::semantic::SemanticModel;

pub struct HazardAnalyzer {
    config: SafetyConfig,
}

impl HazardAnalyzer {
    pub fn new(config: SafetyConfig) -> Self {
        Self { config }
    }
    
    pub fn perform_hara(&self, model: &SemanticModel) -> Result<Vec<Hazard>, SafetyError> {
        let mut hazards = Vec::new();
        
        for operational_situation in model.operational_situations() {
            let situation_hazards = self.identify_hazards_for_situation(model, &operational_situation)?;
            hazards.extend(situation_hazards);
        }
        
        for hazard in &mut hazards {
            self.assess_risk(hazard)?;
            self.determine_asil(hazard);
            self.identify_mitigation_measures(model, hazard)?;
        }
        
        Ok(hazards)
    }
    
    fn identify_hazards_for_situation(&self, model: &SemanticModel, situation: &OperationalSituation) -> Result<Vec<Hazard>, SafetyError> {
        let mut hazards = Vec::new();
        
        for capability in model.capabilities() {
            if capability.is_safety_related {
                let hazard_scenarios = self.generate_hazard_scenarios(capability, situation);
                
                for scenario in hazard_scenarios {
                    let hazard = Hazard {
                        id: format!("HAZ-{}-{}", capability.id, scenario.id),
                        title: scenario.title,
                        description: scenario.description,
                        hazard_type: scenario.hazard_type,
                        severity: HazardSeverity::S0,
                        exposure: ExposureLevel::E0,
                        controllability: ControllabilityLevel::C0,
                        integrity_level: IntegrityLevel::ASIL_QM,
                        safety_goals: Vec::new(),
                        mitigation_measures: Vec::new(),
                        residual_risk: RiskLevel::Unacceptable,
                    };
                    
                    hazards.push(hazard);
                }
            }
        }
        
        Ok(hazards)
    }
    
    fn generate_hazard_scenarios(&self, capability: &Capability, situation: &OperationalSituation) -> Vec<HazardScenario> {
        let mut scenarios = Vec::new();
        
        scenarios.push(HazardScenario {
            id: "loss".to_string(),
            title: format!("Loss of {}", capability.name),
            description: format!("Complete loss of {} capability during {}", capability.name, situation.name),
            hazard_type: HazardType::Functional,
        });
        
        scenarios.push(HazardScenario {
            id: "degraded".to_string(),
            title: format!("Degraded {}", capability.name),
            description: format!("Partial or degraded {} capability during {}", capability.name, situation.name),
            hazard_type: HazardType::Functional,
        });
        
        scenarios.push(HazardScenario {
            id: "unintended".to_string(),
            title: format!("Unintended activation of {}", capability.name),
            description: format!("Spurious or unintended activation of {} during {}", capability.name, situation.name),
            hazard_type: HazardType::Functional,
        });
        
        scenarios
    }
    
    fn assess_risk(&self, hazard: &mut Hazard) -> Result<(), SafetyError> {
        hazard.severity = self.assess_severity(&hazard.description);
        hazard.exposure = self.assess_exposure(&hazard.description);
        hazard.controllability = self.assess_controllability(&hazard.description);
        
        Ok(())
    }
    
    fn assess_severity(&self, description: &str) -> HazardSeverity {
        if description.contains("fatal") || description.contains("life-threatening") {
            return HazardSeverity::S3;
        }
        
        if description.contains("severe") || description.contains("injury") {
            return HazardSeverity::S2;
        }
        
        if description.contains("minor") || description.contains("light") {
            return HazardSeverity::S1;
        }
        
        HazardSeverity::S0
    }
    
    fn assess_exposure(&self, description: &str) -> ExposureLevel {
        if description.contains("continuous") || description.contains("always") {
            return ExposureLevel::E4;
        }
        
        if description.contains("frequently") || description.contains("often") {
            return ExposureLevel::E3;
        }
        
        if description.contains("occasionally") {
            return ExposureLevel::E2;
        }
        
        if description.contains("rarely") {
            return ExposureLevel::E1;
        }
        
        ExposureLevel::E0
    }
    
    fn assess_controllability(&self, description: &str) -> ControllabilityLevel {
        if description.contains("uncontrollable") || description.contains("cannot avoid") {
            return ControllabilityLevel::C3;
        }
        
        if description.contains("difficult to control") {
            return ControllabilityLevel::C2;
        }
        
        if description.contains("normally controllable") {
            return ControllabilityLevel::C1;
        }
        
        ControllabilityLevel::C0
    }
    
    fn determine_asil(&self, hazard: &mut Hazard) {
        hazard.integrity_level = match (&hazard.severity, &hazard.exposure, &hazard.controllability) {
            (HazardSeverity::S3, ExposureLevel::E4, ControllabilityLevel::C3) => IntegrityLevel::ASIL_D,
            (HazardSeverity::S3, ExposureLevel::E4, ControllabilityLevel::C2) => IntegrityLevel::ASIL_D,
            (HazardSeverity::S3, ExposureLevel::E4, ControllabilityLevel::C1) => IntegrityLevel::ASIL_C,
            (HazardSeverity::S3, ExposureLevel::E4, ControllabilityLevel::C0) => IntegrityLevel::ASIL_QM,
            
            (HazardSeverity::S3, ExposureLevel::E3, ControllabilityLevel::C3) => IntegrityLevel::ASIL_D,
            (HazardSeverity::S3, ExposureLevel::E3, ControllabilityLevel::C2) => IntegrityLevel::ASIL_C,
            (HazardSeverity::S3, ExposureLevel::E3, ControllabilityLevel::C1) => IntegrityLevel::ASIL_B,
            (HazardSeverity::S3, ExposureLevel::E3, ControllabilityLevel::C0) => IntegrityLevel::ASIL_QM,
            
            (HazardSeverity::S3, ExposureLevel::E2, ControllabilityLevel::C3) => IntegrityLevel::ASIL_C,
            (HazardSeverity::S3, ExposureLevel::E2, ControllabilityLevel::C2) => IntegrityLevel::ASIL_B,
            (HazardSeverity::S3, ExposureLevel::E2, ControllabilityLevel::C1) => IntegrityLevel::ASIL_A,
            
            (HazardSeverity::S2, ExposureLevel::E4, ControllabilityLevel::C3) => IntegrityLevel::ASIL_D,
            (HazardSeverity::S2, ExposureLevel::E4, ControllabilityLevel::C2) => IntegrityLevel::ASIL_C,
            (HazardSeverity::S2, ExposureLevel::E4, ControllabilityLevel::C1) => IntegrityLevel::ASIL_B,
            
            (HazardSeverity::S2, ExposureLevel::E3, ControllabilityLevel::C3) => IntegrityLevel::ASIL_C,
            (HazardSeverity::S2, ExposureLevel::E3, ControllabilityLevel::C2) => IntegrityLevel::ASIL_B,
            (HazardSeverity::S2, ExposureLevel::E3, ControllabilityLevel::C1) => IntegrityLevel::ASIL_A,
            
            (HazardSeverity::S2, ExposureLevel::E2, ControllabilityLevel::C3) => IntegrityLevel::ASIL_B,
            (HazardSeverity::S2, ExposureLevel::E2, ControllabilityLevel::C2) => IntegrityLevel::ASIL_A,
            
            (HazardSeverity::S1, ExposureLevel::E4, ControllabilityLevel::C3) => IntegrityLevel::ASIL_B,
            (HazardSeverity::S1, ExposureLevel::E4, ControllabilityLevel::C2) => IntegrityLevel::ASIL_A,
            
            (HazardSeverity::S1, ExposureLevel::E3, ControllabilityLevel::C3) => IntegrityLevel::ASIL_A,
            
            _ => IntegrityLevel::ASIL_QM,
        };
    }
    
    fn identify_mitigation_measures(&self, model: &SemanticModel, hazard: &mut Hazard) -> Result<(), SafetyError> {
        let mut measures = Vec::new();
        
        if matches!(hazard.integrity_level, IntegrityLevel::ASIL_C | IntegrityLevel::ASIL_D) {
            measures.push(MitigationMeasure {
                id: format!("{}-MIT-1", hazard.id),
                description: "Implement redundant safety function".to_string(),
                measure_type: MitigationType::Redundancy,
                effectiveness: 0.9,
                implemented_in: Vec::new(),
            });
        }
        
        measures.push(MitigationMeasure {
            id: format!("{}-MIT-2", hazard.id),
            description: "Add runtime monitoring and diagnostics".to_string(),
            measure_type: MitigationType::Detection,
            effectiveness: 0.95,
            implemented_in: Vec::new(),
        });
        
        measures.push(MitigationMeasure {
            id: format!("{}-MIT-3", hazard.id),
            description: "Implement fail-safe default state".to_string(),
            measure_type: MitigationType::Control,
            effectiveness: 0.85,
            implemented_in: Vec::new(),
        });
        
        if hazard.hazard_type == HazardType::Random {
            measures.push(MitigationMeasure {
                id: format!("{}-MIT-4", hazard.id),
                description: "Add hardware error detection and correction".to_string(),
                measure_type: MitigationType::Detection,
                effectiveness: 0.99,
                implemented_in: Vec::new(),
            });
        }
        
        hazard.mitigation_measures = measures;
        
        Ok(())
    }
}

struct HazardScenario {
    id: String,
    title: String,
    description: String,
    hazard_type: HazardType,
}

struct OperationalSituation {
    id: String,
    name: String,
    description: String,
}

struct Capability {
    id: String,
    name: String,
    is_safety_related: bool,
}

pub fn generate_hara_report(hazards: &[Hazard]) -> String {
    let mut report = String::new();
    
    report.push_str("Hazard Analysis and Risk Assessment (HARA) Report\n");
    report.push_str("==================================================\n\n");
    
    report.push_str(&format!("Total Hazards Identified: {}\n\n", hazards.len()));
    
    let asil_d = hazards.iter().filter(|h| h.integrity_level == IntegrityLevel::ASIL_D).count();
    let asil_c = hazards.iter().filter(|h| h.integrity_level == IntegrityLevel::ASIL_C).count();
    let asil_b = hazards.iter().filter(|h| h.integrity_level == IntegrityLevel::ASIL_B).count();
    let asil_a = hazards.iter().filter(|h| h.integrity_level == IntegrityLevel::ASIL_A).count();
    let qm = hazards.iter().filter(|h| h.integrity_level == IntegrityLevel::ASIL_QM).count();
    
    report.push_str("ASIL Distribution:\n");
    report.push_str("------------------\n");
    report.push_str(&format!("ASIL D: {}\n", asil_d));
    report.push_str(&format!("ASIL C: {}\n", asil_c));
    report.push_str(&format!("ASIL B: {}\n", asil_b));
    report.push_str(&format!("ASIL A: {}\n", asil_a));
    report.push_str(&format!("QM: {}\n\n", qm));
    
    report.push_str("Detailed Hazard List:\n");
    report.push_str("---------------------\n\n");
    
    for hazard in hazards {
        report.push_str(&format!("ID: {}\n", hazard.id));
        report.push_str(&format!("Title: {}\n", hazard.title));
        report.push_str(&format!("Description: {}\n", hazard.description));
        report.push_str(&format!("Severity: {:?}, Exposure: {:?}, Controllability: {:?}\n", 
            hazard.severity, hazard.exposure, hazard.controllability));
        report.push_str(&format!("ASIL: {:?}\n", hazard.integrity_level));
        report.push_str(&format!("Mitigation Measures: {}\n", hazard.mitigation_measures.len()));
        report.push_str("\n");
    }
    
    report
}

pub fn export_hara_to_table(hazards: &[Hazard]) -> String {
    let mut table = String::new();
    
    table.push_str("ID | Hazard | Severity | Exposure | Controllability | ASIL | Mitigation\n");
    table.push_str("---|--------|----------|----------|-----------------|------|------------\n");
    
    for hazard in hazards {
        table.push_str(&format!(
            "{} | {} | {:?} | {:?} | {:?} | {:?} | {}\n",
            hazard.id,
            hazard.title,
            hazard.severity,
            hazard.exposure,
            hazard.controllability,
            hazard.integrity_level,
            hazard.mitigation_measures.len()
        ));
    }
    
    table
}
