use super::ast::*;
use super::lexer::Token;
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }
    
    pub fn parse(mut self) -> Result<Model, String> {
        let mut model = Model::new();
        
        while !self.is_at_end() {
            match self.current() {
                Token::Model => {
                    // Parse new-style model block
                    return self.parse_model_block();
                }
                Token::OperationalAnalysis => {
                    model.operational_analysis.push(self.parse_operational_analysis()?);
                }
                Token::SystemAnalysis => {
                    model.system_analysis.push(self.parse_system_analysis()?);
                }
                Token::LogicalArchitecture => {
                    model.logical_architecture.push(self.parse_logical_architecture()?);
                }
                Token::PhysicalArchitecture => {
                    model.physical_architecture.push(self.parse_physical_architecture()?);
                }
                Token::Epbs => {
                    model.epbs.push(self.parse_epbs()?);
                }
                Token::SafetyAnalysis => {
                    model.safety_analysis.push(self.parse_safety_analysis()?);
                }
                Token::Trace => {
                    model.traces.push(self.parse_trace()?);
                }
                Token::Eof => break,
                _ => return Err(format!("Unexpected token: {}", self.current())),
            }
        }
        
        Ok(model)
    }
    
    fn parse_model_block(&mut self) -> Result<Model, String> {
        self.expect(Token::Model)?;
        
        // Model name (identifier)
        let _model_name = self.expect_identifier()?;
        
        self.expect(Token::LeftBrace)?;
        
        let mut model = Model::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Metadata => {
                    // Parse and skip metadata block for now
                    self.advance();
                    self.skip_block()?;
                }
                Token::Requirements => {
                    // Parse requirements block
                    self.advance();
                    match self.current() {
                        Token::Stakeholder | Token::System | Token::Identifier(_) => {
                            self.advance(); // Skip sub-type
                            self.skip_block()?;
                        }
                        _ => {}
                    }
                }
                Token::Architecture => {
                    self.advance();
                    match self.current() {
                        Token::Logical => {
                            self.advance();
                            self.skip_block()?;
                        }
                        Token::Physical => {
                            self.advance();
                            self.skip_block()?;
                        }
                        _ => {}
                    }
                }
                Token::Scenarios => {
                    self.advance();
                    self.skip_block()?;
                }
                Token::Identifier(ref id) if id == "traceability" => {
                    self.advance();
                    self.skip_block()?;
                }
                Token::OperationalAnalysis => {
                    model.operational_analysis.push(self.parse_operational_analysis()?);
                }
                Token::SystemAnalysis => {
                    model.system_analysis.push(self.parse_system_analysis()?);
                }
                Token::LogicalArchitecture => {
                    model.logical_architecture.push(self.parse_logical_architecture()?);
                }
                Token::Eof => break,
                _ => {
                    // Skip unknown tokens within model block
                    self.advance();
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        Ok(model)
    }
    
    fn parse_operational_analysis(&mut self) -> Result<OperationalAnalysis, String> {
        self.expect(Token::OperationalAnalysis)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut actors = Vec::new();
        let mut capabilities = Vec::new();
        let mut activities = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Actor => {
                    actors.push(self.parse_actor()?);
                }
                Token::Identifier(ref id) if id == "operational_capability" => {
                    capabilities.push(self.parse_operational_capability()?);
                }
                Token::Identifier(ref id) if id == "operational_activity" => {
                    activities.push(self.parse_operational_activity()?);
                }
                _ => {
                    return Err(format!("Unexpected token in operational_analysis: {}", self.current()));
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(OperationalAnalysis {
            name,
            actors,
            capabilities,
            activities,
        })
    }
    
    fn parse_actor(&mut self) -> Result<Actor, String> {
        self.expect(Token::Actor)?;
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(Actor { name, attributes })
    }
    
    fn parse_operational_capability(&mut self) -> Result<OperationalCapability, String> {
        self.advance(); // Skip 'operational_capability'
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(OperationalCapability { name, attributes })
    }
    
    fn parse_operational_activity(&mut self) -> Result<OperationalActivity, String> {
        self.advance(); // Skip 'operational_activity'
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(OperationalActivity { name, attributes })
    }
    
    fn parse_system_analysis(&mut self) -> Result<SystemAnalysis, String> {
        self.expect(Token::SystemAnalysis)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut requirements = Vec::new();
        let mut functions = Vec::new();
        let mut components = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Requirement => {
                    requirements.push(self.parse_requirement()?);
                }
                Token::Identifier(ref id) if id == "system_function" => {
                    functions.push(self.parse_system_function()?);
                }
                Token::Identifier(ref id) if id == "system_component" => {
                    components.push(self.parse_system_component()?);
                }
                _ => {
                    return Err(format!("Unexpected token in system_analysis: {}", self.current()));
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(SystemAnalysis {
            name,
            requirements,
            functions,
            components,
        })
    }
    
    fn parse_requirement(&mut self) -> Result<Requirement, String> {
        self.expect(Token::Requirement)?;
        let id = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(Requirement { id, attributes })
    }
    
    fn parse_system_function(&mut self) -> Result<SystemFunction, String> {
        self.advance(); // Skip 'system_function'
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(SystemFunction { name, attributes })
    }
    
    fn parse_system_component(&mut self) -> Result<SystemComponent, String> {
        self.advance(); // Skip 'system_component'
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(SystemComponent { name, attributes })
    }
    
    fn parse_logical_architecture(&mut self) -> Result<LogicalArchitecture, String> {
        self.expect(Token::LogicalArchitecture)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut components = Vec::new();
        let mut interfaces = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Component => {
                    components.push(self.parse_logical_component()?);
                }
                Token::Interface => {
                    interfaces.push(self.parse_logical_interface()?);
                }
                Token::Trace => {
                    // Skip traces for now, they're collected at model level
                    self.skip_block()?;
                }
                _ => {
                    return Err(format!("Unexpected token in logical_architecture: {}", self.current()));
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(LogicalArchitecture {
            name,
            components,
            interfaces,
        })
    }
    
    fn parse_logical_component(&mut self) -> Result<LogicalComponent, String> {
        self.expect(Token::Component)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut functions = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Function => {
                    functions.push(self.parse_logical_function()?);
                }
                Token::Identifier(_) | Token::Description | Token::Version | Token::Author |
                Token::Priority | Token::Rationale | Token::Verification | Token::Traces |
                Token::SafetyLevel | Token::Parent | Token::Properties | Token::Signals |
                Token::Provides | Token::Requires => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => {
                    // Skip unknown tokens in component
                    self.advance();
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(LogicalComponent {
            name,
            functions,
            attributes,
        })
    }
    
    fn parse_logical_function(&mut self) -> Result<LogicalFunction, String> {
        self.expect(Token::Function)?;
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(LogicalFunction { name, attributes })
    }
    
    fn parse_logical_interface(&mut self) -> Result<LogicalInterface, String> {
        self.expect(Token::Interface)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();
        let mut from = String::new();
        let mut to = String::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Token::Identifier(ref id) = self.current() {
                if id == "from" {
                    self.advance();
                    self.expect(Token::Colon)?;
                    from = self.expect_string()?;
                } else if id == "to" {
                    self.advance();
                    self.expect(Token::Colon)?;
                    to = self.expect_string()?;
                } else {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
            } else {
                break;
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(LogicalInterface {
            name,
            from,
            to,
            attributes,
        })
    }
    
    fn parse_physical_architecture(&mut self) -> Result<PhysicalArchitecture, String> {
        self.expect(Token::PhysicalArchitecture)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Node => {
                    nodes.push(self.parse_physical_node()?);
                }
                Token::Identifier(ref id) if id == "physical_link" => {
                    links.push(self.parse_physical_link()?);
                }
                _ => {
                    return Err(format!("Unexpected token in physical_architecture: {}", self.current()));
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(PhysicalArchitecture { name, nodes, links })
    }
    
    fn parse_physical_node(&mut self) -> Result<PhysicalNode, String> {
        self.expect(Token::Node)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut deployments = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Token::Deploys = self.current() {
                deployments.push(self.parse_deployment()?);
            } else if let Token::Identifier(_) = self.current() {
                let (key, value) = self.parse_attribute()?;
                attributes.insert(key, value);
            } else {
                break;
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(PhysicalNode {
            name,
            deployments,
            attributes,
        })
    }
    
    fn parse_deployment(&mut self) -> Result<Deployment, String> {
        self.expect(Token::Deploys)?;
        let component = self.expect_string()?;
        let attributes = if self.check(&Token::LeftBrace) {
            self.parse_attributes_block()?
        } else {
            HashMap::new()
        };
        
        Ok(Deployment {
            component,
            attributes,
        })
    }
    
    fn parse_physical_link(&mut self) -> Result<PhysicalLink, String> {
        self.advance(); // Skip 'physical_link'
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        let connections = if let Some(AttributeValue::List(ref connects)) = attributes.get("connects") {
            connects.iter()
                .filter_map(|v| v.as_string().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        };
        
        Ok(PhysicalLink {
            name,
            connections,
            attributes,
        })
    }
    
    fn parse_epbs(&mut self) -> Result<Epbs, String> {
        self.expect(Token::Epbs)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut systems = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Token::System = self.current() {
                systems.push(self.parse_epbs_system()?);
            } else {
                return Err(format!("Unexpected token in epbs: {}", self.current()));
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(Epbs { name, systems })
    }
    
    fn parse_epbs_system(&mut self) -> Result<EpbsSystem, String> {
        self.expect(Token::System)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut subsystems = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Subsystem => {
                    subsystems.push(self.parse_epbs_subsystem()?);
                }
                Token::Identifier(_) => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => break,
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(EpbsSystem {
            name,
            subsystems,
            attributes,
        })
    }
    
    fn parse_epbs_subsystem(&mut self) -> Result<EpbsSubsystem, String> {
        self.expect(Token::Subsystem)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut items = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Item => {
                    items.push(self.parse_epbs_item()?);
                }
                Token::Identifier(_) => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => break,
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(EpbsSubsystem {
            name,
            items,
            attributes,
        })
    }
    
    fn parse_epbs_item(&mut self) -> Result<EpbsItem, String> {
        self.expect(Token::Item)?;
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(EpbsItem { name, attributes })
    }
    
    fn parse_safety_analysis(&mut self) -> Result<SafetyAnalysis, String> {
        self.expect(Token::SafetyAnalysis)?;
        self.expect(Token::LeftBrace)?;
        
        let mut hazards = Vec::new();
        let mut fmea = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Hazard => {
                    hazards.push(self.parse_hazard()?);
                }
                Token::Fmea => {
                    fmea.push(self.parse_fmea_entry()?);
                }
                Token::Identifier(_) => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => break,
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(SafetyAnalysis {
            hazards,
            fmea,
            attributes,
        })
    }
    
    fn parse_hazard(&mut self) -> Result<Hazard, String> {
        self.expect(Token::Hazard)?;
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(Hazard { name, attributes })
    }
    
    fn parse_fmea_entry(&mut self) -> Result<FmeaEntry, String> {
        self.expect(Token::Fmea)?;
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(FmeaEntry { name, attributes })
    }
    
    fn parse_trace(&mut self) -> Result<Trace, String> {
        self.expect(Token::Trace)?;
        let from = self.expect_string()?;
        
        let trace_type = if let Token::Satisfies = self.current() {
            self.advance();
            "satisfies".to_string()
        } else if let Token::Implements = self.current() {
            self.advance();
            "implements".to_string()
        } else {
            return Err("Expected trace type (satisfies, implements, etc.)".to_string());
        };
        
        let to = self.expect_string()?;
        let attributes = if self.check(&Token::LeftBrace) {
            self.parse_attributes_block()?
        } else {
            HashMap::new()
        };
        
        Ok(Trace {
            from,
            to,
            trace_type,
            attributes,
        })
    }
    
    fn parse_attributes_block(&mut self) -> Result<HashMap<String, AttributeValue>, String> {
        self.expect(Token::LeftBrace)?;
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let (key, value) = self.parse_attribute()?;
            attributes.insert(key, value);
        }
        
        self.expect(Token::RightBrace)?;
        Ok(attributes)
    }
    
    fn parse_attribute(&mut self) -> Result<(String, AttributeValue), String> {
        let key = self.expect_identifier()?;
        self.expect(Token::Colon)?;
        let value = self.parse_attribute_value()?;
        
        Ok((key, value))
    }
    
    fn parse_attribute_value(&mut self) -> Result<AttributeValue, String> {
        match self.current() {
            Token::StringLiteral(_) => {
                let s = self.expect_string()?;
                Ok(AttributeValue::String(s))
            }
            Token::Number(_) => {
                if let Token::Number(n) = self.current().clone() {
                    self.advance();
                    Ok(AttributeValue::Number(n))
                } else {
                    unreachable!()
                }
            }
            Token::LeftBracket => self.parse_list(),
            Token::Identifier(ref id) => {
                let value = id.clone();
                self.advance();
                Ok(AttributeValue::String(value))
            }
            _ => Err(format!("Expected attribute value, got: {}", self.current())),
        }
    }
    
    fn parse_list(&mut self) -> Result<AttributeValue, String> {
        self.expect(Token::LeftBracket)?;
        let mut list = Vec::new();
        
        while !self.check(&Token::RightBracket) && !self.is_at_end() {
            list.push(self.parse_attribute_value()?);
            
            if self.check(&Token::Comma) {
                self.advance();
            } else if !self.check(&Token::RightBracket) {
                return Err("Expected comma or closing bracket in list".to_string());
            }
        }
        
        self.expect(Token::RightBracket)?;
        Ok(AttributeValue::List(list))
    }
    
    fn skip_block(&mut self) -> Result<(), String> {
        // Expect opening brace first
        self.expect(Token::LeftBrace)?;
        
        let mut depth = 1;  // Start at 1 since we just consumed opening brace
        
        while !self.is_at_end() && depth > 0 {
            match self.current() {
                Token::LeftBrace => {
                    depth += 1;
                    self.advance();
                }
                Token::RightBrace => {
                    depth -= 1;
                    self.advance();
                }
                _ => self.advance(),
            }
        }
        
        if depth != 0 {
            return Err("Unmatched braces in skip_block".to_string());
        }
        
        Ok(())
    }
    
    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(self.current()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {}", expected, self.current()))
        }
    }
    
    fn expect_string(&mut self) -> Result<String, String> {
        if let Token::StringLiteral(s) = self.current() {
            let result = s.clone();
            self.advance();
            Ok(result)
        } else {
            Err(format!("Expected string literal, got {}", self.current()))
        }
    }
    
    fn expect_identifier(&mut self) -> Result<String, String> {
        match self.current() {
            Token::Identifier(id) => {
                let result = id.clone();
                self.advance();
                Ok(result)
            }
            // Allow keywords as identifiers in attribute context
            Token::Description => { self.advance(); Ok("description".to_string()) }
            Token::Version => { self.advance(); Ok("version".to_string()) }
            Token::Author => { self.advance(); Ok("author".to_string()) }
            Token::Priority => { self.advance(); Ok("priority".to_string()) }
            Token::Rationale => { self.advance(); Ok("rationale".to_string()) }
            Token::Verification => { self.advance(); Ok("verification".to_string()) }
            Token::Traces => { self.advance(); Ok("traces".to_string()) }
            Token::SafetyLevel => { self.advance(); Ok("safety_level".to_string()) }
            Token::Parent => { self.advance(); Ok("parent".to_string()) }
            Token::Properties => { self.advance(); Ok("properties".to_string()) }
            Token::Signals => { self.advance(); Ok("signals".to_string()) }
            _ => Err(format!("Expected identifier, got {}", self.current()))
        }
    }
    
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }
    
    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current()) == std::mem::discriminant(token)
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.current(), Token::Eof)
    }
}
