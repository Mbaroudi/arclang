use super::*;
use crate::compiler::semantic::SemanticModel;

pub fn check_compliance(model: &SemanticModel, config: &SafetyConfig) -> ComplianceStatus {
    let mut non_compliances = Vec::new();
    let mut recommendations = Vec::new();
    let mut total_checks = 0;
    let mut passed_checks = 0;
    
    let dal = extract_dal_from_config(config);
    
    check_planning_process(model, dal, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_development_process(model, dal, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_verification_process(model, dal, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_configuration_management(model, dal, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_quality_assurance(model, dal, &mut non_compliances, &mut total_checks, &mut passed_checks);
    check_certification_liaison(model, dal, &mut non_compliances, &mut total_checks, &mut passed_checks);
    
    if !non_compliances.is_empty() {
        recommendations.push(format!("Complete all DO-178C objectives for DAL {:?}", dal));
        recommendations.push("Consider using qualified tools to reduce verification burden".to_string());
        if matches!(dal, DO178Level::LevelA | DO178Level::LevelB) {
            recommendations.push("Implement formal methods (DO-333) for critical components".to_string());
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

fn extract_dal_from_config(config: &SafetyConfig) -> DO178Level {
    match config.target_integrity_level {
        IntegrityLevel::DAL_A => DO178Level::LevelA,
        IntegrityLevel::DAL_B => DO178Level::LevelB,
        IntegrityLevel::DAL_C => DO178Level::LevelC,
        IntegrityLevel::DAL_D => DO178Level::LevelD,
        IntegrityLevel::DAL_E => DO178Level::LevelE,
        _ => DO178Level::LevelC,
    }
}

fn check_planning_process(model: &SemanticModel, dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_software_development_plan() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 4.1".to_string(),
            description: "Plan for Software Aspects of Certification (PSAC) missing".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Planning".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_software_development_standards() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 4.2".to_string(),
            description: "Software Development Plan missing".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Planning".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_verification_plan() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 4.3".to_string(),
            description: "Software Verification Plan missing".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Planning".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_configuration_management_plan() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 4.4".to_string(),
            description: "Software Configuration Management Plan missing".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Planning".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_quality_assurance_plan() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 4.5".to_string(),
            description: "Software Quality Assurance Plan missing".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Planning".to_string()],
        });
    }
}

fn check_development_process(model: &SemanticModel, dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_high_level_requirements() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 5.1".to_string(),
            description: "High-Level Requirements (HLR) not documented".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Development".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_low_level_requirements() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 5.2".to_string(),
            description: "Low-Level Requirements (LLR) not documented".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Development".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_software_architecture() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 5.3".to_string(),
            description: "Software Architecture not documented".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Development".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_source_code() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 5.4".to_string(),
            description: "Source Code not developed".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Development".to_string()],
        });
    }
    
    check_derived_requirements(model, dal, non_compliances, total_checks, passed_checks);
}

fn check_verification_process(model: &SemanticModel, dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    check_reviews_and_analyses(model, dal, non_compliances, total_checks, passed_checks);
    check_testing(model, dal, non_compliances, total_checks, passed_checks);
    check_structural_coverage(model, dal, non_compliances, total_checks, passed_checks);
}

fn check_reviews_and_analyses(model: &SemanticModel, dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_requirements_review() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 6.3.1".to_string(),
            description: "Requirements review not performed".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Verification".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_design_review() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 6.3.2".to_string(),
            description: "Design review not performed".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Verification".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_code_review() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 6.3.3".to_string(),
            description: "Code review not performed".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Verification".to_string()],
        });
    }
    
    if matches!(dal, DO178Level::LevelA | DO178Level::LevelB) {
        *total_checks += 1;
        if model.has_requirements_based_test_coverage() {
            *passed_checks += 1;
        } else {
            non_compliances.push(NonCompliance {
                clause: "DO-178C Section 6.4.2".to_string(),
                description: "Requirements-based test coverage analysis not complete".to_string(),
                severity: ComplianceSeverity::Critical,
                affected_elements: vec!["Verification".to_string()],
            });
        }
    }
}

fn check_testing(model: &SemanticModel, dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    let has_unit_tests = model.test_specifications().iter()
        .any(|t| t.test_level == "unit");
    
    if has_unit_tests || matches!(dal, DO178Level::LevelC | DO178Level::LevelD | DO178Level::LevelE) {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 6.4.3".to_string(),
            description: format!("Software unit testing required for DAL {:?}", dal),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Verification".to_string()],
        });
    }
    
    *total_checks += 1;
    let has_integration_tests = model.test_specifications().iter()
        .any(|t| t.test_level == "integration");
    
    if has_integration_tests {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 6.4.4".to_string(),
            description: "Software integration testing not performed".to_string(),
            severity: ComplianceSeverity::Critical,
            affected_elements: vec!["Verification".to_string()],
        });
    }
}

