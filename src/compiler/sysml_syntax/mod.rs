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
mod tokens;

pub use expression::Expr;
use expression::Expressions;
use tokens::{expression_text, is_symbol, is_word, tokenize, Token};

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
    /// Packages whose members this namespace imports (`import P::*;`), as written.
    pub imports: Vec<String>,
}


/// Keywords that open a declaration, with the metaclass of the usage and of
/// the definition (`part` / `part def`).
const KINDS: &[(&[&str], &str, Option<&str>)] = &[
    (&["library", "package"], "LibraryPackage", None),
    (&["package"], "Package", None),
    (&["metadata"], "MetadataUsage", Some("MetadataDefinition")),
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

/// Control node keywords and their metaclasses.
const CONTROL_NODES: &[(&str, &str)] = &[("fork", "ForkNode"), ("join", "JoinNode"), ("decide", "DecisionNode"), ("merge", "MergeNode")];

/// Name under which a state definition's entry action is known: it
/// redefines the library feature of that name.
pub const ENTRY_ACTION: &str = "entryAction";

struct Reader {
    tokens: Vec<Token>,
    position: usize,
    declarations: Vec<Declaration>,
    /// Imports written outside any package.
    top_imports: Vec<String>,
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
            // Not a declaration: what names resolve to inside the namespace.
            let mut segments: Vec<String> = Vec::new();
            loop {
                if head.take_symbol("*") {
                    break;
                }
                segments.push(head.name().ok_or_else(|| head.unexpected("an imported name"))?);
                if !head.take_symbol("::") {
                    break;
                }
            }
            let path = segments.join("::");
            match owner {
                Some(owner) => self.declarations[owner].imports.push(path),
                None => self.top_imports.push(path),
            }
            return Ok(None);
        }
        // `#OA package ...`: prefix metadata, usages of metadata definitions
        // owned by the declaration they annotate.
        let mut metadata: Vec<String> = Vec::new();
        while head.take_symbol("#") {
            metadata.push(head.path()?);
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
            if !is_word(head.peek(), "from") && !is_word(head.peek(), "of") {
                Self::short_and_name(&mut head, &mut declaration)?;
            }
            // `of T`: what flows, the flow's payload feature.
            let payload = if head.take_word("of") { Some(head.path()?) } else { None };
            Self::expect_word(&mut head, "from")?;
            declaration.ends = Some(Self::ends(&mut head, "to")?);
            if let Some(payload) = payload {
                self.finish(&head)?;
                let flow = self.push(declaration, owner);
                let feature = Declaration { metaclass: "PayloadFeature", membership: "FeatureMembership", typed_by: vec![payload], ..Declaration::default() };
                self.push(feature, Some(flow));
                self.annotate(flow, metadata);
                return Ok(Some(flow));
            }
        } else if head.take_word("message") {
            // A message is a flow between two events, with no connector ends.
            declaration.metaclass = "FlowUsage";
            declaration.is_abstract = true;
            Self::short_and_name(&mut head, &mut declaration)?;
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
        } else if let Some(node) = CONTROL_NODES.iter().find(|(word, _)| is_word(head.peek(), word)).map(|(_, node)| *node) {
            // `fork <'id'> 'AND 1';`: a control node of an action.
            head.position += 1;
            declaration.metaclass = node;
            declaration.membership = "FeatureMembership";
            self.named(&mut head, &mut declaration)?;
        } else if head.take_word("succession") {
            // `succession <'id'> first a then b`: a sequence link of a chain.
            declaration.metaclass = "SuccessionAsUsage";
            if !is_word(head.peek(), "first") {
                Self::short_and_name(&mut head, &mut declaration)?;
            }
            Self::expect_word(&mut head, "first")?;
            declaration.ends = Some(Self::ends(&mut head, "then")?);
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
            declaration.metaclass = "PerformActionUsage";
            if head.take_word("action") {
                self.named(&mut head, &mut declaration)?;
            } else {
                // `perform a;`: performs the action `a` declared elsewhere.
                declaration.references = vec![head.path()?];
            }
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
        } else if follows && owner.is_some_and(|o| matches!(self.declarations[o].metaclass, "StateDefinition" | "StateUsage")) {
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
        // Prefix metadata first, as the pilot owns them, then the multiplicity.
        let bounds = declaration.bounds.take();
        let declared = self.push(declaration, owner);
        self.annotate(declared, metadata);
        if bounds.is_some() {
            let range = Declaration { metaclass: "MultiplicityRange", membership: "OwningMembership", bounds, ..Declaration::default() };
            self.push(range, Some(declared));
        }
        Ok(Some(declared))
    }

    /// The prefix metadata of a declaration, as metadata usages it owns.
    fn annotate(&mut self, declared: usize, metadata: Vec<String>) {
        // A relationship is no namespace: its metadata annotate it.
        let membership = if self.declarations[declared].metaclass == "Dependency" { "Annotation" } else { "OwningMembership" };
        for definition in metadata {
            let usage = Declaration { metaclass: "MetadataUsage", membership, typed_by: vec![definition], ..Declaration::default() };
            self.push(usage, Some(declared));
        }
    }

    /// `<'short'> name` of a connector written with its own keyword.
    fn short_and_name(head: &mut Head, declaration: &mut Declaration) -> Result<(), String> {
        if head.take_symbol("<") {
            declaration.short_name = Some(head.name().ok_or_else(|| head.unexpected("a short name"))?);
            if !head.take_symbol(">") {
                return Err(head.unexpected("`>`"));
            }
        }
        if !["from", "of", "first"].iter().any(|w| is_word(head.peek(), w)) {
            declaration.name = head.name();
        }
        Ok(())
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
    let mut reader = Reader { tokens: tokenize(text)?, position: 0, declarations: Vec::new(), top_imports: Vec::new() };
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
    /// the library packages of the model (the export imports its profile);
    /// any other package is named by a qualified name.
    fn lexical(&self, name: &str, from: Option<usize>) -> Option<usize> {
        let mut scope = from;
        loop {
            let found = match scope {
                Some(owner) => self.member(owner, name, 0).or_else(|| self.imported(owner, name)),
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
            .filter(|(_, declaration)| declaration.metaclass == "LibraryPackage")
            .find_map(|(package, _)| self.members.get(&(Some(package), name)).copied())
    }

    /// The declarations a feature chain `A.b.c` (or `P::A.b.c`) goes
    /// through: what its first part names, then each feature in turn.
    pub fn chain(&self, path: &str, from: Option<usize>) -> Option<Vec<usize>> {
        let mut parts = path.split('.');
        let mut current = self.resolve(parts.next()?, from)?;
        let mut chain = vec![current];
        for part in parts {
            current = self.member(current, part, 0)?;
            chain.push(current);
        }
        Some(chain)
    }

    /// A member of a package `owner` imports with `import P::*;`.
    fn imported(&self, owner: usize, name: &str) -> Option<usize> {
        let declaration = &self.declarations[owner];
        declaration.imports.iter().find_map(|path| {
            // An imported package is named from inside the importing namespace.
            let package = self.resolve_package(path, Some(owner))?;
            self.members.get(&(Some(package), name)).copied()
        })
    }

    /// The package a path names, looked up outwards without imports.
    fn resolve_package(&self, path: &str, from: Option<usize>) -> Option<usize> {
        let mut segments = path.split("::");
        let first = segments.next()?;
        let mut scope = from;
        let mut current = loop {
            let found = match scope {
                Some(owner) => self.members.get(&(Some(owner), first)).copied(),
                None => self.members.get(&(None, first)).copied(),
            };
            if let Some(found) = found {
                break found;
            }
            scope = self.declarations[scope?].owner;
        };
        for segment in segments {
            current = *self.members.get(&(Some(current), segment))?;
        }
        Some(current)
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
mod tests;
