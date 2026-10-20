//! Views rendered as element tables (`render Views::asElementTable;`) that
//! say which table they are (`attribute 'table' : String = "icd";`): the
//! `table` declarations ArcLang exports, read back as written.

use super::{list_literal, Mapper};
use crate::compiler::source_edit::string_literal;
use crate::compiler::sysml_import::records::Element;

/// The parameters that are lists, written `a, b` in SysML.
const LISTS: [&str; 3] = ["columns", "rows", "kinds"];
const KINDS: [&str; 6] = ["requirements_coverage", "traceability_matrix", "allocation_matrix", "interface_n2", "icd", "elements"];

impl<'g> Mapper<'g> {
    pub(super) fn sysml_tables(&mut self) {
        let views: Vec<&'g Element> = self.graph.of_kinds(&["ViewUsage"]).filter(|v| !self.is_library(v)).collect();
        for view in views {
            let attributes: Vec<(String, String)> = view
                .children
                .iter()
                .filter_map(|c| self.graph.get(c))
                .filter(|c| c.kind == "AttributeUsage")
                .filter_map(|c| Some((c.name.clone()?, c.value.clone()?)))
                .collect();
            let Some(kind) = attributes.iter().find(|(k, _)| k == "table").map(|(_, v)| v.trim_matches('"').to_string()) else { continue };
            let name = Self::display(view);
            if !KINDS.contains(&kind.as_str()) {
                self.notes.push(format!("view '{name}' is a table of kind '{kind}' ArcLang does not compute: left out"));
                continue;
            }
            self.count("tables");
            self.line(0, &format!("table {} {{", string_literal(&name)));
            self.line(1, &format!("kind: {kind}"));
            if let Some(id) = view.short_name.as_ref().filter(|id| **id != name) {
                self.line(1, &format!("id: {}", string_literal(id)));
            }
            for (key, value) in attributes.iter().filter(|(k, _)| k != "table") {
                let text = value.trim_matches('"').replace("\\\"", "\"");
                let line = if LISTS.contains(&key.as_str()) {
                    let items: Vec<String> = text.split(", ").map(str::to_string).filter(|s| !s.is_empty()).collect();
                    format!("{key}: {}", list_literal(&items))
                } else if value == "true" || value == "false" || value.parse::<f64>().is_ok() {
                    format!("{key}: {value}")
                } else {
                    format!("{key}: {}", string_literal(&text))
                };
                self.line(1, &line);
            }
            if let Some(doc) = view.docs.first() {
                self.line(1, &format!("description: {}", string_literal(doc.trim())));
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }
}
