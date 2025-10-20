use arclang::compiler::lexer::{Lexer, Token};

#[test]
fn test_version_string_with_decimals() {
    let input = r#"version: "1.0.0""#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Version);
    assert_eq!(tokens[1], Token::Colon);
    assert_eq!(tokens[2], Token::StringLiteral("1.0.0".to_string()));
    assert_eq!(tokens[3], Token::Eof);
}

#[test]
fn test_decimal_in_technical_string() {
    let input = r#"description: "0.5m accuracy""#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Description);
    assert_eq!(tokens[2], Token::StringLiteral("0.5m accuracy".to_string()));
}

#[test]
fn test_safety_standard_with_space() {
    let input = r#"safety_standard: "ISO 26262""#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    // safety_standard is actually Identifier, not a keyword (it's with underscore)
    assert_eq!(tokens[0], Token::Identifier("safety_standard".to_string()));
    assert_eq!(tokens[2], Token::StringLiteral("ISO 26262".to_string()));
}

#[test]
fn test_component_dot_notation() {
    let input = "Component.Interface";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Identifier("Component".to_string()));
    assert_eq!(tokens[1], Token::Dot);
    assert_eq!(tokens[2], Token::Identifier("Interface".to_string()));
    assert_eq!(tokens[3], Token::Eof);
}

#[test]
fn test_arrow_operator() {
    let input = "A -> B";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Identifier("A".to_string()));
    assert_eq!(tokens[1], Token::Arrow);
    assert_eq!(tokens[2], Token::Identifier("B".to_string()));
}

#[test]
fn test_connect_with_arrow() {
    let input = "connect SensorA.Output -> ControllerB.Input";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Connect);
    assert_eq!(tokens[1], Token::Identifier("SensorA".to_string()));
    assert_eq!(tokens[2], Token::Dot);
    assert_eq!(tokens[3], Token::Identifier("Output".to_string()));
    assert_eq!(tokens[4], Token::Arrow);
    assert_eq!(tokens[5], Token::Identifier("ControllerB".to_string()));
}

#[test]
fn test_number_with_underscores() {
    let input = "speed: 1_000_000";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[2], Token::Number(1000000.0));
}

#[test]
fn test_decimal_number() {
    let input = "value: 3.14159";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[2], Token::Number(3.14159));
}

#[test]
fn test_negative_number() {
    let input = "temp: -40";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[2], Token::Number(-40.0));
}

#[test]
fn test_negative_decimal() {
    let input = "offset: -2.5";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[2], Token::Number(-2.5));
}

#[test]
fn test_number_with_underscores_and_decimal() {
    let input = "large: 1_234.567_89";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[2], Token::Number(1234.56789));
}

#[test]
fn test_new_keywords_model() {
    let input = "model MyModel { }";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Model);
    assert_eq!(tokens[1], Token::Identifier("MyModel".to_string()));
}

#[test]
fn test_new_keywords_metadata() {
    let input = "metadata { version: \"1.0\" }";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Metadata);
    assert_eq!(tokens[2], Token::Version);
}

#[test]
fn test_new_keywords_requirements() {
    let input = "requirements stakeholder { }";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Requirements);
    assert_eq!(tokens[1], Token::Stakeholder);
}

#[test]
fn test_new_keywords_architecture() {
    let input = "architecture logical { }";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Architecture);
    assert_eq!(tokens[1], Token::Logical);
}

#[test]
fn test_new_keywords_provides_requires() {
    let input = "provides interface IData { } requires interface IControl { }";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Provides);
    assert_eq!(tokens[1], Token::Interface);
    // Count tokens: provides interface IData { } requires ...
    // 0:provides 1:interface 2:IData 3:{ 4:} 5:requires
    assert_eq!(tokens[5], Token::Requires);
    assert_eq!(tokens[6], Token::Interface);
}

#[test]
fn test_new_keywords_scenarios() {
    let input = "scenarios { scenario Test { steps: [] } }";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Scenarios);
    assert_eq!(tokens[2], Token::Scenario);
    assert_eq!(tokens[4], Token::LeftBrace);
    assert_eq!(tokens[5], Token::Steps);
}

#[test]
fn test_complex_technical_string() {
    let input = r#"description: "Range: 30-180 km/h, Temp: -40°C to 85°C""#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Description);
    assert_eq!(tokens[2], Token::StringLiteral("Range: 30-180 km/h, Temp: -40°C to 85°C".to_string()));
}

#[test]
fn test_percentage_in_string() {
    let input = r#"coverage: "95% diagnostic coverage""#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert!(matches!(tokens[2], Token::StringLiteral(_)));
}

#[test]
fn test_parentheses_in_string() {
    let input = r#"desc: "Time gap (1.0s, 1.5s, 2.0s)""#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[2], Token::StringLiteral("Time gap (1.0s, 1.5s, 2.0s)".to_string()));
}

