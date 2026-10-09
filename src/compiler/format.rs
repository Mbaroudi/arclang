//! Source formatter (`arclang fmt`).
//!
//! The formatter works on the lossless token stream, not on the syntax tree:
//! it rewrites the whitespace between tokens and nothing else. Comments, the
//! order of declarations, the spelling of literals and the author's line
//! breaks all survive, because none of them is ever rebuilt from the model.
//!
//! What it normalizes:
//! - indentation, from bracket nesting (four spaces per level);
//! - spacing inside a line (`key: value`, `[a, b]`, `{ a: 1 }`, `a -> b`);
//! - blank lines (at most one, none right after `{` or right before `}`);
//! - trailing whitespace and the final newline.
//!
//! It never moves a token to another line. Every result is checked before it
//! is returned: the formatted text must lex to exactly the same tokens and
//! comments as the input, otherwise formatting fails instead of writing.

use super::lexer::{Lexeme, LexemeKind, Lexer, Token};

const INDENT: &str = "    ";

/// Format ArcLang source. Fails on source that does not lex, and on any
/// result that would not carry the same tokens and comments as the input.
pub fn format_source(source: &str) -> Result<String, String> {
    let lexemes = Lexer::new(source).tokenize_lossless()?;
    let formatted = layout(&lexemes);
    verify(&lexemes, &formatted)?;
    Ok(formatted)
}

/// Whether `source` is already in formatted form.
pub fn is_formatted(source: &str) -> Result<bool, String> {
    Ok(format_source(source)? == source)
}

fn layout(lexemes: &[Lexeme]) -> String {
    let mut out = String::new();
    // Indentation of the line each unclosed bracket was opened on.
    let mut open_lines: Vec<usize> = Vec::new();
    let mut line_indent = 0;

    for (index, lexeme) in lexemes.iter().enumerate() {
        let previous = index.checked_sub(1).map(|i| &lexemes[i]);

        match previous {
            None => {}
            Some(prev) if starts_line(prev, lexeme) => {
                out.push('\n');
                if lexeme.newlines_before > 1 && !is_opener(prev) && !is_closer(lexeme) {
                    out.push('\n');
                }
                line_indent = indent_for_line(&lexemes[index..], &open_lines);
                out.push_str(&INDENT.repeat(line_indent));
            }
            Some(prev) => {
                let path_separator = is_path_separator(lexemes, index - 1);
                if !path_separator && needs_space(prev, lexeme, lexemes.get(index + 1)) {
                    out.push(' ');
                }
            }
        }

        out.push_str(lexeme_text(lexeme));

        if is_opener(lexeme) {
            open_lines.push(line_indent);
        } else if is_closer(lexeme) {
            open_lines.pop();
        }
    }

    if !out.is_empty() {
        out.push('\n');
    }
    out
}

/// A line comment always ends its line; otherwise the author's break is kept.
fn starts_line(previous: &Lexeme, lexeme: &Lexeme) -> bool {
    lexeme.newlines_before > 0 || previous.kind == LexemeKind::LineComment
}

/// Indentation of a line: one level inside the innermost open bracket, or the
/// level of the line that opened the bracket this line starts by closing.
fn indent_for_line(line: &[Lexeme], open_lines: &[usize]) -> usize {
    let mut remaining = open_lines.len();
    let mut indent = open_lines.last().map_or(0, |level| level + 1);
    for (position, lexeme) in line.iter().enumerate() {
        let on_this_line = position == 0 || lexeme.newlines_before == 0;
        if !on_this_line || !is_closer(lexeme) || remaining == 0 {
            break;
        }
        remaining -= 1;
        indent = open_lines[remaining];
    }
    indent
}

fn lexeme_text(lexeme: &Lexeme) -> &str {
    match lexeme.kind {
        // A line comment's text runs to the end of the line; drop what the
        // eye cannot see (trailing spaces, a carriage return).
        LexemeKind::LineComment => lexeme.text.trim_end(),
        LexemeKind::BlockComment | LexemeKind::Token(_) => &lexeme.text,
    }
}

