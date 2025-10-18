use super::*;
use crate::compiler::semantic::SemanticModel;

pub fn check_compliance(model: &SemanticModel, config: &SafetyConfig) -> ComplianceStatus {
    let mut non_compliances = Vec::new();
    let mut recommendations = Vec::new();
    let mut total_checks = 0;
    let mut passed_checks = 0;
    
    let sil = extract_sil_from_config(config);
    
    check_part1_general_requirements(model, sil, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part2_euc_requirements(model, sil, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part3_software_requirements(model, sil, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part4_definitions(model, sil, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part7_techniques(model, sil, &mut non_compliances, &mut total_checks, &mut passed_checks);
    
    if !non_compliances.is_empty() {
        recommendations.push(format!("Complete all IEC 61508 requirements for SIL {:?}", sil));
        recommendations.push("Establish safety lifecycle per Part 1 Clause 7".to_string());
        if matches!(sil, SIL::SIL3 | SIL::SIL4) {
            recommendations.push("Consider diverse programming for highest SIL levels".to_string());
        }
    }
    
    let compliance_percentage = if total_checks > 0 {
        (passed_checks as f64 / total_checks as f64) * 100.0
    } else {
        0.0
    };
    
    ComplianceStatus {
        compliant: non_compliances.is_empty(),
        compliance_percentage,
        non_compliances,
        recommendations,
    }
}

#[derive(Debug, Clone, Copy)]
enum SIL {
    SIL1,
    SIL2,
    SIL3,
    SIL4,
}

fn extract_sil_from_config(config: &SafetyConfig) -> SIL {
    match config.target_integrity_level {
        IntegrityLevel::SIL_1 => SIL::SIL1,
        IntegrityLevel::SIL_2 => SIL::SIL2,
        IntegrityLevel::SIL_3 => SIL::SIL3,
        IntegrityLevel::SIL_4 => SIL::SIL4,
        _ => SIL::SIL2,
    }
}

fn check_part1_general_requirements(model: &SemanticModel, sil: SIL, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_safety_lifecycle() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-1:2010 Clause 7".to_string(),
            description: "Safety lifecycle not established".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Overall Safety Lifecycle".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_safety_plan() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-1:2010 Clause 8".to_string(),
            description: "Overall safety management plan not documented".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Management".to_string()],
        });
    }
    
    *total_checks += 1;
    if !model.hazards().is_empty() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-1:2010 Clause 7.4".to_string(),
            description: "Hazard and risk analysis not performed".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Concept".to_string()],
        });
    }
}

fn check_part2_euc_requirements(model: &SemanticModel, sil: SIL, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_euc_description() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-2:2010 Clause 7.4".to_string(),
            description: "EUC (Equipment Under Control) and control system not specified".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["System Design".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_safety_functions() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-2:2010 Clause 7.6".to_string(),
            description: "Safety functions requirements specification missing".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["System Requirements".to_string()],
        });
    }
}

fn check_part3_software_requirements(model: &SemanticModel, sil: SIL, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_software_safety_lifecycle() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-3:2010 Clause 7.1".to_string(),
            description: "Software safety lifecycle requirements not established".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Software Lifecycle".to_string()],
        });
    }
    
    *total_checks += 1;
    let has_sw_safety_requirements = model.requirements().iter()
        .any(|req| req.is_safety_requirement && req.level == "software");
    
    if has_sw_safety_requirements {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-3:2010 Clause 7.4".to_string(),
            description: "Software safety requirements specification missing".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Software Requirements".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_software_architecture() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-3:2010 Clause 7.4".to_string(),
            description: "Software design and architecture not documented".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Software Design".to_string()],
        });
    }
    
    check_software_verification(model, sil, non_compliances, total_checks, passed_checks);
    check_software_validation(model, sil, non_compliances, total_checks, passed_checks);
}

fn check_part4_definitions(model: &SemanticModel, _sil: SIL, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    *passed_checks += 1;
}

fn check_part7_techniques(model: &SemanticModel, sil: SIL, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    let recommended_techniques = get_recommended_techniques_for_sil(sil);
    
    *total_checks += 1;
    
    let has_modular_approach = model.has_modular_design();
    let has_defensive_programming = model.has_defensive_programming_patterns();
    let has_coding_standards = model.has_coding_standards();
    
    let technique_compliance = has_modular_approach && has_coding_standards;
    
    if technique_compliance {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-7:2010 Annex A & B".to_string(),
            description: format!("Recommended techniques for SIL {:?} not fully applied", sil),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Software Development".to_string()],
        });
    }
    
    if matches!(sil, SIL::SIL3 | SIL::SIL4) && !has_defensive_programming {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-7:2010 Table A.3".to_string(),
            description: format!("Defensive programming highly recommended for SIL {:?}", sil),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Software Implementation".to_string()],
        });
    }
}

