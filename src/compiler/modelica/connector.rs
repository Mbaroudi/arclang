//! The physics a port carries (`physics:`) and the Modelica Standard
//! Library connector it becomes.
//!
//! | `physics:`                 | connector                                                        |
//! |----------------------------|------------------------------------------------------------------|
//! | `signal`                   | `Modelica.Blocks.Interfaces.RealInput` (in) / `RealOutput` (out) |
//! | `electrical`               | `Modelica.Electrical.Analog.Interfaces.PositivePin` / `NegativePin` (`pole: negative`) |
//! | `mechanical_translational` | `Modelica.Mechanics.Translational.Interfaces.Flange_a` / `Flange_b` |
//! | `mechanical_rotational`    | `Modelica.Mechanics.Rotational.Interfaces.Flange_a` / `Flange_b` |
//! | `thermal`                  | `Modelica.Thermal.HeatTransfer.Interfaces.HeatPort_a` / `HeatPort_b` |
//! | `fluid`                    | deferred: not exported                                           |
//!
//! A flange or heat port is `_b` when the port says `flange: b`, or is an
//! `out` port saying nothing; `_a` otherwise.

use crate::compiler::ast::{AttributeValue, PortDirection};
use std::collections::HashMap;

/// The values `physics:` takes.
pub const PHYSICS: [&str; 6] = ["signal", "electrical", "mechanical_translational", "mechanical_rotational", "thermal", "fluid"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Physics {
    Signal,
    Electrical,
    Translational,
    Rotational,
    Thermal,
    Fluid,
}

impl Physics {
    pub fn parse(text: &str) -> Option<Physics> {
        Some(match text {
            "signal" => Physics::Signal,
            "electrical" => Physics::Electrical,
            "mechanical_translational" => Physics::Translational,
            "mechanical_rotational" => Physics::Rotational,
            "thermal" => Physics::Thermal,
            "fluid" => Physics::Fluid,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        PHYSICS[self as usize]
    }
}

/// Orientation of a port, as the model writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    In,
    Out,
    InOut,
    /// A physical port: Arcadia does not orient them.
    None,
}

impl From<&PortDirection> for Orientation {
    fn from(direction: &PortDirection) -> Self {
        match direction {
            PortDirection::In => Orientation::In,
            PortDirection::Out => Orientation::Out,
            _ => Orientation::InOut,
        }
    }
}

fn text<'a>(attributes: &'a HashMap<String, AttributeValue>, key: &str) -> Option<&'a str> {
    attributes.get(key).and_then(|v| v.as_string())
}

/// The physics a port declares: its own `physics:`, else that of its
/// definition (`is:`, in `inherited`). `Err` for a value that is none.
pub fn physics_of(attributes: &HashMap<String, AttributeValue>, inherited: Option<&HashMap<String, AttributeValue>>) -> Result<Option<Physics>, String> {
    let written = text(attributes, "physics").or_else(|| inherited.and_then(|i| text(i, "physics")));
    match written {
        None => Ok(None),
        Some(value) => Physics::parse(value).map(Some).ok_or_else(|| format!("physics '{value}' is not one of {}", PHYSICS.join(", "))),
    }
}

/// The MSL connector of a port, or why it has none.
pub fn connector(physics: Physics, orientation: Orientation, attributes: &HashMap<String, AttributeValue>) -> Result<&'static str, String> {
    let side_b = match text(attributes, "flange") {
        Some("b") => true,
        Some("a") => false,
        Some(other) => return Err(format!("flange '{other}' is not a or b")),
        None => orientation == Orientation::Out,
    };
    Ok(match physics {
        Physics::Signal => match orientation {
            Orientation::In => "Modelica.Blocks.Interfaces.RealInput",
            Orientation::Out => "Modelica.Blocks.Interfaces.RealOutput",
            _ => return Err("a signal port is causal: declare it `port in` or `port out`".to_string()),
        },
        Physics::Electrical => match text(attributes, "pole") {
            None | Some("positive") => "Modelica.Electrical.Analog.Interfaces.PositivePin",
            Some("negative") => "Modelica.Electrical.Analog.Interfaces.NegativePin",
            Some(other) => return Err(format!("pole '{other}' is not positive or negative")),
        },
        Physics::Translational if side_b => "Modelica.Mechanics.Translational.Interfaces.Flange_b",
        Physics::Translational => "Modelica.Mechanics.Translational.Interfaces.Flange_a",
        Physics::Rotational if side_b => "Modelica.Mechanics.Rotational.Interfaces.Flange_b",
        Physics::Rotational => "Modelica.Mechanics.Rotational.Interfaces.Flange_a",
        Physics::Thermal if side_b => "Modelica.Thermal.HeatTransfer.Interfaces.HeatPort_b",
        Physics::Thermal => "Modelica.Thermal.HeatTransfer.Interfaces.HeatPort_a",
        Physics::Fluid => return Err("fluid ports are deferred: Modelica.Fluid needs a medium, ArcLang does not export them yet".to_string()),
    })
}

