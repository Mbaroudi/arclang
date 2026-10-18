//! Tokens of the KerML / SysML v2 textual notation (KerMLExpressions.xtext
//! terminals): words, quoted names, strings, numbers, regular comments
//! (which are elements), operators. Notes (`//`, `//* */`) and white space
//! are dropped.

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// An identifier or keyword (`part`, `x`); keywords are told apart by the parser.
    Word(String),
    /// `'quoted name'`, unescaped.
    Name(String),
    /// `"string"`, unescaped.
    Str(String),
    /// `12`
    Integer(String),
    /// `1.5`, `.5`, `1e3`, `2.5E-3`
    Real(String),
    /// `/* ... */` with its delimiters removed.
    Comment(String),
    /// Any operator or punctuation: `::>`, `:>>`, `->`, `{`...
    Symbol(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: Kind,
    /// Byte offset of the token in the text, for messages.
    pub offset: usize,
    pub line: usize,
}

/// Symbols, longest first so that `::>` wins over `::` and `:`.
const SYMBOLS: &[&str] = &[
    "::>", ":>>", "!==", "===", "@@", "::", ":>", ":=", "->", "=>", "==", "!=", "<=", ">=", "**", "??", ".?", "..", "~", "@", "#", "$", "?", "|", "&",
    "%", "^", "!", "[", "]", "{", "}", "(", ")", ";", ",", ".", "=", ":", "<", ">", "+", "-", "*", "/",
];

pub fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    let mut line = 1;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        // Notes: `//* ... */` and `// ...` are not elements.
        if bytes[i..].starts_with(b"//*") {
            let end = find(bytes, i + 3, b"*/").ok_or_else(|| format!("line {line}: unterminated note"))?;
            line += count_lines(&bytes[i..end + 2]);
            i = end + 2;
            continue;
        }
        if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i..].starts_with(b"/*") {
            let end = find(bytes, i + 2, b"*/").ok_or_else(|| format!("line {line}: unterminated comment"))?;
            let body = &text[i + 2..end];
            tokens.push(Token { kind: Kind::Comment(body.to_string()), offset: start, line });
            line += count_lines(&bytes[i..end + 2]);
            i = end + 2;
            continue;
        }
        if c == b'\'' || c == b'"' {
            let (value, end) = quoted(text, i, c as char).map_err(|e| format!("line {line}: {e}"))?;
            tokens.push(Token { kind: if c == b'\'' { Kind::Name(value) } else { Kind::Str(value) }, offset: start, line });
            line += count_lines(&bytes[i..end]);
            i = end;
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let mut end = i + 1;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            tokens.push(Token { kind: Kind::Word(text[i..end].to_string()), offset: start, line });
            i = end;
            continue;
        }
        if c.is_ascii_digit() || (c == b'.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit)) {
            let (kind, end) = number(bytes, i, text);
            tokens.push(Token { kind, offset: start, line });
            i = end;
            continue;
        }
        let Some(symbol) = SYMBOLS.iter().find(|s| bytes[i..].starts_with(s.as_bytes())) else {
            return Err(format!("line {line}: unexpected character '{}'", text[i..].chars().next().unwrap_or('?')));
        };
        tokens.push(Token { kind: Kind::Symbol(symbol), offset: start, line });
        i += symbol.len();
    }
    Ok(tokens)
}

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    (from..bytes.len().saturating_sub(needle.len() - 1)).find(|&j| bytes[j..].starts_with(needle))
}

fn count_lines(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == b'\n').count()
}

/// A quoted name or string from `start` (the opening quote): its value and
/// the offset after the closing quote.
fn quoted(text: &str, start: usize, quote: char) -> Result<(String, usize), String> {
    let mut value = String::new();
    let mut chars = text[start + 1..].char_indices();
    while let Some((j, c)) = chars.next() {
        match c {
            '\\' => {
                let (_, escaped) = chars.next().ok_or("unterminated escape")?;
                value.push(match escaped {
                    'b' => '\u{8}',
                    't' => '\t',
                    'n' => '\n',
                    'f' => '\u{c}',
                    'r' => '\r',
                    other => other,
                });
            }
            c if c == quote => return Ok((value, start + 1 + j + 1)),
            c => value.push(c),
        }
    }
    Err(format!("unterminated {}", if quote == '\'' { "name" } else { "string" }))
}

/// `DECIMAL_VALUE? '.' (DECIMAL_VALUE | EXP_VALUE) | EXP_VALUE | DECIMAL_VALUE`.
/// `1..3` is two integers around a range operator.
fn number(bytes: &[u8], start: usize, text: &str) -> (Kind, usize) {
    let digits = |mut j: usize| {
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        j
    };
    let exponent = |j: usize| -> Option<usize> {
        if j < bytes.len() && (bytes[j] == b'e' || bytes[j] == b'E') {
            let mut k = j + 1;
            if k < bytes.len() && (bytes[k] == b'+' || bytes[k] == b'-') {
                k += 1;
            }
            let end = digits(k);
            if end > k {
                return Some(end);
            }
        }
        None
    };
    let mut end = digits(start);
    let mut real = false;
    if end < bytes.len() && bytes[end] == b'.' && bytes.get(end + 1).is_some_and(u8::is_ascii_digit) {
        end = digits(end + 1);
        real = true;
    }
    if let Some(after) = exponent(end) {
        end = after;
        real = true;
    }
    let kind = if real { Kind::Real(text[start..end].to_string()) } else { Kind::Integer(text[start..end].to_string()) };
    (kind, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<Kind> {
        tokenize(text).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn symbols_names_and_numbers() {
        assert_eq!(
            kinds("part <'P 1'> p :> q::r [1..*] = 2.5e3; // note"),
            vec![
                Kind::Word("part".into()),
                Kind::Symbol("<"),
                Kind::Name("P 1".into()),
                Kind::Symbol(">"),
                Kind::Word("p".into()),
                Kind::Symbol(":>"),
                Kind::Word("q".into()),
                Kind::Symbol("::"),
                Kind::Word("r".into()),
                Kind::Symbol("["),
                Kind::Integer("1".into()),
                Kind::Symbol(".."),
                Kind::Symbol("*"),
                Kind::Symbol("]"),
                Kind::Symbol("="),
                Kind::Real("2.5e3".into()),
                Kind::Symbol(";"),
            ]
        );
    }

    #[test]
    fn comments_strings_and_escapes() {
        assert_eq!(
            kinds("doc /* a\n b */ \"x\\\"y\" //* dropped */ ::> .?"),
            vec![Kind::Word("doc".into()), Kind::Comment(" a\n b ".into()), Kind::Str("x\"y".into()), Kind::Symbol("::>"), Kind::Symbol(".?")]
        );
        assert_eq!(kinds(".5 x.y 3..4"), vec![Kind::Real(".5".into()), Kind::Word("x".into()), Kind::Symbol("."), Kind::Word("y".into()), Kind::Integer("3".into()), Kind::Symbol(".."), Kind::Integer("4".into())]);
    }
}
