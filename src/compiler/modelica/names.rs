//! Modelica identifiers from ArcLang names: classes in UpperCamelCase,
//! instances in lowerCamelCase, never a keyword, never starting with a digit.

/// Modelica 3.6 keywords: an identifier spelling one gets a trailing `_`.
const KEYWORDS: &[&str] = &[
    "algorithm", "and", "annotation", "block", "break", "class", "connect", "connector", "constant", "constrainedby", "der", "discrete",
    "each", "else", "elseif", "elsewhen", "encapsulated", "end", "enumeration", "equation", "expandable", "extends", "external", "false",
    "final", "flow", "for", "function", "if", "import", "impure", "in", "initial", "inner", "input", "loop", "model", "not", "operator",
    "or", "outer", "output", "package", "parameter", "partial", "protected", "public", "pure", "record", "redeclare", "replaceable",
    "return", "stream", "then", "true", "type", "when", "while", "within", "time",
];

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect()
}

fn guard(identifier: String, fallback: &str) -> String {
    let identifier = if identifier.is_empty() { fallback.to_string() } else { identifier };
    let identifier = if identifier.starts_with(|c: char| c.is_ascii_digit()) { format!("_{identifier}") } else { identifier };
    if KEYWORDS.contains(&identifier.as_str()) {
        format!("{identifier}_")
    } else {
        identifier
    }
}

fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// `"Battery pack"` -> `BatteryPack`, `"EV range simulation"` -> `EVRangeSimulation`.
pub fn class_name(text: &str) -> String {
    guard(words(text).iter().map(|w| capitalized(w)).collect(), "Element")
}

/// `"Battery pack"` -> `batteryPack`, `"EV hardware"` -> `evHardware`.
pub fn instance_name(text: &str) -> String {
    let words = words(text);
    let mut out = String::new();
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            // A first word in capitals (an acronym) is lowered whole.
            if word.chars().all(|c| !c.is_ascii_lowercase()) {
                out.push_str(&word.to_ascii_lowercase());
            } else {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    out.push(first.to_ascii_lowercase());
                    out.push_str(chars.as_str());
                }
            }
        } else {
            out.push_str(&capitalized(word));
        }
    }
    guard(out, "element")
}

/// A port or attribute name kept as written where Modelica allows it:
/// `hv_p` stays `hv_p`, `torque cmd` becomes `torque_cmd`.
pub fn member_name(text: &str) -> String {
    guard(words(text).join("_"), "member")
}

/// A name not yet in `taken`, suffixed `_2`, `_3`... when needed.
pub fn unique(name: String, taken: &mut Vec<String>) -> String {
    let mut candidate = name.clone();
    let mut n = 2;
    while taken.contains(&candidate) {
        candidate = format!("{name}_{n}");
        n += 1;
    }
    taken.push(candidate.clone());
    candidate
}

/// A Modelica string literal's content: `"` and `\` escaped.
pub fn string(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Text for `Documentation(info=...)`: HTML-escaped, then a string.
pub fn html(text: &str) -> String {
    string(&text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"))
}

/// A real literal: plain for ordinary magnitudes, scientific otherwise
/// (`2.952e8`), always the shortest text that reads back the same value.
pub fn real(value: f64) -> String {
    // Fifteen significant digits: what a decimal written in the model holds,
    // without the noise of its conversion (70 % is 0.7, not 0.7000000000000001).
    let value: f64 = format!("{value:.14e}").parse().unwrap_or(value);
    let magnitude = value.abs();
    if magnitude != 0.0 && !(1e-3..1e6).contains(&magnitude) {
        format!("{value:e}")
    } else {
        format!("{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reals_carry_no_conversion_noise() {
        assert_eq!(real(70.0 * 0.01), "0.7");
        assert_eq!(real(0.1 + 0.2), "0.3");
        assert_eq!(real(100.0 * 1000.0 / 3600.0), "27.7777777777778");
        assert_eq!(real(2.7e8), "2.7e8");
        assert_eq!(real(0.025), "0.025");
    }

    #[test]
    fn classes_instances_and_members() {
        assert_eq!(class_name("Battery pack"), "BatteryPack");
        assert_eq!(class_name("EV range simulation"), "EVRangeSimulation");
        assert_eq!(class_name("22 kW onboard charger"), "_22KWOnboardCharger");
        assert_eq!(instance_name("Battery pack"), "batteryPack");
        assert_eq!(instance_name("EV hardware"), "evHardware");
        assert_eq!(instance_name("Vehicle control unit"), "vehicleControlUnit");
        assert_eq!(member_name("torque cmd"), "torque_cmd");
        assert_eq!(member_name("in"), "in_");
        assert_eq!(member_name("flow"), "flow_");
    }

    #[test]
    fn reals_are_short_and_exact() {
        assert_eq!(real(2.952e8), "2.952e8");
        assert_eq!(real(400.0), "400");
        assert_eq!(real(0.9), "0.9");
        assert_eq!(real(2500.0), "2500");
        assert_eq!(real(1e-4), "1e-4");
        assert_eq!("2.952e8".parse::<f64>().unwrap(), 2.952e8);
    }

    #[test]
    fn unique_names_are_suffixed() {
        let mut taken = Vec::new();
        assert_eq!(unique("a".into(), &mut taken), "a");
        assert_eq!(unique("a".into(), &mut taken), "a_2");
    }
}
