//! The model on a map: GeoJSON (RFC 7946), which QGIS, ArcGIS, Leaflet,
//! Mapbox, kepler.gl and GitHub draw as they are.
//!
//! Every element with a `location` is a point; every physical link is a
//! line, along its `route` when it has one, else straight between its
//! located ends. Coordinates are WGS 84 longitude, latitude (and altitude
//! when written), in that order, as RFC 7946 requires. Elements without a
//! location are not drawn: a map never guesses where something is.

use super::{distance, location, route, route_length, GeoPoint};
use crate::compiler::ast::{AttributeValue, Model};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

fn coordinates(point: GeoPoint) -> Value {
    match point.alt {
        Some(alt) => json!([point.lon, point.lat, alt]),
        None => json!([point.lon, point.lat]),
    }
}

struct Collector {
    features: Vec<Value>,
    located: HashMap<String, GeoPoint>,
}

impl Collector {
    fn element(&mut self, kind: &str, layer: &str, id: &str, name: &str, attributes: &HashMap<String, AttributeValue>) {
        let Some(point) = attributes.get("location").and_then(|v| location(v).ok()) else { return };
        self.located.insert(id.to_string(), point);
        self.located.insert(name.to_string(), point);
        let mut properties = Map::new();
        properties.insert("id".into(), json!(id));
        properties.insert("name".into(), json!(name));
        properties.insert("kind".into(), json!(kind));
        properties.insert("layer".into(), json!(layer));
        if let Some(description) = attributes.get("description").and_then(AttributeValue::as_string) {
            properties.insert("description".into(), json!(description));
        }
        self.features.push(json!({
            "type": "Feature",
            "id": id,
            "geometry": { "type": "Point", "coordinates": coordinates(point) },
            "properties": properties,
        }));
    }
}

