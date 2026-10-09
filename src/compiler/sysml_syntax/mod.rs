//! The abstract syntax of an exported SysML v2 text: which KerML / SysML
//! metaclass each declaration is, what owns it, what it specializes and
//! what a connector connects.
//!
//! This reads ONE dialect: the subset of the SysML v2 textual notation that
//! `sysmlv2_generator` writes. It is not a SysML parser. Reading the export
//! back, rather than mapping the ArcLang model a second time, keeps a single
//! definition of what a model means in SysML: the text the OMG pilot
//! implementation validates. The reader itself is checked against the
//! pilot's own abstract syntax of the same text
//! (`tools/sysml_abstract_syntax_check.py`). A statement it does not know is
//! an error, never a guess.
//!
//! Feature values and constraint bodies are parsed into expressions
//! (`expression`). Not read: multiplicities, and everything the pilot
//! derives implicitly (connector end features, conjugated port definitions,
//! implied specializations).

use std::collections::HashMap;

mod expression;

pub use expression::Expr;
use expression::Expressions;

/// One declared element of the export.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Declaration {
    /// KerML / SysML v2 metaclass: `PartDefinition`, `ActionUsage`...
    pub metaclass: &'static str,
    /// Metaclass of the membership through which the owner holds it.
    pub membership: &'static str,
    pub name: Option<String>,
    pub short_name: Option<String>,
    /// Index of the owning declaration; `None` for the root package.
    pub owner: Option<usize>,
    pub direction: Option<&'static str>,
    pub is_abstract: bool,
    /// `: T` — names as written.
    pub typed_by: Vec<String>,
    /// `:> S`
    pub specializes: Vec<String>,
    /// `:>> R`
    pub redefines: Vec<String>,
    /// What a `satisfy` or `verify` refers to.
    pub references: Vec<String>,
    /// Source and target of a connector, as written (`p_A.out`).
    pub ends: Option<(String, String)>,
    /// Send and receive events of a message, as written.
    pub events: Option<(String, String)>,
    /// Body of a `doc`.
    pub body: Option<String>,
    /// Text of the expression bound to a feature, or asserted by a constraint.
    pub expression: Option<String>,
    /// That expression, parsed.
    pub value: Option<Expr>,
    /// Whether the value is a default (`default =`) rather than bound (`=`).
    pub is_default: bool,
    /// Bounds of a `MultiplicityRange`, as written: `[4]` has no lower
    /// bound of its own, and an upper bound of `None` is `*`.
    pub bounds: Option<(Option<u64>, Option<u64>)>,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    /// A name written between single quotes.
    Quoted(String),
    Text(String),
    Number(String),
    Symbol(&'static str),
    /// `doc /* body */`
    Doc(String),
}

impl Token {
    fn source(&self) -> String {
        match self {
            Token::Word(text) | Token::Number(text) => text.clone(),
            Token::Quoted(name) => format!("'{}'", name.replace('\\', "\\\\").replace('\'', "\\'")),
            Token::Text(text) => text.clone(),
            Token::Symbol(symbol) => (*symbol).to_string(),
            Token::Doc(body) => format!("doc /* {} */", body),
        }
    }
}

const SYMBOLS: &[&str] = &[
    ":>>", "::", ":>", "<=", ">=", "==", "!=", "{", "}", ";", ",", ".", "(", ")", "[", "]", ":", "=", "<", ">",
    "+", "-", "*", "/",
];

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    let rest_starts = |i: usize, pattern: &str| pattern.chars().enumerate().all(|(k, c)| chars.get(i + k) == Some(&c));
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if rest_starts(i, "//") {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if rest_starts(i, "/*") {
            let start = i + 2;
            let mut end = start;
            while end < chars.len() && !rest_starts(end, "*/") {
                end += 1;
            }
            if end >= chars.len() {
                return Err("unterminated comment".to_string());
            }
            // A comment is the body of the `doc` it follows; otherwise a note.
            if tokens.last() == Some(&Token::Word("doc".to_string())) {
                tokens.pop();
                let body: String = chars[start..end].iter().collect();
                tokens.push(Token::Doc(body.trim().to_string()));
            }
            i = end + 2;
        } else if c == '\'' || c == '"' {
            let mut value = String::new();
            let mut raw = String::from(c);
            i += 1;
            loop {
                match chars.get(i) {
                    None => return Err("unterminated name or string".to_string()),
                    Some('\\') => {
                        let escaped = *chars.get(i + 1).ok_or("unterminated escape")?;
                        value.push(escaped);
                        raw.push('\\');
                        raw.push(escaped);
                        i += 2;
                    }
                    Some(end) if *end == c => {
                        raw.push(c);
                        i += 1;
                        break;
                    }
                    Some(other) => {
                        value.push(*other);
                        raw.push(*other);
                        i += 1;
                    }
                }
            }
            tokens.push(if c == '\'' { Token::Quoted(value) } else { Token::Text(raw) });
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token::Word(chars[start..i].iter().collect()));
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric()
                    || (chars[i] == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())))
            {
                i += 1;
            }
            tokens.push(Token::Number(chars[start..i].iter().collect()));
        } else if let Some(symbol) = SYMBOLS.iter().find(|symbol| rest_starts(i, symbol)) {
            tokens.push(Token::Symbol(symbol));
            i += symbol.len();
        } else {
            return Err(format!("unexpected character '{}'", c));
        }
    }
    Ok(tokens)
}

