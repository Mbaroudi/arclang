//! The cybersecurity analysis of the profile, read back: a
//! `#SecurityScales` package (an enumeration per scale) is `security_scales`, a
//! `#Security` package a `security_analysis`, its `#PrimaryAsset` and
//! `#Threat` parts its assets and threats, with what they name.

use super::Mapper;
use crate::compiler::source_edit::string_literal;
use crate::compiler::sysml_import::records::Element;

/// Metadata of the elements of the analysis: no structure of a level.
pub(super) const SECURITY_METADATA: &[&str] = &["PrimaryAsset", "Threat", "SecurityScales"];

impl<'g> Mapper<'g> {
    pub(super) fn security(&mut self) {
        let scales: Vec<&'g Element> = self.graph.of_kinds(&["Package"]).filter(|d| !self.is_library(d) && Self::has_metadata(d, &["SecurityScales"])).collect();
        for package in scales {
            let id = self.remember(&package.id, Self::identity_id(package), "SecurityScales");
            self.line(0, "security_scales {");
            self.line(1, &format!("id: {}", string_literal(&id)));
            for scale in package.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "EnumerationDefinition") {
                // A scale is an ArcLang key: one that is not an identifier is left out.
                let Some(key) = scale.name.clone().filter(|k| k.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')) else {
                    self.count("security scales whose name is not an ArcLang key (left out)");
                    continue;
                };
                let literals: Vec<String> = scale.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "EnumerationUsage").filter_map(|c| c.name.clone()).collect();
                self.line(1, &format!("{key}: {}", super::list_literal(&literals)));
            }
            self.line(0, "}");
            self.line(0, "");
        }
        let packages: Vec<&'g Element> = self.graph.of_kinds(&["Package"]).filter(|p| !self.is_library(p) && Self::has_metadata(p, &["Security"])).collect();
        for package in packages {
            let id = self.remember(&package.id, Self::display(package), "SecurityAnalysis");
            self.count("security analyses");
            self.line(0, "security_analysis {");
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.attributes(1, &[package], &["description", "id"]);
            let children: Vec<&'g Element> = package.children.iter().filter_map(|c| self.graph.get(c)).collect();
            for (metadata, keyword, counted) in [("PrimaryAsset", "asset", "primary assets"), ("Threat", "threat", "threats")] {
                for element in children.iter().filter(|c| Self::has_metadata(c, &[metadata])) {
                    self.security_element(element, keyword, counted);
                }
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    fn security_element(&mut self, element: &'g Element, keyword: &str, counted: &'static str) {
        let id = self.remember(&element.id, Self::identity_id(element), if keyword == "asset" { "PrimaryAsset" } else { "Threat" });
        self.count(counted);
        self.line(1, &format!("{keyword} {} {{", string_literal(&Self::display(element))));
        self.line(2, &format!("id: {}", string_literal(&id)));
        if let Some(doc) = Self::description(element) {
            self.line(2, &format!("description: {}", string_literal(&doc)));
        }
        self.attributes(2, &[element], &["description", "id"]);
        self.reference_attributes(2, &[element.id.clone()]);
        self.line(1, "}");
    }
}
