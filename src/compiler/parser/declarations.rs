//! Capability and class declarations: the blocks whose content is more than
//! an unordered set of attributes. Capabilities nest; class fields keep the
//! order they were written in.

use super::Parser;
use crate::compiler::ast::{
    AttributeValue, Capability, CapabilityLevel, ClassDef, DataAttribute, OperationalCapability,
};
use crate::compiler::lexer::Token;
use std::collections::HashMap;

/// Attributes of a class that describe the class itself; every other
/// attribute is a field.
const CLASS_ATTRIBUTES: [&str; 2] = ["id", "description"];

/// One capability block as written, with the capabilities declared inside.
struct CapabilityBlock {
    id: String,
    /// Whether the author wrote the id; a default one is derived.
    explicit_id: bool,
    name: String,
    attributes: HashMap<String, AttributeValue>,
    nested: Vec<CapabilityBlock>,
}

fn nests_operational(token: &Token) -> bool {
    matches!(token, Token::Identifier(word) if word == "operational_capability")
}

fn nests_capability(token: &Token) -> bool {
    matches!(token, Token::Capability | Token::CapabilityRealization)
}

fn text(attributes: &HashMap<String, AttributeValue>, key: &str) -> Option<String> {
    attributes
        .get(key)
        .and_then(|value| value.as_string())
        .map(str::to_string)
}

impl Parser {
    /// Parse: operational_capability Name { id: ... involves: [..]
    /// operational_capability Nested { ... } }.
    ///
    /// Returns the capability followed by every capability nested in it,
    /// each one naming its parent: a consumer that walks the list sees all
    /// of them, and one that draws containment has what it needs.
    pub(super) fn parse_operational_capabilities(
        &mut self,
    ) -> Result<Vec<OperationalCapability>, String> {
        let block = self.capability_block("OC", nests_operational)?;
        let mut declared = Vec::new();
        flatten(block, None, &mut |block, parent| {
            let level = match parent {
                Some(_) => CapabilityLevel::SubCapability,
                None => CapabilityLevel::Capability,
            };
            declared.push(OperationalCapability {
                id: block.id,
                name: block.name,
                level,
                color: None,
                stereotype: None,
                children: Vec::new(),
                parent,
                involves: Self::string_list(&block.attributes, "involves"),
                attributes: block.attributes,
            });
        });
        Ok(declared)
    }

    /// Parse: capability Name { id: ... involves: [..] realizes: "..."
    /// mission: "..." capability Nested { ... } }. Also used for logical
    /// `capability_realization` blocks. Nested capabilities are returned
    /// after their parent, each one naming it.
    pub(super) fn parse_capabilities(&mut self) -> Result<Vec<Capability>, String> {
        let block = self.capability_block("CAP", nests_capability)?;
        let mut declared = Vec::new();
        flatten(block, None, &mut |block, parent| {
            declared.push(Capability {
                id: block.id,
                name: block.name,
                involves: Self::string_list(&block.attributes, "involves"),
                realizes: text(&block.attributes, "realizes"),
                mission: text(&block.attributes, "mission"),
                parent,
                attributes: block.attributes,
            });
        });
        Ok(declared)
    }

    fn capability_block(
        &mut self,
        id_prefix: &str,
        nests: fn(&Token) -> bool,
    ) -> Result<CapabilityBlock, String> {
        self.advance(); // Skip the capability keyword
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;

        let mut attributes = HashMap::new();
        let mut nested = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // `capability: "x"` is an attribute; `capability Name {` nests.
            if nests(self.current()) && !self.peek_is_colon() {
                nested.push(self.capability_block(id_prefix, nests)?);
                continue;
            }
            let (key, value) = self.parse_attribute()?;
            if attributes.contains_key(&key) {
                return Err(self.err(format!(
                    "capability '{name}' declares attribute '{key}' twice"
                )));
            }
            attributes.insert(key, value);
        }
        self.expect(Token::RightBrace)?;
        self.require_text_id("capability", &name, &attributes)?;

        let id = text(&attributes, "id")
            .unwrap_or_else(|| format!("{id_prefix}-{}", name.replace(' ', "_")));
        Ok(CapabilityBlock {
            id,
            name,
            explicit_id: attributes.contains_key("id"),
            attributes,
            nested,
        })
    }

    /// An `id` that is not text would be ignored and a default one used.
    fn require_text_id(
        &self,
        what: &str,
        name: &str,
        attributes: &HashMap<String, AttributeValue>,
    ) -> Result<(), String> {
        match attributes.get("id") {
            None | Some(AttributeValue::String(_)) => Ok(()),
            Some(other) => Err(self.err(format!(
                "{what} '{name}': id must be text, got '{}'",
                other.display()
            ))),
        }
    }

    /// Parse: class Name { id: "..." speed: "float" ... } — Arcadia Class.
    /// Every attribute but `id` and `description` is a field, kept in the
    /// order it was declared, as UML and Capella show them.
    pub(super) fn parse_class(&mut self) -> Result<ClassDef, String> {
        self.expect(Token::Class)?;
        let name = self.expect_name()?;
        self.expect(Token::LeftBrace)?;

        let mut attributes = HashMap::new();
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let (key, value) = self.parse_attribute()?;
            let is_field = !CLASS_ATTRIBUTES.contains(&key.as_str());
            if attributes.contains_key(&key) {
                let what = if is_field { "field" } else { "attribute" };
                return Err(self.err(format!("class '{name}' declares {what} '{key}' twice")));
            }
            if is_field {
                let Some(field_type) = value.as_string() else {
                    return Err(self.err(format!(
                        "class '{name}': field '{key}' must name its type as text \
                         (`{key}: \"float\"`)"
                    )));
                };
                fields.push(DataAttribute {
                    name: key.clone(),
                    attr_type: field_type.to_string(),
                    default_value: None,
                    enumeration: None,
                });
            }
            attributes.insert(key, value);
        }
        self.expect(Token::RightBrace)?;
        self.require_text_id("class", &name, &attributes)?;

        let id = text(&attributes, "id").unwrap_or_else(|| name.clone());
        Ok(ClassDef {
            id,
            name,
            fields,
            attributes,
        })
    }
}

/// Visit a block, then the blocks nested in it, handing each one the id of
/// the block that contains it. A nested block without an id of its own is
/// identified under its parent (`CAP-Parent/Child`): the same name under
/// two parents is two capabilities.
fn flatten(
    block: CapabilityBlock,
    parent: Option<String>,
    declare: &mut impl FnMut(CapabilityBlock, Option<String>),
) {
    let CapabilityBlock {
        id,
        explicit_id,
        name,
        attributes,
        nested,
    } = block;
    let id = match (&parent, explicit_id) {
        (Some(parent), false) => format!("{parent}/{}", name.replace(' ', "_")),
        _ => id,
    };
    let own_id = id.clone();
    declare(
        CapabilityBlock {
            id,
            explicit_id,
            name,
            attributes,
            nested: Vec::new(),
        },
        parent,
    );
    for child in nested {
        flatten(child, Some(own_id.clone()), declare);
    }
}
