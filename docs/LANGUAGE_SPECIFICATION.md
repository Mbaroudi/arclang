# 📋 ArcLang Language Specification v1.0

**Formal specification for the ArcLang Domain-Specific Language**

---

## Table of Contents

1. [Introduction](#introduction)
2. [Lexical Structure](#lexical-structure)
3. [Syntax Specification](#syntax-specification)
4. [Type System](#type-system)
5. [Semantic Rules](#semantic-rules)
6. [Arcadia Levels](#arcadia-levels)
7. [Traceability System](#traceability-system)
8. [Safety Extensions](#safety-extensions)
9. [Formal Grammar (EBNF)](#formal-grammar-ebnf)
10. [Compliance](#compliance)

---

## Introduction

### Purpose

ArcLang is a textual Domain-Specific Language (DSL) for Model-Based Systems Engineering (MBSE) following the Arcadia methodology. This specification defines the complete syntax, semantics, and type system.

### Scope

This specification covers:
- Lexical structure (tokens, keywords, identifiers)
- Syntactic structure (grammar rules)
- Type system and type checking
- Semantic constraints and validation rules
- Arcadia methodology alignment
- Traceability mechanisms
- Safety-critical extensions

### Conformance

A conforming ArcLang implementation must:
1. Accept all valid programs defined in this specification
2. Reject all invalid programs with appropriate error messages
3. Preserve semantic meaning through compilation
4. Generate valid Capella XML output

---

## Lexical Structure

### Character Set

ArcLang uses UTF-8 encoding. All Unicode characters are valid in string literals and comments.

```
SourceCharacter ::= U+0000 to U+10FFFF
```

### Whitespace

Whitespace is used to separate tokens and is otherwise ignored.

```
Whitespace ::= Space | Tab | Newline | CarriageReturn
Space      ::= U+0020
Tab        ::= U+0009
Newline    ::= U+000A
CarriageReturn ::= U+000D
```

### Comments

**Single-line comments:**
```arc
// This is a single-line comment
```

**Multi-line comments:**
```arc
/*
 * This is a multi-line comment
 * It can span multiple lines
 */
```

**Documentation comments:**
```arc
/// Documentation comment for the following element
/// @author: John Doe
/// @safety_critical
requirement "REQ-001" { ... }
```

### Keywords

Reserved keywords that cannot be used as identifiers:

```
operational_analysis    system_analysis        logical_architecture
physical_architecture   epbs                   component
function               actor                   capability
requirement            trace                   satisfies
implements             deploys                 derives_from
refines                mitigates              verifies
hazard                 fmea                    fta
interface              port                    exchange
data                   signal                  node
link                   configuration_item      module
```

### Identifiers

```
Identifier ::= IdentifierStart IdentifierPart*
IdentifierStart ::= Letter | '_'
IdentifierPart ::= Letter | Digit | '_' | '-'
Letter ::= 'a'..'z' | 'A'..'Z'
Digit ::= '0'..'9'
```

**Examples:**
```arc
valid_identifier
ValidIdentifier
valid-identifier-123
_internal_id
```

### Literals

#### String Literals

```
StringLiteral ::= '"' StringCharacter* '"'
StringCharacter ::= EscapeSequence | SourceCharacter - ('"' | '\')
EscapeSequence ::= '\' ('"' | '\' | 'n' | 'r' | 't')
```

**Examples:**
```arc
"Simple string"
"String with \"quotes\""
"Multi\nline\nstring"
"Unicode: 日本語"
```

#### Number Literals

```
NumberLiteral ::= IntegerLiteral | FloatLiteral
IntegerLiteral ::= Digit+
FloatLiteral ::= Digit+ '.' Digit+ Exponent?
Exponent ::= ('e' | 'E') ('+' | '-')? Digit+
```

**Examples:**
```arc
42
3.14159
1.0e-6
2.5E+10
```

#### Boolean Literals

```
BooleanLiteral ::= 'true' | 'false'
```

---

## Syntax Specification

### Program Structure

```
Program ::= TopLevelDeclaration*

TopLevelDeclaration ::=
    | OperationalAnalysis
    | SystemAnalysis
    | LogicalArchitecture
    | PhysicalArchitecture
    | EPBS
    | TraceDeclaration
```

### Operational Analysis (OA)

**Purpose**: Capture stakeholder needs and operational context

```
OperationalAnalysis ::= 'operational_analysis' StringLiteral '{' OAElement* '}'

OAElement ::=
    | ActorDeclaration
    | CapabilityDeclaration
    | OperationalScenario

ActorDeclaration ::= 'actor' StringLiteral '{' ActorAttribute* '}'

ActorAttribute ::=
    | 'id' ':' StringLiteral
    | 'description' ':' StringLiteral
    | 'role' ':' StringLiteral
    | 'external' ':' BooleanLiteral

CapabilityDeclaration ::= 'capability' StringLiteral '{' CapabilityAttribute* '}'

CapabilityAttribute ::=
    | 'id' ':' StringLiteral
    | 'description' ':' StringLiteral
    | 'actors' ':' '[' StringList ']'
    | 'preconditions' ':' StringLiteral
    | 'postconditions' ':' StringLiteral
```

**Example:**
```arc
operational_analysis "Vehicle Operations" {
    actor "Driver" {
        id: "ACT-001"
        description: "Human operator of vehicle"
        role: "Primary user"
        external: true
    }
    
    capability "Drive Safely" {
        id: "CAP-001"
        description: "Operate vehicle safely in traffic"
        actors: ["ACT-001"]
        preconditions: "Vehicle is started and ready"
        postconditions: "Vehicle reaches destination safely"
    }
}
```

### System Analysis (SA)

**Purpose**: Define what the system must do (requirements)

```
SystemAnalysis ::= 'system_analysis' StringLiteral '{' SAElement* '}'

SAElement ::=
    | RequirementDeclaration
    | ModuleDeclaration

RequirementDeclaration ::= 'requirement' StringLiteral '{' RequirementAttribute* '}'

RequirementAttribute ::=
    | 'id' ':' StringLiteral                    // Optional (defaults to name)
    | 'description' ':' StringLiteral           // Required
    | 'priority' ':' Priority
    | 'type' ':' RequirementType
    | 'verification_method' ':' VerificationMethod
    | 'safety_level' ':' SafetyLevel
    | 'dal' ':' DAL
    | 'sil' ':' SIL
    | 'derived_from' ':' '[' StringList ']'
    | 'rationale' ':' StringLiteral
    | 'status' ':' Status

Priority ::= 'Critical' | 'High' | 'Medium' | 'Low'
RequirementType ::= 'Functional' | 'Performance' | 'Safety' | 'Interface' | 'Constraint'
VerificationMethod ::= 'Test' | 'Analysis' | 'Inspection' | 'Demonstration'
SafetyLevel ::= 'ASIL_QM' | 'ASIL_A' | 'ASIL_B' | 'ASIL_C' | 'ASIL_D'
DAL ::= 'DAL_A' | 'DAL_B' | 'DAL_C' | 'DAL_D' | 'DAL_E'
SIL ::= 'SIL_1' | 'SIL_2' | 'SIL_3' | 'SIL_4'
Status ::= 'Draft' | 'Review' | 'Approved' | 'Implemented' | 'Verified'
```

**Example:**
```arc
system_analysis "ACC Requirements" {
    requirement "REQ-ACC-001" {
        description: "System shall maintain safe following distance"
        priority: "Critical"
        type: "Functional"
        verification_method: "Test"
        safety_level: "ASIL_B"
        rationale: "Required for safe autonomous operation"
        status: "Approved"
    }
    
    requirement "REQ-ACC-002" {
        description: "Response time shall be less than 100ms"
        priority: "High"
        type: "Performance"
        derived_from: ["REQ-ACC-001"]
        verification_method: "Test"
    }
}
```

### Logical Architecture (LA)

**Purpose**: Define how the system works (logical components)

```
LogicalArchitecture ::= 'logical_architecture' StringLiteral '{' LAElement* '}'

LAElement ::=
    | ComponentDeclaration
    | InterfaceDeclaration
    | ExchangeDeclaration

ComponentDeclaration ::= 'component' StringLiteral '{' ComponentBody '}'

ComponentBody ::=
    | 'id' ':' StringLiteral
    | 'type' ':' ComponentType
    | 'description' ':' StringLiteral
    | 'safety_level' ':' SafetyLevel
    | FunctionDeclaration*
    | PortDeclaration*
    | ComponentDeclaration*

ComponentType ::= 'Logical' | 'Behavioral' | 'System'

FunctionDeclaration ::= 'function' StringLiteral '{' FunctionAttribute* '}'

FunctionAttribute ::=
    | 'id' ':' StringLiteral
    | 'description' ':' StringLiteral
    | 'inputs' ':' '[' StringList ']'
    | 'outputs' ':' '[' StringList ']'
    | 'preconditions' ':' StringLiteral
    | 'postconditions' ':' StringLiteral
    | 'execution_time' ':' StringLiteral
    | 'memory_usage' ':' StringLiteral

PortDeclaration ::= 'port' StringLiteral '{' PortAttribute* '}'

PortAttribute ::=
    | 'id' ':' StringLiteral
    | 'direction' ':' PortDirection
    | 'type' ':' StringLiteral
    | 'data_type' ':' StringLiteral
    | 'protocol' ':' StringLiteral

PortDirection ::= 'in' | 'out' | 'inout'
```

**Example:**
```arc
logical_architecture "ACC Architecture" {
    component "Distance Controller" {
        id: "LC-001"
        type: "Logical"
        description: "Controls vehicle speed based on distance"
        safety_level: "ASIL_B"
        
        function "Calculate Target Speed" {
            id: "LF-001"
            inputs: ["distance_data", "current_speed", "set_speed"]
            outputs: ["target_speed"]
            execution_time: "10ms"
        }
        
        port "DistanceInput" {
            id: "PORT-IN-001"
            direction: "in"
            type: "Data"
            data_type: "DistanceMeasurement"
        }
        
        port "SpeedOutput" {
            id: "PORT-OUT-001"
            direction: "out"
            type: "Control"
            data_type: "SpeedCommand"
        }
    }
}
```

### Physical Architecture (PA)

**Purpose**: Define where components run (hardware)

```
PhysicalArchitecture ::= 'physical_architecture' StringLiteral '{' PAElement* '}'

PAElement ::=
    | NodeDeclaration
    | LinkDeclaration

NodeDeclaration ::= 'node' StringLiteral '{' NodeAttribute* '}'

NodeAttribute ::=
    | 'id' ':' StringLiteral
    | 'type' ':' NodeType
    | 'processor' ':' StringLiteral
    | 'memory' ':' StringLiteral
    | 'os' ':' StringLiteral
    | 'power' ':' StringLiteral
    | 'deploys' StringLiteral

NodeType ::= 'ECU' | 'Sensor' | 'Actuator' | 'Gateway' | 'Display' | 'Computer'

LinkDeclaration ::= 'link' StringLiteral '{' LinkAttribute* '}'

LinkAttribute ::=
    | 'id' ':' StringLiteral
    | 'type' ':' LinkType
    | 'bandwidth' ':' StringLiteral
    | 'protocol' ':' StringLiteral
    | 'connects' ':' '[' StringList ']'

LinkType ::= 'CAN' | 'Ethernet' | 'FlexRay' | 'LIN' | 'ARINC429' | 'MIL-STD-1553'
```

**Example:**
```arc
physical_architecture "ACC Hardware" {
    node "Main ECU" {
        id: "PN-001"
        type: "ECU"
        processor: "ARM Cortex-M7 @ 400MHz"
        memory: "2MB Flash, 512KB RAM"
        os: "AUTOSAR Classic 4.3"
        power: "12V automotive"
        
        deploys "LC-001"
        deploys "LC-002"
    }
    
    link "CAN Bus" {
        id: "PL-001"
        type: "CAN"
        bandwidth: "500 kbps"
        protocol: "CAN 2.0B"
        connects: ["PN-001", "PN-002"]
    }
}
```

### EPBS (End Product Breakdown Structure)

**Purpose**: Define physical product structure

```
EPBS ::= 'epbs' StringLiteral '{' EPBSElement* '}'

EPBSElement ::= ConfigurationItemDeclaration

ConfigurationItemDeclaration ::= 'configuration_item' StringLiteral '{' CIAttribute* '}'

CIAttribute ::=
    | 'id' ':' StringLiteral
    | 'type' ':' CIType
    | 'part_number' ':' StringLiteral
    | 'supplier' ':' StringLiteral
    | 'version' ':' StringLiteral
    | 'implements' StringLiteral

CIType ::= 'Hardware' | 'Software' | 'System' | 'CSCI' | 'HWCI'
```

**Example:**
```arc
epbs "ACC Product" {
    configuration_item "Front Radar Module" {
        id: "CI-001"
        type: "Hardware"
        part_number: "77GHz-RADAR-001"
        supplier: "Bosch"
        version: "2.5"
        
        implements "PN-002"
    }
}
```

---

## Type System

### Primitive Types

```
PrimitiveType ::=
    | 'int8' | 'int16' | 'int32' | 'int64'
    | 'uint8' | 'uint16' | 'uint32' | 'uint64'
    | 'float32' | 'float64'
    | 'boolean'
    | 'string'
```

### Structured Types

```
StructType ::= 'struct' '{' FieldDeclaration* '}'

FieldDeclaration ::= 'field' StringLiteral '{' FieldAttribute* '}'

FieldAttribute ::=
    | 'type' ':' TypeReference
    | 'unit' ':' StringLiteral
    | 'range' ':' StringLiteral
```

**Example:**
```arc
data "SensorData" {
    type: "struct"
    
    field "timestamp" {
        type: "uint32"
        unit: "milliseconds"
    }
    
    field "value" {
        type: "float32"
        unit: "meters"
        range: "0..300"
    }
    
    field "valid" {
        type: "boolean"
    }
}
```

### Enumerated Types

```
EnumType ::= 'enum' '{' EnumValue (',' EnumValue)* '}'

EnumValue ::= StringLiteral
```

**Example:**
```arc
data "VehicleState" {
    type: "enum"
    values: ["INIT", "READY", "ACTIVE", "FAULT", "SHUTDOWN"]
}
```

### Array Types

```
ArrayType ::= 'array' '[' IntegerLiteral ']' 'of' TypeReference
```

**Example:**
```arc
data "SensorArray" {
    type: "array"
    size: 10
    element_type: "float32"
}
```

---

## Semantic Rules

### Identifier Uniqueness

**Rule**: All identifiers within the same scope must be unique.

```arc
// ✅ Valid
system_analysis "Requirements" {
    requirement "REQ-001" { ... }
    requirement "REQ-002" { ... }
}

// ❌ Invalid: Duplicate ID
system_analysis "Requirements" {
    requirement "REQ-001" { ... }
    requirement "REQ-001" { ... }  // Error: Duplicate identifier
}
```

### Traceability Constraints

**Rule**: Trace sources and targets must reference existing elements.

```arc
// ✅ Valid
requirement "REQ-001" { ... }
component "LC-001" { ... }
trace "LC-001" satisfies "REQ-001" { ... }

// ❌ Invalid: Non-existent target
trace "LC-001" satisfies "REQ-999" { ... }  // Error: REQ-999 not found
```

### Type Compatibility

**Rule**: Port connections must have compatible types.

```arc
// ✅ Valid
component "Sensor" {
    port "Output" {
        direction: "out"
        data_type: "float32"
    }
}

component "Controller" {
    port "Input" {
        direction: "in"
        data_type: "float32"
    }
}

// ❌ Invalid: Type mismatch
component "Display" {
    port "Input" {
        direction: "in"
        data_type: "string"  // Incompatible with float32
    }
}
```

### Safety Level Consistency

**Rule**: Child elements cannot have lower safety level than parent.

```arc
// ✅ Valid
component "Controller" {
    safety_level: "ASIL_C"
    
    function "Process" {
        safety_level: "ASIL_C"  // Same level OK
    }
}

// ❌ Invalid: Lower safety level
component "SafetyController" {
    safety_level: "ASIL_D"
    
    function "CriticalProcess" {
        safety_level: "ASIL_B"  // Error: Cannot be lower than parent
    }
}
```

### Deployment Constraints

**Rule**: A logical component can only be deployed once.

```arc
// ✅ Valid
logical_architecture "LA" {
    component "Controller" { id: "LC-001" }
}

physical_architecture "PA" {
    node "ECU1" {
        deploys "LC-001"
    }
}

// ❌ Invalid: Double deployment
physical_architecture "PA" {
    node "ECU1" { deploys "LC-001" }
    node "ECU2" { deploys "LC-001" }  // Error: Already deployed
}
```

---

## Arcadia Levels

### Level Definitions

| Level | Name | Purpose | Key Elements |
|-------|------|---------|--------------|
| **OA** | Operational Analysis | Stakeholder needs | Actors, Capabilities |
| **SA** | System Analysis | What system does | Requirements |
| **LA** | Logical Architecture | How system works | Components, Functions |
| **PA** | Physical Architecture | Where system runs | Nodes, Links |
| **EPBS** | Product Structure | What is built | Configuration Items |

### Level Transitions

```
OA → SA: Capabilities become Requirements
SA → LA: Requirements satisfied by Components
LA → PA: Components deployed on Nodes
PA → EPBS: Nodes implemented by Configuration Items
```

---

## Traceability System

### Trace Types

```
TraceType ::=
    | 'satisfies'      // Component → Requirement
    | 'implements'     // Function → Component
    | 'deploys'        // Node → Component
    | 'derives_from'   // Requirement → Requirement
    | 'refines'        // Element → Element
    | 'mitigates'      // Requirement → Hazard
    | 'verifies'       // Test → Requirement
```

### Trace Declaration

```
TraceDeclaration ::= 'trace' StringLiteral TraceType StringLiteral '{' TraceAttribute* '}'

TraceAttribute ::=
    | 'rationale' ':' StringLiteral           // Required
    | 'coverage' ':' Coverage
    | 'verification' ':' StringLiteral
    | 'status' ':' Status

Coverage ::= 'Full' | 'Partial' | 'None'
```

**Example:**
```arc
trace "LC-001" satisfies "REQ-001" {
    rationale: "Distance controller implements safe following distance requirement"
    coverage: "Full"
    verification: "Tested in TC-001"
    status: "Approved"
}
```

---

## Safety Extensions

### Hazard Analysis

```
HazardDeclaration ::= 'hazard' StringLiteral '{' HazardAttribute* '}'

HazardAttribute ::=
    | 'description' ':' StringLiteral
    | 'severity' ':' Severity
    | 'exposure' ':' Exposure
    | 'controllability' ':' Controllability
    | 'asil' ':' SafetyLevel
    | 'safety_goal' ':' StringLiteral
    | 'mitigation' ':' '[' StringList ']'

Severity ::= 'S0' | 'S1' | 'S2' | 'S3'
Exposure ::= 'E0' | 'E1' | 'E2' | 'E3' | 'E4'
Controllability ::= 'C0' | 'C1' | 'C2' | 'C3'
```

### FMEA (Failure Mode and Effects Analysis)

```
FMEADeclaration ::= 'fmea' StringLiteral '{' FMEAAttribute* '}'

FMEAAttribute ::=
    | 'component' ':' StringLiteral
    | 'failure_mode' ':' StringLiteral
    | 'effects' ':' StringLiteral
    | 'causes' ':' StringLiteral
    | 'severity' ':' IntegerLiteral         // 1-10
    | 'occurrence' ':' IntegerLiteral       // 1-10
    | 'detection' ':' IntegerLiteral        // 1-10
    | 'rpn' ':' IntegerLiteral              // S × O × D
    | 'actions' ':' '[' StringList ']'
```

---

## Formal Grammar (EBNF)

### Complete Grammar

```ebnf
Program ::= TopLevelDeclaration*

TopLevelDeclaration ::=
    | OperationalAnalysis
    | SystemAnalysis
    | LogicalArchitecture
    | PhysicalArchitecture
    | EPBS
    | TraceDeclaration
    | HazardDeclaration
    | FMEADeclaration

OperationalAnalysis ::= 'operational_analysis' StringLiteral '{' OAElement* '}'
SystemAnalysis ::= 'system_analysis' StringLiteral '{' SAElement* '}'
LogicalArchitecture ::= 'logical_architecture' StringLiteral '{' LAElement* '}'
PhysicalArchitecture ::= 'physical_architecture' StringLiteral '{' PAElement* '}'
EPBS ::= 'epbs' StringLiteral '{' EPBSElement* '}'

OAElement ::= ActorDeclaration | CapabilityDeclaration
SAElement ::= RequirementDeclaration | ModuleDeclaration
LAElement ::= ComponentDeclaration | InterfaceDeclaration
PAElement ::= NodeDeclaration | LinkDeclaration
EPBSElement ::= ConfigurationItemDeclaration

ActorDeclaration ::= 'actor' StringLiteral '{' ActorAttribute* '}'
CapabilityDeclaration ::= 'capability' StringLiteral '{' CapabilityAttribute* '}'
RequirementDeclaration ::= 'requirement' StringLiteral '{' RequirementAttribute* '}'
ComponentDeclaration ::= 'component' StringLiteral '{' ComponentBody '}'
FunctionDeclaration ::= 'function' StringLiteral '{' FunctionAttribute* '}'
NodeDeclaration ::= 'node' StringLiteral '{' NodeAttribute* '}'
TraceDeclaration ::= 'trace' StringLiteral TraceType StringLiteral '{' TraceAttribute* '}'

ComponentBody ::= ComponentAttribute | FunctionDeclaration | PortDeclaration | ComponentDeclaration

Attribute ::= Identifier ':' Value
Value ::= StringLiteral | NumberLiteral | BooleanLiteral | Array
Array ::= '[' ValueList? ']'
ValueList ::= Value (',' Value)*
StringList ::= StringLiteral (',' StringLiteral)*

Identifier ::= [a-zA-Z_][a-zA-Z0-9_-]*
StringLiteral ::= '"' [^"]* '"'
NumberLiteral ::= [0-9]+ ('.' [0-9]+)?
BooleanLiteral ::= 'true' | 'false'
```

---

## Compliance

### ISO 26262 Compliance

ArcLang supports ISO 26262 through:
- ASIL levels (QM, A, B, C, D)
- Hazard analysis (HARA)
- Safety goals and requirements
- Decomposition and allocation
- Verification tracking

### DO-178C Compliance

ArcLang supports DO-178C through:
- DAL levels (A, B, C, D, E)
- Requirements traceability
- Derived requirements
- Verification methods
- Software partitioning

### IEC 61508 Compliance

ArcLang supports IEC 61508 through:
- SIL levels (1, 2, 3, 4)
- Safety instrumented functions
- Failure modes analysis
- Systematic capability

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-10-19 | Initial specification |

---

**Specification Author**: Malek Baroudi & Bilel Laasami  
**Status**: Final  
**Standards Compliance**: ISO 26262, DO-178C, IEC 61508
