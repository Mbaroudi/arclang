use super::ast::*;
use super::lexer::{Span, Token};
use std::collections::HashMap;

mod declarations;

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

    /// Parse the whole input as ONE attribute value (`"text"`, `25 ms`,
    /// `[A, B]`, `{ cpu: 4 }`): what may stand after `key:` in a block.
    pub fn parse_value(mut self) -> Result<AttributeValue, String> {
        let value = self.parse_attribute_value()?;
        if self.is_at_end() {
            Ok(value)
        } else {
            Err(self.err(format!("Expected a single value, got {} after it", self.current())))
        }
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
                    // Parse new-style model/system block; keep any imports
                    // declared before the header.
                    let pre_imports = std::mem::take(&mut model.imports);
                    let mut parsed = self.parse_model_block()?;
                    parsed.imports.splice(0..0, pre_imports);
                    return Ok(parsed);
                }
                Token::Requirements => {
                    // Top-level requirements block (alternative syntax 3)
                    let pre_imports = std::mem::take(&mut model.imports);
                    let mut parsed = self.parse_model_with_toplevel_blocks()?;
                    parsed.imports.splice(0..0, pre_imports);
                    return Ok(parsed);
                }
                Token::ImportKw if !self.peek_is_colon() => {
                    model.imports.push(self.parse_import_decl()?);
                }
                Token::Architecture => {
                    // Headerless fragment starting with `architecture <layer>`
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
                Token::SecurityAnalysis => {
                    model.security_analysis.push(self.parse_security_analysis()?);
                }
                Token::SecurityScales => {
                    let scales = self.parse_security_scales()?;
                    if model.security_scales.is_some() {
                        self.warnings.push("security_scales declared more than once; the first declaration is kept".to_string());
                    } else {
                        model.security_scales = Some(scales);
                    }
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
                Token::DiagramKw if !self.peek_is_colon() => {
                    model.diagrams.push(self.parse_diagram()?);
                }
                Token::Class => {
                    model.classes.push(self.parse_class()?);
                }
                Token::Association if !self.peek_is_colon() => {
                    model.associations.push(self.parse_association()?);
                }
                Token::Enumeration => {
                    model.data_types.push(self.parse_data_type(true)?);
                }
                Token::DataType if !self.peek_is_colon() => {
                    model.data_types.push(self.parse_data_type(false)?);
                }
                Token::ExchangeItemKw if !self.peek_is_colon() => {
                    model.exchange_items.push(self.parse_exchange_item()?);
                }
                Token::TestCase if !self.peek_is_colon() => {
                    model.test_cases.push(self.parse_test_case()?);
                }
                Token::ConstraintKw if !self.peek_is_colon() => {
                    model.constraints.push(self.parse_constraint()?);
                }
                Token::Type if !self.peek_is_colon() => {
                    model.types.push(self.parse_type_def()?);
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
                Token::SecurityAnalysis => {
                    model.security_analysis.push(self.parse_security_analysis()?);
                }
                Token::SecurityScales => {
                    let scales = self.parse_security_scales()?;
                    if model.security_scales.is_some() {
                        self.warnings.push("security_scales declared more than once; the first declaration is kept".to_string());
                    } else {
                        model.security_scales = Some(scales);
                    }
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
                Token::DiagramKw if !self.peek_is_colon() => {
                    model.diagrams.push(self.parse_diagram()?);
                }
                Token::Class => {
                    model.classes.push(self.parse_class()?);
                }
                Token::Association if !self.peek_is_colon() => {
                    model.associations.push(self.parse_association()?);
                }
                Token::Enumeration => {
                    model.data_types.push(self.parse_data_type(true)?);
                }
                Token::DataType if !self.peek_is_colon() => {
                    model.data_types.push(self.parse_data_type(false)?);
                }
                Token::ExchangeItemKw if !self.peek_is_colon() => {
                    model.exchange_items.push(self.parse_exchange_item()?);
                }
                Token::TestCase if !self.peek_is_colon() => {
                    model.test_cases.push(self.parse_test_case()?);
                }
                Token::ConstraintKw if !self.peek_is_colon() => {
                    model.constraints.push(self.parse_constraint()?);
                }
                Token::Type if !self.peek_is_colon() => {
                    model.types.push(self.parse_type_def()?);
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
                Token::ImportKw if !self.peek_is_colon() => {
                    model.imports.push(self.parse_import_decl()?);
                }
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
                Token::SecurityAnalysis => {
                    model.security_analysis.push(self.parse_security_analysis()?);
                }
                Token::SecurityScales => {
                    let scales = self.parse_security_scales()?;
                    if model.security_scales.is_some() {
                        self.warnings.push("security_scales declared more than once; the first declaration is kept".to_string());
                    } else {
                        model.security_scales = Some(scales);
                    }
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
                Token::DiagramKw if !self.peek_is_colon() => {
                    model.diagrams.push(self.parse_diagram()?);
                }
                Token::Class => {
                    model.classes.push(self.parse_class()?);
                }
                Token::Association if !self.peek_is_colon() => {
                    model.associations.push(self.parse_association()?);
                }
                Token::Enumeration => {
                    model.data_types.push(self.parse_data_type(true)?);
                }
                Token::DataType if !self.peek_is_colon() => {
                    model.data_types.push(self.parse_data_type(false)?);
                }
                Token::ExchangeItemKw if !self.peek_is_colon() => {
                    model.exchange_items.push(self.parse_exchange_item()?);
                }
                Token::TestCase if !self.peek_is_colon() => {
                    model.test_cases.push(self.parse_test_case()?);
                }
                Token::ConstraintKw if !self.peek_is_colon() => {
                    model.constraints.push(self.parse_constraint()?);
                }
                Token::Type if !self.peek_is_colon() => {
                    model.types.push(self.parse_type_def()?);
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

    /// Whether the token after the current one is a port direction (`in`, `out`, `inout`).
    fn peek_is_port_direction(&self) -> bool {
        matches!(self.tokens.get(self.position + 1), Some(Token::In | Token::Out)) || matches!(self.tokens.get(self.position + 1), Some(Token::Identifier(d)) if d == "inout")
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
                Token::SecurityAnalysis => {
                    model.security_analysis.push(self.parse_security_analysis()?);
                }
                Token::SecurityScales => {
                    let scales = self.parse_security_scales()?;
                    if model.security_scales.is_some() {
                        self.warnings.push("security_scales declared more than once; the first declaration is kept".to_string());
                    } else {
                        model.security_scales = Some(scales);
                    }
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
                Token::DiagramKw if !self.peek_is_colon() => {
                    model.diagrams.push(self.parse_diagram()?);
                }
                Token::Class => {
                    model.classes.push(self.parse_class()?);
                }
                Token::Association if !self.peek_is_colon() => {
                    model.associations.push(self.parse_association()?);
                }
                Token::Enumeration => {
                    model.data_types.push(self.parse_data_type(true)?);
                }
                Token::DataType if !self.peek_is_colon() => {
                    model.data_types.push(self.parse_data_type(false)?);
                }
                Token::ExchangeItemKw if !self.peek_is_colon() => {
                    model.exchange_items.push(self.parse_exchange_item()?);
                }
                Token::TestCase if !self.peek_is_colon() => {
                    model.test_cases.push(self.parse_test_case()?);
                }
                Token::ConstraintKw if !self.peek_is_colon() => {
                    model.constraints.push(self.parse_constraint()?);
                }
                Token::Type if !self.peek_is_colon() => {
                    model.types.push(self.parse_type_def()?);
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
        let mut level_attributes: HashMap<String, AttributeValue> = HashMap::new();
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
        let mut processes = Vec::new();
        let mut communication_means = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Actor => {
                    actors.push(self.parse_actor()?);
                }
                Token::Identifier(ref id) if id == "operational_entity" || id == "entity" => {
                    entities.push(self.parse_operational_entity()?);
                }
                Token::Identifier(ref id) if id == "operational_capability" => {
                    capabilities.extend(self.parse_operational_capabilities()?);
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
                Token::OperationalProcess => {
                    self.advance();
                    let process_name = self.expect_name()?;
                    let (attributes, controls, sequences) = self.parse_chain_body("operational_process", &process_name)?;
                    let id = attributes
                        .get("id")
                        .and_then(|v| v.as_string())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| process_name.clone());
                    let involves = Self::string_list(&attributes, "involves");
                    if involves.is_empty() {
                        self.warn(format!("operational_process '{process_name}' involves nothing yet (involves: [...])"));
                    }
                    processes.push(Self::chain(id, process_name, involves, controls, sequences, attributes));
                }
                Token::CommunicationMeans => {
                    self.advance();
                    let cm_name = self.expect_name()?;
                    let attributes = self.parse_attributes_block()?;
                    let from = attributes.get("from").and_then(|v| v.as_string()).map(|s| s.to_string())
                        .ok_or_else(|| self.err(format!("communication_means '{}' is missing 'from'", cm_name)))?;
                    let to = attributes.get("to").and_then(|v| v.as_string()).map(|s| s.to_string())
                        .ok_or_else(|| self.err(format!("communication_means '{}' is missing 'to'", cm_name)))?;
                    communication_means.push(OperationalExchange {
                        from,
                        to,
                        data_type: "CommunicationMeans".to_string(),
                        label: Some(cm_name),
                        protocol: attributes.get("protocol").and_then(|v| v.as_string()).map(|s| s.to_string()),
                        attributes,
                    });
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    if key == "name" {
                        if let AttributeValue::String(s) = value {
                            name = s;
                        }
                    } else {
                        level_attributes.insert(key, value);
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in operational_analysis: {}", self.current())));
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(OperationalAnalysis {
            id: level_attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default(),
            attributes: level_attributes,
            name,
            processes,
            communication_means,
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
            .unwrap_or_else(|| default_element_id("OE", &name));
        
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
    
    /// Parse: operational_activity Name { attributes, port in|out Name, nested activities }
    fn parse_operational_activity(&mut self) -> Result<OperationalActivity, String> {
        self.advance(); // Skip 'operational_activity'
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        let mut attributes = HashMap::new();
        let mut ports = Vec::new();
        let mut sub_activities = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current().clone() {
                Token::Port if !self.peek_is_colon() => ports.push(self.parse_function_port()?),
                Token::Identifier(ref id) if (id == "operational_activity" || id == "activity") && !self.peek_is_colon() => {
                    sub_activities.push(self.parse_operational_activity()?);
                }
                _ => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
            }
        }
        self.expect(Token::RightBrace)?;
        
        // Extract ID from attributes, or generate from name
        let id = attributes.get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| default_element_id("OA", &name));
        
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
            sub_activities,
            ports,
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
        let mut level_attributes: HashMap<String, AttributeValue> = HashMap::new();
        self.expect(Token::SystemAnalysis)?;
        let mut name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;

        let mut requirements = Vec::new();
        let mut functions = Vec::new();
        let mut components = Vec::new();
        let mut external_actors = Vec::new();
        let mut functional_exchanges = Vec::new();
        let mut component_exchanges = Vec::new();
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
                    capabilities.extend(self.parse_capabilities()?);
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
                Token::Identifier(ref id) if id == "component_exchange" => {
                    component_exchanges.push(self.parse_named_component_exchange()?);
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
                        level_attributes.insert(key, value);
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in system_analysis: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(SystemAnalysis {
            id: level_attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default(),
            attributes: level_attributes,
            name,
            requirements,
            functions,
            components,
            external_actors,
            functional_exchanges,
            component_exchanges,
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

    /// Parse: functional_chain Name { id: ... involves: ["F1", "FE1", "F2"]
    /// control and "Split" { id: ... } sequence "F1" -> "Split" { condition: "..." } }
    fn parse_functional_chain(&mut self) -> Result<FunctionalChain, String> {
        self.expect(Token::FunctionalChain)?;
        let name = self.expect_name()?;
        let (attributes, controls, sequences) = self.parse_chain_body("functional_chain", &name)?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("FC-{}", name.replace(' ', "_")));
        let involves = Self::string_list(&attributes, "involves");
        if involves.is_empty() {
            // Capella keeps chains whose path is not described yet.
            self.warn(format!("functional_chain '{name}' involves nothing yet (involves: [...])"));
        }
        Ok(Self::chain(id, name, involves, controls, sequences, attributes))
    }

    /// The body of a chain or an operational process: attributes, control
    /// nodes and sequence links, in any order.
    fn parse_chain_body(&mut self, what: &str, name: &str) -> Result<(HashMap<String, AttributeValue>, Vec<ControlNode>, Vec<SequenceLink>), String> {
        self.expect(Token::LeftBrace)?;
        let mut attributes = HashMap::new();
        let mut controls: Vec<ControlNode> = Vec::new();
        let mut sequences: Vec<SequenceLink> = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Control if !self.peek_is_colon() => controls.push(self.parse_control_node(name)?),
                Token::Sequence if !self.peek_is_colon() => sequences.push(self.parse_sequence_link(name)?),
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                other => return Err(self.err(format!("Unexpected token in {what} '{name}': {other}"))),
            }
        }
        self.expect(Token::RightBrace)?;
        Ok((attributes, controls, sequences))
    }

    /// A chain whose control nodes and links get their default ids under
    /// the chain's id (known only once the attributes are read).
    fn chain(id: String, name: String, involves: Vec<String>, controls: Vec<ControlNode>, sequences: Vec<SequenceLink>, attributes: HashMap<String, AttributeValue>) -> FunctionalChain {
        let controls = controls
            .into_iter()
            .map(|control| ControlNode { id: if control.id.is_empty() { format!("{id}.{}", control.name) } else { control.id }, ..control })
            .collect();
        let sequences = sequences
            .into_iter()
            .map(|link| SequenceLink { id: if link.id.is_empty() { format!("{id}.{}->{}", link.from, link.to) } else { link.id }, ..link })
            .collect();
        FunctionalChain { id, name, involves, controls, sequences, attributes }
    }

    /// Parse: control and|or|iterate "Name" { id: "..." } — a control node of
    /// a chain. The id stays empty until the chain's id is known.
    fn parse_control_node(&mut self, chain: &str) -> Result<ControlNode, String> {
        self.expect(Token::Control)?;
        let kind_word = match self.current() {
            Token::Identifier(word) => word.clone(),
            other => return Err(self.err(format!("functional_chain '{chain}': expected a control kind (and, or, iterate), got {other}"))),
        };
        let kind = ControlKind::parse(&kind_word)
            .ok_or_else(|| self.err(format!("functional_chain '{chain}': '{kind_word}' is not a control kind: use and, or or iterate")))?;
        self.advance();
        let name = self.expect_name()?;
        let attributes = if self.check(&Token::LeftBrace) { self.parse_attributes_block()? } else { HashMap::new() };
        let id = attributes.get("id").and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_default();
        Ok(ControlNode { id, name, kind, attributes })
    }

    /// Parse: sequence "From" -> "To" { condition: "..." id: "..." } — a
    /// sequence link of a chain, between steps and control nodes.
    fn parse_sequence_link(&mut self, _chain: &str) -> Result<SequenceLink, String> {
        self.expect(Token::Sequence)?;
        let from = self.expect_name()?;
        self.expect(Token::Arrow)?;
        let to = self.expect_name()?;
        let attributes = if self.check(&Token::LeftBrace) { self.parse_attributes_block()? } else { HashMap::new() };
        let text = |key: &str| attributes.get(key).and_then(|v| v.as_string()).map(|s| s.to_string());
        Ok(SequenceLink { id: text("id").unwrap_or_default(), from, to, condition: text("condition"), attributes })
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
        let id = attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default();
        Ok(FunctionPort { id, name, direction, port_type, data_type, attributes })
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
        let id = attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default();
        Ok(ComponentPort { id, name, direction, interface_type, attributes })
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
        let mut regions: Vec<Region> = Vec::new();
        let mut attributes: HashMap<String, AttributeValue> = HashMap::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::StateKw | Token::Mode => {
                    states.push(self.parse_state(&mut transitions)?);
                }
                Token::Transition => {
                    transitions.push(self.parse_transition()?);
                }
                Token::Region if !self.peek_is_colon() => {
                    // A region of its own: its states and transitions join the
                    // machine's flat lists, its initial state is its own.
                    let region = self.parse_region(&name)?;
                    states.extend(region.states.iter().cloned());
                    transitions.extend(region.transitions.iter().cloned());
                    if initial_state.is_empty() {
                        initial_state = region.initial.clone();
                    }
                    regions.push(region);
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    if key == "initial" {
                        if let AttributeValue::String(s) = value {
                            initial_state = s;
                        }
                    } else {
                        attributes.insert(key, value);
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in state_machine: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let id = attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default();
        Ok(StateMachine { id, name, attributes, initial_state, states, transitions, regions })
    }

    /// Parse: region "Name" { id: "..." initial: "A" state "A" {} transition A -> B {} }
    fn parse_region(&mut self, machine: &str) -> Result<Region, String> {
        self.expect(Token::Region)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        let mut states = Vec::new();
        let mut transitions = Vec::new();
        let mut attributes: HashMap<String, AttributeValue> = HashMap::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::StateKw | Token::Mode => states.push(self.parse_state(&mut transitions)?),
                Token::Transition => transitions.push(self.parse_transition()?),
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                other => return Err(self.err(format!("Unexpected token in region '{name}' of state_machine '{machine}': {other}"))),
            }
        }
        self.expect(Token::RightBrace)?;
        let text = |key: &str| attributes.get(key).and_then(|v| v.as_string()).map(str::to_string);
        Ok(Region {
            id: text("id").unwrap_or_else(|| format!("{machine}.{name}")),
            name,
            initial: text("initial").unwrap_or_default(),
            states,
            transitions,
            attributes,
        })
    }

    /// Parse: diagram "Name" { view: lab shows: [A, B] context: true }
    fn parse_diagram(&mut self) -> Result<DiagramDecl, String> {
        self.expect(Token::DiagramKw)?;
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let view = attributes
            .get("view")
            .and_then(|value| value.as_string())
            .map(str::to_string)
            .ok_or_else(|| {
                self.err(format!(
                    "diagram '{name}' needs `view: lab` (one of oab, sab, lab, pab, cap, cdb, pbs)"
                ))
            })?;
        let shows = match attributes.get("shows") {
            Some(AttributeValue::List(items)) => items
                .iter()
                .filter_map(|item| item.as_string().map(str::to_string))
                .collect(),
            Some(AttributeValue::String(one)) => vec![one.clone()],
            Some(_) => return Err(self.err(format!("diagram '{name}': `shows` is a list of element ids or names"))),
            None => Vec::new(),
        };
        let context = match attributes.get("context") {
            Some(AttributeValue::Boolean(flag)) => *flag,
            Some(AttributeValue::String(text)) => text == "true",
            _ => false,
        };
        let description = attributes.get("description").and_then(|value| value.as_string()).map(str::to_string);
        Ok(DiagramDecl { name, view, shows, context, description, attributes })
    }

    /// Parse: state Name { entry: [...] exit: [...] state Sub { ... } transition A -> B { ... } }
    /// A state holds a region: states, modes and transitions of its own.
    /// Transitions declared inside join the machine's, resolved by name.
    fn parse_state(&mut self, transitions: &mut Vec<Transition>) -> Result<State, String> {
        let kind = if matches!(self.current(), Token::Mode) { StateKind::Mode } else { StateKind::State };
        self.advance();
        let name = self.expect_name()?;
        let mut attributes = HashMap::new();
        let mut sub_states = Vec::new();
        let mut own_transitions: Vec<Transition> = Vec::new();
        if self.check(&Token::LeftBrace) {
            self.expect(Token::LeftBrace)?;
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                match self.current() {
                    Token::StateKw | Token::Mode => sub_states.push(self.parse_state(transitions)?),
                    Token::Transition => {
                        // Owned by this region, and in the machine's flat list.
                        let transition = self.parse_transition()?;
                        own_transitions.push(transition.clone());
                        transitions.push(transition);
                    }
                    _ if self.peek_is_colon() => {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    }
                    _ => return Err(self.err(format!("Unexpected token in state '{name}': {}", self.current()))),
                }
            }
            self.expect(Token::RightBrace)?;
        }
        Ok(State {
            id: attributes.get("id").and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_default(),
            name,
            kind,
            entry_actions: Self::string_list(&attributes, "entry"),
            exit_actions: Self::string_list(&attributes, "exit"),
            internal_transitions: Vec::new(),
            sub_states,
            transitions: own_transitions,
            color: attributes.get("color").and_then(|v| v.as_string()).map(|s| s.to_string()),
            initial: attributes.get("initial").and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_default(),
            attributes,
        })
    }

    fn parse_transition(&mut self) -> Result<Transition, String> {
        self.expect(Token::Transition)?;
        let from = self.expect_name()?;
        self.expect(Token::Arrow)?;
        let to = self.expect_name()?;
        let attributes = if self.check(&Token::LeftBrace) { self.parse_attributes_block()? } else { HashMap::new() };
        let get = |key: &str| attributes.get(key).and_then(|v| v.as_string()).map(|s| s.to_string());
        Ok(Transition {
            id: get("id").unwrap_or_default(),
            from,
            to,
            trigger: get("trigger").unwrap_or_default(),
            guard: get("guard"),
            action: get("action").or_else(|| {
                let list = Self::string_list(&attributes, "action");
                if list.is_empty() { None } else { Some(list.join(", ")) }
            }),
            timing: get("timing"),
            priority: get("priority"),
            attributes,
        })
    }

    /// Parse: message A -> B "label" { type: async }
    fn parse_message(&mut self) -> Result<Message, String> {
        self.expect(Token::Message)?;
        let from = self.expect_name()?;
        self.expect(Token::Arrow)?;
        let to = self.expect_name()?;
        let label = self.optional_name().unwrap_or_default();
        let attributes = if self.check(&Token::LeftBrace) { self.parse_attributes_block()? } else { HashMap::new() };
        let message_type = match attributes.get("type").and_then(|v| v.as_string()) {
            Some("async") | Some("asynchronous") => MessageType::Asynchronous,
            _ => MessageType::Synchronous,
        };
        Ok(Message {
            id: attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default(),
            from,
            to,
            label,
            message_type,
            activation: true,
            timing: attributes.get("timing").and_then(|v| v.as_string()).map(|s| s.to_string()),
            params: None,
            attributes,
        })
    }

    /// Parse: fragment alt "label" { operand "a" { message ... } operand "b" { ... } }
    /// or fragment loop "while armed" { message ... }. Messages land in the
    /// scenario's flat list, in order; the fragment keeps their rows.
    fn parse_fragment(&mut self, messages: &mut Vec<Message>, invariants: &mut Vec<StateInvariant>) -> Result<CombinedFragment, String> {
        self.expect(Token::Fragment)?;
        let kind_word = match self.current() {
            Token::Identifier(word) => word.clone(),
            other => return Err(self.err(format!("Expected a fragment kind (alt, opt, loop, par, seq, critical), got {other}"))),
        };
        let fragment_type = FragmentType::parse(&kind_word)
            .ok_or_else(|| self.err(format!("'{kind_word}' is not a fragment kind: use alt, opt, loop, par, seq or critical")))?;
        self.advance();
        let label = match self.current() {
            Token::StringLiteral(text) => { let text = text.clone(); self.advance(); text }
            _ => String::new(),
        };
        self.expect(Token::LeftBrace)?;
        let start = messages.len();
        let mut operands: Vec<FragmentOperand> = Vec::new();
        let mut implicit = FragmentOperand { label: String::new(), messages: Vec::new(), start: messages.len(), end: 0, fragments: Vec::new(), invariants: Vec::new() };
        let mut explicit = false;
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current() {
                Token::Operand => {
                    explicit = true;
                    self.advance();
                    let operand_label = match self.current() {
                        Token::StringLiteral(text) => { let text = text.clone(); self.advance(); text }
                        _ => String::new(),
                    };
                    self.expect(Token::LeftBrace)?;
                    let mut operand = FragmentOperand { label: operand_label, messages: Vec::new(), start: messages.len(), end: 0, fragments: Vec::new(), invariants: Vec::new() };
                    while !self.check(&Token::RightBrace) && !self.is_at_end() {
                        match self.current() {
                            Token::Message => { let message = self.parse_message()?; operand.messages.push(message.clone()); messages.push(message); }
                            Token::Fragment => { let nested = self.parse_fragment(messages, invariants)?; operand.fragments.push(nested); }
                            Token::StateKw | Token::Mode | Token::Function => { let at = self.parse_invariant(messages, invariants, &mut operand.fragments)?; operand.invariants.push(at); }
                            other => return Err(self.err(format!("Unexpected token in operand: {other}"))),
                        }
                    }
                    self.expect(Token::RightBrace)?;
                    operand.end = messages.len().wrapping_sub(1);
                    operands.push(operand);
                }
                Token::Message => { let message = self.parse_message()?; implicit.messages.push(message.clone()); messages.push(message); }
                Token::Fragment => { let nested = self.parse_fragment(messages, invariants)?; implicit.fragments.push(nested); }
                Token::StateKw | Token::Mode | Token::Function => { let at = self.parse_invariant(messages, invariants, &mut implicit.fragments)?; implicit.invariants.push(at); }
                other => return Err(self.err(format!("Unexpected token in fragment: {other}"))),
            }
        }
        self.expect(Token::RightBrace)?;
        if !explicit || !implicit.messages.is_empty() || !implicit.fragments.is_empty() {
            implicit.end = messages.len().wrapping_sub(1);
            if explicit {
                return Err(self.err("a fragment mixes operand blocks and loose messages: put every message in an operand".to_string()));
            }
            operands.push(implicit);
        }
        Ok(CombinedFragment { fragment_type, label, condition: None, operands, start, end: messages.len().wrapping_sub(1) })
    }

    /// Parse: state "Lifeline" "Armed" { id: "..." message ... } — a state,
    /// mode or function asserted on a lifeline. The messages of its body
    /// land in the scenario's flat list; the invariant keeps their rows.
    /// Fragments opened in its body belong to the enclosing frame.
    fn parse_invariant(&mut self, messages: &mut Vec<Message>, invariants: &mut Vec<StateInvariant>, fragments: &mut Vec<CombinedFragment>) -> Result<usize, String> {
        let kind = match self.current() {
            Token::Function => InvariantKind::Function,
            _ => InvariantKind::State,
        };
        self.advance();
        let lifeline = self.expect_name()?;
        let name = self.expect_name()?;
        let mut attributes = HashMap::new();
        let start = messages.len();
        let at = invariants.len();
        // Reserve the slot: an invariant comes before those its body holds.
        invariants.push(StateInvariant { id: String::new(), lifeline: lifeline.clone(), name: name.clone(), kind, start, end: 0, attributes: HashMap::new() });
        if self.check(&Token::LeftBrace) {
            self.advance();
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                match self.current() {
                    Token::Message => messages.push(self.parse_message()?),
                    Token::Fragment => { let nested = self.parse_fragment(messages, invariants)?; fragments.push(nested); }
                    Token::StateKw | Token::Mode | Token::Function => { self.parse_invariant(messages, invariants, fragments)?; }
                    _ if self.peek_is_colon() => {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    }
                    other => return Err(self.err(format!("Unexpected token in {} '{name}' on '{lifeline}': {other}", kind.keyword()))),
                }
            }
            self.expect(Token::RightBrace)?;
        }
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{lifeline}.{name}#{}", at + 1));
        invariants[at] = StateInvariant { id, lifeline, name, kind, start, end: messages.len().wrapping_sub(1), attributes };
        Ok(at)
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
        let mut fragments = Vec::new();
        let mut invariants: Vec<StateInvariant> = Vec::new();
        let mut scenario_attributes: HashMap<String, AttributeValue> = HashMap::new();

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
                    messages.push(self.parse_message()?);
                }
                Token::Fragment => {
                    let fragment = self.parse_fragment(&mut messages, &mut invariants)?;
                    fragments.push(fragment);
                }
                Token::StateKw | Token::Mode | Token::Function => {
                    let _ = self.parse_invariant(&mut messages, &mut invariants, &mut fragments)?;
                }
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    scenario_attributes.insert(key, value);
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in scenario: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        // An invariant's id defaults under its scenario: the same lifeline
        // asserts the same state in many scenarios.
        let invariants = invariants
            .into_iter()
            .map(|invariant| StateInvariant { id: if invariant.attributes.contains_key("id") { invariant.id } else { format!("{name}.{}", invariant.id) }, ..invariant })
            .collect();
        Ok(Scenario {
            id: scenario_attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default(),
            name,
            attributes: scenario_attributes,
            participants,
            messages,
            fragments,
            invariants,
            timing_constraints: Vec::new(),
        })
    }

    /// Parse a number and, when a unit follows, the typed quantity:
    /// the second value is the integer written when the literal had no
    /// decimal point (`800` gives `Some(800)`, `800.0` gives `None`).
    /// `25 ms`, `100 Mbps`, `50 km/h`, `40 %`. A unit is an identifier right after
    /// the number that is NOT the key of the next attribute (not followed by
    /// ':'). An identifier that is not a known unit is an error.
    fn parse_number_or_quantity(&mut self) -> Result<(f64, Option<i64>, Option<super::quantity::Quantity>), String> {
        let (n, integer) = match self.current() {
            Token::Number(n) => (*n, None),
            Token::Integer(i) => (*i as f64, Some(*i)),
            other => return Err(self.err(format!("Expected a number, got {}", other))),
        };
        self.advance();
        if matches!(self.tokens.get(self.position), Some(Token::Percent)) {
            let quantity = super::quantity::Quantity::new(n, "%").map_err(|e| self.err(e.to_string()))?;
            self.advance();
            return Ok((n, integer, Some(quantity)));
        }
        let Some(Token::Identifier(unit)) = self.tokens.get(self.position) else {
            return Ok((n, integer, None));
        };
        if matches!(self.tokens.get(self.position + 1), Some(Token::Colon)) {
            return Ok((n, integer, None));
        }
        let unit = unit.clone();
        // Compound symbols lex as three tokens: `km` `/` `h`.
        if let (Some(Token::Slash), Some(Token::Identifier(denominator))) =
            (self.tokens.get(self.position + 1), self.tokens.get(self.position + 2))
        {
            let compound = format!("{}/{}", unit, denominator);
            if let Ok(quantity) = super::quantity::Quantity::new(n, &compound) {
                self.advance();
                self.advance();
                self.advance();
                return Ok((n, integer, Some(quantity)));
            }
        }
        let quantity = super::quantity::Quantity::new(n, &unit).map_err(|_| {
            self.err(format!(
                "unknown unit '{}' in quantity '{} {}' (known units include ms, s, Hz, Mbps, MB, m, kg, N, V, W)",
                unit, n, unit
            ))
        })?;
        self.advance();
        Ok((n, integer, Some(quantity)))
    }

    /// Parse: type Name [extends Base] { required: [..]  attr: value  port in|out|inout Name {..} }
    fn parse_type_def(&mut self) -> Result<TypeDef, String> {
        self.expect(Token::Type)?;
        let name = self.expect_name()?;
        let extends = if matches!(self.current(), Token::Identifier(word) if word == "extends") {
            self.advance();
            Some(self.expect_name()?)
        } else {
            None
        };
        self.expect(Token::LeftBrace)?;
        let mut attributes = HashMap::new();
        let mut ports = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Port) && !self.peek_is_colon() {
                ports.push(self.parse_component_port()?);
            } else {
                let (key, value) = self.parse_attribute()?;
                attributes.insert(key, value);
            }
        }
        self.expect(Token::RightBrace)?;
        if attributes.contains_key("is") {
            return Err(self.err(format!(
                "type '{}': a type specializes with `type {} extends Base {{ ... }}`, not with `is:`",
                name, name
            )));
        }
        let required = Self::string_list(&attributes, "required");
        Ok(TypeDef { name, extends, attributes, ports, required })
    }

    /// Parse: constraint Name { id: "CST-1" description: "..." assert: <expression> }
    fn parse_constraint(&mut self) -> Result<Constraint, String> {
        self.expect(Token::ConstraintKw)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;
        let mut attributes = HashMap::new();
        let mut expression = None;
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let is_assert = matches!(self.current(), Token::Identifier(key) if key == "assert") && self.peek_is_colon();
            if is_assert {
                if expression.is_some() {
                    return Err(self.err(format!("constraint '{}' has more than one `assert:`", name)));
                }
                self.advance(); // assert
                self.advance(); // :
                expression = Some(self.parse_expression()?);
            } else {
                let (key, value) = self.parse_attribute()?;
                attributes.insert(key, value);
            }
        }
        self.expect(Token::RightBrace)?;
        let expression = expression
            .ok_or_else(|| self.err(format!("constraint '{}' declares no `assert: <expression>`", name)))?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());
        Ok(Constraint { id, name, expression, attributes })
    }

    /// expression := additive ( ("<"|"<="|">"|">="|"=="|"!=") additive )?
    fn parse_expression(&mut self) -> Result<Expr, String> {
        let lhs = self.parse_additive()?;
        let op = match self.current() {
            Token::Less => BinOp::Lt,
            Token::LessEq => BinOp::Le,
            Token::Greater => BinOp::Gt,
            Token::GreaterEq => BinOp::Ge,
            Token::EqualEq => BinOp::Eq,
            Token::NotEq => BinOp::Ne,
            _ => return Ok(lhs),
        };
        self.advance();
        let rhs = self.parse_additive()?;
        Ok(Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) })
    }

    /// additive := term ( ("+"|"-") term )*
    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_term()?;
        loop {
            let op = match self.current() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                // The lexer reads `-5` as one negative number; after an
                // operand that is a subtraction (`a -5 ms` = `a - 5 ms`).
                Token::Number(_) | Token::Integer(_) if self.current_is_negative_number() => {
                    let (n, _, quantity) = self.parse_number_or_quantity()?;
                    let rhs = match quantity {
                        Some(q) => Expr::Quantity(super::quantity::Quantity { value: -q.value, unit: q.unit }),
                        None => Expr::Number(-n),
                    };
                    lhs = Expr::Binary { op: BinOp::Sub, lhs: Box::new(lhs), rhs: Box::new(rhs) };
                    continue;
                }
                _ => return Ok(lhs),
            };
            self.advance();
            let rhs = self.parse_term()?;
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
    }

    /// term := unary ( ("*"|"/") unary )*
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.current() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => return Ok(lhs),
            };
            self.advance();
            let rhs = self.parse_unary()?;
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
    }

    /// unary := "-" unary | primary
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.check(&Token::Minus) {
            self.advance();
            return Ok(Expr::Neg(Box::new(self.parse_unary()?)));
        }
        self.parse_primary()
    }

    /// primary := Number Unit? | Name "." Attribute | Name | Function "(" args ")" | "(" expression ")"
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current().clone() {
            Token::Number(_) | Token::Integer(_) => Ok(match self.parse_number_or_quantity()? {
                (n, _, None) => Expr::Number(n),
                (_, _, Some(quantity)) => Expr::Quantity(quantity),
            }),
            Token::LeftParen => {
                self.advance();
                let inner = self.parse_additive()?;
                if !self.check(&Token::RightParen) {
                    return Err(self.err(format!("Expected ')', got {}", self.current())));
                }
                self.advance();
                Ok(inner)
            }
            Token::Identifier(function) if matches!(self.tokens.get(self.position + 1), Some(Token::LeftParen)) => {
                self.advance(); // function name
                self.advance(); // (
                let mut args = Vec::new();
                while !self.check(&Token::RightParen) && !self.is_at_end() {
                    args.push(self.parse_additive()?);
                    if self.check(&Token::Comma) {
                        self.advance();
                    } else if !self.check(&Token::RightParen) {
                        return Err(self.err(format!("Expected ',' or ')' in arguments of {}(), got {}", function, self.current())));
                    }
                }
                if !self.check(&Token::RightParen) {
                    return Err(self.err(format!("Unclosed argument list of {}()", function)));
                }
                self.advance();
                Ok(Expr::Call { function, args })
            }
            Token::Identifier(_) | Token::StringLiteral(_) => {
                let was_string = matches!(self.current(), Token::StringLiteral(_));
                let name = self.expect_name()?;
                // `.attribute` where the attribute is a keyword token (latency, type...)
                if self.check(&Token::Dot) {
                    let attribute = match self.tokens.get(self.position + 1) {
                        Some(Token::Identifier(attribute)) => Some(attribute.clone()),
                        Some(token) => token.keyword_text().map(str::to_string),
                        None => None,
                    };
                    if let Some(attribute) = attribute {
                        self.advance();
                        self.advance();
                        return Ok(Expr::Attr { element: name, attribute });
                    }
                    return Err(self.err("Expected an attribute name after '.'"));
                }
                // Bare dotted identifier: the last segment is the attribute.
                if !was_string {
                    if let Some((element, attribute)) = name.rsplit_once('.') {
                        return Ok(Expr::Attr { element: element.to_string(), attribute: attribute.to_string() });
                    }
                }
                Ok(Expr::Ref(name))
            }
            // A keyword used as a bare name (`sum(chain, latency)`).
            token => match token.keyword_text() {
                Some(text) => {
                    self.advance();
                    Ok(Expr::Ref(text.to_string()))
                }
                None => Err(self.err(format!("Expected a value, a name or '(' in expression, got {}", token))),
            },
        }
    }

    /// Parse: test_case Name { verifies: ["REQ-1"] method: "test" description: "..." }
    fn parse_test_case(&mut self) -> Result<TestCase, String> {
        self.expect(Token::TestCase)?;
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());
        let verifies = Self::string_list(&attributes, "verifies");
        if verifies.is_empty() {
            return Err(self.err(format!(
                "test_case '{}' must verify at least one requirement (verifies: [...])",
                name
            )));
        }
        let method = attributes
            .get("method")
            .and_then(|v| v.as_string())
            .unwrap_or("test")
            .to_lowercase();
        const METHODS: [&str; 4] = ["test", "analysis", "inspection", "demonstration"];
        if !METHODS.contains(&method.as_str()) {
            return Err(self.err(format!(
                "test_case '{}': method '{}' is not one of test|analysis|inspection|demonstration",
                name, method
            )));
        }
        Ok(TestCase { id, name, verifies, method, attributes })
    }

    /// Parse: enumeration Name { values: ["A", "B"] } or data_type Name { base: "float" unit: "m/s" }.
    fn parse_data_type(&mut self, is_enumeration: bool) -> Result<DataType, String> {
        self.advance(); // Skip 'enumeration' or 'data_type'
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());
        let enumeration_values = if is_enumeration {
            Some(
                Self::string_list(&attributes, "values")
                    .into_iter()
                    .map(|v| EnumValue { name: v, value: None })
                    .collect(),
            )
        } else {
            None
        };
        Ok(DataType {
            id,
            name,
            base_type: attributes.get("base").and_then(|v| v.as_string()).map(|s| s.to_string()),
            unit: attributes.get("unit").and_then(|v| v.as_string()).map(|s| s.to_string()),
            enumeration_values,
            attributes,
        })
    }

    /// Parse: exchange_item Name { mechanism: FLOW elements: ["RadarFrame"] }.
    fn parse_exchange_item(&mut self) -> Result<ExchangeItem, String> {
        self.expect(Token::ExchangeItemKw)?;
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());
        let mechanism = attributes
            .get("mechanism")
            .and_then(|v| v.as_string())
            .unwrap_or("DATA")
            .to_uppercase();
        const MECHANISMS: [&str; 5] = ["EVENT", "FLOW", "OPERATION", "DATA", "SHARED_DATA"];
        if !MECHANISMS.contains(&mechanism.as_str()) {
            return Err(self.err(format!(
                "exchange_item '{}': mechanism '{}' is not one of EVENT|FLOW|OPERATION|DATA|SHARED_DATA (Arcadia)",
                name, mechanism
            )));
        }
        Ok(ExchangeItem {
            id,
            name,
            stereotype: mechanism,
            attributes: Vec::new(),
            elements: Self::string_list(&attributes, "elements"),
            properties: attributes,
        })
    }

    /// Parse: physical_path Name { involves: ["Link1", "Link2"] }.
    fn parse_physical_path(&mut self) -> Result<PhysicalPath, String> {
        self.expect(Token::PhysicalPathKw)?;
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let id = attributes
            .get("id")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());
        let involves = Self::string_list(&attributes, "involves");
        if involves.is_empty() {
            return Err(self.err(format!(
                "physical_path '{}' must involve at least one physical link (involves: [...])",
                name
            )));
        }
        Ok(PhysicalPath { id, name, involves, attributes })
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
        let (attributes, ports) = self.parse_ported_block()?;

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

        Ok(ExternalActor { id, name, color, ports, attributes })
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

        let id = attributes.get("id").and_then(|v| v.as_string()).map(|s| s.to_string()).or_else(|| label.clone()).unwrap_or_else(|| format!("{from} -> {to}"));
        Ok(FunctionalExchange { id, from_port: from, to_port: to, data_type, label, attributes })
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
            id: String::new(),
            attributes: HashMap::new(),
            name,
            requirements,
            functions: Vec::new(),
            components: Vec::new(),
            external_actors: Vec::new(),
            functional_exchanges: Vec::new(),
            component_exchanges: Vec::new(),
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
            id: attributes
                .get("id")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .unwrap_or_else(|| default_element_id("SF", &name)),
            name,
            category: FunctionCategory::System,
            color: Some("#70AD47".to_string()),
            icon: None,
            ports,
            sub_functions,
            attributes,
        })
    }
    
    /// A function inside a function: the same body (attributes, ports, functions).
    fn parse_nested_function(&mut self) -> Result<SystemFunction, String> {
        self.parse_system_function()
    }
    
    fn parse_system_component(&mut self) -> Result<SystemComponent, String> {
        self.advance(); // Skip 'system_component'
        let name = self.expect_name()?;
        let (attributes, ports) = self.parse_ported_block()?;
        Ok(SystemComponent { name, ports, attributes })
    }

    /// An attribute block that also declares component ports:
    /// `{ attributes, port in|out|inout Name { ... } }`.
    fn parse_ported_block(&mut self) -> Result<(HashMap<String, AttributeValue>, Vec<ComponentPort>), String> {
        self.expect(Token::LeftBrace)?;
        let mut attributes = HashMap::new();
        let mut ports = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Port) && !self.peek_is_colon() {
                ports.push(self.parse_component_port()?);
            } else {
                let (key, value) = self.parse_attribute()?;
                attributes.insert(key, value);
            }
        }
        self.expect(Token::RightBrace)?;
        Ok((attributes, ports))
    }
    
    fn parse_logical_architecture(&mut self) -> Result<LogicalArchitecture, String> {
        self.expect(Token::LogicalArchitecture)?;
        let name = self.expect_name()?;
        self.parse_logical_architecture_block(name)
    }
    
    fn parse_logical_architecture_block(&mut self, name: String) -> Result<LogicalArchitecture, String> {
        let mut level_attributes: HashMap<String, AttributeValue> = HashMap::new();
        self.expect(Token::LeftBrace)?;

        let mut name = name;
        let mut components = Vec::new();
        let mut functions = Vec::new();
        let mut interfaces = Vec::new();
        let mut component_exchanges = Vec::new();
        let mut functional_exchanges = Vec::new();
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
                Token::Identifier(ref id) if id == "functional_exchange" || id == "exchange" => {
                    functional_exchanges.push(self.parse_functional_exchange()?);
                }
                Token::Flow => {
                    functional_exchanges.push(self.parse_functional_exchange()?);
                }
                Token::Function => {
                    functions.push(self.parse_logical_function()?);
                }
                Token::CapabilityRealization | Token::Capability => {
                    capability_realizations.extend(self.parse_capabilities()?);
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
                        level_attributes.insert(key, value);
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in logical_architecture: {}", self.current())));
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(LogicalArchitecture {
            id: level_attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default(),
            attributes: level_attributes,
            name,
            components,
            functions,
            interfaces,
            component_exchanges,
            functional_exchanges,
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
            id: attributes.get("id").and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_else(|| name.clone()),
            from_port: from,
            to_port: to,
            exchange_item,
            label,
            attributes,
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

        let id = attributes.get("id").and_then(|v| v.as_string()).map(|s| s.to_string()).or_else(|| label.clone()).unwrap_or_else(|| format!("{from_port} -> {to_port}"));
        Ok(ComponentExchange { id, from_port, to_port, exchange_item, label, attributes })
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
            .unwrap_or_else(|| default_element_id("LC", &name));
        
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
    
    /// A function of the logical or physical level:
    /// `function Name { attributes, port in|out Name, function Nested { ... } }`.
    /// The body block is optional: `function "Name"` declares a bare function.
    fn parse_logical_function(&mut self) -> Result<LogicalFunction, String> {
        self.expect(Token::Function)?;
        let name = self.expect_name()?;
        let mut attributes = HashMap::new();
        let mut ports = Vec::new();
        let mut sub_functions = Vec::new();
        if self.check(&Token::LeftBrace) {
            self.expect(Token::LeftBrace)?;
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                match self.current() {
                    Token::Port if !self.peek_is_colon() => ports.push(self.parse_function_port()?),
                    Token::Function if !self.peek_is_colon() => sub_functions.push(self.parse_logical_function()?),
                    _ => {
                        let (key, value) = self.parse_attribute()?;
                        attributes.insert(key, value);
                    }
                }
            }
            self.expect(Token::RightBrace)?;
        }
        let id = attributes.get("id").and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_else(|| name.clone());
        Ok(LogicalFunction { id, name, ports, sub_functions, attributes })
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
        let mut level_attributes: HashMap<String, AttributeValue> = HashMap::new();
        self.expect(Token::LeftBrace)?;

        let mut name = name;
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        let mut physical_exchanges = Vec::new();
        let mut functions = Vec::new();
        let mut functional_exchanges = Vec::new();
        let mut functional_chains = Vec::new();
        let mut capability_realizations = Vec::new();
        let mut paths = Vec::new();

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
                Token::Identifier(ref id) if id == "functional_exchange" || id == "exchange" => {
                    functional_exchanges.push(self.parse_functional_exchange()?);
                }
                Token::Flow => {
                    functional_exchanges.push(self.parse_functional_exchange()?);
                }
                Token::Function => {
                    functions.push(self.parse_logical_function()?);
                }
                Token::FunctionalChain => {
                    functional_chains.push(self.parse_functional_chain()?);
                }
                Token::CapabilityRealization | Token::Capability => {
                    capability_realizations.extend(self.parse_capabilities()?);
                }
                Token::Identifier(ref id) if id == "physical_exchange" => {
                    physical_exchanges.push(self.parse_physical_exchange()?);
                }
                Token::PhysicalPathKw => {
                    paths.push(self.parse_physical_path()?);
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
                        level_attributes.insert(key, value);
                    }
                }
                _ => {
                    return Err(self.err(format!("Unexpected token in physical_architecture: {}", self.current())));
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(PhysicalArchitecture {
            id: level_attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default(),
            attributes: level_attributes,
            name,
            nodes,
            functions,
            links,
            physical_exchanges,
            functional_exchanges,
            functional_chains,
            capability_realizations,
            paths,
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
            id: attributes.get("id").and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_else(|| name.clone()),
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
            attributes,
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
        let mut component_ports = Vec::new();
        let mut node_functions = Vec::new();
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
                Token::Port if self.peek_is_port_direction() => {
                    component_ports.push(self.parse_component_port()?);
                }
                Token::Port => {
                    node_ports.push(self.parse_physical_port()?);
                }
                Token::Function if !self.peek_is_colon() => {
                    node_functions.push(self.parse_logical_function()?);
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
            functions: node_functions,
            hardware_components,
            deployments,
            component_ports,
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
                name: name.clone(),
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
            name,
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
            id: default_element_id("PN", &name),
            name,
            node_type: NodeType::Hardware,
            color: Some("#FFE699".to_string()),
            processor: None,
            memory: None,
            behavior_components: Vec::new(),
            functions: Vec::new(),
            hardware_components: Vec::new(),
            deployments: Vec::new(),
            ports: Vec::new(),
            component_ports: Vec::new(),
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
            name,
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
        let mut epbs_attributes: HashMap<String, AttributeValue> = HashMap::new();

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
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    epbs_attributes.insert(key, value);
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

        let id = epbs_attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_default();
        Ok(Epbs { id, name, attributes: epbs_attributes, systems })
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
    
    /// `security_analysis { level: "sa" id: "..." threat … asset … }`.
    fn parse_security_analysis(&mut self) -> Result<SecurityAnalysis, String> {
        self.expect(Token::SecurityAnalysis)?;
        self.expect(Token::LeftBrace)?;
        let mut threats = Vec::new();
        let mut assets = Vec::new();
        let mut attributes = HashMap::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // `threat` and `asset` are keywords only here: elsewhere they are
            // ordinary names (`port out threat`).
            match self.current() {
                Token::Identifier(word) if word == "threat" && !self.peek_is_colon() => threats.push(self.parse_threat()?),
                Token::Identifier(word) if word == "asset" && !self.peek_is_colon() => assets.push(self.parse_asset()?),
                _ if self.peek_is_colon() => {
                    let (key, value) = self.parse_attribute()?;
                    attributes.insert(key, value);
                }
                _ => return Err(self.err(format!("Unexpected token in security_analysis: {}", self.current()))),
            }
        }
        self.expect(Token::RightBrace)?;
        if let Some(level) = attributes.get("level").and_then(|v| v.as_string()) {
            if !matches!(level, "oa" | "sa" | "la" | "pa") {
                return Err(self.err(format!("security_analysis: level '{level}' is not one of oa, sa, la, pa")));
            }
        }
        if threats.is_empty() && assets.is_empty() {
            self.warnings.push("security_analysis declares no threat and no asset".to_string());
        }
        let id = attributes.remove("id").and_then(|v| v.as_string().map(str::to_string)).unwrap_or_default();
        Ok(SecurityAnalysis { id, threats, assets, attributes })
    }

    /// `threat "Name" { id: kind: level: applies_to: [...] involves: [...] … }`.
    fn parse_threat(&mut self) -> Result<Threat, String> {
        self.advance(); // `threat`
        let name = self.expect_name()?;
        let mut attributes = self.parse_attributes_block()?;
        let id = attributes.remove("id").and_then(|v| v.as_string().map(str::to_string)).unwrap_or_else(|| name.clone());
        let kind = attributes.remove("kind").map(|v| v.display());
        let applies_to = Self::take_names(&mut attributes, "applies_to", &name, "threat")?;
        let involves = Self::take_names(&mut attributes, "involves", &name, "threat")?;
        if let Some(level) = attributes.get("level") {
            if level.as_integer().is_none() {
                return Err(self.err(format!("threat '{name}': level must be an integer, got {}", level.display())));
            }
        }
        Ok(Threat { id, name, kind, applies_to, involves, attributes })
    }

    /// `asset "Name" { id: kind: "functional" members: [...] … }`.
    fn parse_asset(&mut self) -> Result<PrimaryAsset, String> {
        self.advance(); // `asset`
        let name = self.expect_name()?;
        let mut attributes = self.parse_attributes_block()?;
        let id = attributes.remove("id").and_then(|v| v.as_string().map(str::to_string)).unwrap_or_else(|| name.clone());
        let kind = match attributes.remove("kind") {
            None => AssetKind::Functional,
            Some(value) => AssetKind::parse(&value.display()).ok_or_else(|| {
                self.err(format!("asset '{name}': kind '{}' is not one of functional, information, enterprise", value.display()))
            })?,
        };
        let members = Self::take_names(&mut attributes, "members", &name, "asset")?;
        Ok(PrimaryAsset { id, name, kind, members, attributes })
    }

    /// `security_scales { confidentiality: ["0", "1"] … threat_kinds: [...] }`.
    fn parse_security_scales(&mut self) -> Result<SecurityScales, String> {
        self.expect(Token::SecurityScales)?;
        let mut attributes = self.parse_attributes_block()?;
        let id = attributes.remove("id").and_then(|v| v.as_string().map(str::to_string)).unwrap_or_default();
        for key in attributes.keys() {
            if !SecurityScales::NEEDS.contains(&key.as_str()) && key != "threat_kinds" {
                self.warnings.push(format!("security_scales: '{key}' is not a security need (confidentiality, integrity, availability, traceability) nor threat_kinds"));
            }
        }
        Ok(SecurityScales { id, attributes })
    }

    /// A list attribute of names, removed from the block; a single name is
    /// accepted as a list of one.
    fn take_names(attributes: &mut HashMap<String, AttributeValue>, key: &str, owner: &str, kind: &str) -> Result<Vec<String>, String> {
        match attributes.remove(key) {
            None => Ok(Vec::new()),
            Some(AttributeValue::List(items)) => items
                .iter()
                .map(|item| item.as_string().map(str::to_string).ok_or_else(|| format!("{kind} '{owner}': {key} must list names, got {}", item.display())))
                .collect(),
            Some(AttributeValue::String(single)) => Ok(vec![single]),
            Some(other) => Err(format!("{kind} '{owner}': {key} must be a list of names, got {}", other.display())),
        }
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
    
    /// `import "relative/path.arc"` — path resolution happens in the
    /// compiler (the parser has no filesystem access).
    fn parse_import_decl(&mut self) -> Result<String, String> {
        self.expect(Token::ImportKw)?;
        match self.current().clone() {
            Token::StringLiteral(path) => {
                self.advance();
                Ok(path)
            }
            other => Err(self.err(format!(
                "import expects a quoted file path, got {}",
                other
            ))),
        }
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
            Token::Number(_) | Token::Integer(_) => Ok(match self.parse_number_or_quantity()? {
                (_, Some(i), None) => AttributeValue::Integer(i),
                (n, None, None) => AttributeValue::Number(n),
                (_, _, Some(quantity)) => AttributeValue::Quantity(quantity),
            }),
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
    
    /// The lexer reads `-5` and `-5.0` as one negative literal.
    fn current_is_negative_number(&self) -> bool {
        match self.current() {
            Token::Number(n) => *n < 0.0,
            Token::Integer(i) => *i < 0,
            _ => false,
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
        let mut functions = Vec::new();
        let mut ports = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Function) && !self.peek_is_colon() {
                functions.push(self.parse_logical_function()?);
            } else if self.check(&Token::Port) && !self.peek_is_colon() {
                ports.push(self.parse_component_port()?);
            } else if self.peek_is_colon() {
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
            .unwrap_or_else(|| default_element_id("BC", &name));
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
        if let Some(single) = attributes.get("allocated_function").and_then(|v| v.as_string()) {
            allocated_functions.push(single.to_string());
        }
        let allocated_component = attributes.get("allocated_component").and_then(|v| v.as_string()).map(str::to_string);

        Ok(BehaviorComponent {
            id,
            name,
            allocated_component,
            allocated_functions,
            color,
            functions,
            ports,
            attributes,
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
            id: default_element_id("HC", &name),
            name,
            hw_type,
            specs,
            color,
        })
    }
}

/// Default identity of an element declared without an explicit `id`:
/// `<PREFIX>-<Name>`, spaces written as underscores.
///
/// The whole name is used. Earlier pre-releases kept only its first three
/// characters, so `BrakingModule` and `BrakeActuation` silently shared the
/// identity `BC-Bra` (and therefore one UUID).
pub(crate) fn default_element_id(prefix: &str, name: &str) -> String {
    format!("{prefix}-{}", name.replace(' ', "_"))
}
