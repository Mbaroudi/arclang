//! Recursive-descent parser of the SysML v2 textual notation, following
//! SysML.xtext (with KerMLExpressions.xtext for expressions): a text to a
//! tree of `Node`s. Names stay as written; nothing is resolved here.

mod behavior;
mod expr;

use super::lexer::{tokenize, Kind, Token};
use super::syntax::*;

/// Words that cannot be bare names.
pub const RESERVED: &[&str] = &[
    "about", "abstract", "accept", "action", "actor", "after", "alias", "all", "allocate", "allocation", "analysis", "and", "as", "assert", "assign",
    "assume", "at", "attribute", "bind", "binding", "by", "calc", "case", "comment", "concern", "connect", "connection", "constant", "constraint",
    "crosses", "decide", "def", "default", "defined", "dependency", "derived", "do", "doc", "else", "end", "entry", "enum", "event", "exhibit", "exit",
    "expose", "false", "filter", "first", "flow", "for", "fork", "frame", "from", "hastype", "if", "implies", "import", "in", "include", "individual",
    "inout", "interface", "istype", "item", "join", "language", "library", "locale", "loop", "merge", "message", "meta", "metadata", "new", "nonunique",
    "not", "null", "objective", "occurrence", "of", "or", "ordered", "out", "package", "parallel", "part", "perform", "port", "private", "protected",
    "public", "redefines", "references", "render", "rendering", "rep", "require", "requirement", "return", "satisfy", "send", "snapshot", "specializes",
    "stakeholder", "standard", "state", "subject", "subsets", "succession", "terminate", "then", "timeslice", "to", "transition", "true", "until", "use",
    "variant", "variation", "verification", "verify", "via", "view", "viewpoint", "when", "while", "xor",
];

pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) position: usize,
    /// Whether the body being read is a state body: `then x;` is a transition there.
    pub(super) in_state: bool,
    /// Whether the body may end with a result expression (calculations, constraints, cases).
    pub(super) in_calculation: bool,
}

pub type Parsed<T> = Result<T, String>;

/// Parse a whole text: the members of the root namespace.
pub fn parse(text: &str) -> Parsed<Vec<Node>> {
    let tokens = tokenize(text)?;
    let mut parser = Parser { tokens, position: 0, in_state: false, in_calculation: false };
    let mut members = Vec::new();
    while parser.peek().is_some() {
        members.push(parser.statement()?);
    }
    Ok(members)
}

impl Parser {
    // --- cursor -----------------------------------------------------------

    pub(super) fn peek(&self) -> Option<&Kind> {
        self.tokens.get(self.position).map(|t| &t.kind)
    }

    pub(super) fn peek_at(&self, ahead: usize) -> Option<&Kind> {
        self.tokens.get(self.position + ahead).map(|t| &t.kind)
    }

    pub(super) fn line(&self) -> usize {
        self.tokens.get(self.position).or_else(|| self.tokens.last()).map(|t| t.line).unwrap_or(0)
    }

    pub(super) fn advance(&mut self) -> Option<Kind> {
        let token = self.tokens.get(self.position).map(|t| t.kind.clone());
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    pub(super) fn error(&self, what: &str) -> String {
        let found = match self.peek() {
            Some(Kind::Word(w)) => format!("`{w}`"),
            Some(Kind::Name(n)) => format!("'{n}'"),
            Some(Kind::Str(s)) => format!("\"{s}\""),
            Some(Kind::Integer(n)) | Some(Kind::Real(n)) => n.clone(),
            Some(Kind::Comment(_)) => "a comment".to_string(),
            Some(Kind::Symbol(s)) => format!("`{s}`"),
            None => "the end of the text".to_string(),
        };
        format!("line {}: expected {what}, found {found}", self.line())
    }

    pub(super) fn is_word(&self, word: &str) -> bool {
        matches!(self.peek(), Some(Kind::Word(w)) if w == word)
    }

    pub(super) fn is_word_at(&self, ahead: usize, word: &str) -> bool {
        matches!(self.peek_at(ahead), Some(Kind::Word(w)) if w == word)
    }

    pub(super) fn is_symbol(&self, symbol: &str) -> bool {
        matches!(self.peek(), Some(Kind::Symbol(s)) if *s == symbol)
    }

    pub(super) fn is_symbol_at(&self, ahead: usize, symbol: &str) -> bool {
        matches!(self.peek_at(ahead), Some(Kind::Symbol(s)) if *s == symbol)
    }

    pub(super) fn take_word(&mut self, word: &str) -> bool {
        let found = self.is_word(word);
        if found {
            self.position += 1;
        }
        found
    }

    pub(super) fn take_symbol(&mut self, symbol: &str) -> bool {
        let found = self.is_symbol(symbol);
        if found {
            self.position += 1;
        }
        found
    }

    pub(super) fn expect_word(&mut self, word: &str) -> Parsed<()> {
        if self.take_word(word) { Ok(()) } else { Err(self.error(&format!("`{word}`"))) }
    }

    pub(super) fn expect_symbol(&mut self, symbol: &str) -> Parsed<()> {
        if self.take_symbol(symbol) { Ok(()) } else { Err(self.error(&format!("`{symbol}`"))) }
    }

    /// Whether the token at `ahead` can start a name (an unreserved word or a quoted name).
    pub(super) fn is_name_at(&self, ahead: usize) -> bool {
        match self.peek_at(ahead) {
            Some(Kind::Word(w)) => !RESERVED.contains(&w.as_str()),
            Some(Kind::Name(_)) => true,
            _ => false,
        }
    }

    pub(super) fn is_name(&self) -> bool {
        self.is_name_at(0)
    }

    pub(super) fn name(&mut self) -> Parsed<String> {
        match self.peek() {
            Some(Kind::Word(w)) if !RESERVED.contains(&w.as_str()) => {
                let w = w.clone();
                self.position += 1;
                Ok(w)
            }
            Some(Kind::Name(n)) => {
                let n = n.clone();
                self.position += 1;
                Ok(n)
            }
            _ => Err(self.error("a name")),
        }
    }

    pub(super) fn string(&mut self) -> Parsed<String> {
        match self.peek() {
            Some(Kind::Str(s)) => {
                let s = s.clone();
                self.position += 1;
                Ok(s)
            }
            _ => Err(self.error("a string")),
        }
    }

    /// `$::`? (Name `::`)* Name
    pub(super) fn qualified_name(&mut self) -> Parsed<QualifiedName> {
        let global = self.is_symbol("$") && self.is_symbol_at(1, "::");
        if global {
            self.position += 2;
        }
        let mut segments = vec![self.name()?];
        while self.is_symbol("::") && self.is_name_at(1) {
            self.position += 1;
            segments.push(self.name()?);
        }
        Ok(QualifiedName { global, segments })
    }

    /// A qualified name, or a feature chain of them (`a.b::c.d`).
    pub(super) fn target(&mut self) -> Parsed<Target> {
        let mut steps = vec![self.qualified_name()?];
        while self.is_symbol(".") && self.is_name_at(1) {
            self.position += 1;
            steps.push(self.qualified_name()?);
        }
        Ok(if steps.len() == 1 { Target::Name(steps.pop().unwrap()) } else { Target::Chain(steps) })
    }

    // --- statements -------------------------------------------------------

    /// One element of a body: a member, an import, an alias, a filter, an
    /// annotation, a relationship, a succession.
    pub(super) fn statement(&mut self) -> Parsed<Node> {
        let line = self.line();
        let visibility = self.visibility();
        let mut node = self.statement_after_visibility()?;
        if node.prefixes.visibility.is_none() {
            node.prefixes.visibility = visibility;
        }
        if node.line == 0 {
            node.line = line;
        }
        Ok(node)
    }

    fn visibility(&mut self) -> Option<&'static str> {
        for v in ["public", "private", "protected"] {
            if self.take_word(v) {
                return Some(v);
            }
        }
        None
    }

