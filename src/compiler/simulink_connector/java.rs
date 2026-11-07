//! What the connector's output owes to Java: the iteration order of a
//! `HashMap` or `HashSet` of strings (which the generated MATLAB follows),
//! the normalization of names, and how a double is printed. Each is
//! reproduced exactly, so the scripts are those of the connector to the
//! character.

use unicode_normalization::UnicodeNormalization;

/// `String.hashCode()`: over the UTF-16 code units, wrapping.
pub fn string_hash(text: &str) -> i32 {
    text.encode_utf16().fold(0i32, |h, unit| h.wrapping_mul(31).wrapping_add(unit as i32))
}

/// `HashMap.hash(key)`: the high bits spread into the low ones.
fn spread(hash: i32) -> u32 {
    let h = hash as u32;
    h ^ (h >> 16)
}

/// The order a `HashMap` (or `HashSet`) of default capacity iterates over these keys, put in this
/// order: the indexes of the first occurrence of each distinct key. The table doubles past three
/// quarters full; a bucket keeps its keys in the order they came, a resize splits it in order.
pub fn hash_order<S: AsRef<str>>(keys: &[S]) -> Vec<usize> {
    let mut distinct: Vec<usize> = Vec::new();
    for (i, key) in keys.iter().enumerate() {
        if !distinct.iter().any(|&j| keys[j].as_ref() == key.as_ref()) {
            distinct.push(i);
        }
    }
    let mut capacity: u32 = 16;
    while distinct.len() as f64 > capacity as f64 * 0.75 {
        capacity *= 2;
    }
    let mut order = distinct;
    order.sort_by_key(|&i| spread(string_hash(keys[i].as_ref())) & (capacity - 1));
    order
}

/// The distinct keys in the order a Java `HashSet` iterates over them.
pub fn hash_set<S: AsRef<str> + Clone>(keys: &[S]) -> Vec<S> {
    hash_order(keys).into_iter().map(|i| keys[i].clone()).collect()
}

/// The connector's `NameHelper.normalize`: trimmed, accents removed, every character that is not
/// an ASCII letter, digit or space made `_`, then every space `_`.
pub fn normalize(name: Option<&str>) -> String {
    let Some(name) = name else { return String::new() };
    let trimmed = name.trim_matches(|c: char| c <= ' ');
    trimmed
        .nfd()
        // `\p{InCombiningDiacriticalMarks}`: the block U+0300 to U+036F only.
        .filter(|c| !('\u{300}'..='\u{36f}').contains(c))
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// `Double.toString(value)` (JDK 19 and later: the shortest decimal that reads back the same).
pub fn double(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if value == 0.0 {
        return if value.is_sign_negative() { "-0.0" } else { "0.0" }.to_string();
    }
    let magnitude = value.abs();
    if (1e-3..1e7).contains(&magnitude) {
        let text = format!("{value}");
        return if text.contains('.') { text } else { format!("{text}.0") };
    }
    // Scientific: d.dddE±n, the mantissa with at least one decimal.
    let text = format!("{value:e}");
    let (mantissa, exponent) = text.split_once('e').unwrap_or((&text, "0"));
    let mantissa = if mantissa.contains('.') { mantissa.to_string() } else { format!("{mantissa}.0") };
    format!("{mantissa}E{exponent}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_hash_as_in_java() {
        assert_eq!(string_hash(""), 0);
        assert_eq!(string_hash("a"), 97);
        assert_eq!(string_hash("hello"), 99162322);
        assert_eq!(string_hash("busEI_ExchangeItem_1"), "busEI_ExchangeItem_1".encode_utf16().fold(0i32, |h, u| h.wrapping_mul(31).wrapping_add(u as i32)));
    }

    #[test]
    fn a_hash_set_iterates_by_bucket_then_by_insertion() {
        // "Aa" and "BB" share a hash (2112): same bucket, kept in the order they came.
        assert_eq!(hash_set(&["BB", "Aa", "c"]), vec!["BB", "Aa", "c"]);
        assert_eq!(hash_set(&["c", "a", "b", "a"]), vec!["a", "b", "c"]);
    }

    #[test]
    fn names_are_normalized_as_the_connector_does() {
        assert_eq!(normalize(Some("  Mode 1 copy ")), "Mode_1_copy");
        assert_eq!(normalize(Some("Température (°C)")), "Temperature___C_");
        assert_eq!(normalize(None), "");
    }

    #[test]
    fn doubles_print_as_in_java() {
        assert_eq!(double(2.1852015073194484), "2.1852015073194484");
        assert_eq!(double(3.0), "3.0");
        assert_eq!(double(1e7), "1.0E7");
        assert_eq!(double(0.0001), "1.0E-4");
        assert_eq!(double(-12.5), "-12.5");
    }
}
