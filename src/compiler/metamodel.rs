//! The ArcLang metamodel — declared in code, independent of Capella.
//!
//! This module is the single machine-readable description of what an
//! ArcLang model *is*: the element kinds of each Arcadia layer, the typed
//! attributes each kind carries, the enumerations those attributes draw
//! from, the unit table, and the rules governing traceability links.
//!
//! Everything downstream derives from it:
//! - the compiler validates attribute values against it
//!   (`metamodel_check`), so `latency: 135 MHz` is a type error and
//!   `asil: "High"` is an invalid enumeration value, not a string nobody reads;
//! - `arclang metamodel` prints it (JSON or Markdown) and
//!   `spec/METAMODEL.md` is generated from it — a golden test fails when the
//!   document and the code disagree. The spec cannot lie about the compiler.
//! - the SysML v2 exporter uses its mapping column to choose target
//!   constructs and ISQ value types.
//!
//! Versioned with the language (see `docs/VERSIONING.md`): adding kinds,
//! attributes or enum values is MINOR; removing or retyping is MAJOR.

use super::quantity::{Dimension, UnitSpec, UNITS};
use serde::Serialize;

/// Version of the language this metamodel describes.
pub const LANGUAGE_VERSION: &str = "4.0.0";

/// Arcadia layer (plus the transverse concerns) an element kind belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Layer {
    Operational,
    System,
    Logical,
    Physical,
    Epbs,
    Data,
    Transverse,
}

impl Layer {
    pub fn label(self) -> &'static str {
        match self {
            Layer::Operational => "Operational Analysis",
            Layer::System => "System Analysis",
            Layer::Logical => "Logical Architecture",
            Layer::Physical => "Physical Architecture",
            Layer::Epbs => "EPBS",
            Layer::Data => "Data model",
            Layer::Transverse => "Transverse",
        }
    }
}

