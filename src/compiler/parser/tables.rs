//! `table "Name" { kind: requirements_coverage scope: "SA" columns: [...] }`:
//! a live table of the model (see `compiler::tables`).

use super::Parser;
use crate::compiler::lexer::Token;
use crate::compiler::tables::{check_declaration, TableDecl};

impl Parser {
    /// Parse: table "Name" { kind: <kind> ... }
    pub(super) fn parse_table(&mut self) -> Result<TableDecl, String> {
        if !matches!(self.current(), Token::Identifier(word) if word == "table") {
            return Err(self.err(format!("expected `table`, found {}", self.current())));
        }
        self.advance();
        let name = self.expect_name()?;
        let attributes = self.parse_attributes_block()?;
        let kind = attributes.get("kind").and_then(|v| v.as_string()).map(str::to_string).ok_or_else(|| {
            self.err(format!("table '{name}' needs `kind:` (one of {})", crate::compiler::tables::KINDS.join(", ")))
        })?;
        let id = attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_else(|| name.clone());
        let decl = TableDecl { name, id, kind, attributes };
        check_declaration(&decl).map_err(|e| self.err(e))?;
        Ok(decl)
    }
}
