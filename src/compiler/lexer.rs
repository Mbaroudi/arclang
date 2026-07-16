use std::fmt;

/// Source position of a token (1-based line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    OperationalAnalysis,
    SystemAnalysis,
    LogicalArchitecture,
    PhysicalArchitecture,
    Epbs,
    Actor,
    Requirement,
    Component,
    Function,
    Interface,
    Node,
    System,
    Subsystem,
    Item,
    SafetyAnalysis,
    Hazard,
    Fmea,
    Trace,
    Deploys,
    Implements,
    Satisfies,
    Validates,
    // New MBSE keywords
    Model,
    Metadata,
    Version,
    Author,
    Description,
    Requirements,
    Stakeholder,
    Architecture,
    Logical,
    Physical,
    Provides,
    Requires,
    BehaviorComponent,
    HardwareComponent,
    Signals,
    Connect,
    Connection,
    Via,
    Scenarios,
    Scenario,
    Steps,
    Precondition,
    Postcondition,
    Properties,
    Parent,
    SafetyLevel,
    Priority,
    Traces,
    Verification,
    Rationale,
    // Additional syntax variants
    Port,
    Flow,
    Inputs,
    Outputs,
    ExecutionTime,
    Type,
    DataType,
    Rate,
    Unit,
    From,
    To,
    Protocol,
    Latency,
    Property,
    Value,
    ValidationKeyword,
    TestCase,
    Measure,
    DataFlows,
    SafetyMeasures,
    Req,
    InterfaceIn,
    InterfaceOut,
    In,
    Out,
    Dataflow,
    Step,
    Action,
    Participants,
    
    // Literals
    Identifier(String),
    StringLiteral(String),
    Number(f64),
    
    // Symbols
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Dot,
    Arrow,
    Minus,
    
    // End of file
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::StringLiteral(s) => write!(f, "String(\"{}\")", s),
            Token::Number(n) => write!(f, "Number({})", n),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Token {
    /// The source spelling of a keyword token (inverse of the lexer's keyword
    /// table). Lets keyword tokens double as attribute keys, e.g. `via: "CANBus"`.
    pub fn keyword_text(&self) -> Option<&'static str> {
        let text = match self {
            Token::OperationalAnalysis => "operational_analysis",
            Token::SystemAnalysis => "system_analysis",
            Token::LogicalArchitecture => "logical_architecture",
            Token::PhysicalArchitecture => "physical_architecture",
            Token::Epbs => "epbs",
            Token::Actor => "actor",
            Token::Requirement => "requirement",
            Token::Component => "component",
            Token::Function => "function",
            Token::Interface => "interface",
            Token::Node => "node",
            Token::System => "system",
            Token::Subsystem => "subsystem",
            Token::Item => "item",
            Token::SafetyAnalysis => "safety_analysis",
            Token::Hazard => "hazard",
            Token::Fmea => "fmea",
            Token::Trace => "trace",
            Token::Deploys => "deploys",
            Token::Implements => "implements",
            Token::Satisfies => "satisfies",
            Token::Validates => "validates",
            Token::Model => "model",
            Token::Metadata => "metadata",
            Token::Version => "version",
            Token::Author => "author",
            Token::Description => "description",
            Token::Requirements => "requirements",
            Token::Stakeholder => "stakeholder",
            Token::Architecture => "architecture",
            Token::Logical => "logical",
            Token::Physical => "physical",
            Token::Provides => "provides",
            Token::Requires => "requires",
            Token::BehaviorComponent => "behavior_component",
            Token::HardwareComponent => "hardware_component",
            Token::Signals => "signals",
            Token::Connect => "connect",
            Token::Connection => "connection",
            Token::Via => "via",
            Token::Scenarios => "scenarios",
            Token::Scenario => "scenario",
            Token::Steps => "steps",
            Token::Precondition => "precondition",
            Token::Postcondition => "postcondition",
            Token::Properties => "properties",
            Token::Parent => "parent",
            Token::SafetyLevel => "safety_level",
            Token::Priority => "priority",
            Token::Traces => "traces",
            Token::Verification => "verification",
            Token::Rationale => "rationale",
            Token::Port => "port",
            Token::Flow => "flow",
            Token::Inputs => "inputs",
            Token::Outputs => "outputs",
            Token::ExecutionTime => "execution_time",
            Token::Type => "type",
            Token::DataType => "data_type",
            Token::Rate => "rate",
            Token::Unit => "unit",
            Token::From => "from",
            Token::To => "to",
            Token::Protocol => "protocol",
            Token::Latency => "latency",
            Token::Property => "property",
            Token::Value => "value",
            Token::ValidationKeyword => "validation",
            Token::TestCase => "test_case",
            Token::Measure => "measure",
            Token::DataFlows => "data_flows",
            Token::SafetyMeasures => "safety_measures",
            Token::Req => "req",
            Token::InterfaceIn => "interface_in",
            Token::InterfaceOut => "interface_out",
            Token::In => "in",
            Token::Out => "out",
            Token::Dataflow => "dataflow",
            Token::Step => "step",
            Token::Action => "action",
            Token::Participants => "participants",
            _ => return None,
        };
        Some(text)
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(self) -> Result<Vec<Token>, String> {
        self.tokenize_spanned().map(|(tokens, _)| tokens)
    }

    /// Tokenize, returning each token together with its source position.
    /// The two vectors are always the same length.
    pub fn tokenize_spanned(mut self) -> Result<(Vec<Token>, Vec<Span>), String> {
        let mut tokens = Vec::new();
        let mut spans = Vec::new();

        loop {
            self.skip_whitespace();

            if self.is_at_end() {
                tokens.push(Token::Eof);
                spans.push(self.span());
                break;
            }

            if self.current_char() == '/' && self.peek_char() == Some('/') {
                self.skip_line_comment();
                continue;
            }

            if self.current_char() == '/' && self.peek_char() == Some('*') {
                let span = self.span();
                self.skip_block_comment().map_err(|e| format!("{} at {}", e, span))?;
                continue;
            }

            let span = self.span();
            let token = self.next_token().map_err(|e| format!("{} at {}", e, span))?;
            tokens.push(token);
            spans.push(span);
        }

        Ok((tokens, spans))
    }

    fn span(&self) -> Span {
        Span { line: self.line, column: self.column }
    }
    
    fn next_token(&mut self) -> Result<Token, String> {
        let ch = self.current_char();
        
        match ch {
            '{' => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            '}' => {
                self.advance();
                Ok(Token::RightBrace)
            }
            '[' => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::RightBracket)
            }
            ':' => {
                self.advance();
                Ok(Token::Colon)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            '.' => {
                self.advance();
                Ok(Token::Dot)
            }
            '-' => {
                if self.peek_char() == Some('>') {
                    self.advance();
                    self.advance();
                    Ok(Token::Arrow)
                } else if self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
                    self.read_number()
                } else {
                    self.advance();
                    Ok(Token::Minus)
                }
            }
            '"' => self.read_string_literal(),
            _ if ch.is_ascii_digit() => self.read_number(),
            _ if ch.is_alphabetic() || ch == '_' => self.read_identifier_or_keyword(),
            _ => Err(format!("Unexpected character: '{}'", ch)),
        }
    }
    
    fn read_string_literal(&mut self) -> Result<Token, String> {
        self.advance(); // Skip opening quote
        let mut string = String::new();
        
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    match self.current_char() {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'r' => string.push('\r'),
                        '"' => string.push('"'),
                        '\\' => string.push('\\'),
                        _ => string.push(self.current_char()),
                    }
                    self.advance();
                }
            } else {
                string.push(self.current_char());
                self.advance();
            }
        }
        
        if self.is_at_end() {
            return Err("Unterminated string literal".to_string());
        }
        
        self.advance(); // Skip closing quote
        Ok(Token::StringLiteral(string))
    }
    
    fn read_number(&mut self) -> Result<Token, String> {
        let mut number_str = String::new();
        let mut has_decimal = false;
        
        if self.current_char() == '-' {
            number_str.push('-');
            self.advance();
        }
        
        while !self.is_at_end() {
            let ch = self.current_char();
            
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else if ch == '.' && !has_decimal && self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
                has_decimal = true;
                number_str.push(ch);
                self.advance();
            } else if ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        number_str.parse::<f64>()
            .map(Token::Number)
            .map_err(|_| format!("Invalid number: {}", number_str))
    }
    
    fn read_identifier_or_keyword(&mut self) -> Result<Token, String> {
        let mut ident = String::new();
        
        while !self.is_at_end() && (self.current_char().is_alphanumeric() || self.current_char() == '_') {
            ident.push(self.current_char());
            self.advance();
        }
        
        let token = match ident.as_str() {
            "operational_analysis" => Token::OperationalAnalysis,
            "system_analysis" => Token::SystemAnalysis,
            "logical_architecture" => Token::LogicalArchitecture,
            "physical_architecture" => Token::PhysicalArchitecture,
            "epbs" => Token::Epbs,
            "actor" => Token::Actor,
            "requirement" => Token::Requirement,
            "component" => Token::Component,
            "function" => Token::Function,
            "interface" => Token::Interface,
            "node" => Token::Node,
            "system" => Token::System,
            "subsystem" => Token::Subsystem,
            "item" => Token::Item,
            "safety_analysis" => Token::SafetyAnalysis,
            "hazard" => Token::Hazard,
            "fmea" => Token::Fmea,
            "trace" => Token::Trace,
            "deploys" => Token::Deploys,
            "implements" => Token::Implements,
            "satisfies" => Token::Satisfies,
            "validates" => Token::Validates,
            // New MBSE keywords
            "model" => Token::Model,
            "metadata" => Token::Metadata,
            "version" => Token::Version,
            "author" => Token::Author,
            "description" => Token::Description,
            "requirements" => Token::Requirements,
            "stakeholder" => Token::Stakeholder,
            "architecture" => Token::Architecture,
            "logical" => Token::Logical,
            "physical" => Token::Physical,
            "provides" => Token::Provides,
            "requires" => Token::Requires,
            "behavior_component" => Token::BehaviorComponent,
            "hardware_component" => Token::HardwareComponent,
            "signals" => Token::Signals,
            "connect" => Token::Connect,
            "connection" => Token::Connection,
            "via" => Token::Via,
            "scenarios" => Token::Scenarios,
            "scenario" => Token::Scenario,
            "steps" => Token::Steps,
            "precondition" => Token::Precondition,
            "postcondition" => Token::Postcondition,
            "properties" => Token::Properties,
            "parent" => Token::Parent,
            "safety_level" => Token::SafetyLevel,
            "priority" => Token::Priority,
            "traces" => Token::Traces,
            "verification" => Token::Verification,
            "rationale" => Token::Rationale,
            // Additional syntax variants
            "port" => Token::Port,
            "flow" => Token::Flow,
            "inputs" => Token::Inputs,
            "outputs" => Token::Outputs,
            "execution_time" => Token::ExecutionTime,
            "type" => Token::Type,
            "data_type" => Token::DataType,
            "rate" => Token::Rate,
            "unit" => Token::Unit,
            "from" => Token::From,
            "to" => Token::To,
            "protocol" => Token::Protocol,
            "latency" => Token::Latency,
            "property" => Token::Property,
            "value" => Token::Value,
            "validation" => Token::ValidationKeyword,
            "test_case" => Token::TestCase,
            "measure" => Token::Measure,
            "data_flows" => Token::DataFlows,
            "safety_measures" => Token::SafetyMeasures,
            "req" => Token::Req,
            "interface_in" => Token::InterfaceIn,
            "interface_out" => Token::InterfaceOut,
            "in" => Token::In,
            "out" => Token::Out,
            "dataflow" => Token::Dataflow,
            "step" => Token::Step,
            "participants" => Token::Participants,
            _ => Token::Identifier(ident),
        };
        
        Ok(token)
    }
    
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.current_char().is_whitespace() {
            self.advance();
        }
    }
    
    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }
    
    fn skip_block_comment(&mut self) -> Result<(), String> {
        self.advance(); // Skip '/'
        self.advance(); // Skip '*'
        
        while !self.is_at_end() {
            if self.current_char() == '*' && self.peek_char() == Some('/') {
                self.advance(); // Skip '*'
                self.advance(); // Skip '/'
                return Ok(());
            }
            self.advance();
        }
        
        Err("Unterminated block comment".to_string())
    }
    
    fn current_char(&self) -> char {
        self.input[self.position]
    }
    
    fn peek_char(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }
    
    fn advance(&mut self) {
        if self.position < self.input.len() {
            if self.input[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_simple() {
        let input = r#"operational_analysis "Test" { actor "User" {} }"#;
        let tokens = Lexer::new(input).tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::OperationalAnalysis);
        assert_eq!(tokens[1], Token::StringLiteral("Test".to_string()));
        assert_eq!(tokens[2], Token::LeftBrace);
    }
    
    #[test]
    fn test_tokenize_with_comments() {
        let input = r#"
        // Line comment
        actor "Test" { /* block comment */ }
        "#;
        let tokens = Lexer::new(input).tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::Actor);
        assert_eq!(tokens[1], Token::StringLiteral("Test".to_string()));
    }
}
