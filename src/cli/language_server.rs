//! ArcLang Language Server (LSP over stdio).
//!
//! Diagnostics come straight from the strict compiler: parser/semantic errors
//! (which carry `at line L, column C` positions) are published as LSP errors,
//! compilation warnings as LSP warnings.

use regex::Regex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::compiler::{Compiler, CompilerConfig};

pub struct ArcLangLanguageServer {
    client: Client,
}

impl ArcLangLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    async fn check(&self, uri: Url, text: &str) {
        let diagnostics = compute_diagnostics(text);
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ArcLangLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "arclang-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "arclang-lsp ready")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.check(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // FULL sync: the last change carries the whole document.
        if let Some(change) = params.content_changes.into_iter().last() {
            self.check(params.text_document.uri, &change.text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.check(params.text_document.uri, &text).await;
        }
    }
}

/// Compile the source and turn errors/warnings into LSP diagnostics.
pub fn compute_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut compiler = Compiler::new(CompilerConfig::default());
    match compiler.compile_string(source) {
        Ok(result) => result
            .warnings
            .iter()
            .map(|warning| diagnostic_from_message(warning, DiagnosticSeverity::WARNING))
            .collect(),
        Err(error) => vec![diagnostic_from_message(
            &error.to_string(),
            DiagnosticSeverity::ERROR,
        )],
    }
}

/// Extract `at line L, column C` from a compiler message; defaults to 0:0.
fn diagnostic_from_message(message: &str, severity: DiagnosticSeverity) -> Diagnostic {
    // Compiled once per call; diagnostics volume is tiny, clarity wins.
    let position_re = Regex::new(r"at line (\d+), column (\d+)").unwrap();
    let (line, column) = position_re
        .captures(message)
        .and_then(|caps| {
            let line: u32 = caps[1].parse().ok()?;
            let column: u32 = caps[2].parse().ok()?;
            // LSP positions are 0-based; compiler spans are 1-based.
            Some((line.saturating_sub(1), column.saturating_sub(1)))
        })
        .unwrap_or((0, 0));

    Diagnostic {
        range: Range {
            start: Position { line, character: column },
            end: Position { line, character: column + 1 },
        },
        severity: Some(severity),
        source: Some("arclang".to_string()),
        message: message.to_string(),
        ..Default::default()
    }
}

/// Run the language server on stdio (blocks until the client disconnects).
pub async fn run_stdio() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(ArcLangLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_diagnostic_carries_compiler_position() {
        let source = "model Test {\n  garbage here\n}\n";
        let diagnostics = compute_diagnostics(source);
        assert_eq!(diagnostics.len(), 1);
        let diag = &diagnostics[0];
        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        // Compiler says line 2, column 3 -> LSP 0-based 1:2
        assert_eq!(diag.range.start.line, 1);
        assert_eq!(diag.range.start.character, 2);
        assert!(diag.message.contains("garbage"));
    }

    #[test]
    fn valid_model_with_unmodeled_block_yields_warning() {
        let source = "model Test {\n  scenarios { anything at all }\n}\n";
        let diagnostics = compute_diagnostics(source);
        assert!(!diagnostics.is_empty());
        assert!(diagnostics
            .iter()
            .all(|d| d.severity == Some(DiagnosticSeverity::WARNING)));
    }

    #[test]
    fn clean_model_has_no_diagnostics() {
        let source = "model Test {\n}\n\narchitecture logical {\n  component \"C\" { id: \"LC-001\" }\n}\n";
        let diagnostics = compute_diagnostics(source);
        assert!(diagnostics.is_empty(), "unexpected: {diagnostics:?}");
    }
}