/// Type of an attribute value.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "of")]
pub enum AttrType {
    /// Free text.
    Text,
    /// A stable identifier (`id:`); unique across the model.
    Identifier,
    Number,
    Boolean,
    /// A number with a unit of the given dimension (`latency: 25 ms`).
    Quantity(Dimension),
    /// One value of a named enumeration (see [`Metamodel::enums`]).
    Enum(&'static str),
    /// How many of the element its owner has: a count (`4`) or a range
    /// (`"0..1"`, `"1..*"`, `"*"`).
    Multiplicity,
    /// A reference to another element, by id or unambiguous name. The
    /// string names the expected kind, or `"Element"` for any kind.
    Reference(&'static str),
    List(Box<AttrType>),
}

impl AttrType {
    /// Short notation used in the Markdown spec.
    pub fn notation(&self) -> String {
        match self {
            AttrType::Text => "text".into(),
            AttrType::Identifier => "identifier".into(),
            AttrType::Number => "number".into(),
            AttrType::Boolean => "boolean".into(),
            AttrType::Multiplicity => "multiplicity".into(),
            AttrType::Quantity(d) => format!("quantity<{}>", d.label()),
            AttrType::Enum(name) => format!("enum {}", name),
            AttrType::Reference(kind) => format!("ref<{}>", kind),
            AttrType::List(inner) => format!("list<{}>", inner.notation()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AttrSpec {
    pub key: &'static str,
    #[serde(rename = "type")]
    pub ty: AttrType,
    pub doc: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct KindSpec {
    /// Name used in diagnostics and in `SemanticModel::all_elements`.
    pub name: &'static str,
    pub layer: Layer,
    /// Arcadia / Capella metaclass this kind corresponds to.
    pub arcadia: &'static str,
    /// SysML v2 construct the exporter maps this kind to.
    pub sysml: &'static str,
    pub doc: &'static str,
    pub attributes: Vec<AttrSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnumSpec {
    pub name: &'static str,
    /// Canonical spellings. Values are matched ignoring case and the
    /// separators `-`, `_` and space (`ASIL_D`, `asil d` → `ASIL-D`).
    pub values: &'static [&'static str],
    pub doc: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceRule {
    pub kind: &'static str,
    pub from: &'static str,
    pub to: &'static str,
    pub doc: &'static str,
}

/// A relationship type of the element graph (`compiler::elements`), as
/// served by the Systems Modeling API.
#[derive(Debug, Clone, Serialize)]
pub struct RelationshipSpec {
    pub name: &'static str,
    pub source: &'static str,
    pub target: &'static str,
    pub doc: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metamodel {
    pub version: &'static str,
    pub kinds: Vec<KindSpec>,
    pub relationships: Vec<RelationshipSpec>,
    pub enums: Vec<EnumSpec>,
    pub trace_rules: Vec<TraceRule>,
    pub units: &'static [UnitSpec],
}

fn attr(key: &'static str, ty: AttrType, doc: &'static str) -> AttrSpec {
    AttrSpec { key, ty, doc }
}

fn list(inner: AttrType) -> AttrType {
    AttrType::List(Box::new(inner))
}

fn common() -> Vec<AttrSpec> {
    vec![
        attr("id", AttrType::Identifier, "Stable identity; drives the UUIDv5. Defaults to the name."),
        attr("name", AttrType::Text, "Display name; overrides the block name."),
        attr("description", AttrType::Text, "Free-text description."),
        attr("is", AttrType::Reference("Type"), "User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited."),
    ]
}

fn multiplicity() -> AttrSpec {
    attr(
        "multiplicity",
        AttrType::Multiplicity,
        "How many of the element its owner has: a count (`4`) or a range as text (`\"0..1\"`, `\"1..*\"`, `\"*\"`). One when absent.",
    )
}

fn safety() -> Vec<AttrSpec> {
    vec![
        attr("safety_level", AttrType::Enum("SafetyLevel"), "ASIL, DAL or SIL assigned to the element."),
        attr("asil", AttrType::Enum("Asil"), "ISO 26262 ASIL (QM, ASIL-A..D)."),
        attr("dal", AttrType::Enum("Dal"), "DO-178C design assurance level (DAL-A..E)."),
    ]
}

fn timing() -> Vec<AttrSpec> {
    vec![
        attr("latency", AttrType::Quantity(Dimension::Time), "Execution latency; summed along functional chains by the gate."),
        attr("execution_time", AttrType::Quantity(Dimension::Time), "Alias of latency."),
        attr("wcet", AttrType::Quantity(Dimension::Time), "Worst-case execution time."),
        attr("period", AttrType::Quantity(Dimension::Time), "Activation period."),
        attr("deadline", AttrType::Quantity(Dimension::Time), "Completion deadline."),
        attr("frequency", AttrType::Quantity(Dimension::Frequency), "Activation frequency."),
    ]
}

fn communication() -> Vec<AttrSpec> {
    vec![
        attr("protocol", AttrType::Text, "Transport protocol (CAN FD, Ethernet, ARINC 429...)."),
        attr("bandwidth", AttrType::Quantity(Dimension::DataRate), "Available or required data rate."),
        attr("latency", AttrType::Quantity(Dimension::Time), "Transport latency."),
        attr("frequency", AttrType::Quantity(Dimension::Frequency), "Message rate."),
        attr("period", AttrType::Quantity(Dimension::Time), "Message period."),
        attr("size", AttrType::Quantity(Dimension::Information), "Payload size."),
        attr("data_type", AttrType::Text, "Exchanged data type."),
    ]
}

fn resources() -> Vec<AttrSpec> {
    vec![
        attr("memory", AttrType::Text, "Memory configuration, descriptive (`\"2 GB DDR4, 64 GB eMMC\"`). Use `ram`/`flash`/`storage` for typed sizes."),
        attr("ram", AttrType::Quantity(Dimension::Information), "RAM capacity."),
        attr("flash", AttrType::Quantity(Dimension::Information), "Flash capacity."),
        attr("storage", AttrType::Quantity(Dimension::Information), "Mass storage capacity."),
        attr("power", AttrType::Quantity(Dimension::Power), "Power consumption."),
        attr("voltage", AttrType::Quantity(Dimension::Voltage), "Supply voltage."),
        attr("mass", AttrType::Quantity(Dimension::Mass), "Mass."),
        attr("weight", AttrType::Quantity(Dimension::Mass), "Alias of mass."),
    ]
}

/// Extend / include / generalization towards capabilities of the same
/// level, for the capability kind `kind`.
fn capability_relations(kind: &'static str) -> Vec<AttrSpec> {
    vec![
        attr("extends", list(AttrType::Reference(kind)), "Capabilities of the same level this one extends."),
        attr("includes", list(AttrType::Reference(kind)), "Capabilities of the same level this one includes."),
        attr("specializes", list(AttrType::Reference(kind)), "Capabilities of the same level this one is a special case of."),
    ]
}

fn concat(parts: Vec<Vec<AttrSpec>>) -> Vec<AttrSpec> {
    parts.into_iter().flatten().collect()
}

fn kind(
    name: &'static str,
    layer: Layer,
    arcadia: &'static str,
    sysml: &'static str,
    doc: &'static str,
    attributes: Vec<AttrSpec>,
) -> KindSpec {
    KindSpec { name, layer, arcadia, sysml, doc, attributes }
}

impl Metamodel {
    /// The metamodel of the current language version.
    pub fn current() -> Metamodel {
        let component_attrs = || {
            concat(vec![
                common(),
                vec![
                    attr("type", AttrType::Text, "Component category (free text)."),
                    attr("layer", AttrType::Text, "Overrides the architectural layer label."),
                ],
                safety(),
                resources(),
            ])
        };
        let function_attrs = || {
            concat(vec![
                common(),
                vec![
                    attr("category", AttrType::Text, "Functional category (free text)."),
                    attr("inputs", list(AttrType::Text), "Input data names."),
                    attr("outputs", list(AttrType::Text), "Output data names."),
                ],
                timing(),
                safety(),
            ])
        };
        let exchange_attrs = || {
            concat(vec![
                common(),
                vec![
                    attr("from", AttrType::Reference("Element"), "Source element or port."),
                    attr("to", AttrType::Reference("Element"), "Target element or port."),
                ],
                communication(),
            ])
        };
        let chain_attrs = || {
            concat(vec![
                common(),
                vec![
                    attr("involves", list(AttrType::Reference("Element")), "Ordered functions/exchanges of the chain."),
                    attr("latency_budget", AttrType::Quantity(Dimension::Time), "End-to-end budget checked by the gate."),
                    attr("capability", AttrType::Reference("Capability"), "Capability this chain exemplifies."),
                ],
            ])
        };

        let kinds = vec![
            // ---- Operational Analysis ----
            kind("Actor", Layer::Operational, "OperationalActor", "part def (actor)", "Human or external system interacting with the system.",
                concat(vec![common(), vec![attr("category", AttrType::Text, "Actor category.")], safety()])),
            kind("OperationalEntity", Layer::Operational, "Entity", "part def", "Organisation or system of the operational world.", component_attrs()),
            kind("OperationalCapability", Layer::Operational, "OperationalCapability", "use case def", "Expected ability of the operational world. May be declared inside another operational capability.",
                concat(vec![common(), vec![
                    attr("involves", list(AttrType::Reference("Element")), "Actors, entities, activities and processes involved."),
                ], capability_relations("OperationalCapability")])),
            kind("OperationalActivity", Layer::Operational, "OperationalActivity", "action def", "Activity performed by an entity or actor.", function_attrs()),
            kind("OperationalExchange", Layer::Operational, "CommunicationMean / OperationalExchange", "connect", "Interaction between operational entities.", exchange_attrs()),
            kind("OperationalProcess", Layer::Operational, "OperationalProcess", "action def (sequence)", "Ordered path of activities fulfilling a capability.", chain_attrs()),
            // ---- System Analysis ----
            kind("Requirement", Layer::System, "Requirement", "requirement", "Stakeholder, system or safety requirement.",
                concat(vec![common(), vec![
                    attr("priority", AttrType::Enum("Priority"), "Business priority."),
                    attr("category", AttrType::Text, "Requirement category (functional, performance, safety...)."),
                    attr("rationale", AttrType::Text, "Why the requirement exists."),
                    attr("source", AttrType::Text, "Origin document or stakeholder."),
                    attr("status", AttrType::Text, "Lifecycle status."),
                    attr("verification_method", AttrType::Enum("VerificationMethod"), "Intended verification method."),
                ], safety()])),
            kind("Mission", Layer::System, "Mission", "use case def", "High-level goal the system contributes to.", common()),
            kind("Capability", Layer::System, "Capability", "use case def", "Expected ability of the system. May be declared inside another capability.",
                concat(vec![common(), vec![
                    attr("mission", AttrType::Reference("Mission"), "Mission this capability contributes to."),
                    attr("realizes", AttrType::Reference("OperationalCapability"), "Operational capability realized."),
                    attr("involves", list(AttrType::Reference("Element")), "Functions, actors and chains involved."),
                ], capability_relations("Capability")])),
            kind("SystemFunction", Layer::System, "SystemFunction", "action def", "Function the system performs.", function_attrs()),
            kind("FunctionPort", Layer::System, "FunctionInputPort / FunctionOutputPort", "in/out item", "Oriented port of a function (strictly in XOR out).",
                vec![attr("data_type", AttrType::Text, "Exchanged data type."), attr("type", AttrType::Enum("PortKind"), "data | control | event.")]),
            kind("FunctionalExchange", Layer::System, "FunctionalExchange", "flow", "Data flow between function ports.", exchange_attrs()),
            kind("SystemActor", Layer::System, "SystemActor", "part def (actor)", "External actor at system level.", concat(vec![common(), safety()])),
            kind("SystemComponent", Layer::System, "SystemComponent", "part def", "The system or one of its external components.", component_attrs()),
            kind("FunctionalChain", Layer::System, "FunctionalChain", "action def (sequence)", "Ordered functions and exchanges realizing one dataflow path.", chain_attrs()),
            // ---- Logical Architecture ----
            kind("LogicalComponent", Layer::Logical, "LogicalComponent", "part def + part", "Behavioural building block, may be nested.",
                concat(vec![component_attrs(), vec![multiplicity()]])),
            kind("LogicalFunction", Layer::Logical, "LogicalFunction", "action def (perform)", "Function allocated to a logical component.", function_attrs()),
            kind("ComponentPort", Layer::Logical, "ComponentPort", "port", "Oriented port of a component (in, out, inout).",
                vec![attr("interface", AttrType::Text, "Interface type carried."), attr("protocol", AttrType::Text, "Protocol.")]),
            kind("LogicalInterface", Layer::Logical, "Interface", "interface def", "Contract between two components.",
                concat(vec![common(), vec![attr("from", AttrType::Reference("LogicalComponent"), "Provider."), attr("to", AttrType::Reference("LogicalComponent"), "Consumer.")], communication()])),
            kind("ComponentExchange", Layer::Logical, "ComponentExchange", "connect", "Exchange between component ports.", exchange_attrs()),
            kind("CapabilityRealization", Layer::Logical, "CapabilityRealization", "use case def", "Realization of a system capability by components. May be declared inside another realization.",
                concat(vec![common(), vec![attr("realizes", AttrType::Reference("Capability"), "Capability realized."), attr("involves", list(AttrType::Reference("Element")), "Components and chains involved.")], capability_relations("CapabilityRealization")])),
            // ---- Physical Architecture ----
            kind("PhysicalNode", Layer::Physical, "PhysicalComponent (NODE)", "part def + part", "Hardware node hosting behaviour components.",
                concat(vec![component_attrs(), vec![multiplicity(), attr("processor", AttrType::Text, "Processor."), attr("cpu", AttrType::Text, "CPU description."), attr("redundancy", AttrType::Text, "Redundancy scheme.")]])),
            kind("PhysicalPort", Layer::Physical, "PhysicalPort", "port", "Unoriented physical connector.", vec![attr("connector", AttrType::Text, "Connector type.")]),
            kind("BehaviorComponent", Layer::Physical, "PhysicalComponent (BEHAVIOR)", "part", "Software/behaviour deployed on a node.", concat(vec![common(), safety()])),
            kind("HardwareComponent", Layer::Physical, "PhysicalComponent (NODE, nested)", "part", "Hardware part of a node.", concat(vec![common(), resources()])),
            kind("PhysicalLink", Layer::Physical, "PhysicalLink", "connect (binding)", "Physical medium between nodes or ports.", exchange_attrs()),
            kind("PhysicalExchange", Layer::Physical, "ComponentExchange (physical)", "flow", "Message routed over a link.",
                concat(vec![exchange_attrs(), vec![attr("via", AttrType::Reference("PhysicalLink"), "Carrying link."), attr("message_type", AttrType::Text, "Message/frame type.")]])),
            kind("Deployment", Layer::Physical, "ComponentDeploymentLink", "allocate", "Allocation of a logical component to a node.", common()),
            kind("PhysicalPath", Layer::Physical, "PhysicalPath", "connect (sequence)", "Ordered links routing an exchange.",
                concat(vec![common(), vec![attr("involves", list(AttrType::Reference("PhysicalLink")), "Ordered links.")]])),
            // ---- EPBS ----
            kind("EpbsSystem", Layer::Epbs, "ConfigurationItem (SYSTEM)", "part def", "Top configuration item.", common()),
            kind("EpbsSubsystem", Layer::Epbs, "ConfigurationItem (CS)", "part def", "Subsystem configuration item.", common()),
            kind("EpbsItem", Layer::Epbs, "ConfigurationItem (HW/SW)", "part def", "Leaf configuration item or assembly.",
                concat(vec![common(), vec![attr("part_number", AttrType::Text, "Part number."), attr("supplier", AttrType::Text, "Supplier.")], resources()])),
            // ---- Data ----
            kind("Class", Layer::Data, "Class", "attribute def", "Structured data element; every attribute but `id` and `description` is a field, kept in declared order.", common()),
            kind("Enumeration", Layer::Data, "Enumeration", "enum def", "Enumerated data type.",
                concat(vec![common(), vec![attr("values", list(AttrType::Text), "Literals.")]])),
            kind("DataType", Layer::Data, "DataType", "attribute def", "Primitive data type.",
                concat(vec![common(), vec![attr("base", AttrType::Text, "Base type."), attr("unit", AttrType::Enum("Unit"), "Unit symbol of the values.")]])),
            kind("ExchangeItem", Layer::Data, "ExchangeItem", "item def", "Set of data elements exchanged together.",
                concat(vec![common(), vec![attr("mechanism", AttrType::Enum("ExchangeMechanism"), "Exchange mechanism."), attr("elements", list(AttrType::Reference("Class")), "Grouped data elements.")]])),
            // ---- Transverse ----
            kind("Hazard", Layer::Transverse, "(safety extension)", "requirement (hazard)", "HARA entry: hazardous event and its classification.",
                concat(vec![common(), vec![
                    attr("severity", AttrType::Enum("HazardSeverity"), "ISO 26262 S0-S3 or DO-178C failure condition."),
                    attr("exposure", AttrType::Enum("Exposure"), "ISO 26262 E0-E4."),
                    attr("controllability", AttrType::Enum("Controllability"), "ISO 26262 C0-C3."),
                    attr("condition", AttrType::Enum("FailureCondition"), "DO-178C failure condition class."),
                    attr("asil", AttrType::Enum("Asil"), "Declared ASIL; must match S/E/C."),
                    attr("dal", AttrType::Enum("Dal"), "Declared DAL; must match the condition."),
                    attr("mitigated_by", list(AttrType::Reference("Requirement")), "Safety requirements mitigating the hazard."),
                ]])),
            kind("FmeaEntry", Layer::Transverse, "(safety extension)", "requirement (failure mode)", "FMEA line: failure mode with S/O/D rating.",
                concat(vec![common(), vec![
                    attr("failure_mode", AttrType::Text, "Failure mode."),
                    attr("effect", AttrType::Text, "Effect."),
                    attr("cause", AttrType::Text, "Cause."),
                    attr("severity", AttrType::Text, "Severity: rating, class (S3) or description — free text by design, FMEA practices differ."),
                    attr("occurrence", AttrType::Text, "Occurrence: rating, class or rate description."),
                    attr("detection", AttrType::Text, "Detection: rating, class or detection means."),
                    attr("rpn", AttrType::Number, "Risk priority number."),
                ]])),
            kind("Trace", Layer::Transverse, "AbstractTrace / Realization", "satisfy / verify / allocate", "Typed link between two elements.",
                vec![attr("from", AttrType::Reference("Element"), "Source."), attr("to", AttrType::Reference("Element"), "Target."), attr("type", AttrType::Enum("TraceKind"), "Link kind."), attr("rationale", AttrType::Text, "Justification.")]),
            kind("TestCase", Layer::Transverse, "(V&V extension)", "verification def", "Verification case covering requirements.",
                concat(vec![common(), vec![
                    attr("verifies", list(AttrType::Reference("Requirement")), "Requirements verified."),
                    attr("method", AttrType::Enum("VerificationMethod"), "Verification method."),
                    attr("procedure", AttrType::Text, "Procedure."),
                    attr("expected", AttrType::Text, "Expected result."),
                ]])),
            kind("Model", Layer::Transverse, "Project / SystemEngineering", "package", "The model itself: root of the element graph, owner of every top-level element.", vec![
                    attr("name", AttrType::Text, "Model name."),
                    attr("version", AttrType::Text, "Model version."),
                    attr("description", AttrType::Text, "Free-text description."),
                ]),
            kind("Type", Layer::Transverse, "(extension — closest Capella notion: REC/RPL)", "abstract part def / action def, specialized with :>", "Reusable definition: `type Name extends Base { ... }`. Declares typed attributes, ports and the attributes instances must provide. Redefinitions must keep the dimension. Types have their own namespace.",
                vec![
                    attr("required", list(AttrType::Text), "Attributes every instance must provide (declared or inherited)."),
                    attr("description", AttrType::Text, "Free-text description (not inherited)."),
                ]),
            kind("Constraint", Layer::Transverse, "Constraint", "assert constraint", "Dimension-checked comparison over typed attributes (`assert: <expression>`). Ill-formed is a compile error; violated is a warning and a gate blocker.", common()),
            kind("StateMachine", Layer::Transverse, "StateMachine", "state def", "Modes and states of an element.",
                concat(vec![common(), vec![attr("initial", AttrType::Reference("State"), "Initial state.")]])),
            kind("State", Layer::Transverse, "State / Mode", "state", "State (undergone) or mode (chosen behaviour).", common()),
            kind("Transition", Layer::Transverse, "StateTransition", "transition", "Transition between states.",
                vec![
                    attr("trigger", AttrType::Text, "Triggering event."),
                    attr("guard", AttrType::Text, "Guard condition."),
                    attr("action", AttrType::Text, "Effect."),
                    attr("timing", AttrType::Quantity(Dimension::Time), "Timing constraint."),
                    attr("priority", AttrType::Number, "Priority among concurrent transitions."),
                ]),
            kind("Scenario", Layer::Transverse, "Scenario", "interaction (sequence)", "Sequence of messages between participants.",
                concat(vec![common(), vec![attr("participants", list(AttrType::Reference("Element")), "Lifelines.")]])),
            kind("Message", Layer::Transverse, "SequenceMessage", "message", "Message between two participants.",
                vec![attr("type", AttrType::Enum("MessageKind"), "sync | async."), attr("timing", AttrType::Quantity(Dimension::Time), "Timing constraint.")]),
        ];

        let enums = vec![
            EnumSpec { name: "Asil", values: &["QM", "ASIL-A", "ASIL-B", "ASIL-C", "ASIL-D"], doc: "ISO 26262 automotive safety integrity level." },
            EnumSpec { name: "Dal", values: &["DAL-A", "DAL-B", "DAL-C", "DAL-D", "DAL-E"], doc: "DO-178C design assurance level." },
            EnumSpec { name: "SafetyLevel", values: &["QM", "ASIL-A", "ASIL-B", "ASIL-C", "ASIL-D", "DAL-A", "DAL-B", "DAL-C", "DAL-D", "DAL-E", "SIL-1", "SIL-2", "SIL-3", "SIL-4"], doc: "Any integrity level: ASIL (ISO 26262), DAL (DO-178C) or SIL (IEC 61508)." },
            EnumSpec { name: "Priority", values: &["Critical", "High", "Medium", "Low"], doc: "Requirement priority." },
            EnumSpec { name: "HazardSeverity", values: &["S0", "S1", "S2", "S3", "Catastrophic", "Hazardous", "Major", "Minor", "No Effect"], doc: "ISO 26262 severity class or DO-178C failure condition." },
            EnumSpec { name: "Exposure", values: &["E0", "E1", "E2", "E3", "E4"], doc: "ISO 26262 exposure class." },
            EnumSpec { name: "Controllability", values: &["C0", "C1", "C2", "C3"], doc: "ISO 26262 controllability class." },
            EnumSpec { name: "FailureCondition", values: &["Catastrophic", "Hazardous", "Major", "Minor", "No Effect"], doc: "DO-178C failure condition classification." },
            EnumSpec { name: "VerificationMethod", values: &["test", "analysis", "inspection", "demonstration"], doc: "Verification method." },
            EnumSpec { name: "ExchangeMechanism", values: &["EVENT", "FLOW", "OPERATION", "DATA", "SHARED_DATA"], doc: "Arcadia exchange item mechanism." },
            EnumSpec { name: "TraceKind", values: &["satisfies", "implements", "validates", "verifies", "realizes", "refines", "allocates"], doc: "Traceability link kinds." },
            EnumSpec { name: "PortKind", values: &["data", "control", "event"], doc: "Function port nature." },
            EnumSpec { name: "MessageKind", values: &["sync", "async"], doc: "Scenario message kind." },
            EnumSpec { name: "Unit", values: &[], doc: "Any symbol of the unit table below." },
        ];

        let trace_rules = vec![
            TraceRule { kind: "satisfies", from: "component, function, node, actor", to: "Requirement", doc: "Architecture element satisfies a requirement. Counted by the gate." },
            TraceRule { kind: "implements", from: "component, node", to: "function, Requirement", doc: "Element implements a function or requirement." },
            TraceRule { kind: "verifies", from: "TestCase", to: "Requirement", doc: "Verification case covers a requirement (also via `verifies:`)." },
            TraceRule { kind: "validates", from: "TestCase, Scenario", to: "Requirement, Capability", doc: "Validation evidence." },
            TraceRule { kind: "realizes", from: "lower-layer element", to: "upper-layer element", doc: "Inter-layer realization: SA→OA, LA→SA, PA→LA, EPBS→PA." },
            TraceRule { kind: "refines", from: "Requirement", to: "Requirement", doc: "Requirement decomposition." },
            TraceRule { kind: "allocates", from: "component, node", to: "function, component", doc: "Allocation (also expressed by nesting and `deployment`)." },
        ];

        let relationship = |name, source, target, doc| RelationshipSpec { name, source, target, doc };
        let relationships = vec![
            relationship("Trace", "element", "element", "A `trace` declaration; `traceKind` is one of the TraceKind values."),
            relationship("Verification", "TestCase", "Requirement", "`verifies:` of a test case."),
            relationship("Mitigation", "Requirement", "Hazard", "`mitigated_by:` of a hazard."),
            relationship("Typing", "element or PhysicalLink", "Type", "`is:` — the element is an instance of the type."),
            relationship("Specialization", "Type", "Type", "`extends` — the source type specializes the target."),
            relationship("Involvement", "chain, path, capability", "element", "`involves:`; `order` is the 1-based position."),
            relationship("Realization", "capability", "capability", "`realizes:` across layers."),
            relationship("CapabilityExtend", "capability", "capability of the same level", "`extends:` of a capability."),
            relationship("CapabilityInclude", "capability", "capability of the same level", "`includes:` of a capability."),
            relationship("CapabilityGeneralization", "capability", "capability of the same level", "`specializes:` of a capability."),
            relationship("Contribution", "Capability", "Mission", "`mission:` of a capability."),
            relationship("Deployment", "LogicalComponent", "PhysicalNode", "`deploys` / `deployment`."),
            relationship("OperationalExchange", "operational entity", "operational entity", "Interaction or communication means."),
            relationship("FunctionalExchange", "FunctionPort or function", "FunctionPort or function", "Data flow between functions."),
            relationship("ComponentExchange", "ComponentPort or component", "ComponentPort or component", "Exchange between components."),
            relationship("LogicalInterface", "LogicalComponent", "LogicalComponent", "Interface contract between two components."),
            relationship("PhysicalLink", "PhysicalNode", "PhysicalNode", "Physical medium; carries typed attributes and may be typed."),
            relationship("PhysicalExchange", "PhysicalNode", "PhysicalNode", "Message routed over a link."),
            relationship("Transition", "State", "State", "State machine transition."),
            relationship("Message", "element", "element", "Scenario message; `order` is the 1-based position."),
        ];

        Metamodel { version: LANGUAGE_VERSION, kinds, relationships, enums, trace_rules, units: UNITS }
    }

    pub fn kind(&self, name: &str) -> Option<&KindSpec> {
        self.kinds.iter().find(|k| k.name == name)
    }

    pub fn relationship(&self, name: &str) -> Option<&RelationshipSpec> {
        self.relationships.iter().find(|r| r.name == name)
    }

    pub fn enumeration(&self, name: &str) -> Option<&EnumSpec> {
        self.enums.iter().find(|e| e.name == name)
    }

    /// Canonical spelling of `raw` in enumeration `name`, if it is a value.
    /// Matching ignores case and the separators `-`, `_` and space.
    pub fn normalize_enum(&self, name: &str, raw: &str) -> Option<&'static str> {
        if name == "Unit" {
            return super::quantity::lookup_unit(raw.trim()).map(|u| u.symbol);
        }
        let wanted = canonical_key(raw);
        self.enumeration(name)?
            .values
            .iter()
            .copied()
            .find(|value| canonical_key(value) == wanted)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("metamodel serializes")
    }

    /// Deterministic Markdown rendering: the content of `spec/METAMODEL.md`.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# ArcLang metamodel\n\n");
        out.push_str(&format!("Language version **{}**. ", self.version));
        out.push_str("GENERATED from `src/compiler/metamodel.rs` by `arclang metamodel --format markdown`; ");
        out.push_str("a golden test fails when this file and the compiler disagree. Do not edit by hand.\n\n");
        out.push_str("The metamodel is independent of Capella: it names the Arcadia metaclass each kind corresponds to, ");
        out.push_str("and the SysML v2 construct the exporter maps it to. Attribute values are typed; the compiler reports ");
        out.push_str("type violations as `metamodel:` warnings and the production gate turns them into blockers.\n\n");

        out.push_str("## Element kinds\n\n");
        let mut current_layer: Option<Layer> = None;
        for kind in &self.kinds {
            if current_layer != Some(kind.layer) {
                current_layer = Some(kind.layer);
                out.push_str(&format!("### {}\n\n", kind.layer.label()));
            }
            out.push_str(&format!("#### {}\n\n", kind.name));
            out.push_str(&format!("{}\n\n", kind.doc));
            out.push_str(&format!("- Arcadia: `{}`\n- SysML v2: `{}`\n\n", kind.arcadia, kind.sysml));
            out.push_str("| Attribute | Type | Meaning |\n|---|---|---|\n");
            for attribute in &kind.attributes {
                out.push_str(&format!("| `{}` | `{}` | {} |\n", attribute.key, attribute.ty.notation(), attribute.doc));
            }
            out.push('\n');
        }

        out.push_str("## Relationships\n\n");
        out.push_str("The element graph served by the Systems Modeling API (`/api/systems-modeling`) is made of the kinds above and of these relationship types. ");
        out.push_str("A relationship whose end does not resolve is reported in the commit, never dropped.\n\n");
        out.push_str("| Relationship | Source | Target | Meaning |\n|---|---|---|---|\n");
        for relationship in &self.relationships {
            out.push_str(&format!("| `{}` | {} | {} | {} |\n", relationship.name, relationship.source, relationship.target, relationship.doc));
        }
        out.push('\n');

        out.push_str("## Enumerations\n\n");
        out.push_str("Values match ignoring case and the separators `-`, `_` and space.\n\n");
        out.push_str("| Enumeration | Values | Meaning |\n|---|---|---|\n");
        for enumeration in &self.enums {
            let values = if enumeration.values.is_empty() {
                "(unit table)".to_string()
            } else {
                enumeration.values.iter().map(|v| format!("`{}`", v)).collect::<Vec<_>>().join(", ")
            };
            out.push_str(&format!("| {} | {} | {} |\n", enumeration.name, values, enumeration.doc));
        }
        out.push('\n');

        out.push_str("## Units\n\n");
        out.push_str("A quantity is `<number> <unit>` (`latency: 25 ms`). Symbols are case-sensitive. ");
        out.push_str("Each dimension converts to its canonical unit for arithmetic.\n\n");
        out.push_str("| Symbol | Dimension | Factor to canonical | SysML v2 (SI) |\n|---|---|---|---|\n");
        for unit in self.units {
            out.push_str(&format!(
                "| `{}` | {} ({}) | {} | `{}` |\n",
                unit.symbol,
                unit.dimension.label(),
                unit.dimension.canonical_unit(),
                unit.factor,
                unit.sysml
            ));
        }
        out.push('\n');

        out.push_str("## Traceability rules\n\n");
        out.push_str("Dangling endpoints are compile errors. Kind mismatches are `metamodel:` warnings.\n\n");
        out.push_str("| Kind | From | To | Meaning |\n|---|---|---|---|\n");
        for rule in &self.trace_rules {
            out.push_str(&format!("| `{}` | {} | {} | {} |\n", rule.kind, rule.from, rule.to, rule.doc));
        }
        out
    }
}

/// Case- and separator-insensitive key used to match enumeration values.
pub fn canonical_key(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, '-' | '_' | ' '))
        .flat_map(char::to_uppercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_normalization_ignores_case_and_separators() {
        let mm = Metamodel::current();
        assert_eq!(mm.normalize_enum("Asil", "ASIL_D"), Some("ASIL-D"));
        assert_eq!(mm.normalize_enum("Asil", "asil d"), Some("ASIL-D"));
        assert_eq!(mm.normalize_enum("Asil", "ASILB"), Some("ASIL-B"));
        assert_eq!(mm.normalize_enum("Asil", "High"), None);
        assert_eq!(mm.normalize_enum("HazardSeverity", "no_effect"), Some("No Effect"));
        assert_eq!(mm.normalize_enum("VerificationMethod", "Test"), Some("test"));
        assert_eq!(mm.normalize_enum("Unit", "ms"), Some("ms"));
        assert_eq!(mm.normalize_enum("Unit", "furlong"), None);
        assert_eq!(mm.normalize_enum("NoSuchEnum", "x"), None);
    }

    #[test]
    fn every_enum_attribute_names_a_declared_enumeration() {
        let mm = Metamodel::current();
        for kind in &mm.kinds {
            for attribute in &kind.attributes {
                let mut ty = &attribute.ty;
                while let AttrType::List(inner) = ty {
                    ty = inner;
                }
                if let AttrType::Enum(name) = ty {
                    assert!(mm.enumeration(name).is_some(), "{}.{} uses undeclared enum {}", kind.name, attribute.key, name);
                }
            }
        }
    }

    #[test]
    fn kind_names_and_attribute_keys_are_unique() {
        let mm = Metamodel::current();
        let mut names: Vec<_> = mm.kinds.iter().map(|k| k.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), mm.kinds.len(), "duplicate kind names");
        for kind in &mm.kinds {
            let mut keys: Vec<_> = kind.attributes.iter().map(|a| a.key).collect();
            keys.sort();
            keys.dedup();
            assert_eq!(keys.len(), kind.attributes.len(), "duplicate attribute in {}", kind.name);
        }
    }

    #[test]
    fn markdown_and_json_are_deterministic() {
        let a = Metamodel::current();
        let b = Metamodel::current();
        assert_eq!(a.to_markdown(), b.to_markdown());
        assert_eq!(a.to_json(), b.to_json());
        assert!(a.to_markdown().contains("#### LogicalComponent"));
        assert!(a.to_json().contains(&format!("\"version\": \"{}\"", LANGUAGE_VERSION)));
    }
}
