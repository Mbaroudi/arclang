use arclang::compiler::lexer::{Lexer, Token};

#[test]
fn test_complete_acc_model_tokenization() {
    let input = r#"
    model AdaptiveCruiseControl {
        metadata {
            name: "Adaptive Cruise Control System"
            version: "1.0.0"
            author: "System Architect"
            description: "ASIL-B compliant adaptive cruise control system for automotive applications"
            safety_standard: "ISO 26262"
        }

        requirements stakeholder {
            req STK-001 "Adaptive Speed Control" {
                description: "The system shall maintain vehicle speed at driver-set target while adapting to traffic conditions"
                priority: Critical
                safety_level: ASIL_B
                rationale: "Core ACC functionality for safe highway driving"
            }
        }

        requirements system {
            req SYS-001 "Target Speed Control" {
                description: "System shall control vehicle speed to match driver-set target speed ±2 km/h"
                priority: Critical
                safety_level: ASIL_B
                traces: [STK-001]
                verification: "Vehicle speed measurement and control accuracy test"
            }
        }

        architecture logical {
            component SensingSubsystem "Forward Sensing Subsystem" {
                description: "Detects and tracks objects in vehicle forward path"
                safety_level: ASIL_B
                
                provides interface IObjectDetection {
                    description: "Provides detected object data"
                    signals: [
                        "ObjectDistance: Real (m)",
                        "ObjectRelativeSpeed: Real (m/s)",
                        "DetectionConfidence: Integer (0-100%)",
                        "SensorStatus: Enum {OK, DEGRADED, FAILED}"
                    ]
                }
            }

            component ControllerSubsystem "ACC Control Subsystem" {
                description: "Main adaptive cruise control logic"
                safety_level: ASIL_B
                
                requires interface IObjectDetection
            }

            connect SensingSubsystem.IObjectDetection -> ControllerSubsystem
        }

        architecture physical {
            component RadarECU "Radar Electronic Control Unit" {
                description: "77GHz radar processing unit (Continental ARS540)"
                implements: [RadarSensor]
                properties: {
                    "Processor": "Infineon AURIX TC397",
                    "PowerConsumption": "8W",
                    "OperatingTemp": "-40°C to 85°C",
                    "CANBusSpeed": "500 kbps"
                }
            }

            connect RadarECU -> ACCMainECU via "CAN Bus (500 kbps)"
        }

        scenarios {
            scenario NormalFollowing "Following Lead Vehicle" {
                description: "ACC maintains safe distance behind slower vehicle"
                precondition: "ACC active, lead vehicle detected ahead, ego speed > 30 km/h"
                steps: [
                    "RadarSensor detects vehicle 80m ahead traveling at 80 km/h",
                    "DistanceController commands 0.15g deceleration",
                    "Vehicle decelerates smoothly to maintain 2.0s time gap"
                ]
                postcondition: "Safe 2.0s time gap maintained"
                traces: [SYS-001, SYS-003, SYS-004]
            }
        }

        traceability {
            trace STK-001 -> [SYS-001, SYS-005]
            trace SYS-001 -> [SpeedController, ControllerSubsystem]
            trace RadarSensor -> [RadarECU]
        }
    }
    "#;
    
    let result = Lexer::new(input).tokenize();
    assert!(result.is_ok(), "Complete ACC model should tokenize without errors");
    
    let tokens = result.unwrap();
    
    // Verify critical tokens are present
    assert!(tokens.iter().any(|t| matches!(t, Token::Model)), "Should have 'model' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Metadata)), "Should have 'metadata' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Version)), "Should have 'version' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Requirements)), "Should have 'requirements' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Stakeholder)), "Should have 'stakeholder' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Architecture)), "Should have 'architecture' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Logical)), "Should have 'logical' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Physical)), "Should have 'physical' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Provides)), "Should have 'provides' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Requires)), "Should have 'requires' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Connect)), "Should have 'connect' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Via)), "Should have 'via' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Scenarios)), "Should have 'scenarios' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Scenario)), "Should have 'scenario' keyword");
    assert!(tokens.iter().any(|t| matches!(t, Token::Dot)), "Should have dot tokens for Component.Interface");
    assert!(tokens.iter().any(|t| matches!(t, Token::Arrow)), "Should have arrow tokens for connections");
    
    // Verify strings with special characters work
    assert!(tokens.iter().any(|t| {
        if let Token::StringLiteral(s) = t {
            s.contains("1.0.0") || s.contains("ISO 26262") || s.contains("±2 km/h") || 
            s.contains("-40°C to 85°C") || s.contains("0.15g") || s.contains("(500 kbps)")
        } else {
            false
        }
    }), "Should handle strings with decimals, special chars, and technical notation");
    
    println!("✓ Successfully tokenized {} tokens from complete ACC model", tokens.len());
}

