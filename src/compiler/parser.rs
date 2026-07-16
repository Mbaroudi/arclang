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
                Token::Model | Token::System => {
                    // Parse new-style model/system block
                    return self.parse_model_block();
                }
                Token::Requirements => {
                    // Top-level requirements block (alternative syntax 3)
                    return self.parse_model_with_toplevel_blocks();
                }
                Token::LogicalArchitecture => {
                    // Alternative syntax: logical_architecture without model wrapper
                    model.logical_architecture.push(self.parse_logical_architecture()?);
                }
                Token::PhysicalArchitecture => {
                    // Alternative syntax: physical_architecture without model wrapper
                    model.physical_architecture.push(self.parse_physical_architecture()?);
                }
                Token::OperationalAnalysis => {
                    model.operational_analysis.push(self.parse_operational_analysis()?);
                }
                Token::SystemAnalysis => {
                    model.system_analysis.push(self.parse_system_analysis()?);
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
                Token::Scenario | Token::Dataflow => {
                    // Skip scenario and dataflow blocks for now
                    self.skip_until_brace_balanced()?;
                }
                Token::Eof => break,
                _ => return Err(format!("Unexpected token: {}", self.current())),
            }
        }
        
        Ok(model)
    }
    
    fn parse_model_block(&mut self) -> Result<Model, String> {
        // Support both 'model' and 'system' keywords
        if matches!(self.current(), Token::Model | Token::System) {
            self.advance();
        } else {
            return Err(format!("Expected 'model' or 'system', got {}", self.current()));
        }
        
        // Model name can be either identifier or string
        let _model_name = match self.current() {
            Token::Identifier(id) => {
                let name = id.clone();
                self.advance();
                name
            }
            Token::StringLiteral(s) => {
                let name = s.clone();
                self.advance();
                name
            }
            _ => return Err(format!("Expected model name (identifier or string), got {}", self.current())),
        };
        
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
                    // Parse requirements block with subtype (stakeholder/system/safety)
                    model.system_analysis.push(self.parse_requirements_block()?);
                }
                Token::Architecture => {
                    self.advance();
                    match self.current() {
                        Token::Logical => {
                            self.advance(); // consume 'logical'
                            model.logical_architecture.push(self.parse_logical_architecture_block("Logical Architecture".to_string())?);
                        }
                        Token::Physical => {
                            self.advance(); // consume 'physical'
                            model.physical_architecture.push(self.parse_physical_architecture_block("Physical Architecture".to_string())?);
                        }
                        _ => {
                            // Unknown architecture type (operational, etc.) - skip it
                            self.advance(); // skip the subtype token
                            self.skip_block()?;
                        }
                    }
                }
                Token::LogicalArchitecture => {
                    // Alternative syntax 1: logical_architecture with identifier
                    self.advance();
                    // Check if followed by identifier (name) - skip it
                    if matches!(self.current(), Token::Identifier(_)) {
                        self.advance();
                    }
                    self.skip_block()?;
                }
                Token::PhysicalArchitecture => {
                    // Alternative syntax 1: physical_architecture with identifier
                    self.advance();
                    // Check if followed by identifier (name) - skip it
                    if matches!(self.current(), Token::Identifier(_)) {
                        self.advance();
                    }
                    self.skip_block()?;
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
        
        // Continue parsing top-level blocks after model block
        while !self.is_at_end() {
            match self.current() {
                Token::Requirements => {
                    model.system_analysis.push(self.parse_requirements_block()?);
                }
                Token::Architecture => {
                    self.advance();
                    match self.current() {
                        Token::Logical => {
                            self.advance();
                            model.logical_architecture.push(self.parse_logical_architecture_block("Logical Architecture".to_string())?);
                        }
                        Token::Physical => {
                            self.advance();
                            model.physical_architecture.push(self.parse_physical_architecture_block("Physical Architecture".to_string())?);
                        }
                        _ => {
                            self.advance();
                            self.skip_block()?;
                        }
                    }
                }
                Token::Trace => {
                    model.traces.push(self.parse_trace()?);
                }
                Token::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
        
        Ok(model)
    }
    
    fn parse_model_with_toplevel_blocks(&mut self) -> Result<Model, String> {
        // Alternative syntax 3: model "Name" { } followed by top-level blocks
        let mut model = Model::new();
        
        // Check if we start with model declaration
        let has_model_decl = matches!(self.current(), Token::Model | Token::System);
        if has_model_decl {
            // Parse model declaration
            self.advance(); // Skip 'model' or 'system'
            
            // Model name can be identifier or string
            if matches!(self.current(), Token::Identifier(_) | Token::StringLiteral(_)) {
                self.advance(); // Skip name
            }
            
            // Model attributes block
            if self.check(&Token::LeftBrace) {
                self.skip_block()?; // Skip model metadata/attributes
            }
        }
        
        // Now parse top-level blocks
        while !self.is_at_end() {
            match self.current() {
                Token::Requirements => {
                    // Parse top-level requirements blocks (stakeholder/system/safety)
                    model.system_analysis.push(self.parse_requirements_block()?);
                }
                Token::Architecture => {
                    self.advance();
                    match self.current() {
                        Token::Logical => {
                            self.advance(); // consume 'logical'
                            model.logical_architecture.push(self.parse_logical_architecture_block("Logical Architecture".to_string())?);
                        }
                        Token::Physical => {
                            self.advance(); // consume 'physical'
                            model.physical_architecture.push(self.parse_physical_architecture_block("Physical Architecture".to_string())?);
                        }
                        _ => {
                            // Unknown architecture type (operational, etc.) - skip it
                            self.advance(); // skip the subtype token
                            self.skip_block()?;
                        }
                    }
                }
                Token::LogicalArchitecture => {
                    self.advance();
                    // logical_architecture without string name - skip block content
                    self.skip_block()?;
                }
                Token::PhysicalArchitecture => {
                    self.advance();
                    // physical_architecture without string name - skip block content  
                    self.skip_block()?;
                }
                Token::DataFlows => {
                    self.advance();
                    self.skip_block()?;
                }
                Token::SafetyAnalysis => {
                    self.advance();
                    self.skip_block()?;
                }
                Token::ValidationKeyword => {
                    self.advance();
                    self.skip_block()?;
                }
                Token::Identifier(ref id) if id == "traces" || id == "traceability" => {
                    self.advance();
                    self.skip_block()?;
                }
                Token::Eof => break,
                _ => {
                    self.advance(); // Skip unknown tokens
                }
            }
        }
        
        Ok(model)
    }
    
    fn parse_operational_analysis(&mut self) -> Result<OperationalAnalysis, String> {
        self.expect(Token::OperationalAnalysis)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut actors = Vec::new();
        let mut entities = Vec::new();
        let mut capabilities = Vec::new();
        let mut activities = Vec::new();
        let mut exchanges = Vec::new();
        let mut traces = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Actor => {
                    actors.push(self.parse_actor()?);
                }
                Token::Identifier(ref id) if id == "operational_entity" || id == "entity" => {
                    entities.push(self.parse_operational_entity()?);
                }
                Token::Identifier(ref id) if id == "operational_capability" => {
                    capabilities.push(self.parse_operational_capability()?);
                }
                Token::Identifier(ref id) if id == "operational_activity" => {
                    activities.push(self.parse_operational_activity()?);
                }
                Token::Identifier(ref id) if id == "operational_exchange" || id == "exchange" || id == "interaction" => {
                    exchanges.push(self.parse_operational_exchange()?);
                }
                Token::Trace => {
                    // Parse traces and collect them
                    traces.push(self.parse_trace()?);
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
            entities,
            capabilities,
            activities,
            exchanges,
            capability_associations: Vec::new(),
            traces,
        })
    }
    
    fn parse_operational_entity(&mut self) -> Result<OperationalEntity, String> {
        self.advance(); // Skip 'operational_entity' or 'entity'
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();
        let mut activities = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Identifier(ref id) if id == "operational_activity" || id == "activity" => {
                    activities.push(self.parse_operational_activity()?);
                }
                Token::Identifier(ref key) => {
                    let key = key.clone();
                    self.advance();
                    self.expect(Token::Colon)?;
                    let value = self.parse_attribute_value()?;
                    attributes.insert(key, value);
                }
                _ => {
                    return Err(format!("Unexpected token in operational_entity: {}", self.current()));
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        let id = attributes.get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("OE-{}", name.chars().take(3).collect::<String>()));
        
        Ok(OperationalEntity {
            id,
            name,
            entity_type: EntityType::System,
            activities,
            attributes,
        })
    }
    
    fn parse_actor(&mut self) -> Result<Actor, String> {
        self.expect(Token::Actor)?;
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(Actor {
            name,
            id: None,
            icon: "person".to_string(),
            attributes,
        })
    }
    
    fn parse_operational_capability(&mut self) -> Result<OperationalCapability, String> {
        self.advance(); // Skip 'operational_capability'
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(OperationalCapability {
            id: format!("CAP-{}", name.chars().take(3).collect::<String>()),
            name,
            level: CapabilityLevel::Capability,
            color: None,
            stereotype: None,
            children: Vec::new(),
            attributes,
        })
    }
    
    fn parse_operational_activity(&mut self) -> Result<OperationalActivity, String> {
        self.advance(); // Skip 'operational_activity'
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        // Extract ID from attributes, or generate from name
        let id = attributes.get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("OA-{}", name.chars().take(3).collect::<String>()));
        
        let performed_by = attributes.get("performed_by")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let category = attributes.get("category")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "general".to_string());
        
        Ok(OperationalActivity {
            id,
            name,
            performed_by,
            category,
            icon: "circle".to_string(),
            color: "#FFD966".to_string(),
            sub_activities: Vec::new(),
            attributes,
        })
    }
    
    fn parse_operational_exchange(&mut self) -> Result<OperationalExchange, String> {
        self.advance(); // Skip 'operational_exchange', 'exchange', or 'interaction'
        
        // Parse: "ID1" -> "ID2" { ... }
        let from = self.expect_string()?;
        self.expect(Token::Arrow)?;
        let to = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let (key, value) = self.parse_attribute()?;
            attributes.insert(key, value);
        }
        
        self.expect(Token::RightBrace)?;
        
        let data_type = attributes.get("data_type")
            .and_then(|v| v.as_string())
            .unwrap_or("Data")
            .to_string();
            
        let label = attributes.get("label")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
            
        let protocol = attributes.get("protocol")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        
        Ok(OperationalExchange {
            from,
            to,
            data_type,
            label,
            protocol,
            attributes,
        })
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
            external_actors: Vec::new(),
            functional_exchanges: Vec::new(),
        })
    }
    
    fn parse_requirements_block(&mut self) -> Result<SystemAnalysis, String> {
        self.expect(Token::Requirements)?;
        
        // Get subtype (stakeholder, system, safety, etc.)
        let subtype = match self.current() {
            Token::Stakeholder => "Stakeholder",
            Token::System => "System",
            Token::Identifier(ref id) if id == "safety" => "Safety",
            _ => "Requirements",
        };
        self.advance(); // consume subtype
        
        let name = format!("{} Requirements", subtype);
        self.expect(Token::LeftBrace)?;
        
        let mut requirements = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Req => {
                    requirements.push(self.parse_req_statement()?);
                }
                Token::Requirement => {
                    requirements.push(self.parse_requirement()?);
                }
                _ => {
                    self.advance(); // skip unknown tokens
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(SystemAnalysis {
            name,
            requirements,
            functions: Vec::new(),
            components: Vec::new(),
            external_actors: Vec::new(),
            functional_exchanges: Vec::new(),
        })
    }
    
    fn parse_req_statement(&mut self) -> Result<Requirement, String> {
        self.expect(Token::Req)?;
        let id = self.expect_identifier_or_string()?;
        let title = if matches!(self.current(), Token::StringLiteral(_)) {
            self.expect_string()?
        } else {
            String::new()
        };
        let mut attributes = self.parse_attributes_block()?;
        
        // Add title to attributes if provided
        if !title.is_empty() {
            attributes.insert("title".to_string(), AttributeValue::String(title));
        }
        
        Ok(Requirement { id, attributes })
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
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();
        let mut sub_functions = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Function) {
                let sub_func = self.parse_nested_function()?;
                sub_functions.push(sub_func);
            } else {
                let (key, value) = self.parse_attribute()?;
                attributes.insert(key, value);
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(SystemFunction {
            id: format!("SF-{}", name.chars().take(3).collect::<String>()),
            name,
            category: FunctionCategory::System,
            color: Some("#70AD47".to_string()),
            icon: None,
            ports: Vec::new(),
            sub_functions,
            attributes,
        })
    }
    
    fn parse_nested_function(&mut self) -> Result<SystemFunction, String> {
        self.advance(); // Skip 'function'
        let name = self.expect_string()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(SystemFunction {
            id: format!("SF-{}", name.chars().take(3).collect::<String>()),
            name,
            category: FunctionCategory::System,
            color: Some("#70AD47".to_string()),
            icon: None,
            ports: Vec::new(),
            sub_functions: Vec::new(),
            attributes,
        })
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
        self.parse_logical_architecture_block(name)
    }
    
    fn parse_logical_architecture_block(&mut self, name: String) -> Result<LogicalArchitecture, String> {
        self.expect(Token::LeftBrace)?;
        
        let mut components = Vec::new();
        let mut interfaces = Vec::new();
        let mut component_exchanges = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Component => {
                    components.push(self.parse_logical_component()?);
                }
                Token::Interface => {
                    interfaces.push(self.parse_logical_interface()?);
                }
                Token::Connection => {
                    component_exchanges.push(self.parse_component_exchange()?);
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
            component_exchanges,
            unallocated_functions: Vec::new(),
        })
    }
    
    fn parse_component_exchange(&mut self) -> Result<ComponentExchange, String> {
        self.expect(Token::Connection)?;
        
        let name = self.expect_string()?;
        let mut from = String::new();
        let mut to = String::new();
        
        // Support both syntaxes:
        // 1. connection "from" -> "to" { ... }
        // 2. connection "name" { from: "x" to: "y" ... }
        if self.check(&Token::Arrow) {
            self.expect(Token::Arrow)?;
            to = self.expect_string()?;
            from = name.clone();
        }
        
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::From => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    from = self.expect_identifier_or_string()?;
                }
                Token::To => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    to = self.expect_identifier_or_string()?;
                }
                Token::Identifier(ref id) if id == "from" => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    from = self.expect_identifier_or_string()?;
                }
                Token::Identifier(ref id) if id == "to" => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    to = self.expect_identifier_or_string()?;
                }
                _ => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        let label = attributes.get("label")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        
        let exchange_item = attributes.get("data_type")
            .and_then(|v| v.as_string())
            .unwrap_or("Data")
            .to_string();
        
        Ok(ComponentExchange {
            from_port: from,
            to_port: to,
            exchange_item,
            label,
        })
    }
    
    fn parse_logical_component(&mut self) -> Result<LogicalComponent, String> {
        self.expect(Token::Component)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut functions = Vec::new();
        let mut interfaces_in = Vec::new();
        let mut interfaces_out = Vec::new();
        let mut sub_components = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Component => {
                    // Nested component
                    sub_components.push(self.parse_logical_component()?);
                }
                Token::Function => {
                    functions.push(self.parse_logical_function()?);
                }
                Token::InterfaceIn => {
                    interfaces_in.push(self.parse_interface_definition()?);
                }
                Token::InterfaceOut => {
                    interfaces_out.push(self.parse_interface_definition()?);
                }
                Token::Provides => {
                    // Parse provided interface: provides "InterfaceName" { protocol: "CAN" }
                    self.advance(); // skip 'provides'
                    let interface_def = self.parse_provides_requires_interface(true)?;
                    interfaces_out.push(interface_def);
                }
                Token::Requires => {
                    // Parse required interface: requires "InterfaceName" { protocol: "CAN" }
                    self.advance(); // skip 'requires'
                    let interface_def = self.parse_provides_requires_interface(false)?;
                    interfaces_in.push(interface_def);
                }
                Token::Identifier(_) | Token::Description | Token::Version | Token::Author |
                Token::Priority | Token::Rationale | Token::Verification | Token::Traces |
                Token::SafetyLevel | Token::Parent | Token::Properties | Token::Signals => {
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
        
        let id = attributes.get("id")
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| format!("LC-{}", name.chars().take(3).collect::<String>()));
        
        Ok(LogicalComponent {
            id,
            name,
            component_type: "Logical".to_string(),
            color: Some("#5B9BD5".to_string()),
            sub_components,
            allocated_functions: Vec::new(),
            ports: Vec::new(),
            functions,
            interfaces_in,
            interfaces_out,
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
            match self.current() {
                Token::From => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    from = self.expect_string()?;
                }
                Token::To => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    to = self.expect_string()?;
                }
                Token::Description => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    let desc = self.expect_string()?;
                    attributes.insert("description".to_string(), AttributeValue::String(desc));
                }
                Token::Identifier(_) => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => break,
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
    
    fn parse_connection_as_interface(&mut self) -> Result<LogicalInterface, String> {
        self.expect(Token::Connection)?;
        let name = self.expect_string()?;
        
        let mut from = String::new();
        let mut to = String::new();
        let mut attributes = HashMap::new();
        
        if self.check(&Token::Arrow) {
            self.expect(Token::Arrow)?;
            to = self.expect_string()?;
            from = name.clone();
        }
        
        self.expect(Token::LeftBrace)?;
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::From => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    from = self.expect_identifier_or_string()?;
                }
                Token::To => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    to = self.expect_identifier_or_string()?;
                }
                Token::Identifier(ref id) => {
                    if id == "from" {
                        self.advance();
                        self.expect(Token::Colon)?;
                        from = self.expect_identifier_or_string()?;
                    } else if id == "to" {
                        self.advance();
                        self.expect(Token::Colon)?;
                        to = self.expect_identifier_or_string()?;
                    } else {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    }
                }
                Token::Description | Token::Version | Token::Author | Token::Priority | 
                Token::Rationale | Token::Verification | Token::Traces | Token::SafetyLevel |
                Token::Parent | Token::Properties | Token::Signals | Token::DataType |
                Token::Protocol | Token::Rate | Token::Unit => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => break,
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        let connection_name = if from.is_empty() || to.is_empty() {
            name
        } else {
            format!("{} -> {}", from, to)
        };
        
        Ok(LogicalInterface {
            name: connection_name,
            from,
            to,
            attributes,
        })
    }
    
    fn expect_identifier_or_string(&mut self) -> Result<String, String> {
        match self.current() {
            Token::Identifier(id) => {
                let result = id.clone();
                self.advance();
                Ok(result)
            }
            Token::StringLiteral(s) => {
                let result = s.clone();
                self.advance();
                Ok(result)
            }
            _ => Err(format!("Expected identifier or string, got {}", self.current())),
        }
    }
    
    fn parse_physical_architecture(&mut self) -> Result<PhysicalArchitecture, String> {
        self.expect(Token::PhysicalArchitecture)?;
        let name = self.expect_string()?;
        self.parse_physical_architecture_block(name)
    }
    
    fn parse_physical_architecture_block(&mut self, name: String) -> Result<PhysicalArchitecture, String> {
        self.expect(Token::LeftBrace)?;
        
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Node => {
                    nodes.push(self.parse_physical_node()?);
                }
                Token::Component => {
                    nodes.push(self.parse_physical_component_as_node()?);
                }
                Token::Connection => {
                    links.push(self.parse_connection_as_physical_link()?);
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
        
        Ok(PhysicalArchitecture {
            name,
            nodes,
            links,
            physical_exchanges: Vec::new(),
        })
    }
    
    fn parse_physical_node(&mut self) -> Result<PhysicalNode, String> {
        self.expect(Token::Node)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut deployments = Vec::new();
        let mut behavior_components = Vec::new();
        let mut hardware_components = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Deploys => {
                    deployments.push(self.parse_deployment()?);
                }
                Token::BehaviorComponent => {
                    behavior_components.push(self.parse_behavior_component()?);
                }
                Token::HardwareComponent => {
                    hardware_components.push(self.parse_hardware_component()?);
                }
                Token::Type | Token::Identifier(_) => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => break,
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        // Generate unique ID from attributes or full name
        let id = if let Some(AttributeValue::String(ref id_val)) = attributes.get("id") {
            id_val.clone()
        } else {
            // Use full name sanitized for ID
            format!("PN-{}", name.replace(" ", "_").replace("-", "_"))
        };
        
        Ok(PhysicalNode {
            id,
            name,
            node_type: NodeType::Hardware,
            color: Some("#FFE699".to_string()),
            processor: None,
            memory: None,
            behavior_components,
            hardware_components,
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
        
        let (from, to) = if connections.len() >= 2 {
            (connections[0].clone(), connections[1].clone())
        } else {
            (String::new(), String::new())
        };
        
        Ok(PhysicalLink {
            from,
            to,
            protocol: attributes.get("protocol").and_then(|v| v.as_string()).unwrap_or("Unknown").to_string(),
            bandwidth: None,
            color: None,
            connections,
            attributes,
        })
    }
    
    fn parse_physical_component_as_node(&mut self) -> Result<PhysicalNode, String> {
        self.expect(Token::Component)?;
        let name = self.expect_string()?;
        let attributes = if self.check(&Token::LeftBrace) {
            self.parse_attributes_block()?
        } else {
            HashMap::new()
        };
        
        Ok(PhysicalNode {
            id: format!("PN-{}", name.chars().take(3).collect::<String>()),
            name,
            node_type: NodeType::Hardware,
            color: Some("#FFE699".to_string()),
            processor: None,
            memory: None,
            behavior_components: Vec::new(),
            hardware_components: Vec::new(),
            deployments: Vec::new(),
            attributes,
        })
    }
    
    fn parse_connection_as_physical_link(&mut self) -> Result<PhysicalLink, String> {
        self.expect(Token::Connection)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();
        let mut connections = Vec::new();
        let mut from_node = String::new();
        let mut to_node = String::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Identifier(ref id) => {
                    if id == "from" {
                        self.advance();
                        self.expect(Token::Colon)?;
                        from_node = self.expect_identifier_or_string()?;
                        connections.push(from_node.clone());
                    } else if id == "to" {
                        self.advance();
                        self.expect(Token::Colon)?;
                        to_node = self.expect_identifier_or_string()?;
                        connections.push(to_node.clone());
                    } else {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    }
                }
                Token::Protocol | Token::DataType | Token::Description | Token::Rate |
                Token::Unit | Token::Version | Token::Author => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => break,
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(PhysicalLink {
            from: from_node,
            to: to_node,
            protocol: attributes.get("protocol").and_then(|v| v.as_string()).unwrap_or("Unknown").to_string(),
            bandwidth: None,
            color: None,
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
        
        // Support two syntaxes:
        // 1. trace { from: "X" to: "Y" type: "..." }  (attribute block syntax)
        // 2. trace "X" -> "Y" { ... }  (arrow syntax)
        // 3. trace "X" satisfies "Y" { ... }  (keyword syntax)
        
        if self.check(&Token::LeftBrace) {
            // Attribute block syntax
            let attributes = self.parse_attributes_block()?;
            
            let from = attributes.get("from")
                .and_then(|v| v.as_string())
                .ok_or("trace block missing 'from' attribute")?
                .to_string();
            
            let to = attributes.get("to")
                .and_then(|v| v.as_string())
                .ok_or("trace block missing 'to' attribute")?
                .to_string();
            
            let trace_type = attributes.get("type")
                .and_then(|v| v.as_string())
                .unwrap_or("relates_to")
                .to_string();
            
            return Ok(Trace {
                from,
                to,
                trace_type,
                attributes,
            });
        }
        
        // Arrow or keyword syntax
        let from = self.expect_string()?;
        
        // Support both syntaxes:
        // 1. trace "X" satisfies "Y" { rationale: "..." }
        // 2. trace "X" -> "Y" { trace_type: "satisfies", rationale: "..." }
        let (trace_type, to) = if let Token::Arrow = self.current() {
            // Arrow syntax: read trace_type from attributes
            self.advance(); // skip '->'
            let to = self.expect_string()?;
            let attributes = if self.check(&Token::LeftBrace) {
                self.parse_attributes_block()?
            } else {
                HashMap::new()
            };
            
            let trace_type = if let Some(AttributeValue::String(t)) = attributes.get("trace_type") {
                t.clone()
            } else {
                "relates_to".to_string() // default if not specified
            };
            
            return Ok(Trace {
                from,
                to,
                trace_type,
                attributes,
            });
        } else if let Token::Satisfies = self.current() {
            self.advance();
            ("satisfies".to_string(), self.expect_string()?)
        } else if let Token::Implements = self.current() {
            self.advance();
            ("implements".to_string(), self.expect_string()?)
        } else if let Token::Validates = self.current() {
            self.advance();
            ("validates".to_string(), self.expect_string()?)
        } else {
            return Err("Expected trace type (satisfies, implements, validates, etc.) or arrow".to_string());
        };
        
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
        // Accept both identifiers and reserved keywords as attribute keys
        let key = match self.current() {
            Token::Identifier(id) => {
                let k = id.clone();
                self.advance();
                k
            }
            // Commonly used attribute keywords
            Token::Inputs => { self.advance(); "inputs".to_string() }
            Token::Outputs => { self.advance(); "outputs".to_string() }
            Token::Type => { self.advance(); "type".to_string() }
            Token::Port => { self.advance(); "port".to_string() }
            Token::Flow => { self.advance(); "flow".to_string() }
            Token::Description => { self.advance(); "description".to_string() }
            Token::From => { self.advance(); "from".to_string() }
            Token::To => { self.advance(); "to".to_string() }
            Token::Rationale => { self.advance(); "rationale".to_string() }
            Token::Priority => { self.advance(); "priority".to_string() }
            Token::Version => { self.advance(); "version".to_string() }
            Token::Author => { self.advance(); "author".to_string() }
            Token::Parent => { self.advance(); "parent".to_string() }
            Token::Protocol => { self.advance(); "protocol".to_string() }
            Token::Rate => { self.advance(); "rate".to_string() }
            Token::Latency => { self.advance(); "latency".to_string() }
            Token::Unit => { self.advance(); "unit".to_string() }
            Token::Value => { self.advance(); "value".to_string() }
            Token::Property => { self.advance(); "property".to_string() }
            Token::Action => { self.advance(); "action".to_string() }
            Token::Measure => { self.advance(); "measure".to_string() }
            Token::SafetyLevel => { self.advance(); "safety_level".to_string() }
            Token::SafetyMeasures => { self.advance(); "safety_measures".to_string() }
            Token::SafetyAnalysis => { self.advance(); "safety_analysis".to_string() }
            Token::DataType => { self.advance(); "data_type".to_string() }
            Token::Component => { self.advance(); "component".to_string() }
            _ => return Err(format!("Expected attribute key, got: {}", self.current())),
        };
        
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
            Token::Function => { self.advance(); Ok("function".to_string()) }
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
    
    fn skip_until_brace_balanced(&mut self) -> Result<(), String> {
        // Skip the keyword (scenario, dataflow, etc.)
        self.advance();
        
        // Skip identifiers/strings until we hit a brace
        while !self.is_at_end() && !matches!(self.current(), Token::LeftBrace) {
            self.advance();
        }
        
        // Now skip the balanced braces
        if matches!(self.current(), Token::LeftBrace) {
            self.skip_block()?;
        }
        
        Ok(())
    }
    
    fn parse_interface_definition(&mut self) -> Result<InterfaceDefinition, String> {
        // Parse interface_in or interface_out
        if !matches!(self.current(), Token::InterfaceIn | Token::InterfaceOut) {
            return Err(format!("Expected interface_in or interface_out, got {}", self.current()));
        }
        self.advance();
        
        // Optional colon (support both "interface_out: "Name"" and "interface_out "Name"")
        if self.check(&Token::Colon) {
            self.advance();
        }
        
        // Interface name (string literal)
        let name = self.expect_string()?;
        
        // Expect left brace
        self.expect(Token::LeftBrace)?;
        
        let mut protocol = None;
        let mut format = None;
        let mut attributes = HashMap::new();
        
        // Parse interface attributes
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Protocol => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    protocol = Some(self.expect_string()?);
                }
                Token::Identifier(ref id) if id == "format" => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    format = Some(self.expect_string()?);
                }
                Token::Identifier(_) => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => {
                    self.advance();
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(InterfaceDefinition {
            name,
            protocol,
            format,
            attributes,
        })
    }
    
    fn parse_provides_requires_interface(&mut self, is_provided: bool) -> Result<InterfaceDefinition, String> {
        // Parse: provides "InterfaceName" { protocol: "CAN" }
        // or: requires "InterfaceName" { protocol: "CAN" }
        
        // Interface name (string literal)
        let name = self.expect_string()?;
        
        // Expect left brace
        self.expect(Token::LeftBrace)?;
        
        let mut protocol = None;
        let mut format = None;
        let mut attributes = HashMap::new();
        
        // Parse interface attributes
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Protocol => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    protocol = Some(self.expect_string()?);
                }
                Token::Identifier(ref id) if id == "format" => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    format = Some(self.expect_string()?);
                }
                Token::Identifier(_) => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => {
                    self.advance();
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(InterfaceDefinition {
            name,
            protocol,
            format,
            attributes,
        })
    }
    
    fn parse_behavior_component(&mut self) -> Result<BehaviorComponent, String> {
        self.expect(Token::BehaviorComponent)?;
        let name = self.expect_string()?;
        self.expect(Token::LeftBrace)?;
        
        let mut allocated_functions = Vec::new();
        let mut color = None;
        let mut id = None;
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Token::Identifier(ref key) = self.current() {
                let key_clone = key.clone();
                self.advance();
                self.expect(Token::Colon)?;
                
                match key_clone.as_str() {
                    "id" => {
                        id = Some(self.expect_string()?);
                    }
                    "color" => {
                        color = Some(self.expect_string()?);
                    }
                    "allocated_functions" => {
                        if self.check(&Token::LeftBracket) {
                            self.advance();
                            while !self.check(&Token::RightBracket) && !self.is_at_end() {
                                allocated_functions.push(self.expect_string()?);
                                if self.check(&Token::Comma) {
                                    self.advance();
                                }
                            }
                            self.expect(Token::RightBracket)?;
                        }
                    }
                    _ => {
                        match self.current() {
                            Token::StringLiteral(_) | Token::Number(_) => {
                                self.advance();
                            }
                            Token::LeftBracket => {
                                self.skip_block()?;
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(BehaviorComponent {
            id: id.unwrap_or_else(|| format!("BC-{}", name.chars().take(3).collect::<String>())),
            name,
            allocated_functions,
            color,
        })
    }
    
    fn parse_hardware_component(&mut self) -> Result<HardwareComponent, String> {
        self.expect(Token::HardwareComponent)?;
        let name = self.expect_string()?;
        
        let mut hw_type = "Generic".to_string();
        let mut specs = None;
        let mut color = None;
        
        if self.check(&Token::LeftBrace) {
            self.advance();
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                if let Token::Identifier(ref key) = self.current() {
                    let key_clone = key.clone();
                    self.advance();
                    self.expect(Token::Colon)?;
                    
                    match key_clone.as_str() {
                        "type" => {
                            hw_type = self.expect_string()?;
                        }
                        "specs" => {
                            specs = Some(self.expect_string()?);
                        }
                        "color" => {
                            color = Some(self.expect_string()?);
                        }
                        _ => {
                            match self.current() {
                                Token::StringLiteral(_) | Token::Number(_) => {
                                    self.advance();
                                }
                                _ => {
                                    self.advance();
                                }
                            }
                        }
                    }
                } else {
                    break;
                }
            }
            self.expect(Token::RightBrace)?;
        }
        
        Ok(HardwareComponent {
            id: format!("HC-{}", name.chars().take(3).collect::<String>()),
            name,
            hw_type,
            specs,
            color,
        })
    }
}
