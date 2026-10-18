//! Expressions (KerMLExpressions.xtext), by precedence: conditional,
//! null-coalescing, implies, or, xor, and, equality, classification,
//! relational, range, additive, multiplicative, exponentiation, unary,
//! extent, primary (chains, indexes, arrows, collect, select).

use super::{Parsed, Parser};
use crate::compiler::sysml_parser::lexer::Kind;
use crate::compiler::sysml_parser::syntax::*;

impl Parser {
    pub(super) fn expression(&mut self) -> Parsed<Expr> {
        self.conditional()
    }

    fn binary(operator: &str, lhs: Expr, rhs: Expr) -> Expr {
        Expr::Operator { operator: operator.to_string(), operands: vec![lhs, rhs] }
    }

    /// `if c ? a else b`
    fn conditional(&mut self) -> Parsed<Expr> {
        if self.is_word("if") {
            self.position += 1;
            let condition = self.null_coalescing()?;
            self.expect_symbol("?")?;
            let then = self.expression()?;
            self.expect_word("else")?;
            let otherwise = self.expression()?;
            return Ok(Expr::Conditional { condition: Box::new(condition), then: Box::new(then), otherwise: Box::new(otherwise) });
        }
        self.null_coalescing()
    }

    fn null_coalescing(&mut self) -> Parsed<Expr> {
        let mut lhs = self.implies()?;
        while self.take_symbol("??") {
            let rhs = self.implies()?;
            lhs = Self::binary("??", lhs, rhs);
        }
        Ok(lhs)
    }

    fn implies(&mut self) -> Parsed<Expr> {
        let mut lhs = self.or()?;
        while self.take_word("implies") {
            let rhs = self.or()?;
            lhs = Self::binary("implies", lhs, rhs);
        }
        Ok(lhs)
    }

    fn or(&mut self) -> Parsed<Expr> {
        let mut lhs = self.xor()?;
        loop {
            if self.take_symbol("|") {
                let rhs = self.xor()?;
                lhs = Self::binary("|", lhs, rhs);
            } else if self.take_word("or") {
                let rhs = self.xor()?;
                lhs = Self::binary("or", lhs, rhs);
            } else {
                return Ok(lhs);
            }
        }
    }

    fn xor(&mut self) -> Parsed<Expr> {
        let mut lhs = self.and()?;
        while self.take_word("xor") {
            let rhs = self.and()?;
            lhs = Self::binary("xor", lhs, rhs);
        }
        Ok(lhs)
    }

    fn and(&mut self) -> Parsed<Expr> {
        let mut lhs = self.equality()?;
        loop {
            if self.take_symbol("&") {
                let rhs = self.equality()?;
                lhs = Self::binary("&", lhs, rhs);
            } else if self.take_word("and") {
                let rhs = self.equality()?;
                lhs = Self::binary("and", lhs, rhs);
            } else {
                return Ok(lhs);
            }
        }
    }

    fn equality(&mut self) -> Parsed<Expr> {
        let mut lhs = self.classification()?;
        loop {
            let op = match self.peek() {
                Some(Kind::Symbol(s)) if matches!(*s, "==" | "!=" | "===" | "!==") => *s,
                _ => return Ok(lhs),
            };
            self.position += 1;
            let rhs = self.classification()?;
            lhs = Self::binary(op, lhs, rhs);
        }
    }

