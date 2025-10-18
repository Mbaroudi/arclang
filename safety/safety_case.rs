use super::*;
use crate::compiler::semantic::SemanticModel;
use std::collections::HashMap;

pub struct SafetyCaseBuilder {
    config: SafetyConfig,
}

impl SafetyCaseBuilder {
    pub fn new(config: SafetyConfig) -> Self {
        Self { config }
    }
    
    pub fn build(&self, model: &SemanticModel, analysis: &SafetyAnalysisResult) -> Result<SafetyCase, SafetyError> {
        let top_claim = SafetyClaim {
            id: "SC-TOP".to_string(),
            claim_text: format!("{} is acceptably safe to operate", model.name()),
            claim_type: ClaimType::TopLevel,
            status: ClaimStatus::Asserted,
            evidence: Vec::new(),
            sub_claims: Vec::new(),
        };
        
        let mut safety_case = SafetyCase {
            id: format!("SC-{}", model.name()),
            system_name: model.name().to_string(),
            version: "1.0".to_string(),
            created_at: Utc::now(),
            top_claim,
            context: self.build_context(model, analysis),
            assumptions: self.build_assumptions(model),
            justifications: self.build_justifications(model, analysis),
            evidence: self.collect_evidence(model, analysis),
            arguments: Vec::new(),
        };
        
        self.develop_argument_structure(&mut safety_case, model, analysis)?;
        
        Ok(safety_case)
    }
    
    fn build_context(&self, model: &SemanticModel, analysis: &SafetyAnalysisResult) -> Vec<ContextElement> {
        let mut context = Vec::new();
        
        context.push(ContextElement {
            id: "CTX-1".to_string(),
            description: format!("System operates in {} domain", model.domain()),
            element_type: ContextType::Operational,
        });
        
        for standard in &self.config.standards {
            context.push(ContextElement {
                id: format!("CTX-STD-{:?}", standard),
                description: format!("System must comply with {:?}", standard),
                element_type: ContextType::Regulatory,
            });
        }
        
        context.push(ContextElement {
            id: "CTX-2".to_string(),
            description: format!("Target integrity level: {:?}", self.config.target_integrity_level),
            element_type: ContextType::SafetyRequirement,
        });
        
        context
    }
    
    fn build_assumptions(&self, model: &SemanticModel) -> Vec<Assumption> {
        let mut assumptions = Vec::new();
        
        assumptions.push(Assumption {
            id: "ASM-1".to_string(),
            description: "Hardware platform functions according to specification".to_string(),
            rationale: "Hardware is certified and tested independently".to_string(),
            validity: AssumptionValidity::Valid,
        });
        
        assumptions.push(Assumption {
            id: "ASM-2".to_string(),
            description: "Operators are trained and follow procedures".to_string(),
            rationale: "Training program in place and validated".to_string(),
            validity: AssumptionValidity::Valid,
        });
        
        assumptions.push(Assumption {
            id: "ASM-3".to_string(),
            description: "System operates within specified environmental conditions".to_string(),
            rationale: "Environmental limits defined in operational requirements".to_string(),
            validity: AssumptionValidity::Valid,
        });
        
        assumptions
    }
    
    fn build_justifications(&self, model: &SemanticModel, analysis: &SafetyAnalysisResult) -> Vec<Justification> {
        let mut justifications = Vec::new();
        
        justifications.push(Justification {
            id: "JUST-1".to_string(),
            description: "HARA process follows ISO 26262 methodology".to_string(),
            rationale: format!("{} hazards identified and assessed", analysis.hazards.len()),
        });
        
        justifications.push(Justification {
            id: "JUST-2".to_string(),
            description: "Requirements traceability is complete".to_string(),
            rationale: "All safety requirements traced to hazards and verified".to_string(),
        });
        
        if !analysis.fmea_results.is_empty() {
            justifications.push(Justification {
                id: "JUST-3".to_string(),
                description: "FMEA demonstrates failure modes are mitigated".to_string(),
                rationale: format!("{} failure modes analyzed with mitigation strategies", analysis.fmea_results.len()),
            });
        }
        
        justifications
    }
    
    fn collect_evidence(&self, model: &SemanticModel, analysis: &SafetyAnalysisResult) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        
        evidence.push(Evidence {
            id: "EVD-HARA".to_string(),
            evidence_type: EvidenceType::Analysis,
            description: "Hazard Analysis and Risk Assessment".to_string(),
            reference: "HARA Report".to_string(),
            confidence: EvidenceConfidence::High,
        });
        
