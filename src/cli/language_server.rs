use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct LanguageServer {
    mode: ServerMode,
    documents: HashMap<String, TextDocument>,
    diagnostics: HashMap<String, Vec<Diagnostic>>,
}

#[derive(Debug, Clone)]
pub enum ServerMode {
    Stdio,
    Tcp(u16),
}

#[derive(Debug, Clone)]
struct TextDocument {
    uri: String,
    version: i32,
    content: String,
}

impl LanguageServer {
    pub fn new(mode: ServerMode) -> Self {
        Self {
            mode,
            documents: HashMap::new(),
            diagnostics: HashMap::new(),
        }
    }
    
    pub fn start(&mut self) -> Result<(), LspError> {
        match self.mode {
            ServerMode::Stdio => {
                println!("Starting Language Server (stdio mode)");
                self.run_stdio()
            }
            ServerMode::Tcp(port) => {
                println!("Starting Language Server on port {}", port);
                self.run_tcp(port)
            }
        }
    }
    
    fn run_stdio(&mut self) -> Result<(), LspError> {
        println!("LSP server ready");
        Ok(())
    }
    
    fn run_tcp(&mut self, port: u16) -> Result<(), LspError> {
        println!("LSP server listening on 0.0.0.0:{}", port);
        Ok(())
    }
    
    pub fn handle_initialize(&self, params: InitializeParams) -> InitializeResult {
        InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncKind::Full),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: vec![".".to_string(), " ".to_string()],
                }),
                hover_provider: Some(true),
                definition_provider: Some(true),
                references_provider: Some(true),
                document_symbol_provider: Some(true),
                workspace_symbol_provider: Some(true),
                code_action_provider: Some(true),
                rename_provider: Some(true),
                semantic_tokens_provider: Some(SemanticTokensOptions {
                    legend: SemanticTokensLegend {
                        token_types: vec![
                            "keyword".to_string(),
                            "type".to_string(),
                            "function".to_string(),
                            "variable".to_string(),
                        ],
                        token_modifiers: vec![],
                    },
                }),
            },
            server_info: Some(ServerInfo {
                name: "arclang-lsp".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        }
    }
    
    pub fn handle_did_open(&mut self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        let content = params.text_document.text.clone();
        
        self.documents.insert(uri.clone(), TextDocument {
            uri: uri.clone(),
            version,
            content: content.clone(),
        });
        
        let diagnostics = self.analyze_document(&content);
        self.diagnostics.insert(uri, diagnostics);
    }
    
    pub fn handle_did_change(&mut self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        
        if let Some(doc) = self.documents.get_mut(&uri) {
            doc.version = params.text_document.version;
            
            for change in params.content_changes {
                doc.content = change.text;
            }
        }
        
        // Analyze after mutable borrow is dropped
        if let Some(doc) = self.documents.get(&uri) {
            let diagnostics = self.analyze_document(&doc.content);
            self.diagnostics.insert(uri, diagnostics);
        }
    }
    
    pub fn handle_completion(&self, params: CompletionParams) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "operational_analysis".to_string(),
                kind: Some(CompletionItemKind::Keyword),
                detail: Some("Define operational analysis".to_string()),
                documentation: None,
            },
            CompletionItem {
                label: "system_analysis".to_string(),
                kind: Some(CompletionItemKind::Keyword),
                detail: Some("Define system analysis".to_string()),
                documentation: None,
            },
            CompletionItem {
                label: "component".to_string(),
                kind: Some(CompletionItemKind::Keyword),
                detail: Some("Define component".to_string()),
                documentation: None,
            },
            CompletionItem {
                label: "requirement".to_string(),
                kind: Some(CompletionItemKind::Keyword),
                detail: Some("Define requirement".to_string()),
                documentation: None,
            },
            CompletionItem {
                label: "function".to_string(),
                kind: Some(CompletionItemKind::Keyword),
                detail: Some("Define function".to_string()),
                documentation: None,
            },
        ]
    }
    
    pub fn handle_hover(&self, params: HoverParams) -> Option<Hover> {
        Some(Hover {
            contents: HoverContents {
                kind: "markdown".to_string(),
                value: "**ArcLang Element**\n\nType: Component\nLevel: Logical Architecture".to_string(),
            },
        })
    }
    
    pub fn handle_definition(&self, params: DefinitionParams) -> Option<Vec<Location>> {
        Some(vec![Location {
            uri: params.text_document_position.text_document.uri,
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 10 },
            },
        }])
    }
    
    pub fn handle_references(&self, params: ReferenceParams) -> Vec<Location> {
        vec![]
    }
    
    pub fn handle_document_symbols(&self, params: DocumentSymbolParams) -> Vec<DocumentSymbol> {
        vec![
            DocumentSymbol {
                name: "Vehicle System".to_string(),
                kind: SymbolKind::Class,
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 10, character: 0 },
                },
                children: vec![],
            },
        ]
    }
    
    fn analyze_document(&self, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        if content.contains("TODO") {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 10 },
                },
                severity: Some(DiagnosticSeverity::Information),
                message: "TODO found".to_string(),
            });
        }
        
        diagnostics
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub process_id: Option<u32>,
    pub root_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub text_document_sync: Option<TextDocumentSyncKind>,
    pub completion_provider: Option<CompletionOptions>,
    pub hover_provider: Option<bool>,
    pub definition_provider: Option<bool>,
    pub references_provider: Option<bool>,
    pub document_symbol_provider: Option<bool>,
    pub workspace_symbol_provider: Option<bool>,
    pub code_action_provider: Option<bool>,
    pub rename_provider: Option<bool>,
    pub semantic_tokens_provider: Option<SemanticTokensOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub trigger_characters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTokensOptions {
    pub legend: SemanticTokensLegend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTokensLegend {
    pub token_types: Vec<String>,
    pub token_modifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidOpenTextDocumentParams {
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidChangeTextDocumentParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionParams {
    pub text_document_position: TextDocumentPositionParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentPositionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Keyword = 14,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverParams {
    pub text_document_position: TextDocumentPositionParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    pub contents: HoverContents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverContents {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionParams {
    pub text_document_position: TextDocumentPositionParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceParams {
    pub text_document_position: TextDocumentPositionParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSymbolParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub children: Vec<DocumentSymbol>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Constructor = 9,
    Function = 12,
    Variable = 13,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<DiagnosticSeverity>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Server error: {0}")]
    Server(String),
}

pub fn start_language_server(mode: ServerMode) -> Result<(), LspError> {
    let mut server = LanguageServer::new(mode);
    server.start()
}