fn is_symbol(token: Option<&Token>, symbol: &str) -> bool {
    matches!(token, Some(Token::Symbol(s)) if *s == symbol)
}

fn is_word(token: Option<&Token>, word: &str) -> bool {
    matches!(token, Some(Token::Word(w)) if w == word)
}

/// Source text of an expression, spaced the way the exporter writes it.
fn expression_text(tokens: &[Token]) -> String {
    let mut text = String::new();
    for (index, token) in tokens.iter().enumerate() {
        let tight_before = matches!(token, Token::Symbol("." | "," | ")" | "]" | "::"));
        let tight_after = index > 0 && matches!(tokens[index - 1], Token::Symbol("." | "(" | "[" | "::"));
        // `f(x)`: a call keeps its parenthesis attached.
        let call = matches!(token, Token::Symbol("(")) && index > 0 && matches!(tokens[index - 1], Token::Word(_));
        if index > 0 && !tight_before && !tight_after && !call {
            text.push(' ');
        }
        text.push_str(&token.source());
    }
    text
}

/// Keywords that open a declaration, with the metaclass of the usage and of
/// the definition (`part` / `part def`).
const KINDS: &[(&[&str], &str, Option<&str>)] = &[
    (&["package"], "Package", None),
    (&["part"], "PartUsage", Some("PartDefinition")),
    (&["action"], "ActionUsage", Some("ActionDefinition")),
    (&["item"], "ItemUsage", Some("ItemDefinition")),
    (&["attribute"], "AttributeUsage", Some("AttributeDefinition")),
    (&["port"], "PortUsage", Some("PortDefinition")),
    (&["requirement"], "RequirementUsage", Some("RequirementDefinition")),
    (&["constraint"], "ConstraintUsage", Some("ConstraintDefinition")),
    (&["connection"], "ConnectionUsage", Some("ConnectionDefinition")),
    (&["interface"], "InterfaceUsage", Some("InterfaceDefinition")),
    (&["state"], "StateUsage", Some("StateDefinition")),
    (&["use", "case"], "UseCaseUsage", Some("UseCaseDefinition")),
    (&["verification"], "VerificationCaseUsage", Some("VerificationCaseDefinition")),
    (&["occurrence"], "OccurrenceUsage", Some("OccurrenceDefinition")),
    (&["enum"], "EnumerationUsage", Some("EnumerationDefinition")),
];

/// Name under which a state definition's entry action is known: it
/// redefines the library feature of that name.
pub const ENTRY_ACTION: &str = "entryAction";

struct Reader {
    tokens: Vec<Token>,
    position: usize,
    declarations: Vec<Declaration>,
}

/// What ended a statement head.
#[derive(PartialEq)]
enum End {
    Semicolon,
    Body,
    /// The closing brace of the enclosing body: a trailing expression.
    Close,
}

/// Cursor over the tokens of one statement head.
struct Head<'t> {
    tokens: &'t [Token],
    position: usize,
}