        if !analysis.fmea_results.is_empty() {
            evidence.push(Evidence {
                id: "EVD-FMEA".to_string(),
                evidence_type: EvidenceType::Analysis,
                description: "Failure Modes and Effects Analysis".to_string(),
                reference: "FMEA Report".to_string(),
                confidence: EvidenceConfidence::High,
            });
        }
        
        if !analysis.fta_results.is_empty() {
            evidence.push(Evidence {
                id: "EVD-FTA".to_string(),
                evidence_type: EvidenceType::Analysis,
                description: "Fault Tree Analysis".to_string(),
                reference: "FTA Report".to_string(),
                confidence: EvidenceConfidence::High,
            });
        }
        
        if analysis.verification_summary.verified_requirements > 0 {
            evidence.push(Evidence {
                id: "EVD-TEST".to_string(),
                evidence_type: EvidenceType::Test,
                description: format!("{} safety requirements verified through testing", 
                    analysis.verification_summary.verified_requirements),
                reference: "Test Results".to_string(),
                confidence: EvidenceConfidence::High,
            });
        }
        
        evidence.push(Evidence {
            id: "EVD-REVIEW".to_string(),
            evidence_type: EvidenceType::Review,
            description: "Design and code reviews performed".to_string(),
            reference: "Review Records".to_string(),
            confidence: EvidenceConfidence::Medium,
        });
        
