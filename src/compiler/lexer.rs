use std::fmt;

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
    Signals,
    Connect,
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

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }
    
    pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            if self.is_at_end() {
                tokens.push(Token::Eof);
                break;
            }
            
            if self.current_char() == '/' && self.peek_char() == Some('/') {
                self.skip_line_comment();
                continue;
            }
            
            if self.current_char() == '/' && self.peek_char() == Some('*') {
                self.skip_block_comment()?;
                continue;
            }
            
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        Ok(tokens)
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
            "signals" => Token::Signals,
            "connect" => Token::Connect,
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
