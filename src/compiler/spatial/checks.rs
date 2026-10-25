//! What the compiler asserts about the geometry of a model.
//!
//! Only what holds whatever the medium: a link is never shorter than the
//! way it takes nor than the distance between located ends, never longer
//! than the most it may be, and no signal crosses it faster than light.
//! Distances are on the mean-radius sphere, so every comparison allows
//! [`SPHERE_TOLERANCE`] in the model's favour: a finding is never a
//! rounding artefact.

use super::{distance, location, route, route_length, GeoPoint, SPEED_OF_LIGHT, SPHERE_TOLERANCE};
use crate::compiler::ast::{AttributeValue, Model, PhysicalLink};
use crate::compiler::quantity::Dimension;
use std::collections::HashMap;

/// One thing the geometry of a model contradicts.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub kind: &'static str,
    pub element: String,
    pub attribute: &'static str,
    pub message: String,
}

/// Every geometric contradiction of the model, in a stable order.
pub fn check(ast: &Model) -> Vec<Finding> {
    let mut out = Vec::new();
    for architecture in &ast.physical_architecture {
        let located: HashMap<&str, GeoPoint> = architecture
            .nodes
            .iter()
            .filter_map(|node| node.attributes.get("location").and_then(|v| location(v).ok()).map(|at| (node, at)))
            .flat_map(|(node, at)| [(node.id.as_str(), at), (node.name.as_str(), at)])
            .collect();
        for link in &architecture.links {
            check_link(link, &located, &mut out);
        }
    }
    out
}

fn length_of(value: Option<&AttributeValue>) -> Option<f64> {
    match value?.as_quantity()? {
        Ok(q) if q.dimension() == Dimension::Length => Some(q.canonical()),
        _ => None,
    }
}

fn seconds(value: Option<&AttributeValue>) -> Option<f64> {
    match value?.as_quantity()? {
        Ok(q) if q.dimension() == Dimension::Time => Some(q.canonical()),
        _ => None,
    }
}

fn km(metres: f64) -> String {
    if metres >= 1000.0 { format!("{:.2} km", metres / 1000.0) } else { format!("{metres:.1} m") }
}

fn check_link(link: &PhysicalLink, located: &HashMap<&str, GeoPoint>, out: &mut Vec<Finding>) {
    let element = link.attributes.get("id").and_then(AttributeValue::as_string).map(str::to_string).unwrap_or_else(|| link.name.clone());
    let mut push = |attribute: &'static str, message: String| out.push(Finding { kind: "PhysicalLink", element: element.clone(), attribute, message });
    let declared = length_of(link.attributes.get("length"));
    let most = length_of(link.attributes.get("max_length"));
    // The end of a link is a node or one of its ports (`Node.port`).
    let end = |name: &str| located.get(name).or_else(|| name.split_once('.').and_then(|(owner, _)| located.get(owner))).copied();
    let way = link.attributes.get("route").and_then(|v| route(v).ok()).map(|points| route_length(&points));
    let apart = match (end(&link.from), end(&link.to)) {
        (Some(a), Some(b)) => Some(distance(a, b)),
        _ => None,
    };
    let floor = |metres: f64| metres * (1.0 - SPHERE_TOLERANCE);
    if let (Some(length), Some(way)) = (declared, way) {
        if length < floor(way) {
            push("length", format!("the link is shorter than its route: {} declared, the route covers {}", km(length), km(way)));
        }
    }
    if let (Some(length), None, Some(apart)) = (declared, way, apart) {
        if length < floor(apart) {
            push("length", format!("the link is shorter than the distance between its ends: {} declared, its ends are {} apart", km(length), km(apart)));
        }
    }
    if let Some(most) = most {
        // The least the medium can be: what is declared, else its route, else the distance between its ends.
        let least = declared.or(way.map(floor)).or(apart.map(floor));
        if let Some(least) = least.filter(|least| *least > most) {
            let what = if declared.is_some() { "its length" } else if way.is_some() { "its route" } else { "the distance between its ends" };
            push("max_length", format!("the link is beyond its max_length: {what} is {}, at most {} allowed", km(least), km(most)));
        }
    }
    if let Some(latency) = seconds(link.attributes.get("latency")) {
        let least = declared.or(way.map(floor)).or(apart.map(floor));
        if let Some(least) = least {
            let light = least / SPEED_OF_LIGHT;
            if latency < light {
                push("latency", format!("a latency of {:.3} ms is faster than light over {}: no signal crosses it in less than {:.3} ms", latency * 1e3, km(least), light * 1e3));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn findings(link: &str) -> Vec<Finding> {
        let model = format!(
            "model T {{ name: \"t\" }}\nphysical_architecture \"P\" {{\n  node \"Paris\" {{ id: \"PN-P\" location: {{ lat: 48.8566 deg lon: 2.3522 deg }} }}\n  node \"London\" {{ id: \"PN-L\" location: {{ lat: 51.5074 deg lon: -0.1278 deg }} }}\n  physical_link \"Cable\" {{ from: \"PN-P\" to: \"PN-L\" {link} }}\n}}\n"
        );
        check(&Compiler::new(CompilerConfig::default()).compile_string(&model).expect("compiles").ast)
    }

    #[test]
    fn a_link_shorter_than_the_distance_between_its_ends_is_found() {
        let found = findings("length: 100 km");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!((found[0].attribute, found[0].element.as_str()), ("length", "Cable"));
        assert!(found[0].message.contains("its ends are 34"), "{}", found[0].message);
        assert!(findings("length: 400 km").is_empty());
    }

    #[test]
    fn a_rounding_of_the_sphere_is_never_a_finding() {
        // 343.5 km on the ellipsoid; 342 km is within the sphere's tolerance.
        assert!(findings("length: 342 km").is_empty());
    }

    #[test]
    fn a_route_is_measured_and_bounds_the_length() {
        let detour = "route: [{ lat: 48.8566 deg lon: 2.3522 deg }, { lat: 50.95 deg lon: 1.85 deg }, { lat: 51.5074 deg lon: -0.1278 deg }]";
        let found = findings(&format!("{detour} length: 360 km"));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].message.contains("shorter than its route"), "{}", found[0].message);
        assert!(findings(&format!("{detour} length: 400 km")).is_empty());
    }

    #[test]
    fn max_length_and_light_are_limits() {
        let found = findings("max_length: 100 km");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].attribute, "max_length");
        let faster = findings("length: 350 km latency: 1 ms");
        assert_eq!(faster.len(), 1, "{faster:?}");
        assert!(faster[0].message.contains("faster than light"), "{}", faster[0].message);
        assert!(findings("length: 350 km latency: 2 ms").is_empty());
    }

    #[test]
    fn links_without_geometry_are_not_judged() {
        let model = "model T { name: \"t\" }\nphysical_architecture \"P\" {\n  node \"A\" { id: \"PN-A\" }\n  node \"B\" { id: \"PN-B\" }\n  physical_link \"L\" { from: \"PN-A\" to: \"PN-B\" length: 1 m latency: 1 ns }\n}\n";
        assert!(check(&Compiler::new(CompilerConfig::default()).compile_string(model).unwrap().ast).iter().all(|f| f.attribute == "latency"));
    }
}
