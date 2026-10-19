//! Product lines, read back: a `variation part` is a component
//! `variation: true`, a variant of it `variant: true`, and a
//! `#Configuration` part a `configuration` with the variants it `selects`.

use super::Mapper;
use crate::compiler::source_edit::string_literal;
use crate::compiler::sysml_import::records::Element;

impl<'g> Mapper<'g> {
    /// The variability lines of a component: whether it is a variation (its
    /// usage or its definition says so), or a variant of the variation
    /// written around it; a variant whose variation is not a written
    /// component is a plain component.
    pub(super) fn variability(&mut self, depth: usize, index: usize) {
        if self.is_variation_node(index) {
            self.line(depth, "variation: true");
        }
        let is_variant = self.nodes[index].usage.is_some_and(|u| u.is_variant);
        match self.nodes[index].parent {
            Some(parent) if is_variant && self.is_variation_node(parent) => self.line(depth, "variant: true"),
            _ if is_variant => self.count("variants whose variation is not a component (written as components)"),
            _ => {}
        }
    }

    fn is_variation_node(&self, index: usize) -> bool {
        let node = &self.nodes[index];
        node.usage.is_some_and(|u| u.is_variation) || node.def.is_some_and(|d| d.is_variation)
    }

    pub(super) fn configurations(&mut self) {
        let configurations: Vec<&'g Element> = self.graph.of_kinds(&["PartUsage", "PartDefinition"]).filter(|p| !self.is_library(p) && Self::has_metadata(p, &["Configuration"])).collect();
        for config in configurations {
            let id = self.remember(&config.id, Self::identity_id(config), "Configuration");
            self.count("configurations");
            self.line(0, &format!("configuration {} {{", string_literal(&Self::display(config))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.attributes(1, &[config], &["description", "id"]);
            self.reference_attributes(1, &[config.id.clone()]);
            self.line(0, "}");
            self.line(0, "");
        }
    }
}