#[test]
fn test_decimal_values_in_strings() {
    let input = r#"
    version: "1.0.0"
    description: "System shall control vehicle speed ±2 km/h"
    temp_range: "-40°C to 85°C"
    decel: "0.15g deceleration"
    gap: "Time gap (1.0s, 1.5s, 2.0s)"
    coverage: ">90% diagnostic coverage"
    speed: "speed > 30 km/h"
    "#;
    
    let result = Lexer::new(input).tokenize();
    assert!(result.is_ok(), "Should handle all decimal and special chars in strings");
    
    let tokens = result.unwrap();
    
    // Find string literals and verify they contain the values
    let string_literals: Vec<String> = tokens.iter()
        .filter_map(|t| if let Token::StringLiteral(s) = t { Some(s.clone()) } else { None })
        .collect();
    
    assert!(string_literals.iter().any(|s| s.contains("1.0.0")), "Should have version string");
    assert!(string_literals.iter().any(|s| s.contains("±2 km/h")), "Should have plus-minus symbol");
    assert!(string_literals.iter().any(|s| s.contains("-40°C")), "Should have degree symbol");
    assert!(string_literals.iter().any(|s| s.contains("0.15g")), "Should have decimal in technical value");
    assert!(string_literals.iter().any(|s| s.contains("(1.0s")), "Should have decimals in parentheses");
    assert!(string_literals.iter().any(|s| s.contains(">90%")), "Should have percentage");
    assert!(string_literals.iter().any(|s| s.contains("> 30")), "Should have greater-than symbol");
}

#[test]
fn test_component_dot_interface_connections() {
    let input = r#"
    connect SensingSubsystem.IObjectDetection -> ControllerSubsystem
    connect ControllerSubsystem.IVehicleCommands -> ActuationSubsystem
    connect RadarECU -> ACCMainECU via "CAN Bus (500 kbps)"
    "#;
    
    let result = Lexer::new(input).tokenize();
    assert!(result.is_ok(), "Should tokenize component.interface syntax");
    
    let tokens = result.unwrap();
    
    // Count dots and arrows
    let dot_count = tokens.iter().filter(|t| matches!(t, Token::Dot)).count();
    let arrow_count = tokens.iter().filter(|t| matches!(t, Token::Arrow)).count();
    
    assert!(dot_count >= 2, "Should have at least 2 dots for Component.Interface");
    assert_eq!(arrow_count, 3, "Should have exactly 3 arrows for connections");
    
    // Verify 'via' keyword present
    assert!(tokens.iter().any(|t| matches!(t, Token::Via)), "Should have 'via' keyword");
}

#[test]
fn test_properties_block() {
    let input = r#"
    properties: {
        "Processor": "Infineon AURIX TC397",
        "PowerConsumption": "8W",
        "OperatingTemp": "-40°C to 85°C",
        "CANBusSpeed": "500 kbps",
        "Memory": "4MB Flash, 512KB RAM"
    }
    "#;
    
    let result = Lexer::new(input).tokenize();
    assert!(result.is_ok(), "Should handle properties block with technical values");
}

#[test]
fn test_safety_level_identifiers() {
    let input = r#"
    safety_level: ASIL_B
    safety_level: ASIL_D
    priority: Critical
    priority: High
    "#;
    
    let result = Lexer::new(input).tokenize();
    assert!(result.is_ok(), "Should handle safety level and priority identifiers");
    
    let tokens = result.unwrap();
    
    // Verify identifiers are correctly parsed
    assert!(tokens.iter().any(|t| {
        if let Token::Identifier(s) = t { s == "ASIL_B" || s == "ASIL_D" } else { false }
    }), "Should have ASIL level identifiers");
    
    assert!(tokens.iter().any(|t| {
        if let Token::Identifier(s) = t { s == "Critical" || s == "High" } else { false }
    }), "Should have priority identifiers");
}
