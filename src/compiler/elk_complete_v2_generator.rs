//! ELK Complete Generator V2 - Integrated with Phase 1 & 2 Rendering Pipeline
//!
//! This is the NEW integrated generator that uses:
//! - the canonical `SemanticModel` (single derivation, no AST re-analysis)
//! - SemanticContext::from_model for phase detection
//! - LayoutStrategy for context-aware configuration
//! - ArcadiaRulesEngine for methodology compliance
//! - ProfessionalStyler for Capella colors
//! - PostProcessor for alignment
//! - QualityMetrics for scoring

use super::semantic::SemanticModel;
use serde_json::{json, Value};

// Import Phase 1 & 2 modules
use crate::compiler::semantic_analyzer::{ElementType, SemanticContext};
use crate::compiler::layout_strategy::StrategySelector;
use crate::compiler::arcadia_rules_engine::ArcadiaRulesEngine;
use crate::compiler::professional_styler::{ProfessionalStyler, StyleConfig};
use crate::compiler::post_processor::{PostProcessor, PostProcessConfig};
use crate::compiler::quality_metrics_v2::QualityMetrics;

/// ELK Complete Generator V2 with integrated rendering pipeline
#[derive(Debug, Clone)]
pub struct ElkCompleteV2Generator {
    pub enable_arcadia_rules: bool,
    pub enable_professional_styling: bool,
    pub enable_post_processing: bool,
    pub enable_quality_metrics: bool,
}

impl Default for ElkCompleteV2Generator {
    fn default() -> Self {
        Self {
            enable_arcadia_rules: true,
            enable_professional_styling: true,
            enable_post_processing: true,
            enable_quality_metrics: true,
        }
    }
}

