//! A copy of a declaration: the element with everything declared inside
//! it, written again as it is written, with new identities. Each `id:` of
//! the copy is given a new value by the caller; an element that writes no
//! id takes its identity from its name, so the copy's name changes instead.

use super::*;

/// Where a copy goes.
pub enum CopyTarget<'a> {
    /// Right after the original, in the same container.
    Beside,
    Into(Container<'a>),
}

/// Copy the declaration of `element` to `target`. `fresh_id` gives the new
/// value of every `id:` of the copy; `new_name`, the name of the copy. The text must parse; the caller checks
/// the compiled model.
pub fn duplicate_element(
    source: &str,
    element: &str,
    target: CopyTarget,
    fresh_id: &mut dyn FnMut(&str) -> String,
    new_name: &str,
) -> Result<String, EditError> {
    let document = Document::read(source)?;
    let (start, end, block) = match document.find_element(element) {
        Ok(block) => (document.header_start(&block), block.close, Some(block)),
        Err(EditError::NotFound(_)) => {
            let (start, name) = document.find_bare_declaration(element).ok_or_else(|| EditError::NotFound(element.to_string()))?;
            (start, name, None)
        }
        Err(other) => return Err(other),
    };
    // The new identities, lexeme by lexeme, inside the copied range.
    let mut replacements = document.fresh_ids(start, end, fresh_id);
    // The copy is told apart by its name as well as by its ids ("Camera 2").
    {
        // The name: the first string (or the last word) of the declaration line.
        let line_end = block.as_ref().map_or(end + 1, |block| block.open);
        let name_at = (start + 1..line_end)
            .find(|index| matches!(token(&document.lexemes[*index]), Some(Token::StringLiteral(_))))
            .or_else(|| (start + 1..line_end).rev().find(|index| is_word(&document.lexemes[*index])))
            .ok_or_else(|| EditError::Unsafe(format!("'{element}' writes neither an id nor a name this copy can change")))?;
        replacements.push((name_at, string_literal(new_name)));
    }
    let declaration = document.copied(start, end, &mut replacements);
    let edited = match target {
        CopyTarget::Beside => document.insert_beside(start, end, &declaration),
        CopyTarget::Into(container) => {
            let target = match container {
                Container::Element(other) => document.find_element(other)?,
                Container::Layer { keyword, name } => document.find_layer(keyword, name)?,
                Container::Document => return Err(EditError::Invalid("a copy goes into an element or a layer".to_string())),
            };
            if let Some(block) = &block {
                if target.open >= block.open && target.close <= block.close {
                    return Err(EditError::Unsafe(format!("'{element}' cannot be copied inside itself")));
                }
            }
            document.insert_declaration(&target, &declaration)
        }
    };
    syntax_tree(&edited, "the edited model")?;
    Ok(edited)
}

impl Document {
    /// Every `id:` written between lexemes `start` and `end`, with the new value `fresh_id` gives it.
    fn fresh_ids(&self, start: usize, end: usize, fresh_id: &mut dyn FnMut(&str) -> String) -> Vec<(usize, String)> {
        let mut replacements = Vec::new();
        for index in start..=end {
            let lexeme = &self.lexemes[index];
            let is_id_key = is_word(lexeme) && lexeme.text == "id" && is(&self.lexemes, index + 1, &Token::Colon);
            if !is_id_key {
                continue;
            }
            if let Some(Token::StringLiteral(id)) = self.lexemes.get(index + 2).and_then(token) {
                replacements.push((index + 2, string_literal(&fresh_id(id))));
            }
        }
        replacements
    }

