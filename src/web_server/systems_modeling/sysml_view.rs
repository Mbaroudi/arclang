//! The SysML v2 vocabulary of a commit: the abstract syntax of the model's
//! SysML v2 export (see `compiler::sysml_syntax`), linked back to the
//! ArcLang elements it was exported from.

use super::*;
use crate::compiler::sysml_records;
use crate::compiler::sysmlv2_generator::generate_sysmlv2;

pub(super) fn build(result: &crate::CompilationResult, arclang: &View) -> Result<View, String> {
    let text = generate_sysmlv2(&result.semantic_model, &result.ast);
    let mut rendered = sysml_records::from_text(&text)?;

    // The export gives an element its ArcLang identifier as short name:
    // that is the bridge between the two vocabularies.
    let by_identifier: HashMap<&str, &str> = arclang
        .records
        .iter()
        .filter(|record| record.get("relatedElement").is_none())
        .filter_map(|record| Some((record["shortName"].as_str()?, record["@id"].as_str()?)))
        .collect();
    for record in &mut rendered.records {
        let source = record["declaredShortName"].as_str().and_then(|short| by_identifier.get(short));
        if let (Some(source), Some(object)) = (source, record.as_object_mut()) {
            object.insert("arclang:element".into(), reference(source));
        }
    }
    Ok(View::new(rendered.records, rendered.roots, rendered.ends))
}
