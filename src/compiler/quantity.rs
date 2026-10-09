//! Typed physical quantities: a number with a unit, carrying a dimension.
//!
//! This is the foundation of the typed metamodel. An attribute such as
//! `latency: 25 ms` is no longer an opaque string the production gate has
//! to re-parse: it is a `Quantity` whose dimension (Time) is known at compile
//! time, can be checked against the attribute's declared type, and converts
//! to a canonical SI value for arithmetic (timing budgets, bandwidth sums).
//!
//! Legacy string forms (`"25 ms"`, `"100Mbps"`, `"0.1 s"`) parse to the same
//! `Quantity` so existing models keep their semantics (semver MINOR).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Physical dimension of a quantity. Each dimension has ONE canonical unit
/// (SI base or coherent derived unit) that conversions normalize to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dimension {
    /// canonical: second (s)
    Time,
    /// canonical: hertz (Hz)
    Frequency,
    /// canonical: bit per second (bit/s)
    DataRate,
    /// canonical: bit
    Information,
    /// canonical: metre (m)
    Length,
    /// canonical: kilogram (kg)
    Mass,
    /// canonical: newton (N)
    Force,
    /// canonical: volt (V)
    Voltage,
    /// canonical: ampere (A)
    Current,
    /// canonical: watt (W)
    Power,
    /// canonical: joule (J)
    Energy,
    /// canonical: pascal (Pa)
    Pressure,
    /// canonical: kelvin (K)
    Temperature,
    /// canonical: metre per second (m/s)
    Speed,
    /// canonical: one (ratio); percent is 0.01
    Ratio,
}

impl Dimension {
    /// Symbol of the canonical unit of this dimension.
    pub fn canonical_unit(self) -> &'static str {
        match self {
            Dimension::Time => "s",
            Dimension::Frequency => "Hz",
            Dimension::DataRate => "bit/s",
            Dimension::Information => "bit",
            Dimension::Length => "m",
            Dimension::Mass => "kg",
            Dimension::Force => "N",
            Dimension::Voltage => "V",
            Dimension::Current => "A",
            Dimension::Power => "W",
            Dimension::Energy => "J",
            Dimension::Pressure => "Pa",
            Dimension::Temperature => "K",
            Dimension::Speed => "m/s",
            Dimension::Ratio => "one",
        }
    }

    /// Human-readable name used in diagnostics ("a time quantity").
    pub fn label(self) -> &'static str {
        match self {
            Dimension::Time => "time",
            Dimension::Frequency => "frequency",
            Dimension::DataRate => "data rate",
            Dimension::Information => "data size",
            Dimension::Length => "length",
            Dimension::Mass => "mass",
            Dimension::Force => "force",
            Dimension::Voltage => "voltage",
            Dimension::Current => "electric current",
            Dimension::Power => "power",
            Dimension::Energy => "energy",
            Dimension::Pressure => "pressure",
            Dimension::Temperature => "temperature",
            Dimension::Speed => "speed",
            Dimension::Ratio => "ratio",
        }
    }

    /// SysML v2 standard-library quantity value type (ISQ) for this dimension.
    pub fn sysml_value_type(self) -> &'static str {
        match self {
            Dimension::Time => "DurationValue",
            Dimension::Frequency => "FrequencyValue",
            Dimension::DataRate => "BinaryDigitRateValue",
            Dimension::Information => "StorageCapacityValue",
            Dimension::Length => "LengthValue",
            Dimension::Mass => "MassValue",
            Dimension::Force => "ForceValue",
            Dimension::Voltage => "ElectricPotentialValue",
            Dimension::Current => "ElectricCurrentValue",
            Dimension::Power => "PowerValue",
            Dimension::Energy => "EnergyValue",
            Dimension::Pressure => "PressureValue",
            Dimension::Temperature => "ThermodynamicTemperatureValue",
            Dimension::Speed => "SpeedValue",
            Dimension::Ratio => "DimensionOneValue",
        }
    }
}

/// Exponents of the base dimensions (time, length, mass, electric current,
/// temperature, information). Products and quotients of quantities are
/// tracked with these, so `voltage * current` is a power and
/// `data size / data rate` is a time. Every canonical unit of the table is
/// SI-coherent, so canonical values multiply and divide without extra factors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DimVec {
    pub time: i8,
    pub length: i8,
    pub mass: i8,
    pub current: i8,
    pub temperature: i8,
    pub information: i8,
}

impl DimVec {
    pub const NONE: DimVec = DimVec { time: 0, length: 0, mass: 0, current: 0, temperature: 0, information: 0 };

