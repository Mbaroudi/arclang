//! Actions, states, transitions, requirements, cases, views and metadata:
//! the usages with a syntax of their own.

use super::{Parsed, Parser};
use crate::compiler::sysml_parser::lexer::Kind;
use crate::compiler::sysml_parser::syntax::*;

impl Parser {
    /// `then [mult] target;` (a succession from the previous member), or
    /// `then [mult] usage` (a succession to the usage declared next).
    pub(super) fn then_statement(&mut self) -> Parsed<Node> {
        let line = self.line();
        self.expect_word("then")?;
        let multiplicity = if self.is_symbol("[") { Some(self.multiplicity_range()?) } else { None };
        // A usage follows: `then action b;`, `then merge m;`, `then #meta x;`:
        // the usage itself, marked as succeeding the previous member (the
        // succession is implied when records are built).
        if self.is_declaration_start_here() {
            let _ = multiplicity;
            let mut usage = self.statement()?;
            usage.membership_kind = Some("after-then");
            usage.line = line;
            return Ok(usage);
        }
        let target = self.connector_end()?;
        let mut node = Node::new(if self.in_state { "TransitionUsage" } else { "SuccessionAsUsage" }, "FeatureMembership");
        node.line = line;
        if self.in_state {
            node.detail = Detail::Transition { source: None, trigger: None, guard: None, effect: None, target, is_else: false };
        } else {
            node.detail = Detail::Connector { ends: vec![End { name: None, multiplicity, target: None, flow_feature: None }, target] };
        }
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// Whether a declaration (not a reference) starts here.
    fn is_declaration_start_here(&self) -> bool {
        match self.peek() {
            Some(Kind::Word(w)) => {
                super::RESERVED.contains(&w.as_str())
                    && !matches!(w.as_str(), "to" | "then" | "else" | "if")
            }
            Some(Kind::Symbol(s)) => matches!(*s, "#" | "@"),
            Some(Kind::Name(_)) => self.is_name() && matches!(self.peek_at(1), Some(Kind::Symbol(s)) if matches!(*s, ":" | ":>" | "::>" | ":>>" | "[" | "=" | ":=")),
            _ => false,
        }
    }

    /// `else target;` after a guarded succession: a default transition.
    pub(super) fn else_statement(&mut self) -> Parsed<Node> {
        let line = self.line();
        self.expect_word("else")?;
        let target = self.connector_end()?;
        let mut node = Node::new("TransitionUsage", "FeatureMembership");
        node.line = line;
        node.detail = Detail::Transition { source: None, trigger: None, guard: None, effect: None, target, is_else: true };
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `first x;` in an action body: the initial node.
    pub(super) fn initial_node(&mut self) -> Parsed<Node> {
        self.expect_word("first")?;
        let target = self.target()?;
        let mut node = Node::new("InitialNode", "Membership");
        node.detail = Detail::Initial(target);
        node.members = self.body()?;
        Ok(node)
    }

    /// `if g then target;` after an action: a guarded succession. Called
    /// from `if_node` when `then` follows the condition.
    fn guarded_succession(&mut self, mut node: Node, guard: Expr) -> Parsed<Node> {
        self.expect_word("then")?;
        let target = self.connector_end()?;
        node.kind = "TransitionUsage";
        node.detail = Detail::Transition { source: None, trigger: None, guard: Some(guard), effect: None, target, is_else: false };
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `objective [#meta] [decl] [= v] body`
    pub(super) fn objective(&mut self) -> Parsed<Node> {
        self.expect_word("objective")?;
        let mut node = Node::new("RequirementUsage", "ObjectiveMembership");
        node.metadata = self.prefix_metadata()?;
        self.finish_usage_into(&mut node)?;
        Ok(node)
    }

    /// `subject x : T;`, `actor a : A;`, `stakeholder s;`
    pub(super) fn framed(&mut self, kind: &'static str, membership: &'static str, _unused: Option<()>) -> Parsed<Node> {
        self.position += 1;
        let mut node = Node::new(kind, membership);
        node.metadata = self.prefix_metadata()?;
        self.finish_usage_into(&mut node)?;
        Ok(node)
    }

    /// `assume|require constraint [decl] body` | `assume|require x [spec] body`
    /// | `assume|require #meta ...`
    pub(super) fn requirement_constraint(&mut self) -> Parsed<Node> {
        let kind_word: &'static str = if self.take_word("assume") {
            "assume"
        } else {
            self.expect_word("require")?;
            "require"
        };
        let mut node = Node::new("ConstraintUsage", "RequirementConstraintMembership");
        node.membership_kind = Some(kind_word);
        node.metadata = self.prefix_metadata()?;
        if self.take_word("constraint") || !node.metadata.is_empty() && !self.is_name_followed_by_reference() {
            self.finish_usage_into(&mut node)?;
        } else {
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Reference, target, conjugated: false });
            self.feature_specialization_part(&mut node)?;
            self.usage_body(&mut node)?;
        }
        Ok(node)
    }

