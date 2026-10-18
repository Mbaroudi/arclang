//! The syntax tree of a KerML / SysML v2 text: one `Node` per element as
//! the grammar (SysML.xtext) declares it, with what the text says and
//! nothing derived. Names are kept as written; `records` resolves them.

/// A qualified name as written: `$::A::B` has `global`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct QualifiedName {
    pub global: bool,
    pub segments: Vec<String>,
}

impl QualifiedName {
    pub fn simple(name: &str) -> Self {
        QualifiedName { global: false, segments: vec![name.to_string()] }
    }
    pub fn last(&self) -> &str {
        self.segments.last().map(String::as_str).unwrap_or("")
    }
    pub fn text(&self) -> String {
        let body = self.segments.iter().map(|s| quote(s)).collect::<Vec<_>>().join("::");
        if self.global { format!("$::{body}") } else { body }
    }
}

/// A name as the notation writes it: bare when it is an identifier.
pub fn quote(name: &str) -> String {
    let plain = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if plain { name.to_string() } else { format!("'{}'", name.replace('\\', "\\\\").replace('\'', "\\'")) }
}

/// What a specialization or an end points at: a qualified name, or a
/// feature chain `a.b.c` (each step a qualified name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    Name(QualifiedName),
    Chain(Vec<QualifiedName>),
}