    const fn new(time: i8, length: i8, mass: i8, current: i8, temperature: i8, information: i8) -> Self {
        DimVec { time, length, mass, current, temperature, information }
    }

    pub fn is_dimensionless(self) -> bool {
        self == DimVec::NONE
    }

    pub fn multiply(self, other: DimVec) -> DimVec {
        DimVec::new(
            self.time + other.time,
            self.length + other.length,
            self.mass + other.mass,
            self.current + other.current,
            self.temperature + other.temperature,
            self.information + other.information,
        )
    }

    pub fn divide(self, other: DimVec) -> DimVec {
        DimVec::new(
            self.time - other.time,
            self.length - other.length,
            self.mass - other.mass,
            self.current - other.current,
            self.temperature - other.temperature,
            self.information - other.information,
        )
    }

    /// The named dimension with exactly these exponents, if any. A
    /// dimensionless vector is `None` (a plain number or a ratio).
    pub fn named(self) -> Option<Dimension> {
        if self.is_dimensionless() {
            return None;
        }
        ALL_DIMENSIONS.iter().copied().find(|d| d.vector() == self)
    }

    /// Unit of a value with these exponents, in canonical SI units:
    /// the named dimension's unit, or a product such as `m^2·s^-1`.
    pub fn unit(self) -> String {
        if let Some(dimension) = self.named() {
            return dimension.canonical_unit().to_string();
        }
        let parts: Vec<String> = [
            ("kg", self.mass),
            ("m", self.length),
            ("s", self.time),
            ("A", self.current),
            ("K", self.temperature),
            ("bit", self.information),
        ]
        .iter()
        .filter(|(_, exponent)| *exponent != 0)
        .map(|(symbol, exponent)| if *exponent == 1 { symbol.to_string() } else { format!("{}^{}", symbol, exponent) })
        .collect();
        parts.join("·")
    }

    /// "time", "power", "plain number", or "quantity in m^2·s^-1".
    pub fn label(self) -> String {
        if self.is_dimensionless() {
            return "plain number".to_string();
        }
        match self.named() {
            Some(dimension) => dimension.label().to_string(),
            None => format!("quantity in {}", self.unit()),
        }
    }
}

const ALL_DIMENSIONS: [Dimension; 15] = [
    Dimension::Time,
    Dimension::Frequency,
    Dimension::DataRate,
    Dimension::Information,
    Dimension::Length,
    Dimension::Mass,
    Dimension::Force,
    Dimension::Voltage,
    Dimension::Current,
    Dimension::Power,
    Dimension::Energy,
    Dimension::Pressure,
    Dimension::Temperature,
    Dimension::Speed,
    Dimension::Ratio,
];

impl Dimension {
    /// Base-dimension exponents of this dimension's canonical unit.
    pub fn vector(self) -> DimVec {
        //                       time len mass cur temp info
        match self {
            Dimension::Time => DimVec::new(1, 0, 0, 0, 0, 0),
            Dimension::Frequency => DimVec::new(-1, 0, 0, 0, 0, 0),
            Dimension::DataRate => DimVec::new(-1, 0, 0, 0, 0, 1),
            Dimension::Information => DimVec::new(0, 0, 0, 0, 0, 1),
            Dimension::Length => DimVec::new(0, 1, 0, 0, 0, 0),
            Dimension::Mass => DimVec::new(0, 0, 1, 0, 0, 0),
            Dimension::Force => DimVec::new(-2, 1, 1, 0, 0, 0),
            Dimension::Voltage => DimVec::new(-3, 2, 1, -1, 0, 0),
            Dimension::Current => DimVec::new(0, 0, 0, 1, 0, 0),
            Dimension::Power => DimVec::new(-3, 2, 1, 0, 0, 0),
            Dimension::Energy => DimVec::new(-2, 2, 1, 0, 0, 0),
            Dimension::Pressure => DimVec::new(-2, -1, 1, 0, 0, 0),
            Dimension::Temperature => DimVec::new(0, 0, 0, 0, 1, 0),
            Dimension::Speed => DimVec::new(-1, 1, 0, 0, 0, 0),
            Dimension::Ratio => DimVec::NONE,
        }
    }
}

/// One entry of the unit table.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct UnitSpec {
    /// Symbol as written in ArcLang source (`ms`, `Mbps`, `kg`).
    pub symbol: &'static str,
    pub dimension: Dimension,
    /// Multiply a value in this unit by `factor` to get the canonical unit.
    pub factor: f64,
    /// Name in the SysML v2 `SI` library (or an expression over it).
    pub sysml: &'static str,
}

