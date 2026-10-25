//! Spatial modelling — where the elements of a system stand, how big they
//! are, and what a physical link covers.
//!
//! The values are ordinary ArcLang attribute blocks with units, so they
//! parse, format, edit and diff like any other attribute:
//!
//! ```text
//! node "Charger A" {
//!     location: { lat: 48.8566 deg lon: 2.3522 deg alt: 35 m }   // WGS 84
//!     position: { x: 1.2 m y: 0 m z: 0.4 m }                     // in its owner
//!     orientation: { yaw: 90 deg }                                 // about z, then y, then x
//!     shape: { box: [2 m, 1 m, 0.5 m] }                           // or { cylinder: [diameter, height] }
//! }
//! physical_link "Fibre" { route: [{ lat: .. lon: .. }, { lat: .. lon: .. }] length: 12 km }
//! ```
//!
//! This module reads those values (refusing anything it cannot read with
//! the reason) and measures distances on the Earth. [`checks`] holds what
//! the compiler asserts about a model's geometry, [`geojson`] and [`step`]
//! the exports to maps and to CAD.

pub mod checks;
pub mod geojson;
pub mod step;

use super::ast::AttributeValue;
use super::quantity::{Dimension, Quantity};
use std::collections::HashMap;

/// Mean Earth radius (IUGG), metres. Distances are computed on the sphere
/// of this radius: within 0.5 % of the WGS 84 ellipsoid everywhere.
pub const EARTH_RADIUS: f64 = 6_371_008.8;
/// How far the spherical distance may be from the ellipsoidal one: checks
/// that compare a declared length with a distance allow this much.
pub const SPHERE_TOLERANCE: f64 = 0.005;
/// Speed of light in vacuum, m/s: no link carries a signal faster.
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// A point on the Earth, WGS 84.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
    /// Metres above the ellipsoid, when written.
    pub alt: Option<f64>,
}

/// An offset in an owner's frame, metres.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Offset {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A rotation, radians: yaw about z, then pitch about y, then roll about x.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rotation {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
}

/// The envelope of an element, metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// Along x, y and z, from the element's origin.
    Box { length: f64, width: f64, height: f64 },
    /// Axis along z, base on the element's origin.
    Cylinder { diameter: f64, height: f64 },
}

impl Shape {
    pub fn volume(&self) -> f64 {
        match *self {
            Shape::Box { length, width, height } => length * width * height,
            Shape::Cylinder { diameter, height } => std::f64::consts::PI * diameter * diameter / 4.0 * height,
        }
    }
}

fn entries(value: &AttributeValue, what: &str, example: &str) -> Result<HashMap<String, AttributeValue>, String> {
    match value {
        AttributeValue::Map(map) => Ok(map.clone()),
        other => Err(format!("expected {what} block such as `{example}`, got '{}'", other.display())),
    }
}

fn refuse_unknown(map: &HashMap<String, AttributeValue>, known: &[&str], what: &str) -> Result<(), String> {
    let mut unknown: Vec<&String> = map.keys().filter(|k| !known.contains(&k.as_str())).collect();
    unknown.sort();
    match unknown.first() {
        Some(key) => Err(format!("{what} has no '{key}' (it takes {})", known.join(", "))),
        None => Ok(()),
    }
}

fn with_article(label: &str) -> String {
    let article = if label.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
    format!("{article} {label}")
}

/// A value of the given dimension, in its canonical unit (metres, radians).
fn measure(value: &AttributeValue, key: &str, dimension: Dimension) -> Result<f64, String> {
    let quantity = match value.as_quantity() {
        Some(Ok(quantity)) => quantity,
        Some(Err(_)) | None => {
            let example = if dimension == Dimension::Angle { "deg" } else { "m" };
            return Err(format!("'{key}' must be {} with its unit, such as `{key}: 1 {example}` (got '{}')", with_article(dimension.label()), value.display()));
        }
    };
    if quantity.dimension() != dimension {
        return Err(format!("'{key}' is {} ('{quantity}'), expected {}", with_article(quantity.dimension().label()), with_article(dimension.label())));
    }
    let canonical = quantity.canonical();
    if !canonical.is_finite() {
        return Err(format!("'{key}' is not a finite value"));
    }
    Ok(canonical)
}

fn optional(map: &HashMap<String, AttributeValue>, key: &str, dimension: Dimension) -> Result<Option<f64>, String> {
    map.get(key).map(|v| measure(v, key, dimension)).transpose()
}