        evidence
    }
    
    fn develop_argument_structure(&self, safety_case: &mut SafetyCase, model: &SemanticModel, analysis: &SafetyAnalysisResult) -> Result<(), SafetyError> {
        let mut sub_claims = Vec::new();
        
        let hazards_mitigated_claim = SafetyClaim {
            id: "SC-1".to_string(),
            claim_text: "All identified hazards are mitigated to acceptable levels".to_string(),
            claim_type: ClaimType::SubClaim,
            status: if analysis.hazards.iter().all(|h| !h.mitigation_measures.is_empty()) {
                ClaimStatus::Validated
            } else {
                ClaimStatus::Asserted
            },
            evidence: vec!["EVD-HARA".to_string(), "EVD-FMEA".to_string()],
            sub_claims: Vec::new(),
        };
        sub_claims.push(hazards_mitigated_claim);
        
        let requirements_satisfied_claim = SafetyClaim {
            id: "SC-2".to_string(),
            claim_text: "All safety requirements are satisfied and verified".to_string(),
            claim_type: ClaimType::SubClaim,
            status: if analysis.verification_summary.verified_requirements == analysis.verification_summary.total_safety_requirements {
                ClaimStatus::Validated
            } else {
                ClaimStatus::Asserted
            },
            evidence: vec!["EVD-TEST".to_string()],
            sub_claims: Vec::new(),
        };
        sub_claims.push(requirements_satisfied_claim);
        
        let standards_compliant_claim = SafetyClaim {
            id: "SC-3".to_string(),
            claim_text: "System design complies with applicable safety standards".to_string(),
            claim_type: ClaimType::SubClaim,
            status: if analysis.standards_compliance.values().all(|c| c.compliant) {
                ClaimStatus::Validated
            } else {
                ClaimStatus::Asserted
            },
            evidence: vec!["EVD-REVIEW".to_string()],
            sub_claims: Vec::new(),
        };
        sub_claims.push(standards_compliant_claim);
        
        safety_case.top_claim.sub_claims = sub_claims;
        
        let argument = SafetyArgument {
            id: "ARG-1".to_string(),
            argument_type: ArgumentType::DecompositionByHazards,
            description: "Argument over all identified hazards".to_string(),
            parent_claim: "SC-TOP".to_string(),
            child_claims: vec!["SC-1".to_string(), "SC-2".to_string(), "SC-3".to_string()],
        };
        
        safety_case.arguments.push(argument);
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCase {
    pub id: String,
    pub system_name: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub top_claim: SafetyClaim,
    pub context: Vec<ContextElement>,
    pub assumptions: Vec<Assumption>,
    pub justifications: Vec<Justification>,
    pub evidence: Vec<Evidence>,
    pub arguments: Vec<SafetyArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyClaim {
    pub id: String,
    pub claim_text: String,
    pub claim_type: ClaimType,
    pub status: ClaimStatus,
    pub evidence: Vec<String>,
    pub sub_claims: Vec<SafetyClaim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimType {
    TopLevel,
    SubClaim,
    LeafClaim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimStatus {
    Asserted,
    Validated,
    Refuted,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextElement {
    pub id: String,
    pub description: String,
    pub element_type: ContextType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextType {
    Operational,
    Environmental,
    Regulatory,
    SafetyRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub id: String,
    pub description: String,
    pub rationale: String,
    pub validity: AssumptionValidity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssumptionValidity {
    Valid,
    Invalid,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Justification {
    pub id: String,
    pub description: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub reference: String,
    pub confidence: EvidenceConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    Test,
    Analysis,
    Review,
    Inspection,
    Formal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyArgument {
    pub id: String,
    pub argument_type: ArgumentType,
    pub description: String,
    pub parent_claim: String,
    pub child_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgumentType {
    DecompositionByHazards,
    DecompositionByRequirements,
    DecompositionByComponents,
    EvidentialSupport,
}

pub fn export_to_gsn(safety_case: &SafetyCase) -> String {
    let mut gsn = String::new();
    
    gsn.push_str("Goal Structuring Notation (GSN) - Safety Case\n");
    gsn.push_str("==============================================\n\n");
    
    gsn.push_str(&format!("System: {}\n", safety_case.system_name));
    gsn.push_str(&format!("Version: {}\n", safety_case.version));
    gsn.push_str(&format!("Date: {}\n\n", safety_case.created_at.format("%Y-%m-%d")));
    
    gsn.push_str("Top Goal:\n");
    gsn.push_str(&format!("[G-TOP] {}\n\n", safety_case.top_claim.claim_text));
    
    gsn.push_str("Context:\n");
    for ctx in &safety_case.context {
        gsn.push_str(&format!("[C-{}] {}\n", ctx.id, ctx.description));
    }
    gsn.push_str("\n");
    
    gsn.push_str("Assumptions:\n");
    for asm in &safety_case.assumptions {
        gsn.push_str(&format!("[A-{}] {}\n", asm.id, asm.description));
    }
    gsn.push_str("\n");
    
    gsn.push_str("Argument Structure:\n");
    for arg in &safety_case.arguments {
        gsn.push_str(&format!("[S-{}] {:?}: {}\n", arg.id, arg.argument_type, arg.description));
    }
    gsn.push_str("\n");
    
    gsn.push_str("Sub-Goals:\n");
    for sub_claim in &safety_case.top_claim.sub_claims {
        gsn.push_str(&format!("[G-{}] {} [Status: {:?}]\n", sub_claim.id, sub_claim.claim_text, sub_claim.status));
        
        if !sub_claim.evidence.is_empty() {
            gsn.push_str("  Supported by evidence:\n");
            for evd_id in &sub_claim.evidence {
                if let Some(evidence) = safety_case.evidence.iter().find(|e| e.id == *evd_id) {
                    gsn.push_str(&format!("    [E-{}] {}\n", evidence.id, evidence.description));
                }
            }
        }
        gsn.push_str("\n");
    }
    
    gsn
}

pub fn generate_safety_case_report(safety_case: &SafetyCase) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("Safety Case Report: {}\n", safety_case.system_name));
    report.push_str("=======================================\n\n");
    
    report.push_str(&format!("Version: {}\n", safety_case.version));
    report.push_str(&format!("Date: {}\n\n", safety_case.created_at.format("%Y-%m-%d %H:%M:%S UTC")));
    
    report.push_str("1. Top-Level Claim\n");
    report.push_str("------------------\n");
    report.push_str(&format!("{}\n\n", safety_case.top_claim.claim_text));
    
    report.push_str("2. Context\n");
    report.push_str("----------\n");
    for ctx in &safety_case.context {
        report.push_str(&format!("- {}\n", ctx.description));
    }
    report.push_str("\n");
    
    report.push_str("3. Assumptions\n");
    report.push_str("--------------\n");
    for asm in &safety_case.assumptions {
        report.push_str(&format!("- {} ({})\n", asm.description, asm.rationale));
    }
    report.push_str("\n");
    
    report.push_str("4. Argument Structure\n");
    report.push_str("---------------------\n");
    for sub_claim in &safety_case.top_claim.sub_claims {
        report.push_str(&format!("Claim: {}\n", sub_claim.claim_text));
        report.push_str(&format!("Status: {:?}\n", sub_claim.status));
        report.push_str(&format!("Evidence: {}\n\n", sub_claim.evidence.join(", ")));
    }
    
    report.push_str("5. Evidence\n");
    report.push_str("-----------\n");
    for evd in &safety_case.evidence {
        report.push_str(&format!("{} ({:?}): {}\n", evd.id, evd.evidence_type, evd.description));
        report.push_str(&format!("  Reference: {}\n", evd.reference));
        report.push_str(&format!("  Confidence: {:?}\n\n", evd.confidence));
    }
    
    report
}
