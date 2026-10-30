//! ArcLang Language Server (LSP over stdio).
//!
//! What an editor gets from the compiler itself: diagnostics with the rule
//! each finding names, the outline of the file, completion of keywords,
//! attributes and enumeration values from the metamodel, hover with the
//! metamodel's own documentation, go to the declaration of an element, and
//! rename through the lossless source editor (refused when the element
//! would not keep its identity).
//!
//! Everything is computed from the text alone, by functions a test can
//! call; the server only keeps the latest text of each open document.

use std::collections::HashMap;
use std::sync::RwLock;

use regex::Regex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::compiler::metamodel::{AttrType, KindSpec, Metamodel};
use crate::compiler::rules;
use crate::compiler::source_edit::{self, Declaration};
use crate::compiler::{elements, Compiler, CompilerConfig};

pub struct ArcLangLanguageServer {
    client: Client,
    documents: RwLock<HashMap<Url, String>>,
}

impl ArcLangLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client, documents: RwLock::new(HashMap::new()) }
    }

    fn remember(&self, uri: &Url, text: &str) {
        if let Ok(mut documents) = self.documents.write() {
            documents.insert(uri.clone(), text.to_string());
        }
    }

    fn text_of(&self, uri: &Url) -> Option<String> {
        self.documents.read().ok().and_then(|documents| documents.get(uri).cloned())
    }

    async fn check(&self, uri: Url, text: &str) {
        self.remember(&uri, text);
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
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".to_string(), "\"".to_string(), "[".to_string(), ",".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "arclang-lsp ready").await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.check(params.text_document.uri, &params.text_document.text).await;
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

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Ok(mut documents) = self.documents.write() {
            documents.remove(&params.text_document.uri);
        }
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let position = params.text_document_position;
        let Some(text) = self.text_of(&position.text_document.uri) else { return Ok(None) };
        let items = complete(&text, position.position.line, position.position.character);
        Ok(if items.is_empty() { None } else { Some(CompletionResponse::Array(items)) })
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let position = params.text_document_position_params;
        let Some(text) = self.text_of(&position.text_document.uri) else { return Ok(None) };
        Ok(hover(&text, position.position.line, position.position.character).map(|value| Hover {
            contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value }),
            range: None,
        }))
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> LspResult<Option<GotoDefinitionResponse>> {
        let position = params.text_document_position_params;
        let Some(text) = self.text_of(&position.text_document.uri) else { return Ok(None) };
        Ok(definition(&text, position.position.line, position.position.character).map(|range| {
            GotoDefinitionResponse::Scalar(Location { uri: position.text_document.uri, range })
        }))
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> LspResult<Option<DocumentSymbolResponse>> {
        let Some(text) = self.text_of(&params.text_document.uri) else { return Ok(None) };
        Ok(Some(DocumentSymbolResponse::Nested(symbols(&text))))
    }

    async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        let position = params.text_document_position;
        let Some(text) = self.text_of(&position.text_document.uri) else { return Ok(None) };
        match rename(&text, position.position.line, position.position.character, &params.new_name) {
            Ok(edited) => {
                let whole = Range { start: Position::new(0, 0), end: end_of(&text) };
                let mut changes = HashMap::new();
                changes.insert(position.text_document.uri, vec![TextEdit { range: whole, new_text: edited }]);
                Ok(Some(WorkspaceEdit { changes: Some(changes), ..Default::default() }))
            }
            Err(reason) => Err(tower_lsp::jsonrpc::Error { code: tower_lsp::jsonrpc::ErrorCode::InvalidRequest, message: reason.into(), data: None }),
        }
    }
}

// --- diagnostics ------------------------------------------------------------

