use super::ast::*;
use super::lexer::{Span, Token};
use std::collections::HashMap;

/// Result of a parse, including non-fatal warnings (e.g. constructs that are
/// syntactically accepted but not yet represented in the model).
#[derive(Debug)]
pub struct ParseOutcome {
    pub model: Model,
    pub warnings: Vec<String>,
}

pub struct Parser {
    tokens: Vec<Token>,
    spans: Vec<Span>,
    position: usize,
    warnings: Vec<String>,
    /// Traces declared inside architecture blocks; hoisted to the model level.
    pending_traces: Vec<Trace>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            spans: Vec::new(),
            position: 0,
            warnings: Vec::new(),
            pending_traces: Vec::new(),
        }
    }

    pub fn with_spans(tokens: Vec<Token>, spans: Vec<Span>) -> Self {
        Self {
            tokens,
            spans,
            position: 0,
            warnings: Vec::new(),
            pending_traces: Vec::new(),
        }
    }

    pub fn parse(self) -> Result<Model, String> {
        self.parse_with_warnings().map(|outcome| outcome.model)
    }

    pub fn parse_with_warnings(mut self) -> Result<ParseOutcome, String> {
        let mut model = self.parse_model_root()?;
        model.traces.append(&mut self.pending_traces);
        Ok(ParseOutcome { model, warnings: self.warnings })
    }

    fn parse_model_root(&mut self) -> Result<Model, String> {
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
                    model.logical_architecture.push(self.parse_logical_architecture()?);
                }
                Token::PhysicalArchitecture => {
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
                Token::Scenario => {
                    model.scenarios.push(self.parse_scenario()?);
                }
                Token::Scenarios => {
                    self.parse_scenarios_container(&mut model)?;
                }
                Token::StateMachineKw => {
                    model.state_machines.push(self.parse_state_machine()?);
                }
                Token::Dataflow => {
                    self.warn_unmodeled_block("top level")?;
                }
                Token::Eof => break,
                _ => return Err(self.err(format!("Unexpected token at top level: {}", self.current()))),
            }
        }

        Ok(model)
    }

    /// Build an error message carrying the current source position when available.
    fn err(&self, msg: impl Into<String>) -> String {
        let msg = msg.into();
        match self.current_span() {
            Some(span) => format!("{} at {}", msg, span),
            None => msg,
        }
    }

    /// Record a warning carrying the current source position when available.
    fn warn(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        let msg = match self.current_span() {
            Some(span) => format!("{} at {}", msg, span),
            None => msg,
        };
        self.warnings.push(msg);
    }

    /// Warn that a known-but-unmodeled construct is being skipped, then skip it.
    /// This is loud by design: nothing may disappear from a model silently.
    fn warn_unmodeled_block(&mut self, context: &str) -> Result<(), String> {
        self.warn(format!(
            "'{}' block ({}) is not yet represented in the compiled model; its contents are ignored",
            self.current(),
            context
        ));
        self.skip_until_brace_balanced()
    }

    fn current_span(&self) -> Option<Span> {
        self.spans.get(self.position).copied()
    }
    
    fn parse_model_block(&mut self) -> Result<Model, String> {
        // Support both 'model' and 'system' keywords
        if matches!(self.current(), Token::Model | Token::System) {
            self.advance();
        } else {
            return Err(self.err(format!("Expected 'model' or 'system', got {}", self.current())));
        }

        // Model name can be either identifier or string
        let model_name = self.expect_name()?;

        self.expect(Token::LeftBrace)?;

        let mut model = Model::new();
        model.attributes.insert("name".to_string(), AttributeValue::String(model_name));

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Metadata => {
                    // metadata { version: "..." author: "..." } -> model attributes
                    self.advance();
                    let metadata = self.parse_attributes_block()?;
                    model.attributes.extend(metadata);
                }
                Token::Requirements => {
                    // Parse requirements block with subtype (stakeholder/system/safety)
                    model.system_analysis.push(self.parse_requirements_block()?);
                }
                Token::Architecture => {
                    self.parse_architecture_into(&mut model)?;
                }
                Token::LogicalArchitecture => {
                    model.logical_architecture.push(self.parse_logical_architecture()?);
                }
                Token::PhysicalArchitecture => {
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
                Token::Scenario => {
                    model.scenarios.push(self.parse_scenario()?);
                }
                Token::Scenarios => {
                    self.parse_scenarios_container(&mut model)?;
                }
                Token::StateMachineKw => {
                    model.state_machines.push(self.parse_state_machine()?);
                }
                Token::Dataflow | Token::DataFlows => {
                    self.warn_unmodeled_block("model block")?;
                }
                Token::Identifier(ref id) if id == "traceability" => {
                    self.warn_unmodeled_block("model block")?;
                }
                Token::Eof => break,
                _ => {
                    // Model header attributes: name: "...", version: "...", etc.
                    if self.peek_is_colon() {
                        let (key, value) = self.parse_attribute()?;
                        model.attributes.insert(key, value);
                    } else {
                        return Err(self.err(format!(
                            "Unexpected token in model block: {}",
                            self.current()
                        )));
                    }
                }
            }
        }

        self.expect(Token::RightBrace)?;

        // Continue parsing top-level blocks after the model block
        while !self.is_at_end() {
            match self.current() {
                Token::Requirements => {
                    model.system_analysis.push(self.parse_requirements_block()?);
                }
                Token::Architecture => {
                    self.parse_architecture_into(&mut model)?;
                }
                Token::LogicalArchitecture => {
                    model.logical_architecture.push(self.parse_logical_architecture()?);
                }
                Token::PhysicalArchitecture => {
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
                Token::Scenario => {
                    model.scenarios.push(self.parse_scenario()?);
                }
                Token::Scenarios => {
                    self.parse_scenarios_container(&mut model)?;
                }
                Token::StateMachineKw => {
                    model.state_machines.push(self.parse_state_machine()?);
                }
                Token::Dataflow | Token::DataFlows => {
                    self.warn_unmodeled_block("top level")?;
                }
                Token::ValidationKeyword => {
                    self.warn_unmodeled_block("top level")?;
                }
                Token::Identifier(ref id) if id == "traces" || id == "traceability" => {
                    self.warn_unmodeled_block("top level")?;
                }
                Token::Eof => break,
                _ => {
                    return Err(self.err(format!(
                        "Unexpected token after model block: {}",
                        self.current()
                    )));
                }
            }
        }

        Ok(model)
    }

    /// Parse `architecture logical { ... }` / `architecture physical { ... }`
    /// into the model. Other subtypes are accepted syntactically but warned about.
    fn parse_architecture_into(&mut self, model: &mut Model) -> Result<(), String> {
        self.expect(Token::Architecture)?;
        match self.current() {
            Token::Logical => {
                self.advance();
                let name = self.optional_name().unwrap_or_else(|| "Logical Architecture".to_string());
                model.logical_architecture.push(self.parse_logical_architecture_block(name)?);
            }
            Token::Physical => {
                self.advance();
                let name = self.optional_name().unwrap_or_else(|| "Physical Architecture".to_string());
                model.physical_architecture.push(self.parse_physical_architecture_block(name)?);
            }
            _ => {
                self.warn(format!(
                    "architecture subtype '{}' is not yet represented in the compiled model; its contents are ignored",
                    self.current()
                ));
                self.skip_until_brace_balanced()?;
            }
        }
        Ok(())
    }

    /// Consume an identifier (possibly dotted, e.g. `Vehicle.MonitorEnvironment`)
    /// or string literal name if present.
    fn optional_name(&mut self) -> Option<String> {
        match self.current() {
            Token::Identifier(id) => {
                let mut name = id.clone();
                self.advance();
                // Dotted references: A.B.C
                while self.check(&Token::Dot) {
                    if let Some(Token::Identifier(next)) = self.tokens.get(self.position + 1) {
                        name.push('.');
                        name.push_str(next);
                        self.advance(); // '.'
                        self.advance(); // identifier
                    } else {
                        break;
                    }
                }
                Some(name)
            }
            Token::StringLiteral(s) => {
                let name = s.clone();
                self.advance();
                Some(name)
            }
            _ => None,
        }
    }

    /// Expect an identifier or string literal name.
    fn expect_name(&mut self) -> Result<String, String> {
        self.optional_name()
            .ok_or_else(|| self.err(format!("Expected name (identifier or string), got {}", self.current())))
    }

    fn peek_is_colon(&self) -> bool {
        matches!(self.tokens.get(self.position + 1), Some(Token::Colon))
    }
    
    fn parse_model_with_toplevel_blocks(&mut self) -> Result<Model, String> {
        // Alternative syntax: top-level blocks without a model wrapper
        let mut model = Model::new();

        while !self.is_at_end() {
            match self.current() {
                Token::Requirements => {
                    // Parse top-level requirements blocks (stakeholder/system/safety)
                    model.system_analysis.push(self.parse_requirements_block()?);
                }
                Token::Architecture => {
                    self.parse_architecture_into(&mut model)?;
                }
                Token::LogicalArchitecture => {
                    model.logical_architecture.push(self.parse_logical_architecture()?);
                }
                Token::PhysicalArchitecture => {
                    model.physical_architecture.push(self.parse_physical_architecture()?);
                }
                Token::SafetyAnalysis => {
                    model.safety_analysis.push(self.parse_safety_analysis()?);
                }
                Token::Trace => {
                    model.traces.push(self.parse_trace()?);
                }
                Token::Scenario => {
                    model.scenarios.push(self.parse_scenario()?);
                }
                Token::Scenarios => {
                    self.parse_scenarios_container(&mut model)?;
                }
                Token::StateMachineKw => {
                    model.state_machines.push(self.parse_state_machine()?);
                }
                Token::DataFlows | Token::Dataflow | Token::ValidationKeyword => {
                    self.warn_unmodeled_block("top level")?;
                }
                Token::Identifier(ref id) if id == "traces" || id == "traceability" => {
                    self.warn_unmodeled_block("top level")?;
                }
                Token::Eof => break,
                _ => {
                    return Err(self.err(format!("Unexpected token at top level: {}", self.current())));
                }
            }
        }

        Ok(model)
    }
    
    fn parse_operational_analysis(&mut self) -> Result<OperationalAnalysis, String> {
        self.expect(Token::OperationalAnalysis)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut name = name;
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
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    if key == "name" {
                        if let AttributeValue::String(s) = value {
                            name = s;
                        }
                    } else {
                        self.warn(format!(
                            "attribute '{}' on operational_analysis is not yet stored in the model",
                            key
                        ));
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in operational_analysis: {}", self.current())));
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
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();
        let mut activities = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Identifier(ref id) if id == "operational_activity" || id == "activity" => {
                    activities.push(self.parse_operational_activity()?);
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in operational_entity: {}", self.current())));
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
        let name = self.expect_name()?;
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
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;

        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("OC-{}", name.replace(' ', "_")));

        Ok(OperationalCapability {
            id,
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
        let name = self.expect_name()?;
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

        // Two forms:
        //   exchange "A" -> "B" { ... }          (arrow form)
        //   interaction Name { from: A to: B }   (block form)
        let name = self.expect_name()?;

        let (mut from, mut to) = (String::new(), String::new());
        if self.check(&Token::Arrow) {
            self.advance();
            from = name.clone();
            to = self.expect_name()?;
        }

        self.expect(Token::LeftBrace)?;

        let mut attributes = HashMap::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let (key, value) = self.parse_attribute()?;
            attributes.insert(key, value);
        }

        self.expect(Token::RightBrace)?;

        if from.is_empty() {
            from = attributes
                .get("from")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    self.err(format!("exchange '{}' is missing a 'from' endpoint", name))
                })?;
            to = attributes
                .get("to")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    self.err(format!("exchange '{}' is missing a 'to' endpoint", name))
                })?;
            attributes
                .entry("label".to_string())
                .or_insert_with(|| AttributeValue::String(name.clone()));
        }
        
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
        let mut name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;

        let mut requirements = Vec::new();
        let mut functions = Vec::new();
        let mut components = Vec::new();
        let mut external_actors = Vec::new();
        let mut functional_exchanges = Vec::new();
        let mut missions = Vec::new();
        let mut capabilities = Vec::new();
        let mut functional_chains = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Requirement => {
                    requirements.push(self.parse_requirement()?);
                }
                Token::Req => {
                    requirements.push(self.parse_req_statement()?);
                }
                Token::Function => {
                    functions.push(self.parse_system_function()?);
                }
                Token::Mission => {
                    missions.push(self.parse_mission()?);
                }
                Token::Capability => {
                    capabilities.push(self.parse_capability()?);
                }
                Token::FunctionalChain => {
                    functional_chains.push(self.parse_functional_chain()?);
                }
                Token::Actor => {
                    external_actors.push(self.parse_external_actor()?);
                }
                Token::Identifier(ref id) if id == "system_function" => {
                    functions.push(self.parse_system_function()?);
                }
                Token::Identifier(ref id) if id == "system_component" => {
                    components.push(self.parse_system_component()?);
                }
                Token::Identifier(ref id)
                    if id == "functional_exchange" || id == "exchange" =>
                {
                    functional_exchanges.push(self.parse_functional_exchange()?);
                }
                Token::Flow => {
                    functional_exchanges.push(self.parse_functional_exchange()?);
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    if key == "name" {
                        if let AttributeValue::String(s) = value {
                            name = s;
                        }
                    } else {
                        self.warn(format!(
                            "attribute '{}' on system_analysis is not yet stored in the model",
                            key
                        ));
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in system_analysis: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(SystemAnalysis {
            name,
            requirements,
            functions,
            components,
            external_actors,
            functional_exchanges,
            missions,
            capabilities,
            functional_chains,
        })
    }

    /// Parse: mission Name { id: "..." description: "..." }
    fn parse_mission(&mut self) -> Result<Mission, String> {
        self.expect(Token::Mission)?;
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("MIS-{}", name.replace(' ', "_")));
        Ok(Mission { id, name, attributes })
    }

    /// Parse: capability Name { id: ... involves: [..] realizes: "..." mission: "..." }
    /// Also used for LA `capability_realization` blocks.
    fn parse_capability(&mut self) -> Result<Capability, String> {
        self.advance(); // Skip 'capability' or 'capability_realization'
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("CAP-{}", name.replace(' ', "_")));
        let involves = Self::string_list(&attributes, "involves");
        let realizes = attributes
            .get("realizes")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        let mission = attributes
            .get("mission")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        Ok(Capability { id, name, involves, realizes, mission, attributes })
    }

    /// Parse: functional_chain Name { id: ... involves: ["F1", "FE1", "F2"] }
    fn parse_functional_chain(&mut self) -> Result<FunctionalChain, String> {
        self.expect(Token::FunctionalChain)?;
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("FC-{}", name.replace(' ', "_")));
        let involves = Self::string_list(&attributes, "involves");
        if involves.is_empty() {
            return Err(self.err(format!(
                "functional_chain '{}' must involve at least one function (involves: [...])",
                name
            )));
        }
        Ok(FunctionalChain { id, name, involves, attributes })
    }

    /// Parse a function port: `port in Name { data_type: "..." }` (in XOR out).
    fn parse_function_port(&mut self) -> Result<FunctionPort, String> {
        self.expect(Token::Port)?;
        let direction = match self.current() {
            Token::In => { self.advance(); PortDirection::In }
            Token::Out => { self.advance(); PortDirection::Out }
            _ => {
                return Err(self.err(
                    "function ports are strictly 'in' or 'out' (Arcadia): port in|out Name { ... }",
                ));
            }
        };
        let name = self.expect_name()?;
        let attributes = if self.check(&Token::LeftBrace) {
            self.parse_attributes_block()?
        } else {
            HashMap::new()
        };
        let port_type = match attributes.get("type").and_then(|v| v.as_string()) {
            Some("control") => PortType::Control,
            Some("event") => PortType::Event,
            _ => PortType::Data,
        };
        let data_type = attributes
            .get("data_type")
            .and_then(|v| v.as_string())
            .unwrap_or("Data")
            .to_string();
        Ok(FunctionPort { name, direction, port_type, data_type })
    }

    /// Parse a component port: `port in|out|inout Name { protocol: "..." }`.
    fn parse_component_port(&mut self) -> Result<ComponentPort, String> {
        self.expect(Token::Port)?;
        let direction = match self.current() {
            Token::In => { self.advance(); PortDirection::In }
            Token::Out => { self.advance(); PortDirection::Out }
            Token::Identifier(ref d) if d == "inout" => { self.advance(); PortDirection::InOut }
            _ => {
                return Err(self.err(
                    "component ports are 'in', 'out' or 'inout' (Arcadia): port in|out|inout Name { ... }",
                ));
            }
        };
        let name = self.expect_name()?;
        let attributes = if self.check(&Token::LeftBrace) {
            self.parse_attributes_block()?
        } else {
            HashMap::new()
        };
        let interface_type = attributes
            .get("interface")
            .or_else(|| attributes.get("protocol"))
            .and_then(|v| v.as_string())
            .unwrap_or("Data")
            .to_string();
        Ok(ComponentPort { name, direction, interface_type })
    }

    /// Parse an UNORIENTED physical port: `port Name { ... }` (Arcadia).
    fn parse_physical_port(&mut self) -> Result<PhysicalPort, String> {
        self.expect(Token::Port)?;
        if matches!(self.current(), Token::In | Token::Out) {
            return Err(self.err(
                "physical ports are not oriented (Arcadia): port Name { ... }",
            ));
        }
        let name = self.expect_name()?;
        let attributes = if self.check(&Token::LeftBrace) {
            self.parse_attributes_block()?
        } else {
            HashMap::new()
        };
        Ok(PhysicalPort { name, attributes })
    }

    /// Parse: state_machine Name { initial: "Idle" state A {..} mode B {..} transition A -> B { trigger: } }
    fn parse_state_machine(&mut self) -> Result<StateMachine, String> {
        self.expect(Token::StateMachineKw)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;

        let mut initial_state = String::new();
        let mut states = Vec::new();
        let mut transitions = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::StateKw | Token::Mode => {
                    let kind = if matches!(self.current(), Token::Mode) {
                        StateKind::Mode
                    } else {
                        StateKind::State
                    };
                    self.advance();
                    let state_name = self.expect_name()?;
                    let attributes = if self.check(&Token::LeftBrace) {
                        self.parse_attributes_block()?
                    } else {
                        HashMap::new()
                    };
                    states.push(State {
                        name: state_name,
                        kind,
                        entry_actions: Self::string_list(&attributes, "entry"),
                        exit_actions: Self::string_list(&attributes, "exit"),
                        internal_transitions: Vec::new(),
                        sub_states: Vec::new(),
                        color: attributes.get("color").and_then(|v| v.as_string()).map(|s| s.to_string()),
                    });
                }
                Token::Transition => {
                    self.advance();
                    let from = self.expect_name()?;
                    self.expect(Token::Arrow)?;
                    let to = self.expect_name()?;
                    let attributes = if self.check(&Token::LeftBrace) {
                        self.parse_attributes_block()?
                    } else {
                        HashMap::new()
                    };
                    let get = |key: &str| attributes.get(key).and_then(|v| v.as_string()).map(|s| s.to_string());
                    transitions.push(Transition {
                        from,
                        to,
                        trigger: get("trigger").unwrap_or_default(),
                        guard: get("guard"),
                        action: get("action"),
                        timing: get("timing"),
                        priority: get("priority"),
                    });
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    if key == "initial" {
                        if let AttributeValue::String(s) = value {
                            initial_state = s;
                        }
                    } else {
                        self.warn(format!(
                            "attribute '{}' on state_machine is not yet stored in the model",
                            key
                        ));
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in state_machine: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(StateMachine { name, initial_state, states, transitions })
    }

    /// Parse: scenarios [Name] { scenario A { ... } scenario B { ... } }
    fn parse_scenarios_container(&mut self, model: &mut Model) -> Result<(), String> {
        self.expect(Token::Scenarios)?;
        let _container_name = self.optional_name();
        self.expect(Token::LeftBrace)?;
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Scenario => {
                    model.scenarios.push(self.parse_scenario()?);
                }
                _ => {
                    return Err(self.err(format!(
                        "Unexpected token in scenarios container: {} (expected 'scenario')",
                        self.current()
                    )));
                }
            }
        }
        self.expect(Token::RightBrace)?;
        Ok(())
    }

    /// Parse: scenario Name { participants: [..] message A -> B "label" { type: async } }
    fn parse_scenario(&mut self) -> Result<Scenario, String> {
        self.expect(Token::Scenario)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;

        let mut participants = Vec::new();
        let mut messages = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Participants => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    let value = self.parse_attribute_value()?;
                    if let AttributeValue::List(items) = value {
                        for item in items {
                            if let Some(reference) = item.as_string() {
                                participants.push(Participant {
                                    id: reference.to_string(),
                                    name: reference.to_string(),
                                    participant_type: ParticipantType::Component,
                                    lifeline_color: "#5B9BD5".to_string(),
                                });
                            }
                        }
                    }
                }
                Token::Message => {
                    self.advance();
                    let from = self.expect_name()?;
                    self.expect(Token::Arrow)?;
                    let to = self.expect_name()?;
                    let label = self.optional_name().unwrap_or_default();
                    let attributes = if self.check(&Token::LeftBrace) {
                        self.parse_attributes_block()?
                    } else {
                        HashMap::new()
                    };
                    let message_type = match attributes.get("type").and_then(|v| v.as_string()) {
                        Some("async") | Some("asynchronous") => MessageType::Asynchronous,
                        _ => MessageType::Synchronous,
                    };
                    messages.push(Message {
                        from,
                        to,
                        label,
                        message_type,
                        activation: true,
                        timing: attributes.get("timing").and_then(|v| v.as_string()).map(|s| s.to_string()),
                        params: None,
                    });
                }
                _ if self.peek_is_colon() => {
                    let (key, _) = self.parse_attribute()?;
                    self.warn(format!("attribute '{}' on scenario is not yet stored in the model", key));
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in scenario: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Scenario {
            name,
            participants,
            messages,
            fragments: Vec::new(),
            timing_constraints: Vec::new(),
        })
    }

    fn string_list(attributes: &HashMap<String, AttributeValue>, key: &str) -> Vec<String> {
        match attributes.get(key) {
            Some(AttributeValue::List(items)) => items
                .iter()
                .filter_map(|v| v.as_string().map(|s| s.to_string()))
                .collect(),
            Some(AttributeValue::String(single)) => vec![single.clone()],
            _ => Vec::new(),
        }
    }

    fn parse_external_actor(&mut self) -> Result<ExternalActor, String> {
        self.expect(Token::Actor)?;
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;

        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("SA-ACT-{}", name.replace(' ', "_")));
        let color = attributes
            .get("color")
            .and_then(|v| v.as_string())
            .unwrap_or("#FFFFFF")
            .to_string();

        Ok(ExternalActor { id, name, color, attributes })
    }

    /// Parse a functional exchange in either form:
    ///   functional_exchange Name { from: A to: B exchange_item: "..." }
    ///   flow "A" -> "B" { ... }
    fn parse_functional_exchange(&mut self) -> Result<FunctionalExchange, String> {
        self.advance(); // Skip 'functional_exchange', 'exchange', or 'flow'
        let name = self.expect_name()?;

        let (mut from, mut to) = (String::new(), String::new());
        if self.check(&Token::Arrow) {
            self.advance();
            from = name.clone();
            to = self.expect_name()?;
        }

        let attributes = self.parse_attributes_block()?;

        if from.is_empty() {
            from = attributes
                .get("from")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    self.err(format!("functional_exchange '{}' is missing a 'from' endpoint", name))
                })?;
            to = attributes
                .get("to")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    self.err(format!("functional_exchange '{}' is missing a 'to' endpoint", name))
                })?;
        }

        let data_type = attributes
            .get("exchange_item")
            .or_else(|| attributes.get("data_type"))
            .and_then(|v| v.as_string())
            .unwrap_or("Data")
            .to_string();
        let label = attributes
            .get("label")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .or(Some(name));

        Ok(FunctionalExchange { from_port: from, to_port: to, data_type, label })
    }
    
    fn parse_requirements_block(&mut self) -> Result<SystemAnalysis, String> {
        self.expect(Token::Requirements)?;
        
        // Optional subtype (stakeholder, system, safety, ...)
        let subtype = match self.current() {
            Token::Stakeholder => {
                self.advance();
                "Stakeholder"
            }
            Token::System => {
                self.advance();
                "System"
            }
            Token::Identifier(ref id) if id == "safety" => {
                self.advance();
                "Safety"
            }
            Token::LeftBrace => "Requirements",
            _ => {
                self.advance();
                "Requirements"
            }
        };

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
                    return Err(self.err(format!(
                        "Unexpected token in requirements block: {} (expected 'req' or 'requirement')",
                        self.current()
                    )));
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
            missions: Vec::new(),
            capabilities: Vec::new(),
            functional_chains: Vec::new(),
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
        let id = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;

        Ok(Requirement { id, attributes })
    }
    
    fn parse_system_function(&mut self) -> Result<SystemFunction, String> {
        self.advance(); // Skip 'system_function'
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();
        let mut sub_functions = Vec::new();
        
        let mut ports = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Function) {
                let sub_func = self.parse_nested_function()?;
                sub_functions.push(sub_func);
            } else if self.check(&Token::Port) {
                ports.push(self.parse_function_port()?);
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
            ports,
            sub_functions,
            attributes,
        })
    }
    
    fn parse_nested_function(&mut self) -> Result<SystemFunction, String> {
        self.advance(); // Skip 'function'
        let name = self.expect_name()?;
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
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(SystemComponent { name, attributes })
    }
    
    fn parse_logical_architecture(&mut self) -> Result<LogicalArchitecture, String> {
        self.expect(Token::LogicalArchitecture)?;
        let name = self.expect_name()?;
        self.parse_logical_architecture_block(name)
    }
    
    fn parse_logical_architecture_block(&mut self, name: String) -> Result<LogicalArchitecture, String> {
        self.expect(Token::LeftBrace)?;

        let mut name = name;
        let mut components = Vec::new();
        let mut interfaces = Vec::new();
        let mut component_exchanges = Vec::new();
        let mut capability_realizations = Vec::new();
        let mut functional_chains = Vec::new();
        
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
                Token::Identifier(ref id) if id == "component_exchange" => {
                    component_exchanges.push(self.parse_named_component_exchange()?);
                }
                Token::CapabilityRealization | Token::Capability => {
                    capability_realizations.push(self.parse_capability()?);
                }
                Token::FunctionalChain => {
                    functional_chains.push(self.parse_functional_chain()?);
                }
                Token::Trace => {
                    // Traces are collected at the model level
                    let trace = self.parse_trace()?;
                    self.pending_traces.push(trace);
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    if key == "name" {
                        if let AttributeValue::String(s) = value {
                            name = s;
                        }
                    } else {
                        self.warn(format!(
                            "attribute '{}' on logical_architecture is not yet stored in the model",
                            key
                        ));
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in logical_architecture: {}", self.current())));
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
            capability_realizations,
            functional_chains,
        })
    }
    
    fn parse_component_exchange(&mut self) -> Result<ComponentExchange, String> {
        self.expect(Token::Connection)?;
        
        let name = self.expect_name()?;
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
    
    /// Parse: component_exchange "Name" { from_port: "..." to_port: "..." ... }
    fn parse_named_component_exchange(&mut self) -> Result<ComponentExchange, String> {
        self.advance(); // Skip 'component_exchange'
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;

        let from_port = attributes
            .get("from_port")
            .or_else(|| attributes.get("from"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                self.err(format!("component_exchange '{}' is missing 'from_port'", name))
            })?;
        let to_port = attributes
            .get("to_port")
            .or_else(|| attributes.get("to"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                self.err(format!("component_exchange '{}' is missing 'to_port'", name))
            })?;
        let exchange_item = attributes
            .get("exchange_item")
            .or_else(|| attributes.get("data_type"))
            .and_then(|v| v.as_string())
            .unwrap_or("Data")
            .to_string();
        let label = attributes
            .get("label")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .or(Some(name));

        Ok(ComponentExchange { from_port, to_port, exchange_item, label })
    }

    fn parse_logical_component(&mut self) -> Result<LogicalComponent, String> {
        self.expect(Token::Component)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut functions = Vec::new();
        let mut interfaces_in = Vec::new();
        let mut interfaces_out = Vec::new();
        let mut sub_components = Vec::new();
        let mut ports = Vec::new();
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
                Token::Port => {
                    ports.push(self.parse_component_port()?);
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
                    if self.peek_is_colon() {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    } else {
                        return Err(self.err(format!(
                            "Unexpected token in component: {}",
                            self.current()
                        )));
                    }
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
            ports,
            functions,
            interfaces_in,
            interfaces_out,
            attributes,
        })
    }
    
    fn parse_logical_function(&mut self) -> Result<LogicalFunction, String> {
        self.expect(Token::Function)?;
        let name = self.expect_name()?;
        // The body block is optional: `function "Name"` declares a bare function.
        let attributes = if self.check(&Token::LeftBrace) {
            self.parse_attributes_block()?
        } else {
            HashMap::new()
        };

        Ok(LogicalFunction { name, attributes })
    }
    
    fn parse_logical_interface(&mut self) -> Result<LogicalInterface, String> {
        self.expect(Token::Interface)?;
        let name = self.expect_name()?;
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
        let name = self.expect_name()?;
        
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
            _ => Err(self.err(format!("Expected identifier or string, got {}", self.current()))),
        }
    }
    
    fn parse_physical_architecture(&mut self) -> Result<PhysicalArchitecture, String> {
        self.expect(Token::PhysicalArchitecture)?;
        let name = self.expect_name()?;
        self.parse_physical_architecture_block(name)
    }
    
    fn parse_physical_architecture_block(&mut self, name: String) -> Result<PhysicalArchitecture, String> {
        self.expect(Token::LeftBrace)?;

        let mut name = name;
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        let mut physical_exchanges = Vec::new();

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
                Token::Identifier(ref id) if id == "physical_link" || id == "link" => {
                    links.push(self.parse_physical_link()?);
                }
                Token::Identifier(ref id) if id == "physical_exchange" => {
                    physical_exchanges.push(self.parse_physical_exchange()?);
                }
                Token::Identifier(ref id) if id == "deployment" => {
                    // deployment "Component" -> "NodeId" [{ ... }]
                    self.advance();
                    let component = self.expect_name()?;
                    self.expect(Token::Arrow)?;
                    let node_ref = self.expect_name()?;
                    let attributes = if self.check(&Token::LeftBrace) {
                        self.parse_attributes_block()?
                    } else {
                        HashMap::new()
                    };
                    match nodes.iter_mut().find(|n| n.id == node_ref || n.name == node_ref) {
                        Some(node) => node.deployments.push(Deployment { component, attributes }),
                        None => {
                            return Err(self.err(format!(
                                "deployment target '{}' does not match any node declared above",
                                node_ref
                            )));
                        }
                    }
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    if key == "name" {
                        if let AttributeValue::String(s) = value {
                            name = s;
                        }
                    } else {
                        self.warn(format!(
                            "attribute '{}' on physical_architecture is not yet stored in the model",
                            key
                        ));
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in physical_architecture: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(PhysicalArchitecture {
            name,
            nodes,
            links,
            physical_exchanges,
        })
    }

    /// Parse: physical_exchange Name { from: "..." to: "..." via: "..." ... }
    fn parse_physical_exchange(&mut self) -> Result<PhysicalExchange, String> {
        self.advance(); // Skip 'physical_exchange'
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;

        let from = attributes
            .get("from")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .ok_or_else(|| self.err(format!("physical_exchange '{}' is missing 'from'", name)))?;
        let to = attributes
            .get("to")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .ok_or_else(|| self.err(format!("physical_exchange '{}' is missing 'to'", name)))?;

        Ok(PhysicalExchange {
            from,
            to,
            via: attributes.get("via").and_then(|v| v.as_string()).map(|s| s.to_string()),
            message_type: attributes
                .get("message_type")
                .and_then(|v| v.as_string())
                .unwrap_or("Data")
                .to_string(),
            frequency: attributes
                .get("frequency")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string()),
            label: Some(name),
        })
    }
    
    fn parse_physical_node(&mut self) -> Result<PhysicalNode, String> {
        self.expect(Token::Node)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut deployments = Vec::new();
        let mut behavior_components = Vec::new();
        let mut hardware_components = Vec::new();
        let mut node_ports = Vec::new();
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
                Token::Port => {
                    node_ports.push(self.parse_physical_port()?);
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in node: {}", self.current())));
                }
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
            ports: node_ports,
            attributes,
        })
    }
    
    fn parse_deployment(&mut self) -> Result<Deployment, String> {
        self.expect(Token::Deploys)?;
        let component = self.expect_name()?;
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
        self.advance(); // Skip 'physical_link' or 'link'
        let name = self.expect_name()?;

        // Arrow form: link "A" -> "B" { ... }
        let mut arrow_endpoints = None;
        if self.check(&Token::Arrow) {
            self.advance();
            let to = self.expect_name()?;
            arrow_endpoints = Some((name.clone(), to));
        }

        let attributes = self.parse_attributes_block()?;

        if let Some((from, to)) = arrow_endpoints {
            return Ok(PhysicalLink {
                from: from.clone(),
                to: to.clone(),
                protocol: attributes
                    .get("protocol")
                    .or_else(|| attributes.get("type"))
                    .and_then(|v| v.as_string())
                    .unwrap_or("Unknown")
                    .to_string(),
                bandwidth: None,
                color: None,
                connections: vec![from, to],
                attributes,
            });
        }

        let mut connections = if let Some(AttributeValue::List(ref connects)) = attributes.get("connects") {
            connects.iter()
                .filter_map(|v| v.as_string().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        };

        let (from, to) = if connections.len() >= 2 {
            (connections[0].clone(), connections[1].clone())
        } else {
            // Alternative form: link Name { from: "..." to: "..." }
            let from = attributes
                .get("from")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let to = attributes
                .get("to")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .unwrap_or_default();
            if !from.is_empty() && !to.is_empty() {
                connections = vec![from.clone(), to.clone()];
            } else {
                return Err(self.err(format!(
                    "link '{}' must declare endpoints via 'connects: [..]' or 'from:'/'to:'",
                    name
                )));
            }
            (from, to)
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
        let name = self.expect_name()?;

        let mut attributes = HashMap::new();
        if self.check(&Token::LeftBrace) {
            self.expect(Token::LeftBrace)?;
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                match self.current() {
                    Token::InterfaceIn | Token::InterfaceOut => {
                        self.warn(
                            "interfaces on physical components are not yet stored in the model",
                        );
                        self.parse_interface_definition()?;
                    }
                    Token::Function => {
                        self.warn(
                            "functions on physical components are not yet stored in the model",
                        );
                        self.parse_logical_function()?;
                    }
                    _ if self.peek_is_colon() => {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    }
                    _ => {
                        return Err(self.err(format!(
                            "Unexpected token in physical component: {}",
                            self.current()
                        )));
                    }
                }
            }
            self.expect(Token::RightBrace)?;
        }
        
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
            ports: Vec::new(),
            attributes,
        })
    }
    
    fn parse_connection_as_physical_link(&mut self) -> Result<PhysicalLink, String> {
        self.expect(Token::Connection)?;
        let name = self.expect_name()?;
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
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut systems: Vec<EpbsSystem> = Vec::new();
        let mut orphan_subsystems = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::System => {
                    systems.push(self.parse_epbs_system()?);
                }
                Token::Subsystem => {
                    // Subsystem declared directly under epbs: grouped under an
                    // implicit system named after the EPBS block.
                    orphan_subsystems.push(self.parse_epbs_subsystem()?);
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in epbs: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        if !orphan_subsystems.is_empty() {
            systems.push(EpbsSystem {
                name: name.clone(),
                subsystems: orphan_subsystems,
                attributes: HashMap::new(),
            });
        }

        Ok(Epbs { name, systems })
    }
    
    fn parse_epbs_system(&mut self) -> Result<EpbsSystem, String> {
        self.expect(Token::System)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut subsystems = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Subsystem => {
                    subsystems.push(self.parse_epbs_subsystem()?);
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in epbs system: {}", self.current())));
                }
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
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut items = Vec::new();
        let mut attributes = HashMap::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Item => {
                    items.push(self.parse_epbs_item()?);
                }
                Token::Identifier(ref id) if id == "assembly" => {
                    // assembly "Name" { ... } is an item grouping
                    self.advance();
                    let item_name = self.expect_name()?;
                    let item_attributes = self.parse_attributes_block()?;
                    items.push(EpbsItem { name: item_name, attributes: item_attributes });
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in epbs subsystem: {}", self.current())));
                }
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
        let name = self.expect_name()?;
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
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in safety_analysis: {}", self.current())));
                }
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
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        
        Ok(Hazard { name, attributes })
    }
    
    fn parse_fmea_entry(&mut self) -> Result<FmeaEntry, String> {
        self.expect(Token::Fmea)?;
        let name = self.expect_name()?;
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
        } else if let Token::Realizes = self.current() {
            self.advance();
            ("realizes".to_string(), self.expect_name()?)
        } else if let Token::Refines = self.current() {
            self.advance();
            ("refines".to_string(), self.expect_name()?)
        } else {
            return Err(self.err("Expected trace type (satisfies, implements, validates, etc.) or arrow"));
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
            // Any keyword can double as an attribute key (via:, type:, from:, ...)
            token => match token.keyword_text() {
                Some(text) => {
                    let k = text.to_string();
                    self.advance();
                    k
                }
                None => {
                    return Err(self.err(format!("Expected attribute key, got: {}", self.current())))
                }
            },
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
            Token::LeftBrace => {
                // Nested attribute map: properties: { cpu: "32 cores" ... }
                let map = self.parse_attributes_block()?;
                Ok(AttributeValue::Map(map))
            }
            Token::Identifier(_) => {
                // Bare identifier values, including dotted refs (A.B.C)
                let value = self.expect_name()?;
                Ok(AttributeValue::String(value))
            }
            _ => Err(self.err(format!("Expected attribute value, got: {}", self.current()))),
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
                return Err(self.err("Expected comma or closing bracket in list"));
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
            Err(self.err(format!("Expected {:?}, got {}", expected, self.current())))
        }
    }
    
    fn expect_string(&mut self) -> Result<String, String> {
        if let Token::StringLiteral(s) = self.current() {
            let result = s.clone();
            self.advance();
            Ok(result)
        } else {
            Err(self.err(format!("Expected string literal, got {}", self.current())))
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
            _ => Err(self.err(format!("Expected identifier, got {}", self.current())))
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
            return Err(self.err(format!("Expected interface_in or interface_out, got {}", self.current())));
        }
        self.advance();
        
        // Optional colon (support both "interface_out: "Name"" and "interface_out "Name"")
        if self.check(&Token::Colon) {
            self.advance();
        }
        
        // Interface name (string literal)
        let name = self.expect_name()?;
        
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
                    if self.peek_is_colon() {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    } else {
                        return Err(self.err(format!(
                            "Unexpected token in interface definition: {}",
                            self.current()
                        )));
                    }
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
    
    fn parse_provides_requires_interface(&mut self, _is_provided: bool) -> Result<InterfaceDefinition, String> {
        // Parse: provides "InterfaceName" { protocol: "CAN" }
        //    or: provides interface IControl { ... }

        // Optional 'interface' keyword
        if self.check(&Token::Interface) {
            self.advance();
        }

        // Interface name (identifier or string literal)
        let name = self.expect_name()?;

        self.expect(Token::LeftBrace)?;

        let mut protocol = None;
        let mut format = None;
        let mut attributes = HashMap::new();

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
                    if self.peek_is_colon() {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    } else {
                        return Err(self.err(format!(
                            "Unexpected token in interface definition: {}",
                            self.current()
                        )));
                    }
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
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        
        let mut attributes = HashMap::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.peek_is_colon() {
                let (key, value) = self.parse_attribute()?;
                attributes.insert(key, value);
            } else {
                return Err(self.err(format!(
                    "Unexpected token in behavior_component: {}",
                    self.current()
                )));
            }
        }

        self.expect(Token::RightBrace)?;

        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("BC-{}", name.chars().take(3).collect::<String>()));
        let color = attributes
            .get("color")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        let mut allocated_functions: Vec<String> =
            if let Some(AttributeValue::List(items)) = attributes.get("allocated_functions") {
                items
                    .iter()
                    .filter_map(|v| v.as_string().map(|s| s.to_string()))
                    .collect()
            } else {
                Vec::new()
            };
        if let Some(single) = attributes
            .get("allocated_component")
            .or_else(|| attributes.get("allocated_function"))
            .and_then(|v| v.as_string())
        {
            allocated_functions.push(single.to_string());
        }

        Ok(BehaviorComponent {
            id,
            name,
            allocated_functions,
            color,
        })
    }
    
    fn parse_hardware_component(&mut self) -> Result<HardwareComponent, String> {
        self.expect(Token::HardwareComponent)?;
        let name = self.expect_name()?;
        
        let mut hw_type = "Generic".to_string();
        let mut specs = None;
        let mut color = None;

        if self.check(&Token::LeftBrace) {
            let attributes = self.parse_attributes_block()?;
            if let Some(t) = attributes.get("type").and_then(|v| v.as_string()) {
                hw_type = t.to_string();
            }
            specs = attributes.get("specs").and_then(|v| v.as_string()).map(|s| s.to_string());
            color = attributes.get("color").and_then(|v| v.as_string()).map(|s| s.to_string());
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
