//! Fragmented Capella models. Capella can store a package, a level or a
//! component in a `.capellafragment` file of its own (and its diagrams in an
//! `.airdfragment`); the main file then holds a proxy in its place:
//! `<ownedArchitectures xsi:type="…" href="fragments/LA.capellafragment#id"/>`.
//! The proxies are replaced by the roots of the fragment files, so that the
//! import sees the model Capella shows, never the main file alone.

use super::{parse, short_kind, Node};
use std::path::{Path, PathBuf};

/// The `(file, id)` a proxy of a fragment of this extension points to.
fn proxy_target<'a>(node: &'a Node, extension: &str) -> Option<(&'a str, &'a str)> {
    if !node.children.is_empty() || node.attrs.contains_key("id") || node.attrs.contains_key("uid") {
        return None;
    }
    let (file, id) = node.attr("href")?.rsplit_once('#')?;
    file.ends_with(&format!(".{extension}")).then_some((file, id))
}

/// `%20` and the other escapes of a URI path.
fn decode_uri(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(byte) = path.get(i + 1..i + 3).and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The fragment files referenced anywhere under `node`, as written.
pub(super) fn referenced(node: &Node, extension: &str) -> Vec<String> {
    let mut files: Vec<String> = node.descendants().into_iter().filter_map(|n| proxy_target(n, extension)).map(|(file, _)| decode_uri(file)).collect();
    files.sort();
    files.dedup();
    files
}

/// Replace, under `node`, every proxy of a fragment by the root of its file,
/// read relative to `base` (the folder of the file holding the proxy); the
/// fragments of a fragment are read relative to their own folder. Returns
/// the fragment files read. A fragment that cannot be read, that is not the
/// element its proxy names, or that holds itself is an error: a part of the
/// model would be missing.
pub(super) fn splice(node: &mut Node, base: &Path, extension: &str) -> Result<Vec<PathBuf>, String> {
    let mut read = Vec::new();
    splice_into(node, base, extension, &mut Vec::new(), &mut read)?;
    Ok(read)
}

fn splice_into(node: &mut Node, base: &Path, extension: &str, chain: &mut Vec<PathBuf>, read: &mut Vec<PathBuf>) -> Result<(), String> {
    for child in &mut node.children {
        let Some((file, id)) = proxy_target(child, extension) else {
            splice_into(child, base, extension, chain, read)?;
            continue;
        };
        let joined = base.join(decode_uri(file));
        let display = joined.display().to_string();
        let path = joined.canonicalize().map_err(|e| format!("the fragment {display} cannot be read ({e}): the model would be incomplete"))?;
        if chain.contains(&path) {
            return Err(format!("the fragment {display} holds itself"));
        }
        if read.contains(&path) {
            return Err(format!("the fragment {display} is referenced twice: an element has one place in a model"));
        }
        let xml = std::fs::read_to_string(&path).map_err(|e| format!("the fragment {display} cannot be read ({e}): the model would be incomplete"))?;
        let document = parse(&xml).map_err(|e| format!("the fragment {display} is not valid XML: {e}"))?;
        let mut root = document.children.into_iter().next().ok_or_else(|| format!("the fragment {display} is empty"))?;
        // Model elements are known by `id`, Sirius diagrams by `uid`.
        let root_id = root.attr("id").or_else(|| root.attr("uid")).unwrap_or("").to_string();
        if root_id != id {
            return Err(format!("the fragment {display} holds '{root_id}', not the element '{id}' its proxy names"));
        }
        chain.push(path.clone());
        splice_into(&mut root, path.parent().unwrap_or(base), extension, chain, read)?;
        chain.pop();
        read.push(path);
        // The proxy says where the element sits (its tag) and, usually, its
        // type; the fragment root holds the element itself.
        root.attrs.retain(|key, _| !key.starts_with("xmlns") && key != "xmi:version");
        if child.kind.is_empty() {
            child.kind = short_kind(&root.tag);
        }
        child.attrs = root.attrs;
        child.children = root.children;
        child.text = root.text;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{import_file, import_strings};
    use super::*;

    const HEAD: &str = r#"xmi:version="2.0" xmlns:xmi="http://www.omg.org/XMI" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#;

    fn main_file(href: &str) -> String {
        format!(
            r#"<org.polarsys.capella.core.data.capellamodeller:Project {HEAD} id="p1" name="Fragmented">
  <ownedModelRoots xsi:type="org.polarsys.capella.core.data.capellamodeller:SystemEngineering" id="se1" name="Fragmented">
    <ownedArchitectures xsi:type="org.polarsys.capella.core.data.ctx:SystemAnalysis" href="{href}"/>
  </ownedModelRoots>
</org.polarsys.capella.core.data.capellamodeller:Project>"#
        )
    }

    fn system_analysis(functions: &str) -> String {
        format!(
            r#"<org.polarsys.capella.core.data.ctx:SystemAnalysis {HEAD} id="sa1" name="System Analysis">
  <ownedFunctionPkg xsi:type="org.polarsys.capella.core.data.ctx:SystemFunctionPkg" id="sfp" name="System Functions">
    <ownedSystemFunctions xsi:type="org.polarsys.capella.core.data.ctx:SystemFunction" id="root" name="Root System Function">{functions}</ownedSystemFunctions>
  </ownedFunctionPkg>
</org.polarsys.capella.core.data.ctx:SystemAnalysis>"#
        )
    }

    const MEASURE: &str = r#"<ownedFunctions xsi:type="org.polarsys.capella.core.data.ctx:SystemFunction" id="f-measure" name="Measure speed"/>"#;

    /// A fresh folder holding `files` (path relative to it, content).
    fn project(name: &str, files: &[(&str, String)]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("arclang-fragments-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, content) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        dir
    }

    #[test]
    fn a_fragmented_model_is_imported_with_its_fragments() {
        let dir = project(
            "whole",
            &[
                ("Model.capella", main_file("fragments/System%20Analysis.capellafragment#sa1")),
                ("fragments/System Analysis.capellafragment", system_analysis(MEASURE)),
            ],
        );
        let imported = import_file(&dir.join("Model.capella")).expect("imports");
        assert!(imported.source.contains("function \"Measure speed\""), "{}", imported.source);
        assert!(imported.source.contains("id: \"f-measure\""), "{}", imported.source);
        assert_eq!(imported.counts.get("fragment files read").copied(), Some(1), "{:?}", imported.counts);
    }

    #[test]
    fn a_fragment_of_a_fragment_is_read_from_its_own_folder() {
        let functions = r#"<ownedFunctions xsi:type="org.polarsys.capella.core.data.ctx:SystemFunction" href="deeper/Measure.capellafragment#f-measure"/>"#;
        let measure = format!(r#"<org.polarsys.capella.core.data.ctx:SystemFunction {HEAD} id="f-measure" name="Measure speed"/>"#);
        let dir = project(
            "nested",
            &[
                ("Model.capella", main_file("fragments/SA.capellafragment#sa1")),
                ("fragments/SA.capellafragment", system_analysis(functions)),
                ("fragments/deeper/Measure.capellafragment", measure),
            ],
        );
        let imported = import_file(&dir.join("Model.capella")).expect("imports");
        assert!(imported.source.contains("function \"Measure speed\""), "{}", imported.source);
        assert_eq!(imported.counts.get("fragment files read").copied(), Some(2));
    }

    #[test]
    fn a_missing_fragment_fails_the_import_and_names_the_file() {
        let dir = project("missing", &[("Model.capella", main_file("fragments/SA.capellafragment#sa1"))]);
        let failure = import_file(&dir.join("Model.capella")).err().expect("refused");
        assert!(failure.message.contains("SA.capellafragment") && failure.message.contains("incomplete"), "{}", failure.message);
    }

    #[test]
    fn a_fragment_that_is_not_the_element_of_its_proxy_fails_the_import() {
        let dir = project(
            "other",
            &[("Model.capella", main_file("fragments/SA.capellafragment#sa-other")), ("fragments/SA.capellafragment", system_analysis(MEASURE))],
        );
        let failure = import_file(&dir.join("Model.capella")).err().expect("refused");
        assert!(failure.message.contains("sa-other"), "{}", failure.message);
    }

    #[test]
    fn a_diagram_fragment_known_by_its_uid_is_spliced() {
        let aird = r#"<viewpoint:DAnalysis xmlns:viewpoint="http://www.eclipse.org/sirius/1.1.0" uid="_main"><ownedViews href="fragments/SA.airdfragment#_sa"/></viewpoint:DAnalysis>"#;
        let fragment = r#"<viewpoint:DAnalysis xmlns:viewpoint="http://www.eclipse.org/sirius/1.1.0" uid="_sa"><ownedRepresentationDescriptors uid="_d1" name="[SAB] Overview"/></viewpoint:DAnalysis>"#;
        let dir = project("aird", &[("Model.aird", aird.to_string()), ("fragments/SA.airdfragment", fragment.to_string())]);
        let mut document = parse(aird).unwrap();
        let read = splice(&mut document, &dir, "airdfragment").expect("spliced");
        assert_eq!(read.len(), 1);
        let names: Vec<&str> = document.descendants().into_iter().map(|n| n.name()).collect();
        assert!(names.contains(&"[SAB] Overview"), "{names:?}");
    }

    #[test]
    fn a_fragment_that_holds_itself_fails_instead_of_looping() {
        let looping = format!(
            r#"<org.polarsys.capella.core.data.ctx:SystemAnalysis {HEAD} id="sa1" name="SA"><ownedFunctionPkg href="../fragments/SA.capellafragment#sa1"/></org.polarsys.capella.core.data.ctx:SystemAnalysis>"#
        );
        let dir = project("cycle", &[("Model.capella", main_file("fragments/SA.capellafragment#sa1")), ("fragments/SA.capellafragment", looping)]);
        let failure = import_file(&dir.join("Model.capella")).err().expect("refused");
        assert!(failure.message.contains("holds itself"), "{}", failure.message);
    }

    #[test]
    fn a_fragment_referenced_twice_fails_the_import() {
        let twice = r#"<org.polarsys.capella.core.data.capellamodeller:Project xmi:version="2.0" xmlns:xmi="http://www.omg.org/XMI" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" id="p1" name="Twice">
  <ownedModelRoots xsi:type="org.polarsys.capella.core.data.capellamodeller:SystemEngineering" id="se1" name="Twice">
    <ownedArchitectures xsi:type="org.polarsys.capella.core.data.ctx:SystemAnalysis" href="fragments/SA.capellafragment#sa1"/>
    <ownedArchitectures xsi:type="org.polarsys.capella.core.data.ctx:SystemAnalysis" href="fragments/SA.capellafragment#sa1"/>
  </ownedModelRoots>
</org.polarsys.capella.core.data.capellamodeller:Project>"#;
        let dir = project("twice", &[("Model.capella", twice.to_string()), ("fragments/SA.capellafragment", system_analysis(MEASURE))]);
        let failure = import_file(&dir.join("Model.capella")).err().expect("refused");
        assert!(failure.message.contains("referenced twice"), "{}", failure.message);
    }

    #[test]
    fn a_fragmented_model_given_as_text_is_refused_with_its_fragments_named() {
        let failure = import_strings(&main_file("fragments/SA.capellafragment#sa1"), None, "Model.capella").err().expect("refused");
        assert!(failure.message.contains("fragments/SA.capellafragment"), "{}", failure.message);
    }

    #[test]
    fn uri_escapes_are_decoded() {
        assert_eq!(decode_uri("LA-Logical%20System.capellafragment"), "LA-Logical System.capellafragment");
        assert_eq!(decode_uri("a%2"), "a%2");
        assert_eq!(decode_uri("plain"), "plain");
    }
}
