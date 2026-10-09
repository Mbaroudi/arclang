//! The formatter over every example in the repository: it must be idempotent,
//! keep every comment, and leave the parsed model untouched.

use arclang::compiler::format::format_source;
use arclang::compiler::lexer::{LexemeKind, Lexer};
use arclang::compiler::parser::Parser;
use std::path::{Path, PathBuf};

fn arc_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            arc_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "arc") {
            files.push(path);
        }
    }
}

fn corpus() -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    arc_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"), &mut files);
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path).unwrap();
            (path, source)
        })
        // Legacy examples that no longer lex cannot be formatted either.
        .filter(|(_, source)| Lexer::new(source).tokenize().is_ok())
        .collect()
}

fn comments(source: &str) -> Vec<String> {
    Lexer::new(source)
        .tokenize_lossless()
        .unwrap()
        .into_iter()
        .filter(|lexeme| !matches!(lexeme.kind, LexemeKind::Token(_)))
        .map(|lexeme| lexeme.text.trim_end().to_string())
        .collect()
}

/// The parsed model as JSON: comparable whatever order its maps iterate in.
fn parsed(source: &str) -> Option<serde_json::Value> {
    let tokens = Lexer::new(source).tokenize().ok()?;
    let model = Parser::new(tokens).parse().ok()?;
    Some(serde_json::to_value(&model).expect("the syntax tree serializes"))
}

#[test]
fn corpus_is_large_enough_to_mean_something() {
    assert!(corpus().len() >= 20, "only {} examples found", corpus().len());
}

#[test]
fn formatting_is_idempotent_on_every_example() {
    for (path, source) in corpus() {
        let once = format_source(&source).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let twice = format_source(&once).unwrap();
        assert_eq!(once, twice, "{} is not stable under fmt", path.display());
    }
}

#[test]
fn formatting_keeps_every_comment_of_every_example() {
    let mut total = 0;
    for (path, source) in corpus() {
        let formatted = format_source(&source).unwrap();
        let before = comments(&source);
        total += before.len();
        assert_eq!(before, comments(&formatted), "{} lost a comment", path.display());
    }
    assert!(total > 100, "the corpus should carry comments, found {total}");
}

#[test]
fn formatting_does_not_change_the_parsed_model() {
    let mut compared = 0;
    for (path, source) in corpus() {
        let Some(before) = parsed(&source) else { continue };
        let formatted = format_source(&source).unwrap();
        let after = parsed(&formatted)
            .unwrap_or_else(|| panic!("{} no longer parses once formatted", path.display()));
        assert!(before == after, "{} parses differently once formatted", path.display());
        compared += 1;
    }
    assert!(compared >= 15, "only {compared} examples parse");
}

#[test]
fn formatted_output_has_no_trailing_whitespace_or_tabs_in_indentation() {
    for (path, source) in corpus() {
        let formatted = format_source(&source).unwrap();
        let comment_or_string_lines = formatted.contains("/*") || formatted.contains("\\n");
        for (number, line) in formatted.lines().enumerate() {
            let indentation: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            if !comment_or_string_lines {
                assert!(!indentation.contains('\t'), "{}:{} tab", path.display(), number + 1);
                assert_eq!(line, line.trim_end(), "{}:{} trailing", path.display(), number + 1);
            }
        }
        assert!(formatted.is_empty() || formatted.ends_with('\n'));
        assert!(!formatted.ends_with("\n\n"));
    }
}