/// `{ lat: 48.8566 deg lon: 2.3522 deg alt: 35 m }`.
pub fn location(value: &AttributeValue) -> Result<GeoPoint, String> {
    let map = entries(value, "a location", "{ lat: 48.8566 deg lon: 2.3522 deg }")?;
    refuse_unknown(&map, &["lat", "lon", "alt"], "a location")?;
    let degrees = |key: &str| -> Result<f64, String> {
        let value = map.get(key).ok_or_else(|| format!("a location needs '{key}'"))?;
        // A billionth of a degree is a tenth of a millimetre: no more is meant.
        Ok((measure(value, key, Dimension::Angle)?.to_degrees() * 1e9).round() / 1e9)
    };
    let (lat, lon) = (degrees("lat")?, degrees("lon")?);
    if !(-90.0..=90.0).contains(&lat) {
        return Err(format!("latitude {lat}° is beyond the poles (-90° to 90°)"));
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err(format!("longitude {lon}° is beyond -180° to 180°"));
    }
    Ok(GeoPoint { lat, lon, alt: optional(&map, "alt", Dimension::Length)? })
}

/// `{ x: 1.2 m y: 0 m z: 0.4 m }`; a missing axis is 0.
pub fn position(value: &AttributeValue) -> Result<Offset, String> {
    let map = entries(value, "a position", "{ x: 1 m y: 0 m z: 0 m }")?;
    refuse_unknown(&map, &["x", "y", "z"], "a position")?;
    let axis = |key: &str| optional(&map, key, Dimension::Length).map(|v| v.unwrap_or(0.0));
    Ok(Offset { x: axis("x")?, y: axis("y")?, z: axis("z")? })
}

/// `{ yaw: 90 deg pitch: 0 deg roll: 0 deg }`; a missing angle is 0.
pub fn orientation(value: &AttributeValue) -> Result<Rotation, String> {
    let map = entries(value, "an orientation", "{ yaw: 90 deg }")?;
    refuse_unknown(&map, &["yaw", "pitch", "roll"], "an orientation")?;
    let angle = |key: &str| optional(&map, key, Dimension::Angle).map(|v| v.unwrap_or(0.0));
    Ok(Rotation { yaw: angle("yaw")?, pitch: angle("pitch")?, roll: angle("roll")? })
}

/// `{ box: [length, width, height] }` or `{ cylinder: [diameter, height] }`.
pub fn shape(value: &AttributeValue) -> Result<Shape, String> {
    let map = entries(value, "a shape", "{ box: [2 m, 1 m, 0.5 m] }")?;
    refuse_unknown(&map, &["box", "cylinder"], "a shape")?;
    if map.len() != 1 {
        return Err("a shape is one of `box` or `cylinder`".to_string());
    }
    let (kind, sizes) = map.iter().next().expect("one entry");
    let (names, example): (&[&str], &str) = match kind.as_str() {
        "box" => (&["length", "width", "height"], "[2 m, 1 m, 0.5 m]"),
        _ => (&["diameter", "height"], "[0.3 m, 1.2 m]"),
    };
    let items = match sizes {
        AttributeValue::List(items) if items.len() == names.len() => items,
        _ => return Err(format!("a {kind} takes {} ({}), such as `{kind}: {example}`", names.len(), names.join(", "))),
    };
    let mut values = Vec::new();
    for (name, item) in names.iter().zip(items) {
        let size = measure(item, name, Dimension::Length)?;
        if size <= 0.0 {
            return Err(format!("the {name} of a {kind} must be more than 0"));
        }
        values.push(size);
    }
    Ok(match kind.as_str() {
        "box" => Shape::Box { length: values[0], width: values[1], height: values[2] },
        _ => Shape::Cylinder { diameter: values[0], height: values[1] },
    })
}

/// `[{ lat: .. lon: .. }, { lat: .. lon: .. }, ...]`: at least two points.
pub fn route(value: &AttributeValue) -> Result<Vec<GeoPoint>, String> {
    let AttributeValue::List(items) = value else {
        return Err(format!("expected a route `[{{ lat: .. lon: .. }}, {{ lat: .. lon: .. }}]`, got '{}'", value.display()));
    };
    if items.len() < 2 {
        return Err("a route has at least two points".to_string());
    }
    items.iter().enumerate().map(|(i, item)| location(item).map_err(|reason| format!("point {} of the route: {reason}", i + 1))).collect()
}