impl<'t> Head<'t> {
    fn peek(&self) -> Option<&'t Token> {
        self.tokens.get(self.position)
    }

    fn take_word(&mut self, word: &str) -> bool {
        let found = is_word(self.peek(), word);
        if found {
            self.position += 1;
        }
        found
    }

    fn take_symbol(&mut self, symbol: &str) -> bool {
        let found = is_symbol(self.peek(), symbol);
        if found {
            self.position += 1;
        }
        found
    }

    /// A name: a basic identifier or a quoted name.
    fn name(&mut self) -> Option<String> {
        let name = match self.peek()? {
            Token::Word(word) => word.clone(),
            Token::Quoted(name) => name.clone(),
            _ => return None,
        };
        self.position += 1;
        Some(name)
    }

    /// `A`, `A::B` or a feature chain `a.b.c`, as written.
    fn path(&mut self) -> Result<String, String> {
        let mut path = self.name().ok_or_else(|| self.unexpected("a name"))?;
        loop {
            let separator = match self.peek() {
                Some(Token::Symbol(".")) => ".",
                Some(Token::Symbol("::")) => "::",
                _ => return Ok(path),
            };
            self.position += 1;
            let segment = self.name().ok_or_else(|| self.unexpected("a name"))?;
            path.push_str(separator);
            path.push_str(&segment);
        }
    }

    fn paths(&mut self) -> Result<Vec<String>, String> {
        let mut paths = vec![self.path()?];
        while self.take_symbol(",") {
            paths.push(self.path()?);
        }
        Ok(paths)
    }

    fn rest(&mut self) -> &'t [Token] {
        let rest = &self.tokens[self.position.min(self.tokens.len())..];
        self.position = self.tokens.len();
        rest
    }

    fn done(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn unexpected(&self, expected: &str) -> String {
        format!("expected {} in `{}`", expected, expression_text(self.tokens))
    }
}

impl Reader {
    fn head(&mut self) -> Result<(Vec<Token>, End), String> {
        let start = self.position;
        let mut depth = 0usize;
        loop {
            let token = self.tokens.get(self.position).ok_or("unexpected end of the export")?;
            let end = match token {
                Token::Symbol("(" | "[") => {
                    depth += 1;
                    None
                }
                Token::Symbol(")" | "]") => {
                    depth = depth.saturating_sub(1);
                    None
                }
                Token::Symbol(";") if depth == 0 => Some(End::Semicolon),
                Token::Symbol("{") if depth == 0 => Some(End::Body),
                Token::Symbol("}") if depth == 0 => Some(End::Close),
                _ => None,
            };
            match end {
                None => self.position += 1,
                Some(end) => {
                    let head = self.tokens[start..self.position].to_vec();
                    if end != End::Close {
                        self.position += 1;
                    }
                    return Ok((head, end));
                }
            }
        }
    }

    fn push(&mut self, mut declaration: Declaration, owner: Option<usize>) -> usize {
        declaration.owner = owner;
        if declaration.membership.is_empty() {
            let owner_is_namespace_only = owner.map_or(true, |o| self.declarations[o].metaclass == "Package");
            let is_feature = declaration.metaclass.ends_with("Usage");
            declaration.membership = if is_feature && !owner_is_namespace_only {
                "FeatureMembership"
            } else {
                "OwningMembership"
            };
        }
        self.declarations.push(declaration);
        self.declarations.len() - 1
    }

    /// The statements of a body, up to its closing brace (or the end of the
    /// text for the top level).
    fn body(&mut self, owner: Option<usize>) -> Result<(), String> {
        // The last action or occurrence declared here: what `then` follows.
        let mut previous: Option<String> = None;
        loop {
            match self.tokens.get(self.position) {
                None if owner.is_none() => return Ok(()),
                None => return Err("unclosed body".to_string()),
                Some(Token::Symbol("}")) => {
                    self.position += 1;
                    return Ok(());
                }
                Some(Token::Doc(body)) => {
                    let documentation = Declaration {
                        metaclass: "Documentation",
                        membership: "OwningMembership",
                        body: Some(body.clone()),
                        ..Declaration::default()
                    };
                    self.position += 1;
                    self.push(documentation, owner);
                    continue;
                }
                Some(_) => {}
            }
            let (head, end) = self.head()?;
            if end == End::Close {
                // A constraint's asserted expression closes its body.
                let owner = owner.ok_or("expression outside a declaration")?;
                self.declarations[owner].expression = Some(expression_text(&head));
                self.declarations[owner].value = Some(Expressions::parse(&head)?);
                continue;
            }
            let declared = self.statement(&head, owner, &mut previous)?;
            if end == End::Body {
                let inner = declared.ok_or_else(|| format!("`{}` cannot have a body", expression_text(&head)))?;
                self.body(Some(inner))?;
            }
        }
    }

