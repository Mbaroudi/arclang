use super::constraint_engine::*;
use super::semantic_enhanced::*;
use super::capella_metamodel::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SafetyLevel {
    ASIL_A,
    ASIL_B,
    ASIL_C,
    ASIL_D,
    ASIL_QM,
    
    DAL_A,
    DAL_B,
    DAL_C,
    DAL_D,
    DAL_E,
    
    SIL_1,
    SIL_2,
    SIL_3,
    SIL_4,
    SIL_0,
    
    None,
}

impl SafetyLevel {
    pub fn from_string(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ASIL-A" | "ASIL_A" | "ASILA" => SafetyLevel::ASIL_A,
            "ASIL-B" | "ASIL_B" | "ASILB" => SafetyLevel::ASIL_B,
            "ASIL-C" | "ASIL_C" | "ASILC" => SafetyLevel::ASIL_C,
            "ASIL-D" | "ASIL_D" | "ASILD" => SafetyLevel::ASIL_D,
            "ASIL-QM" | "ASIL_QM" | "QM" => SafetyLevel::ASIL_QM,
            
            "DAL-A" | "DAL_A" | "DALA" => SafetyLevel::DAL_A,
            "DAL-B" | "DAL_B" | "DALB" => SafetyLevel::DAL_B,
            "DAL-C" | "DAL_C" | "DALC" => SafetyLevel::DAL_C,
            "DAL-D" | "DAL_D" | "DALD" => SafetyLevel::DAL_D,
            "DAL-E" | "DAL_E" | "DALE" => SafetyLevel::DAL_E,
            
            "SIL-1" | "SIL_1" | "SIL1" => SafetyLevel::SIL_1,
            "SIL-2" | "SIL_2" | "SIL2" => SafetyLevel::SIL_2,
            "SIL-3" | "SIL_3" | "SIL3" => SafetyLevel::SIL_3,
            "SIL-4" | "SIL_4" | "SIL4" => SafetyLevel::SIL_4,
            "SIL-0" | "SIL_0" | "SIL0" => SafetyLevel::SIL_0,
            
            _ => SafetyLevel::None,
        }
    }
    
    pub fn color_code(&self) -> &'static str {
        match self {
            SafetyLevel::ASIL_D | SafetyLevel::DAL_A | SafetyLevel::SIL_4 => "#D32F2F",
            SafetyLevel::ASIL_C | SafetyLevel::DAL_B | SafetyLevel::SIL_3 => "#F57C00",
            SafetyLevel::ASIL_B | SafetyLevel::DAL_C | SafetyLevel::SIL_2 => "#FBC02D",
            SafetyLevel::ASIL_A | SafetyLevel::DAL_D | SafetyLevel::SIL_1 => "#689F38",
            SafetyLevel::ASIL_QM | SafetyLevel::DAL_E | SafetyLevel::SIL_0 => "#78909C",
            SafetyLevel::None => "#9E9E9E",
        }
    }
    
    pub fn border_width(&self) -> f64 {
        match self {
            SafetyLevel::ASIL_D | SafetyLevel::DAL_A | SafetyLevel::SIL_4 => 4.0,
            SafetyLevel::ASIL_C | SafetyLevel::DAL_B | SafetyLevel::SIL_3 => 3.5,
            SafetyLevel::ASIL_B | SafetyLevel::DAL_C | SafetyLevel::SIL_2 => 3.0,
            SafetyLevel::ASIL_A | SafetyLevel::DAL_D | SafetyLevel::SIL_1 => 2.5,
            _ => 2.0,
        }
    }
    
    pub fn minimum_spacing(&self) -> f64 {
        match self {
            SafetyLevel::ASIL_D | SafetyLevel::DAL_A | SafetyLevel::SIL_4 => 80.0,
            SafetyLevel::ASIL_C | SafetyLevel::DAL_B | SafetyLevel::SIL_3 => 70.0,
            SafetyLevel::ASIL_B | SafetyLevel::DAL_C | SafetyLevel::SIL_2 => 60.0,
            SafetyLevel::ASIL_A | SafetyLevel::DAL_D | SafetyLevel::SIL_1 => 50.0,
            _ => 40.0,
        }
    }
    
    pub fn criticality_score(&self) -> f64 {
        match self {
            SafetyLevel::ASIL_D | SafetyLevel::DAL_A | SafetyLevel::SIL_4 => 1.0,
            SafetyLevel::ASIL_C | SafetyLevel::DAL_B | SafetyLevel::SIL_3 => 0.8,
            SafetyLevel::ASIL_B | SafetyLevel::DAL_C | SafetyLevel::SIL_2 => 0.6,
            SafetyLevel::ASIL_A | SafetyLevel::DAL_D | SafetyLevel::SIL_1 => 0.4,
            SafetyLevel::ASIL_QM | SafetyLevel::DAL_E | SafetyLevel::SIL_0 => 0.2,
            SafetyLevel::None => 0.0,
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            SafetyLevel::ASIL_A => "ASIL-A",
            SafetyLevel::ASIL_B => "ASIL-B",
            SafetyLevel::ASIL_C => "ASIL-C",
            SafetyLevel::ASIL_D => "ASIL-D",
            SafetyLevel::ASIL_QM => "QM",
            SafetyLevel::DAL_A => "DAL-A",
            SafetyLevel::DAL_B => "DAL-B",
            SafetyLevel::DAL_C => "DAL-C",
            SafetyLevel::DAL_D => "DAL-D",
            SafetyLevel::DAL_E => "DAL-E",
            SafetyLevel::SIL_1 => "SIL-1",
            SafetyLevel::SIL_2 => "SIL-2",
            SafetyLevel::SIL_3 => "SIL-3",
            SafetyLevel::SIL_4 => "SIL-4",
            SafetyLevel::SIL_0 => "SIL-0",
            SafetyLevel::None => "None",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyStandard {
    ISO26262,
    DO178C,
    IEC61508,
    ISO21434,
    MISRA_C,
}

impl SafetyStandard {
    pub fn name(&self) -> &'static str {
        match self {
            SafetyStandard::ISO26262 => "ISO 26262",
            SafetyStandard::DO178C => "DO-178C",
            SafetyStandard::IEC61508 => "IEC 61508",
            SafetyStandard::ISO21434 => "ISO 21434",
            SafetyStandard::MISRA_C => "MISRA C",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SafetyIntelligence {
    standard: SafetyStandard,
    enable_auto_coloring: bool,
    enforce_spacing: bool,
}

#[derive(Debug, Clone)]
pub struct SafetyAnnotation {
    pub element_id: String,
    pub safety_level: SafetyLevel,
    pub justification: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    pub violation_id: String,
    pub standard: SafetyStandard,
    pub requirement_id: String,
    pub severity: ViolationSeverity,
    pub element_ids: Vec<String>,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub struct ComplianceReport {
    pub standard: SafetyStandard,
    pub timestamp: String,
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub warnings: usize,
    pub violations: Vec<ComplianceViolation>,
    pub coverage_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct TraceLink {
    pub source_id: String,
    pub target_id: String,
    pub trace_type: TraceType,
    pub bidirectional: bool,
    pub verified: bool,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceType {
    RequirementToFunction,
    FunctionToComponent,
    ComponentToPhysical,
    RequirementToComponent,
    RequirementToRequirement,
    TestToRequirement,
    VerificationToRequirement,
}

impl TraceType {
    pub fn visual_style(&self) -> TraceVisualStyle {
        match self {
            TraceType::RequirementToFunction => TraceVisualStyle {
                stroke_color: "#1976D2".to_string(),
                stroke_width: 1.5,
                stroke_dasharray: Some("5,3".to_string()),
                arrow_type: "open".to_string(),
                label_prefix: "satisfies".to_string(),
            },
            TraceType::FunctionToComponent => TraceVisualStyle {
                stroke_color: "#388E3C".to_string(),
                stroke_width: 2.0,
                stroke_dasharray: Some("10,5".to_string()),
                arrow_type: "diamond".to_string(),
                label_prefix: "allocated_to".to_string(),
            },
            TraceType::ComponentToPhysical => TraceVisualStyle {
                stroke_color: "#F57C00".to_string(),
                stroke_width: 2.0,
                stroke_dasharray: Some("8,4".to_string()),
                arrow_type: "box".to_string(),
                label_prefix: "deployed_on".to_string(),
            },
            TraceType::RequirementToComponent => TraceVisualStyle {
                stroke_color: "#7B1FA2".to_string(),
                stroke_width: 1.5,
                stroke_dasharray: Some("3,3".to_string()),
                arrow_type: "open".to_string(),
                label_prefix: "realizes".to_string(),
            },
            TraceType::RequirementToRequirement => TraceVisualStyle {
                stroke_color: "#0097A7".to_string(),
                stroke_width: 1.5,
                stroke_dasharray: Some("2,2".to_string()),
                arrow_type: "circle".to_string(),
                label_prefix: "refines".to_string(),
            },
            TraceType::TestToRequirement => TraceVisualStyle {
                stroke_color: "#C2185B".to_string(),
                stroke_width: 1.5,
                stroke_dasharray: Some("6,2".to_string()),
                arrow_type: "triangle".to_string(),
                label_prefix: "verifies".to_string(),
            },
            TraceType::VerificationToRequirement => TraceVisualStyle {
                stroke_color: "#5D4037".to_string(),
                stroke_width: 1.5,
                stroke_dasharray: Some("4,4".to_string()),
                arrow_type: "chevron".to_string(),
                label_prefix: "validates".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceVisualStyle {
    pub stroke_color: String,
    pub stroke_width: f64,
    pub stroke_dasharray: Option<String>,
    pub arrow_type: String,
    pub label_prefix: String,
}

impl SafetyIntelligence {
    pub fn new(standard: SafetyStandard) -> Self {
        Self {
            standard,
            enable_auto_coloring: true,
            enforce_spacing: true,
        }
    }
    
    pub fn detect_safety_levels(
        &self,
        model: &EnhancedSemanticModel,
    ) -> HashMap<String, SafetyLevel> {
        let mut safety_levels = HashMap::new();
        
        for element in &model.elements {
            let level = if let Some(asil) = element.attributes.get("asil") {
                SafetyLevel::from_string(asil)
            } else if let Some(dal) = element.attributes.get("dal") {
                SafetyLevel::from_string(dal)
            } else if let Some(sil) = element.attributes.get("sil") {
                SafetyLevel::from_string(sil)
            } else if let Some(safety) = element.attributes.get("safety_level") {
                SafetyLevel::from_string(safety)
            } else {
                SafetyLevel::None
            };
            
            if level != SafetyLevel::None {
                safety_levels.insert(element.id.clone(), level);
            }
        }
        
        safety_levels
    }
    
    pub fn apply_safety_coloring(
        &self,
        element_id: &str,
        safety_level: SafetyLevel,
    ) -> SafetyVisualProperties {
        SafetyVisualProperties {
            border_color: safety_level.color_code().to_string(),
            border_width: safety_level.border_width(),
            background_tint: format!("{}20", safety_level.color_code()),
            label_suffix: format!(" [{}]", safety_level.display_name()),
            highlight: safety_level.criticality_score() > 0.7,
        }
    }
    
    pub fn enforce_safety_spacing(
        &self,
        layout: &mut HashMap<String, ElementBounds>,
        safety_levels: &HashMap<String, SafetyLevel>,
    ) {
        if !self.enforce_spacing {
            return;
        }
        
        let critical_elements: Vec<_> = safety_levels.iter()
            .filter(|(_, level)| level.criticality_score() >= 0.8)
            .map(|(id, _)| id.clone())
            .collect();
        
        for i in 0..critical_elements.len() {
            for j in i+1..critical_elements.len() {
                let id1 = &critical_elements[i];
                let id2 = &critical_elements[j];
                
                if let (Some(bounds1), Some(bounds2)) = (layout.get(id1), layout.get(id2)) {
                    let level1 = safety_levels.get(id1).unwrap();
                    let level2 = safety_levels.get(id2).unwrap();
                    
                    let required_spacing = level1.minimum_spacing().max(level2.minimum_spacing());
                    let current_distance = bounds1.distance_to(bounds2);
                    
                    if current_distance < required_spacing {
                        let gap = required_spacing - current_distance;
                        
                        if let Some(b2) = layout.get_mut(id2) {
                            b2.x += gap;
                        }
                    }
                }
            }
        }
    }
    
    pub fn check_compliance(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> ComplianceReport {
        let violations = match self.standard {
            SafetyStandard::ISO26262 => self.check_iso26262_compliance(model, layout),
            SafetyStandard::DO178C => self.check_do178c_compliance(model, layout),
            SafetyStandard::IEC61508 => self.check_iec61508_compliance(model, layout),
            SafetyStandard::ISO21434 => self.check_iso21434_compliance(model, layout),
            SafetyStandard::MISRA_C => Vec::new(),
        };
        
        let total_checks = self.get_total_checks_count();
        let failed_checks = violations.iter().filter(|v| v.severity == ViolationSeverity::Error).count();
        let warnings = violations.iter().filter(|v| v.severity == ViolationSeverity::Warning).count();
        let passed_checks = total_checks - failed_checks - warnings;
        
        let coverage_percentage = if total_checks > 0 {
            (passed_checks as f64 / total_checks as f64) * 100.0
        } else {
            0.0
        };
        
        ComplianceReport {
            standard: self.standard.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_checks,
            passed_checks,
            failed_checks,
            warnings,
            violations,
            coverage_percentage,
        }
    }
    
    fn check_iso26262_compliance(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        violations.extend(self.check_asil_decomposition(model));
        violations.extend(self.check_safety_architecture(model));
        violations.extend(self.check_functional_safety_concept(model));
        violations.extend(self.check_safety_requirements_traceability(model));
        violations.extend(self.check_safety_element_spacing(model, layout));
        
        violations
    }
    
    fn check_asil_decomposition(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        for element in &model.elements {
            if let Some(asil) = element.attributes.get("asil") {
                let level = SafetyLevel::from_string(asil);
                
                if level == SafetyLevel::ASIL_D && !element.children.is_empty() {
                    let child_asils: Vec<_> = element.children.iter()
                        .filter_map(|child_id| {
                            model.elements.iter()
                                .find(|e| &e.id == child_id)
                                .and_then(|e| e.attributes.get("asil"))
                        })
                        .collect();
                    
                    if child_asils.is_empty() {
                        violations.push(ComplianceViolation {
                            violation_id: format!("ISO26262-ASIL-001-{}", element.id),
                            standard: SafetyStandard::ISO26262,
                            requirement_id: "Part 9, Clause 5.4.7".to_string(),
                            severity: ViolationSeverity::Warning,
                            element_ids: vec![element.id.clone()],
                            description: format!(
                                "ASIL-D element '{}' has children without ASIL ratings. ASIL decomposition may be required.",
                                element.name
                            ),
                            recommendation: "Assign ASIL ratings to child elements according to decomposition rules.".to_string(),
                        });
                    }
                }
            }
        }
        
        violations
    }
    
    fn check_safety_architecture(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let safety_elements: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.attributes.get("asil").map(|a| a != "QM").unwrap_or(false) ||
                e.name.to_lowercase().contains("safety")
            })
            .collect();
        
        for element in safety_elements {
            let has_redundancy = element.attributes.get("redundancy").is_some();
            let has_monitoring = model.relationships.iter()
                .any(|r| r.source_id == element.id && r.target_id.contains("monitor"));
            
            let asil_level = element.attributes.get("asil")
                .map(|a| SafetyLevel::from_string(a))
                .unwrap_or(SafetyLevel::None);
            
            if asil_level == SafetyLevel::ASIL_D && !has_redundancy && !has_monitoring {
                violations.push(ComplianceViolation {
                    violation_id: format!("ISO26262-ARCH-001-{}", element.id),
                    standard: SafetyStandard::ISO26262,
                    requirement_id: "Part 6, Clause 7".to_string(),
                    severity: ViolationSeverity::Warning,
                    element_ids: vec![element.id.clone()],
                    description: format!(
                        "ASIL-D element '{}' lacks redundancy or monitoring mechanisms.",
                        element.name
                    ),
                    recommendation: "Consider adding redundancy or monitoring for ASIL-D elements.".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_functional_safety_concept(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let has_safety_goals = model.elements.iter()
            .any(|e| e.element_type == CapellaElementType::Requirement && 
                    e.attributes.get("type").map(|t| t == "safety_goal").unwrap_or(false));
        
        if !has_safety_goals {
            violations.push(ComplianceViolation {
                violation_id: "ISO26262-FSC-001".to_string(),
                standard: SafetyStandard::ISO26262,
                requirement_id: "Part 3, Clause 8".to_string(),
                severity: ViolationSeverity::Error,
                element_ids: Vec::new(),
                description: "No safety goals defined in the model.".to_string(),
                recommendation: "Define safety goals with ASIL ratings as part of the functional safety concept.".to_string(),
            });
        }
        
        violations
    }
    
    fn check_safety_requirements_traceability(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let safety_requirements: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.element_type == CapellaElementType::Requirement &&
                e.attributes.get("asil").is_some()
            })
            .collect();
        
        for req in safety_requirements {
            let has_trace = model.relationships.iter()
                .any(|r| r.source_id == req.id && r.relationship_type == RelationshipType::Traces);
            
            if !has_trace {
                violations.push(ComplianceViolation {
                    violation_id: format!("ISO26262-TRACE-001-{}", req.id),
                    standard: SafetyStandard::ISO26262,
                    requirement_id: "Part 8, Clause 6".to_string(),
                    severity: ViolationSeverity::Warning,
                    element_ids: vec![req.id.clone()],
                    description: format!(
                        "Safety requirement '{}' lacks traceability to implementation.",
                        req.name
                    ),
                    recommendation: "Establish traceability links from safety requirements to design elements.".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_safety_element_spacing(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let safety_levels = self.detect_safety_levels(model);
        
        let critical_elements: Vec<_> = safety_levels.iter()
            .filter(|(_, level)| level.criticality_score() >= 0.8)
            .collect();
        
        for i in 0..critical_elements.len() {
            for j in i+1..critical_elements.len() {
                let (id1, level1) = critical_elements[i];
                let (id2, level2) = critical_elements[j];
                
                if let (Some(bounds1), Some(bounds2)) = (layout.get(id1), layout.get(id2)) {
                    let required_spacing = level1.minimum_spacing().max(level2.minimum_spacing());
                    let current_distance = bounds1.distance_to(bounds2);
                    
                    if current_distance < required_spacing {
                        violations.push(ComplianceViolation {
                            violation_id: format!("ISO26262-SPACE-001-{}-{}", id1, id2),
                            standard: SafetyStandard::ISO26262,
                            requirement_id: "Part 9, Visual Guidelines".to_string(),
                            severity: ViolationSeverity::Info,
                            element_ids: vec![id1.clone(), id2.clone()],
                            description: format!(
                                "Critical safety elements are too close ({}px, required: {}px).",
                                current_distance as i32, required_spacing as i32
                            ),
                            recommendation: format!(
                                "Increase spacing between critical elements to at least {}px.",
                                required_spacing as i32
                            ),
                        });
                    }
                }
            }
        }
        
        violations
    }
    
    fn check_do178c_compliance(
        &self,
        model: &EnhancedSemanticModel,
        _layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        violations.extend(self.check_dal_assignments(model));
        violations.extend(self.check_software_architecture(model));
        violations.extend(self.check_requirements_coverage(model));
        
        violations
    }
    
    fn check_dal_assignments(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let software_components: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.element_type == CapellaElementType::LogicalComponent ||
                e.element_type == CapellaElementType::PhysicalComponent
            })
            .collect();
        
        for component in software_components {
            if component.attributes.get("dal").is_none() {
                violations.push(ComplianceViolation {
                    violation_id: format!("DO178C-DAL-001-{}", component.id),
                    standard: SafetyStandard::DO178C,
                    requirement_id: "Section 2.3.2".to_string(),
                    severity: ViolationSeverity::Warning,
                    element_ids: vec![component.id.clone()],
                    description: format!(
                        "Software component '{}' does not have a DAL assignment.",
                        component.name
                    ),
                    recommendation: "Assign a Design Assurance Level (DAL A-E) to all software components.".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_software_architecture(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let dal_a_components: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.attributes.get("dal").map(|d| d == "DAL-A" || d == "A").unwrap_or(false)
            })
            .collect();
        
        for component in dal_a_components {
            let has_partitioning = component.attributes.get("partitioned").is_some();
            
            if !has_partitioning {
                violations.push(ComplianceViolation {
                    violation_id: format!("DO178C-ARCH-001-{}", component.id),
                    standard: SafetyStandard::DO178C,
                    requirement_id: "Section 5.1.6".to_string(),
                    severity: ViolationSeverity::Warning,
                    element_ids: vec![component.id.clone()],
                    description: format!(
                        "DAL-A component '{}' should demonstrate partitioning.",
                        component.name
                    ),
                    recommendation: "Consider implementing and documenting partitioning for DAL-A components.".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_requirements_coverage(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let requirements: Vec<_> = model.elements.iter()
            .filter(|e| e.element_type == CapellaElementType::Requirement)
            .collect();
        
        for req in requirements {
            let trace_count = model.relationships.iter()
                .filter(|r| r.source_id == req.id && r.relationship_type == RelationshipType::Traces)
                .count();
            
            if trace_count == 0 {
                violations.push(ComplianceViolation {
                    violation_id: format!("DO178C-COV-001-{}", req.id),
                    standard: SafetyStandard::DO178C,
                    requirement_id: "Section 6.3.1".to_string(),
                    severity: ViolationSeverity::Error,
                    element_ids: vec![req.id.clone()],
                    description: format!(
                        "Requirement '{}' has no traceability to design or code.",
                        req.name
                    ),
                    recommendation: "Establish bidirectional traceability from requirements to design and code.".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_iec61508_compliance(
        &self,
        model: &EnhancedSemanticModel,
        _layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        violations.extend(self.check_sil_assignments(model));
        violations.extend(self.check_systematic_capability(model));
        violations.extend(self.check_safety_functions(model));
        
        violations
    }
    
    fn check_sil_assignments(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let safety_functions: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.element_type == CapellaElementType::Function &&
                e.name.to_lowercase().contains("safety")
            })
            .collect();
        
        for func in safety_functions {
            if func.attributes.get("sil").is_none() {
                violations.push(ComplianceViolation {
                    violation_id: format!("IEC61508-SIL-001-{}", func.id),
                    standard: SafetyStandard::IEC61508,
                    requirement_id: "Part 1, Clause 7.4".to_string(),
                    severity: ViolationSeverity::Error,
                    element_ids: vec![func.id.clone()],
                    description: format!(
                        "Safety function '{}' does not have a SIL rating.",
                        func.name
                    ),
                    recommendation: "Assign a Safety Integrity Level (SIL 1-4) to all safety functions.".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_systematic_capability(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let sil_3_4_elements: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.attributes.get("sil")
                    .map(|s| s == "SIL-3" || s == "SIL-4" || s == "3" || s == "4")
                    .unwrap_or(false)
            })
            .collect();
        
        for element in sil_3_4_elements {
            let has_diverse_redundancy = element.attributes.get("diverse_redundancy").is_some();
            
            if !has_diverse_redundancy {
                violations.push(ComplianceViolation {
                    violation_id: format!("IEC61508-SC-001-{}", element.id),
                    standard: SafetyStandard::IEC61508,
                    requirement_id: "Part 2, Clause 7.4.2.3".to_string(),
                    severity: ViolationSeverity::Warning,
                    element_ids: vec![element.id.clone()],
                    description: format!(
                        "SIL 3/4 element '{}' should implement diverse redundancy.",
                        element.name
                    ),
                    recommendation: "Consider implementing diverse redundancy for high-SIL elements.".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_safety_functions(&self, model: &EnhancedSemanticModel) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let safety_functions: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.attributes.get("sil").is_some()
            })
            .collect();
        
        for func in safety_functions {
            let allocated_count = func.allocated_to.len();
            
            if allocated_count == 0 {
                violations.push(ComplianceViolation {
                    violation_id: format!("IEC61508-ALLOC-001-{}", func.id),
                    standard: SafetyStandard::IEC61508,
                    requirement_id: "Part 3, Clause 7.4".to_string(),
                    severity: ViolationSeverity::Error,
                    element_ids: vec![func.id.clone()],
                    description: format!(
                        "Safety function '{}' is not allocated to any component.",
                        func.name
                    ),
                    recommendation: "Allocate safety functions to implementing components.".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_iso21434_compliance(
        &self,
        model: &EnhancedSemanticModel,
        _layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        
        let security_elements: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.name.to_lowercase().contains("security") ||
                e.name.to_lowercase().contains("cyber")
            })
            .collect();
        
        if security_elements.is_empty() {
            violations.push(ComplianceViolation {
                violation_id: "ISO21434-SEC-001".to_string(),
                standard: SafetyStandard::ISO21434,
                requirement_id: "Clause 9".to_string(),
                severity: ViolationSeverity::Warning,
                element_ids: Vec::new(),
                description: "No cybersecurity elements identified in the model.".to_string(),
                recommendation: "Consider cybersecurity requirements according to ISO 21434.".to_string(),
            });
        }
        
        violations
    }
    
    fn get_total_checks_count(&self) -> usize {
        match self.standard {
            SafetyStandard::ISO26262 => 25,
            SafetyStandard::DO178C => 20,
            SafetyStandard::IEC61508 => 22,
            SafetyStandard::ISO21434 => 15,
            SafetyStandard::MISRA_C => 0,
        }
    }
    
    pub fn extract_trace_links(&self, model: &EnhancedSemanticModel) -> Vec<TraceLink> {
        let mut traces = Vec::new();
        
        for relationship in &model.relationships {
            let trace_type = self.infer_trace_type(&relationship.source_id, &relationship.target_id, model);
            
            if let Some(ttype) = trace_type {
                traces.push(TraceLink {
                    source_id: relationship.source_id.clone(),
                    target_id: relationship.target_id.clone(),
                    trace_type: ttype,
                    bidirectional: false,
                    verified: relationship.attributes.get("verified").map(|v| v == "true").unwrap_or(false),
                    attributes: relationship.attributes.clone(),
                });
            }
        }
        
        for element in &model.elements {
            for allocated_id in &element.allocated_to {
                traces.push(TraceLink {
                    source_id: element.id.clone(),
                    target_id: allocated_id.clone(),
                    trace_type: TraceType::FunctionToComponent,
                    bidirectional: false,
                    verified: false,
                    attributes: HashMap::new(),
                });
            }
        }
        
        traces
    }
    
    fn infer_trace_type(
        &self,
        source_id: &str,
        target_id: &str,
        model: &EnhancedSemanticModel,
    ) -> Option<TraceType> {
        let source = model.elements.iter().find(|e| e.id == source_id)?;
        let target = model.elements.iter().find(|e| e.id == target_id)?;
        
        if source.element_type == CapellaElementType::Requirement {
            if target.element_type == CapellaElementType::Function ||
               target.element_type == CapellaElementType::SystemFunction {
                return Some(TraceType::RequirementToFunction);
            } else if target.element_type == CapellaElementType::Component ||
                      target.element_type == CapellaElementType::SystemComponent {
                return Some(TraceType::RequirementToComponent);
            } else if target.element_type == CapellaElementType::Requirement {
                return Some(TraceType::RequirementToRequirement);
            }
        }
        
        if (source.element_type == CapellaElementType::Function ||
            source.element_type == CapellaElementType::SystemFunction) &&
           (target.element_type == CapellaElementType::Component ||
            target.element_type == CapellaElementType::SystemComponent) {
            return Some(TraceType::FunctionToComponent);
        }
        
        if (source.element_type == CapellaElementType::LogicalComponent ||
            source.element_type == CapellaElementType::Component) &&
           (target.element_type == CapellaElementType::PhysicalComponent ||
            target.element_type == CapellaElementType::NodeComponent) {
            return Some(TraceType::ComponentToPhysical);
        }
        
        None
    }
}

#[derive(Debug, Clone)]
pub struct SafetyVisualProperties {
    pub border_color: String,
    pub border_width: f64,
    pub background_tint: String,
    pub label_suffix: String,
    pub highlight: bool,
}

impl Default for SafetyIntelligence {
    fn default() -> Self {
        Self::new(SafetyStandard::ISO26262)
    }
}