fn check_structural_coverage(model: &SemanticModel, dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    let required_coverage = match dal {
        DO178Level::LevelA => StructuralCoverage::ModifiedConditionDecisionCoverage,
        DO178Level::LevelB => StructuralCoverage::DecisionCoverage,
        DO178Level::LevelC => StructuralCoverage::StatementCoverage,
        DO178Level::LevelD | DO178Level::LevelE => StructuralCoverage::None,
    };
    
    if required_coverage != StructuralCoverage::None {
        *total_checks += 1;
        if model.has_structural_coverage_analysis(&required_coverage) {
            *passed_checks += 1;
        } else {
            non_compliances.push(NonCompliance {
                clause: "DO-178C Section 6.4.4".to_string(),
                description: format!("Structural coverage analysis ({:?}) not performed", required_coverage),
                severity: ComplianceSeverity::Critical,
                affected_elements: vec!["Verification".to_string()],
            });
        }
    }
}

fn check_configuration_management(model: &SemanticModel, _dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_configuration_identification() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 7.2".to_string(),
            description: "Configuration identification not established".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["CM".to_string()],
        });
    }
    
    *total_checks += 1;
    if model.has_baseline_control() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 7.3".to_string(),
            description: "Baseline and traceability not established".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["CM".to_string()],
        });
    }
}

fn check_quality_assurance(model: &SemanticModel, _dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_qa_records() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 8.2".to_string(),
            description: "Quality assurance records not maintained".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["QA".to_string()],
        });
    }
}

fn check_certification_liaison(model: &SemanticModel, _dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    if model.has_certification_artifacts() {
        *passed_checks += 1;
    } else {
        non_compliances.push(NonCompliance {
            clause: "DO-178C Section 9".to_string(),
            description: "Certification liaison process not established".to_string(),
            severity: ComplianceSeverity::Major,
            affected_elements: vec!["Certification".to_string()],
        });
    }
}

fn check_derived_requirements(model: &SemanticModel, _dal: DO178Level, non_compliances: &mut Vec<NonCompliance>, total_checks: &mut usize, passed_checks: &mut usize) {
    *total_checks += 1;
    
    let derived_reqs = model.requirements().iter()
        .filter(|req| req.is_derived)
        .collect::<Vec<_>>();
    
    if derived_reqs.is_empty() {
        *passed_checks += 1;
        return;
    }
    
    for req in derived_reqs {
        if !req.has_justification() {
            non_compliances.push(NonCompliance {
                clause: "DO-178C Section 5.1.3".to_string(),
                description: format!("Derived requirement {} lacks rationale/justification", req.id),
                severity: ComplianceSeverity::Major,
                affected_elements: vec![req.id.clone()],
            });
            return;
        }
    }
    
    *passed_checks += 1;
}

#[derive(Debug, Clone, PartialEq)]
enum StructuralCoverage {
    None,
    StatementCoverage,
    DecisionCoverage,
    ModifiedConditionDecisionCoverage,
}

pub fn generate_do178c_objectives_table(dal: DO178Level, analysis: &SafetyAnalysisResult) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("DO-178C Objectives for DAL {:?}\n", dal));
    report.push_str("=====================================\n\n");
    
    report.push_str("Objective | Description | Status | Evidence\n");
    report.push_str("----------|-------------|--------|----------\n");
    
    let objectives = get_objectives_for_dal(dal);
    
    for obj in objectives {
        let status = "Pending";
        let evidence = "-";
        report.push_str(&format!("{} | {} | {} | {}\n", obj.number, obj.description, status, evidence));
    }
    
    report
}

struct DO178Objective {
    number: String,
    description: String,
}

fn get_objectives_for_dal(dal: DO178Level) -> Vec<DO178Objective> {
    vec![
        DO178Objective {
            number: "A-1".to_string(),
            description: "High-level requirements comply with system requirements".to_string(),
        },
        DO178Objective {
            number: "A-2".to_string(),
            description: "High-level requirements are accurate and consistent".to_string(),
        },
        DO178Objective {
            number: "A-3".to_string(),
            description: "Low-level requirements comply with high-level requirements".to_string(),
        },
        DO178Objective {
            number: "A-4".to_string(),
            description: "Low-level requirements are accurate and consistent".to_string(),
        },
        DO178Objective {
            number: "A-5".to_string(),
            description: "Software architecture is consistent with high-level requirements".to_string(),
        },
        DO178Objective {
            number: "A-6".to_string(),
            description: "Source code complies with low-level requirements".to_string(),
        },
        DO178Objective {
            number: "A-7".to_string(),
            description: "Source code is accurate and consistent".to_string(),
        },
    ]
}