const fn unit(symbol: &'static str, dimension: Dimension, factor: f64, sysml: &'static str) -> UnitSpec {
    UnitSpec { symbol, dimension, factor, sysml }
}

/// The unit table. Symbols are case-sensitive (SI): `ms` is milliseconds,
/// `Ms` is not a unit. A few widespread non-SI spellings (`Kbps`, `KB`,
/// `kbit/s`) are accepted as aliases because they are everywhere in ICDs.
pub const UNITS: &[UnitSpec] = &[
    // Time
    unit("ns", Dimension::Time, 1e-9, "ns"),
    unit("us", Dimension::Time, 1e-6, "us"),
    unit("µs", Dimension::Time, 1e-6, "us"),
    unit("ms", Dimension::Time, 1e-3, "ms"),
    unit("s", Dimension::Time, 1.0, "s"),
    unit("min", Dimension::Time, 60.0, "min"),
    unit("h", Dimension::Time, 3600.0, "h"),
    // Frequency
    unit("Hz", Dimension::Frequency, 1.0, "Hz"),
    unit("kHz", Dimension::Frequency, 1e3, "kHz"),
    unit("MHz", Dimension::Frequency, 1e6, "MHz"),
    unit("GHz", Dimension::Frequency, 1e9, "GHz"),
    // Data rate
    unit("bps", Dimension::DataRate, 1.0, "'bit/s'"),
    unit("bit/s", Dimension::DataRate, 1.0, "'bit/s'"),
    unit("kbps", Dimension::DataRate, 1e3, "'kbit/s'"),
    unit("Kbps", Dimension::DataRate, 1e3, "'kbit/s'"),
    unit("kbit/s", Dimension::DataRate, 1e3, "'kbit/s'"),
    unit("Mbps", Dimension::DataRate, 1e6, "'Mbit/s'"),
    unit("Mbit/s", Dimension::DataRate, 1e6, "'Mbit/s'"),
    unit("Gbps", Dimension::DataRate, 1e9, "'Gbit/s'"),
    unit("Gbit/s", Dimension::DataRate, 1e9, "'Gbit/s'"),
    // Information
    unit("bit", Dimension::Information, 1.0, "bit"),
    unit("B", Dimension::Information, 8.0, "B"),
    unit("kB", Dimension::Information, 8e3, "kB"),
    unit("KB", Dimension::Information, 8e3, "kB"),
    unit("MB", Dimension::Information, 8e6, "MB"),
    unit("GB", Dimension::Information, 8e9, "GB"),
    unit("KiB", Dimension::Information, 8.0 * 1024.0, "KiB"),
    unit("MiB", Dimension::Information, 8.0 * 1024.0 * 1024.0, "MiB"),
    unit("GiB", Dimension::Information, 8.0 * 1024.0 * 1024.0 * 1024.0, "GiB"),
    // Length
    unit("mm", Dimension::Length, 1e-3, "mm"),
    unit("cm", Dimension::Length, 1e-2, "cm"),
    unit("m", Dimension::Length, 1.0, "m"),
    unit("km", Dimension::Length, 1e3, "km"),
    // Mass
    unit("g", Dimension::Mass, 1e-3, "g"),
    unit("kg", Dimension::Mass, 1.0, "kg"),
    unit("t", Dimension::Mass, 1e3, "t"),
    // Force
    unit("N", Dimension::Force, 1.0, "N"),
    unit("kN", Dimension::Force, 1e3, "kN"),
    // Electricity
    unit("mV", Dimension::Voltage, 1e-3, "mV"),
    unit("V", Dimension::Voltage, 1.0, "V"),
    unit("kV", Dimension::Voltage, 1e3, "kV"),
    unit("mA", Dimension::Current, 1e-3, "mA"),
    unit("A", Dimension::Current, 1.0, "A"),
    // Power / energy
    unit("mW", Dimension::Power, 1e-3, "mW"),
    unit("W", Dimension::Power, 1.0, "W"),
    unit("kW", Dimension::Power, 1e3, "kW"),
    unit("J", Dimension::Energy, 1.0, "J"),
    unit("kJ", Dimension::Energy, 1e3, "kJ"),
    unit("Wh", Dimension::Energy, 3600.0, "'W*h'"),
    unit("kWh", Dimension::Energy, 3.6e6, "'kW*h'"),
    // Pressure
    unit("Pa", Dimension::Pressure, 1.0, "Pa"),
    unit("kPa", Dimension::Pressure, 1e3, "kPa"),
    unit("bar", Dimension::Pressure, 1e5, "bar"),
    // Temperature (absolute scales only; offsets are not linear factors)
    unit("K", Dimension::Temperature, 1.0, "K"),
    // Speed
    unit("m/s", Dimension::Speed, 1.0, "'m/s'"),
    unit("km/h", Dimension::Speed, 1000.0 / 3600.0, "'km/h'"),
    // Ratio
    unit("percent", Dimension::Ratio, 0.01, "'%'"),
    unit("%", Dimension::Ratio, 0.01, "'%'"),
];