fn token(lexeme: &Lexeme) -> Option<&Token> {
    match &lexeme.kind {
        LexemeKind::Token(token) => Some(token),
        LexemeKind::LineComment | LexemeKind::BlockComment => None,
    }
}

fn is_opener(lexeme: &Lexeme) -> bool {
    matches!(
        token(lexeme),
        Some(Token::LeftBrace | Token::LeftBracket | Token::LeftParen)
    )
}

fn is_closer(lexeme: &Lexeme) -> bool {
    matches!(
        token(lexeme),
        Some(Token::RightBrace | Token::RightBracket | Token::RightParen)
    )
}

/// Operators whose spacing carries meaning for the reader: `130 km/h` is a
/// unit, `a / b` a division, `Fusion.out` a path. Written tight they stay
/// tight; written with any space they get one on each side.
fn is_tightness_preserving(token: &Token) -> bool {
    matches!(
        token,
        Token::Dot | Token::Slash | Token::Star | Token::Plus | Token::Minus
    )
}

/// Whether the lexeme at `index` is the second colon of a tight `::`, as in
/// `Units::Speed`: nothing follows it but the next path segment.
fn is_path_separator(lexemes: &[Lexeme], index: usize) -> bool {
    let is_colon = |lexeme: &Lexeme| matches!(token(lexeme), Some(Token::Colon));
    let colon = &lexemes[index];
    let tight = !colon.space_before && colon.newlines_before == 0;
    let follower_tight = lexemes
        .get(index + 1)
        .is_some_and(|next| !next.space_before && next.newlines_before == 0);
    index > 0 && is_colon(colon) && is_colon(&lexemes[index - 1]) && tight && follower_tight
}

fn is_opening(token: &Token) -> bool {
    matches!(token, Token::LeftBracket | Token::LeftParen | Token::Colon | Token::Comma)
}

fn is_word(token: &Token) -> bool {
    matches!(token, Token::Identifier(_)) || token.keyword_text().is_some()
}

/// Whether one space separates two lexemes on the same line. Removing a space
/// the author wrote is only done where it cannot join two tokens into one.
fn needs_space(previous: &Lexeme, lexeme: &Lexeme, next: Option<&Lexeme>) -> bool {
    let (Some(left), Some(right)) = (token(previous), token(lexeme)) else {
        return true; // comments are always set off by a space
    };

    if is_tightness_preserving(right) {
        let tight_after = next.is_some_and(|n| !n.space_before && n.newlines_before == 0);
        return lexeme.space_before || !tight_after;
    }
    if is_tightness_preserving(left) {
        return lexeme.space_before || previous.space_before;
    }

    match (left, right) {
        // `SF-004` written bare lexes as a word and a negative number; keep
        // it in one piece, as the author wrote it.
        (_, Token::Number(_)) if lexeme.text.starts_with('-') && !is_opening(left) => {
            lexeme.space_before
        }
        // `40%` or `40 %`: as the author wrote it.
        (Token::Number(_), Token::Percent) => lexeme.space_before,
        (_, Token::Colon | Token::Comma) => false,
        (Token::LeftBrace, Token::RightBrace) => false,
        (Token::LeftBracket | Token::LeftParen, _) => false,
        (_, Token::RightBracket | Token::RightParen) => false,
        // `sum(...)`: a call keeps its parenthesis attached.
        (word, Token::LeftParen) if is_word(word) => lexeme.space_before,
        _ => true,
    }
}

