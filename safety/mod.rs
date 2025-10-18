pub mod iso26262;
pub mod do178c;
pub mod iec61508;
pub mod fmea;
pub mod fta;
pub mod hazard_analysis;
pub mod safety_case;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub standards: Vec<SafetyStandard>,
    pub target_integrity_level: IntegrityLevel,
    pub certification_authority: Option<String>,
    pub project_safety_goals: Vec<SafetyGoal>,
    pub fmea_config: FMEAConfig,
    pub fta_config: FTAConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SafetyStandard {
    ISO26262 { edition: String },
    DO178C { level: DO178Level },
    IEC61508 { edition: String },
    IEC62304,
    MISRAC,
    ARINC653,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IntegrityLevel {
    ASIL_QM,
    ASIL_A,
    ASIL_B,
    ASIL_C,
    ASIL_D,
    DAL_E,
    DAL_D,
    DAL_C,
    DAL_B,
    DAL_A,
    SIL_1,
    SIL_2,
    SIL_3,
    SIL_4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DO178Level {
    LevelA,
    LevelB,
    LevelC,
    LevelD,
    LevelE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyGoal {
    pub id: String,
    pub description: String,
    pub hazard_id: String,
    pub integrity_level: IntegrityLevel,
    pub functional_safety_concept: String,
    pub verification_measures: Vec<VerificationMeasure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMeasure {
    pub method: VerificationMethod,
    pub coverage_target: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    Test,
    Review,
    Analysis,
    Inspection,
    Simulation,
    FormalProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FMEAConfig {
    pub severity_scale: Vec<SeverityLevel>,
    pub occurrence_scale: Vec<OccurrenceLevel>,
    pub detection_scale: Vec<DetectionLevel>,
    pub rpn_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityLevel {
    pub level: u32,
    pub description: String,
    pub criteria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccurrenceLevel {
    pub level: u32,
    pub description: String,
    pub probability_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionLevel {
    pub level: u32,
    pub description: String,
    pub detection_capability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FTAConfig {
    pub top_event: String,
    pub cut_set_order: u32,
    pub probability_threshold: f64,
    pub include_common_cause: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAnalysisResult {
    pub project: String,
    pub timestamp: DateTime<Utc>,
    pub standards_compliance: HashMap<SafetyStandard, ComplianceStatus>,
    pub hazards: Vec<Hazard>,
    pub fmea_results: Vec<FMEAEntry>,
    pub fta_results: Vec<FaultTree>,
    pub safety_requirements: Vec<SafetyRequirement>,
    pub verification_summary: VerificationSummary,
    pub gaps: Vec<SafetyGap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub compliant: bool,
    pub compliance_percentage: f64,
    pub non_compliances: Vec<NonCompliance>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonCompliance {
    pub clause: String,
    pub description: String,
    pub severity: ComplianceSeverity,
    pub affected_elements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceSeverity {
    Critical,
    Major,
    Minor,
    Observation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hazard {
    pub id: String,
    pub title: String,
    pub description: String,
    pub hazard_type: HazardType,
    pub severity: HazardSeverity,
    pub exposure: ExposureLevel,
    pub controllability: ControllabilityLevel,
    pub integrity_level: IntegrityLevel,
    pub safety_goals: Vec<String>,
    pub mitigation_measures: Vec<MitigationMeasure>,
    pub residual_risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HazardType {
    Functional,
    Environmental,
    Systematic,
    Random,
    CommonCause,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HazardSeverity {
    S0,
    S1,
    S2,
    S3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExposureLevel {
    E0,
    E1,
    E2,
    E3,
    E4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControllabilityLevel {
    C0,
    C1,
    C2,
    C3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationMeasure {
    pub id: String,
    pub description: String,
    pub measure_type: MitigationType,
    pub effectiveness: f64,
    pub implemented_in: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationType {
    Prevention,
    Detection,
    Control,
    Redundancy,
    Isolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Acceptable,
    Tolerable,
    Unacceptable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FMEAEntry {
    pub id: String,
    pub component: String,
    pub function: String,
    pub failure_mode: String,
    pub failure_cause: String,
    pub failure_effect_local: String,
    pub failure_effect_system: String,
    pub severity: u32,
    pub occurrence: u32,
    pub detection: u32,
    pub rpn: u32,
    pub current_controls: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub responsibility: String,
    pub target_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultTree {
    pub id: String,
    pub top_event: FaultEvent,
    pub gates: Vec<FaultGate>,
    pub basic_events: Vec<BasicEvent>,
    pub minimal_cut_sets: Vec<CutSet>,
    pub top_event_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultEvent {
    pub id: String,
    pub description: String,
    pub event_type: FaultEventType,
    pub probability: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultEventType {
    TopEvent,
    IntermediateEvent,
    BasicEvent,
    UndevelopedEvent,
    ConditionalEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultGate {
    pub id: String,
    pub gate_type: GateType,
    pub input_events: Vec<String>,
    pub output_event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateType {
    AND,
    OR,
    XOR,
    NOT,
    VOTE { k: u32, n: u32 },
    PAND,
    POR,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicEvent {
    pub id: String,
    pub description: String,
    pub failure_rate: f64,
    pub exposure_time: f64,
    pub probability: f64,
    pub diagnostic_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutSet {
    pub events: Vec<String>,
    pub probability: f64,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRequirement {
    pub id: String,
    pub derived_from: Vec<String>,
    pub requirement_text: String,
    pub integrity_level: IntegrityLevel,
    pub verification_method: VerificationMethod,
    pub verification_status: SafetyVerificationStatus,
    pub allocated_to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SafetyVerificationStatus {
    NotStarted,
    InProgress,
    Verified,
    Failed,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub total_safety_requirements: usize,
    pub verified_requirements: usize,
    pub coverage_by_method: HashMap<VerificationMethod, VerificationCoverage>,
    pub coverage_by_integrity_level: HashMap<IntegrityLevel, VerificationCoverage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCoverage {
    pub total: usize,
    pub verified: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyGap {
    pub gap_type: SafetyGapType,
    pub severity: ComplianceSeverity,
    pub description: String,
    pub affected_elements: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyGapType {
    MissingHazardAnalysis,
    MissingSafetyRequirement,
    MissingVerification,
    InsufficientIntegrityLevel,
    MissingFMEA,
    MissingFTA,
    IncompleteMitigation,
    NonCompliantDesign,
}

#[derive(Debug, thiserror::Error)]
pub enum SafetyError {
    #[error("Invalid integrity level: {0}")]
    InvalidIntegrityLevel(String),
    
    #[error("Hazard not found: {0}")]
    HazardNotFound(String),
    
    #[error("Safety requirement not satisfied: {0}")]
    RequirementNotSatisfied(String),
    
    #[error("Insufficient verification coverage: expected {expected}%, got {actual}%")]
    InsufficientCoverage { expected: f64, actual: f64 },
    
    #[error("Non-compliant with {standard}: {reason}")]
    NonCompliant { standard: String, reason: String },
    
    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

pub struct SafetyAnalyzer {
    config: SafetyConfig,
}

impl SafetyAnalyzer {
    pub fn new(config: SafetyConfig) -> Self {
        Self { config }
    }
    
    pub fn analyze(&self, model: &crate::compiler::semantic::SemanticModel) -> Result<SafetyAnalysisResult, SafetyError> {
        let mut result = SafetyAnalysisResult {
            project: model.name().to_string(),
            timestamp: Utc::now(),
            standards_compliance: HashMap::new(),
            hazards: Vec::new(),
            fmea_results: Vec::new(),
            fta_results: Vec::new(),
            safety_requirements: Vec::new(),
            verification_summary: VerificationSummary {
                total_safety_requirements: 0,
                verified_requirements: 0,
                coverage_by_method: HashMap::new(),
                coverage_by_integrity_level: HashMap::new(),
            },
            gaps: Vec::new(),
        };
        
        self.extract_hazards(model, &mut result)?;
        self.extract_safety_requirements(model, &mut result)?;
        self.check_standards_compliance(model, &mut result)?;
        self.compute_verification_summary(&mut result);
        self.identify_gaps(model, &mut result);
        
        Ok(result)
    }
    
    fn extract_hazards(&self, model: &crate::compiler::semantic::SemanticModel, result: &mut SafetyAnalysisResult) -> Result<(), SafetyError> {
        for hazard_decl in model.hazards() {
            let hazard = Hazard {
                id: hazard_decl.id.clone(),
                title: hazard_decl.title.clone(),
                description: hazard_decl.description.clone(),
                hazard_type: self.map_hazard_type(&hazard_decl.hazard_type),
                severity: hazard_decl.severity,
                exposure: hazard_decl.exposure,
                controllability: hazard_decl.controllability,
                integrity_level: self.determine_integrity_level(
                    hazard_decl.severity,
                    hazard_decl.exposure,
                    hazard_decl.controllability
                ),
                safety_goals: hazard_decl.safety_goals.clone(),
                mitigation_measures: Vec::new(),
                residual_risk: RiskLevel::Tolerable,
            };
            
            result.hazards.push(hazard);
        }
        
        Ok(())
    }
    
    fn extract_safety_requirements(&self, model: &crate::compiler::semantic::SemanticModel, result: &mut SafetyAnalysisResult) -> Result<(), SafetyError> {
        for req in model.requirements() {
            if req.is_safety_requirement {
                let safety_req = SafetyRequirement {
                    id: req.id.clone(),
                    derived_from: req.traces.satisfies.clone(),
                    requirement_text: req.text.clone(),
                    integrity_level: req.safety_level.unwrap_or(IntegrityLevel::ASIL_QM),
                    verification_method: VerificationMethod::Test,
                    verification_status: SafetyVerificationStatus::NotStarted,
                    allocated_to: req.allocated_to.clone(),
                };
                
                result.safety_requirements.push(safety_req);
            }
        }
        
        Ok(())
    }
    
    fn check_standards_compliance(&self, model: &crate::compiler::semantic::SemanticModel, result: &mut SafetyAnalysisResult) -> Result<(), SafetyError> {
        for standard in &self.config.standards {
            let compliance = match standard {
                SafetyStandard::ISO26262 { .. } => iso26262::check_compliance(model, &self.config),
                SafetyStandard::DO178C { .. } => do178c::check_compliance(model, &self.config),
                SafetyStandard::IEC61508 { .. } => iec61508::check_compliance(model, &self.config),
                _ => ComplianceStatus {
                    compliant: false,
                    compliance_percentage: 0.0,
                    non_compliances: Vec::new(),
                    recommendations: vec!["Standard not yet implemented".to_string()],
                },
            };
            
            result.standards_compliance.insert(standard.clone(), compliance);
        }
        
        Ok(())
    }
    
    fn compute_verification_summary(&self, result: &mut SafetyAnalysisResult) {
        result.verification_summary.total_safety_requirements = result.safety_requirements.len();
        result.verification_summary.verified_requirements = result.safety_requirements.iter()
            .filter(|req| req.verification_status == SafetyVerificationStatus::Verified)
            .count();
    }
    
    fn identify_gaps(&self, _model: &crate::compiler::semantic::SemanticModel, result: &mut SafetyAnalysisResult) {
        for hazard in &result.hazards {
            if hazard.mitigation_measures.is_empty() {
                result.gaps.push(SafetyGap {
                    gap_type: SafetyGapType::IncompleteMitigation,
                    severity: ComplianceSeverity::Major,
                    description: format!("Hazard {} has no mitigation measures", hazard.id),
                    affected_elements: vec![hazard.id.clone()],
                    recommendation: "Define and implement mitigation measures".to_string(),
                });
            }
        }
        
        for req in &result.safety_requirements {
            if req.verification_status == SafetyVerificationStatus::NotStarted {
                result.gaps.push(SafetyGap {
                    gap_type: SafetyGapType::MissingVerification,
                    severity: ComplianceSeverity::Major,
                    description: format!("Safety requirement {} not verified", req.id),
                    affected_elements: vec![req.id.clone()],
                    recommendation: "Create and execute verification test cases".to_string(),
                });
            }
        }
    }
    
    fn map_hazard_type(&self, type_str: &str) -> HazardType {
        match type_str.to_lowercase().as_str() {
            "functional" => HazardType::Functional,
            "environmental" => HazardType::Environmental,
            "systematic" => HazardType::Systematic,
            "random" => HazardType::Random,
            _ => HazardType::Functional,
        }
    }
    
    fn determine_integrity_level(&self, severity: HazardSeverity, exposure: ExposureLevel, controllability: ControllabilityLevel) -> IntegrityLevel {
        match (&severity, &exposure, &controllability) {
            (HazardSeverity::S3, ExposureLevel::E4, ControllabilityLevel::C3) => IntegrityLevel::ASIL_D,
            (HazardSeverity::S3, _, ControllabilityLevel::C3) => IntegrityLevel::ASIL_C,
            (HazardSeverity::S3, _, _) => IntegrityLevel::ASIL_B,
            (HazardSeverity::S2, _, _) => IntegrityLevel::ASIL_B,
            (HazardSeverity::S1, _, _) => IntegrityLevel::ASIL_A,
            _ => IntegrityLevel::ASIL_QM,
        }
    }
}