/// Compile the source and turn errors and findings into LSP diagnostics,
/// each finding carrying its rule as the diagnostic code.
pub fn compute_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut compiler = Compiler::new(CompilerConfig::default());
    match compiler.compile_string(source) {
        Ok(result) => {
            let outline = source_edit::outline(source).unwrap_or_default();
            rules::findings(&result)
                .into_iter()
                .map(|finding| {
                    let severity = match finding.severity {
                        rules::Severity::Error => DiagnosticSeverity::ERROR,
                        rules::Severity::Warning => DiagnosticSeverity::WARNING,
                        rules::Severity::Info => DiagnosticSeverity::INFORMATION,
                    };
                    let range = position_in(&finding.message)
                        .or_else(|| finding.element.as_deref().and_then(|element| find_declaration(&outline, element)).map(|d| line_range(d)))
                        .unwrap_or_else(|| Range::new(Position::new(0, 0), Position::new(0, 1)));
                    Diagnostic {
                        range,
                        severity: Some(severity),
                        code: Some(NumberOrString::String(finding.rule.to_string())),
                        source: Some("arclang".to_string()),
                        message: finding.message,
                        ..Default::default()
                    }
                })
                .collect()
        }
        Err(error) => diagnostics_from_error(source, &error.to_string()),
    }
}

/// A compile error as diagnostics where they are: a parse error at the line
/// and column it gives; a semantic error (one per line of the message) on
/// the line that writes the name it quotes last, else on the declaration of
/// the first name it quotes, else at the top.
fn diagnostics_from_error(source: &str, message: &str) -> Vec<Diagnostic> {
    if position_in(message).is_some() {
        return vec![diagnostic_from_message(message, DiagnosticSeverity::ERROR)];
    }
    let outline = source_edit::outline(source).unwrap_or_default();
    let quoted_re = Regex::new(r"'([^']+)'").ok();
    let lines: Vec<&str> = message.lines().map(str::trim).filter(|l| !l.is_empty() && !l.ends_with(':')).collect();
    let lines = if lines.is_empty() { vec![message.trim()] } else { lines };
    lines
        .into_iter()
        .map(|line| {
            let names: Vec<String> = quoted_re.as_ref().map(|re| re.captures_iter(line).map(|c| c[1].to_string()).collect()).unwrap_or_default();
            let written = names.iter().rev().find_map(|name| {
                let needle = format!("\"{name}\"");
                source.lines().enumerate().find_map(|(n, text)| text.find(&needle).map(|at| {
                    let column = text[..at].chars().count() as u32;
                    Range::new(Position::new(n as u32, column), Position::new(n as u32, column + needle.chars().count() as u32))
                }))
            });
            let declared = || names.iter().find_map(|name| find_declaration(&outline, name)).map(line_range);
            Diagnostic {
                range: written.or_else(declared).unwrap_or_else(|| Range::new(Position::new(0, 0), Position::new(0, 1))),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("arclang".to_string()),
                message: line.trim_start_matches("Semantic error: ").to_string(),
                ..Default::default()
            }
        })
        .collect()
}

/// `at line L, column C` in a compiler message, as a one-character range.
fn position_in(message: &str) -> Option<Range> {
    let position_re = Regex::new(r"at line (\d+), column (\d+)").ok()?;
    let caps = position_re.captures(message)?;
    let line: u32 = caps[1].parse().ok()?;
    let column: u32 = caps[2].parse().ok()?;
    // LSP positions are 0-based; compiler spans are 1-based.
    let (line, column) = (line.saturating_sub(1), column.saturating_sub(1));
    Some(Range::new(Position::new(line, column), Position::new(line, column + 1)))
}

fn diagnostic_from_message(message: &str, severity: DiagnosticSeverity) -> Diagnostic {
    Diagnostic {
        range: position_in(message).unwrap_or_else(|| Range::new(Position::new(0, 0), Position::new(0, 1))),
        severity: Some(severity),
        source: Some("arclang".to_string()),
        message: message.to_string(),
        ..Default::default()
    }
}

// --- the outline --------------------------------------------------------------

/// The declaration line of `declaration`, as a range.
fn line_range(declaration: &Declaration) -> Range {
    let line = declaration.line.saturating_sub(1) as u32;
    let column = declaration.column.saturating_sub(1) as u32;
    Range::new(Position::new(line, column), Position::new(line, column + declaration.keyword.len() as u32))
}