    /// `a istype T`, `a hastype T`, `a @ T`, `a @@ T`, `a as T`, `a meta T`,
    /// and the forms on the implicit self (`istype T`, `@ T`, `as T`).
    fn classification(&mut self) -> Parsed<Expr> {
        let test_op = |p: &Self| -> Option<&'static str> {
            match p.peek() {
                Some(Kind::Word(w)) if w == "istype" => Some("istype"),
                Some(Kind::Word(w)) if w == "hastype" => Some("hastype"),
                Some(Kind::Symbol("@")) => Some("@"),
                Some(Kind::Symbol("@@")) => Some("@@"),
                _ => None,
            }
        };
        let cast_op = |p: &Self| -> Option<&'static str> {
            match p.peek() {
                Some(Kind::Word(w)) if w == "as" => Some("as"),
                Some(Kind::Word(w)) if w == "meta" => Some("meta"),
                _ => None,
            }
        };
        // Operator first: the operand is the implicit self.
        if let Some(op) = test_op(self) {
            self.position += 1;
            let ty = self.qualified_name()?;
            return Ok(Expr::TypeTest { operand: None, operator: op.to_string(), ty });
        }
        if let Some(op) = cast_op(self) {
            self.position += 1;
            let ty = self.qualified_name()?;
            return Ok(Expr::Cast { operand: None, operator: op.to_string(), ty });
        }
        let operand = self.relational()?;
        if let Some(op) = test_op(self) {
            self.position += 1;
            let ty = self.qualified_name()?;
            return Ok(Expr::TypeTest { operand: Some(Box::new(operand)), operator: op.to_string(), ty });
        }
        if let Some(op) = cast_op(self) {
            self.position += 1;
            let ty = self.qualified_name()?;
            return Ok(Expr::Cast { operand: Some(Box::new(operand)), operator: op.to_string(), ty });
        }
        Ok(operand)
    }

    fn relational(&mut self) -> Parsed<Expr> {
        let mut lhs = self.range()?;
        loop {
            let op = match self.peek() {
                Some(Kind::Symbol(s)) if matches!(*s, "<" | ">" | "<=" | ">=") => *s,
                _ => return Ok(lhs),
            };
            self.position += 1;
            let rhs = self.range()?;
            lhs = Self::binary(op, lhs, rhs);
        }
    }

    fn range(&mut self) -> Parsed<Expr> {
        let lhs = self.additive()?;
        if self.take_symbol("..") {
            let rhs = self.additive()?;
            return Ok(Self::binary("..", lhs, rhs));
        }
        Ok(lhs)
    }

    fn additive(&mut self) -> Parsed<Expr> {
        let mut lhs = self.multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Kind::Symbol(s)) if matches!(*s, "+" | "-") => *s,
                _ => return Ok(lhs),
            };
            self.position += 1;
            let rhs = self.multiplicative()?;
            lhs = Self::binary(op, lhs, rhs);
        }
    }

    fn multiplicative(&mut self) -> Parsed<Expr> {
        let mut lhs = self.exponentiation()?;
        loop {
            let op = match self.peek() {
                Some(Kind::Symbol(s)) if matches!(*s, "*" | "/" | "%") => *s,
                _ => return Ok(lhs),
            };
            self.position += 1;
            let rhs = self.exponentiation()?;
            lhs = Self::binary(op, lhs, rhs);
        }
    }

    fn exponentiation(&mut self) -> Parsed<Expr> {
        let lhs = self.unary()?;
        let op = match self.peek() {
            Some(Kind::Symbol(s)) if matches!(*s, "**" | "^") => *s,
            _ => return Ok(lhs),
        };
        self.position += 1;
        let rhs = self.exponentiation()?;
        Ok(Self::binary(op, lhs, rhs))
    }

    fn unary(&mut self) -> Parsed<Expr> {
        let op = match self.peek() {
            Some(Kind::Symbol(s)) if matches!(*s, "+" | "-" | "~") => Some(s.to_string()),
            Some(Kind::Word(w)) if w == "not" => Some("not".to_string()),
            _ => None,
        };
        if let Some(op) = op {
            self.position += 1;
            let operand = self.extent()?;
            return Ok(Expr::Operator { operator: op, operands: vec![operand] });
        }
        self.extent()
    }

    /// `all T`
    fn extent(&mut self) -> Parsed<Expr> {
        if self.take_word("all") {
            let ty = self.qualified_name()?;
            return Ok(Expr::Extent(ty));
        }
        self.primary_expression()
    }

    /// A base expression and its chains, indexes, arrows, collects and selects.
    pub(super) fn primary_expression(&mut self) -> Parsed<Expr> {
        let mut expr = self.base_expression()?;
        loop {
            if self.is_symbol(".") && self.is_name_at(1) {
                self.position += 1;
                let chain = self.target()?;
                expr = Expr::Chain { base: Box::new(expr), chain };
                continue;
            }
            if self.is_symbol(".") && self.is_symbol_at(1, "{") {
                self.position += 1;
                let body = self.body_expression()?;
                expr = Expr::Collect { base: Box::new(expr), body: Box::new(body) };
                continue;
            }
            if self.take_symbol(".?") {
                let body = self.body_expression()?;
                expr = Expr::Select { base: Box::new(expr), body: Box::new(body) };
                continue;
            }
            if self.is_symbol("#") && self.is_symbol_at(1, "(") {
                self.position += 2;
                let index = self.sequence_expression()?;
                self.expect_symbol(")")?;
                expr = Expr::Index { base: Box::new(expr), index: Box::new(index) };
                continue;
            }
            if self.take_symbol("[") {
                let index = self.sequence_expression()?;
                self.expect_symbol("]")?;
                expr = Expr::Bracket { base: Box::new(expr), index: Box::new(index) };
                continue;
            }
            if self.take_symbol("->") {
                let ty = self.target()?;
                let argument = if self.is_symbol("{") {
                    ArrowArgument::Body(Box::new(self.body_expression()?))
                } else if self.is_symbol("(") {
                    ArrowArgument::Arguments(self.argument_list()?)
                } else {
                    ArrowArgument::Function(self.qualified_name()?)
                };
                expr = Expr::Arrow { base: Box::new(expr), ty, argument };
                continue;
            }
            return Ok(expr);
        }
    }

    /// `null`, `()`, literals, `x`, `x.metadata`, `f(args)`, `new T(args)`,
    /// `{ body }`, `(a, b)`.
    fn base_expression(&mut self) -> Parsed<Expr> {
        match self.peek().cloned() {
            Some(Kind::Word(w)) if w == "null" => {
                self.position += 1;
                Ok(Expr::Null)
            }
            Some(Kind::Word(w)) if w == "true" || w == "false" => {
                self.position += 1;
                Ok(Expr::Literal(Literal::Boolean(w == "true")))
            }
            Some(Kind::Word(w)) if w == "new" => {
                self.position += 1;
                let ty = self.target()?;
                let arguments = self.argument_list()?;
                Ok(Expr::Constructor { ty, arguments })
            }
            Some(Kind::Str(s)) => {
                self.position += 1;
                Ok(Expr::Literal(Literal::String(s)))
            }
            Some(Kind::Integer(n)) => {
                self.position += 1;
                Ok(Expr::Literal(Literal::Integer(n)))
            }
            Some(Kind::Real(n)) => {
                self.position += 1;
                Ok(Expr::Literal(Literal::Real(n)))
            }
            Some(Kind::Symbol("*")) => {
                self.position += 1;
                Ok(Expr::Literal(Literal::Infinity))
            }
            Some(Kind::Symbol("{")) => self.body_expression(),
            Some(Kind::Symbol("(")) => {
                self.position += 1;
                if self.take_symbol(")") {
                    return Ok(Expr::Null);
                }
                let inner = self.sequence_expression()?;
                self.expect_symbol(")")?;
                Ok(inner)
            }
            _ if self.is_name() || (self.is_symbol("$") && self.is_symbol_at(1, "::")) => {
                let name = self.qualified_name()?;
                // `x.metadata`
                if self.is_symbol(".") && self.is_word_at(1, "metadata") {
                    self.position += 2;
                    return Ok(Expr::MetadataAccess(name));
                }
                // `f(args)`, `a.b(args)`
                let mut steps = vec![name];
                while self.is_symbol(".") && self.is_name_at(1) && self.invocation_follows_chain() {
                    self.position += 1;
                    steps.push(self.qualified_name()?);
                }
                if self.is_symbol("(") {
                    let arguments = self.argument_list()?;
                    let function = if steps.len() == 1 { Target::Name(steps.pop().unwrap()) } else { Target::Chain(steps) };
                    return Ok(Expr::Invocation { function, arguments });
                }
                let name = steps.remove(0);
                let mut expr = Expr::Reference(name);
                if !steps.is_empty() {
                    expr = Expr::Chain { base: Box::new(expr), chain: if steps.len() == 1 { Target::Name(steps.pop().unwrap()) } else { Target::Chain(steps) } };
                }
                Ok(expr)
            }
            _ => Err(self.error("an expression")),
        }
    }

    /// After `a`, whether `.b.c(` follows: an invocation of a feature chain.
    fn invocation_follows_chain(&self) -> bool {
        let mut i = self.position;
        while let Some(token) = self.tokens.get(i) {
            match &token.kind {
                Kind::Symbol(".") => {}
                Kind::Symbol("::") => {}
                Kind::Word(_) | Kind::Name(_) => {}
                Kind::Symbol("(") => return true,
                _ => return false,
            }
            i += 1;
        }
        false
    }

    /// `{ in x; in y; result }`
    fn body_expression(&mut self) -> Parsed<Expr> {
        self.expect_symbol("{")?;
        let mut parameters = Vec::new();
        while self.is_word("in") {
            self.position += 1;
            parameters.push(self.name()?);
            self.expect_symbol(";")?;
        }
        let result = self.expression()?;
        self.expect_symbol("}")?;
        Ok(Expr::Body { parameters, result: Box::new(result) })
    }

    /// `a, b, c` (a sequence) or a single expression.
    pub(super) fn sequence_expression(&mut self) -> Parsed<Expr> {
        let first = self.expression()?;
        if !self.is_symbol(",") {
            return Ok(first);
        }
        let mut items = vec![first];
        while self.take_symbol(",") {
            if self.is_symbol(")") || self.is_symbol("]") {
                break;
            }
            items.push(self.expression()?);
        }
        Ok(Expr::Sequence(items))
    }

    /// `(a, b)` or `(x = a, y = b)` or `()`
    pub(super) fn argument_list(&mut self) -> Parsed<Vec<Argument>> {
        self.expect_symbol("(")?;
        let mut arguments = Vec::new();
        if self.take_symbol(")") {
            return Ok(arguments);
        }
        loop {
            let named = self.is_name() && self.named_argument_follows();
            if named {
                let name = self.qualified_name()?;
                self.expect_symbol("=")?;
                let value = self.expression()?;
                arguments.push(Argument { name: Some(name), value });
            } else {
                let value = self.expression()?;
                arguments.push(Argument { name: None, value });
            }
            if !self.take_symbol(",") {
                break;
            }
        }
        self.expect_symbol(")")?;
        Ok(arguments)
    }

    /// Whether `name (:: name)* =` follows (and not `==`).
    fn named_argument_follows(&self) -> bool {
        let mut i = self.position;
        while let Some(token) = self.tokens.get(i) {
            match &token.kind {
                Kind::Word(_) | Kind::Name(_) | Kind::Symbol("::") => {}
                Kind::Symbol("=") => return true,
                _ => return false,
            }
            i += 1;
        }
        false
    }
}