    fn statement_after_visibility(&mut self) -> Parsed<Node> {
        match self.peek() {
            Some(Kind::Comment(_)) => return self.comment(),
            Some(Kind::Word(w)) => match w.as_str() {
                "import" => return self.import("Import", false),
                "expose" => return self.import("Expose", true),
                "alias" => return self.alias(),
                "filter" => {
                    self.position += 1;
                    let expr = self.expression()?;
                    self.expect_symbol(";")?;
                    let mut node = Node::new("ElementFilter", "ElementFilterMembership");
                    node.detail = Detail::Filter(expr);
                    return Ok(node);
                }
                "comment" | "doc" | "rep" | "language" | "locale" => return self.comment(),
                "dependency" => return self.dependency(Vec::new()),
                "variant" => {
                    self.position += 1;
                    let mut node = self.usage_statement()?;
                    node.membership = "VariantMembership";
                    return Ok(node);
                }
                "then" => return self.then_statement(),
                "else" if self.in_state || !self.in_calculation => return self.else_statement(),
                "first" if !self.is_declaration_start(1) && !self.then_follows() => return self.initial_node(),
                "return" => {
                    self.position += 1;
                    let mut node = self.usage_statement()?;
                    node.membership = "ReturnParameterMembership";
                    return Ok(node);
                }
                "objective" => return self.objective(),
                "subject" => return self.framed("ReferenceUsage", "SubjectMembership", None),
                "actor" => return self.framed("PartUsage", "ActorMembership", None),
                "stakeholder" => return self.framed("PartUsage", "StakeholderMembership", None),
                "assume" | "require" => return self.requirement_constraint(),
                "frame" => return self.concern_or_verification("frame"),
                "verify" => return self.concern_or_verification("verify"),
                "render" => return self.rendering(),
                "accept" if self.in_state => return self.target_transition(),
                "do" if self.in_state && self.then_follows() => return self.target_transition(),
                "entry" | "do" | "exit" if self.in_state => return self.state_subaction(),
                "metadata" if !self.is_word_at(1, "def") => return self.metadata_usage(Vec::new()),
                _ => {}
            },
            Some(Kind::Symbol("@")) => return self.metadata_usage(Vec::new()),
            Some(Kind::Symbol("#")) => {
                let metadata = self.prefix_metadata()?;
                if self.is_word("dependency") {
                    return self.dependency(metadata);
                }
                if self.is_symbol("@") || (self.is_word("metadata") && !self.is_word_at(1, "def")) {
                    return self.metadata_usage(metadata);
                }
                if self.is_word("package") || self.is_word("library") || self.is_word("standard") {
                    let mut node = self.package()?;
                    node.metadata = metadata;
                    return Ok(node);
                }
                return self.definition_or_usage(metadata);
            }
            _ => {}
        }
        if self.is_word("package") || self.is_word("library") || self.is_word("standard") {
            return self.package();
        }
        if self.in_calculation && self.is_result_expression() {
            let expr = self.expression()?;
            let mut node = Node::new("Expression", "ResultExpressionMembership");
            node.detail = Detail::Result(expr);
            return Ok(node);
        }
        self.definition_or_usage(Vec::new())
    }