/// Look a unit symbol up in the table.
pub fn lookup_unit(symbol: &str) -> Option<&'static UnitSpec> {
    UNITS.iter().find(|u| u.symbol == symbol)
}

/// A numeric value with a unit. `unit` is always a symbol from [`UNITS`]:
/// a `Quantity` with an unknown unit cannot be constructed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

/// Why a text or token pair is not a quantity.
#[derive(Debug, Clone, PartialEq)]
pub enum QuantityError {
    /// No numeric prefix at all ("Continuous", "fast").
    NotANumber(String),
    /// A number followed by something that is not a known unit symbol.
    UnknownUnit { value: f64, unit: String },
    /// A bare number: dimension cannot be inferred.
    MissingUnit(f64),
}

impl fmt::Display for QuantityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantityError::NotANumber(text) => write!(f, "'{}' is not a quantity (expected <number> <unit>)", text),
            QuantityError::UnknownUnit { value, unit } => {
                write!(f, "unknown unit '{}' in '{} {}'", unit, value, unit)
            }
            QuantityError::MissingUnit(value) => write!(f, "'{}' has no unit", value),
        }
    }
}

impl Quantity {
    /// Build a quantity from a value and a unit symbol; fails on unknown units.
    pub fn new(value: f64, unit: &str) -> Result<Self, QuantityError> {
        match lookup_unit(unit) {
            Some(spec) => Ok(Self { value, unit: spec.symbol.to_string() }),
            None => Err(QuantityError::UnknownUnit { value, unit: unit.to_string() }),
        }
    }

    /// Parse a legacy textual quantity: `"25 ms"`, `"100Mbps"`, `"0.1 s"`,
    /// `"-3 V"`. Whitespace between number and unit is optional.
    pub fn parse(text: &str) -> Result<Self, QuantityError> {
        let trimmed = text.trim();
        let number_end = trimmed
            .char_indices()
            .take_while(|(i, c)| {
                c.is_ascii_digit() || *c == '.' || *c == '_' || (*i == 0 && (*c == '-' || *c == '+'))
            })
            .map(|(i, c)| i + c.len_utf8())
            .last()
            .unwrap_or(0);
        let number_text: String = trimmed[..number_end].chars().filter(|c| *c != '_').collect();
        let value: f64 = number_text
            .parse()
            .map_err(|_| QuantityError::NotANumber(text.to_string()))?;
        let unit = trimmed[number_end..].trim();
        if unit.is_empty() {
            return Err(QuantityError::MissingUnit(value));
        }
        Self::new(value, unit)
    }