#[test]
fn test_via_keyword_in_connection() {
    let input = r#"connect A -> B via "CAN Bus""#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    // 0:connect 1:A 2:-> 3:B 4:via 5:"CAN Bus"
    assert_eq!(tokens[0], Token::Connect);
    assert_eq!(tokens[1], Token::Identifier("A".to_string()));
    assert_eq!(tokens[2], Token::Arrow);
    assert_eq!(tokens[3], Token::Identifier("B".to_string()));
    assert_eq!(tokens[4], Token::Via);
    assert_eq!(tokens[5], Token::StringLiteral("CAN Bus".to_string()));
}

#[test]
fn test_properties_keyword() {
    let input = "properties { power: 5W }";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Properties);
}

#[test]
fn test_parent_keyword() {
    let input = "parent: SensingSubsystem";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Parent);
}

#[test]
fn test_precondition_postcondition() {
    let input = "precondition: \"active\" postcondition: \"complete\"";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Precondition);
    assert_eq!(tokens[3], Token::Postcondition);
}

#[test]
fn test_traces_verification_rationale() {
    let input = "traces: [REQ-001] verification: \"test\" rationale: \"reason\"";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    // Find the token positions
    assert_eq!(tokens[0], Token::Traces);
    assert_eq!(tokens[1], Token::Colon);
    assert_eq!(tokens[2], Token::LeftBracket);
    // REQ is Identifier, then Minus, then 001 is Number
    // Then ] then verification
    assert!(tokens.iter().any(|t| matches!(t, Token::Verification)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Rationale)));
}

#[test]
fn test_minus_not_part_of_identifier() {
    let input = "test-case";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    // Should tokenize as: Identifier("test"), Minus, Identifier("case")
    assert_eq!(tokens[0], Token::Identifier("test".to_string()));
    assert_eq!(tokens[1], Token::Minus);
    assert_eq!(tokens[2], Token::Identifier("case".to_string()));
}

#[test]
fn test_full_model_structure() {
    let input = r#"
    model AdaptiveCruiseControl {
        metadata {
            version: "1.0.0"
            author: "Engineer"
        }
        
        requirements stakeholder {
            req REQ-001 "Control Speed" {
                priority: Critical
                safety_level: ASIL_B
            }
        }
        
        architecture logical {
            component Radar {
                provides interface IData {
                    signals: ["distance", "speed"]
                }
            }
            
            connect Radar.IData -> Controller
        }
    }
    "#;
    
    let result = Lexer::new(input).tokenize();
    assert!(result.is_ok(), "Full model should tokenize successfully");
    
    let tokens = result.unwrap();
    assert!(tokens.len() > 50, "Should have many tokens");
    
    // Verify key tokens exist
    assert!(tokens.iter().any(|t| matches!(t, Token::Model)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Metadata)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Version)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Requirements)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Architecture)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Connect)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Dot)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Arrow)));
}

#[test]
fn test_edge_case_dot_at_end_of_number() {
    // This should parse as number, not number + dot
    let input = "value: 42.";
    let result = Lexer::new(input).tokenize();
    
    // Should either fail or parse as 42 followed by Dot
    assert!(result.is_err() || {
        let tokens = result.unwrap();
        tokens[2] == Token::Number(42.0) && tokens[3] == Token::Dot
    });
}

#[test]
fn test_multiple_dots_in_path() {
    let input = "A.B.C.D";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[0], Token::Identifier("A".to_string()));
    assert_eq!(tokens[1], Token::Dot);
    assert_eq!(tokens[2], Token::Identifier("B".to_string()));
    assert_eq!(tokens[3], Token::Dot);
    assert_eq!(tokens[4], Token::Identifier("C".to_string()));
    assert_eq!(tokens[5], Token::Dot);
    assert_eq!(tokens[6], Token::Identifier("D".to_string()));
}

#[test]
fn test_arrow_not_confused_with_minus() {
    let input = "a - b";
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[1], Token::Minus);
    assert!(tokens[1] != Token::Arrow);
}

#[test]
fn test_comment_preservation() {
    let input = r#"
    // This is a comment
    model Test {
        /* Multi-line
           comment */
        version: "1.0"
    }
    "#;
    
    let result = Lexer::new(input).tokenize();
    assert!(result.is_ok());
}

#[test]
fn test_empty_string() {
    let input = r#"name: """#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    assert_eq!(tokens[2], Token::StringLiteral("".to_string()));
}

#[test]
fn test_escaped_characters_in_string() {
    let input = r#"text: "Line 1\nLine 2\tTabbed""#;
    let tokens = Lexer::new(input).tokenize().unwrap();
    
    let expected = "Line 1\nLine 2\tTabbed";
    assert_eq!(tokens[2], Token::StringLiteral(expected.to_string()));
}
