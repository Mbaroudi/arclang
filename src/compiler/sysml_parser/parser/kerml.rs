//! The KerML declarations SysML text does not have (KerML.xtext): types
//! (`classifier`, `datatype`, `struct`, `behavior`, `function`,
//! `metaclass`...) and features (`feature`, `step`, `expr`, `inv`...), with
//! the KerML spellings of their relationships. The kernel libraries are
//! written this way; a SysML model may import them.

use super::{Parsed, Parser};
use crate::compiler::sysml_parser::lexer::Kind;
use crate::compiler::sysml_parser::syntax::*;

/// Whether a KerML declaration is a type (a classifier) or a feature.
#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Type,
    Feature,
}

/// The metaclass a KerML keyword declares, with the words it takes.
fn keyword(first: &str, second: Option<&str>) -> Option<(&'static str, Shape, usize)> {
    Some(match (first, second) {
        ("assoc", Some("struct")) => ("AssociationStructure", Shape::Type, 2),
        ("assoc", _) => ("Association", Shape::Type, 1),
        ("type", _) => ("Type", Shape::Type, 1),
        ("classifier", _) => ("Classifier", Shape::Type, 1),
        ("class", _) => ("Class", Shape::Type, 1),
        ("struct", _) => ("Structure", Shape::Type, 1),
        ("datatype", _) => ("DataType", Shape::Type, 1),
        ("behavior", _) => ("Behavior", Shape::Type, 1),
        ("function", _) => ("Function", Shape::Type, 1),
        ("predicate", _) => ("Predicate", Shape::Type, 1),
        ("interaction", _) => ("Interaction", Shape::Type, 1),
        ("metaclass", _) => ("Metaclass", Shape::Type, 1),
        ("feature", _) => ("Feature", Shape::Feature, 1),
        ("step", _) => ("Step", Shape::Feature, 1),
        ("expr", _) => ("Expression", Shape::Feature, 1),
        ("bool", _) => ("BooleanExpression", Shape::Feature, 1),
        ("inv", _) => ("Invariant", Shape::Feature, 1),
        ("connector", _) => ("Connector", Shape::Feature, 1),
        ("multiplicity", _) => ("MultiplicityRange", Shape::Feature, 1),
        _ => return None,
    })
}

/// KerML's reserved words (KerML.xtext keywords).
pub const KERML_RESERVED: &[&str] = &[
    "about", "abstract", "alias", "all", "and", "as", "assoc", "behavior", "binding", "bool", "by", "chains", "class", "classifier", "comment",
    "composite", "conjugate", "conjugates", "conjugation", "connector", "const", "crosses", "datatype", "default", "dependency", "derived",
    "differences", "disjoining", "disjoint", "doc", "else", "end", "expr", "false", "feature", "featured", "featuring", "filter", "first", "flow",
    "for", "from", "function", "hastype", "if", "implies", "import", "in", "inout", "interaction", "intersects", "inv", "inverse", "inverting",
    "istype", "language", "library", "locale", "member", "meta", "metaclass", "metadata", "multiplicity", "namespace", "new", "nonunique", "not",
    "null", "of", "or", "ordered", "out", "package", "portion", "predicate", "private", "protected", "public", "redefines", "redefinition",
    "references", "rep", "return", "specialization", "specializes", "standard", "step", "struct", "subclassifier", "subset", "subsets", "subtype",
    "succession", "then", "to", "true", "type", "typed", "typing", "unions", "var", "xor",
];

/// KerML statements that declare one relationship (`subset a subsets b;`).
const RELATIONSHIP_STATEMENTS: &[&str] = &["disjoint", "subset", "specialization", "subclassifier", "subtype", "redefinition", "conjugation", "conjugate", "disjoining", "featuring", "inverting", "typing"];

/// KerML prefixes with no SysML counterpart worth keeping.
pub(super) const KERML_PREFIXES: &[&str] = &["var", "composite", "portion", "member", "readonly", "const"];

impl Parser {
    /// A KerML declaration at the current word, if it is one (the keyword
    /// must be followed by what a declaration starts with); else the node back.
    pub(super) fn kerml_declaration(&mut self, mut node: Node) -> Result<Parsed<Node>, Node> {
        let Some(Kind::Word(first)) = self.peek().cloned() else { return Err(node) };
        let second = match self.peek_at(1) {
            Some(Kind::Word(w)) => Some(w.clone()),
            _ => None,
        };
        if self.reserved == KERML_RESERVED && RELATIONSHIP_STATEMENTS.contains(&first.as_str()) {
            return Ok(self.relationship_statement(node));
        }
        // A KerML flow declares a feature (`flow f : T subsets x;`); one with
        // ends is read by the SysML flow.
        let flow_feature = first == "flow" && self.reserved == KERML_RESERVED && !self.statement_has_word(&["from", "of", "to"]);
        let found = if flow_feature { Some(("ItemFlow", Shape::Feature, 1)) } else { keyword(&first, second.as_deref()) };
        let Some((kind, shape, words)) = found else { return Err(node) };
        if !self.declaration_follows(words) {
            return Err(node);
        }
        self.position += words;
        node.kind = kind;
        Ok(match shape {
            Shape::Type => {
                node.membership = "OwningMembership";
                self.kerml_type(node)
            }
            Shape::Feature if kind == "Connector" => self.kerml_connector(node),
            Shape::Feature => self.kerml_feature(node),
        })
    }