/// One end of a connection, as the check sees it.
#[derive(Debug, Clone, Copy)]
pub struct End<'a> {
    pub label: &'a str,
    pub physics: Physics,
    pub orientation: Orientation,
    /// The end is a port of the model holding the connection (an outside
    /// connector), not of one of its parts.
    pub outside: bool,
}

/// Whether two ends may be connected: the same physics, and for signals
/// an output feeding an input (or an outside connector relayed inward,
/// which keeps its direction).
pub fn compatible(a: End<'_>, b: End<'_>) -> Result<(), String> {
    if a.physics != b.physics {
        return Err(format!("'{}' is {} and '{}' is {}: they cannot be connected", a.label, a.physics.label(), b.label, b.physics.label()));
    }
    if a.physics != Physics::Signal {
        return Ok(());
    }
    let relayed = a.outside != b.outside;
    match (a.orientation, b.orientation) {
        (Orientation::In, Orientation::In) | (Orientation::Out, Orientation::Out) if relayed => Ok(()),
        (Orientation::Out, Orientation::In) | (Orientation::In, Orientation::Out) if !relayed => Ok(()),
        (x, y) if x == y => Err(format!("'{}' and '{}' are both signal {}puts: a signal goes from an output to an input", a.label, b.label, if x == Orientation::In { "in" } else { "out" })),
        _ => Err(format!("'{}' and '{}': a signal relayed through a port keeps its direction", a.label, b.label)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(pairs: &[(&str, &str)]) -> HashMap<String, AttributeValue> {
        pairs.iter().map(|(k, v)| (k.to_string(), AttributeValue::String(v.to_string()))).collect()
    }

    #[test]
    fn connectors_follow_physics_direction_pole_and_flange() {
        let none = HashMap::new();
        assert_eq!(connector(Physics::Signal, Orientation::In, &none).unwrap(), "Modelica.Blocks.Interfaces.RealInput");
        assert_eq!(connector(Physics::Signal, Orientation::Out, &none).unwrap(), "Modelica.Blocks.Interfaces.RealOutput");
        assert!(connector(Physics::Signal, Orientation::None, &none).unwrap_err().contains("causal"));
        assert_eq!(connector(Physics::Electrical, Orientation::None, &none).unwrap(), "Modelica.Electrical.Analog.Interfaces.PositivePin");
        assert_eq!(connector(Physics::Electrical, Orientation::None, &attrs(&[("pole", "negative")])).unwrap(), "Modelica.Electrical.Analog.Interfaces.NegativePin");
        assert_eq!(connector(Physics::Rotational, Orientation::None, &attrs(&[("flange", "b")])).unwrap(), "Modelica.Mechanics.Rotational.Interfaces.Flange_b");
        assert_eq!(connector(Physics::Translational, Orientation::Out, &none).unwrap(), "Modelica.Mechanics.Translational.Interfaces.Flange_b");
        assert_eq!(connector(Physics::Thermal, Orientation::None, &none).unwrap(), "Modelica.Thermal.HeatTransfer.Interfaces.HeatPort_a");
        assert!(connector(Physics::Fluid, Orientation::None, &none).unwrap_err().contains("deferred"));
    }

    #[test]
    fn physics_comes_from_the_port_or_its_definition() {
        assert_eq!(physics_of(&attrs(&[("physics", "thermal")]), None).unwrap(), Some(Physics::Thermal));
        assert_eq!(physics_of(&HashMap::new(), Some(&attrs(&[("physics", "electrical")]))).unwrap(), Some(Physics::Electrical));
        assert_eq!(physics_of(&HashMap::new(), None).unwrap(), None);
        assert!(physics_of(&attrs(&[("physics", "hydraulic")]), None).is_err());
    }

    #[test]
    fn signals_go_from_outputs_to_inputs() {
        let end = |label, orientation, outside| End { label, physics: Physics::Signal, orientation, outside };
        assert!(compatible(end("a", Orientation::Out, false), end("b", Orientation::In, false)).is_ok());
        assert!(compatible(end("a", Orientation::In, false), end("b", Orientation::In, false)).is_err());
        assert!(compatible(end("a", Orientation::In, true), end("b", Orientation::In, false)).is_ok(), "relayed inward");
        assert!(compatible(end("a", Orientation::Out, true), end("b", Orientation::In, false)).is_err());
        let pin = End { label: "p", physics: Physics::Electrical, orientation: Orientation::None, outside: false };
        let flange = End { label: "f", physics: Physics::Rotational, orientation: Orientation::None, outside: false };
        assert!(compatible(pin, flange).unwrap_err().contains("electrical"));
    }
}
