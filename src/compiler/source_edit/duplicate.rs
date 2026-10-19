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
/// value of every `id:` of the copy; `new_name`, the name of the copy when
/// its declaration writes no id. The text must parse; the caller checks
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
    let mut replacements: Vec<(usize, String)> = Vec::new();
    for index in start..=end {
        let lexeme = &document.lexemes[index];
        let is_id_key = is_word(lexeme) && lexeme.text == "id" && is(&document.lexemes, index + 1, &Token::Colon);
        if !is_id_key {
            continue;
        }
        if let Some(Token::StringLiteral(id)) = document.lexemes.get(index + 2).and_then(token) {
            replacements.push((index + 2, string_literal(&fresh_id(id))));
        }
    }
    let writes_id = block.as_ref().is_some_and(|block| document.written_id(block).is_some());
    if !writes_id {
        // The name is the identity: the first string (or the last word) of the declaration line.
        let line_end = block.as_ref().map_or(end + 1, |block| block.open);
        let name_at = (start + 1..line_end)
            .find(|index| matches!(token(&document.lexemes[*index]), Some(Token::StringLiteral(_))))
            .or_else(|| (start + 1..line_end).rev().find(|index| is_word(&document.lexemes[*index])))
            .ok_or_else(|| EditError::Unsafe(format!("'{element}' writes neither an id nor a name this copy can change")))?;
        replacements.push((name_at, string_literal(new_name)));
    }
    // The copied text, with the new identities spliced in from the end.
    let origin = document.lexemes[start].offset;
    let mut copy: Vec<char> = document.chars[origin..document.end_of(end)].to_vec();
    replacements.sort_by_key(|(index, _)| std::cmp::Reverse(*index));
    for (index, text) in &replacements {
        let from = document.lexemes[*index].offset - origin;
        let to = document.end_of(*index) - origin;
        copy.splice(from..to, text.chars());
    }
    let copy: String = copy.into_iter().collect();
    let base = document.indentation_of(start);
    let declaration = copy
        .trim()
        .lines()
        .enumerate()
        .map(|(index, line)| if index == 0 { line } else { line.strip_prefix(base.as_str()).unwrap_or(line) })
        .collect::<Vec<_>>()
        .join("\n");
    let edited = match target {
        CopyTarget::Beside => {
            let indented: Vec<String> = declaration.lines().map(|line| format!("{base}{line}")).collect();
            // After the end of the line: a comment that ends it stays with the original.
            let trailing = document.lexemes.get(end + 1).filter(|next| next.newlines_before == 0 && token(next).is_none());
            let at = trailing.map_or(document.end_of(end), |_| document.end_of(end + 1));
            document.splice(at, at, &format!("\n{}", indented.join("\n")))
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_takes_new_ids_and_keeps_the_text() {
        let source = "logical_architecture L {\n    component \"A\" {\n        id: \"LC-A\" // the first\n        function \"F\" { id: \"LF-F\" }\n        port out p\n    }\n    component \"B\" { id: \"LC-B\" }\n}\n";
        let mut fresh = |id: &str| format!("{id}-2");
        let beside = duplicate_element(source, "LC-A", CopyTarget::Beside, &mut fresh, "A 2").unwrap();
        assert!(beside.contains("id: \"LC-A-2\" // the first") && beside.contains("function \"F\" { id: \"LF-F-2\" }") && beside.contains("id: \"LC-A\" // the first"), "{beside}");
        let into = duplicate_element(source, "LF-F", CopyTarget::Into(Container::Element("LC-B")), &mut fresh, "F 2").unwrap();
        assert!(into.find("function \"F\" { id: \"LF-F-2\" }").unwrap() > into.find("component \"B\"").unwrap(), "{into}");
        let unnamed = "logical_architecture L {\n    component \"A\" {}\n}\n";
        let renamed = duplicate_element(unnamed, "A", CopyTarget::Beside, &mut fresh, "A 2").unwrap();
        assert!(renamed.contains("component \"A\" {}\n    component \"A 2\" {}"), "{renamed}");
    }
}
