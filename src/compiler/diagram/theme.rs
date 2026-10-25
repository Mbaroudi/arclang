//! Diagram colour themes for the CLI: the built-in presets and the rules a
//! theme must pass before `arclang diagram --theme` draws with it.
//!
//! The viewer's `viewer/arcviz-theme.js` (colour science in
//! `viewer/arcviz-color.js`) is the reference; this module mirrors its rules
//! and messages so that the CLI refuses the same files the viewer and the
//! workbench refuse. `tests/fixtures/themes` holds the verdicts both must
//! reach (checked here and by tools/diagram_render/theme.test.mjs).

use serde_json::{Map, Value};
use std::path::Path;

/// The presets, by id, as their JSON files (the viewer reads the same files).
pub const PRESETS: [(&str, &str); 6] = [
    ("arclang-default", include_str!("viewer/themes/arclang-default.json")),
    ("capella-classic", include_str!("viewer/themes/capella-classic.json")),
    ("colorblind-safe", include_str!("viewer/themes/colorblind-safe.json")),
    ("print-grayscale", include_str!("viewer/themes/print-grayscale.json")),
    ("high-contrast", include_str!("viewer/themes/high-contrast.json")),
    ("monochrome-document", include_str!("viewer/themes/monochrome-document.json")),
];

const VERSION: u64 = 1;

/// Concept families: key, label, silhouette (see CONCEPTS in arcviz-theme.js).
const CONCEPTS: [(&str, &str, &str); 19] = [
    ("operational_entity", "operational entity or actor", "rect"),
    ("operational_activity", "operational activity", "rect"),
    ("operational_capability", "operational capability", "pill"),
    ("system_actor", "actor", "rect"),
    ("system_component", "system component", "rect"),
    ("system_function", "system function", "rect"),
    ("logical_component", "logical component", "rect"),
    ("logical_function", "logical function", "rect"),
    ("physical_node", "physical node", "rect"),
    ("behaviour_component", "behaviour component", "rect"),
    ("physical_function", "physical function", "rect"),
    ("capability", "capability", "pill"),
    ("mission", "mission", "pill"),
    ("functional_chain", "functional chain", "rect"),
    ("data_type", "class or data type", "rect"),
    ("exchange_item", "exchange item", "rect"),
    ("state_mode", "state or mode", "pill"),
    ("requirement", "requirement", "rect"),
    ("configuration_item", "configuration item", "rect"),
];

const EXCHANGES: [(&str, &str); 11] = [
    ("functional_exchange", "functional exchange"),
    ("component_exchange", "component exchange"),
    ("physical_exchange", "physical exchange"),
    ("interaction", "interaction"),
    ("communication_mean", "communication mean"),
    ("physical_link", "physical link"),
    ("physical_path", "physical path"),
    ("transition", "transition"),
    ("message", "message"),
    ("trace", "trace"),
    ("relation", "relation"),
];

const SAME_CONCEPT: [&[&str]; 2] = [
    &["system_function", "logical_function", "physical_function"],
    &["system_component", "logical_component", "behaviour_component"],
];

/// What each view draws together: name, concepts, exchanges.
const VIEWS: [(&str, &[&str], &[&str]); 8] = [
    ("Operational", &["operational_entity", "operational_activity"], &["interaction", "communication_mean"]),
    ("System", &["system_actor", "system_component", "system_function"], &["functional_exchange", "component_exchange", "trace"]),
    ("Logical", &["system_actor", "logical_component", "logical_function"], &["functional_exchange", "component_exchange", "trace"]),
    (
        "Physical",
        &["system_actor", "physical_node", "behaviour_component", "physical_function"],
        &["functional_exchange", "component_exchange", "physical_exchange", "physical_link", "physical_path", "trace"],
    ),
    ("Modes and states", &["state_mode"], &["transition"]),
    (
        "Capabilities",
        &["mission", "capability", "operational_capability", "system_actor", "operational_entity", "system_function", "functional_chain"],
        &["relation", "trace"],
    ),
    ("Data", &["data_type", "exchange_item"], &["relation"]),
    ("Product breakdown", &["configuration_item", "requirement"], &["relation"]),
];

/// Colours the tool keeps: name, colour, also reserved on lines.
const RESERVED: [(&str, &str, bool); 4] = [
    ("error red", "#E51400", false),
    ("warning amber", "#BF8803", false),
    ("selection accent", "#0B5CAD", true),
    ("AI-proposal purple", "#8A5CF5", true),
];

