//! A small XMI element tree, written the way Capella writes its files:
//! `xsi:type` first, then `id`, then the other attributes in the order given.

/// The Capella package of a short kind prefix (`ctx` in `ctx:SystemFunction`).
fn package(prefix: &str) -> String {
    match prefix {
        "libraries" | "re" | "xmi" | "Requirements" | "CapellaRequirements" | "stpa" | "cybersecurity" => prefix.to_string(),
        "datatype" | "datavalue" => format!("org.polarsys.capella.core.data.information.{prefix}"),
        "deployment" => "org.polarsys.capella.core.data.pa.deployment".to_string(),
        other => format!("org.polarsys.capella.core.data.{other}"),
    }
}

/// `ctx:SystemFunction` -> `org.polarsys.capella.core.data.ctx:SystemFunction`.
pub(super) fn xsi_type(kind: &str) -> String {
    match kind.split_once(':') {
        Some((prefix, name)) => format!("{}:{name}", package(prefix)),
        None => kind.to_string(),
    }
}

/// The namespaces a Capella 7 model file declares.
pub(super) const NAMESPACES: &[(&str, &str)] = &[
    ("xmlns:xmi", "http://www.omg.org/XMI"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("xmlns:libraries", "http://www.polarsys.org/capella/common/libraries/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.capellacommon", "http://www.polarsys.org/capella/core/common/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.capellacore", "http://www.polarsys.org/capella/core/core/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.capellamodeller", "http://www.polarsys.org/capella/core/modeller/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.cs", "http://www.polarsys.org/capella/core/cs/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.ctx", "http://www.polarsys.org/capella/core/ctx/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.epbs", "http://www.polarsys.org/capella/core/epbs/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.fa", "http://www.polarsys.org/capella/core/fa/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.information", "http://www.polarsys.org/capella/core/information/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.information.datatype", "http://www.polarsys.org/capella/core/information/datatype/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.information.datavalue", "http://www.polarsys.org/capella/core/information/datavalue/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.interaction", "http://www.polarsys.org/capella/core/interaction/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.la", "http://www.polarsys.org/capella/core/la/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.oa", "http://www.polarsys.org/capella/core/oa/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.pa", "http://www.polarsys.org/capella/core/pa/7.0.0"),
    ("xmlns:org.polarsys.capella.core.data.pa.deployment", "http://www.polarsys.org/capella/core/pa/deployment/7.0.0"),
    ("xmlns:re", "http://www.polarsys.org/capella/common/re/7.0.0"),
    ("xmlns:CapellaRequirements", "http://www.polarsys.org/capella/requirements"),
    ("xmlns:Requirements", "http://www.polarsys.org/kitalpha/requirements"),
];

#[derive(Debug, Clone, Default)]
pub(crate) struct El {
    tag: String,
    attrs: Vec<(String, String)>,
    children: Vec<El>,
    /// Text content (`<semanticResources>x.capella</semanticResources>`).
    text: String,
}

impl El {
    /// A contained element: its feature (`ownedFunctions`), its kind and id.
    pub(super) fn new(tag: &str, kind: &str, id: &str) -> Self {
        let mut el = El { tag: tag.to_string(), ..Default::default() };
        if !kind.is_empty() {
            el.attrs.push(("xsi:type".to_string(), xsi_type(kind)));
        }
        el.attrs.push(("id".to_string(), id.to_string()));
        el
    }
    /// The root element of a file: its tag is its type.
    pub(super) fn root(tag: &str) -> Self {
        El { tag: tag.to_string(), ..Default::default() }
    }
    /// An element holding only text.
    pub(super) fn text(tag: &str, text: &str) -> Self {
        El { tag: tag.to_string(), text: text.to_string(), ..Default::default() }
    }
    pub(super) fn attr(mut self, key: &str, value: &str) -> Self {
        self.set(key, value);
        self
    }
    pub(super) fn set(&mut self, key: &str, value: &str) {
        match self.attrs.iter_mut().find(|(k, _)| k == key) {
            Some(slot) => slot.1 = value.to_string(),
            None => self.attrs.push((key.to_string(), value.to_string())),
        }
    }
    pub(super) fn rename(&mut self, tag: &str) {
        self.tag = tag.to_string();
    }
    pub(super) fn remove(&mut self, key: &str) {
        self.attrs.retain(|(k, _)| k != key);
    }
    pub(super) fn opt(self, key: &str, value: Option<&str>) -> Self {
        match value {
            Some(value) if !value.is_empty() => self.attr(key, value),
            _ => self,
        }
    }
    pub(super) fn push(&mut self, child: El) {
        self.children.push(child);
    }
    pub(super) fn with(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }
    /// The value of an attribute.
    pub(super) fn get(&self, key: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
    /// The `xsi:type` of the element.
    pub(super) fn kind(&self) -> &str {
        self.attrs.iter().find(|(k, _)| k == "xsi:type").map(|(_, v)| v.as_str()).unwrap_or("")
    }
    pub(super) fn children(&self) -> &[El] {
        &self.children
    }
    pub(super) fn children_mut(&mut self) -> &mut [El] {
        &mut self.children
    }
    /// The element of this id, at any depth.
    pub(super) fn find(&self, id: &str) -> Option<&El> {
        if self.id() == id {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(id))
    }
    /// The element of this id, at any depth.
    pub(super) fn find_mut(&mut self, id: &str) -> Option<&mut El> {
        if self.id() == id {
            return Some(self);
        }
        self.children.iter_mut().find_map(|child| child.find_mut(id))
    }
    /// Remove and return the children for which `take` holds.
    pub(super) fn take_children(&mut self, take: impl Fn(&El) -> bool) -> Vec<El> {
        let (taken, kept): (Vec<El>, Vec<El>) = std::mem::take(&mut self.children).into_iter().partition(|c| take(c));
        self.children = kept;
        taken
    }
    pub(super) fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
    pub(super) fn id(&self) -> &str {
        self.attrs.iter().find(|(k, _)| k == "id").map(|(_, v)| v.as_str()).unwrap_or("")
    }

    pub(super) fn render(&self, out: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        out.push_str(&indent);
        out.push('<');
        out.push_str(&self.tag);
        for (key, value) in &self.attrs {
            out.push(' ');
            out.push_str(key);
            out.push_str("=\"");
            out.push_str(&escape(value));
            out.push('"');
        }
        if !self.text.is_empty() {
            out.push('>');
            out.push_str(&escape(&self.text));
            out.push_str("</");
            out.push_str(&self.tag);
            out.push_str(">\n");
            return;
        }
        if self.children.is_empty() {
            out.push_str("/>\n");
            return;
        }
        out.push_str(">\n");
        for child in &self.children {
            child.render(out, depth + 1);
        }
        out.push_str(&indent);
        out.push_str("</");
        out.push_str(&self.tag);
        out.push_str(">\n");
    }
}

/// An attribute value as XMI writes it.
pub(super) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\n' => out.push_str("&#xA;"),
            '\r' => out.push_str("&#xD;"),
            '\t' => out.push_str("&#x9;"),
            c => out.push(c),
        }
    }
    out
}

/// A document: the XML declaration, then the root.
pub(super) fn document(root: &El) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    root.render(&mut out, 0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_written_with_their_capella_package() {
        assert_eq!(xsi_type("ctx:SystemFunction"), "org.polarsys.capella.core.data.ctx:SystemFunction");
        assert_eq!(xsi_type("datatype:NumericType"), "org.polarsys.capella.core.data.information.datatype:NumericType");
        assert_eq!(xsi_type("libraries:ModelInformation"), "libraries:ModelInformation");
    }

    #[test]
    fn elements_render_as_capella_writes_them() {
        let el = El::new("ownedFunctions", "ctx:SystemFunction", "f1").attr("name", "Say \"hi\" & go").with(El::new("inputs", "fa:FunctionInputPort", "p1"));
        let mut out = String::new();
        el.render(&mut out, 0);
        assert_eq!(
            out,
            "<ownedFunctions xsi:type=\"org.polarsys.capella.core.data.ctx:SystemFunction\" id=\"f1\" name=\"Say &quot;hi&quot; &amp; go\">\n  <inputs xsi:type=\"org.polarsys.capella.core.data.fa:FunctionInputPort\" id=\"p1\"/>\n</ownedFunctions>\n"
        );
    }
}