    pub fn spec(&self) -> &'static UnitSpec {
        // Invariant: `unit` always comes from the table (see `new`).
        lookup_unit(&self.unit).expect("Quantity unit is always a known unit symbol")
    }

    pub fn dimension(&self) -> Dimension {
        self.spec().dimension
    }

    /// Value expressed in the canonical unit of its dimension.
    pub fn canonical(&self) -> f64 {
        self.value * self.spec().factor
    }

    /// Convert to another unit of the SAME dimension.
    pub fn to_unit(&self, unit: &str) -> Result<Quantity, QuantityError> {
        let target = lookup_unit(unit).ok_or_else(|| QuantityError::UnknownUnit {
            value: self.value,
            unit: unit.to_string(),
        })?;
        if target.dimension != self.dimension() {
            return Err(QuantityError::UnknownUnit { value: self.value, unit: unit.to_string() });
        }
        Ok(Quantity { value: self.canonical() / target.factor, unit: target.symbol.to_string() })
    }

    /// Convenience for the production gate: milliseconds if this is a time.
    pub fn as_millis(&self) -> Option<f64> {
        (self.dimension() == Dimension::Time).then(|| self.canonical() * 1e3)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value.fract() == 0.0 && self.value.abs() < 1e15 {
            write!(f, "{} {}", self.value as i64, self.unit)
        } else {
            write!(f, "{} {}", self.value, self.unit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_strings_with_and_without_space() {
        assert_eq!(Quantity::parse("25 ms").unwrap(), Quantity::new(25.0, "ms").unwrap());
        assert_eq!(Quantity::parse("100Mbps").unwrap(), Quantity::new(100.0, "Mbps").unwrap());
        assert_eq!(Quantity::parse("0.1 s").unwrap().canonical(), 0.1);
        assert_eq!(Quantity::parse("  -3 V ").unwrap().value, -3.0);
        assert_eq!(Quantity::parse("1_000 ms").unwrap().value, 1000.0);
    }

    #[test]
    fn rejects_non_quantities_with_a_reason() {
        assert_eq!(
            Quantity::parse("Continuous"),
            Err(QuantityError::NotANumber("Continuous".to_string()))
        );
        assert_eq!(Quantity::parse("50"), Err(QuantityError::MissingUnit(50.0)));
        assert_eq!(
            Quantity::parse("50ms cycle"),
            Err(QuantityError::UnknownUnit { value: 50.0, unit: "ms cycle".to_string() })
        );
        assert!(Quantity::new(1.0, "Ms").is_err(), "unit symbols are case-sensitive");
    }

    #[test]
    fn converts_within_a_dimension_and_refuses_across() {
        let latency = Quantity::parse("0.25 s").unwrap();
        assert_eq!(latency.as_millis(), Some(250.0));
        assert_eq!(latency.to_unit("ms").unwrap(), Quantity::new(250.0, "ms").unwrap());
        assert_eq!(Quantity::parse("500 kbps").unwrap().to_unit("Mbps").unwrap().value, 0.5);
        assert_eq!(Quantity::parse("1 KiB").unwrap().to_unit("B").unwrap().value, 1024.0);
        assert!(Quantity::parse("135 MHz").unwrap().to_unit("Mbps").is_err());
        assert_eq!(Quantity::parse("135 MHz").unwrap().as_millis(), None);
    }

    #[test]
    fn every_dimension_has_its_canonical_unit_in_the_table() {
        for unit in UNITS {
            let canonical = unit.dimension.canonical_unit();
            if canonical == "one" {
                continue;
            }
            let spec = lookup_unit(canonical)
                .unwrap_or_else(|| panic!("canonical unit {} of {:?} missing from table", canonical, unit.dimension));
            assert_eq!(spec.factor, 1.0, "canonical unit {} must have factor 1", canonical);
        }
    }

    #[test]
    fn dimension_vectors_are_distinct_and_compose() {
        // No two named dimensions share a vector (Ratio is dimensionless).
        for (i, a) in ALL_DIMENSIONS.iter().enumerate() {
            for b in &ALL_DIMENSIONS[i + 1..] {
                assert_ne!(a.vector(), b.vector(), "{:?} and {:?} are indistinguishable", a, b);
            }
        }
        let v = |d: Dimension| d.vector();
        assert_eq!(v(Dimension::Voltage).multiply(v(Dimension::Current)).named(), Some(Dimension::Power));
        assert_eq!(v(Dimension::Power).multiply(v(Dimension::Time)).named(), Some(Dimension::Energy));
        assert_eq!(v(Dimension::Length).divide(v(Dimension::Time)).named(), Some(Dimension::Speed));
        assert_eq!(v(Dimension::Information).divide(v(Dimension::DataRate)).named(), Some(Dimension::Time));
        assert_eq!(v(Dimension::Force).divide(v(Dimension::Length).multiply(v(Dimension::Length))).named(), Some(Dimension::Pressure));
        assert!(v(Dimension::Frequency).multiply(v(Dimension::Time)).is_dimensionless());
        assert_eq!(v(Dimension::Time).multiply(v(Dimension::Time)).label(), "quantity in s^2");
        assert_eq!(v(Dimension::Length).multiply(v(Dimension::Length)).divide(v(Dimension::Time)).unit(), "m^2·s^-1");
        assert_eq!(DimVec::NONE.label(), "plain number");
    }

    #[test]
    fn displays_integers_without_decimals() {
        assert_eq!(Quantity::new(25.0, "ms").unwrap().to_string(), "25 ms");
        assert_eq!(Quantity::new(0.5, "s").unwrap().to_string(), "0.5 s");
    }

    #[test]
    fn serializes_as_value_and_unit() {
        let json = serde_json::to_string(&Quantity::new(25.0, "ms").unwrap()).unwrap();
        assert_eq!(json, r#"{"value":25.0,"unit":"ms"}"#);
    }
}