/// The declaration known by `reference`: by its written id, else by name.
fn find_declaration<'a>(outline: &'a [Declaration], reference: &str) -> Option<&'a Declaration> {
    fn walk<'a>(list: &'a [Declaration], by: &dyn Fn(&Declaration) -> bool, into: &mut Vec<&'a Declaration>) {
        for declaration in list {
            if by(declaration) {
                into.push(declaration);
            }
            walk(&declaration.children, by, into);
        }
    }
    let mut by_id = Vec::new();
    walk(outline, &|d| d.id.as_deref() == Some(reference), &mut by_id);
    if let Some(found) = by_id.first() {
        return Some(found);
    }
    let mut by_name = Vec::new();
    walk(outline, &|d| d.name.as_deref() == Some(reference) && d.keyword != "trace", &mut by_name);
    by_name.first().copied()
}

/// The innermost declaration whose block holds `line` (1-based), with its ancestors first.
fn enclosing<'a>(outline: &'a [Declaration], line: usize) -> Vec<&'a Declaration> {
    let mut path = Vec::new();
    let mut level = outline;
    loop {
        let Some(holder) = level.iter().find(|d| d.line <= line && line <= d.end_line) else { break };
        path.push(holder);
        level = &holder.children;
    }
    path
}

fn end_of(text: &str) -> Position {
    let lines: Vec<&str> = text.split('\n').collect();
    let last = lines.len().saturating_sub(1);
    Position::new(last as u32, lines.get(last).map(|l| l.chars().count() as u32).unwrap_or(0))
}

/// The symbols of the file, nested as the braces nest.
#[allow(deprecated)]
pub fn symbols(source: &str) -> Vec<DocumentSymbol> {
    fn convert(declaration: &Declaration) -> DocumentSymbol {
        // A trace is named by its whole line: its ends and its kind.
        let name = if declaration.keyword == "trace" {
            declaration.header.clone()
        } else {
            declaration.name.clone().unwrap_or_else(|| declaration.keyword.clone())
        };
        let detail = match (&declaration.id, &declaration.name) {
            (Some(id), _) => format!("{} {}", declaration.keyword, id),
            _ => declaration.keyword.clone(),
        };
        let start = Position::new(declaration.line.saturating_sub(1) as u32, declaration.column.saturating_sub(1) as u32);
        let end = Position::new(declaration.end_line.saturating_sub(1) as u32, u32::MAX / 2);
        DocumentSymbol {
            name,
            detail: Some(detail),
            kind: symbol_kind(&declaration.keyword),
            tags: None,
            deprecated: None,
            range: Range::new(start, end),
            selection_range: line_range(declaration),
            children: Some(declaration.children.iter().map(convert).collect()),
        }
    }
    source_edit::outline(source).unwrap_or_default().iter().map(convert).collect()
}

fn symbol_kind(keyword: &str) -> SymbolKind {
    match keyword {
        "model" | "operational_analysis" | "system_analysis" | "logical_architecture" | "physical_architecture" | "epbs" | "architecture" | "requirements" | "safety_analysis" | "security_analysis" | "security_scales" | "scenarios" => SymbolKind::MODULE,
        "component" | "node" | "entity" | "behavior_component" | "hardware_component" | "configuration_item" | "system" => SymbolKind::CLASS,
        "function" | "activity" | "operational_activity" | "functional_chain" | "operational_process" => SymbolKind::FUNCTION,
        "actor" => SymbolKind::OBJECT,
        "req" | "requirement" => SymbolKind::PROPERTY,
        "interface" | "port" => SymbolKind::INTERFACE,
        "trace" | "deployment" | "link" | "physical_link" | "component_exchange" | "functional_exchange" => SymbolKind::EVENT,
        "state_machine" | "state" | "mode" | "transition" => SymbolKind::ENUM_MEMBER,
        "scenario" => SymbolKind::EVENT,
        "class" | "data_type" | "exchange_item" | "type" => SymbolKind::STRUCT,
        "enumeration" => SymbolKind::ENUM,
        "diagram" => SymbolKind::FILE,
        "table" => SymbolKind::ARRAY, // tables hook
        _ => SymbolKind::VARIABLE,
    }
}

// --- the word under the cursor ------------------------------------------------