    /// Read one statement head. Returns the declaration a body would belong to.
    fn statement(&mut self, tokens: &[Token], owner: Option<usize>, previous: &mut Option<String>) -> Result<Option<usize>, String> {
        let mut head = Head { tokens, position: 0 };
        let mut declaration = Declaration::default();

        head.take_word("private");
        if head.take_word("import") {
            return Ok(None); // library imports are not declarations of the model
        }
        declaration.is_abstract = head.take_word("abstract");
        let follows = head.take_word("then");
        for direction in ["in", "out", "inout"] {
            if head.take_word(direction) {
                declaration.direction = Some(direction);
            }
        }
        let is_end = head.take_word("end");
        head.take_word("ref");
        head.take_word("event");

        // Connector and relationship statements that start with their own keyword.
        if head.take_word("connect") {
            declaration.metaclass = "ConnectionUsage";
            declaration.ends = Some(Self::ends(&mut head, "to")?);
        } else if head.take_word("allocate") {
            declaration.metaclass = "AllocationUsage";
            declaration.ends = Some(Self::ends(&mut head, "to")?);
        } else if head.take_word("flow") {
            declaration.metaclass = "FlowUsage";
            Self::expect_word(&mut head, "from")?;
            declaration.ends = Some(Self::ends(&mut head, "to")?);
        } else if head.take_word("message") {
            // A message is a flow between two events, with no connector ends.
            declaration.metaclass = "FlowUsage";
            declaration.is_abstract = true;
            declaration.name = head.name();
            Self::expect_word(&mut head, "from")?;
            declaration.events = Some(Self::ends(&mut head, "to")?);
        } else if head.take_word("dependency") {
            declaration.metaclass = "Dependency";
            if !is_word(head.peek(), "from") {
                declaration.name = head.name();
            }
            Self::expect_word(&mut head, "from")?;
            declaration.ends = Some(Self::ends(&mut head, "to")?);
        } else if head.take_word("satisfy") {
            declaration.metaclass = "SatisfyRequirementUsage";
            let requirement = head.path()?;
            Self::expect_word(&mut head, "by")?;
            declaration.ends = Some((head.path()?, requirement.clone()));
            declaration.references = vec![requirement];
        } else if head.take_word("verify") {
            declaration.metaclass = "RequirementUsage";
            declaration.membership = "RequirementVerificationMembership";
            declaration.references = vec![head.path()?];
        } else if head.take_word("objective") {
            declaration.metaclass = "RequirementUsage";
            declaration.membership = "ObjectiveMembership";
        } else if head.take_word("transition") {
            // A transition owns the succession from its source to its target.
            declaration.metaclass = "TransitionUsage";
            declaration.name = head.name();
            Self::expect_word(&mut head, "first")?;
            let succession = Declaration {
                metaclass: "SuccessionAsUsage",
                membership: "OwningMembership",
                ends: Some(Self::ends(&mut head, "then")?),
                ..Declaration::default()
            };
            self.finish(&head)?;
            let transition = self.push(declaration, owner);
            self.push(succession, Some(transition));
            return Ok(Some(transition));
        } else if head.take_word("entry") {
            // `entry;` — the empty entry action of a state definition.
            declaration.metaclass = "ActionUsage";
            declaration.membership = "StateSubactionMembership";
        } else if head.take_word("perform") {
            Self::expect_word(&mut head, "action")?;
            declaration.metaclass = "PerformActionUsage";
            self.named(&mut head, &mut declaration)?;
        } else if head.take_word("include") {
            Self::expect_word(&mut head, "use")?;
            Self::expect_word(&mut head, "case")?;
            declaration.metaclass = "IncludeUseCaseUsage";
            self.named(&mut head, &mut declaration)?;
        } else if head.take_word("assert") {
            Self::expect_word(&mut head, "constraint")?;
            declaration.metaclass = "AssertConstraintUsage";
            self.named(&mut head, &mut declaration)?;
        } else if let Some((keywords, usage, definition)) =
            KINDS.iter().find(|(keywords, _, _)| keywords.iter().enumerate().all(|(k, word)| is_word(tokens.get(head.position + k), word)))
        {
            head.position += keywords.len();
            declaration.metaclass = match (head.take_word("def"), definition) {
                (true, Some(definition)) => definition,
                (true, None) => return Err(head.unexpected("no `def`")),
                (false, _) => usage,
            };
            if tokens.first() == Some(&Token::Word("event".to_string())) || is_word(tokens.get(1), "event") {
                declaration.metaclass = "EventOccurrenceUsage";
            }
            self.named(&mut head, &mut declaration)?;
            // An enumeration is a variation: abstract, its values the variants.
            declaration.is_abstract |= declaration.metaclass == "EnumerationDefinition";
        } else if is_end || is_symbol(head.peek(), ":>>") {
            // A feature declared without a keyword: a connector end
            // (`end source;`) or a bare redefinition (`:>> prefix = mega;`).
            declaration.metaclass = "ReferenceUsage";
            self.named(&mut head, &mut declaration)?;
            // The typed ends of an interface definition are ports.
            let in_interface = owner.is_some_and(|o| self.declarations[o].metaclass == "InterfaceDefinition");
            if is_end && in_interface && !declaration.typed_by.is_empty() {
                declaration.metaclass = "PortUsage";
            }
        } else if follows && owner.is_some_and(|o| self.declarations[o].metaclass == "StateDefinition") {
            // `then S;` after `entry;` — the succession to the initial state.
            let target = head.path()?;
            declaration.metaclass = "SuccessionAsUsage";
            declaration.ends = Some((ENTRY_ACTION.to_string(), target));
            self.finish(&head)?;
            return Ok(Some(self.push(declaration, owner)));
        } else if owner.is_some_and(|o| self.declarations[o].metaclass == "EnumerationDefinition") {
            declaration.metaclass = "EnumerationUsage";
            declaration.membership = "VariantMembership";
            declaration.name = Some(head.name().ok_or_else(|| head.unexpected("an enumeration value"))?);
        } else {
            return Err(format!("unknown statement `{}`", expression_text(tokens)));
        }
        self.finish(&head)?;

        if follows {
            // `then X` declares the succession from the previous step first.
            if let (Some(before), Some(name)) = (previous.as_ref(), declaration.name.as_ref()) {
                let succession = Declaration {
                    metaclass: "SuccessionAsUsage",
                    ends: Some((before.clone(), name.clone())),
                    ..Declaration::default()
                };
                self.push(succession, owner);
            }
        }
        if matches!(declaration.metaclass, "ActionUsage" | "EventOccurrenceUsage") && declaration.name.is_some() {
            *previous = declaration.name.clone();
        }
        // A stated multiplicity is an element of its own, owned by the usage.
        let bounds = declaration.bounds.take();
        let declared = self.push(declaration, owner);
        if bounds.is_some() {
            let range = Declaration { metaclass: "MultiplicityRange", membership: "OwningMembership", bounds, ..Declaration::default() };
            self.push(range, Some(declared));
        }
        Ok(Some(declared))
    }