const DISTINCT: f64 = 10.0;
const RESERVED_LIMIT: f64 = 10.0;
const CVD_ERROR: f64 = 5.0;
const CVD_WARNING: f64 = 8.0;
const GRAY_ERROR: f64 = 5.0;
const GRAY_WARNING: f64 = 8.0;
const TEXT: f64 = 4.5;
const GRAPHICS: f64 = 3.0;
const FILLS: usize = 16;
const FILLS_WARNING: usize = 13;
const LINES: usize = 10;
const SAME_COLOUR: f64 = 2.0;
const STYLES: [&str; 3] = ["solid", "dashed", "dotted"];

/// One finding of the validator.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub rule: &'static str,
    pub message: String,
}

/// The validator's verdict; `theme` is the complete theme when it passed.
#[derive(Debug, Clone)]
pub struct Verdict {
    pub ok: bool,
    pub errors: Vec<Finding>,
    pub warnings: Vec<Finding>,
    pub theme: Option<Value>,
}

/// A preset by id.
pub fn preset(id: &str) -> Option<Value> {
    PRESETS
        .iter()
        .find(|(name, _)| *name == id)
        .map(|(_, text)| serde_json::from_str(text).expect("the built-in themes are JSON"))
}

/// The theme `--theme` names: a preset id, or a theme file. A file the
/// validator refuses is an error that lists every reason.
pub fn resolve(argument: &str) -> Result<Value, String> {
    if let Some(theme) = preset(argument) {
        return Ok(theme);
    }
    let path = Path::new(argument);
    if !path.exists() {
        let ids: Vec<&str> = PRESETS.iter().map(|(id, _)| *id).collect();
        return Err(format!("no theme '{argument}': name a preset ({}) or a theme file", ids.join(", ")));
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("the theme {} cannot be read: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| format!("the theme {} is not JSON: {e}", path.display()))?;
    let verdict = validate(&value);
    match verdict.theme {
        Some(theme) if verdict.ok => Ok(theme),
        _ => Err(format!(
            "the theme {} is refused:\n  - {}",
            path.display(),
            verdict.errors.iter().map(|f| f.message.as_str()).collect::<Vec<_>>().join("\n  - ")
        )),
    }
}

struct Report {
    errors: Vec<Finding>,
    warnings: Vec<Finding>,
}

impl Report {
    fn error(&mut self, rule: &'static str, message: String) {
        if !self.errors.iter().any(|f| f.message == message) {
            self.errors.push(Finding { rule, message });
        }
    }
    fn warning(&mut self, rule: &'static str, message: String) {
        if !self.warnings.iter().any(|f| f.message == message) {
            self.warnings.push(Finding { rule, message });
        }
    }
}

/// Check a theme (the parsed JSON of a theme file).
pub fn validate(theme: &Value) -> Verdict {
    let mut report = Report { errors: Vec::new(), warnings: Vec::new() };
    check_structure(theme, &mut report);
    if !report.errors.is_empty() {
        return Verdict { ok: false, errors: report.errors, warnings: report.warnings, theme: None };
    }
    let full = complete(theme);
    check_complete(&full, "", &mut report);
    let dark = dark_of(&full);
    if let Some(dark) = &dark {
        check_complete(dark, "dark variant: ", &mut report);
    }
    if !report.errors.is_empty() {
        return Verdict { ok: false, errors: report.errors, warnings: report.warnings, theme: None };
    }
    check_rules(&full, "", &mut report);
    if let Some(dark) = &dark {
        check_rules(dark, "dark variant: ", &mut report);
    }
    Verdict { ok: report.errors.is_empty(), errors: report.errors, warnings: report.warnings, theme: Some(full) }
}

// --- structure -----------------------------------------------------------

fn json_text(value: Option<&Value>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "undefined".to_string())
}

fn check_structure(theme: &Value, report: &mut Report) {
    let Some(object) = theme.as_object() else {
        report.error("structure", "a theme is a JSON object".to_string());
        return;
    };
    if object.get("version").and_then(Value::as_u64) != Some(VERSION) || !object.get("version").is_some_and(Value::is_number) {
        report.error("structure", format!("version must be {VERSION} (found {})", json_text(object.get("version"))));
    }
    match object.get("name").and_then(Value::as_str) {
        Some(name) if !name.trim().is_empty() && name.chars().count() <= 80 => {}
        _ => report.error("structure", "name must be a text of 1 to 80 characters".to_string()),
    }
    if let Some(id) = object.get("id") {
        let good = id.as_str().is_some_and(|id| {
            let mut chars = id.chars();
            id.len() <= 64
                && chars.next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        });
        if !good {
            report.error("structure", "id must be lower-case letters, digits and dashes".to_string());
        }
    }
    if let Some(base) = object.get("extends") {
        if base.as_str().and_then(preset).is_none() {
            let ids: Vec<&str> = PRESETS.iter().map(|(id, _)| *id).collect();
            let shown = base.as_str().map(str::to_string).unwrap_or_else(|| base.to_string());
            report.error("structure", format!("extends names no preset: \"{shown}\" (presets: {})", ids.join(", ")));
        }
    }
    const ALLOWED: [&str; 13] = ["version", "id", "name", "description", "source", "extends", "background", "text", "muted", "concepts", "exchanges", "markings", "dark"];
    for key in object.keys() {
        if !ALLOWED.contains(&key.as_str()) {
            report.error("structure", format!("unknown field \"{key}\""));
        }
    }
    check_palette(object, "", report);
    if let Some(dark) = object.get("dark") {
        match dark.as_object() {
            Some(dark) => check_palette(dark, "dark.", report),
            None => report.error("structure", "dark must be an object: the dark variant".to_string()),
        }
    }
}

fn check_colour(value: Option<&Value>, place: &str, report: &mut Report) {
    if let Some(value) = value {
        if value.as_str().and_then(parse).is_none() {
            report.error("structure", format!("{place} is not a colour: {value} (write #RRGGBB)"));
        }
    }
}

fn check_palette(part: &Map<String, Value>, prefix: &str, report: &mut Report) {
    for key in ["background", "text", "muted"] {
        check_colour(part.get(key), &format!("{prefix}{key}"), report);
    }
    let concept_keys: Vec<&str> = CONCEPTS.iter().map(|c| c.0).collect();
    let exchange_keys: Vec<&str> = EXCHANGES.iter().map(|e| e.0).collect();
    let sections: [(&str, &Vec<&str>, &[&str]); 2] = [
        ("concepts", &concept_keys, &["fill", "stroke", "text", "border"]),
        ("exchanges", &exchange_keys, &["line", "width", "style", "text"]),
    ];
    for (section, known, fields) in sections {
        let Some(entries) = part.get(section) else { continue };
        let Some(entries) = entries.as_object() else {
            report.error("structure", format!("{prefix}{section} must be an object"));
            continue;
        };
        for (key, value) in entries {
            let place = format!("{prefix}{section}.{key}");
            if !known.contains(&key.as_str()) {
                let what = if section == "concepts" { "concept" } else { "exchange" };
                report.error("structure", format!("unknown {what} \"{key}\" in {prefix}{section}"));
                continue;
            }
            let Some(value) = value.as_object() else {
                report.error("structure", format!("{place} must be an object"));
                continue;
            };
            for field in value.keys() {
                if !fields.contains(&field.as_str()) {
                    report.error("structure", format!("unknown field \"{field}\" in {place}"));
                }
            }
            for field in ["fill", "stroke", "text", "line"] {
                check_colour(value.get(field), &format!("{place}.{field}"), report);
            }
            for field in ["style", "border"] {
                if let Some(style) = value.get(field) {
                    if !style.as_str().is_some_and(|s| STYLES.contains(&s)) {
                        report.error("structure", format!("{place}.{field} must be solid, dashed or dotted"));
                    }
                }
            }
            if let Some(width) = value.get("width") {
                if !width.as_f64().is_some_and(|w| (0.5..=6.0).contains(&w)) {
                    report.error("structure", format!("{place}.width must be a number from 0.5 to 6"));
                }
            }
        }
    }
    if let Some(markings) = part.get("markings") {
        let Some(markings) = markings.as_object() else {
            report.error("structure", format!("{prefix}markings must be an object"));
            return;
        };
        for key in markings.keys() {
            if key != "variant" && key != "tag" {
                report.error("structure", format!("unknown marking \"{key}\" (variant, tag)"));
            }
        }
        if let Some(variant) = markings.get("variant") {
            check_colour(variant.get("stroke"), &format!("{prefix}markings.variant.stroke"), report);
            if let Some(style) = variant.get("style") {
                if !style.as_str().is_some_and(|s| STYLES.contains(&s)) {
                    report.error("structure", format!("{prefix}markings.variant.style must be solid, dashed or dotted"));
                }
            }
        }
        if let Some(tag) = markings.get("tag") {
            for field in ["fill", "stroke", "text"] {
                check_colour(tag.get(field), &format!("{prefix}markings.tag.{field}"), report);
            }
        }
    }
}

// --- completing ------------------------------------------------------------

fn merge(base: &Value, over: &Value) -> Value {
    let mut out = base.as_object().cloned().unwrap_or_default();
    for (key, value) in over.as_object().cloned().unwrap_or_default() {
        out.insert(key, value);
    }
    for part in ["concepts", "exchanges", "markings"] {
        let mut merged = base.get(part).and_then(Value::as_object).cloned().unwrap_or_default();
        for (key, value) in over.get(part).and_then(Value::as_object).cloned().unwrap_or_default() {
            let mut entry = merged.get(&key).and_then(Value::as_object).cloned().unwrap_or_default();
            for (field, v) in value.as_object().cloned().unwrap_or_default() {
                entry.insert(field, v);
            }
            merged.insert(key, Value::Object(entry));
        }
        out.insert(part.to_string(), Value::Object(merged));
    }
    Value::Object(out)
}

/// The theme with its `extends` preset under it.
pub fn complete(theme: &Value) -> Value {
    let base = theme.get("extends").and_then(Value::as_str).and_then(preset);
    let mut full = match &base {
        Some(base) => merge(base, theme),
        None => theme.clone(),
    };
    if let (Some(base), Some(object)) = (&base, full.as_object_mut()) {
        if let (Some(base_dark), Some(own_dark)) = (base.get("dark").filter(|d| d.is_object()), theme.get("dark").filter(|d| d.is_object())) {
            object.insert("dark".to_string(), merge(base_dark, own_dark));
        }
    }
    if let Some(object) = full.as_object_mut() {
        object.remove("extends");
    }
    full
}

fn dark_of(full: &Value) -> Option<Value> {
    let dark = full.get("dark").filter(|d| d.is_object())?;
    let mut light = full.clone();
    light.as_object_mut()?.remove("dark");
    let mut out = merge(&light, dark);
    out.as_object_mut()?.remove("dark");
    Some(out)
}

fn check_complete(full: &Value, prefix: &str, report: &mut Report) {
    for key in ["background", "text", "muted"] {
        if full.get(key).and_then(Value::as_str).is_none_or(str::is_empty) {
            report.error("complete", format!("{prefix}{key} is missing"));
        }
    }
    for (key, label, _) in CONCEPTS {
        let entry = full.get("concepts").and_then(|c| c.get(key));
        let has = |field: &str| entry.and_then(|e| e.get(field)).and_then(Value::as_str).is_some_and(|s| !s.is_empty());
        if !has("fill") || !has("stroke") {
            report.error("complete", format!("{prefix}{label} has no fill and stroke (concepts.{key})"));
        }
    }
    for (key, label) in EXCHANGES {
        let line = full.get("exchanges").and_then(|e| e.get(key)).and_then(|e| e.get("line")).and_then(Value::as_str);
        if line.is_none_or(str::is_empty) {
            report.error("complete", format!("{prefix}{label} has no line colour (exchanges.{key})"));
        }
    }
}

// --- the rules ----------------------------------------------------------------

struct Full<'a>(&'a Value);