    /// After `#meta`, whether a bare reference follows (`require #m x;`): a
    /// name that ends the statement.
    fn is_name_followed_by_reference(&self) -> bool {
        self.is_name() && (self.is_symbol_at(1, ";") || self.is_symbol_at(1, "{") || self.is_symbol_at(1, ".") || self.is_symbol_at(1, "::"))
    }

    /// `frame concern [decl] body` | `frame x [spec] body`;
    /// `verify requirement [decl] body` | `verify x [spec] body`.
    pub(super) fn concern_or_verification(&mut self, word: &str) -> Parsed<Node> {
        self.position += 1;
        let (kind, membership, keyword): (&'static str, &'static str, &str) = if word == "frame" {
            ("ConcernUsage", "FramedConcernMembership", "concern")
        } else {
            ("RequirementUsage", "RequirementVerificationMembership", "requirement")
        };
        let mut node = Node::new(kind, membership);
        node.membership_kind = Some(if word == "frame" { "frame" } else { "verify" });
        node.metadata = self.prefix_metadata()?;
        if self.take_word(keyword) {
            self.finish_usage_into(&mut node)?;
        } else {
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Reference, target, conjugated: false });
            self.feature_specialization_part(&mut node)?;
            self.usage_body(&mut node)?;
        }
        Ok(node)
    }

    /// `render rendering [decl] body` | `render x [spec] body`
    pub(super) fn rendering(&mut self) -> Parsed<Node> {
        self.expect_word("render")?;
        let mut node = Node::new("RenderingUsage", "ViewRenderingMembership");
        node.metadata = self.prefix_metadata()?;
        if self.take_word("rendering") {
            self.finish_usage_into(&mut node)?;
        } else {
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Reference, target, conjugated: false });
            self.feature_specialization_part(&mut node)?;
            self.usage_body(&mut node)?;
        }
        Ok(node)
    }

    /// `entry;`, `entry action a { }`, `entry perform x;`, `do send ...;`, `exit accept ...;`
    pub(super) fn state_subaction(&mut self) -> Parsed<Node> {
        let kind_word: &'static str = match self.advance() {
            Some(Kind::Word(w)) if w == "entry" => "entry",
            Some(Kind::Word(w)) if w == "do" => "do",
            _ => "exit",
        };
        let mut node = Node::new("ActionUsage", "StateSubactionMembership");
        node.membership_kind = Some(kind_word);
        if self.take_symbol(";") {
            return Ok(node);
        }
        self.performed_action(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `action a [spec]` | `x [spec]` (a performance) | `accept ...` | `send ...` | `assign ...`
    fn performed_action(&mut self, node: &mut Node) -> Parsed<()> {
        if self.is_word("accept") || (self.is_word("action") && self.accept_follows()) {
            self.accept_declaration(node)?;
            return Ok(());
        }
        if self.is_word("send") || (self.is_word("action") && self.word_follows_declaration("send")) {
            self.send_declaration(node)?;
            return Ok(());
        }
        if self.is_word("assign") || (self.is_word("action") && self.word_follows_declaration("assign")) {
            self.assign_declaration(node)?;
            return Ok(());
        }
        node.kind = "PerformActionUsage";
        if self.take_word("action") {
            self.usage_declaration(node)?;
        } else {
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Reference, target, conjugated: false });
            self.feature_specialization_part(node)?;
        }
        self.value_part(node)
    }

    fn accept_follows(&self) -> bool {
        self.word_follows_declaration("accept")
    }

    /// Whether `action [decl] WORD` follows: an action node named before its keyword.
    pub(super) fn word_follows_declaration(&self, word: &str) -> bool {
        let mut i = self.position + 1;
        let mut depth = 0;
        while let Some(token) = self.tokens.get(i) {
            match &token.kind {
                Kind::Word(w) if w == word && depth == 0 => return true,
                Kind::Symbol("[") => depth += 1,
                Kind::Symbol("]") => depth -= 1,
                Kind::Symbol(";") | Kind::Symbol("{") | Kind::Symbol("}") | Kind::Symbol("=") => return false,
                Kind::Word(w) if depth == 0 && super::RESERVED.contains(&w.as_str()) && !matches!(w.as_str(), "ordered" | "nonunique" | "defined" | "by" | "subsets" | "redefines" | "references") => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// `[action [decl]] accept payload [via x]`
    fn accept_declaration(&mut self, node: &mut Node) -> Parsed<()> {
        node.kind = "AcceptActionUsage";
        if self.take_word("action") {
            self.usage_declaration(node)?;
        }
        self.expect_word("accept")?;
        let mut payload = Node::new("ReferenceUsage", "ParameterMembership");
        self.accept_payload(&mut payload)?;
        let via = if self.take_word("via") { Some(self.expression()?) } else { None };
        node.detail = Detail::Accept { payload: Box::new(payload), via };
        Ok(())
    }

    /// `Payload` | `[id] [spec] (at|after|when) e`
    fn accept_payload(&mut self, payload: &mut Node) -> Parsed<()> {
        let trigger_here = |p: &Self| p.is_word("at") || p.is_word("after") || p.is_word("when");
        if trigger_here(self) {
            return self.trigger(payload);
        }
        self.payload(payload)?;
        if trigger_here(self) {
            return self.trigger(payload);
        }
        Ok(())
    }

    fn trigger(&mut self, payload: &mut Node) -> Parsed<()> {
        let kind = match self.advance() {
            Some(Kind::Word(w)) => w,
            _ => unreachable!("a trigger keyword was seen"),
        };
        let expr = self.expression()?;
        payload.detail = Detail::Trigger(Trigger { kind, expr });
        Ok(())
    }

    /// `[action [decl]] send [payload] [via x] [to y]`
    fn send_declaration(&mut self, node: &mut Node) -> Parsed<()> {
        node.kind = "SendActionUsage";
        if self.take_word("action") {
            self.usage_declaration(node)?;
        }
        self.expect_word("send")?;
        let payload = if self.is_word("via") || self.is_word("to") || self.is_symbol(";") || self.is_symbol("{") { None } else { Some(self.expression()?) };
        let via = if self.take_word("via") { Some(self.expression()?) } else { None };
        let to = if self.take_word("to") { Some(self.expression()?) } else { None };
        node.detail = Detail::Send { payload, via, to };
        Ok(())
    }

    /// `[action [decl]] assign target := value`
    fn assign_declaration(&mut self, node: &mut Node) -> Parsed<()> {
        node.kind = "AssignmentActionUsage";
        if self.take_word("action") {
            self.usage_declaration(node)?;
        }
        self.expect_word("assign")?;
        let target = self.primary_expression()?;
        self.expect_symbol(":=")?;
        let value = self.expression()?;
        node.detail = Detail::Assign { target, value };
        Ok(())
    }

    pub(super) fn accept_node(&mut self, mut node: Node) -> Parsed<Node> {
        self.accept_declaration(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    pub(super) fn send_node(&mut self, mut node: Node) -> Parsed<Node> {
        self.send_declaration(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    pub(super) fn assign_node(&mut self, mut node: Node) -> Parsed<Node> {
        self.assign_declaration(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `if c { } [else { } | else if ...]`, or `if g then target;` (a guarded succession).
    pub(super) fn if_node(&mut self, mut node: Node) -> Parsed<Node> {
        self.expect_word("if")?;
        let condition = self.expression()?;
        if self.is_word("then") {
            return self.guarded_succession(node, condition);
        }
        node.kind = "IfActionUsage";
        let then = self.action_body_parameter()?;
        let otherwise = if self.take_word("else") {
            if self.is_word("if") {
                let nested = Node::new("ActionUsage", "ParameterMembership");
                Some(Box::new(self.if_node(nested)?))
            } else {
                Some(Box::new(self.action_body_parameter()?))
            }
        } else {
            None
        };
        node.detail = Detail::If { condition, then: Box::new(then), otherwise };
        Ok(node)
    }

    /// `[action [decl]] { body }`: the body of an if, while or for.
    fn action_body_parameter(&mut self) -> Parsed<Node> {
        let mut body = Node::new("ActionUsage", "ParameterMembership");
        if self.take_word("action") {
            self.usage_declaration(&mut body)?;
        }
        self.expect_symbol("{")?;
        self.position -= 1;
        body.members = self.body_in(false, false)?;
        Ok(body)
    }

    /// `while c { } [until e;]` | `loop { } [until e;]`
    pub(super) fn while_node(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "WhileLoopActionUsage";
        let condition = if self.take_word("while") {
            Some(self.expression()?)
        } else {
            self.expect_word("loop")?;
            None
        };
        let body = Box::new(self.action_body_parameter()?);
        let until = if self.take_word("until") {
            let e = self.expression()?;
            self.expect_symbol(";")?;
            Some(e)
        } else {
            None
        };
        node.detail = Detail::While { condition, body, until };
        Ok(node)
    }

    /// `for x [: T] in c { }`
    pub(super) fn for_node(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "ForLoopActionUsage";
        self.expect_word("for")?;
        let mut variable = Node::new("ReferenceUsage", "FeatureMembership");
        self.usage_declaration(&mut variable)?;
        self.expect_word("in")?;
        let collection = self.expression()?;
        let body = Box::new(self.action_body_parameter()?);
        node.detail = Detail::For { variable: Box::new(variable), collection, body };
        Ok(node)
    }

    /// `terminate [x] body`
    pub(super) fn terminate_node(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "TerminateActionUsage";
        if self.take_word("action") {
            self.usage_declaration(&mut node)?;
        }
        self.expect_word("terminate")?;
        let target = if self.is_symbol(";") || self.is_symbol("{") { None } else { Some(self.expression()?) };
        node.detail = Detail::Terminate(target);
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `merge|decide|join|fork [decl] body`
    pub(super) fn control_node(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = match self.advance() {
            Some(Kind::Word(w)) if w == "merge" => "MergeNode",
            Some(Kind::Word(w)) if w == "decide" => "DecisionNode",
            Some(Kind::Word(w)) if w == "join" => "JoinNode",
            _ => "ForkNode",
        };
        self.usage_declaration(&mut node)?;
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `transition [decl first] source [accept ...] [if g] [do e] then target body`,
    /// and a target transition after a state: `transition|accept|if|do ... then target`.
    pub(super) fn transition(&mut self, mut node: Node) -> Parsed<Node> {
        node.kind = "TransitionUsage";
        self.expect_word("transition")?;
        let mut source = None;
        if self.is_word("first") {
            self.position += 1;
            source = Some(self.target()?);
        } else if !(self.is_word("accept") || self.is_word("if") || self.is_word("do") || self.is_word("then")) {
            // `transition t first a ...` names the transition before `first`;
            // `transition a then b` names the source.
            let save = self.position;
            let mut probe = Node::default();
            let declared = self.is_symbol("<") || self.is_name();
            if declared {
                self.usage_declaration(&mut probe)?;
            }
            if declared && self.take_word("first") {
                node.short_name = probe.short_name;
                node.name = probe.name;
                node.specializations = probe.specializations;
                source = Some(self.target()?);
            } else {
                self.position = save;
                source = Some(self.target()?);
            }
        }
        let (trigger, guard, effect) = self.transition_parts()?;
        self.expect_word("then")?;
        let target = self.connector_end()?;
        node.detail = Detail::Transition { source, trigger, guard, effect, target, is_else: false };
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `[accept payload [via x]] [if g] [do effect]`
    fn transition_parts(&mut self) -> Parsed<(Option<Box<Node>>, Option<Expr>, Option<Box<Node>>)> {
        let mut trigger = None;
        if self.take_word("accept") {
            let mut payload = Node::new("ReferenceUsage", "ParameterMembership");
            self.accept_payload(&mut payload)?;
            let via = if self.take_word("via") { Some(self.expression()?) } else { None };
            let mut accept = Node::new("AcceptActionUsage", "TransitionFeatureMembership");
            accept.membership_kind = Some("trigger");
            accept.detail = Detail::Accept { payload: Box::new(payload), via };
            trigger = Some(Box::new(accept));
        }
        let guard = if self.take_word("if") { Some(self.expression()?) } else { None };
        let mut effect = None;
        if self.take_word("do") {
            let mut action = Node::new("ActionUsage", "TransitionFeatureMembership");
            action.membership_kind = Some("effect");
            if !self.is_word("then") {
                self.performed_action(&mut action)?;
                if self.is_symbol("{") {
                    action.members = self.body_in(false, false)?;
                }
            }
            effect = Some(Box::new(action));
        }
        Ok((trigger, guard, effect))
    }

    /// In a state body, `accept ... then t;`, `do ... then t;` start a target
    /// transition (the source is the previous state).
    pub(super) fn target_transition(&mut self) -> Parsed<Node> {
        let mut node = Node::new("TransitionUsage", "FeatureMembership");
        self.take_word("transition");
        let (trigger, guard, effect) = self.transition_parts()?;
        self.expect_word("then")?;
        let target = self.connector_end()?;
        node.detail = Detail::Transition { source: None, trigger, guard, effect, target, is_else: false };
        self.usage_body(&mut node)?;
        Ok(node)
    }

    /// `@T;`, `@T { x = 1; }`, `metadata m : T about a, b { }`, `#T` handled elsewhere.
    pub(super) fn metadata_usage(&mut self, metadata: Vec<Node>) -> Parsed<Node> {
        let line = self.line();
        let mut node = Node::new("MetadataUsage", "OwningMembership");
        node.metadata = metadata;
        node.line = line;
        if !self.take_symbol("@") {
            self.expect_word("metadata")?;
        }
        // `metadata <s> m : T` names the usage; `@T` types it.
        if self.is_symbol("<") || (self.is_name() && (self.is_symbol_at(1, ":") || (self.is_word_at(1, "defined") && self.is_word_at(2, "by")))) {
            self.identification(&mut node)?;
            if !self.take_symbol(":") {
                self.expect_word("defined")?;
                self.expect_word("by")?;
            }
        }
        let ty = self.qualified_name()?;
        node.specializations.push(Specialization { kind: SpecKind::Typing, target: Target::Name(ty), conjugated: false });
        let mut about = Vec::new();
        if self.take_word("about") {
            about.push(self.qualified_name()?);
            while self.take_symbol(",") {
                about.push(self.qualified_name()?);
            }
        }
        if !about.is_empty() {
            node.detail = Detail::Annotation { about, locale: None, language: None, body: String::new() };
        }
        node.members = self.metadata_body()?;
        Ok(node)
    }

    /// The body of a metadata usage: `x = 1;` redefines the feature `x`.
    pub(super) fn metadata_body(&mut self) -> Parsed<Vec<Node>> {
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
            // Definitions, aliases, imports and annotations as anywhere.
            let bare_feature = self.is_name() || self.is_symbol(":>>") || self.is_word("ref") || self.is_word("redefines");
            if !bare_feature {
                members.push(self.statement()?);
                continue;
            }
            let mut node = Node::new("ReferenceUsage", "FeatureMembership");
            node.line = line;
            self.take_word("ref");
            if !self.take_symbol(":>>") {
                self.take_word("redefines");
            }
            let target = self.target()?;
            node.specializations.push(Specialization { kind: SpecKind::Redefinition, target, conjugated: false });
            self.feature_specialization_part(&mut node)?;
            self.value_part(&mut node)?;
            node.members = self.metadata_body()?;
            members.push(node);
        }
        self.position += 1;
        Ok(members)
    }
}
