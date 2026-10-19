//! A ReqIF file as a tree: elements by local name (namespace prefixes
//! dropped, `xhtml:div` is `div`), attributes and text with every entity
//! and character reference decoded.

use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Default, Clone)]
pub struct Node {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
    /// The text directly inside the element, in order with nothing between.
    pub text: String,
    /// Text and children interleaved, for mixed content (XHTML).
    pub content: Vec<Content>,
}

#[derive(Debug, Clone)]
pub enum Content {
    Text(String),
    Child(usize),
}

impl Node {
    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// Every descendant named `name`, in document order.
    pub fn descendants<'a>(&'a self, name: &str, out: &mut Vec<&'a Node>) {
        for child in &self.children {
            if child.name == name {
                out.push(child);
            }
            child.descendants(name, out);
        }
    }

    /// The text of the first `…-REF` element under this one (`<DEFINITION><X-REF>id</X-REF></DEFINITION>`).
    pub fn reference(&self) -> Option<String> {
        self.children.iter().find(|c| c.name.ends_with("-REF")).map(|c| c.text.trim().to_string())
    }
}

/// Decode `&lt;` `&gt;` `&amp;` `&quot;` `&apos;` and numeric references.
pub fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find(';').filter(|&end| end <= 10) {
            Some(end) => {
                match reference(&after[..end]) {
                    Some(c) => out.push(c),
                    None => out.push_str(&rest[start..start + end + 2]),
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn reference(name: &str) -> Option<char> {
    match name {
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{a0}'),
        _ => {
            let code = name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")).map(|h| u32::from_str_radix(h, 16)).or_else(|| name.strip_prefix('#').map(|d| d.parse::<u32>()))?;
            code.ok().and_then(char::from_u32)
        }
    }
}

fn local(name: &[u8]) -> String {
    let name = String::from_utf8_lossy(name);
    name.rsplit(':').next().unwrap_or(&name).to_string()
}

fn element(e: &quick_xml::events::BytesStart) -> Node {
    let attributes = e
        .attributes()
        .flatten()
        .map(|a| (local(a.key.as_ref()), unescape(&String::from_utf8_lossy(&a.value))))
        .collect();
    Node { name: local(e.name().as_ref()), attributes, ..Node::default() }
}

fn push_text(node: &mut Node, text: &str) {
    node.text.push_str(text);
    match node.content.last_mut() {
        Some(Content::Text(last)) => last.push_str(text),
        _ => node.content.push(Content::Text(text.to_string())),
    }
}

pub fn parse(xml: &str) -> Result<Node, String> {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<Node> = vec![Node { name: "#document".to_string(), ..Node::default() }];
    loop {
        let event = reader.read_event().map_err(|e| format!("ReqIF parse error at byte {}: {e}", reader.buffer_position()))?;
        match event {
            Event::Eof => break,
            Event::Start(e) => stack.push(element(&e)),
            Event::Empty(e) => {
                let node = element(&e);
                let parent = stack.last_mut().expect("the document is open");
                parent.content.push(Content::Child(parent.children.len()));
                parent.children.push(node);
            }
            Event::End(_) => {
                let node = stack.pop().filter(|_| !stack.is_empty()).ok_or("ReqIF parse error: unbalanced end tag")?;
                let parent = stack.last_mut().ok_or("ReqIF parse error: unbalanced end tag")?;
                parent.content.push(Content::Child(parent.children.len()));
                parent.children.push(node);
            }
            Event::Text(t) => {
                let text = t.decode().map_err(|e| e.to_string())?;
                push_text(stack.last_mut().expect("the document is open"), &unescape(&text));
            }
            Event::CData(t) => {
                let text = t.decode().map_err(|e| e.to_string())?.to_string();
                push_text(stack.last_mut().expect("the document is open"), &text);
            }
            Event::GeneralRef(r) => {
                let name = r.decode().map_err(|e| e.to_string())?;
                let decoded = reference(&name).map(String::from).unwrap_or_else(|| format!("&{name};"));
                push_text(stack.last_mut().expect("the document is open"), &decoded);
            }
            _ => {}
        }
    }
    if stack.len() != 1 {
        return Err("ReqIF parse error: the file ends inside an element".to_string());
    }
    Ok(stack.pop().expect("the document"))
}

/// The readable text of mixed XHTML content: blocks and line breaks become
/// new lines, spaces are collapsed.
pub fn plain_text(node: &Node) -> String {
    fn walk(node: &Node, out: &mut String) {
        let block = matches!(node.name.as_str(), "div" | "p" | "br" | "li" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul" | "ol" | "table");
        if block {
            out.push('\n');
        }
        for item in &node.content {
            match item {
                Content::Text(text) => out.push_str(text),
                Content::Child(index) => walk(&node.children[*index], out),
            }
        }
        if block {
            out.push('\n');
        }
    }
    let mut raw = String::new();
    walk(node, &mut raw);
    raw.lines().map(|l| l.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_and_prefixes_are_decoded() {
        let root = parse(r#"<a x="1 &lt; 2 &#38; &#x41;"><x:b>C &amp; D</x:b></a>"#).unwrap();
        let a = &root.children[0];
        assert_eq!(a.attribute("x"), Some("1 < 2 & A"));
        assert_eq!(a.child("b").unwrap().text, "C & D");
    }

    #[test]
    fn xhtml_becomes_lines_of_text() {
        let root = parse("<v><div><p>One  <b>bold</b></p><p>Two</p></div></v>").unwrap();
        assert_eq!(plain_text(&root.children[0]), "One bold\nTwo");
    }
}