impl Target {
    pub fn text(&self) -> String {
        match self {
            Target::Name(n) => n.text(),
            Target::Chain(steps) => steps.iter().map(QualifiedName::text).collect::<Vec<_>>().join("."),
        }
    }
    pub fn first(&self) -> &QualifiedName {
        match self {
            Target::Name(n) => n,
            Target::Chain(steps) => &steps[0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecKind {
    /// `: T` (FeatureTyping), `~T` when conjugated
    Typing,
    /// `:> S` on a classifier (Subclassification)
    Subclassification,
    /// `:> s` on a feature (Subsetting)
    Subsetting,
    /// `::> r` (ReferenceSubsetting)
    Reference,
    /// `=> c` (CrossSubsetting)
    Crossing,
    /// `:>> r` (Redefinition)
    Redefinition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Specialization {
    pub kind: SpecKind,
    pub target: Target,
    /// `~T`: a conjugated port typing.
    pub conjugated: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Multiplicity {
    /// `[n]`: no lower bound; `[l..u]`: both.
    pub lower: Option<Expr>,
    pub upper: Option<Expr>,
    pub ordered: bool,
    pub nonunique: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// `= e`
    Bound,
    /// `:= e`
    Initial,
    /// `default = e` or `default e`
    Default,
    /// `default := e`
    DefaultInitial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub kind: ValueKind,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Boolean(bool),
    String(String),
    Integer(String),
    Real(String),
    Infinity,
}

/// What `->` applies: a body, a function reference, or arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrowArgument {
    Body(Box<Expr>),
    Function(QualifiedName),
    Arguments(Vec<Argument>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    /// `name = value` names a parameter (a redefinition).
    pub name: Option<QualifiedName>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null,
    Literal(Literal),
    /// A feature by name: `x`, `a::b`.
    Reference(QualifiedName),
    /// `x.metadata`
    MetadataAccess(QualifiedName),
    /// `f(a, b)`: the type, by name or feature chain.
    Invocation { function: Target, arguments: Vec<Argument> },
    /// `new T(a, b)`
    Constructor { ty: Target, arguments: Vec<Argument> },
    /// `{ in x : T; ... result }`: a calculation body (parameters, members, a result expression).
    Body { members: Vec<Node> },
    /// `(a, b, c)`
    Sequence(Vec<Expr>),
    /// `a op b`, `-a`, `not a`, `a ?? b`, `a implies b`...
    Operator { operator: String, operands: Vec<Expr> },
    /// `if c ? a else b`
    Conditional { condition: Box<Expr>, then: Box<Expr>, otherwise: Box<Expr> },
    /// `a istype T`, `a hastype T`, `a @ T`, `a @@ T`; `istype T` on the implicit self when `operand` is None
    TypeTest { operand: Option<Box<Expr>>, operator: String, ty: QualifiedName },
    /// `a as T`, `a meta T`
    Cast { operand: Option<Box<Expr>>, operator: String, ty: QualifiedName },
    /// `all T`
    Extent(QualifiedName),
    /// `base.a.b`: a feature chain on a value.
    Chain { base: Box<Expr>, chain: Target },
    /// `base[i]`
    Bracket { base: Box<Expr>, index: Box<Expr> },
    /// `base#(i, j)`
    Index { base: Box<Expr>, index: Box<Expr> },
    /// `base->T(args)`, `base->T { body }`, `base->T f`
    Arrow { base: Box<Expr>, ty: Target, argument: ArrowArgument },
    /// `base.{ body }`
    Collect { base: Box<Expr>, body: Box<Expr> },
    /// `base.?{ body }`
    Select { base: Box<Expr>, body: Box<Expr> },
}

/// Prefix keywords of a definition or usage.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Prefixes {
    pub visibility: Option<&'static str>,
    pub direction: Option<&'static str>,
    pub is_abstract: bool,
    pub is_variation: bool,
    pub is_derived: bool,
    pub is_constant: bool,
    pub is_reference: bool,
    pub is_end: bool,
    pub is_individual: bool,
    pub is_standard: bool,
    pub is_parallel: bool,
    pub is_negated: bool,
    /// `snapshot` / `timeslice`
    pub portion: Option<&'static str>,
}

/// One end of a connector (`connect a to b`, `first a then b`, `flow from a.x to b.y`).
#[derive(Debug, Clone, PartialEq)]
pub struct End {
    /// `name ::> target`
    pub name: Option<String>,
    pub multiplicity: Option<Multiplicity>,
    /// The referenced feature; `None` for an end the text leaves empty.
    pub target: Option<Target>,
    /// A flow end's feature after the last `.`: `a.b.x` references `a.b` and redefines `x`.
    pub flow_feature: Option<QualifiedName>,
}

/// A trigger of `accept`: `at e`, `after e`, `when e`.
#[derive(Debug, Clone, PartialEq)]
pub struct Trigger {
    pub kind: String,
    pub expr: Expr,
}

/// The parts of an element the generic declaration does not hold.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Detail {
    #[default]
    None,
    /// Connector ends: `connect`, `bind`, `first .. then`, `flow`, `message`, `allocate`, `interface` ends.
    Connector { ends: Vec<End> },
    /// `dependency from a, b to c`
    Dependency { clients: Vec<QualifiedName>, suppliers: Vec<QualifiedName> },
    /// `import P::*;`, `import P::X;`, `expose`
    Import { target: QualifiedName, namespace: bool, recursive: bool, all: bool, filters: Vec<Expr> },
    /// `alias <s> n for X;`
    Alias { target: QualifiedName },
    /// `comment about X, Y locale "..." /* body */`, `doc`, `rep language "..."`
    Annotation { about: Vec<QualifiedName>, locale: Option<String>, language: Option<String>, body: String },
    /// `filter expr;`
    Filter(Expr),
    /// A result expression at the end of a body.
    Result(Expr),
    /// `first x;` in an action body
    Initial(Target),
    /// `transition first a accept t if g do e then b`, `then b`, `if g then b`, `else b`
    Transition { source: Option<Target>, trigger: Option<Box<Node>>, guard: Option<Expr>, effect: Option<Box<Node>>, target: End, is_else: bool },
    /// `accept payload via x`
    Accept { payload: Box<Node>, via: Option<Expr> },
    /// `send payload via x to y`
    Send { payload: Option<Expr>, via: Option<Expr>, to: Option<Expr> },
    /// `assign target := value`
    Assign { target: Expr, value: Expr },
    /// `if c { } else { }`
    If { condition: Expr, then: Box<Node>, otherwise: Option<Box<Node>> },
    /// `while c { } until e;`, `loop { }`
    While { condition: Option<Expr>, body: Box<Node>, until: Option<Expr> },
    /// `for x in c { }`
    For { variable: Box<Node>, collection: Expr, body: Box<Node> },
    /// `terminate x;`
    Terminate(Option<Expr>),
    /// `satisfy R by x`
    Satisfy { by: Option<Expr> },
    /// A trigger value of an accept payload: `at e`, `after e`, `when e`.
    Trigger(Trigger),
}

/// An element as declared.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Node {
    /// The metaclass: `PartUsage`, `Package`, `Dependency`, `Comment`, `NamespaceImport`...
    pub kind: &'static str,
    /// The membership the owner holds it through: `OwningMembership`,
    /// `FeatureMembership`, `VariantMembership`, `ParameterMembership`,
    /// `EndFeatureMembership`, `SubjectMembership`, `ObjectiveMembership`...
    pub membership: &'static str,
    /// A kind the membership carries: `entry`/`do`/`exit`, `assume`/`require`,
    /// `accept`/`if`/`do` of a transition feature.
    pub membership_kind: Option<&'static str>,
    pub prefixes: Prefixes,
    pub short_name: Option<String>,
    pub name: Option<String>,
    pub specializations: Vec<Specialization>,
    pub multiplicity: Option<Multiplicity>,
    pub value: Option<Value>,
    /// `#Meta` prefixes: metadata usages annotating this element.
    pub metadata: Vec<Node>,
    /// The body, in order: members, imports, aliases, annotations, filters.
    pub members: Vec<Node>,
    pub detail: Detail,
    /// Line of the declaration, for messages.
    pub line: usize,
}

impl Node {
    pub fn new(kind: &'static str, membership: &'static str) -> Self {
        Node { kind, membership, ..Node::default() }
    }

    /// The typings, as written.
    pub fn typings(&self) -> impl Iterator<Item = &Specialization> {
        self.specializations.iter().filter(|s| s.kind == SpecKind::Typing)
    }
}
