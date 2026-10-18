//! The tokens of the dialect: words, quoted names, strings, numbers and
//! symbols, comments dropped except a `doc` body.

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Token {
    Word(String),
    /// A name written between single quotes.
    Quoted(String),
    Text(String),
    Number(String),
    Symbol(&'static str),
    /// `doc /* body */`
    Doc(String),
}

impl Token {
    pub(super) fn source(&self) -> String {
        match self {
            Token::Word(text) | Token::Number(text) => text.clone(),
            Token::Quoted(name) => format!("'{}'", name.replace('\\', "\\\\").replace('\'', "\\'")),
            Token::Text(text) => text.clone(),
            Token::Symbol(symbol) => (*symbol).to_string(),
            Token::Doc(body) => format!("doc /* {} */", body),
        }
    }
}

const SYMBOLS: &[&str] = &[
    ":>>", "::", ":>", "<=", ">=", "==", "!=", "{", "}", ";", ",", ".", "(", ")", "[", "]", ":", "=", "<", ">",
    "+", "-", "*", "/", "#",
];

pub(super) fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    let rest_starts = |i: usize, pattern: &str| pattern.chars().enumerate().all(|(k, c)| chars.get(i + k) == Some(&c));
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if rest_starts(i, "//") {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if rest_starts(i, "/*") {
            let start = i + 2;
            let mut end = start;
            while end < chars.len() && !rest_starts(end, "*/") {
                end += 1;
            }
            if end >= chars.len() {
                return Err("unterminated comment".to_string());
            }
            // A comment is the body of the `doc` it follows; otherwise a note.
            if tokens.last() == Some(&Token::Word("doc".to_string())) {
                tokens.pop();
                let body: String = chars[start..end].iter().collect();
                tokens.push(Token::Doc(body.trim().to_string()));
            }
            i = end + 2;
        } else if c == '\'' || c == '"' {
            let mut value = String::new();
            let mut raw = String::from(c);
            i += 1;
            loop {
                match chars.get(i) {
                    None => return Err("unterminated name or string".to_string()),
                    Some('\\') => {
                        let escaped = *chars.get(i + 1).ok_or("unterminated escape")?;
                        value.push(escaped);
                        raw.push('\\');
                        raw.push(escaped);
                        i += 2;
                    }
                    Some(end) if *end == c => {
                        raw.push(c);
                        i += 1;
                        break;
                    }
                    Some(other) => {
                        value.push(*other);
                        raw.push(*other);
                        i += 1;
                    }
                }
            }
            tokens.push(if c == '\'' { Token::Quoted(value) } else { Token::Text(raw) });
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token::Word(chars[start..i].iter().collect()));
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric()
                    || (chars[i] == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())))
            {
                i += 1;
            }
            tokens.push(Token::Number(chars[start..i].iter().collect()));
        } else if let Some(symbol) = SYMBOLS.iter().find(|symbol| rest_starts(i, symbol)) {
            tokens.push(Token::Symbol(symbol));
            i += symbol.len();
        } else {
            return Err(format!("unexpected character '{}'", c));
        }
    }
    Ok(tokens)
}

pub(super) fn is_symbol(token: Option<&Token>, symbol: &str) -> bool {
    matches!(token, Some(Token::Symbol(s)) if *s == symbol)
}

pub(super) fn is_word(token: Option<&Token>, word: &str) -> bool {
    matches!(token, Some(Token::Word(w)) if w == word)
}

/// Source text of an expression, spaced the way the exporter writes it.
pub(super) fn expression_text(tokens: &[Token]) -> String {
    let mut text = String::new();
    for (index, token) in tokens.iter().enumerate() {
        let tight_before = matches!(token, Token::Symbol("." | "," | ")" | "]" | "::"));
        let tight_after = index > 0 && matches!(tokens[index - 1], Token::Symbol("." | "(" | "[" | "::"));
        // `f(x)`: a call keeps its parenthesis attached.
        let call = matches!(token, Token::Symbol("(")) && index > 0 && matches!(tokens[index - 1], Token::Word(_));
        if index > 0 && !tight_before && !tight_after && !call {
            text.push(' ');
        }
        text.push_str(&token.source());
    }
    text
}