impl ElkCompleteV2Generator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Main generation method with full pipeline.
    ///
    /// Consumes the canonical `SemanticModel` produced by the compilation
    /// pipeline — the semantic context is derived once from it, never
    /// re-analyzed from the AST.
    pub fn generate(&self, model: &SemanticModel) -> Result<GenerationResult, String> {
        println!("🚀 ELK Complete V2 Generator - Starting");
        println!("   Pipeline: Semantic → Strategy → Rules → Style → Post → Quality");
        println!();

        // Step 1: Semantic context from the canonical model (single derivation)
        println!("📊 Step 1: Semantic Context (from canonical model)");
        let semantic = SemanticContext::from_model(model);

        println!("   ✓ Phase detected: {:?}", semantic.phase);
        println!("   ✓ Elements: {}", semantic.elements.len());
        println!("   ✓ Relationships: {} connections", semantic.relationships.connections.len());
        println!("   ✓ Has actors: {}", semantic.has_actors);
        println!("   ✓ Has hierarchy: {}", semantic.has_hierarchy);
        println!("   ✓ Has safety critical: {}", semantic.has_safety_critical);
        println!("   ✓ Recommended strategy: {:?}", semantic.recommended_strategy);
        println!();
        
        // Step 2: Layout Strategy Selection
        println!("📐 Step 2: Layout Strategy Selection");
        let selector = StrategySelector::new();
        let strategy = selector.select(&semantic);
        let layout_config = strategy.configure(&semantic);
        
        println!("   ✓ Selected strategy: {}", strategy.name());
        println!("   ✓ Algorithm: {}", layout_config.algorithm);
        println!("   ✓ Direction: {}", layout_config.direction);
        println!("   ✓ Options: {} configured", layout_config.options.len());
        println!();
        
        // Step 3: Generate base diagram with ELK
        println!("🎨 Step 3: Base Diagram Generation");
        let mut diagram_data = self.generate_elk_diagram(model, &semantic, &layout_config)?;
        println!("   ✓ Generated {} nodes", 
            diagram_data.get("nodes").and_then(|n| n.as_array()).map(|a| a.len()).unwrap_or(0));
        println!("   ✓ Generated {} edges", 
            diagram_data.get("edges").and_then(|e| e.as_array()).map(|a| a.len()).unwrap_or(0));
        println!();
        
        // Step 4: Apply Arcadia Rules
        let rules_result = if self.enable_arcadia_rules {
            println!("📋 Step 4: Arcadia Rules Application");
            let engine = ArcadiaRulesEngine::new();
            let result = engine.apply(&mut diagram_data, &semantic);
            
            println!("   ✓ Rules applied: {}", result.rules_applied);
            println!("   ✓ Rules passed: {}", result.rules_passed);
            println!("   ✓ Rules failed: {}", result.rules_failed);
            
            if !result.violations.is_empty() {
                println!("   ⚠ Violations: {}", result.violations.len());
                for violation in result.violations.iter().take(3) {
                    println!("     - [{}] {}: {}", 
                        format!("{:?}", violation.severity),
                        violation.rule_name,
                        violation.message
                    );
                }
            }
            println!();
            
            Some(result)
        } else {
            None
        };
        
        // Step 5: Apply Professional Styling
        if self.enable_professional_styling {
            println!("🎨 Step 5: Professional Styling");
            let config = StyleConfig::default(); // Capella theme
            let styler = ProfessionalStyler::new(config);
            styler.apply_styles(&mut diagram_data, &semantic);
            
            println!("   ✓ Applied Capella color scheme");
            println!("   ✓ Applied safety indicators");
            println!("   ✓ Added shadows and depth effects");
            println!("   ✓ Generated legend");
            println!();
        }
        
        // Step 6: Post-Processing
        if self.enable_post_processing {
            println!("✨ Step 6: Post-Processing");
            let config = PostProcessConfig::default(); // 10px grid
            let processor = PostProcessor::new(config);
            diagram_data = processor.process(diagram_data);
            
            println!("   ✓ Grid snapped to 10px");
            println!("   ✓ Elements aligned");
            println!("   ✓ Spacing distributed");
            println!("   ✓ Labels optimized");
            println!();
        }
        
        // Step 7: Quality Metrics
        let quality_report = if self.enable_quality_metrics {
            println!("📊 Step 7: Quality Metrics");
            let metrics = QualityMetrics::new();
            let report = metrics.calculate(&diagram_data, &semantic);
            
            println!("   ✓ Overall Score: {:.1}/10", report.overall_score);
            println!("   ✓ Edge Crossings: {} (target: <5)", report.edge_crossings);
            println!("   ✓ Node Overlaps: {} (target: 0)", report.node_overlaps);
            println!("   ✓ Whitespace Balance: {:.2} (target: 0.4-0.6)", report.whitespace_balance);
            println!("   ✓ Alignment Score: {:.2} (target: >0.8)", report.alignment_score);
            println!("   ✓ Arcadia Compliance: {:.0}% (target: >90%)", report.arcadia_compliance);
            
            if !report.warnings.is_empty() {
                println!("   ⚠ Warnings:");
                for warning in &report.warnings {
                    println!("     - {}", warning);
                }
            }
            println!();
            
            Some(report)
        } else {
            None
        };
        
        println!("✅ Generation Complete!");
        println!();
        
        Ok(GenerationResult {
            diagram_data,
            semantic,
            quality_report,
            rules_result,
        })
    }
    
    /// Generate base ELK diagram with strategy configuration.
    ///
    /// Nodes come from the canonical classification (components and physical
    /// nodes); edges come from the canonical `interfaces` (logical component
    /// exchanges, declared interfaces, and physical exchanges), with
    /// endpoints resolved to element ids.
    fn generate_elk_diagram(
        &self,
        model: &SemanticModel,
        semantic: &SemanticContext,
        layout_config: &crate::compiler::layout_strategy::LayoutConfig,
    ) -> Result<Value, String> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for element in &semantic.elements {
            match element.element_type {
                ElementType::Component => {
                    nodes.push(json!({
                        "id": element.id,
                        "type": "component",
                        "width": 100.0,
                        "height": 60.0,
                        "properties": {}
                    }));
                }
                ElementType::PhysicalNode => {
                    nodes.push(json!({
                        "id": element.id,
                        "type": "physical_node",
                        "width": 120.0,
                        "height": 80.0,
                        "properties": {}
                    }));
                }
                _ => {}
            }
        }

        for interface in &model.interfaces {
            edges.push(json!({
                "id": format!("edge_{}", edges.len()),
                "source": SemanticContext::resolve_endpoint(&interface.from, model),
                "target": SemanticContext::resolve_endpoint(&interface.to, model),
            }));
        }

        // Build diagram data structure
        let diagram = json!({
            "nodes": nodes,
            "edges": edges,
            "layout_config": {
                "algorithm": layout_config.algorithm,
                "direction": layout_config.direction,
                "options": layout_config.options
            }
        });

        Ok(diagram)
    }
}