    fn expect_word(head: &mut Head, word: &str) -> Result<(), String> {
        if head.take_word(word) {
            Ok(())
        } else {
            Err(head.unexpected(&format!("`{}`", word)))
        }
    }

    fn ends(head: &mut Head, separator: &str) -> Result<(String, String), String> {
        let source = head.path()?;
        Self::expect_word(head, separator)?;
        Ok((source, head.path()?))
    }

    /// `<short> name : T :> S :>> R connect a to b = expression`
    fn named(&mut self, head: &mut Head, declaration: &mut Declaration) -> Result<(), String> {
        if head.take_symbol("<") {
            declaration.short_name = Some(head.name().ok_or_else(|| head.unexpected("a short name"))?);
            if !head.take_symbol(">") {
                return Err(head.unexpected("`>`"));
            }
        }
        let starts_clause = |head: &Head| {
            matches!(head.peek(), Some(Token::Symbol(":" | ":>" | ":>>" | "=")))
                || is_word(head.peek(), "connect")
                || is_word(head.peek(), "default")
        };
        if !head.done() && !starts_clause(head) {
            declaration.name = Some(head.name().ok_or_else(|| head.unexpected("a name"))?);
        }
        loop {
            if head.take_symbol(":>>") {
                declaration.redefines.extend(head.paths()?);
            } else if head.take_symbol(":>") {
                declaration.specializes.extend(head.paths()?);
            } else if head.take_symbol(":") {
                declaration.typed_by.extend(head.paths()?);
            } else if head.take_symbol("[") {
                declaration.bounds = Some(Self::bounds(head)?);
            } else if head.take_word("connect") {
                declaration.ends = Some(Self::ends(head, "to")?);
            } else if is_word(head.peek(), "default") || is_symbol(head.peek(), "=") {
                declaration.is_default = head.take_word("default");
                if !head.take_symbol("=") {
                    return Err(head.unexpected("`=`"));
                }
                let expression = head.rest();
                declaration.expression = Some(expression_text(expression));
                declaration.value = Some(Expressions::parse(expression)?);
            } else {
                return Ok(());
            }
        }
    }

    /// `4]`, `0..1]`, `1..*]`, `*]` — after the opening bracket.
    fn bounds(head: &mut Head) -> Result<(Option<u64>, Option<u64>), String> {
        fn bound(head: &mut Head) -> Result<Option<u64>, String> {
            if head.take_symbol("*") {
                return Ok(None);
            }
            match head.peek() {
                Some(Token::Number(text)) => {
                    let value = text.parse().map_err(|_| head.unexpected("a multiplicity bound"))?;
                    head.position += 1;
                    Ok(Some(value))
                }
                _ => Err(head.unexpected("a multiplicity bound")),
            }
        }
        let first = bound(head)?;
        let range = if head.take_symbol(".") {
            if !head.take_symbol(".") {
                return Err(head.unexpected("`..`"));
            }
            (first, bound(head)?)
        } else {
            (None, first)
        };
        if !head.take_symbol("]") {
            return Err(head.unexpected("`]`"));
        }
        Ok(range)
    }

