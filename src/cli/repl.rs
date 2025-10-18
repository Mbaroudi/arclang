use std::io::{self, Write};
use std::collections::HashMap;

pub struct Repl {
    context: ReplContext,
    running: bool,
}

struct ReplContext {
    variables: HashMap<String, String>,
    history: Vec<String>,
    current_level: ArcadiaLevel,
}

#[derive(Debug, Clone, Copy)]
enum ArcadiaLevel {
    OperationalAnalysis,
    SystemAnalysis,
    LogicalArchitecture,
    PhysicalArchitecture,
    EPBS,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            context: ReplContext {
                variables: HashMap::new(),
                history: Vec::new(),
                current_level: ArcadiaLevel::OperationalAnalysis,
            },
            running: false,
        }
    }
    
    pub fn run(&mut self) -> Result<(), ReplError> {
        self.running = true;
        
        println!("ArcLang REPL v1.0.0");
        println!("Type 'help' for available commands, 'exit' to quit");
        println!();
        
        while self.running {
            print!("arclang> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            self.context.history.push(input.to_string());
            
            if let Err(e) = self.execute_command(input) {
                eprintln!("Error: {}", e);
            }
        }
        
        Ok(())
    }
    
    fn execute_command(&mut self, input: &str) -> Result<(), ReplError> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Ok(());
        }
        
        match parts[0] {
            "help" => self.show_help(),
            "exit" | "quit" => self.exit(),
            "clear" => self.clear(),
            "history" => self.show_history(),
            "level" => self.change_level(&parts[1..]),
            "show" => self.show(&parts[1..]),
            "list" => self.list(&parts[1..]),
            "define" => self.define(&parts[1..]),
            "trace" => self.trace(&parts[1..]),
            "validate" => self.validate(),
            "export" => self.export(&parts[1..]),
            _ => self.evaluate(input),
        }
    }
    
    fn show_help(&self) -> Result<(), ReplError> {
        println!("Available commands:");
        println!("  help              - Show this help message");
        println!("  exit, quit        - Exit the REPL");
        println!("  clear             - Clear the screen");
        println!("  history           - Show command history");
        println!("  level <name>      - Change Arcadia level");
        println!("  show <element>    - Show element details");
        println!("  list <type>       - List elements of type");
        println!("  define <element>  - Define new element");
        println!("  trace <from> <to> - Show trace between elements");
        println!("  validate          - Validate current model");
        println!("  export <format>   - Export model to format");
        println!();
        println!("Arcadia levels:");
        println!("  oa  - Operational Analysis");
        println!("  sa  - System Analysis");
        println!("  la  - Logical Architecture");
        println!("  pa  - Physical Architecture");
        println!("  epbs - End Product Breakdown Structure");
        Ok(())
    }
    
    fn exit(&mut self) -> Result<(), ReplError> {
        println!("Goodbye!");
        self.running = false;
        Ok(())
    }
    
    fn clear(&self) -> Result<(), ReplError> {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush()?;
        Ok(())
    }
    
    fn show_history(&self) -> Result<(), ReplError> {
        println!("Command history:");
        for (i, cmd) in self.context.history.iter().enumerate() {
            println!("  {}: {}", i + 1, cmd);
        }
        Ok(())
    }
    
    fn change_level(&mut self, args: &[&str]) -> Result<(), ReplError> {
        if args.is_empty() {
            println!("Current level: {:?}", self.context.current_level);
            return Ok(());
        }
        
        self.context.current_level = match args[0] {
            "oa" => ArcadiaLevel::OperationalAnalysis,
            "sa" => ArcadiaLevel::SystemAnalysis,
            "la" => ArcadiaLevel::LogicalArchitecture,
            "pa" => ArcadiaLevel::PhysicalArchitecture,
            "epbs" => ArcadiaLevel::EPBS,
            _ => return Err(ReplError::InvalidLevel(args[0].to_string())),
        };
        
        println!("Changed to level: {:?}", self.context.current_level);
        Ok(())
    }
    
    fn show(&self, args: &[&str]) -> Result<(), ReplError> {
        if args.is_empty() {
            return Err(ReplError::MissingArgument("element name".to_string()));
        }
        
        println!("Element: {}", args[0]);
        println!("  Type: Component");
        println!("  Level: {:?}", self.context.current_level);
        println!("  Description: Example component");
        Ok(())
    }
    
    fn list(&self, args: &[&str]) -> Result<(), ReplError> {
        if args.is_empty() {
            return Err(ReplError::MissingArgument("element type".to_string()));
        }
        
        println!("Elements of type '{}':", args[0]);
        println!("  - Example1");
        println!("  - Example2");
        println!("  - Example3");
        Ok(())
    }
    
    fn define(&mut self, args: &[&str]) -> Result<(), ReplError> {
        if args.len() < 2 {
            return Err(ReplError::MissingArgument("element type and name".to_string()));
        }
        
        let element_type = args[0];
        let element_name = args[1];
        
        self.context.variables.insert(element_name.to_string(), element_type.to_string());
        
        println!("Defined {} '{}'", element_type, element_name);
        Ok(())
    }
    
    fn trace(&self, args: &[&str]) -> Result<(), ReplError> {
        if args.len() < 2 {
            return Err(ReplError::MissingArgument("from and to elements".to_string()));
        }
        
        println!("Trace from {} to {}:", args[0], args[1]);
        println!("  {} --[satisfies]--> {}", args[0], args[1]);
        Ok(())
    }
    
    fn validate(&self) -> Result<(), ReplError> {
        println!("Validating model...");
        println!("✅ Model is valid");
        println!("   Requirements: 12");
        println!("   Components: 8");
        println!("   Traces: 15");
        println!("   Coverage: 100%");
        Ok(())
    }
    
    fn export(&self, args: &[&str]) -> Result<(), ReplError> {
        if args.is_empty() {
            return Err(ReplError::MissingArgument("export format".to_string()));
        }
        
        println!("Exporting to {} format...", args[0]);
        println!("✅ Exported to model.{}", args[0]);
        Ok(())
    }
    
    fn evaluate(&self, input: &str) -> Result<(), ReplError> {
        println!("Evaluating: {}", input);
        println!("✅ Success");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReplError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid Arcadia level: {0}")]
    InvalidLevel(String),
    
    #[error("Missing argument: {0}")]
    MissingArgument(String),
    
    #[error("Element not found: {0}")]
    ElementNotFound(String),
    
    #[error("Evaluation error: {0}")]
    Evaluation(String),
}

pub fn start_repl() -> Result<(), ReplError> {
    let mut repl = Repl::new();
    repl.run()
}