impl Full<'_> {
    fn colour(&self, key: &str) -> Rgb {
        parse(self.0[key].as_str().unwrap_or("")).unwrap_or([0.0; 3])
    }
    fn concept(&self, key: &str, field: &str) -> String {
        self.0["concepts"][key][field].as_str().unwrap_or("").to_string()
    }
    fn concept_text(&self, key: &str) -> String {
        self.0["concepts"][key]["text"].as_str().map(str::to_string).unwrap_or_else(|| self.0["text"].as_str().unwrap_or("").to_string())
    }
    fn border(&self, key: &str) -> String {
        self.0["concepts"][key]["border"].as_str().unwrap_or("solid").to_string()
    }
    fn line(&self, key: &str) -> String {
        self.0["exchanges"][key]["line"].as_str().unwrap_or("").to_string()
    }
    fn line_text(&self, key: &str) -> String {
        self.0["exchanges"][key]["text"].as_str().map(str::to_string).unwrap_or_else(|| self.line(key))
    }
    fn style(&self, key: &str) -> String {
        self.0["exchanges"][key]["style"].as_str().unwrap_or("solid").to_string()
    }
    fn width(&self, key: &str) -> f64 {
        self.0["exchanges"][key]["width"].as_f64().filter(|w| *w != 0.0).unwrap_or(1.4)
    }
}