    /// `connector [all] [decl] ([from] a to b | (a, b, ...)) body`
    fn kerml_connector(&mut self, mut node: Node) -> Parsed<Node> {
        self.take_word("all");
        self.usage_declaration(&mut node)?;
        let mut ends = Vec::new();
        let named_first = self.end_from_name(&mut node);
        if let Some(first) = named_first {
            ends.push(first);
            self.expect_word("to")?;
            ends.push(self.connector_end()?);
        } else if self.take_word("from") || self.is_name() || self.is_symbol("[") {
            ends.push(self.connector_end()?);
            self.expect_word("to")?;
            ends.push(self.connector_end()?);
        } else if self.take_symbol("(") {
            loop {
                ends.push(self.connector_end()?);
                if !self.take_symbol(",") {
                    break;
                }
            }
            self.expect_symbol(")")?;
        }
        node.kind = "Connector";
        node.detail = Detail::Connector { ends };
        node.members.extend(self.body()?);
        Ok(node)
    }

    /// In KerML a binding or succession may start with its first end: the
    /// name the declaration read is that end.
    pub(super) fn end_from_name(&mut self, node: &mut Node) -> Option<End> {
        if !node.specializations.is_empty() || !(self.is_symbol("=") || self.is_word("then") || self.is_word("to")) {
            return None;
        }
        let name = node.name.take()?;
        Some(End { name: None, multiplicity: node.multiplicity.take(), target: Some(Target::Name(QualifiedName::simple(&name))), flow_feature: None })
    }

    /// After `words` keywords: a name, a short name, a specialization, a
    /// multiplicity, a body or the end of the statement.
    fn declaration_follows(&self, words: usize) -> bool {
        match self.peek_at(words) {
            Some(Kind::Symbol(s)) => matches!(*s, "<" | ":" | ":>" | ":>>" | "::>" | "~" | "[" | ";" | "{" | "=" | ":=" | "("),
            Some(Kind::Word(w)) => !matches!(w.as_str(), "def"),
            Some(Kind::Name(_)) => true,
            _ => false,
        }
    }

    /// `name (specializes | :> | conjugates | ~ | disjoint from | unions |
    /// intersects | differences) ... body`
    fn kerml_type(&mut self, mut node: Node) -> Parsed<Node> {
        // `struct all Body`: a sufficient type.
        self.take_word("all");
        self.identification(&mut node)?;
        if self.is_symbol("[") {
            node.multiplicity = Some(self.multiplicity_range()?);
        }
        self.kerml_relationships(&mut node, SpecKind::Subclassification)?;
        let is_calc = matches!(node.kind, "Function" | "Predicate");
        node.members = self.body_in(false, is_calc)?;
        Ok(node)
    }

    /// `name (: | typed by | subsets | redefines | references | chains |
    /// inverse of | featured by ...)* [mult] [= value] body`
    fn kerml_feature(&mut self, mut node: Node) -> Parsed<Node> {
        // `feature all x`: a sufficient feature.
        self.take_word("all");
        self.identification(&mut node)?;
        loop {
            let before = self.position;
            self.feature_specialization_part(&mut node)?;
            self.kerml_relationships(&mut node, SpecKind::Subsetting)?;
            if self.position == before {
                break;
            }
        }
        self.value_part(&mut node)?;
        let is_calc = matches!(node.kind, "Expression" | "BooleanExpression" | "Invariant");
        node.members.extend(self.body_in(false, is_calc)?);
        Ok(node)
    }

    /// The KerML spellings of relationships. Specializations are kept;
    /// disjoining, unioning, chaining and the like name types without
    /// making them generals, and are read past.
    fn kerml_relationships(&mut self, node: &mut Node, general: SpecKind) -> Parsed<()> {
        loop {
            let kind = if self.take_word("specializes") || self.take_symbol(":>") {
                Some(general)
            } else if self.take_word("typed") {
                self.expect_word("by")?;
                Some(SpecKind::Typing)
            } else if self.take_word("conjugates") || self.take_symbol("~") {
                Some(general)
            } else if self.take_word("disjoint") {
                self.expect_word("from")?;
                None
            } else if self.take_word("inverse") {
                self.expect_word("of")?;
                None
            } else if self.take_word("featured") {
                self.expect_word("by")?;
                None
            } else if self.take_word("unions") || self.take_word("intersects") || self.take_word("differences") || self.take_word("chains") {
                None
            } else {
                return Ok(());
            };
            loop {
                let target = self.target()?;
                if let Some(kind) = kind {
                    node.specializations.push(Specialization { kind, target, conjugated: false });
                }
                if !self.take_symbol(",") {
                    break;
                }
            }
        }
    }

    /// A relationship statement, read past: the relationship it declares
    /// between library features is not structure an import needs.
    fn relationship_statement(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "Relationship";
        let mut depth = 0i32;
        while let Some(kind) = self.peek().cloned() {
            self.position += 1;
            match kind {
                Kind::Symbol("{") => depth += 1,
                Kind::Symbol("}") => {
                    depth -= 1;
                    if depth <= 0 {
                        break;
                    }
                }
                Kind::Symbol(";") if depth == 0 => break,
                _ => {}
            }
        }
        Ok(node)
    }

    /// Whether one of `words` comes before the end of the statement, from
    /// the current token.
    pub(super) fn statement_has_word_here(&self, words: &[&str]) -> bool {
        self.statement_has_word_from(0, words)
    }

    /// Whether one of `words` comes before the end of the statement, after
    /// the current (keyword) token.
    fn statement_has_word(&self, words: &[&str]) -> bool {
        self.statement_has_word_from(1, words)
    }

    fn statement_has_word_from(&self, start: usize, words: &[&str]) -> bool {
        let mut i = start;
        while let Some(kind) = self.peek_at(i) {
            match kind {
                Kind::Symbol(";") | Kind::Symbol("{") => return false,
                Kind::Word(w) if words.contains(&w.as_str()) => return true,
                _ => {}
            }
            i += 1;
        }
        false
    }
}
