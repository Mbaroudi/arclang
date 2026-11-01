//! Typed ArcLang quantities as Modelica parameters in SI units.

use crate::compiler::ast::AttributeValue;
use crate::compiler::quantity::Dimension;

/// The Modelica type of a dimension: an `Modelica.Units.SI` type, or
/// `Real` with the unit string when the SI package has none.
pub fn si_type(dimension: Dimension) -> String {
    let si = match dimension {
        Dimension::Time => "Time",
        Dimension::Frequency => "Frequency",
        Dimension::Length => "Length",
        Dimension::Mass => "Mass",
        Dimension::Force => "Force",
        Dimension::Voltage => "Voltage",
        Dimension::Current => "Current",
        Dimension::Power => "Power",
        Dimension::Energy => "Energy",
        Dimension::Pressure => "Pressure",
        Dimension::Temperature => "Temperature",
        Dimension::Speed => "Velocity",
        Dimension::Angle => "Angle",
        Dimension::Charge => "ElectricCharge",
        Dimension::Resistance => "Resistance",
        Dimension::ThermalConductance => "ThermalConductance",
        Dimension::HeatCapacity => "HeatCapacity",
        Dimension::Area => "Area",
        Dimension::MassFlowRate => "MassFlowRate",
        Dimension::Ratio => return "Real(unit=\"1\")".to_string(),
        Dimension::DataRate | Dimension::Information => return format!("Real(unit=\"{}\")", dimension.canonical_unit()),
    };
    format!("Modelica.Units.SI.{si}")
}

/// A parameter an attribute becomes: its type, its value in SI and, for
/// a quantity, as written (for the comment; empty for a plain number).
/// `None` for what is no number.
pub fn parameter(value: &AttributeValue) -> Option<(String, f64, String)> {
    match value {
        AttributeValue::Quantity(quantity) => Some((si_type(quantity.dimension()), quantity.canonical(), quantity.to_string())),
        AttributeValue::Number(n) => Some(("Real".to_string(), *n, String::new())),
        AttributeValue::Integer(i) => Some(("Real".to_string(), *i as f64, String::new())),
        _ => None,
    }
}

/// Splits `Real(unit="1")` into `Real` and its modifier, for a declaration
/// that adds its own (`Real x(unit="1") = 0.9`).
pub fn declaration(ty: &str, name: &str) -> String {
    match ty.split_once('(') {
        Some((base, modifier)) => format!("{base} {name}({modifier}"),
        None => format!("{ty} {name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::quantity::Quantity;

    #[test]
    fn quantities_become_si_parameters() {
        let energy = AttributeValue::Quantity(Quantity::new(82.0, "kWh").unwrap());
        let (ty, value, written) = parameter(&energy).unwrap();
        assert_eq!(ty, "Modelica.Units.SI.Energy");
        assert_eq!(value, 2.952e8);
        assert_eq!(written, "82 kWh");
        let ratio = AttributeValue::Quantity(Quantity::new(90.0, "%").unwrap());
        let (ty, value, _) = parameter(&ratio).unwrap();
        assert_eq!(declaration(&ty, "efficiency"), "Real efficiency(unit=\"1\")");
        assert!((value - 0.9).abs() < 1e-12);
        assert_eq!(si_type(Dimension::DataRate), "Real(unit=\"bit/s\")");
        assert!(parameter(&AttributeValue::String("x".into())).is_none());
        let warm = AttributeValue::Quantity(Quantity::new(35.0, "°C").unwrap());
        let (ty, value, written) = parameter(&warm).unwrap();
        assert_eq!((ty.as_str(), written.as_str()), ("Modelica.Units.SI.Temperature", "35 °C"));
        assert!((value - 308.15).abs() < 1e-9, "Modelica takes kelvin");
        let ohm = AttributeValue::Quantity(Quantity::new(25.0, "mOhm").unwrap());
        assert_eq!(parameter(&ohm).unwrap().0, "Modelica.Units.SI.Resistance");
    }
}