/// The formatter may only change whitespace. Re-lex the result and compare.
fn verify(original: &[Lexeme], formatted: &str) -> Result<(), String> {
    let relexed = Lexer::new(formatted)
        .tokenize_lossless()
        .map_err(|e| format!("internal formatter error: result does not lex ({e})"))?;

    if original.len() != relexed.len() {
        return Err(format!(
            "internal formatter error: {} tokens and comments in, {} out",
            original.len(),
            relexed.len()
        ));
    }
    for (before, after) in original.iter().zip(&relexed) {
        if before.kind != after.kind || lexeme_text(before) != lexeme_text(after) {
            return Err(format!(
                "internal formatter error: `{}` at {} would become `{}`",
                before.text, before.span, after.text
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(source: &str) -> String {
        format_source(source).expect("source formats")
    }

    #[test]
    fn indents_from_nesting() {
        let source = "system_analysis \"S\" {\nfunction \"F\" {\nid: \"F1\"\n}\n}\n";
        assert_eq!(
            fmt(source),
            "system_analysis \"S\" {\n    function \"F\" {\n        id: \"F1\"\n    }\n}\n"
        );
    }

    #[test]
    fn normalizes_spacing_inside_a_line() {
        assert_eq!(
            fmt("actor   \"A\"{id :\"X\" ,tags:[ \"a\",\"b\" ]}"),
            "actor \"A\" { id: \"X\", tags: [\"a\", \"b\"] }\n"
        );
        assert_eq!(fmt("actor \"A\" {  }"), "actor \"A\" {}\n");
        assert_eq!(fmt("connect a->b"), "connect a -> b\n");
    }

    #[test]
    fn keeps_every_comment() {
        let source = "// header\n\n\n/* block\n   kept as is */\nactor \"A\" {   // trailing\n// inside\nid: \"X\" /* inline */\n   // before close\n}\n";
        assert_eq!(
            fmt(source),
            "// header\n\n/* block\n   kept as is */\nactor \"A\" { // trailing\n    // inside\n    id: \"X\" /* inline */\n    // before close\n}\n"
        );
    }

    #[test]
    fn keeps_unit_and_path_spelling() {
        assert_eq!(fmt("speed:130 km/h"), "speed: 130 km/h\n");
        assert_eq!(fmt("assert: a/b<= 2 *c"), "assert: a/b <= 2 * c\n");
        assert_eq!(fmt("to_port: Fusion.out"), "to_port: Fusion.out\n");
        assert_eq!(fmt("load:40%\nmargin : 5  %"), "load: 40%\nmargin: 5 %\n");
        assert_eq!(fmt("type : Units::Speed"), "type: Units::Speed\n");
        assert_eq!(fmt("assert: sum( a ,b )- 3 > 0"), "assert: sum(a, b) - 3 > 0\n");
    }

    #[test]
    fn keeps_bare_hyphenated_identifiers_in_one_piece() {
        assert_eq!(fmt("traces:[STK-001 ,SF-004]"), "traces: [STK-001, SF-004]\n");
        assert_eq!(fmt("offset:-5"), "offset: -5\n");
        assert_eq!(fmt("assert: a - 3 > b -2"), "assert: a - 3 > b -2\n");
    }

    #[test]
    fn collapses_blank_lines() {
        assert_eq!(
            fmt("\n\nactor \"A\" {\n\n\nid: \"X\"\n\n\n\nname: \"N\"\n\n}\n\n\n"),
            "actor \"A\" {\n    id: \"X\"\n\n    name: \"N\"\n}\n"
        );
    }

    #[test]
    fn closing_brackets_return_to_the_opening_line() {
        assert_eq!(
            fmt("steps: [{\na: 1\n}, {\nb: 2\n}]"),
            "steps: [{\n    a: 1\n}, {\n    b: 2\n}]\n"
        );
        assert_eq!(fmt("x: [\n1,\n2\n]"), "x: [\n    1,\n    2\n]\n");
    }

    #[test]
    fn keeps_literal_spelling() {
        let source = "description: \"a \\\"quoted\\\" word\"\nbudget: 1_000 ms\n";
        assert_eq!(fmt(source), source);
    }

    #[test]
    fn is_idempotent() {
        let source = "actor \"A\"{ // c\nid:\"X\"\n\n\n}\ntype \"T\" { speed: 3 m/s }";
        let once = fmt(source);
        assert_eq!(fmt(&once), once);
        assert!(is_formatted(&once).unwrap());
        assert!(!is_formatted(source).unwrap());
    }

    #[test]
    fn empty_source_stays_empty() {
        assert_eq!(fmt(""), "");
        assert_eq!(fmt("  \n\n"), "");
    }

    #[test]
    fn rejects_source_that_does_not_lex() {
        assert!(format_source("actor \"unterminated").is_err());
        assert!(format_source("/* never closed").is_err());
    }
}
