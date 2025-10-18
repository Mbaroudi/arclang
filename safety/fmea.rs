use super::*;
use crate::compiler::semantic::SemanticModel;

pub struct FMEAGenerator {
    config: FMEAConfig,
}

impl FMEAGenerator {
    pub fn new(config: FMEAConfig) -> Self {
        Self { config }
    }
    
    pub fn generate(&self, model: &SemanticModel) -> Result<Vec<FMEAEntry>, SafetyError> {
        let mut fmea_entries = Vec::new();
        
        for component in model.components() {
            for function in component.functions() {
                let failure_modes = self.identify_failure_modes(function);
                
                for failure_mode in failure_modes {
                    let entry = self.create_fmea_entry(component, function, failure_mode)?;
                    
                    if entry.rpn >= self.config.rpn_threshold {
                        fmea_entries.push(entry);
                    } else {
                        fmea_entries.push(entry);
                    }
                }
            }
        }
        
        fmea_entries.sort_by(|a, b| b.rpn.cmp(&a.rpn));
        
        Ok(fmea_entries)
    }
    
    fn identify_failure_modes(&self, function: &FunctionDecl) -> Vec<FailureMode> {
        let mut failure_modes = Vec::new();
        
        failure_modes.push(FailureMode {
            mode: "Loss of function".to_string(),
            cause: "Software defect, incorrect logic".to_string(),
        });
        
        failure_modes.push(FailureMode {
            mode: "Incorrect output".to_string(),
            cause: "Calculation error, incorrect algorithm".to_string(),
        });
        
        failure_modes.push(FailureMode {
            mode: "Delayed response".to_string(),
            cause: "Performance issue, blocking operation".to_string(),
        });
        
        failure_modes.push(FailureMode {
            mode: "Intermittent failure".to_string(),
            cause: "Race condition, timing issue".to_string(),
        });
        
        if function.has_external_communication() {
            failure_modes.push(FailureMode {
                mode: "Communication failure".to_string(),
                cause: "Network error, protocol mismatch".to_string(),
            });
        }
        
        if function.has_memory_operations() {
            failure_modes.push(FailureMode {
                mode: "Memory corruption".to_string(),
                cause: "Buffer overflow, null pointer dereference".to_string(),
            });
        }
        
        failure_modes
    }
    
    fn create_fmea_entry(&self, component: &ComponentDecl, function: &FunctionDecl, failure_mode: FailureMode) -> Result<FMEAEntry, SafetyError> {
        let severity = self.assess_severity(component, function, &failure_mode);
        let occurrence = self.assess_occurrence(&failure_mode);
        let detection = self.assess_detection(component, function, &failure_mode);
        let rpn = severity * occurrence * detection;
        
        let current_controls = self.identify_current_controls(component, function);
        let recommended_actions = if rpn >= self.config.rpn_threshold {
            self.generate_recommended_actions(severity, occurrence, detection, &failure_mode)
        } else {
            Vec::new()
        };
        
        Ok(FMEAEntry {
            id: format!("FMEA-{}-{}-{}", component.id, function.id, failure_mode.mode.replace(" ", "_")),
            component: component.name.clone(),
            function: function.name.clone(),
            failure_mode: failure_mode.mode.clone(),
            failure_cause: failure_mode.cause.clone(),
            failure_effect_local: self.describe_local_effect(function, &failure_mode),
            failure_effect_system: self.describe_system_effect(component, function, &failure_mode),
            severity,
            occurrence,
            detection,
            rpn,
            current_controls,
            recommended_actions,
            responsibility: "System Safety Engineer".to_string(),
            target_completion: None,
        })
    }
    
    fn assess_severity(&self, component: &ComponentDecl, function: &FunctionDecl, failure_mode: &FailureMode) -> u32 {
        if function.is_safety_critical || component.integrity_level == IntegrityLevel::ASIL_D {
            if failure_mode.mode.contains("Loss") {
                return 10;
            } else if failure_mode.mode.contains("Incorrect") {
                return 9;
            }
            return 8;
        }
        
        if component.integrity_level == IntegrityLevel::ASIL_C {
            if failure_mode.mode.contains("Loss") {
                return 8;
            }
            return 7;
        }
        
        if failure_mode.mode.contains("Loss") || failure_mode.mode.contains("Incorrect") {
            return 6;
        }
        
        4
    }
    
    fn assess_occurrence(&self, failure_mode: &FailureMode) -> u32 {
        if failure_mode.cause.contains("Race condition") || failure_mode.cause.contains("timing") {
            return 6;
        }
        
        if failure_mode.cause.contains("defect") || failure_mode.cause.contains("error") {
            return 5;
        }
        
        if failure_mode.cause.contains("Network") || failure_mode.cause.contains("external") {
            return 4;
        }
        
        3
    }
    
    fn assess_detection(&self, component: &ComponentDecl, function: &FunctionDecl, failure_mode: &FailureMode) -> u32 {
        let mut detection_score = 8;
        
        if function.has_input_validation() {
            detection_score -= 2;
        }
        
        if function.has_output_monitoring() {
            detection_score -= 1;
        }
        
        if component.has_diagnostics() {
            detection_score -= 2;
        }
        
        if function.has_unit_tests() {
            detection_score -= 1;
        }
        
        if failure_mode.mode.contains("Intermittent") {
            detection_score += 2;
        }
        
        detection_score.max(1)
    }
    