    fn finish(&self, head: &Head) -> Result<(), String> {
        if head.done() {
            Ok(())
        } else {
            Err(format!("unread `{}` in `{}`", expression_text(&head.tokens[head.position..]), expression_text(head.tokens)))
        }
    }
}

/// Read the declarations of an exported SysML v2 text, in document order.
/// The owner of a declaration always precedes it.
pub fn read(text: &str) -> Result<Vec<Declaration>, String> {
    let mut reader = Reader { tokens: tokenize(text)?, position: 0, declarations: Vec::new() };
    reader.body(None)?;
    Ok(reader.declarations)
}

/// Name resolution over read declarations: what a written name designates.
pub struct Scope<'d> {
    declarations: &'d [Declaration],
    /// (owner, name) → declaration.
    members: HashMap<(Option<usize>, &'d str), usize>,
}

impl<'d> Scope<'d> {
    pub fn new(declarations: &'d [Declaration]) -> Scope<'d> {
        let mut members = HashMap::new();
        for (index, declaration) in declarations.iter().enumerate() {
            if let Some(name) = Self::effective_name(declaration) {
                members.entry((declaration.owner, name)).or_insert(index);
            }
        }
        // A short name designates its element too (`[ms]`), after names.
        for (index, declaration) in declarations.iter().enumerate() {
            if let Some(short) = &declaration.short_name {
                members.entry((declaration.owner, short.as_str())).or_insert(index);
            }
        }
        Scope { declarations, members }
    }

    /// The name an element goes by: the one it declares, or the one it
    /// takes from the library feature it redefines.
    pub fn effective_name(declaration: &'d Declaration) -> Option<&'d str> {
        match (&declaration.name, declaration.membership) {
            (Some(name), _) => Some(name.as_str()),
            (None, "StateSubactionMembership") => Some(ENTRY_ACTION),
            // `:>> period = ...` is known as `period`, like what it redefines.
            (None, _) => declaration.redefines.first().and_then(|redefined| redefined.rsplit("::").next()),
        }
    }

    /// A member of `owner` named `name`: its own, or one it inherits from
    /// what it is typed by or specializes.
    fn member(&self, owner: usize, name: &str, depth: usize) -> Option<usize> {
        if let Some(found) = self.members.get(&(Some(owner), name)) {
            return Some(*found);
        }
        if depth > 16 {
            return None; // a specialization cycle is the pilot's to report
        }
        let declaration = &self.declarations[owner];
        declaration
            .typed_by
            .iter()
            .chain(&declaration.specializes)
            .filter_map(|general| self.resolve(general, declaration.owner))
            .find_map(|general| self.member(general, name, depth + 1))
    }

    /// The member named `name` that `owner` inherits from what it is typed
    /// by or specializes: what a feature of `owner` redefines.
    pub fn inherited(&self, owner: usize, name: &str) -> Option<usize> {
        let declaration = &self.declarations[owner];
        declaration
            .typed_by
            .iter()
            .chain(&declaration.specializes)
            .filter_map(|general| self.resolve(general, declaration.owner))
            .find_map(|general| self.member(general, name, 1))
    }

    /// The first segment of a path: looked up from `from` outwards, then in
    /// the packages of the model (the export imports them all).
    fn lexical(&self, name: &str, from: Option<usize>) -> Option<usize> {
        let mut scope = from;
        loop {
            let found = match scope {
                Some(owner) => self.member(owner, name, 0),
                None => self.members.get(&(None, name)).copied(),
            };
            if found.is_some() {
                return found;
            }
            match scope {
                Some(owner) => scope = self.declarations[owner].owner,
                None => break,
            }
        }
        self.declarations
            .iter()
            .enumerate()
            .filter(|(_, declaration)| declaration.metaclass == "Package")
            .find_map(|(package, _)| self.members.get(&(Some(package), name)).copied())
    }

    /// The declaration a written path designates, seen from inside `from`.
    /// `None` when it is not declared in the model (a library element).
    pub fn resolve(&self, path: &str, from: Option<usize>) -> Option<usize> {
        let mut segments = path.split("::").flat_map(|part| part.split('.'));
        let mut current = self.lexical(segments.next()?, from)?;
        for segment in segments {
            current = self.member(current, segment, 0)?;
        }
        Some(current)
    }

    /// `Package::Definition::feature`, with names quoted as the notation
    /// requires. `None` under an unnamed owner.
    pub fn qualified_name(&self, index: usize) -> Option<String> {
        let declaration = &self.declarations[index];
        let name = super::sysmlv2_generator::sysml_name(Self::effective_name(declaration)?);
        match declaration.owner {
            None => Some(name),
            Some(owner) => Some(format!("{}::{}", self.qualified_name(owner)?, name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_ok(text: &str) -> Vec<Declaration> {
        read(text).expect("the export reads")
    }

    fn summary(declarations: &[Declaration]) -> Vec<String> {
        declarations
            .iter()
            .map(|d| format!("{} {} via {}", d.metaclass, d.name.as_deref().unwrap_or("-"), d.membership))
            .collect()
    }

    #[test]
    fn definitions_and_usages_get_their_metaclass_and_membership() {
        let declarations = read_ok(
            "package M {\n  private import SI::*;\n  part def <'LC-1'> Controller :> Unit {\n    doc /* The controller */\n    attribute latency : DurationValue = 25 [ms];\n    port cmd : Out_Port;\n    perform action perform_Decide : Decide;\n  }\n  part p_LC_1 : Controller;\n  abstract part def Unit;\n}\n",
        );
        assert_eq!(
            summary(&declarations),
            [
                "Package M via OwningMembership",
                "PartDefinition Controller via OwningMembership",
                "Documentation - via OwningMembership",
                "AttributeUsage latency via FeatureMembership",
                "PortUsage cmd via FeatureMembership",
                "PerformActionUsage perform_Decide via FeatureMembership",
                "PartUsage p_LC_1 via OwningMembership",
                "PartDefinition Unit via OwningMembership",
            ]
        );
        let controller = &declarations[1];
        assert_eq!(controller.short_name.as_deref(), Some("LC-1"));
        assert_eq!(controller.specializes, ["Unit"]);
        assert_eq!(declarations[2].body.as_deref(), Some("The controller"));
        assert_eq!(declarations[3].typed_by, ["DurationValue"]);
        assert_eq!(declarations[3].expression.as_deref(), Some("25 [ms]"));
        assert!(declarations[7].is_abstract);
    }

    #[test]
    fn connectors_keep_their_ends() {
        let declarations = read_ok(
            "package M {\n  connect p_A.out to p_B.inp;  // note\n  allocate p_A to p_N;\n  flow from a_F to a_G;\n  connection Bus : CAN connect p_N to p_O {\n    attribute rate : Real = 2;\n  }\n  satisfy req_R by p_A;\n  dependency refines from A to B;\n}\n",
        );
        let ends: Vec<(&str, Option<(String, String)>)> = declarations[1..].iter().map(|d| (d.metaclass, d.ends.clone())).collect();
        let pair = |a: &str, b: &str| Some((a.to_string(), b.to_string()));
        assert_eq!(
            ends,
            [
                ("ConnectionUsage", pair("p_A.out", "p_B.inp")),
                ("AllocationUsage", pair("p_A", "p_N")),
                ("FlowUsage", pair("a_F", "a_G")),
                ("ConnectionUsage", pair("p_N", "p_O")),
                ("AttributeUsage", None),
                ("SatisfyRequirementUsage", pair("p_A", "req_R")),
                ("Dependency", pair("A", "B")),
            ]
        );
        assert_eq!(declarations[4].typed_by, ["CAN"]);
    }

    #[test]
    fn a_multiplicity_is_a_range_owned_by_its_usage() {
        let declarations = read_ok(
            "package M {\n  part a : D [4];\n  part b : D [0..1];\n  part c : D [2..*];\n  part d : D [*];\n  part e : D;\n}\n",
        );
        let ranges: Vec<(Option<usize>, Option<(Option<u64>, Option<u64>)>)> = declarations
            .iter()
            .filter(|d| d.metaclass == "MultiplicityRange")
            .map(|d| (d.owner, d.bounds))
            .collect();
        assert_eq!(
            ranges,
            [
                (Some(1), Some((None, Some(4)))),
                (Some(3), Some((Some(0), Some(1)))),
                (Some(5), Some((Some(2), None))),
                (Some(7), Some((None, None))),
            ]
        );
        assert_eq!(declarations[1].typed_by, ["D"]);
        assert!(read("package M {\n  part a : D [4;\n}\n").is_err());
    }

    #[test]
    fn then_declares_the_succession_before_the_step() {
        let declarations = read_ok(
            "package M {\n  action def Chain {\n    action step1 : A;\n    then action step2 : B;\n  }\n}\n",
        );
        assert_eq!(
            summary(&declarations[2..]),
            [
                "ActionUsage step1 via FeatureMembership",
                "SuccessionAsUsage - via FeatureMembership",
                "ActionUsage step2 via FeatureMembership",
            ]
        );
        assert_eq!(declarations[3].ends, Some(("step1".to_string(), "step2".to_string())));
    }

    #[test]
    fn a_constraint_keeps_its_expression_as_text() {
        let declarations = read_ok(
            "package M {\n  assert constraint <'C-1'> Margin {\n    doc /* margin */\n    (a_F.latency + a_G.latency) <= (a_C.budget * 0.8)\n  }\n}\n",
        );
        assert_eq!(declarations[1].metaclass, "AssertConstraintUsage");
        assert_eq!(declarations[1].expression.as_deref(), Some("(a_F.latency + a_G.latency) <= (a_C.budget * 0.8)"));
    }

    #[test]
    fn expressions_are_parsed_with_the_notation_precedence() {
        let declarations = read_ok(
            "package M {\n  attribute a : DurationValue = 25 [ms];\n  attribute b : Real = -3;\n  attribute c : String = \"x \\\"y\\\"\";\n  attribute d : Boolean = true;\n  attribute e default = 0.5;\n  assert constraint C {\n    (p.x + p.y * 2) <= DataFunctions::max(q.z, 4 [kg])\n  }\n}\n",
        );
        let reference = |name: &str| Expr::Reference(name.to_string());
        let chain = |base: &str, feature: &str| Expr::Chain(Box::new(reference(base)), feature.to_string());
        assert_eq!(declarations[1].value, Some(Expr::Operator("[", vec![Expr::Integer(25), reference("ms")])));
        assert_eq!(declarations[2].value, Some(Expr::Operator("-", vec![Expr::Integer(3)])));
        assert_eq!(declarations[3].value, Some(Expr::Text("x \"y\"".to_string())));
        assert_eq!(declarations[4].value, Some(Expr::Boolean(true)));
        assert_eq!((declarations[5].value.clone(), declarations[5].is_default), (Some(Expr::Rational(0.5)), true));
        assert_eq!(
            declarations[6].value,
            Some(Expr::Operator(
                "<=",
                vec![
                    Expr::Operator("+", vec![chain("p", "x"), Expr::Operator("*", vec![chain("p", "y"), Expr::Integer(2)])]),
                    Expr::Invocation(
                        "DataFunctions::max".to_string(),
                        vec![chain("q", "z"), Expr::Operator("[", vec![Expr::Integer(4), reference("kg")])]
                    ),
                ]
            ))
        );
    }

    #[test]
    fn a_redefining_feature_goes_by_the_name_it_redefines() {
        let declarations = read_ok(
            "package M {\n  action def Task {\n    attribute period : DurationValue;\n  }\n  action def Detect :> Task {\n    attribute :>> period = 33 [ms];\n  }\n  action a : Detect;\n}\n",
        );
        let scope = Scope::new(&declarations);
        let period = scope.resolve("a.period", Some(0)).unwrap();
        assert_eq!(scope.qualified_name(period).as_deref(), Some("M::Detect::period"));
        assert_eq!(declarations[period].name, None, "it declares no name of its own");
    }

    #[test]
    fn a_short_name_resolves_like_a_name() {
        let declarations = read_ok("package M {\n  package Units {\n    attribute <ms> millisecond : DurationUnit;\n  }\n}\n");
        let scope = Scope::new(&declarations);
        assert_eq!(scope.resolve("ms", Some(0)), Some(2));
    }

    #[test]
    fn names_resolve_through_owners_types_and_packages() {
        let declarations = read_ok(
            "package M {\n  part def Camera {\n    port lane : P;\n  }\n  part p_CAM : Camera;\n  connect p_CAM.lane to p_CAM.lane;\n  package Units {\n    attribute <ms> millisecond : DurationUnit;\n  }\n}\n",
        );
        let scope = Scope::new(&declarations);
        let lane = scope.resolve("p_CAM.lane", Some(0)).expect("the port resolves through the part's type");
        assert_eq!(scope.qualified_name(lane).as_deref(), Some("M::Camera::lane"));
        assert_eq!(scope.resolve("millisecond", Some(1)).map(|i| declarations[i].metaclass), Some("AttributeUsage"));
        assert_eq!(scope.resolve("DurationUnit", Some(0)), None, "library names are not declared in the model");
    }

    #[test]
    fn an_unknown_statement_is_an_error() {
        let error = read("package M {\n  viewpoint V;\n}\n").unwrap_err();
        assert!(error.contains("unknown statement `viewpoint V`"), "{error}");
        assert!(read("package M {").is_err());
    }
}