/// The located elements and the physical links of a model, as a GeoJSON
/// FeatureCollection; `notes` say what could not be drawn and why.
pub fn export(ast: &Model) -> (String, Vec<String>) {
    let mut c = Collector { features: Vec::new(), located: HashMap::new() };
    for oa in &ast.operational_analysis {
        for actor in &oa.actors {
            let id = actor.id.clone().unwrap_or_else(|| actor.name.clone());
            c.element("Actor", "Operational Analysis", &id, &actor.name, &actor.attributes);
        }
        for entity in &oa.entities {
            c.element("OperationalEntity", "Operational Analysis", &entity.id, &entity.name, &entity.attributes);
        }
    }
    for sa in &ast.system_analysis {
        for actor in &sa.external_actors {
            c.element("SystemActor", "System Analysis", &actor.id, &actor.name, &actor.attributes);
        }
    }
    for pa in &ast.physical_architecture {
        for node in &pa.nodes {
            c.element("PhysicalNode", "Physical Architecture", &node.id, &node.name, &node.attributes);
        }
    }
    for epbs in &ast.epbs {
        for system in &epbs.systems {
            c.element("EpbsSystem", "EPBS", &system.name, &system.name, &system.attributes);
            for subsystem in &system.subsystems {
                c.element("EpbsSubsystem", "EPBS", &subsystem.name, &subsystem.name, &subsystem.attributes);
                for item in &subsystem.items {
                    c.element("EpbsItem", "EPBS", &item.name, &item.name, &item.attributes);
                }
            }
        }
    }
    let mut notes = Vec::new();
    for pa in &ast.physical_architecture {
        for link in &pa.links {
            let id = link.attributes.get("id").and_then(AttributeValue::as_string).unwrap_or(&link.name).to_string();
            let end = |name: &str| c.located.get(name).or_else(|| name.split_once('.').and_then(|(owner, _)| c.located.get(owner))).copied();
            let (points, measured) = match link.attributes.get("route").and_then(|v| route(v).ok()) {
                Some(points) => {
                    let length = route_length(&points);
                    (points, ("route_length_m", length))
                }
                None => match (end(&link.from), end(&link.to)) {
                    (Some(a), Some(b)) => (vec![a, b], ("distance_m", distance(a, b))),
                    _ => {
                        notes.push(format!("physical link '{id}' is not drawn: it has no route and its ends are not both located"));
                        continue;
                    }
                },
            };
            let mut properties = Map::new();
            properties.insert("id".into(), json!(id));
            properties.insert("name".into(), json!(link.name));
            properties.insert("kind".into(), json!("PhysicalLink"));
            properties.insert("layer".into(), json!("Physical Architecture"));
            properties.insert("from".into(), json!(link.from));
            properties.insert("to".into(), json!(link.to));
            properties.insert(measured.0.into(), json!((measured.1 * 10.0).round() / 10.0));
            if !link.protocol.is_empty() && link.protocol != "Unknown" {
                properties.insert("protocol".into(), json!(link.protocol));
            }
            for key in ["length", "max_length", "latency", "bandwidth"] {
                if let Some(value) = link.attributes.get(key) {
                    properties.insert(key.into(), json!(value.display()));
                }
            }
            c.features.push(json!({
                "type": "Feature",
                "id": id,
                "geometry": { "type": "LineString", "coordinates": points.into_iter().map(coordinates).collect::<Vec<_>>() },
                "properties": properties,
            }));
        }
    }
    let name = ast.attributes.get("name").and_then(AttributeValue::as_string).unwrap_or(&ast.id);
    let collection = json!({ "type": "FeatureCollection", "name": name, "features": c.features });
    (format!("{}\n", serde_json::to_string_pretty(&collection).expect("plain data")), notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    #[test]
    fn located_elements_are_points_and_links_are_lines_in_longitude_latitude_order() {
        let model = "model T { name: \"Grid\" }\noperational_analysis \"OA\" {\n  entity \"Depot\" { id: \"OE-D\" location: { lat: 45.0 deg lon: 5.0 deg } }\n}\nphysical_architecture \"P\" {\n  node \"A\" { id: \"PN-A\" location: { lat: 48.0 deg lon: 2.0 deg alt: 30 m } }\n  node \"B\" { id: \"PN-B\" location: { lat: 49.0 deg lon: 3.0 deg } }\n  node \"C\" { id: \"PN-C\" }\n  physical_link \"AB\" { from: \"PN-A\" to: \"PN-B\" length: 140 km }\n  physical_link \"BC\" { from: \"PN-B\" to: \"PN-C\" }\n  physical_link \"Bent\" { from: \"PN-B\" to: \"PN-C\" route: [{ lat: 49.0 deg lon: 3.0 deg }, { lat: 49.5 deg lon: 3.5 deg }] }\n}\n";
        let result = Compiler::new(CompilerConfig::default()).compile_string(model).unwrap();
        let (text, notes) = export(&result.ast);
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["type"], "FeatureCollection");
        let features = value["features"].as_array().unwrap();
        let by_id = |id: &str| features.iter().find(|f| f["id"] == id).unwrap_or_else(|| panic!("{id} in {text}"));
        assert_eq!(by_id("PN-A")["geometry"]["coordinates"], json!([2.0, 48.0, 30.0]));
        assert_eq!(by_id("OE-D")["properties"]["layer"], "Operational Analysis");
        assert_eq!(by_id("AB")["geometry"]["type"], "LineString");
        assert_eq!(by_id("AB")["geometry"]["coordinates"], json!([[2.0, 48.0, 30.0], [3.0, 49.0]]));
        assert!(by_id("AB")["properties"]["distance_m"].as_f64().unwrap() > 130_000.0);
        assert_eq!(by_id("Bent")["geometry"]["coordinates"].as_array().unwrap().len(), 2);
        assert!(features.iter().all(|f| f["id"] != "PN-C" && f["id"] != "BC"), "nothing is guessed");
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("'BC'"));
    }
}
