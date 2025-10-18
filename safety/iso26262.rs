use super::*;
use crate::compiler::semantic::SemanticModel;

pub fn check_compliance(model: &SemanticModel, config: &SafetyConfig) -> ComplianceStatus {
    let mut non_compliances = Vec::new();
    let mut recommendations = Vec::new();
    let mut total_checks = 0;
    let mut passed_checks = 0;
    
    check_part3_system_design(model, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part4_sw_level(model, config, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part5_hw_level(model, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part6_sw_product(model, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part8_supporting_processes(model, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_part9_asil_oriented(model, config, &mut non_compliances, &mut total_checks, &mut passed_checks);
    
    if !non_compliances.is_empty() {
        recommendations.push("Review all non-compliances and create corrective action plan".to_string());
        recommendations.push("Consider tool qualification for ASIL-C and above (Part 8)".to_string());
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

fn check_part3_system_design(model: &SemanticModel, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_operational_analysis() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-3:2018 Clause 5".to_string(),
            description: "Missing item definition and operational context (Operational Analysis)".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Operational Level".to_string()],
        });
    }
    
    *total_checks += 1;
    if !model.hazards().is_empty() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-3:2018 Clause 6".to_string(),
            description: "Hazard analysis and risk assessment not performed".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["System Level".to_string()],
        });
    }
    
    *total_checks += 1;
    let has_functional_safety_concept = model.requirements().iter()
        .any(|req| req.is_safety_requirement && req.req_type == "functional");
    
    if has_functional_safety_concept {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-3:2018 Clause 7".to_string(),
            description: "Functional safety concept not defined".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["System Level".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_system_architecture() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-3:2018 Clause 8".to_string(),
            description: "Technical safety concept and system architecture missing".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["System Level".to_string()],
        });
    }
}

fn check_part4_sw_level(model: &SemanticModel, config: &SafetyConfig, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    let has_sw_safety_requirements = model.requirements().iter()
        .any(|req| req.is_safety_requirement && req.level == "software");
    
    if has_sw_safety_requirements {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-6:2018 Clause 5".to_string(),
            description: "Software safety requirements specification missing".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Software Level".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_software_architecture() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-6:2018 Clause 6".to_string(),
            description: "Software architectural design missing".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Software Level".to_string()],
        });
    }
    
    check_asil_decomposition(model, config, non_compliances, total_checks, passed_checks);
}

fn check_part5_hw_level(model: &SemanticModel, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    let has_hw_safety_requirements = model.requirements().iter()
        .any(|req| req.is_safety_requirement && req.level == "hardware");
    
    if has_hw_safety_requirements {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-5:2018 Clause 6".to_string(),
            description: "Hardware safety requirements specification missing".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Hardware Level".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_physical_architecture() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-5:2018 Clause 7".to_string(),
            description: "Hardware architectural design missing".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Physical Level".to_string()],
        });
    }
}

fn check_part6_sw_product(model: &SemanticModel, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    let has_unit_design = model.components().iter()
        .any(|c| c.component_type == "software_unit");
    
    if has_unit_design {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-6:2018 Clause 7".to_string(),
            description: "Software unit design and implementation not documented".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Software Level".to_string()],
        });
    }
    
    *total_checks += 1;
    let has_test_spec = model.test_specifications().iter()
        .any(|t| t.test_level == "unit");
    
    if has_test_spec {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-6:2018 Clause 9".to_string(),
            description: "Software unit testing specification missing".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Software Level".to_string()],
        });
    }
}

fn check_part8_supporting_processes(model: &SemanticModel, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_configuration_management() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-8:2018 Clause 5".to_string(),
            description: "Configuration management strategy not defined".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Process".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_change_management() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-8:2018 Clause 6".to_string(),
            description: "Change management process not defined".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Process".to_string()],
        });
    }
}

fn check_part9_asil_oriented(model: &SemanticModel, config: &SafetyConfig, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    
    let requires_analysis = matches!(
        config.target_integrity_level,
        IntegrityLevel::ASIL_C | IntegrityLevel::ASIL_D
    );
    
    if requires_analysis {
        let has_dependent_failure_analysis = model.has_dependent_failure_analysis();
        
        if has_dependent_failure_analysis {
            *passed_checks += 1;
        } else {
            non_compliances.push(NonCompliance {
                clause: "ISO 26262-9:2018 Clause 7".to_string(),
                description: format!("Analysis of dependent failures required for {:?}", config.target_integrity_level),
                severity: ComplianceSeverity::Critical,
                affected_elements: vec!["System Level".to_string()],
            });
        }
    } else {
        *passed_checks += 1;
    }
    
    *total_checks += 1;
    let has_safety_case = model.has_safety_case();
    if has_safety_case {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "ISO 26262-9:2018 Clause 8".to_string(),
            description: "Safety validation and confirmation measures not defined".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["System Level".to_string()],
        });
    }
}

fn check_asil_decomposition(model: &SemanticModel, config: &SafetyConfig, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    
    for req in model.requirements() {
        if req.is_safety_requirement && req.has_asil_decomposition() {
            if !validate_asil_decomposition(&req.asil_decomposition, &config.target_integrity_level) {
                non_compliances.push(NonCompliance {
                    clause: "ISO 26262-9:2018 Clause 5".to_string(),
                    description: format!("Invalid ASIL decomposition for requirement {}", req.id),
                    severity: ComplianceSeverity::Critical,
                    affected_elements: vec![req.id.clone()],
                });
                return;
            }
        }
    }
    
    *passed_checks += 1;
}

fn validate_asil_decomposition(decomposition: &ASILDecomposition, target: &IntegrityLevel) -> bool {
    match (target, &decomposition.component_a_level, &decomposition.component_b_level) {
        (IntegrityLevel::ASIL_D, IntegrityLevel::ASIL_C, IntegrityLevel::ASIL_C) => true,
        (IntegrityLevel::ASIL_D, IntegrityLevel::ASIL_B, IntegrityLevel::ASIL_D) => true,
        (IntegrityLevel::ASIL_C, IntegrityLevel::ASIL_B, IntegrityLevel::ASIL_B) => true,
        (IntegrityLevel::ASIL_C, IntegrityLevel::ASIL_A, IntegrityLevel::ASIL_C) => true,
        (IntegrityLevel::ASIL_B, IntegrityLevel::ASIL_A, IntegrityLevel::ASIL_B) => true,
        _ => false,
    }
}

#[derive(Debug, Clone)]
struct ASILDecomposition {
    component_a_level: IntegrityLevel,
    component_b_level: IntegrityLevel,
}

pub fn generate_iso26262_report(analysis: &SafetyAnalysisResult) -> String {
    let mut report = String::new();
    
    report.push_str("ISO 26262:2018 Compliance Report\n");
    report.push_str("=================================\n\n");
    
    if let Some(compliance) = analysis.standards_compliance.iter()
        .find(|(std, _)| matches!(std, SafetyStandard::ISO26262 { .. }))
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
        
        if !compliance.1.recommendations.is_empty() {
            report.push_str("Recommendations:\n");
            report.push_str("----------------\n");
            for rec in &compliance.1.recommendations {
                report.push_str(&format!("- {}\n", rec));
            }
        }
    }
    
    report
}