fn label_of(key: &str) -> &str {
    CONCEPTS
        .iter()
        .find(|c| c.0 == key)
        .map(|c| c.1)
        .or_else(|| EXCHANGES.iter().find(|e| e.0 == key).map(|e| e.1))
        .unwrap_or(key)
}

fn shape_of(key: &str) -> &str {
    CONCEPTS.iter().find(|c| c.0 == key).map(|c| c.2).unwrap_or("rect")
}

fn same_group(a: &str, b: &str) -> bool {
    SAME_CONCEPT.iter().any(|group| group.contains(&a) && group.contains(&b))
}

/// A measure as the viewer prints it: one decimal, none when whole.
fn round(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if (rounded - rounded.trunc()).abs() < 1e-9 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

fn rgb(text: &str) -> Rgb {
    parse(text).unwrap_or([0.0; 3])
}

fn check_rules(theme: &Value, prefix: &str, report: &mut Report) {
    let full = Full(theme);
    let bg = full.colour("background");
    for group in SAME_CONCEPT {
        let first = group[0];
        for other in &group[1..] {
            for field in ["fill", "stroke"] {
                let (a, b) = (full.concept(first, field), full.concept(other, field));
                if delta_e(rgb(&a), rgb(&b)) >= SAME_COLOUR {
                    report.error(
                        "same-concept",
                        format!("{prefix}{} and {} are one concept on two layers: give them one {field} ({b} vs {a})", label_of(other), label_of(first)),
                    );
                }
            }
        }
    }
    for (key, label, _) in CONCEPTS {
        for field in ["fill", "stroke"] {
            let value = full.concept(key, field);
            for (name, colour, _) in RESERVED {
                let d = delta_e(rgb(&value), rgb(colour));
                if d < RESERVED_LIMIT {
                    report.error("reserved", format!("{prefix}{label} {field} {value} is the {name} the tool keeps for its own meaning (ΔE {}); choose another colour", round(d)));
                }
            }
        }
    }
    for (key, label) in EXCHANGES {
        let value = full.line(key);
        for (name, colour, lines) in RESERVED {
            if !lines {
                continue;
            }
            let d = delta_e(rgb(&value), rgb(colour));
            if d < RESERVED_LIMIT {
                report.error("reserved", format!("{prefix}{label} line {value} is the {name} the tool keeps for its own meaning (ΔE {}); choose another colour", round(d)));
            }
        }
    }
    let text_bg = contrast(full.colour("text"), bg);
    if text_bg < TEXT {
        report.error("contrast", format!("{prefix}text on the background reaches {}:1, under {TEXT}:1 (WCAG AA)", round(text_bg)));
    }
    let muted = full.colour("muted");
    for (key, label, _) in CONCEPTS {
        let fill = rgb(&full.concept(key, "fill"));
        let text = contrast(rgb(&full.concept_text(key)), fill);
        if text < TEXT {
            report.error("contrast", format!("{prefix}{label}: text on its fill reaches {}:1, under {TEXT}:1 (WCAG AA); darken the text or lighten the fill", round(text)));
        }
        let secondary = contrast(muted, fill);
        if secondary < TEXT {
            report.error("contrast", format!("{prefix}{label}: the secondary text on its fill reaches {}:1, under {TEXT}:1 (WCAG AA)", round(secondary)));
        }
        let outline = contrast(rgb(&full.concept(key, "stroke")), bg).max(contrast(fill, bg));
        if outline < GRAPHICS {
            report.error("contrast", format!("{prefix}{label}: neither its outline nor its fill stands out from the background ({}:1, under {GRAPHICS}:1)", round(outline)));
        }
    }
    for (key, label) in EXCHANGES {
        let line = contrast(rgb(&full.line(key)), bg);
        if line < GRAPHICS {
            report.error("contrast", format!("{prefix}{label}: the line on the background reaches {}:1, under {GRAPHICS}:1", round(line)));
        }
        let text = contrast(rgb(&full.line_text(key)), bg);
        if text < TEXT {
            report.error("contrast", format!("{prefix}{label}: its label on the background reaches {}:1, under {TEXT}:1 (WCAG AA); give it a darker text", round(text)));
        }
    }
    for (view, concepts, exchanges) in VIEWS {
        for (a, b) in pairs(concepts) {
            if same_group(a, b) || shape_of(a) != shape_of(b) || full.border(a) != full.border(b) {
                continue;
            }
            distinguish(prefix, view, label_of(a), label_of(b), &full.concept(a, "fill"), &full.concept(b, "fill"), report);
        }
        for (a, b) in pairs(exchanges) {
            if full.style(a) != full.style(b) || (full.width(a) - full.width(b)).abs() >= 1.0 {
                continue;
            }
            distinguish(prefix, view, label_of(a), label_of(b), &full.line(a), &full.line(b), report);
        }
    }
    let keys: Vec<&str> = CONCEPTS.iter().map(|c| c.0).collect();
    for (a, b) in pairs(&keys) {
        if same_group(a, b) || shape_of(a) != shape_of(b) {
            continue;
        }
        if VIEWS.iter().any(|(_, concepts, _)| concepts.contains(&a) && concepts.contains(&b)) {
            continue;
        }
        let d = delta_e(rgb(&full.concept(a, "fill")), rgb(&full.concept(b, "fill")));
        if d < SAME_COLOUR * 2.0 {
            report.warning("one-colour", format!("{prefix}{} and {} share a colour (ΔE {}): one colour should mean one concept", label_of(a), label_of(b), round(d)));
        }
    }
    let fills = distinct_colours(CONCEPTS.iter().map(|c| full.concept(c.0, "fill")).collect());
    if fills > FILLS {
        report.error("palette", format!("{prefix}{fills} different fills: keep to {FILLS} or fewer, a reader learns no more"));
    } else if fills > FILLS_WARNING {
        report.warning("palette", format!("{prefix}{fills} different fills: {FILLS_WARNING} or fewer read more easily"));
    }
    let lines = distinct_colours(EXCHANGES.iter().map(|e| full.line(e.0)).collect());
    if lines > LINES {
        report.error("palette", format!("{prefix}{lines} different line colours: keep to {LINES} or fewer, or tell lines apart by style"));
    }
}

fn distinguish(prefix: &str, view: &str, a: &str, b: &str, colour_a: &str, colour_b: &str, report: &mut Report) {
    let (x, y) = (rgb(colour_a), rgb(colour_b));
    let d = delta_e(x, y);
    if d < DISTINCT {
        report.error(
            "distinct",
            format!("{prefix}{a} and {b} are too close on the {view} view (ΔE {}, under {DISTINCT}); change one of them, or their shape or line style", round(d)),
        );
        return;
    }
    for kind in [Deficiency::Protanopia, Deficiency::Deuteranopia] {
        let dc = delta_e(simulate(x, kind), simulate(y, kind));
        let name = kind.name();
        if dc < CVD_ERROR {
            report.error("cvd", format!("{prefix}{a} and {b} look alike with {name} (ΔE {}) on the {view} view; vary lightness or blue-yellow", round(dc)));
        } else if dc < CVD_WARNING {
            report.warning("cvd", format!("{prefix}{a} and {b} are close with {name} (ΔE {}) on the {view} view", round(dc)));
        }
    }
    let dl = (lightness(x) - lightness(y)).abs();
    if dl < GRAY_ERROR {
        report.error("grayscale", format!("{prefix}{a} and {b} are too close in grayscale (ΔL {}) on the {view} view; vary lightness", round(dl)));
    } else if dl < GRAY_WARNING {
        report.warning("grayscale", format!("{prefix}{a} and {b} are close in grayscale (ΔL {}) on the {view} view", round(dl)));
    }
}

fn pairs<'a>(items: &[&'a str]) -> Vec<(&'a str, &'a str)> {
    let mut out = Vec::new();
    for i in 0..items.len() {
        for j in i + 1..items.len() {
            out.push((items[i], items[j]));
        }
    }
    out
}

