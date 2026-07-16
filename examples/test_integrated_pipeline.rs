// Test Integrated Rendering Pipeline (Phase 1 & 2)
// Demonstrates the complete pipeline with all modules working together

use arclang::compiler::ast::*;
use arclang::compiler::elk_complete_v2_generator::ElkCompleteV2Generator;

fn create_test_model() -> Model {
    let mut model = Model::new();
    
    // Add logical architecture with components
    model.logical_architecture.push(LogicalArchitecture {
        name: "Emergency Braking System".to_string(),
        components: vec![
            LogicalComponent {
                id: "LC-001".to_string(),
                name: "Radar".to_string(),
                component_type: "sensor".to_string(),
                color: None,
                sub_components: vec![],
                allocated_functions: vec![],
                ports: vec![],
                functions: vec![],
                interfaces_in: vec![],
                interfaces_out: vec![
                    InterfaceDefinition {
                        name: "IObstacleData".to_string(),
                        protocol: Some("CAN".to_string()),
                        format: None,
                        attributes: std::collections::HashMap::new(),
                    }
                ],
                attributes: {
                    let mut attrs = std::collections::HashMap::new();
                    attrs.insert(
                        "stereotype".to_string(),
                        AttributeValue::String("<<sensor>>".to_string())
                    );
                    attrs.insert(
                        "safety_level".to_string(),
                        AttributeValue::String("ASIL_D".to_string())
                    );
                    attrs
                },
            },
            LogicalComponent {
                id: "LC-002".to_string(),
                name: "BrakingController".to_string(),
                component_type: "controller".to_string(),
                color: None,
                sub_components: vec![],
                allocated_functions: vec![],
                ports: vec![],
                functions: vec![],
                interfaces_in: vec![
                    InterfaceDefinition {
                        name: "IObstacleData".to_string(),
                        protocol: Some("CAN".to_string()),
                        format: None,
                        attributes: std::collections::HashMap::new(),
                    }
                ],
                interfaces_out: vec![
                    InterfaceDefinition {
                        name: "IBrakingCommand".to_string(),
                        protocol: Some("CAN".to_string()),
                        format: None,
                        attributes: std::collections::HashMap::new(),
                    }
                ],
                attributes: {
                    let mut attrs = std::collections::HashMap::new();
                    attrs.insert(
                        "stereotype".to_string(),
                        AttributeValue::String("<<controller>>".to_string())
                    );
                    attrs.insert(
                        "safety_level".to_string(),
                        AttributeValue::String("ASIL_D".to_string())
                    );
                    attrs
                },
            },
            LogicalComponent {
                id: "LC-003".to_string(),
                name: "BrakeActuator".to_string(),
                component_type: "actuator".to_string(),
                color: None,
                sub_components: vec![],
                allocated_functions: vec![],
                ports: vec![],
                functions: vec![],
                interfaces_in: vec![
                    InterfaceDefinition {
                        name: "IBrakingCommand".to_string(),
                        protocol: Some("CAN".to_string()),
                        format: None,
                        attributes: std::collections::HashMap::new(),
                    }
                ],
                interfaces_out: vec![],
                attributes: {
                    let mut attrs = std::collections::HashMap::new();
                    attrs.insert(
                        "stereotype".to_string(),
                        AttributeValue::String("<<actuator>>".to_string())
                    );
                    attrs.insert(
                        "safety_level".to_string(),
                        AttributeValue::String("ASIL_D".to_string())
                    );
                    attrs
                },
            },
        ],
        component_exchanges: vec![
            ComponentExchange {
                from_port: "LC-001".to_string(),
                to_port: "LC-002".to_string(),
                exchange_item: "ObstacleData".to_string(),
                label: Some("obstacle detection".to_string()),
            },
            ComponentExchange {
                from_port: "LC-002".to_string(),
                to_port: "LC-003".to_string(),
                exchange_item: "BrakingCommand".to_string(),
                label: Some("brake command".to_string()),
            },
        ],
        interfaces: vec![],
        unallocated_functions: vec![],
    });
    
    model
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  ArcLang Rendering Pipeline Integration Test                    ║");
    println!("║  Phase 1 & 2: Semantic → Strategy → Rules → Style → Quality    ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    
    // Create test model
    println!("📝 Creating test model...");
    let model = create_test_model();
    println!("   ✓ Created Emergency Braking System");
    println!("   ✓ 3 components: Radar, Controller, Actuator");
    println!("   ✓ 2 interfaces with safety levels (ASIL-D)");
    println!();
    
    // Generate with integrated pipeline
    println!("🚀 Generating with integrated pipeline...");
    println!();
    
    let generator = ElkCompleteV2Generator::new();

    // Single derivation: build the canonical semantic model once, then
    // feed it to the generator (no AST re-analysis inside).
    let semantic_model = arclang::compiler::semantic::SemanticAnalyzer::new()
        .analyze(&model)
        .expect("semantic analysis failed");

    match generator.generate(&semantic_model) {
        Ok(result) => {
            println!();
            println!("╔══════════════════════════════════════════════════════════════════╗");
            println!("║  INTEGRATION TEST: SUCCESS ✅                                   ║");
            println!("╚══════════════════════════════════════════════════════════════════╝");
            println!();
            
            println!("📊 Results Summary:");
            println!("   Phase: {:?}", result.semantic.phase);
            println!("   Strategy: {:?}", result.semantic.recommended_strategy);
            println!("   Elements: {}", result.semantic.elements.len());
            println!();
            
            if let Some(ref quality) = result.quality_report {
                println!("📈 Quality Report:");
                println!("   Overall Score: {:.1}/10", quality.overall_score);
                println!("   Edge Crossings: {}", quality.edge_crossings);
                println!("   Node Overlaps: {}", quality.node_overlaps);
                println!("   Arcadia Compliance: {:.0}%", quality.arcadia_compliance);
                
                if quality.overall_score >= 7.0 {
                    println!("   ✅ HIGH QUALITY (≥7/10)");
                } else if quality.overall_score >= 5.0 {
                    println!("   ⚠️  MEDIUM QUALITY (5-7/10)");
                } else {
                    println!("   ❌ LOW QUALITY (<5/10)");
                }
                println!();
            }
            
            if let Some(ref rules) = result.rules_result {
                println!("📋 Arcadia Rules:");
                println!("   Applied: {}", rules.rules_applied);
                println!("   Passed: {}", rules.rules_passed);
                println!("   Failed: {}", rules.rules_failed);
                println!();
            }
            
            println!("💾 Saving HTML output...");
            let html = result.to_html();
            std::fs::write("test-output-integrated-pipeline.html", html)
                .expect("Failed to write HTML");
            println!("   ✓ Saved to: test-output-integrated-pipeline.html");
            println!();
            
            println!("🎉 Integration test completed successfully!");
            println!();
            println!("📖 What to check:");
            println!("   1. Open test-output-integrated-pipeline.html in browser");
            println!("   2. Verify quality report in top-right corner");
            println!("   3. Check diagram has Capella colors (green/blue/orange)");
            println!("   4. Look for safety indicators (red borders)");
            println!("   5. Verify grid-snapped alignment");
            println!();
        }
        Err(e) => {
            println!();
            println!("╔══════════════════════════════════════════════════════════════════╗");
            println!("║  INTEGRATION TEST: FAILED ❌                                    ║");
            println!("╚══════════════════════════════════════════════════════════════════╝");
            println!();
            println!("Error: {}", e);
            println!();
            std::process::exit(1);
        }
    }
}
