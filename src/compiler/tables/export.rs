//! A computed table as text: CSV (RFC 4180), TSV (what pastes as a table
//! into a spreadsheet), Markdown and a standalone HTML page. A matrix is
//! written as its grid. The same table always gives the same bytes.

use super::{Cell, Table};

/// The text of a cell: its links by name (a verification case with its
/// verdict), else its text.
pub fn cell_text(cell: &Cell) -> String {
    if cell.links.is_empty() {
        return cell.text.clone();
    }
    cell.links
        .iter()
        .map(|l| match &l.verdict {
            Some(verdict) => format!("{} ({verdict})", l.name),
            None => l.name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The table as a grid of texts: header row first.
pub fn grid(table: &Table) -> Vec<Vec<String>> {
    if let Some(matrix) = &table.matrix {
        let mut out = Vec::new();
        let mut header = vec![String::new()];
        header.extend(matrix.columns.iter().map(|c| c.name.clone()));
        out.push(header);
        for (r, row) in matrix.rows.iter().enumerate() {
            let mut line = vec![row.name.clone()];
            for c in 0..matrix.columns.len() {
                let text = if matrix.diagonal && matrix.rows.get(r).map(|x| &x.uuid) == matrix.columns.get(c).map(|x| &x.uuid) {
                    "\u{25A0}".to_string()
                } else {
                    matrix.cells.iter().find(|cell| cell.row == r && cell.column == c).map(|cell| if matrix.diagonal { cell.items.join("; ") } else { cell.kinds.join("; ") }).unwrap_or_default()
                };
                line.push(text);
            }
            out.push(line);
        }
        return out;
    }
    let mut out = vec![table.columns.iter().map(|c| c.label.clone()).collect::<Vec<_>>()];
    for row in &table.rows {
        out.push(table.columns.iter().map(|c| row.cells.get(&c.key).map(cell_text).unwrap_or_default()).collect());
    }
    out
}

fn csv_field(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

pub fn csv(table: &Table) -> String {
    grid(table).iter().map(|line| line.iter().map(|f| csv_field(f)).collect::<Vec<_>>().join(",") + "\r\n").collect()
}

/// Tab-separated, one line per row: tabs and line breaks inside a cell
/// become spaces, so every spreadsheet reads it as the same grid.
pub fn tsv(table: &Table) -> String {
    grid(table)
        .iter()
        .map(|line| line.iter().map(|f| f.replace(['\t', '\n', '\r'], " ")).collect::<Vec<_>>().join("\t") + "\n")
        .collect()
}

pub fn markdown(table: &Table) -> String {
    let escape = |t: &str| t.replace('|', "\\|").replace('\n', " ");
    let grid = grid(table);
    let mut out = format!("## {}\n\n", table.name);
    out.push_str(&summary_line(table));
    out.push_str("\n\n");
    if let Some((header, body)) = grid.split_first() {
        out.push_str(&format!("| {} |\n", header.iter().map(|h| escape(h)).collect::<Vec<_>>().join(" | ")));
        out.push_str(&format!("|{}\n", header.iter().map(|_| " --- |").collect::<String>()));
        for line in body {
            out.push_str(&format!("| {} |\n", line.iter().map(|h| escape(h)).collect::<Vec<_>>().join(" | ")));
        }
    }
    out
}

/// `12 rows: 9 ok, 2 gaps, 1 failing. Gate ISO26262: blocked (3 blockers).`
pub fn summary_line(table: &Table) -> String {
    let s = &table.summary;
    let mut parts = vec![format!("{} ok", s.ok)];
    if s.warn > 0 {
        parts.push(format!("{} pending", s.warn));
    }
    if s.gap > 0 {
        parts.push(format!("{} with a gap", s.gap));
    }
    if s.fail > 0 {
        parts.push(format!("{} failing", s.fail));
    }
    let gate = if s.gate_passed { "passed".to_string() } else { format!("blocked ({} blocker{} about these rows)", s.gate_blockers, if s.gate_blockers == 1 { "" } else { "s" }) };
    format!("{} row{}: {}. Gate {}: {gate}.", s.rows, if s.rows == 1 { "" } else { "s" }, parts.join(", "), s.gate_standard)
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

const STYLE: &str = r#"
:root { --bg:#fbfaf7; --fg:#1d232b; --muted:#5e6875; --line:#dcd8cf; --head:#efece5; --ok:#1f7a4d; --ok-bg:#e3f3ea; --warn:#8a5a00; --warn-bg:#fdf1d8; --gap:#a3271c; --gap-bg:#fbe5e1; --link:#1957a6; }
@media (prefers-color-scheme: dark) { :root { --bg:#16191d; --fg:#e7e9ec; --muted:#9aa3ad; --line:#323840; --head:#1f242a; --ok:#6fd39f; --ok-bg:#163327; --warn:#f0c46a; --warn-bg:#3a2f15; --gap:#ff8e80; --gap-bg:#3d1c19; --link:#7fb2ff; } }
body { margin: 0; padding: 24px; background: var(--bg); color: var(--fg); font: 13px/1.45 "IBM Plex Sans", "Segoe UI", system-ui, sans-serif; }
h1 { font-size: 18px; margin: 0 0 4px; } p.summary { color: var(--muted); margin: 0 0 16px; }
table { border-collapse: collapse; width: 100%; } th, td { border-bottom: 1px solid var(--line); padding: 5px 8px; text-align: left; vertical-align: top; }
th { position: sticky; top: 0; background: var(--head); font-weight: 600; font-size: 12px; letter-spacing: .02em; }
td.id, td.mono { font-family: "IBM Plex Mono", ui-monospace, monospace; font-size: 12px; white-space: nowrap; }
tr.gap td:first-child, tr.fail td:first-child { box-shadow: inset 3px 0 0 var(--gap); } tr.warn td:first-child { box-shadow: inset 3px 0 0 var(--warn); }
.badge { display: inline-block; padding: 0 7px; border-radius: 9px; font-size: 11px; font-weight: 600; }
.ok { color: var(--ok); background: var(--ok-bg); } .warn { color: var(--warn); background: var(--warn-bg); } .gap, .fail { color: var(--gap); background: var(--gap-bg); }
a { color: var(--link); text-decoration: none; } a:hover { text-decoration: underline; }
td.cell-x { text-align: center; } th.rot { writing-mode: vertical-rl; transform: rotate(180deg); white-space: nowrap; }
"#;

pub fn html(tables: &[Table], title: &str) -> String {
    let mut body = String::new();
    for table in tables {
        body.push_str(&format!("<section><h1>{}</h1><p class=\"summary\">{}</p>", html_escape(&table.name), html_escape(&summary_line(table))));
        body.push_str(&html_table(table));
        body.push_str("</section>\n");
    }
    format!("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title><style>{STYLE}</style></head><body>\n{body}</body></html>\n", html_escape(title))
}

fn link_html(link: &super::Link) -> String {
    let verdict = link.verdict.as_ref().map(|v| {
        let class = match v.as_str() { "pass" => "ok", "fail" => "fail", _ => "warn" };
        format!(" <span class=\"badge {class}\">{}</span>", html_escape(v))
    });
    format!("<a href=\"#{}\" title=\"{} {}\">{}</a>{}", html_escape(&link.id), html_escape(&link.kind), html_escape(&link.id), html_escape(&link.name), verdict.unwrap_or_default())
}

fn html_table(table: &Table) -> String {
    let mut out = String::from("<table>");
    if let Some(matrix) = &table.matrix {
        out.push_str("<thead><tr><th></th>");
        for column in &matrix.columns {
            out.push_str(&format!("<th class=\"rot\">{}</th>", link_html(column)));
        }
        out.push_str("</tr></thead><tbody>");
        for (r, row) in matrix.rows.iter().enumerate() {
            let class = if matrix.empty_rows.contains(&r) { " class=\"gap\"" } else { "" };
            out.push_str(&format!("<tr{class}><td>{}</td>", link_html(row)));
            for c in 0..matrix.columns.len() {
                let found = matrix.cells.iter().find(|cell| cell.row == r && cell.column == c);
                let text = found.map(|cell| if matrix.diagonal { cell.items.join("<br>") } else { cell.kinds.join(", ") }).unwrap_or_default();
                out.push_str(&format!("<td class=\"cell-x\">{text}</td>"));
            }
            out.push_str("</tr>");
        }
        out.push_str("</tbody></table>");
        return out;
    }
    out.push_str("<thead><tr>");
    for column in &table.columns {
        out.push_str(&format!("<th>{}</th>", html_escape(&column.label)));
    }
    out.push_str("</tr></thead><tbody>");
    for row in &table.rows {
        out.push_str(&format!("<tr id=\"{}\" class=\"{}\">", html_escape(&row.id), html_escape(&row.status)));
        for column in &table.columns {
            let cell = row.cells.get(&column.key).cloned().unwrap_or_default();
            let inner = if !cell.links.is_empty() {
                cell.links.iter().map(link_html).collect::<Vec<_>>().join(", ")
            } else if let Some(status) = &cell.status {
                format!("<span class=\"badge {}\">{}</span>", html_escape(status), html_escape(if cell.text.is_empty() { "none" } else { &cell.text }))
            } else {
                html_escape(&cell.text)
            };
            let class = if column.key == "id" { " class=\"id\"" } else { "" };
            out.push_str(&format!("<td{class}>{inner}</td>"));
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody></table>");
    out
}
