//! A minimal Office Open XML workbook (`.xlsx`) written by hand: one sheet
//! per table, a frozen and filterable header, the status colors of the
//! cells, column widths from the content. The parts are stored in a zip
//! without compression and with a fixed date, so the same tables always
//! give the same bytes. No dependency.

use super::export::{cell_text, grid};
use super::Table;

/// CRC-32 (IEEE 802.3), as zip wants it.
fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (n, slot) in table.iter_mut().enumerate() {
        let mut c = n as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *slot = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc = table[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

/// A zip archive of stored entries.
pub fn zip(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    const DOS_DATE: u16 = (1 << 5) | 1; // 1980-01-01
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let offset = out.len() as u32;
        let crc = crc32(data);
        let size = data.len() as u32;
        let header = |signature: u32, central_entry: bool| {
            let mut h = Vec::new();
            h.extend_from_slice(&signature.to_le_bytes());
            if central_entry {
                h.extend_from_slice(&20u16.to_le_bytes()); // made by
            }
            h.extend_from_slice(&20u16.to_le_bytes()); // needed to extract
            h.extend_from_slice(&0u16.to_le_bytes()); // flags
            h.extend_from_slice(&0u16.to_le_bytes()); // stored
            h.extend_from_slice(&0u16.to_le_bytes()); // time
            h.extend_from_slice(&DOS_DATE.to_le_bytes());
            h.extend_from_slice(&crc.to_le_bytes());
            h.extend_from_slice(&size.to_le_bytes());
            h.extend_from_slice(&size.to_le_bytes());
            h.extend_from_slice(&(name.len() as u16).to_le_bytes());
            h.extend_from_slice(&0u16.to_le_bytes()); // extra
            if central_entry {
                h.extend_from_slice(&0u16.to_le_bytes()); // comment
                h.extend_from_slice(&0u16.to_le_bytes()); // disk
                h.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
                h.extend_from_slice(&0u32.to_le_bytes()); // external attributes
                h.extend_from_slice(&offset.to_le_bytes());
            }
            h.extend_from_slice(name.as_bytes());
            h
        };
        out.extend(header(0x0403_4b50, false));
        out.extend_from_slice(data);
        central.extend(header(0x0201_4b50, true));
    }
    let central_offset = out.len() as u32;
    let central_size = central.len() as u32;
    out.extend(central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&central_size.to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

fn xml(text: &str) -> String {
    text.chars()
        .filter(|&c| c == '\t' || c == '\n' || c == '\r' || c >= ' ')
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

/// `A`, `B`, ... `Z`, `AA`...
fn column_name(mut index: usize) -> String {
    let mut name = String::new();
    loop {
        name.insert(0, (b'A' + (index % 26) as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    name
}

/// Style indexes of `styles.xml`.
const HEADER: usize = 1;
const OK: usize = 2;
const WARN: usize = 3;
const GAP: usize = 4;
const MONO: usize = 5;
const WRAP: usize = 6;

const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<fonts count="4"><font><sz val="11"/><name val="Calibri"/></font><font><b/><sz val="11"/><name val="Calibri"/></font><font><sz val="10"/><name val="Consolas"/></font><font><b/><sz val="11"/><color rgb="FF1D232B"/><name val="Calibri"/></font></fonts>
<fills count="6"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill><fill><patternFill patternType="solid"><fgColor rgb="FFEFECE5"/></patternFill></fill><fill><patternFill patternType="solid"><fgColor rgb="FFE3F3EA"/></patternFill></fill><fill><patternFill patternType="solid"><fgColor rgb="FFFDF1D8"/></patternFill></fill><fill><patternFill patternType="solid"><fgColor rgb="FFFBE5E1"/></patternFill></fill></fills>
<borders count="2"><border/><border><bottom style="thin"><color rgb="FFB9B3A6"/></bottom></border></borders>
<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
<cellXfs count="7"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"><alignment vertical="top"/></xf><xf numFmtId="0" fontId="3" fillId="2" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"><alignment vertical="center"/></xf><xf numFmtId="0" fontId="0" fillId="3" borderId="0" xfId="0" applyFill="1"><alignment vertical="top"/></xf><xf numFmtId="0" fontId="0" fillId="4" borderId="0" xfId="0" applyFill="1"><alignment vertical="top"/></xf><xf numFmtId="0" fontId="0" fillId="5" borderId="0" xfId="0" applyFill="1"><alignment vertical="top"/></xf><xf numFmtId="0" fontId="2" fillId="0" borderId="0" xfId="0" applyFont="1"><alignment vertical="top"/></xf><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0" applyAlignment="1"><alignment vertical="top" wrapText="1"/></xf></cellXfs>
<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
</styleSheet>"#;

fn status_style(status: Option<&str>) -> Option<usize> {
    match status? {
        "ok" => Some(OK),
        "warn" => Some(WARN),
        "gap" | "fail" => Some(GAP),
        _ => None,
    }
}

/// The style of each cell of the grid, header row first.
fn styles(table: &Table, rows: usize, columns: usize) -> Vec<Vec<usize>> {
    let mut out = vec![vec![HEADER; columns]];
    if let Some(matrix) = &table.matrix {
        for r in 0..rows.saturating_sub(1) {
            let mut line = vec![if matrix.empty_rows.contains(&r) { GAP } else { HEADER }];
            for c in 0..columns.saturating_sub(1) {
                line.push(if matrix.cells.iter().any(|cell| cell.row == r && cell.column == c) { OK } else { 0 });
            }
            out.push(line);
        }
        return out;
    }
    for row in &table.rows {
        out.push(
            table
                .columns
                .iter()
                .map(|column| {
                    let cell = row.cells.get(&column.key);
                    status_style(cell.and_then(|c| c.status.as_deref())).unwrap_or(if column.key == "id" { MONO } else if cell.is_some_and(|c| cell_text(c).len() > 48) { WRAP } else { 0 })
                })
                .collect(),
        );
    }
    out
}

fn sheet(table: &Table) -> String {
    let grid = grid(table);
    let columns = grid.first().map(Vec::len).unwrap_or(0);
    let styles = styles(table, grid.len(), columns);
    let widths: Vec<f64> = (0..columns)
        .map(|c| {
            let longest = grid.iter().map(|line| line.get(c).map(|t| t.chars().count()).unwrap_or(0)).max().unwrap_or(0);
            (longest as f64 + 2.0).clamp(6.0, 60.0)
        })
        .collect();
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">");
    let pane = if table.matrix.is_some() {
        "<pane xSplit=\"1\" ySplit=\"1\" topLeftCell=\"B2\" activePane=\"bottomRight\" state=\"frozen\"/>"
    } else {
        "<pane ySplit=\"1\" topLeftCell=\"A2\" activePane=\"bottomLeft\" state=\"frozen\"/>"
    };
    out.push_str(&format!("<sheetViews><sheetView workbookViewId=\"0\">{pane}</sheetView></sheetViews>"));
    if columns > 0 {
        out.push_str("<cols>");
        for (c, width) in widths.iter().enumerate() {
            out.push_str(&format!("<col min=\"{n}\" max=\"{n}\" width=\"{width:.1}\" customWidth=\"1\"/>", n = c + 1));
        }
        out.push_str("</cols>");
    }
    out.push_str("<sheetData>");
    for (r, line) in grid.iter().enumerate() {
        out.push_str(&format!("<row r=\"{}\">", r + 1));
        for (c, text) in line.iter().enumerate() {
            let style = styles.get(r).and_then(|s| s.get(c)).copied().unwrap_or(0);
            let reference = format!("{}{}", column_name(c), r + 1);
            if text.is_empty() {
                out.push_str(&format!("<c r=\"{reference}\" s=\"{style}\"/>"));
            } else {
                out.push_str(&format!("<c r=\"{reference}\" s=\"{style}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>", xml(text)));
            }
        }
        out.push_str("</row>");
    }
    out.push_str("</sheetData>");
    if columns > 0 && table.matrix.is_none() {
        out.push_str(&format!("<autoFilter ref=\"A1:{}{}\"/>", column_name(columns - 1), grid.len().max(1)));
    }
    out.push_str("</worksheet>");
    out
}

/// A sheet name Excel accepts: at most 31 characters, none of `[]:*?/\`, unique.
fn sheet_names(tables: &[Table]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for table in tables {
        let clean: String = table.name.chars().map(|c| if "[]:*?/\\".contains(c) { ' ' } else { c }).collect();
        let base: String = clean.trim().chars().take(31).collect();
        let base = if base.is_empty() { "Table".to_string() } else { base };
        let mut name = base.clone();
        let mut n = 2;
        while names.iter().any(|m| m.eq_ignore_ascii_case(&name)) {
            let suffix = format!(" ({n})");
            name = format!("{}{suffix}", base.chars().take(31 - suffix.len()).collect::<String>());
            n += 1;
        }
        names.push(name);
    }
    names
}

/// The workbook of the tables, one sheet each.
pub fn workbook(tables: &[Table]) -> Vec<u8> {
    let names = sheet_names(tables);
    let mut content_types = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>");
    let mut sheets = String::new();
    let mut relationships = String::new();
    for (i, name) in names.iter().enumerate() {
        let n = i + 1;
        content_types.push_str(&format!("<Override PartName=\"/xl/worksheets/sheet{n}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>"));
        sheets.push_str(&format!("<sheet name=\"{}\" sheetId=\"{n}\" r:id=\"rId{n}\"/>", xml(name)));
        relationships.push_str(&format!("<Relationship Id=\"rId{n}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{n}.xml\"/>"));
    }
    content_types.push_str("</Types>");
    let styles_id = names.len() + 1;
    relationships.push_str(&format!("<Relationship Id=\"rId{styles_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>"));
    let mut defined = String::new();
    for (i, table) in tables.iter().enumerate() {
        if table.matrix.is_none() {
            let columns = table.columns.len().max(1);
            defined.push_str(&format!("<definedName name=\"_xlnm._FilterDatabase\" localSheetId=\"{i}\" hidden=\"1\">'{}'!$A$1:${}${}</definedName>", xml(&names[i].replace('\'', "''")), column_name(columns - 1), table.rows.len() + 1));
        }
    }
    let defined = if defined.is_empty() { String::new() } else { format!("<definedNames>{defined}</definedNames>") };
    let workbook = format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets>{sheets}</sheets>{defined}</workbook>");
    let mut entries = vec![
        ("[Content_Types].xml".to_string(), content_types.into_bytes()),
        ("_rels/.rels".to_string(), "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>".as_bytes().to_vec()),
        ("xl/workbook.xml".to_string(), workbook.into_bytes()),
        ("xl/_rels/workbook.xml.rels".to_string(), format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{relationships}</Relationships>").into_bytes()),
        ("xl/styles.xml".to_string(), STYLES.as_bytes().to_vec()),
    ];
    for (i, table) in tables.iter().enumerate() {
        entries.push((format!("xl/worksheets/sheet{}.xml", i + 1), sheet(table).into_bytes()));
    }
    zip(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_and_column_names() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(column_name(0), "A");
        assert_eq!(column_name(25), "Z");
        assert_eq!(column_name(26), "AA");
        assert_eq!(column_name(701), "ZZ");
    }

    #[test]
    fn a_zip_ends_with_its_central_directory() {
        let bytes = zip(&[("a.txt".to_string(), b"hello".to_vec())]);
        assert_eq!(&bytes[0..4], &0x0403_4b50u32.to_le_bytes());
        let end = bytes.len() - 22;
        assert_eq!(&bytes[end..end + 4], &0x0605_4b50u32.to_le_bytes());
    }
}