/// Generation result with all pipeline outputs
pub struct GenerationResult {
    pub diagram_data: Value,
    pub semantic: SemanticContext,
    pub quality_report: Option<crate::compiler::quality_metrics_v2::QualityReport>,
    pub rules_result: Option<crate::compiler::arcadia_rules_engine::RuleApplicationResult>,
}

impl GenerationResult {
    /// Convert to HTML with embedded quality report
    pub fn to_html(&self) -> String {
        let quality_html = if let Some(ref report) = self.quality_report {
            format!(r#"
                <div class="quality-report" style="position: fixed; top: 10px; right: 10px; 
                     background: white; padding: 15px; border: 1px solid #ccc; border-radius: 5px; 
                     box-shadow: 0 2px 5px rgba(0,0,0,0.1); font-family: Arial, sans-serif; z-index: 1000;">
                    <h3 style="margin: 0 0 10px 0; font-size: 14px;">Quality Report</h3>
                    <div style="font-size: 12px;">
                        <p><strong>Overall Score:</strong> {:.1}/10</p>
                        <p><strong>Edge Crossings:</strong> {}</p>
                        <p><strong>Node Overlaps:</strong> {}</p>
                        <p><strong>Whitespace:</strong> {:.0}%</p>
                        <p><strong>Alignment:</strong> {:.0}%</p>
                        <p><strong>Arcadia Compliance:</strong> {:.0}%</p>
                        {}
                    </div>
                </div>
            "#,
                report.overall_score,
                report.edge_crossings,
                report.node_overlaps,
                report.whitespace_balance * 100.0,
                report.alignment_score * 100.0,
                report.arcadia_compliance,
                if !report.warnings.is_empty() {
                    format!("<p style='color: orange; font-size: 11px;'><strong>Warnings:</strong><br/>{}</p>", 
                        report.warnings.join("<br/>"))
                } else {
                    String::new()
                }
            )
        } else {
            String::new()
        };
        
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>ArcLang Diagram - Phase 1 & 2 Pipeline</title>
    <style>
        body {{ margin: 0; padding: 20px; font-family: Arial, sans-serif; }}
        #diagram {{ width: 100%; height: 800px; border: 1px solid #ddd; }}
    </style>
</head>
<body>
    <h1>ArcLang Diagram - ELK Complete V2</h1>
    <p>Generated with Phase 1 & 2 Rendering Pipeline</p>
    <p>Phase: <strong>{:?}</strong> | Strategy: <strong>{:?}</strong></p>
    {}
    <div id="diagram">
        <pre>{}</pre>
    </div>
</body>
</html>"#,
            self.semantic.phase,
            self.semantic.recommended_strategy,
            quality_html,
            serde_json::to_string_pretty(&self.diagram_data).unwrap_or_else(|_| "Error formatting diagram".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generator_creation() {
        let generator = ElkCompleteV2Generator::new();
        assert!(generator.enable_arcadia_rules);
        assert!(generator.enable_quality_metrics);
    }

    #[test]
    fn generates_diagram_from_canonical_model() {
        let generator = ElkCompleteV2Generator::new();
        let model = SemanticModel::default();
        let result = generator.generate(&model).expect("generation should succeed");

        assert_eq!(result.semantic.diagram_type, "functional");
        let nodes = result.diagram_data.get("nodes").and_then(|n| n.as_array()).unwrap();
        assert!(nodes.is_empty());
    }
}