    /// After `name [`, whether a usage keyword follows the closing `]`.
    fn usage_keyword_follows_multiplicity(&self) -> bool {
        let mut i = self.position + 1;
        let mut depth = 0i32;
        while let Some(token) = self.tokens.get(i) {
            match &token.kind {
                Kind::Symbol("[") => depth += 1,
                Kind::Symbol("]") => {
                    depth -= 1;
                    if depth == 0 {
                        return matches!(self.tokens.get(i + 1).map(|t| &t.kind), Some(Kind::Word(w)) if RESERVED.contains(&w.as_str()) && !matches!(w.as_str(), "ordered" | "nonunique" | "defined" | "subsets" | "redefines" | "references" | "crosses" | "default"));
                    }
                }
                Kind::Symbol(";") | Kind::Symbol("{") => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// Whether a `then` ends this statement's head: a target transition.
    pub(super) fn then_follows(&self) -> bool {
        let mut depth = 0i32;
        let mut i = self.position;
        while let Some(token) = self.tokens.get(i) {
            match &token.kind {
                Kind::Word(w) if w == "then" && depth == 0 => return true,
                Kind::Symbol("(") | Kind::Symbol("[") => depth += 1,
                Kind::Symbol(")") | Kind::Symbol("]") => depth -= 1,
                Kind::Symbol(";") | Kind::Symbol("{") | Kind::Symbol("}") if depth == 0 => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// Whether what follows at `ahead` opens a declaration rather than a reference.
    fn is_declaration_start(&self, ahead: usize) -> bool {
        match self.peek_at(ahead) {
            Some(Kind::Word(w)) => RESERVED.contains(&w.as_str()) && !matches!(w.as_str(), "then" | "to" | "else" | "if"),
            Some(Kind::Symbol(s)) => matches!(*s, "#" | "@" | "<" | ":" | ":>" | "::>" | ":>>" | "=>" | "["),
            _ => false,
        }
    }

    /// In a calculation body, a statement that is an expression ends the
    /// body: it has no `;` before the closing brace, and does not open a
    /// declaration.
    fn is_result_expression(&self) -> bool {
        let mut depth: i32 = 0;
        let mut i = self.position;
        // A declaration starts with a name followed by a specialization,
        // a multiplicity or a value; anything else starting a statement
        // and reaching `}` without `;` is an expression.
        if self.is_name() {
            if let Some(Kind::Symbol(s)) = self.peek_at(1) {
                if matches!(*s, ":" | ":>" | "::>" | ":>>" | "=>" | "[" | "=" | ":=" | ";" | "{") {
                    return false;
                }
            }
            if matches!(self.peek_at(1), Some(Kind::Word(w)) if w == "default" || w == "ordered" || w == "nonunique" || w == "defined" || w == "subsets" || w == "redefines" || w == "references") {
                return false;
            }
        } else if !matches!(self.peek(), Some(Kind::Symbol("(")) | Some(Kind::Symbol("-")) | Some(Kind::Symbol("+")) | Some(Kind::Symbol("~")) | Some(Kind::Symbol("{")) | Some(Kind::Integer(_)) | Some(Kind::Real(_)) | Some(Kind::Str(_)))
            && !self.is_word("not")
            && !self.is_word("true")
            && !self.is_word("false")
            && !self.is_word("null")
            && !self.is_word("new")
            && !self.is_word("if")
            && !self.is_word("all")
            && !(self.is_symbol("$") && self.is_symbol_at(1, "::"))
        {
            return false;
        }
        while let Some(token) = self.tokens.get(i) {
            match &token.kind {
                Kind::Symbol("{") | Kind::Symbol("(") | Kind::Symbol("[") => depth += 1,
                Kind::Symbol("}") | Kind::Symbol(")") | Kind::Symbol("]") => {
                    if depth == 0 {
                        return true;
                    }
                    depth -= 1;
                }
                Kind::Symbol(";") if depth == 0 => return false,
                _ => {}
            }
            i += 1;
        }
        true
    }

    // --- packages, imports, aliases, annotations ---------------------------

    fn package(&mut self) -> Parsed<Node> {
        let line = self.line();
        let standard = self.take_word("standard");
        let library = self.take_word("library");
        let metadata = if self.is_symbol("#") { self.prefix_metadata()? } else { Vec::new() };
        self.expect_word("package")?;
        let mut node = Node::new(if library { "LibraryPackage" } else { "Package" }, "OwningMembership");
        node.prefixes.is_standard = standard;
        node.metadata = metadata;
        node.line = line;
        self.identification(&mut node)?;
        node.members = self.body()?;
        Ok(node)
    }

    /// `<short> name?` | `name`, when present.
    pub(super) fn identification(&mut self, node: &mut Node) -> Parsed<bool> {
        if self.is_symbol("<") {
            self.position += 1;
            node.short_name = Some(self.name()?);
            self.expect_symbol(">")?;
            if self.is_name() {
                node.name = Some(self.name()?);
            }
            return Ok(true);
        }
        if self.is_name() {
            node.name = Some(self.name()?);
            return Ok(true);
        }
        Ok(false)
    }

    /// `;` or `{ statements }`.
    pub(super) fn body(&mut self) -> Parsed<Vec<Node>> {
        if self.take_symbol(";") {
            return Ok(Vec::new());
        }
        self.expect_symbol("{")?;
        let mut members = Vec::new();
        while !self.is_symbol("}") {
            if self.peek().is_none() {
                return Err(self.error("`}`"));
            }
            members.push(self.statement()?);
        }
        self.position += 1;
        Ok(members)
    }

    /// A body read in a context: a state body, a calculation body.
    pub(super) fn body_in(&mut self, in_state: bool, in_calculation: bool) -> Parsed<Vec<Node>> {
        let saved = (self.in_state, self.in_calculation);
        self.in_state = in_state;
        self.in_calculation = in_calculation;
        let result = self.body();
        self.in_state = saved.0;
        self.in_calculation = saved.1;
        result
    }

    /// `;` or `{ annotations }` (RelationshipBody).
    fn relationship_body(&mut self) -> Parsed<Vec<Node>> {
        self.body()
    }

    fn import(&mut self, keyword_kind: &'static str, expose: bool) -> Parsed<Node> {
        self.position += 1; // import | expose
        let all = self.take_word("all");
        let target = self.qualified_name()?;
        let mut namespace = false;
        let mut recursive = false;
        if self.is_symbol("::") && self.is_symbol_at(1, "*") {
            self.position += 2;
            namespace = true;
        }
        if self.is_symbol("::") && self.is_symbol_at(1, "**") {
            self.position += 2;
            recursive = true;
        }
        // A filter package: `import P::*[@Meta][x > 1];`
        let mut filters = Vec::new();
        while self.is_symbol("[") {
            self.position += 1;
            filters.push(self.expression()?);
            self.expect_symbol("]")?;
        }
        let kind: &'static str = match (expose, namespace) {
            (false, true) => "NamespaceImport",
            (false, false) => "MembershipImport",
            (true, true) => "NamespaceExpose",
            (true, false) => "MembershipExpose",
        };
        let _ = keyword_kind;
        let mut node = Node::new(kind, "Import");
        node.detail = Detail::Import { target, namespace, recursive, all, filters };
        node.members = self.relationship_body()?;
        Ok(node)
    }

    fn alias(&mut self) -> Parsed<Node> {
        self.position += 1;
        let mut node = Node::new("Alias", "Membership");
        if self.is_symbol("<") {
            self.position += 1;
            node.short_name = Some(self.name()?);
            self.expect_symbol(">")?;
        }
        if self.is_name() {
            node.name = Some(self.name()?);
        }
        self.expect_word("for")?;
        let target = self.qualified_name()?;
        node.detail = Detail::Alias { target };
        node.members = self.relationship_body()?;
        Ok(node)
    }

    /// `comment [id] [about a, b] [locale "x"] /* body */`, `doc [id] [locale] /* */`,
    /// `rep [id] language "x" /* */`, `language "x" /* */`, a bare `/* */`.
    fn comment(&mut self) -> Parsed<Node> {
        let mut node = Node::new("Comment", "OwningMembership");
        let mut about = Vec::new();
        let mut locale = None;
        let mut language = None;
        if self.take_word("comment") {
            self.identification(&mut node)?;
            if self.take_word("about") {
                about.push(self.qualified_name()?);
                while self.take_symbol(",") {
                    about.push(self.qualified_name()?);
                }
            }
        } else if self.take_word("doc") {
            node.kind = "Documentation";
            self.identification(&mut node)?;
        } else if self.take_word("rep") {
            node.kind = "TextualRepresentation";
            self.identification(&mut node)?;
        }
        if self.take_word("locale") {
            locale = Some(self.string()?);
        }
        if self.take_word("language") {
            node.kind = "TextualRepresentation";
            language = Some(self.string()?);
        }
        let body = match self.advance() {
            Some(Kind::Comment(body)) => body.trim().to_string(),
            _ => {
                self.position -= 1;
                return Err(self.error("a comment body `/* ... */`"));
            }
        };
        node.detail = Detail::Annotation { about, locale, language, body };
        Ok(node)
    }

    fn dependency(&mut self, metadata: Vec<Node>) -> Parsed<Node> {
        self.expect_word("dependency")?;
        let mut node = Node::new("Dependency", "OwningMembership");
        node.metadata = metadata;
        // `dependency <id> name from a to b` or `dependency a to b`: a name
        // followed by `from` is the identification.
        if self.is_symbol("<") || (self.is_name() && (self.is_word_at(1, "from") || (self.is_name_at(1) && self.is_word_at(2, "from")))) {
            self.identification(&mut node)?;
            self.expect_word("from")?;
        } else {
            self.take_word("from");
        }
        let mut clients = vec![self.qualified_name()?];
        while self.take_symbol(",") {
            clients.push(self.qualified_name()?);
        }
        self.expect_word("to")?;
        let mut suppliers = vec![self.qualified_name()?];
        while self.take_symbol(",") {
            suppliers.push(self.qualified_name()?);
        }
        node.detail = Detail::Dependency { clients, suppliers };
        node.members = self.relationship_body()?;
        Ok(node)
    }

    /// `# Name` prefixes, each a metadata usage typed by the name.
    pub(super) fn prefix_metadata(&mut self) -> Parsed<Vec<Node>> {
        let mut out = Vec::new();
        while self.take_symbol("#") {
            let ty = self.qualified_name()?;
            let mut node = Node::new("MetadataUsage", "OwningMembership");
            node.specializations.push(Specialization { kind: SpecKind::Typing, target: Target::Name(ty), conjugated: false });
            out.push(node);
        }
        Ok(out)
    }

    // --- prefixes ---------------------------------------------------------

    /// The prefix keywords of a definition or usage, and its `#Meta` extensions.
    fn prefixes(&mut self, node: &mut Node) -> Parsed<()> {
        loop {
            match self.peek() {
                Some(Kind::Word(w)) => match w.as_str() {
                    "in" | "out" | "inout" => {
                        node.prefixes.direction = Some(match w.as_str() {
                            "in" => "in",
                            "out" => "out",
                            _ => "inout",
                        });
                    }
                    "derived" => node.prefixes.is_derived = true,
                    "abstract" => node.prefixes.is_abstract = true,
                    "variation" => node.prefixes.is_variation = true,
                    "constant" => node.prefixes.is_constant = true,
                    "ref" => node.prefixes.is_reference = true,
                    "end" => {
                        node.prefixes.is_end = true;
                        self.position += 1;
                        // `end [0..1] item x`, `end inCart [0..1] item x`: a crossing
                        // feature, with a name when a usage keyword follows its multiplicity.
                        if self.is_symbol("[") || (self.is_name() && self.is_symbol_at(1, "[") && self.usage_keyword_follows_multiplicity()) {
                            let mut cross = Node::new("ReferenceUsage", "OwningMembership");
                            cross.membership_kind = Some("cross");
                            if self.is_name() {
                                cross.name = Some(self.name()?);
                            }
                            self.feature_specialization_part(&mut cross)?;
                            node.members.push(cross);
                        }
                        continue;
                    }
                    "individual" => node.prefixes.is_individual = true,
                    "snapshot" => node.prefixes.portion = Some("snapshot"),
                    "timeslice" => node.prefixes.portion = Some("timeslice"),
                    _ => return Ok(()),
                },
                Some(Kind::Symbol("#")) => {
                    let metadata = self.prefix_metadata()?;
                    node.metadata.extend(metadata);
                    continue;
                }
                _ => return Ok(()),
            }
            self.position += 1;
        }
    }

    // --- definitions and usages -------------------------------------------

    /// A definition or usage statement, after its visibility and leading `#Meta`.
    fn definition_or_usage(&mut self, metadata: Vec<Node>) -> Parsed<Node> {
        let line = self.line();
        let mut node = Node::new("Usage", "FeatureMembership");
        node.metadata = metadata;
        node.line = line;
        self.prefixes(&mut node)?;
        // The kind keyword(s).
        let Some(Kind::Word(word)) = self.peek().cloned() else {
            // No keyword: a default reference usage (`x : T;`, `:>> x = 1;`).
            node.kind = "ReferenceUsage";
            return self.finish_usage(node);
        };
        // `not satisfy ...`
        if word == "not" && self.is_word_at(1, "satisfy") {
            self.position += 1;
            node.prefixes.is_negated = true;
            return self.satisfy(node);
        }
        // `redefines x = 1;`, `subsets y;`: a reference usage declared by its specialization.
        if matches!(word.as_str(), "redefines" | "subsets" | "references" | "crosses" | "defined") {
            node.kind = "ReferenceUsage";
            return self.finish_usage(node);
        }
        let is_def = self.is_word_at(1, "def") || (word == "use" && self.is_word_at(1, "case") && self.is_word_at(2, "def"));
        let def_kind: Option<&'static str> = match word.as_str() {
            "attribute" => Some("AttributeDefinition"),
            "enum" => Some("EnumerationDefinition"),
            "occurrence" => Some("OccurrenceDefinition"),
            "item" => Some("ItemDefinition"),
            "metadata" => Some("MetadataDefinition"),
            "part" => Some("PartDefinition"),
            "connection" => Some("ConnectionDefinition"),
            "flow" => Some("FlowDefinition"),
            "interface" => Some("InterfaceDefinition"),
            "allocation" => Some("AllocationDefinition"),
            "port" => Some("PortDefinition"),
            "action" => Some("ActionDefinition"),
            "calc" => Some("CalculationDefinition"),
            "state" => Some("StateDefinition"),
            "constraint" => Some("ConstraintDefinition"),
            "requirement" => Some("RequirementDefinition"),
            "concern" => Some("ConcernDefinition"),
            "case" => Some("CaseDefinition"),
            "analysis" => Some("AnalysisCaseDefinition"),
            "verification" => Some("VerificationCaseDefinition"),
            "use" => Some("UseCaseDefinition"),
            "view" => Some("ViewDefinition"),
            "viewpoint" => Some("ViewpointDefinition"),
            "rendering" => Some("RenderingDefinition"),
            "def" => Some("Definition"), // `#meta def X` or `individual def X`
            _ => None,
        };
        if is_def || word == "def" {
            let kind = def_kind.ok_or_else(|| self.error("a definition keyword"))?;
            if word == "use" {
                self.position += 2;
            } else if word != "def" {
                self.position += 1;
            }
            self.expect_word("def")?;
            if kind == "Definition" && node.prefixes.is_individual {
                node.kind = "OccurrenceDefinition";
            } else {
                node.kind = kind;
            }
            node.membership = "OwningMembership";
            return self.finish_definition(node);
        }
        let usage_kind: Option<&'static str> = match word.as_str() {
            "attribute" => Some("AttributeUsage"),
            "enum" => Some("EnumerationUsage"),
            "occurrence" => Some("OccurrenceUsage"),
            "item" => Some("ItemUsage"),
            "part" => Some("PartUsage"),
            "port" => Some("PortUsage"),
            "view" => Some("ViewUsage"),
            "rendering" => Some("RenderingUsage"),
            "viewpoint" => Some("ViewpointUsage"),
            "action" => Some("ActionUsage"),
            "calc" => Some("CalculationUsage"),
            "state" => Some("StateUsage"),
            "constraint" => Some("ConstraintUsage"),
            "requirement" => Some("RequirementUsage"),
            "concern" => Some("ConcernUsage"),
            "case" => Some("CaseUsage"),
            "analysis" => Some("AnalysisCaseUsage"),
            "verification" => Some("VerificationCaseUsage"),
            "use" => Some("UseCaseUsage"),
            "event" => Some("EventOccurrenceUsage"),
            _ => None,
        };
        if let Some(kind) = usage_kind {
            // `action name send ...`: an action node named before its keyword.
            if kind == "ActionUsage" {
                for keyword in ["accept", "send", "assign", "terminate", "if", "while", "loop", "for", "merge", "decide", "join", "fork"] {
                    if self.word_follows_declaration(keyword) {
                        return self.named_action_node(node, keyword);
                    }
                }
            }
            if word == "use" {
                self.expect_word("use")?;
                self.expect_word("case")?;
            } else {
                self.position += 1;
            }
            node.kind = kind;
            if kind == "EventOccurrenceUsage" {
                return self.event_occurrence(node);
            }
            return self.finish_usage(node);
        }
        match word.as_str() {
            "connection" | "connect" => self.connection(node, "ConnectionUsage", "connection", "connect"),
            "interface" => self.interface(node),
            "allocation" | "allocate" => self.connection(node, "AllocationUsage", "allocation", "allocate"),
            "binding" | "bind" => self.binding(node),
            "succession" if self.is_word_at(1, "flow") => {
                self.position += 1;
                self.flow(node, "SuccessionFlowUsage")
            }
            "succession" | "first" => self.succession(node),
            "flow" => self.flow(node, "FlowUsage"),
            "message" => self.message(node),
            "perform" => self.performance(node, "perform", "PerformActionUsage", "action"),
            "exhibit" => self.performance(node, "exhibit", "ExhibitStateUsage", "state"),
            "include" => self.performance(node, "include", "IncludeUseCaseUsage", "use"),
            "assert" => self.assertion(node),
            "satisfy" => self.satisfy(node),
            "transition" => self.transition(node),
            "accept" => self.accept_node(node),
            "send" => self.send_node(node),
            "assign" => self.assign_node(node),
            "if" => self.if_node(node),
            "while" | "loop" => self.while_node(node),
            "for" => self.for_node(node),
            "terminate" => self.terminate_node(node),
            "merge" | "decide" | "join" | "fork" => self.control_node(node),
            "individual" => {
                // `individual x : T` already consumed by prefixes; unreachable
                Err(self.error("a usage keyword"))
            }
            _ => {
                if !node.metadata.is_empty() {
                    // `#meta x : T;`: an extended usage.
                    node.kind = "Usage";
                    return self.finish_usage(node);
                }
                if node.prefixes.is_individual || node.prefixes.portion.is_some() || node.prefixes.is_reference || node.prefixes.is_end || node.prefixes.direction.is_some() || self.is_name() || self.is_symbol("<") || self.is_symbol(":") || self.is_symbol(":>") || self.is_symbol("::>") || self.is_symbol(":>>") || self.is_symbol("[") {
                    node.kind = if node.prefixes.is_individual || node.prefixes.portion.is_some() { "OccurrenceUsage" } else { "ReferenceUsage" };
                    return self.finish_usage(node);
                }
                Err(self.error("a declaration"))
            }
        }
    }

    /// `action [decl] accept|send|assign|terminate|if|while|loop|for|merge|decide|join|fork ...`
    fn named_action_node(&mut self, mut node: Node, keyword: &str) -> Parsed<Node> {
        match keyword {
            "accept" => self.accept_node(node),
            "send" => self.send_node(node),
            "assign" => self.assign_node(node),
            "terminate" => self.terminate_node(node),
            _ => {
                self.expect_word("action")?;
                self.usage_declaration(&mut node)?;
                match keyword {
                    "if" => self.if_node(node),
                    "while" | "loop" => self.while_node(node),
                    "for" => self.for_node(node),
                    _ => self.control_node(node),
                }
            }
        }
    }

    /// `[<short> name] [:> A, B] body`
    fn finish_definition(&mut self, mut node: Node) -> Parsed<Node> {
        self.identification(&mut node)?;
        if self.take_symbol(":>") || self.take_word("specializes") {
            loop {
                let target = self.target()?;
                node.specializations.push(Specialization { kind: SpecKind::Subclassification, target, conjugated: false });
                if !self.take_symbol(",") {
                    break;
                }
            }
        }
        let is_state = node.kind == "StateDefinition";
        if is_state && self.take_word("parallel") {
            node.prefixes.is_parallel = true;
        }
        let is_calc = matches!(node.kind, "CalculationDefinition" | "ConstraintDefinition" | "CaseDefinition" | "AnalysisCaseDefinition" | "VerificationCaseDefinition" | "UseCaseDefinition");
        if node.kind == "EnumerationDefinition" {
            node.members = self.enumeration_body()?;
        } else {
            node.members = self.body_in(is_state, is_calc)?;
        }
        Ok(node)
    }

    /// `UsageDeclaration? ValuePart? body` (plus `parallel` for states).
    pub(super) fn finish_usage(&mut self, mut node: Node) -> Parsed<Node> {
        self.usage_declaration(&mut node)?;
        self.value_part(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    pub(super) fn usage_body(&mut self, node: &mut Node) -> Parsed<()> {
        let is_state = node.kind == "StateUsage" || node.kind == "ExhibitStateUsage";
        if is_state && self.take_word("parallel") {
            node.prefixes.is_parallel = true;
        }
        let is_calc = matches!(node.kind, "CalculationUsage" | "ConstraintUsage" | "AssertConstraintUsage" | "CaseUsage" | "AnalysisCaseUsage" | "VerificationCaseUsage" | "UseCaseUsage" | "IncludeUseCaseUsage" | "RequirementUsage" | "ConcernUsage" | "ViewpointUsage" | "SatisfyRequirementUsage");
        // Members the declaration made (a flow's `of payload`) come first.
        let body = self.body_in(is_state, is_calc)?;
        node.members.extend(body);
        Ok(())
    }

    /// `Identification? FeatureSpecializationPart?` | `FeatureSpecializationPart`
    pub(super) fn usage_declaration(&mut self, node: &mut Node) -> Parsed<()> {
        self.identification(node)?;
        self.feature_specialization_part(node)
    }

    /// `(: T | :> s | ::> r | => c | :>> r)* [mult] (ordered|nonunique)* ...`
    pub(super) fn feature_specialization_part(&mut self, node: &mut Node) -> Parsed<()> {
        loop {
            if self.is_symbol("[") {
                let mut multiplicity = self.multiplicity_range()?;
                loop {
                    if self.take_word("ordered") {
                        multiplicity.ordered = true;
                    } else if self.take_word("nonunique") {
                        multiplicity.nonunique = true;
                    } else {
                        break;
                    }
                }
                node.multiplicity = Some(multiplicity);
                continue;
            }
            if self.is_word("ordered") || self.is_word("nonunique") {
                let multiplicity = node.multiplicity.get_or_insert_with(Multiplicity::default);
                if self.take_word("ordered") {
                    multiplicity.ordered = true;
                } else {
                    self.position += 1;
                    multiplicity.nonunique = true;
                }
                continue;
            }
            let kind = if self.take_symbol(":") || (self.is_word("defined") && self.is_word_at(1, "by") && { self.position += 2; true }) {
                SpecKind::Typing
            } else if self.take_symbol("::>") || self.take_word("references") {
                SpecKind::Reference
            } else if self.take_symbol(":>>") || self.take_word("redefines") {
                SpecKind::Redefinition
            } else if self.take_symbol(":>") || self.take_word("subsets") {
                SpecKind::Subsetting
            } else if self.take_symbol("=>") || self.take_word("crosses") {
                SpecKind::Crossing
            } else {
                return Ok(());
            };
            loop {
                let conjugated = kind == SpecKind::Typing && self.take_symbol("~");
                let target = self.target()?;
                node.specializations.push(Specialization { kind, target, conjugated });
                if kind == SpecKind::Reference || kind == SpecKind::Crossing || !self.take_symbol(",") {
                    break;
                }
            }
        }
    }

    /// `[e]` or `[l..u]`
    pub(super) fn multiplicity_range(&mut self) -> Parsed<Multiplicity> {
        self.expect_symbol("[")?;
        let first = self.multiplicity_bound()?;
        let mut multiplicity = Multiplicity::default();
        if self.take_symbol("..") {
            multiplicity.lower = Some(first);
            multiplicity.upper = Some(self.multiplicity_bound()?);
        } else {
            multiplicity.upper = Some(first);
        }
        self.expect_symbol("]")?;
        Ok(multiplicity)
    }

    fn multiplicity_bound(&mut self) -> Parsed<Expr> {
        if self.take_symbol("*") {
            return Ok(Expr::Literal(Literal::Infinity));
        }
        match self.peek() {
            Some(Kind::Integer(n)) => {
                let n = n.clone();
                self.position += 1;
                Ok(Expr::Literal(Literal::Integer(n)))
            }
            _ => Ok(Expr::Reference(self.qualified_name()?)),
        }
    }

    /// `= e`, `:= e`, `default e`, `default = e`, `default := e`
    pub(super) fn value_part(&mut self, node: &mut Node) -> Parsed<()> {
        let kind = if self.take_symbol("=") {
            ValueKind::Bound
        } else if self.take_symbol(":=") {
            ValueKind::Initial
        } else if self.take_word("default") {
            if self.take_symbol(":=") {
                ValueKind::DefaultInitial
            } else {
                self.take_symbol("=");
                ValueKind::Default
            }
        } else {
            return Ok(());
        };
        let expr = self.expression()?;
        node.value = Some(Value { kind, expr });
        Ok(())
    }

    /// `enum def E { a; b = 1; enum c; doc /* */ }`
    fn enumeration_body(&mut self) -> Parsed<Vec<Node>> {
        if self.take_symbol(";") {
            return Ok(Vec::new());
        }
        self.expect_symbol("{")?;
        let mut members = Vec::new();
        while !self.is_symbol("}") {
            if self.peek().is_none() {
                return Err(self.error("`}`"));
            }
            let line = self.line();
            let visibility = self.visibility();
            if matches!(self.peek(), Some(Kind::Comment(_))) || self.is_word("comment") || self.is_word("doc") || self.is_word("rep") || self.is_word("language") || self.is_word("locale") {
                let mut node = self.comment()?;
                node.prefixes.visibility = visibility;
                node.line = line;
                members.push(node);
                continue;
            }
            if self.is_symbol("@") || (self.is_word("metadata") && !self.is_word_at(1, "def")) {
                let mut node = self.metadata_usage(Vec::new())?;
                node.prefixes.visibility = visibility;
                members.push(node);
                continue;
            }
            let mut node = Node::new("EnumerationUsage", "VariantMembership");
            node.line = line;
            node.prefixes.visibility = visibility;
            node.metadata = if self.is_symbol("#") { self.prefix_metadata()? } else { Vec::new() };
            self.take_word("enum");
            self.finish_usage_into(&mut node)?;
            members.push(node);
        }
        self.position += 1;
        Ok(members)
    }

    pub(super) fn finish_usage_into(&mut self, node: &mut Node) -> Parsed<()> {
        self.usage_declaration(node)?;
        self.value_part(node)?;
        self.usage_body(node)
    }

    /// A usage statement after a membership keyword (`variant`, `return`).
    pub(super) fn usage_statement(&mut self) -> Parsed<Node> {
        let metadata = if self.is_symbol("#") { self.prefix_metadata()? } else { Vec::new() };
        self.definition_or_usage(metadata)
    }

    // --- connectors -------------------------------------------------------

    /// One end: `[mult]? (name ::>)? target`.
    pub(super) fn connector_end(&mut self) -> Parsed<End> {
        let multiplicity = if self.is_symbol("[") { Some(self.multiplicity_range()?) } else { None };
        let mut name = None;
        if self.is_name() && (self.is_symbol_at(1, "::>") || self.is_word_at(1, "references")) {
            name = Some(self.name()?);
            self.position += 1;
        }
        let target = self.target()?;
        Ok(End { name, multiplicity, target: Some(target), flow_feature: None })
    }

    /// `a to b` or `(a, b, c)`
    fn connector_part(&mut self) -> Parsed<Vec<End>> {
        if self.take_symbol("(") {
            let mut ends = vec![self.connector_end()?];
            while self.take_symbol(",") {
                ends.push(self.connector_end()?);
            }
            self.expect_symbol(")")?;
            return Ok(ends);
        }
        let first = self.connector_end()?;
        self.expect_word("to")?;
        let second = self.connector_end()?;
        Ok(vec![first, second])
    }

    /// `connection [decl] [= v] [connect ends] body` | `connect ends body`;
    /// the same for `allocation` / `allocate`.
    fn connection(&mut self, mut node: Node, kind: &'static str, keyword: &str, verb: &str) -> Parsed<Node> {
        node.kind = kind;
        if self.take_word(keyword) {
            self.usage_declaration(&mut node)?;
            self.value_part(&mut node)?;
            if self.take_word(verb) {
                let ends = self.connector_part()?;
                node.detail = Detail::Connector { ends };
            }
        } else {
            self.expect_word(verb)?;
            let ends = self.connector_part()?;
            node.detail = Detail::Connector { ends };
        }
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `interface [decl] [= v] [connect ends] body` | `interface ends body`
    fn interface(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "InterfaceUsage";
        self.expect_word("interface")?;
        // `interface a to b` / `interface (a, b)`: ends without a declaration.
        let ends_first = self.is_symbol("(") || (self.is_name() && (self.is_word_at(1, "to") || (self.is_symbol_at(1, ".") && !self.is_symbol_at(1, "::")) && self.ends_follow()));
        if ends_first {
            let ends = self.connector_part()?;
            node.detail = Detail::Connector { ends };
        } else {
            self.usage_declaration(&mut node)?;
            self.value_part(&mut node)?;
            if self.take_word("connect") {
                let ends = self.connector_part()?;
                node.detail = Detail::Connector { ends };
            }
        }
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// Whether the tokens from here form `target to target` before `;`/`{`:
    /// a connector's ends rather than a declaration.
    fn ends_follow(&self) -> bool {
        let mut i = self.position;
        while let Some(token) = self.tokens.get(i) {
            match &token.kind {
                Kind::Word(w) if w == "to" => return true,
                Kind::Word(w) if matches!(w.as_str(), "of" | "from" | "connect" | "default" | "then" | "first" | "by" | "defined" | "subsets" | "redefines" | "references" | "ordered" | "nonunique") => return false,
                Kind::Symbol(";") | Kind::Symbol("{") | Kind::Symbol(":") | Kind::Symbol(":>") | Kind::Symbol(":>>") | Kind::Symbol("::>") | Kind::Symbol("=") | Kind::Symbol(":=") | Kind::Symbol("[") => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// `[binding [decl]] bind a = b body`
    fn binding(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "BindingConnectorAsUsage";
        if self.take_word("binding") {
            self.usage_declaration(&mut node)?;
        }
        self.expect_word("bind")?;
        let first = self.connector_end()?;
        self.expect_symbol("=")?;
        let second = self.connector_end()?;
        node.detail = Detail::Connector { ends: vec![first, second] };
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `[succession [decl]] first a then b body`, or a guarded succession
    /// `succession s first a if g then b`.
    fn succession(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "SuccessionAsUsage";
        if self.take_word("succession") {
            self.usage_declaration(&mut node)?;
        }
        self.expect_word("first")?;
        let first = self.connector_end()?;
        if self.take_word("if") {
            let guard = self.expression()?;
            self.expect_word("then")?;
            let target = self.connector_end()?;
            node.kind = "TransitionUsage";
            node.detail = Detail::Transition { source: first.target, trigger: None, guard: Some(guard), effect: None, target, is_else: false };
            self.usage_body(&mut node)?;
            return Ok(node);
        }
        self.expect_word("then")?;
        let second = self.connector_end()?;
        node.detail = Detail::Connector { ends: vec![first, second] };
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// A flow end: `a.b.x` references `a.b` and names the feature `x`; a
    /// single name is the feature itself.
    fn flow_end(&mut self) -> Parsed<End> {
        let target = self.target()?;
        let (target, flow_feature) = match target {
            Target::Chain(mut steps) if steps.len() >= 2 => {
                let feature = steps.pop().unwrap();
                let base = if steps.len() == 1 { Target::Name(steps.pop().unwrap()) } else { Target::Chain(steps) };
                (Some(base), Some(feature))
            }
            Target::Name(name) => (None, Some(name)),
            chain => (Some(chain), None),
        };
        Ok(End { name: None, multiplicity: None, target, flow_feature })
    }

    /// `flow [decl] [= v] [of payload] [from a to b] body` | `flow a to b body`
    /// (`succession flow` too).
    fn flow(&mut self, mut node: Node, kind: &'static str) -> Parsed<Node> {
        node.kind = kind;
        self.expect_word("flow")?;
        self.flow_declaration(&mut node, false)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `message [decl] [= v] [of payload] [from a to b] body` | `message a to b body`
    fn message(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "FlowUsage";
        self.expect_word("message")?;
        self.flow_declaration(&mut node, true)?;
        node.membership_kind = Some("message");
        self.usage_body(&mut node)?;
        Ok(node)
    }

    fn flow_declaration(&mut self, node: &mut Node, is_message: bool) -> Parsed<()> {
        // Ends without a declaration: `flow a.x to b.y`.
        if self.is_name() && self.ends_follow() {
            let first = if is_message { self.event_end()? } else { self.flow_end()? };
            self.expect_word("to")?;
            let second = if is_message { self.event_end()? } else { self.flow_end()? };
            node.detail = Detail::Connector { ends: vec![first, second] };
            return Ok(());
        }
        self.usage_declaration(node)?;
        self.value_part(node)?;
        if self.take_word("of") {
            let mut payload = Node::new("PayloadFeature", "FeatureMembership");
            self.payload(&mut payload)?;
            node.members.push(payload);
        }
        if self.take_word("from") {
            let first = if is_message { self.event_end()? } else { self.flow_end()? };
            self.expect_word("to")?;
            let second = if is_message { self.event_end()? } else { self.flow_end()? };
            node.detail = Detail::Connector { ends: vec![first, second] };
        }
        Ok(())
    }

    /// A message end: an event occurrence referencing a feature.
    fn event_end(&mut self) -> Parsed<End> {
        let target = self.target()?;
        Ok(End { name: None, multiplicity: None, target: Some(target), flow_feature: None })
    }

    /// `[id] spec [= v]` | `[id] = v` | `T [mult]` | `[mult] T`
    pub(super) fn payload(&mut self, node: &mut Node) -> Parsed<()> {
        if self.is_symbol("[") {
            node.multiplicity = Some(self.multiplicity_range()?);
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Typing, target, conjugated: false });
            return Ok(());
        }
        let named = self.is_symbol("<") || (self.is_name() && (matches!(self.peek_at(1), Some(Kind::Symbol(s)) if matches!(*s, ":" | ":>" | "::>" | ":>>" | "=>" | "[" | "=" | ":=")) || matches!(self.peek_at(1), Some(Kind::Word(w)) if w == "default" || w == "defined" || w == "subsets" || w == "redefines")));
        if named {
            self.identification(node)?;
            self.feature_specialization_part(node)?;
            self.value_part(node)?;
            return Ok(());
        }
        if self.is_symbol("=") || self.is_symbol(":=") || self.is_word("default") {
            return self.value_part(node);
        }
        let target = self.target()?;
        node.specializations.push(Specialization { kind: SpecKind::Typing, target, conjugated: false });
        if self.is_symbol("[") {
            node.multiplicity = Some(self.multiplicity_range()?);
        }
        Ok(())
    }

    /// `event occurrence [decl] body` | `event x [spec] body`
    fn event_occurrence(&mut self, mut node: Node) -> Parsed<Node> {
        if self.take_word("occurrence") {
            self.usage_declaration(&mut node)?;
        } else {
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Reference, target, conjugated: false });
            self.feature_specialization_part(&mut node)?;
        }
        self.value_part(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `perform action a body` | `perform x [spec] [= v] body` (and `exhibit`, `include`).
    fn performance(&mut self, mut node: Node, verb: &str, kind: &'static str, keyword: &str) -> Parsed<Node> {
        node.kind = kind;
        self.expect_word(verb)?;
        if self.take_word(keyword) {
            if keyword == "use" {
                self.expect_word("case")?;
            }
            self.usage_declaration(&mut node)?;
        } else {
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Reference, target, conjugated: false });
            self.feature_specialization_part(&mut node)?;
        }
        self.value_part(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `assert [not] constraint [decl] [= v] body` | `assert [not] x [spec] body`
    /// | `assert [not] satisfy ...`
    fn assertion(&mut self, mut node: Node) -> Parsed<Node> {
        self.expect_word("assert")?;
        let negated = self.take_word("not");
        node.prefixes.is_negated = negated;
        if self.is_word("satisfy") {
            return self.satisfy(node);
        }
        node.kind = "AssertConstraintUsage";
        if self.take_word("constraint") {
            self.usage_declaration(&mut node)?;
        } else {
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Reference, target, conjugated: false });
            self.feature_specialization_part(&mut node)?;
        }
        self.value_part(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `satisfy requirement [decl] [= v] [by x] body` | `satisfy R [spec] [by x] body`
    fn satisfy(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "SatisfyRequirementUsage";
        self.expect_word("satisfy")?;
        if self.take_word("requirement") {
            self.usage_declaration(&mut node)?;
        } else {
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Reference, target, conjugated: false });
            self.feature_specialization_part(&mut node)?;
        }
        self.value_part(&mut node)?;
        let by = if self.take_word("by") { Some(self.primary_expression()?) } else { None };
        node.detail = Detail::Satisfy { by };
        self.usage_body(&mut node)?;
        Ok(node)
    }
}