    /// The text of lexemes `start..=end` with the replacements spliced in,
    /// its lines after the first taken back to the indentation of the first.
    fn copied(&self, start: usize, end: usize, replacements: &mut [(usize, String)]) -> String {
        let origin = self.lexemes[start].offset;
        let mut copy: Vec<char> = self.chars[origin..self.end_of(end)].to_vec();
        replacements.sort_by_key(|(index, _)| std::cmp::Reverse(*index));
        for (index, text) in replacements.iter() {
            let from = self.lexemes[*index].offset - origin;
            let to = self.end_of(*index) - origin;
            copy.splice(from..to, text.chars());
        }
        let copy: String = copy.into_iter().collect();
        let base = self.indentation_of(start);
        copy.trim()
            .lines()
            .enumerate()
            .map(|(index, line)| if index == 0 { line } else { line.strip_prefix(base.as_str()).unwrap_or(line) })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// `declaration` written on the lines after the one lexeme `end` ends,
    /// indented like lexeme `start`; a comment that ends that line stays with it.
    fn insert_beside(&self, start: usize, end: usize, declaration: &str) -> String {
        let base = self.indentation_of(start);
        let indented: Vec<String> = declaration.lines().map(|line| format!("{base}{line}")).collect();
        let trailing = self.lexemes.get(end + 1).filter(|next| next.newlines_before == 0 && token(next).is_none());
        let at = trailing.map_or(self.end_of(end), |_| self.end_of(end + 1));
        self.splice(at, at, &format!("\n{}", indented.join("\n")))
    }

    /// The first lexeme of the line `index` is on, as a declaration line starts.
    fn line_start(&self, index: usize) -> usize {
        let mut start = index;
        while start > 0 && self.lexemes[start].newlines_before == 0 && !matches!(token(&self.lexemes[start - 1]), None | Some(Token::LeftBrace | Token::RightBrace)) {
            start -= 1;
        }
        start
    }

    /// The first and last lexemes of the relationship that writes an end at
    /// `anchor`: on its declaration line (`trace "A" satisfies "B" { }`), or
    /// as an attribute of its block (`from_port: "A.p"`).
    fn relationship_around(&self, anchor: usize) -> Option<(usize, usize)> {
        let start = self.line_start(anchor);
        let is_attribute = is_word(&self.lexemes[start]) && is(&self.lexemes, start + 1, &Token::Colon);
        if is_attribute {
            let block = self.blocks().into_iter().filter(|block| block.open < anchor && anchor < block.close).max_by_key(|block| block.open)?;
            let header = self.header(&block).len();
            return Some((block.open - header, block.close));
        }
        let line = self.line_from(start);
        match self.block_after(&line, start) {
            Some(block) => Some((start, block.close)),
            None => Some((start, line.last().copied().unwrap_or(start))),
        }
    }
}

/// A copy of the relationship that writes an end at lexeme `anchor`, right
/// after it, with `replacements` (lexeme, new text) made in the copy: its
/// ends and its name. Every `id:` of the copy is given by `fresh_id`. The
/// text must parse; the caller checks the compiled model.
pub fn duplicate_relationship(source: &str, anchor: usize, replacements: &[(usize, String)], fresh_id: &mut dyn FnMut(&str) -> String) -> Result<String, EditError> {
    let document = Document::read(source)?;
    let (start, end) = document.relationship_around(anchor).ok_or_else(|| EditError::Unsafe("the declaration of the relationship could not be found".to_string()))?;
    if replacements.iter().any(|(index, _)| *index < start || *index > end) {
        return Err(EditError::Unsafe("a change of the copy falls outside the declaration of the relationship".to_string()));
    }
    let mut all = replacements.to_vec();
    all.extend(document.fresh_ids(start, end, fresh_id));
    let mut seen = std::collections::HashSet::new();
    if !all.iter().all(|(index, _)| seen.insert(*index)) {
        return Err(EditError::Unsafe("one place of the copy would be written twice".to_string()));
    }
    let declaration = document.copied(start, end, &mut all);
    let edited = document.insert_beside(start, end, &declaration);
    syntax_tree(&edited, "the edited model")?;
    Ok(edited)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_takes_new_ids_and_keeps_the_text() {
        let source = "logical_architecture L {\n    component \"A\" {\n        id: \"LC-A\" // the first\n        function \"F\" { id: \"LF-F\" }\n        port out p\n    }\n    component \"B\" { id: \"LC-B\" }\n}\n";
        let mut fresh = |id: &str| format!("{id}-2");
        let beside = duplicate_element(source, "LC-A", CopyTarget::Beside, &mut fresh, "A 2").unwrap();
        assert!(beside.contains("component \"A 2\" {\n        id: \"LC-A-2\" // the first") && beside.contains("function \"F\" { id: \"LF-F-2\" }") && beside.contains("id: \"LC-A\" // the first"), "{beside}");
        let into = duplicate_element(source, "LF-F", CopyTarget::Into(Container::Element("LC-B")), &mut fresh, "F 2").unwrap();
        assert!(into.find("function \"F 2\" { id: \"LF-F-2\" }").unwrap() > into.find("component \"B\"").unwrap(), "{into}");
        let unnamed = "logical_architecture L {\n    component \"A\" {}\n}\n";
        let renamed = duplicate_element(unnamed, "A", CopyTarget::Beside, &mut fresh, "A 2").unwrap();
        assert!(renamed.contains("component \"A\" {}\n    component \"A 2\" {}"), "{renamed}");
    }
}
