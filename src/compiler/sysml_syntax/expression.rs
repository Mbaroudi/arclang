//! Expressions of the exported SysML v2 text: feature values and constraint
//! bodies, parsed with the precedence of the notation.

use super::*;

/// An expression of the export: a feature value or a constraint body.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Rational(f64),
    Text(String),
    Boolean(bool),
    /// A name: `ms`, `a_SF_1`.
    Reference(String),
    /// `a.b.c`: the feature `b.c` of what `a` designates.
    Chain(Box<Expr>, String),
    /// An operator and its operands. `[` is the quantity operator
    /// (`25 [ms]`); `-` with one operand is negation.
    Operator(&'static str, Vec<Expr>),
    /// `Package::function(arguments)`.
    Invocation(String, Vec<Expr>),
}

const COMPARISONS: &[&str] = &["<=", ">=", "==", "!=", "<", ">"];

/// Recursive-descent parser over the tokens of one expression.
pub(super) struct Expressions<'t> {
    tokens: &'t [Token],
    position: usize,
}

impl<'t> Expressions<'t> {
    pub(super) fn parse(tokens: &'t [Token]) -> Result<Expr, String> {
        let mut parser = Expressions { tokens, position: 0 };
        let expression = parser.comparison()?;
        match parser.tokens.get(parser.position) {
            None => Ok(expression),
            Some(_) => Err(format!("unread `{}` in expression `{}`", expression_text(&tokens[parser.position..]), expression_text(tokens))),
        }
    }

    fn operator(&mut self, candidates: &[&'static str]) -> Option<&'static str> {
        let Some(Token::Symbol(symbol)) = self.tokens.get(self.position) else { return None };
        let found = candidates.iter().find(|candidate| *candidate == symbol)?;
        self.position += 1;
        Some(found)
    }

    fn expect(&mut self, symbol: &'static str) -> Result<(), String> {
        match self.operator(&[symbol]) {
            Some(_) => Ok(()),
            None => Err(format!("expected `{}` in expression `{}`", symbol, expression_text(self.tokens))),
        }
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let left = self.additive()?;
        match self.operator(COMPARISONS) {
            Some(operator) => Ok(Expr::Operator(operator, vec![left, self.additive()?])),
            None => Ok(left),
        }
    }

    fn additive(&mut self) -> Result<Expr, String> {
        let mut left = self.multiplicative()?;
        while let Some(operator) = self.operator(&["+", "-"]) {
            left = Expr::Operator(operator, vec![left, self.multiplicative()?]);
        }
        Ok(left)
    }

    fn multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        while let Some(operator) = self.operator(&["*", "/"]) {
            left = Expr::Operator(operator, vec![left, self.unary()?]);
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        match self.operator(&["-"]) {
            Some(operator) => Ok(Expr::Operator(operator, vec![self.unary()?])),
            None => self.quantity(),
        }
    }

    /// `value [unit]`
    fn quantity(&mut self) -> Result<Expr, String> {
        let mut value = self.primary()?;
        while self.operator(&["["]).is_some() {
            let unit = self.comparison()?;
            self.expect("]")?;
            value = Expr::Operator("[", vec![value, unit]);
        }
        Ok(value)
    }

    fn name(&mut self) -> Option<String> {
        let name = match self.tokens.get(self.position)? {
            Token::Word(word) => word.clone(),
            Token::Quoted(name) => name.clone(),
            _ => return None,
        };
        self.position += 1;
        Some(name)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let unexpected = |tokens: &[Token]| format!("unknown expression `{}`", expression_text(tokens));
        let token = self.tokens.get(self.position).ok_or_else(|| unexpected(self.tokens))?;
        match token {
            Token::Number(text) => {
                self.position += 1;
                // The notation tells integers from rationals by the point.
                if text.contains('.') {
                    text.parse().map(Expr::Rational).map_err(|_| unexpected(self.tokens))
                } else {
                    text.parse().map(Expr::Integer).map_err(|_| unexpected(self.tokens))
                }
            }
            Token::Text(raw) => {
                self.position += 1;
                let mut text = String::new();
                let mut chars = raw[1..raw.len() - 1].chars();
                while let Some(c) = chars.next() {
                    text.push(if c == '\\' { chars.next().unwrap_or(c) } else { c });
                }
                Ok(Expr::Text(text))
            }
            Token::Symbol("(") => {
                self.position += 1;
                let inner = self.comparison()?;
                self.expect(")")?;
                Ok(inner)
            }
            Token::Word(word) if word == "true" || word == "false" => {
                self.position += 1;
                Ok(Expr::Boolean(word == "true"))
            }
            Token::Word(_) | Token::Quoted(_) => {
                let mut path = self.name().ok_or_else(|| unexpected(self.tokens))?;
                while self.operator(&["::"]).is_some() {
                    let segment = self.name().ok_or_else(|| unexpected(self.tokens))?;
                    path = format!("{}::{}", path, segment);
                }
                if self.operator(&["("]).is_some() {
                    let mut arguments = Vec::new();
                    if self.operator(&[")"]).is_none() {
                        loop {
                            arguments.push(self.comparison()?);
                            if self.operator(&[","]).is_none() {
                                break;
                            }
                        }
                        self.expect(")")?;
                    }
                    return Ok(Expr::Invocation(path, arguments));
                }
                let mut features = Vec::new();
                while self.operator(&["."]).is_some() {
                    features.push(self.name().ok_or_else(|| unexpected(self.tokens))?);
                }
                let base = Expr::Reference(path);
                Ok(if features.is_empty() { base } else { Expr::Chain(Box::new(base), features.join(".")) })
            }
            _ => Err(unexpected(self.tokens)),
        }
    }
}
