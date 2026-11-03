//! The width of a label as the connector's diagrams are drawn on the platform
//! its reference scripts come from (Eclipse on Linux GTK, where the diagrams'
//! Segoe UI falls back to DejaVu Sans): DejaVu Sans 8 pt at 96 dpi, the advances
//! summed unhinted and the extent rounded up to the pixel, as Pango measures.
//!
//! The advances are those of DejaVu Sans 2.37 (Bitstream Vera license and
//! public domain), in font units (2048 to the em), for U+0020 to U+007E then
//! U+00A0 to U+00FF.

const UNITS_PER_EM: f64 = 2048.0;
/// 8 pt at 96 dpi.
const PIXELS_PER_EM: f64 = 8.0 * 96.0 / 72.0;
/// The advance of a character the table does not have: that of `x`.
const OTHER: u32 = 1212;

const ADVANCES: [u32; 191] = [
    651, 821, 942, 1716, 1303, 1946, 1597, 563, 799, 799, 1024, 1716, 651, 739, 651, 690,
    1303, 1303, 1303, 1303, 1303, 1303, 1303, 1303, 1303, 1303, 690, 690, 1716, 1716, 1716, 1087,
    2048, 1401, 1405, 1430, 1577, 1294, 1178, 1587, 1540, 604, 604, 1343, 1141, 1767, 1532, 1612,
    1235, 1612, 1423, 1300, 1251, 1499, 1401, 2025, 1403, 1251, 1403, 799, 690, 799, 1716, 1024,
    1024, 1255, 1300, 1126, 1300, 1260, 721, 1300, 1298, 569, 569, 1186, 569, 1995, 1298, 1253,
    1300, 1300, 842, 1067, 803, 1298, 1212, 1675, 1212, 1212, 1075, 1303, 690, 1303, 1716, 651,
    821, 1303, 1303, 1303, 1303, 690, 1024, 1024, 2048, 965, 1253, 1716, 739, 2048, 1024, 1024,
    1716, 821, 821, 1024, 1303, 1303, 651, 1024, 821, 965, 1253, 1985, 1985, 1985, 1087, 1401,
    1401, 1401, 1401, 1401, 1401, 1995, 1430, 1294, 1294, 1294, 1294, 604, 604, 604, 604, 1587,
    1532, 1612, 1612, 1612, 1612, 1612, 1716, 1612, 1499, 1499, 1499, 1499, 1251, 1239, 1290, 1255,
    1255, 1255, 1255, 1255, 1255, 2011, 1126, 1260, 1260, 1260, 1260, 569, 569, 569, 569, 1253,
    1298, 1253, 1253, 1253, 1253, 1253, 1716, 1253, 1298, 1298, 1298, 1298, 1212, 1300, 1212,
];

fn advance(c: char) -> u32 {
    let code = c as u32;
    let found = match code {
        0x20..=0x7e => Some(ADVANCES[(code - 0x20) as usize]),
        0xa0..=0xff => Some(ADVANCES[(code - 0xa0 + 95) as usize]),
        _ => None,
    };
    found.filter(|&a| a > 0).unwrap_or(OTHER)
}

/// The width of the text in pixels.
pub fn text_width(text: &str) -> i32 {
    let units: u32 = text.chars().map(advance).sum();
    (units as f64 * PIXELS_PER_EM / UNITS_PER_EM - 1e-9).ceil() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_measure_as_pango_measures_them() {
        assert_eq!(text_width("Mode 1"), 40);
        assert_eq!(text_width("Mode 1 copy"), 69);
        assert_eq!(text_width("State 1"), 39);
        assert_eq!(text_width(""), 0);
    }
}
