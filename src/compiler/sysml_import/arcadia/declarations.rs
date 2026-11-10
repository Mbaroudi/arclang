//! What ArcLang declares and SysML has no notion of, read back: a part
//! carrying `#IoBinding`, `#Twin` or `#Rule` is the `io_binding`, the `twin`
//! or the `rule` it was exported from, its attributes as they were written
//! (what it names is text there: the machine of a binding, the requirements
//! of a twin).

use super::{record_source, Mapper, REFERENCE_KEYS};
use crate::compiler::source_edit::string_literal;
use crate::compiler::sysml_export::declarations::METADATA;
use crate::compiler::sysml_import::records::Element;

/// The keyword of each declaration, and what the report counts it as.
const DECLARATIONS: [(&str, &str, &str); 3] = [("IoBinding", "io_binding", "I/O bindings"), ("Twin", "twin", "twins"), ("Rule", "rule", "rules")];

/// The keys whose value is a block (`data: { file: ... }`), exported as its source.
const BLOCKS: [&str; 2] = ["simulate", "data"];

impl<'g> Mapper<'g> {
    /// Whether a part is one of these declarations: not a component.
    pub(super) fn is_declaration(element: &Element) -> bool {
        Self::has_metadata(element, &METADATA)
    }

    pub(super) fn declarations(&mut self) {
        for (metadata, keyword, counted) in DECLARATIONS {
            let parts: Vec<&'g Element> = self.graph.of_kinds(&["PartUsage"]).filter(|p| !self.is_library(p) && Self::has_metadata(p, &[metadata])).collect();
            for part in parts {
                self.count(counted);
                self.line(0, &format!("{keyword} {} {{", string_literal(&Self::display(part))));
                // The id is the name when none was written.
                if let Some(id) = part.short_name.as_ref().filter(|id| **id != Self::display(part)) {
                    self.line(1, &format!("id: {}", string_literal(id)));
                }
                if let Some(description) = Self::description(part) {
                    self.line(1, &format!("description: {}", string_literal(&description)));
                }
                for line in self.attribute_lines(&[part], &["description"], REFERENCE_KEYS) {
                    let block = line.split_once(": ").filter(|(key, _)| BLOCKS.contains(key)).and_then(|(key, value)| Some(format!("{key}: {}", record_source(value)?)));
                    self.line(1, &block.unwrap_or(line));
                }
                self.line(0, "}");
                self.line(0, "");
            }
        }
    }
}