/// The identifier or quoted string at `character` of `line_text`, without quotes.
fn word_at(line_text: &str, character: u32) -> Option<String> {
    let chars: Vec<char> = line_text.chars().collect();
    let at = (character as usize).min(chars.len());
    let is_word = |c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.';
    // Inside a string: the whole string.
    let mut quotes_before = 0;
    let mut open = None;
    for (index, c) in chars.iter().enumerate().take(at) {
        if *c == '"' {
            quotes_before += 1;
            open = Some(index);
        }
    }
    if quotes_before % 2 == 1 {
        let start = open? + 1;
        let end = chars[start..].iter().position(|c| *c == '"').map(|p| start + p).unwrap_or(chars.len());
        return Some(chars[start..end].iter().collect());
    }
    let mut start = at;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    let mut end = at;
    while end < chars.len() && is_word(chars[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    Some(chars[start..end].iter().collect())
}

fn line_of(text: &str, line: u32) -> &str {
    text.split('\n').nth(line as usize).unwrap_or("")
}

// --- the metamodel behind the keywords ----------------------------------------

/// The metamodel kinds a declaration keyword may stand for.
fn kinds_of(keyword: &str) -> &'static [&'static str] {
    match keyword {
        "actor" => &["Actor", "SystemActor"],
        "entity" => &["OperationalEntity"],
        "operational_capability" => &["OperationalCapability"],
        "activity" | "operational_activity" => &["OperationalActivity"],
        "operational_process" => &["OperationalProcess"],
        "req" | "requirement" => &["Requirement"],
        "mission" => &["Mission"],
        "capability" => &["Capability"],
        "capability_realization" => &["CapabilityRealization"],
        "function" => &["SystemFunction", "LogicalFunction"],
        "functional_exchange" => &["FunctionalExchange"],
        "component" => &["LogicalComponent", "SystemComponent", "BehaviorComponent"],
        "behavior_component" => &["BehaviorComponent"],
        "hardware_component" => &["HardwareComponent"],
        "functional_chain" => &["FunctionalChain"],
        "port" => &["ComponentPort", "FunctionPort", "PhysicalPort"],
        "interface" => &["LogicalInterface"],
        "component_exchange" => &["ComponentExchange"],
        "node" => &["PhysicalNode"],
        "link" | "physical_link" => &["PhysicalLink"],
        "physical_exchange" => &["PhysicalExchange"],
        "deployment" => &["Deployment"],
        "physical_path" => &["PhysicalPath"],
        "configuration_item" => &["EpbsItem", "EpbsSubsystem", "EpbsSystem"],
        "class" => &["Class"],
        "enumeration" => &["Enumeration"],
        "data_type" => &["DataType"],
        "exchange_item" => &["ExchangeItem"],
        "hazard" => &["Hazard"],
        "fmea" => &["FmeaEntry"],
        "note" => &["Note"],
        "configuration" => &["Configuration"],
        "threat" => &["Threat"],
        "asset" => &["PrimaryAsset"],
        "trace" => &["Trace"],
        "test_case" => &["TestCase"],
        "model" => &["Model"],
        "type" => &["Type"],
        "constraint" => &["Constraint"],
        "state_machine" => &["StateMachine"],
        "state" | "mode" => &["State"],
        "transition" => &["Transition"],
        "scenario" => &["Scenario"],
        "message" => &["Message"],
        "diagram" => &["Diagram"],
        "table" => &["Table"], // tables hook
        _ => &[],
    }
}

/// Declaration keywords, by where they may be written.
const TOP_LEVEL: &[&str] = &[
    "model", "import", "operational_analysis", "system_analysis", "logical_architecture", "physical_architecture", "epbs",
    "architecture", "requirements", "safety_analysis", "security_analysis", "security_scales", "trace", "state_machine", "scenario", "scenarios", "class", "enumeration",
    "data_type", "exchange_item", "test_case", "constraint", "type", "diagram", "note", "configuration",
    "table", // tables hook
];
fn inside(keyword: &str) -> &'static [&'static str] {
    match keyword {
        "operational_analysis" => &["actor", "entity", "operational_capability", "operational_activity", "activity", "mission", "operational_process", "trace"],
        "system_analysis" => &["function", "actor", "component", "mission", "capability", "functional_chain", "functional_exchange", "requirement", "req"],
        "logical_architecture" | "architecture" => &["component", "interface", "component_exchange", "functional_exchange", "functional_chain", "capability_realization"],
        "physical_architecture" => &["node", "link", "physical_link", "physical_exchange", "deployment", "physical_path"],
        "epbs" => &["configuration_item", "system"],
        "requirements" => &["req", "requirement"],
        "safety_analysis" => &["hazard", "fmea"],
        "security_analysis" => &["threat", "asset"],
        "component" => &["function", "port", "component", "interface"],
        "node" => &["behavior_component", "hardware_component", "port", "deploys"],
        "entity" | "actor" => &["activity", "operational_activity"],
        "function" => &["port"],
        "state_machine" => &["state", "mode", "transition"],
        "scenario" | "scenarios" => &["scenario", "message"],
        _ => &[],
    }
}