    fn identify_current_controls(&self, component: &ComponentDecl, function: &FunctionDecl) -> Vec<String> {
        let mut controls = Vec::new();
        
        if function.has_input_validation() {
            controls.push("Input range checking".to_string());
        }
        
        if function.has_error_handling() {
            controls.push("Exception handling".to_string());
        }
        
        if component.has_watchdog() {
            controls.push("Watchdog timer".to_string());
        }
        
        if function.has_redundancy() {
            controls.push("Redundant computation".to_string());
        }
        
        controls
    }
    
    fn generate_recommended_actions(&self, severity: u32, occurrence: u32, detection: u32, failure_mode: &FailureMode) -> Vec<String> {
        let mut actions = Vec::new();
        
        if severity >= 8 {
            actions.push("Implement redundant safety mechanism".to_string());
            actions.push("Add fail-safe default behavior".to_string());
        }
        
        if occurrence >= 6 {
            actions.push("Perform detailed code review".to_string());
            actions.push("Add additional test cases for edge conditions".to_string());
        }
        
        if detection >= 7 {
            actions.push("Implement runtime monitoring and diagnostics".to_string());
            actions.push("Add built-in self-test (BIST)".to_string());
        }
        
        if failure_mode.mode.contains("Memory") {
            actions.push("Enable memory protection unit (MPU)".to_string());
            actions.push("Use static analysis tools to detect memory issues".to_string());
        }
        
        if failure_mode.mode.contains("Communication") {
            actions.push("Implement CRC or checksum verification".to_string());
            actions.push("Add timeout and retry mechanisms".to_string());
        }
        
        actions
    }
    
    fn describe_local_effect(&self, function: &FunctionDecl, failure_mode: &FailureMode) -> String {
        format!("Function '{}' experiences {}", function.name, failure_mode.mode.to_lowercase())
    }
    
    fn describe_system_effect(&self, component: &ComponentDecl, function: &FunctionDecl, failure_mode: &FailureMode) -> String {
        if function.is_safety_critical {
            format!("System-level safety function compromised in component '{}'", component.name)
        } else {
            format!("Degraded functionality in component '{}'", component.name)
        }
    }
}

struct FailureMode {
    mode: String,
    cause: String,
}

struct FunctionDecl {
    id: String,
    name: String,
    is_safety_critical: bool,
}

impl FunctionDecl {
    fn has_external_communication(&self) -> bool {
        false
    }
    
    fn has_memory_operations(&self) -> bool {
        true
    }
    
    fn has_input_validation(&self) -> bool {
        false
    }
    
    fn has_output_monitoring(&self) -> bool {
        false
    }
    
    fn has_unit_tests(&self) -> bool {
        false
    }
    
    fn has_error_handling(&self) -> bool {
        false
    }
    
    fn has_redundancy(&self) -> bool {
        false
    }
}

struct ComponentDecl {
    id: String,
    name: String,
    integrity_level: IntegrityLevel,
}

impl ComponentDecl {
    fn functions(&self) -> Vec<FunctionDecl> {
        Vec::new()
    }
    
    fn has_diagnostics(&self) -> bool {
        false
    }
    
    fn has_watchdog(&self) -> bool {
        false
    }
}

pub fn export_fmea_to_csv(entries: &[FMEAEntry]) -> String {
    let mut csv = String::new();
    
    csv.push_str("ID,Component,Function,Failure Mode,Failure Cause,Local Effect,System Effect,Severity,Occurrence,Detection,RPN,Current Controls,Recommended Actions,Responsibility\n");
    
    for entry in entries {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},{},{},{},\"{}\",\"{}\",\"{}\"\n",
            entry.id,
            entry.component,
            entry.function,
            entry.failure_mode,
            entry.failure_cause,
            entry.failure_effect_local,
            entry.failure_effect_system,
            entry.severity,
            entry.occurrence,
            entry.detection,
            entry.rpn,
            entry.current_controls.join("; "),
            entry.recommended_actions.join("; "),
            entry.responsibility
        ));
    }
    
    csv
}

pub fn generate_fmea_summary(entries: &[FMEAEntry]) -> String {
    let mut summary = String::new();
    
    summary.push_str("FMEA Summary Report\n");
    summary.push_str("===================\n\n");
    
    summary.push_str(&format!("Total Failure Modes Analyzed: {}\n", entries.len()));
    
    let high_risk = entries.iter().filter(|e| e.rpn >= 200).count();
    let medium_risk = entries.iter().filter(|e| e.rpn >= 100 && e.rpn < 200).count();
    let low_risk = entries.iter().filter(|e| e.rpn < 100).count();
    
    summary.push_str(&format!("High Risk (RPN >= 200): {}\n", high_risk));
    summary.push_str(&format!("Medium Risk (RPN 100-199): {}\n", medium_risk));
    summary.push_str(&format!("Low Risk (RPN < 100): {}\n\n", low_risk));
    
    if high_risk > 0 {
        summary.push_str("Top 5 High-Risk Failure Modes:\n");
        summary.push_str("-------------------------------\n");
        for (i, entry) in entries.iter().take(5).enumerate() {
            summary.push_str(&format!("{}. {} - {} (RPN: {})\n", 
                i + 1, 
                entry.component, 
                entry.failure_mode, 
                entry.rpn
            ));
        }
    }
    
    summary
}
