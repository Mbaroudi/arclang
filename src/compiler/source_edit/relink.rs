//! Where a relationship writes its ends and its name, so they can be
//! changed in place: an exchange reconnected, a transition turned around, a
//! message relabelled.
//!
//! A relationship writes its ends either on its declaration line
//! (`transition Off -> Armed`, `message "A" -> "B" "label"`, `trace "A"
//! satisfies "B"`, `functional_exchange "A.p" -> "B.p"`) or as attributes of
//! its block (`from_port: "LC-A.p"`). Both are found on the token stream;
//! the caller says how each end may be spelled and checks the compiled
//! result: nothing here is trusted on its own.

use super::*;

/// How the text may write each end of one relationship, and what it is.
pub struct RelationshipEnds<'a> {
    /// The keywords its declaration may start with (`transition`, `functional_exchange`).
    pub keywords: &'a [&'a str],
    /// The designators of its block, for the attribute form (written id, name).
    pub designators: &'a [String],
    /// Its name, when it has one: tells two relationships between the same ends apart.
    pub name: Option<&'a str>,
    pub from: &'a [String],
    pub to: &'a [String],
}

/// One written end or name: the lexeme, the text it spells (unquoted) and
/// whether it is a string literal.
#[derive(Debug, Clone, PartialEq)]
pub struct Spot {
    pub index: usize,
    pub text: String,
    pub quoted: bool,
}

/// Where a relationship writes its two ends and, if it does, its name.
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipSpots {
    pub from: Spot,
    pub to: Spot,
    pub name: Option<Spot>,
}

const SOURCE_KEYS: &[&str] = &["from", "from_port", "source"];
const TARGET_KEYS: &[&str] = &["to", "to_port", "target"];
const NAME_KEYS: &[&str] = &["label", "name", "title"];

fn spelled(lexeme: &Lexeme) -> Option<Spot> {
    match token(lexeme)? {
        Token::StringLiteral(text) => Some(Spot { index: 0, text: text.clone(), quoted: true }),
        Token::Identifier(text) => Some(Spot { index: 0, text: text.clone(), quoted: false }),
        _ => None,
    }
}

impl Document {
    fn spot(&self, index: usize) -> Option<Spot> {
        spelled(&self.lexemes[index]).map(|spot| Spot { index, ..spot })
    }

    /// The lexemes of a declaration line starting at `start`: up to its `{`,
    /// the end of the line or a comment.
    pub(super) fn line_from(&self, start: usize) -> Vec<usize> {
        let mut line = Vec::new();
        for index in start + 1..self.lexemes.len() {
            let lexeme = &self.lexemes[index];
            if lexeme.newlines_before > 0 || matches!(token(lexeme), None | Some(Token::LeftBrace | Token::RightBrace)) {
                break;
            }
            line.push(index);
        }
        line
    }

    /// The block that opens right after a declaration line, if any.
    pub(super) fn block_after(&self, line: &[usize], start: usize) -> Option<Block> {
        let next = line.last().copied().unwrap_or(start) + 1;
        let opens = self.lexemes.get(next).is_some_and(|lexeme| token(lexeme) == Some(&Token::LeftBrace) && lexeme.newlines_before == 0);
        if !opens {
            return None;
        }
        matching_close(&self.lexemes, next).map(|end| Block { open: next, close: end - 1 })
    }

    /// The relationships written on their declaration line with these ends.
    fn header_matches(&self, ends: &RelationshipEnds) -> Vec<RelationshipSpots> {
        let mut found = Vec::new();
        for start in 0..self.lexemes.len() {
            let lexeme = &self.lexemes[start];
            let starts_line = start == 0 || lexeme.newlines_before > 0 || matches!(token(&self.lexemes[start - 1]), Some(Token::LeftBrace | Token::RightBrace));
            if !starts_line || !is_word(lexeme) || !ends.keywords.contains(&lexeme.text.as_str()) {
                continue;
            }
            let line = self.line_from(start);
            let spots: Vec<Spot> = line.iter().filter_map(|index| self.spot(*index)).collect();
            let Some(from_at) = spots.iter().position(|spot| ends.from.contains(&spot.text)) else { continue };
            let Some(to_at) = spots.iter().skip(from_at + 1).position(|spot| ends.to.contains(&spot.text)).map(|at| at + from_at + 1) else { continue };
            let from = spots[from_at].clone();
            let to = spots[to_at].clone();
            let mut name = spots.iter().enumerate().find(|(at, spot)| *at != from_at && *at != to_at && spot.quoted && Some(spot.text.as_str()) == ends.name).map(|(_, spot)| spot.clone());
            if name.is_none() {
                if let Some(block) = self.block_after(&line, start) {
                    name = self.name_attribute(&block, ends.name);
                }
            }
            found.push(RelationshipSpots { from, to, name });
        }
        found
    }

    /// The `label:` (or `name:`) of a block, when it writes the name.
    fn name_attribute(&self, block: &Block, name: Option<&str>) -> Option<Spot> {
        let name = name?;
        NAME_KEYS.iter().find_map(|key| {
            let attribute = self.find_attribute(block, key)?;
            let spot = self.spot(attribute.value_start)?;
            (attribute.value_end == attribute.value_start + 1 && spot.text == name).then_some(spot)
        })
    }