fn distinct_colours(values: Vec<String>) -> usize {
    let mut kept: Vec<Rgb> = Vec::new();
    for value in values {
        let colour = rgb(&value);
        if !kept.iter().any(|other| delta_e(colour, *other) < SAME_COLOUR) {
            kept.push(colour);
        }
    }
    kept.len()
}

// --- colour science (viewer/arcviz-color.js) -------------------------------------

type Rgb = [f64; 3];

/// `#RRGGBB` or `#RGB` to 0..255 channels.
fn parse(text: &str) -> Option<Rgb> {
    let hex = text.trim().strip_prefix('#')?;
    let full: String = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).collect(),
        6 => hex.to_string(),
        _ => return None,
    };
    if !full.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(&full, 16).ok()?;
    Some([((n >> 16) & 255) as f64, ((n >> 8) & 255) as f64, (n & 255) as f64])
}

fn to_linear(c: f64) -> f64 {
    let v = c / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn from_linear(v: f64) -> f64 {
    255.0 * if v <= 0.0031308 { 12.92 * v } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
}

fn luminance(c: Rgb) -> f64 {
    0.2126 * to_linear(c[0]) + 0.7152 * to_linear(c[1]) + 0.0722 * to_linear(c[2])
}

fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

fn lab(c: Rgb) -> [f64; 3] {
    let [r, g, b] = c.map(to_linear);
    let x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047;
    let y = 0.2126729 * r + 0.7151522 * g + 0.072175 * b;
    let z = (0.0193339 * r + 0.119192 * g + 0.9503041 * b) / 1.08883;
    let f = |t: f64| if t > 216.0 / 24389.0 { t.cbrt() } else { (24389.0 / 27.0 * t + 16.0) / 116.0 };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

fn lightness(c: Rgb) -> f64 {
    lab(c)[0]
}

fn delta_e(a: Rgb, b: Rgb) -> f64 {
    delta_e_lab(lab(a), lab(b))
}

/// CIEDE2000 (Sharma, Wu, Dalal 2005).
fn delta_e_lab([l1, a1, b1]: [f64; 3], [l2, a2, b2]: [f64; 3]) -> f64 {
    let rad = std::f64::consts::PI / 180.0;
    let c1 = a1.hypot(b1);
    let c2 = a2.hypot(b2);
    let cm = (c1 + c2) / 2.0;
    let g = 0.5 * (1.0 - (cm.powi(7) / (cm.powi(7) + 25f64.powi(7))).sqrt());
    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;
    let c1p = a1p.hypot(b1);
    let c2p = a2p.hypot(b2);
    let hue = |x: f64, y: f64| {
        if x == 0.0 && y == 0.0 {
            return 0.0;
        }
        let h = y.atan2(x) / rad;
        if h < 0.0 {
            h + 360.0
        } else {
            h
        }
    };
    let h1p = hue(a1p, b1);
    let h2p = hue(a2p, b2);
    let dlp = l2 - l1;
    let dcp = c2p - c1p;
    let mut dhp = 0.0;
    if c1p * c2p != 0.0 {
        dhp = h2p - h1p;
        if dhp > 180.0 {
            dhp -= 360.0;
        } else if dhp < -180.0 {
            dhp += 360.0;
        }
    }
    let dhp_big = 2.0 * (c1p * c2p).sqrt() * (dhp / 2.0 * rad).sin();
    let lpm = (l1 + l2) / 2.0;
    let cpm = (c1p + c2p) / 2.0;
    let mut hpm = h1p + h2p;
    if c1p * c2p != 0.0 {
        hpm = if (h1p - h2p).abs() <= 180.0 {
            (h1p + h2p) / 2.0
        } else if h1p + h2p < 360.0 {
            (h1p + h2p + 360.0) / 2.0
        } else {
            (h1p + h2p - 360.0) / 2.0
        };
    }
    let t = 1.0 - 0.17 * ((hpm - 30.0) * rad).cos() + 0.24 * (2.0 * hpm * rad).cos() + 0.32 * ((3.0 * hpm + 6.0) * rad).cos()
        - 0.2 * ((4.0 * hpm - 63.0) * rad).cos();
    let d_theta = 30.0 * (-((hpm - 275.0) / 25.0).powi(2)).exp();
    let rc = 2.0 * (cpm.powi(7) / (cpm.powi(7) + 25f64.powi(7))).sqrt();
    let sl = 1.0 + (0.015 * (lpm - 50.0).powi(2)) / (20.0 + (lpm - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * cpm;
    let sh = 1.0 + 0.015 * cpm * t;
    let rt = -(2.0 * d_theta * rad).sin() * rc;
    ((dlp / sl).powi(2) + (dcp / sc).powi(2) + (dhp_big / sh).powi(2) + rt * (dcp / sc) * (dhp_big / sh)).sqrt()
}

#[derive(Clone, Copy)]
enum Deficiency {
    Protanopia,
    Deuteranopia,
}

impl Deficiency {
    fn name(self) -> &'static str {
        match self {
            Deficiency::Protanopia => "protanopia",
            Deficiency::Deuteranopia => "deuteranopia",
        }
    }
    /// Machado, Oliveira, Fernandes (2009), severity 1, on linear RGB.
    fn matrix(self) -> [f64; 9] {
        match self {
            Deficiency::Protanopia => [0.152286, 1.052583, -0.204868, 0.114503, 0.786281, 0.099216, -0.003882, -0.048116, 1.051998],
            Deficiency::Deuteranopia => [0.367322, 0.860646, -0.227968, 0.280085, 0.672501, 0.047413, -0.01182, 0.04294, 0.968881],
        }
    }
}

fn simulate(c: Rgb, kind: Deficiency) -> Rgb {
    let m = kind.matrix();
    let [r, g, b] = c.map(to_linear);
    let clamp = |v: f64| v.clamp(0.0, 1.0);
    [
        from_linear(clamp(m[0] * r + m[1] * g + m[2] * b)),
        from_linear(clamp(m[3] * r + m[4] * g + m[5] * b)),
        from_linear(clamp(m[6] * r + m[7] * g + m[8] * b)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> Vec<(String, Value)> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/themes");
        let mut out: Vec<(String, Value)> = std::fs::read_dir(&dir)
            .expect("tests/fixtures/themes")
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .map(|e| (e.file_name().to_string_lossy().into_owned(), serde_json::from_str(&std::fs::read_to_string(e.path()).unwrap()).unwrap()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    #[test]
    fn ciede2000_matches_the_reference_pairs() {
        let pairs = [
            ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
            ([50.0, -1.3802, -84.2814], [50.0, 0.0, -82.7485], 1.0),
            ([60.2574, -34.0099, 36.2677], [60.4626, -34.1751, 39.4387], 1.2644),
            ([2.0776, 0.0795, -1.135], [0.9033, -0.0636, -0.5514], 0.9082),
        ];
        for (a, b, expected) in pairs {
            assert!((delta_e_lab(a, b) - expected).abs() < 1e-4);
        }
        assert_eq!(round(contrast(parse("#777777").unwrap(), parse("#FFFFFF").unwrap())), "4.5");
    }

    #[test]
    fn every_preset_passes() {
        for (id, _) in PRESETS {
            let verdict = validate(&preset(id).unwrap());
            assert!(verdict.ok, "{id}: {:?}", verdict.errors);
        }
    }

    #[test]
    fn the_fixtures_get_the_verdicts_the_viewer_gives() {
        let all = fixtures();
        assert!(all.len() >= 10);
        for (name, fixture) in all {
            let verdict = validate(&fixture["theme"]);
            let expect = &fixture["expect"];
            assert_eq!(verdict.ok, expect["ok"].as_bool().unwrap(), "{name}: {:?}", verdict.errors);
            for rule in expect["rules"].as_array().unwrap() {
                assert!(verdict.errors.iter().any(|f| f.rule == rule.as_str().unwrap()), "{name}: rule {rule} in {:?}", verdict.errors);
            }
            for text in expect["errors"].as_array().unwrap() {
                let text = text.as_str().unwrap();
                assert!(verdict.errors.iter().any(|f| f.message.contains(text)), "{name}: \"{text}\" in {:#?}", verdict.errors);
            }
        }
    }

    #[test]
    fn a_preset_name_resolves_and_a_refused_file_says_why() {
        assert_eq!(resolve("capella-classic").unwrap()["name"], "Capella classic");
        let dir = std::env::temp_dir().join(format!("arclang-theme-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("bad.json");
        std::fs::write(&file, r##"{"version":1,"name":"Red","extends":"capella-classic","concepts":{"system_actor":{"stroke":"#E51400"}}}"##).unwrap();
        let error = resolve(file.to_str().unwrap()).unwrap_err();
        assert!(error.contains("is refused"), "{error}");
        assert!(error.contains("actor stroke #E51400 is the error red"), "{error}");
        let good = dir.join("good.json");
        std::fs::write(&good, r#"{"version":1,"name":"Ours","extends":"print-grayscale"}"#).unwrap();
        let theme = resolve(good.to_str().unwrap()).unwrap();
        assert_eq!(theme["concepts"]["system_actor"]["fill"], "#808080");
        assert!(theme.get("extends").is_none());
        assert!(resolve("no-such-theme").unwrap_err().contains("name a preset"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