/// Great-circle distance on the mean-radius sphere, metres (haversine).
pub fn distance(a: GeoPoint, b: GeoPoint) -> f64 {
    let (phi1, phi2) = (a.lat.to_radians(), b.lat.to_radians());
    let dphi = phi2 - phi1;
    let dlambda = (b.lon - a.lon).to_radians();
    let h = (dphi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (dlambda / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS * h.sqrt().min(1.0).asin()
}

/// Length of a route, metres: the sum of its legs.
pub fn route_length(points: &[GeoPoint]) -> f64 {
    points.windows(2).map(|leg| distance(leg[0], leg[1])).sum()
}

/// A length written in metres, as a model would write it back.
pub fn metres(value: f64) -> Quantity {
    if value >= 1000.0 {
        Quantity { value: (value / 10.0).round() / 100.0, unit: "km".to_string() }
    } else {
        Quantity { value: (value * 100.0).round() / 100.0, unit: "m".to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> AttributeValue {
        let model = format!("model T {{ name: \"t\" }}\nphysical_architecture \"P\" {{\n  node \"A\" {{ id: \"PN-A\" v: {text} }}\n}}\n");
        let result = crate::compiler::Compiler::new(crate::compiler::CompilerConfig::default()).compile_string(&model).expect("compiles");
        result.ast.physical_architecture[0].nodes[0].attributes["v"].clone()
    }

    #[test]
    fn a_location_reads_degrees_and_an_optional_altitude() {
        let paris = location(&parse("{ lat: 48.8566 deg lon: 2.3522 deg alt: 35 m }")).unwrap();
        assert!((paris.lat - 48.8566).abs() < 1e-9 && (paris.lon - 2.3522).abs() < 1e-9 && paris.alt == Some(35.0));
        let south = location(&parse("{ lat: -33.45 deg lon: -70.66 deg }")).unwrap();
        assert!(south.lat < 0.0 && south.lon < 0.0 && south.alt.is_none());
        assert!(location(&parse("{ lat: 0.8527 rad lon: 0.0410 rad }")).unwrap().lat > 48.0);
    }

    #[test]
    fn a_location_that_cannot_be_on_the_earth_is_refused_with_the_reason() {
        assert!(location(&parse("{ lat: 95 deg lon: 0 deg }")).unwrap_err().contains("beyond the poles"));
        assert!(location(&parse("{ lat: 10 deg lon: 200 deg }")).unwrap_err().contains("longitude"));
        assert!(location(&parse("{ lat: 10 lon: 2 deg }")).unwrap_err().contains("with its unit"));
        assert!(location(&parse("{ lat: 10 m lon: 2 deg }")).unwrap_err().contains("is a length ('10 m'), expected an angle"));
        assert!(location(&parse("{ lon: 2 deg }")).unwrap_err().contains("needs 'lat'"));
        assert!(location(&parse("{ lat: 1 deg lon: 2 deg height: 3 m }")).unwrap_err().contains("no 'height'"));
        assert!(location(&parse("\"Paris\"")).unwrap_err().contains("expected a location block"));
    }

    #[test]
    fn positions_orientations_and_shapes_read_their_units() {
        assert_eq!(position(&parse("{ x: 120 cm z: 2 m }")).unwrap(), Offset { x: 1.2, y: 0.0, z: 2.0 });
        assert!((orientation(&parse("{ yaw: 90 deg }")).unwrap().yaw - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
        assert_eq!(shape(&parse("{ box: [2 m, 1 m, 50 cm] }")).unwrap(), Shape::Box { length: 2.0, width: 1.0, height: 0.5 });
        assert_eq!(shape(&parse("{ cylinder: [300 mm, 1.2 m] }")).unwrap(), Shape::Cylinder { diameter: 0.3, height: 1.2 });
        assert!(shape(&parse("{ box: [2 m, 1 m] }")).unwrap_err().contains("takes 3"));
        assert!(shape(&parse("{ box: [2 m, 0 m, 1 m] }")).unwrap_err().contains("more than 0"));
        assert!(shape(&parse("{ sphere: [1 m] }")).unwrap_err().contains("no 'sphere'"));
        assert!(position(&parse("{ x: 3 deg }")).unwrap_err().contains("is an angle ('3 deg'), expected a length"));
    }

    #[test]
    fn distances_are_great_circle_metres() {
        let paris = GeoPoint { lat: 48.8566, lon: 2.3522, alt: None };
        let london = GeoPoint { lat: 51.5074, lon: -0.1278, alt: None };
        // About 343.5 km on the ellipsoid.
        let d = distance(paris, london);
        assert!((d - 343_500.0).abs() < 343_500.0 * SPHERE_TOLERANCE, "{d}");
        assert_eq!(distance(paris, paris), 0.0);
        let r = route_length(&[paris, london, paris]);
        assert!((r - 2.0 * d).abs() < 1e-6);
    }
}