fn kind_spec<'a>(metamodel: &'a Metamodel, name: &str) -> Option<&'a KindSpec> {
    metamodel.kind(name)
}

// --- completion -----------------------------------------------------------------

/// Completions at a position: enumeration values after `key:`, attribute
/// keys and nested keywords inside a block, top-level keywords outside.
pub fn complete(text: &str, line: u32, character: u32) -> Vec<CompletionItem> {
    let metamodel = Metamodel::current();
    let outline = source_edit::outline(text).unwrap_or_default();
    let path = enclosing(&outline, line as usize + 1);
    let current = line_of(text, line);
    let before: String = current.chars().take(character as usize).collect();
    let holder_keyword = path.last().map(|d| d.keyword.as_str()).unwrap_or("");
    let holder_kinds: Vec<&KindSpec> = kinds_of(holder_keyword).iter().filter_map(|k| kind_spec(&metamodel, k)).collect();

    // `key: ` or `key: [..., ` — the values the attribute may take: an
    // enumeration's members, or the elements of the model a reference names.
    let value_re = Regex::new(r#"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(\[[^\]]*)?"?[A-Za-z0-9_.\- ]*$"#).ok();
    if let Some(caps) = value_re.as_ref().and_then(|re| re.captures(&before)) {
        let key = &caps[1];
        let mut items: Vec<CompletionItem> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let sources: Vec<&KindSpec> = if holder_kinds.is_empty() { metamodel.kinds.iter().collect() } else { holder_kinds.clone() };
        let quoted = !before.trim_end().ends_with('"');
        for kind in &sources {
            for attribute in kind.attributes.iter().filter(|a| a.key == key) {
                let wanted = match &attribute.ty {
                    AttrType::Reference(k) => Some(*k),
                    AttrType::List(inner) => match inner.as_ref() { AttrType::Reference(k) => Some(*k), _ => None },
                    _ => None,
                };
                let Some(wanted) = wanted else { continue };
                for (designator, declaration) in referable(&outline, wanted) {
                    if seen.insert(designator.clone()) {
                        items.push(CompletionItem {
                            label: designator.clone(),
                            kind: Some(CompletionItemKind::REFERENCE),
                            detail: Some(format!("{} {}", declaration.keyword, declaration.name.clone().unwrap_or_default())),
                            insert_text: Some(if quoted { format!("\"{designator}\"") } else { designator.clone() }),
                            ..Default::default()
                        });
                    }
                }
            }
        }
        for kind in sources {
            for attribute in kind.attributes.iter().filter(|a| a.key == key) {
                if let AttrType::Enum(name) = &attribute.ty {
                    if let Some(spec) = metamodel.enumeration(name) {
                        for value in spec.values.iter() {
                            if seen.insert(value.to_string()) {
                                items.push(CompletionItem {
                                    label: value.to_string(),
                                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                                    detail: Some(format!("{}: {}", name, spec.doc)),
                                    insert_text: Some(if quoted { format!("\"{value}\"") } else { value.to_string() }),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }
        return items;
    }

    // At the start of a line inside a block: attributes of the kind, then
    // what may be declared inside; outside any block: top-level keywords.
    let word_re = Regex::new(r"^\s*[A-Za-z_]*$").ok();
    if !word_re.as_ref().is_some_and(|re| re.is_match(&before)) {
        return Vec::new();
    }
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for kind in &holder_kinds {
        for attribute in &kind.attributes {
            if seen.insert(attribute.key.to_string()) {
                items.push(CompletionItem {
                    label: attribute.key.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(format!("{} ({})", attribute.ty.notation(), kind.name)),
                    documentation: Some(Documentation::String(attribute.doc.to_string())),
                    insert_text: Some(format!("{}: ", attribute.key)),
                    sort_text: Some(format!("1{}", attribute.key)),
                    ..Default::default()
                });
            }
        }
    }
    let keywords: &[&str] = if path.is_empty() { TOP_LEVEL } else { inside(holder_keyword) };
    for keyword in keywords {
        if seen.insert(keyword.to_string()) {
            let doc = kinds_of(keyword).first().and_then(|k| kind_spec(&metamodel, k)).map(|k| k.doc.to_string());
            items.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: doc.clone(),
                sort_text: Some(format!("2{keyword}")),
                ..Default::default()
            });
        }
    }
    items
}

/// The declarations a reference to `kind` may name (`Element`: any), by
/// their id, else their name, in the order of the file.
fn referable<'a>(outline: &'a [source_edit::Declaration], kind: &str) -> Vec<(String, &'a source_edit::Declaration)> {
    let mut out = Vec::new();
    let mut stack: Vec<&source_edit::Declaration> = outline.iter().rev().collect();
    while let Some(declaration) = stack.pop() {
        stack.extend(declaration.children.iter().rev());
        let kinds = kinds_of(&declaration.keyword);
        let fits = if kind == "Element" { !kinds.is_empty() && !matches!(declaration.keyword.as_str(), "model" | "trace") } else { kinds.iter().any(|k| kind.split(&[',', ' '][..]).any(|w| w == *k)) };
        if !fits {
            continue;
        }
        if let Some(designator) = declaration.id.clone().or_else(|| declaration.name.clone()) {
            out.push((designator, declaration));
        }
    }
    out
}

// --- hover ----------------------------------------------------------------------

/// What the metamodel and the file say about the word under the cursor.
pub fn hover(text: &str, line: u32, character: u32) -> Option<String> {
    let current = line_of(text, line);
    let word = word_at(current, character)?;
    let metamodel = Metamodel::current();
    let outline = source_edit::outline(text).unwrap_or_default();
    let path = enclosing(&outline, line as usize + 1);

    // A declaration keyword: the kind it declares.
    let kinds: Vec<&KindSpec> = kinds_of(&word).iter().filter_map(|k| kind_spec(&metamodel, k)).collect();
    if !kinds.is_empty() && current.trim_start().starts_with(word.as_str()) {
        let mut out = String::new();
        for kind in kinds {
            out.push_str(&format!("**{}** — {} (Arcadia: {}; SysML: {})\n\n{}\n\n", kind.name, kind.layer.label(), kind.arcadia, kind.sysml, kind.doc));
        }
        return Some(out.trim_end().to_string());
    }
    // An attribute key: its type and documentation on the enclosing kind.
    let after: String = current.chars().skip_while(|c| c.is_whitespace()).collect();
    if after.starts_with(&format!("{word}:")) || after.starts_with(&format!("{word} :")) {
        let holder = path.last().map(|d| d.keyword.as_str()).unwrap_or("");
        let mut out = String::new();
        let sources: Vec<&KindSpec> = {
            let own: Vec<&KindSpec> = kinds_of(holder).iter().filter_map(|k| kind_spec(&metamodel, k)).collect();
            if own.is_empty() { metamodel.kinds.iter().collect() } else { own }
        };
        for kind in sources {
            if let Some(attribute) = kind.attributes.iter().find(|a| a.key == word) {
                out.push_str(&format!("**{}** on {}: `{}`\n\n{}\n\n", attribute.key, kind.name, attribute.ty.notation(), attribute.doc));
                if let AttrType::Enum(name) = &attribute.ty {
                    if let Some(spec) = metamodel.enumeration(name) {
                        out.push_str(&format!("Values: {}\n\n", spec.values.join(", ")));
                    }
                }
            }
        }
        if !out.is_empty() {
            return Some(out.trim_end().to_string());
        }
    }
    // A reference to an element: where it is declared.
    let declaration = find_declaration(&outline, &word)?;
    Some(format!(
        "**{}** `{}`{} — declared at line {}",
        declaration.name.clone().unwrap_or_default(),
        declaration.keyword,
        declaration.id.as_ref().map(|id| format!(" (id `{id}`)")).unwrap_or_default(),
        declaration.line
    ))
}

// --- definition ---------------------------------------------------------------

/// The declaration line of the element named under the cursor.
pub fn definition(text: &str, line: u32, character: u32) -> Option<Range> {
    let word = word_at(line_of(text, line), character)?;
    let outline = source_edit::outline(text).unwrap_or_default();
    find_declaration(&outline, &word).map(line_range)
}

// --- rename -------------------------------------------------------------------

/// Rename the element declared or referenced under the cursor; the edited
/// text, or why it was refused.
pub fn rename(text: &str, line: u32, character: u32, new_name: &str) -> Result<String, String> {
    let word = word_at(line_of(text, line), character).ok_or("no element under the cursor")?;
    let outline = source_edit::outline(text).unwrap_or_default();
    let declaration = find_declaration(&outline, &word).ok_or_else(|| format!("'{word}' is not declared in this file"))?;
    let designator = declaration.id.clone().or_else(|| declaration.name.clone()).ok_or("the declaration carries no name")?;
    let before = Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| format!("the model does not compile: {e}"))?;
    let edited = source_edit::rename_element(text, &designator, new_name).map_err(|e| e.to_string())?;
    let after = Compiler::new(CompilerConfig::default()).compile_string(&edited).map_err(|e| format!("edit refused: the renamed model does not compile: {e}"))?;
    let identities = |result: &crate::CompilationResult| {
        let graph = elements::build(&result.ast, &result.semantic_model);
        let mut all: Vec<String> = graph.elements.iter().map(|e| e.uuid.clone()).collect();
        all.extend(graph.relationships.iter().map(|r| r.uuid.clone()));
        all.sort();
        all
    };
    if identities(&before) != identities(&after) {
        return Err(format!(
            "edit refused: renaming '{designator}' would change its identity or what refers to it; give it an `id` and refer to it by that id first"
        ));
    }
    Ok(edited)
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

    const MODEL: &str = r#"model M {
    name: "M"
}
requirements system {
    req "REQ-001" "Brake in time" {
        description: "The system shall brake."
        safety_level: "ASIL-B"
    }
}
logical_architecture LA {
    component "Controller" {
        id: "LC-CTL"
        description: "Controls"
        function "Compute" {}
    }
}
trace "LC-CTL" satisfies "REQ-001"
"#;

    #[test]
    fn error_diagnostic_carries_compiler_position() {
        let diagnostics = compute_diagnostics("model M {\n  name: \"x\"\n}\nlogical_architecture LA {\n  component A {\n    bogus 12\n  }\n}\n");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostics[0].range.start, Position::new(5, 10));
    }

    #[test]
    fn findings_carry_their_rule_and_point_at_the_element() {
        let source = "logical_architecture LA {\n    component \"Lonely\" { id: \"LC-1\" }\n}\n";
        let diagnostics = compute_diagnostics(source);
        let trace = diagnostics.iter().find(|d| d.code == Some(NumberOrString::String("trace.component".to_string()))).expect("trace finding");
        assert_eq!(trace.range.start.line, 1);
        assert_eq!(trace.severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn clean_model_has_no_error() {
        assert!(compute_diagnostics(MODEL).iter().all(|d| d.severity != Some(DiagnosticSeverity::ERROR)));
    }

    #[test]
    fn symbols_nest_as_the_braces_do() {
        let symbols = symbols(MODEL);
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["M", "system", "LA"]);
        let la = &symbols[2];
        let controller = &la.children.as_ref().unwrap()[0];
        assert_eq!(controller.name, "Controller");
        assert_eq!(controller.detail.as_deref(), Some("component LC-CTL"));
        assert_eq!(controller.children.as_ref().unwrap()[0].name, "Compute");
        assert_eq!(controller.selection_range.start.line, 10);
    }

    #[test]
    fn definition_finds_the_declaration_by_id_or_name() {
        // `trace "LC-CTL" satisfies "REQ-001"` is on line 16 (0-based).
        let by_id = definition(MODEL, 16, 8).expect("by id");
        assert_eq!(by_id.start.line, 10);
        let by_name = definition(MODEL, 16, 28).expect("by name");
        assert_eq!(by_name.start.line, 4);
        assert!(definition(MODEL, 1, 5).is_none());
    }

    #[test]
    fn a_semantic_error_is_shown_where_the_name_is_written() {
        let source = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"Camera\" { id: \"LC-CAM\" }\n}\nnote \"Runs at 30 Hz\" { about: [\"LC-NOPE\"] }\n";
        let diagnostics = compute_diagnostics(source);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0].range.start, Position::new(4, 31), "{diagnostics:?}");
        assert!(!diagnostics[0].message.starts_with("Semantic error"), "{}", diagnostics[0].message);
        assert!(diagnostics[0].message.contains("about 'LC-NOPE' names no element"), "{}", diagnostics[0].message);
    }

    #[test]
    fn references_complete_with_the_elements_of_the_right_kind() {
        let source = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"Camera\" { id: \"LC-CAM\" }\n}\nsafety_analysis {\n    hazard \"Unintended steering\" { id: \"HAZ-1\" }\n    fmea \"Stuck\" {\n        leads_to: [\n    }\n}\nnote \"x\" {\n    about: \n}\n";
        let labels = |line: u32, character: u32| complete(source, line, character).into_iter().map(|i| i.label).collect::<Vec<_>>();
        let hazards = labels(7, 19);
        assert!(hazards.contains(&"HAZ-1".to_string()) && !hazards.contains(&"LC-CAM".to_string()), "{hazards:?}");
        let any = labels(11, 11);
        assert!(any.contains(&"LC-CAM".to_string()) && any.contains(&"HAZ-1".to_string()), "{any:?}");
        let top = labels(13, 0);
        assert!(top.contains(&"note".to_string()) && top.contains(&"configuration".to_string()), "{top:?}");
        let safety = complete("safety_analysis {\n    \n}\n", 1, 4).into_iter().map(|i| i.label).collect::<Vec<_>>();
        assert!(safety.contains(&"fmea".to_string()), "{safety:?}");
    }

    #[test]
    fn completion_offers_attributes_inside_a_block_and_enum_values_after_a_key() {
        let text = "logical_architecture LA {\n    component \"C\" {\n        \n    }\n}\n";
        let inside = complete(text, 2, 8);
        let labels: Vec<&str> = inside.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"description"), "{labels:?}");
        assert!(labels.contains(&"safety_level"));
        assert!(labels.contains(&"function"));
        let values = complete("logical_architecture LA {\n    component \"C\" {\n        safety_level: \n    }\n}\n", 2, 22);
        let labels: Vec<&str> = values.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"ASIL-D"), "{labels:?}");
        assert_eq!(values[0].insert_text.as_deref().map(|t| t.starts_with('"')), Some(true));
        let top = complete("", 0, 0);
        assert!(top.iter().any(|i| i.label == "logical_architecture"));
    }

    #[test]
    fn hover_explains_keywords_attributes_and_references() {
        let keyword = hover(MODEL, 10, 6).expect("keyword");
        assert!(keyword.contains("LogicalComponent"));
        let attribute = hover(MODEL, 6, 10).expect("attribute");
        assert!(attribute.contains("safety_level") && attribute.contains("ASIL-B"));
        let reference = hover(MODEL, 16, 8).expect("reference");
        assert!(reference.contains("declared at line 11"));
    }

    #[test]
    fn rename_goes_through_the_source_editor_and_keeps_identity() {
        let renamed = rename(MODEL, 10, 16, "Brake controller").expect("renamed");
        assert!(renamed.contains("component \"Brake controller\" {"));
        assert!(renamed.contains("description: \"Controls\""));
        // Known by name elsewhere: refused.
        let by_name = "logical_architecture LA {\n    component \"A\" {}\n    component \"B\" {}\n    component_exchange \"x\" { from: \"A\" to: \"B\" }\n}\n";
        assert!(rename(by_name, 1, 16, "C").is_err());
    }
}