    /// The relationship written as a block with its ends as attributes.
    fn attribute_match(&self, ends: &RelationshipEnds) -> Result<RelationshipSpots, EditError> {
        let mut last = EditError::NotFound(ends.designators.first().cloned().unwrap_or_default());
        for designator in ends.designators {
            let block = match self.find_declared(designator) {
                Ok(block) => block,
                Err(error) => {
                    last = error;
                    continue;
                }
            };
            let end = |keys: &[&str], spellings: &[String]| {
                keys.iter().find_map(|key| {
                    let attribute = self.find_attribute(&block, key)?;
                    let spot = self.spot(attribute.value_start)?;
                    (attribute.value_end == attribute.value_start + 1 && spellings.contains(&spot.text)).then_some(spot)
                })
            };
            if let (Some(from), Some(to)) = (end(SOURCE_KEYS, ends.from), end(TARGET_KEYS, ends.to)) {
                let header = self.header(&block);
                let header_start = block.open - header.len();
                let name = (header_start..block.open)
                    .filter_map(|index| self.spot(index))
                    .find(|spot| spot.quoted && Some(spot.text.as_str()) == ends.name)
                    .or_else(|| self.name_attribute(&block, ends.name));
                return Ok(RelationshipSpots { from, to, name });
            }
            last = EditError::Unsafe(format!("'{designator}' does not write its ends as from/to attributes this edit can change"));
        }
        Err(last)
    }
}

/// Where the relationship writes its ends: on its declaration line, else
/// as attributes of its block. Two candidates that cannot be told apart by
/// name are refused.
pub fn relationship_spots(source: &str, ends: &RelationshipEnds) -> Result<RelationshipSpots, EditError> {
    let document = Document::read(source)?;
    let mut candidates = document.header_matches(ends);
    if candidates.len() > 1 && ends.name.is_some() {
        candidates.retain(|candidate| candidate.name.is_some());
    }
    let described = || format!("{} {} -> {}", ends.keywords.first().unwrap_or(&"relationship"), ends.from.first().map_or("?", String::as_str), ends.to.first().map_or("?", String::as_str));
    match candidates.len() {
        1 => Ok(candidates.remove(0)),
        0 => document.attribute_match(ends),
        _ => Err(EditError::Ambiguous {
            element: described(),
            lines: candidates.iter().map(|candidate| document.lexemes[candidate.from.index].span.line).collect(),
        }),
    }
}

/// Replace whole lexemes of `source` by new text; the result must still
/// parse. The caller checks on the compiled model that it means what was asked.
pub fn replace_lexemes(source: &str, replacements: &[(usize, String)]) -> Result<String, EditError> {
    let document = Document::read(source)?;
    let mut sorted: Vec<&(usize, String)> = replacements.iter().collect();
    sorted.sort_by(|a, b| b.0.cmp(&a.0));
    if sorted.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(EditError::Unsafe("one place of the text would be written twice".to_string()));
    }
    let mut chars = document.chars.clone();
    for (index, text) in sorted {
        let lexeme = document.lexemes.get(*index).ok_or_else(|| EditError::Invalid(format!("no lexeme {index}")))?;
        let start = lexeme.offset;
        let end = document.end_of(*index);
        chars.splice(start..end, text.chars());
    }
    let edited: String = chars.into_iter().collect();
    syntax_tree(&edited, "the edited model")?;
    Ok(edited)
}

/// `text` as it replaces `spot`: quoted like the spot was, bare only when
/// it can be an identifier.
pub fn spelled_like(spot: &Spot, text: &str) -> String {
    let identifier = text.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if spot.quoted || !identifier {
        string_literal(text)
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn ends_are_found_on_the_line_and_in_the_block() {
        let source = "state_machine M {\n    state Off {}\n    state On {}\n    transition Off -> On { guard: \"go\" }\n}\nscenario S {\n    message \"A\" -> \"B\" \"ping\"\n    message \"A\" -> \"B\" \"pong\"\n}\nlogical_architecture L {\n    component_exchange \"Flow\" { from_port: \"A.p\" to_port: \"B.q\" }\n}\n";
        let transition = relationship_spots(source, &RelationshipEnds { keywords: &["transition"], designators: &[], name: None, from: &strings(&["Off"]), to: &strings(&["On"]) }).unwrap();
        assert_eq!((transition.from.text.as_str(), transition.to.text.as_str(), transition.from.quoted), ("Off", "On", false));
        let pong = relationship_spots(source, &RelationshipEnds { keywords: &["message"], designators: &[], name: Some("pong"), from: &strings(&["A"]), to: &strings(&["B"]) }).unwrap();
        assert_eq!(pong.name.as_ref().map(|s| s.text.as_str()), Some("pong"));
        let edited = replace_lexemes(source, &[(pong.to.index, spelled_like(&pong.to, "C"))]).unwrap();
        assert!(edited.contains("message \"A\" -> \"C\" \"pong\"") && edited.contains("message \"A\" -> \"B\" \"ping\""), "{edited}");
        let none = relationship_spots(source, &RelationshipEnds { keywords: &["message"], designators: &[], name: None, from: &strings(&["A"]), to: &strings(&["B"]) });
        assert!(matches!(none, Err(EditError::Ambiguous { .. })));
        let flow = relationship_spots(source, &RelationshipEnds { keywords: &["component_exchange"], designators: &strings(&["Flow"]), name: Some("Flow"), from: &strings(&["A.p"]), to: &strings(&["B.q"]) }).unwrap();
        assert_eq!(flow.name.as_ref().map(|s| s.text.as_str()), Some("Flow"));
        let swapped = replace_lexemes(source, &[(flow.from.index, spelled_like(&flow.from, "B.q")), (flow.to.index, spelled_like(&flow.to, "A.p"))]).unwrap();
        assert!(swapped.contains("from_port: \"B.q\" to_port: \"A.p\""), "{swapped}");
    }
}