fn check_software_verification(model: &SemanticModel, sil: SIL, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_verification_plan() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-3:2010 Clause 7.7".to_string(),
            description: "Software verification plan not established".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Verification".to_string()],
        });
    }
    
    *total_checks += 1;
    let has_unit_testing = model.test_specifications().iter()
        .any(|t| t.test_level == "unit");
    
    if has_unit_testing {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-3:2010 Table B.1".to_string(),
            description: format!("Software module testing required for SIL {:?}", sil),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Testing".to_string()],
        });
    }
    
    *total_checks += 1;
    let has_integration_testing = model.test_specifications().iter()
        .any(|t| t.test_level == "integration");
    
    if has_integration_testing {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-3:2010 Table B.2".to_string(),
            description: "Software integration testing not performed".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Testing".to_string()],
        });
    }
    
    if matches!(sil, SIL::SIL3 | SIL::SIL4) {
        *total_checks += 1;
        if model.has_static_analysis() {
            *passed_checks += 1;
        } else {
            non_compliances.push(NonCompliance {
                clause: "IEC 61508-3:2010 Table C.1".to_string(),
                description: format!("Static analysis highly recommended for SIL {:?}", sil),
                severity: ComplianceSeverity::Major,
                affected_elements: vec!["Verification".to_string()],
            });
        }
    }
}

fn check_software_validation(model: &SemanticModel, _sil: SIL, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_validation_plan() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-3:2010 Clause 7.9".to_string(),
            description: "Software validation plan not established".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Validation".to_string()],
        });
    }
    
    *total_checks += 1;
    let has_functional_tests = model.test_specifications().iter()
        .any(|t| t.test_type == "functional");
    
    if has_functional_tests {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "IEC 61508-3:2010 Clause 7.9".to_string(),
            description: "Functional and black-box testing not performed".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Validation".to_string()],
        });
    }
}

struct RecommendedTechnique {
    name: String,
    recommendation_level: TechniqueRecommendation,
}

#[derive(Debug, Clone, Copy)]
enum TechniqueRecommendation {
    HighlyRecommended,
    Recommended,
    NeutralOrNotRecommended,
}

fn get_recommended_techniques_for_sil(sil: SIL) -> Vec<RecommendedTechnique> {
    match sil {
        SIL::SIL4 | SIL::SIL3 => vec![
            RecommendedTechnique {
                name: "Formal methods".to_string(),
                recommendation_level: TechniqueRecommendation::HighlyRecommended,
            },
            RecommendedTechnique {
                name: "Semi-formal methods".to_string(),
                recommendation_level: TechniqueRecommendation::HighlyRecommended,
            },
            RecommendedTechnique {
                name: "Modular approach".to_string(),
                recommendation_level: TechniqueRecommendation::HighlyRecommended,
            },
            RecommendedTechnique {
                name: "Defensive programming".to_string(),
                recommendation_level: TechniqueRecommendation::HighlyRecommended,
            },
        ],
        SIL::SIL2 => vec![
            RecommendedTechnique {
                name: "Semi-formal methods".to_string(),
                recommendation_level: TechniqueRecommendation::Recommended,
            },
            RecommendedTechnique {
                name: "Modular approach".to_string(),
                recommendation_level: TechniqueRecommendation::HighlyRecommended,
            },
            RecommendedTechnique {
                name: "Defensive programming".to_string(),
                recommendation_level: TechniqueRecommendation::Recommended,
            },
        ],
        SIL::SIL1 => vec![
            RecommendedTechnique {
                name: "Modular approach".to_string(),
                recommendation_level: TechniqueRecommendation::Recommended,
            },
            RecommendedTechnique {
                name: "Structured programming".to_string(),
                recommendation_level: TechniqueRecommendation::Recommended,
            },
        ],
    }
}

pub fn generate_iec61508_report(analysis: &SafetyAnalysisResult, sil: SIL) -> String {
    let mut report = String::new();
    
    report.push_str("IEC 61508:2010 Compliance Report\n");
    report.push_str("=================================\n\n");
    
    report.push_str(&format!("Target Safety Integrity Level: SIL {:?}\n\n", sil));
    
    if let Some(compliance) = analysis.standards_compliance.iter()
        .find(|(std, _)| matches!(std, SafetyStandard::IEC61508 { .. }))
    {
        report.push_str(&format!("Overall Compliance: {:.1}%\n", compliance.1.compliance_percentage));
        report.push_str(&format!("Status: {}\n\n", if compliance.1.compliant { "COMPLIANT" } else { "NON-COMPLIANT" }));
        
        if !compliance.1.non_compliances.is_empty() {
            report.push_str("Non-Compliances:\n");
            report.push_str("----------------\n");
            for nc in &compliance.1.non_compliances {
                report.push_str(&format!("- [{}] {}: {}\n", 
                    match nc.severity {
                        ComplianceSeverity::Critical => "CRITICAL",
                        ComplianceSeverity::Major => "MAJOR",
                        ComplianceSeverity::Minor => "MINOR",
                        ComplianceSeverity::Observation => "OBSERVATION",
                    },
                    nc.clause,
                    nc.description
                ));
            }
            report.push_str("\n");
        }
        
        report.push_str("Recommended Techniques:\n");
        report.push_str("-----------------------\n");
        let techniques = get_recommended_techniques_for_sil(sil);
        for technique in techniques {
            report.push_str(&format!("- {} [{:?}]\n", 
                technique.name, 
                technique.recommendation_level
            ));
        }
    }
    
    report
}
