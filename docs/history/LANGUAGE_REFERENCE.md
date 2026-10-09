# 📖 ArcLang Language Reference

**Complete syntax guide for ArcLang v1.0.0**

---

## Table of Contents

1. [Introduction](#introduction)
2. [Basic Syntax](#basic-syntax)
3. [Arcadia 5 Levels](#arcadia-5-levels)
4. [Requirements](#requirements)
5. [Components](#components)
6. [Functions](#functions)
7. [Interfaces & Ports](#interfaces--ports)
8. [Traceability](#traceability)
9. [Safety Elements](#safety-elements)
10. [Data Types](#data-types)
11. [Comments](#comments)
12. [Best Practices](#best-practices)

---

## Introduction

ArcLang is a textual Domain-Specific Language (DSL) for Model-Based Systems Engineering (MBSE) following the Arcadia methodology.

### Design Principles
- **Readable**: Clear, English-like syntax
- **Structured**: Follows Arcadia methodology
- **Traceable**: Built-in requirements traceability
- **Safe**: Support for safety standards (ISO 26262, DO-178C)
- **Toolable**: Easy to parse and generate diagrams

---

## Basic Syntax

### File Structure

```arc
// File: model.arc

// 1. System Analysis (Requirements)
system_analysis "..." { ... }

// 2. Logical Architecture (Components)
logical_architecture "..." { ... }

// 3. Physical Architecture (Hardware)
physical_architecture "..." { ... }

// 4. Traceability Links
trace "..." satisfies "..." { ... }
```

### Naming Conventions

```arc
// IDs: Uppercase with hyphens
"REQ-001"
"LC-001"
"LF-001"

// Names: Clear descriptive names
"Adaptive Cruise Control"
"Distance Measurement"
```

### Keywords

Reserved keywords:
```
system_analysis, operational_analysis, logical_architecture,
physical_architecture, epbs, component, function, actor,
requirement, trace, satisfies, implements, deploys, hazard,
fmea, interface, port, data, signal
```

---

## Arcadia 5 Levels

### 1. Operational Analysis (OA)

**Purpose**: Capture stakeholder needs and operational context

```arc
operational_analysis "System Operations" {
    actor "User" {
        id: "ACT-001"
        description: "System operator"
        role: "Primary user"
    }
    
    actor "Maintenance" {
        id: "ACT-002"
        description: "System maintainer"
        role: "Support staff"
    }
    
    operational_capability "Drive Vehicle" {
        id: "OC-001"
        description: "Operate vehicle safely"
        actors: ["ACT-001"]
    }
}
```

**Elements**:
- `actor` - Stakeholders and external entities
- `operational_capability` - What the system must enable
- `operational_scenario` - How capabilities are used

### 2. System Analysis (SA)

**Purpose**: Define what the system must do (requirements)

```arc
system_analysis "System Requirements" {
    requirement "REQ-001" {
        description: "System shall maintain safe distance"
        priority: "Critical"           // Critical, High, Medium, Low
        type: "Functional"              // Functional, Performance, Safety
        verification_method: "Test"     // Test, Analysis, Inspection, Demo
        safety_level: "ASIL_B"         // ISO 26262: ASIL_A/B/C/D
        dal: "DAL_A"                   // DO-178C: DAL_A/B/C/D
    }
    
    requirement "REQ-002" {
        description: "Response time shall be < 100ms"
        priority: "High"
        type: "Performance"
        derived_from: ["REQ-001"]      // Parent requirements
    }
}
```

### 3. Logical Architecture (LA)

**Purpose**: Define how the system works (logical components)

```arc
logical_architecture "System Architecture" {
    component "Sensor Fusion" {
        id: "LC-001"
        type: "Logical"
        description: "Fuses sensor data"
        
        function "Fuse Data" {
            id: "LF-001"
            inputs: ["radar_data", "camera_data"]
            outputs: ["fused_data"]
            description: "Combines multiple sensor inputs"
        }
        
        function "Filter Noise" {
            id: "LF-002"
            inputs: ["fused_data"]
            outputs: ["filtered_data"]
        }
    }
    
    component "Controller" {
        id: "LC-002"
        type: "Logical"
        
        function "Control Speed" {
            id: "LF-003"
            inputs: ["filtered_data", "target_speed"]
            outputs: ["throttle_cmd", "brake_cmd"]
        }
    }
}
```

### 4. Physical Architecture (PA)

**Purpose**: Define where components run (hardware)

```arc
physical_architecture "Hardware Platform" {
    node "Main ECU" {
        id: "PN-001"
        type: "ECU"
        processor: "ARM Cortex-M7 @ 400MHz"
        memory: "2MB Flash, 512KB RAM"
        os: "AUTOSAR Classic 4.3"
        
        deploys "LC-001"               // Deploy logical components
        deploys "LC-002"
    }
    
    node "Sensor ECU" {
        id: "PN-002"
        type: "ECU"
        processor: "ARM Cortex-M4 @ 180MHz"
        memory: "1MB Flash, 256KB RAM"
        
        deploys "LC-003"
    }
    
    link "CAN Bus" {
        id: "PL-001"
        type: "CAN"
        bandwidth: "500 kbps"
        connects: ["PN-001", "PN-002"]
    }
}
```

### 5. EPBS (End Product Breakdown Structure)

**Purpose**: Define physical product structure

```arc
epbs "Product Structure" {
    configuration_item "Control Unit" {
        id: "CI-001"
        type: "Hardware"
        part_number: "ECU-CTRL-001"
        supplier: "Continental"
        
        implements "PN-001"            // Implements physical node
    }
    
    configuration_item "Software Package" {
        id: "CI-002"
        type: "Software"
        version: "1.0.0"
        
        implements "PN-001"
    }
}
```

---

## Requirements

### Basic Requirement

```arc
system_analysis "Requirements" {
    requirement "REQ-001" {
        description: "System shall..."
        priority: "High"
    }
}
```

### Full Requirement Syntax

```arc
requirement "REQ-FULL-001" {
    // Basic information
    description: "Complete requirement description"
    rationale: "Why this requirement exists"
    priority: "Critical"               // Critical, High, Medium, Low
    type: "Functional"                 // Functional, Performance, Safety, Interface
    
    // Verification
    verification_method: "Test"        // Test, Analysis, Inspection, Demonstration
    acceptance_criteria: "Criteria for acceptance"
    
    // Safety (Automotive - ISO 26262)
    safety_level: "ASIL_B"            // ASIL_QM, ASIL_A, ASIL_B, ASIL_C, ASIL_D
    asil: "ASIL_B"                    // Alternative syntax
    
    // Safety (Aerospace - DO-178C)
    dal: "DAL_A"                      // DAL_A, DAL_B, DAL_C, DAL_D, DAL_E
    criticality: "Critical"           // Critical, High, Medium, Low
    
    // Safety (Industrial - IEC 61508)
    sil: "SIL_2"                      // SIL_1, SIL_2, SIL_3, SIL_4
    
    // Traceability
    derived_from: ["REQ-PARENT-001"]  // Parent requirements
    allocated_to: ["LC-001"]          // Implementing components
    
    // Attributes
    status: "Approved"                // Draft, Review, Approved, Implemented
    owner: "John Doe"
    revision: "1.2"
}
```

### Requirement Types

```arc
// Functional Requirement
requirement "REQ-FUNC-001" {
    description: "System shall process input data"
    type: "Functional"
}

// Performance Requirement
requirement "REQ-PERF-001" {
    description: "Processing time shall be < 100ms"
    type: "Performance"
}

// Safety Requirement
requirement "REQ-SAFE-001" {
    description: "System shall detect faults within 50ms"
    type: "Safety"
    safety_level: "ASIL_C"
}

// Interface Requirement
requirement "REQ-INTF-001" {
    description: "System shall use CAN 2.0B protocol"
    type: "Interface"
}
```

---

## Components

### Basic Component

```arc
logical_architecture "Architecture" {
    component "MyComponent" {
        id: "LC-001"
        type: "Logical"
    }
}
```

### Full Component Syntax

```arc
component "CompleteComponent" {
    // Identity
    id: "LC-FULL-001"
    type: "Logical"                   // Logical, Behavioral, Physical
    
    // Description
    description: "Detailed component description"
    purpose: "Why this component exists"
    
    // Classification
    category: "Control"               // Control, Sensor, Actuator, Processing
    criticality: "High"               // High, Medium, Low
    
    // Attributes
    redundancy: "Dual"                // Single, Dual, Triple
    safety_level: "ASIL_B"
    
    // Functions (see Functions section)
    function "Function1" { ... }
    function "Function2" { ... }
    
    // Sub-components
    component "SubComponent" {
        id: "LC-SUB-001"
        type: "Logical"
    }
}
```

### Component Types

```arc
// Logical Component (what it does)
component "LogicalController" {
    id: "LC-001"
    type: "Logical"
}

// Behavioral Component (how it behaves)
component "StateMachine" {
    id: "BC-001"
    type: "Behavioral"
}

// Physical Component (hardware)
component "SensorModule" {
    id: "PC-001"
    type: "Physical"
}
```

---

## Functions

### Basic Function

```arc
function "ProcessData" {
    id: "LF-001"
    inputs: ["raw_data"]
    outputs: ["processed_data"]
}
```

### Full Function Syntax

```arc
function "CompleteFunction" {
    // Identity
    id: "LF-FULL-001"
    
    // Description
    description: "Processes sensor data and applies filtering"
    algorithm: "Kalman filter with 5-state model"
    
    // Interfaces
    inputs: ["sensor_data", "calibration"]
    outputs: ["filtered_data", "status"]
    
    // Behavior
    preconditions: "Sensor must be initialized"
    postconditions: "Data is validated and filtered"
    
    // Performance
    execution_time: "10ms"
    memory_usage: "128KB"
    
    // Safety
    safety_level: "ASIL_B"
    fail_safe_behavior: "Return last valid value"
}
```

### Function Exchanges

```arc
component "Source" {
    id: "LC-001"
    
    function "SendData" {
        id: "LF-001"
        outputs: ["data"]
    }
}

component "Destination" {
    id: "LC-002"
    
    function "ReceiveData" {
        id: "LF-002"
        inputs: ["data"]
    }
}

// Exchange between functions
exchange "DataFlow" {
    from: "LF-001"
    to: "LF-002"
    data: "sensor_readings"
    protocol: "CAN"
}
```

---

## Interfaces & Ports

### Component Interfaces

```arc
component "Controller" {
    id: "LC-001"
    
    // Input ports
    port "SensorInput" {
        id: "PORT-IN-001"
        direction: "in"
        type: "Data"
        data_type: "SensorData"
    }
    
    // Output ports
    port "CommandOutput" {
        id: "PORT-OUT-001"
        direction: "out"
        type: "Control"
        data_type: "ControlCommand"
    }
    
    // Bi-directional ports
    port "Configuration" {
        id: "PORT-INOUT-001"
        direction: "inout"
        type: "Config"
        data_type: "ConfigData"
    }
}
```

### Interface Definitions

```arc
interface "SensorInterface" {
    id: "INTF-001"
    protocol: "CAN 2.0B"
    
    signal "Speed" {
        type: "float32"
        unit: "m/s"
        range: "0..100"
    }
    
    signal "Temperature" {
        type: "int16"
        unit: "celsius"
        range: "-40..125"
    }
}
```

---

## Traceability

### Satisfies (Requirement → Component)

```arc
trace "LC-001" satisfies "REQ-001" {
    rationale: "Controller implements the requirement"
    coverage: "Full"                  // Full, Partial
    verification: "Test case TC-001"
}
```

### Implements (Function → Component)

```arc
trace "LF-001" implements "LC-001" {
    rationale: "Function realizes component behavior"
}
```

### Deploys (Node → Component)

```arc
trace "PN-001" deploys "LC-001" {
    rationale: "ECU hosts the logical component"
}
```

### Derives (Requirement → Requirement)

```arc
trace "REQ-002" derives_from "REQ-001" {
    rationale: "REQ-002 is a refinement of REQ-001"
}
```

### Custom Trace Types

```arc
trace "LC-001" allocated_to "PN-001" {
    rationale: "Component allocated to specific hardware"
}

trace "LF-001" refines "REQ-001" {
    rationale: "Function refines high-level requirement"
}
```

---

## Safety Elements

### Hazards (ISO 26262)

```arc
hazard "HAZ-001" {
    description: "Unintended acceleration"
    
    // HARA (Hazard Analysis and Risk Assessment)
    severity: "S3"                    // S0, S1, S2, S3
    exposure: "E4"                    // E0, E1, E2, E3, E4
    controllability: "C2"             // C0, C1, C2, C3
    
    // Resulting ASIL
    asil: "ASIL_C"                    // ASIL_QM, ASIL_A, ASIL_B, ASIL_C, ASIL_D
    
    // Mitigation
    safety_goal: "Prevent unintended acceleration"
    mitigation: ["REQ-SAFE-001", "REQ-SAFE-002"]
}
```

### FMEA (Failure Mode and Effects Analysis)

```arc
fmea "FMEA-001" {
    component: "LC-001"
    failure_mode: "Stuck sensor output"
    
    // Analysis
    effects: "Incorrect speed reading"
    causes: "Sensor hardware failure, wiring issue"
    
    // Risk Assessment
    severity: 8                       // 1-10
    occurrence: 4                     // 1-10
    detection: 6                      // 1-10
    rpn: 192                          // Risk Priority Number (S×O×D)
    
    // Mitigation
    actions: ["Add sensor redundancy", "Plausibility check"]
    responsible: "Safety Team"
}
```

### FTA (Fault Tree Analysis)

```arc
fault_tree "FTA-001" {
    top_event: "Loss of braking"
    
    gate "OR" {
        id: "G1"
        events: ["E1", "E2"]
    }
    
    basic_event "E1" {
        description: "Brake actuator failure"
        probability: 1e-6              // per hour
    }
    
    basic_event "E2" {
        description: "Controller failure"
        probability: 1e-7
    }
}
```

---

## Data Types

### Primitive Types

```arc
data "Speed" {
    type: "float32"                   // int8, int16, int32, uint8, uint16, 
                                      // uint32, float32, float64, boolean, string
    unit: "m/s"
    range: "0..100"
    resolution: "0.1"
}
```

### Structured Types

```arc
data "SensorData" {
    type: "struct"
    
    field "timestamp" {
        type: "uint32"
        unit: "ms"
    }
    
    field "value" {
        type: "float32"
        unit: "m/s"
    }
    
    field "valid" {
        type: "boolean"
    }
}
```

### Enumerated Types

```arc
data "SystemState" {
    type: "enum"
    values: ["INIT", "RUNNING", "FAULT", "SHUTDOWN"]
}
```

### Arrays

```arc
data "SensorArray" {
    type: "array"
    element_type: "float32"
    size: 10
}
```

---

## Comments

### Single-line Comments

```arc
// This is a single-line comment
requirement "REQ-001" {  // Inline comment
    description: "..."
}
```

### Multi-line Comments

```arc
/*
 * This is a multi-line comment
 * It can span multiple lines
 */
system_analysis "..." { ... }
```

### Documentation Comments

```arc
/// This requirement defines the main safety goal
/// @safety_critical
/// @verified_by: TC-001
requirement "REQ-001" {
    description: "..."
}
```

---

## Best Practices

### 1. Naming Conventions

```arc
// ✅ Good: Clear, descriptive names
requirement "REQ-ACC-DISTANCE" {
    description: "Maintain safe following distance"
}

component "Adaptive Cruise Controller" {
    id: "LC-ACC-CTRL"
}

// ❌ Bad: Cryptic names
requirement "R1" {
    description: "..."
}
```

### 2. ID Conventions

```arc
// Requirements
"REQ-[DOMAIN]-[NUMBER]"        // REQ-ACC-001
"REQ-[SUBSYSTEM]-[TYPE]-[NUM]" // REQ-BRAKE-SAFE-001

// Components
"LC-[SUBSYSTEM]-[NUM]"         // LC-ACC-001 (Logical Component)
"PC-[SUBSYSTEM]-[NUM]"         // PC-SENSOR-001 (Physical Component)

// Functions
"LF-[COMPONENT]-[NUM]"         // LF-ACC-001 (Logical Function)

// Physical Nodes
"PN-[TYPE]-[NUM]"              // PN-ECU-001
```

### 3. Hierarchical Organization

```arc
system_analysis "Vehicle Control System" {
    // Group by subsystem
    requirement "REQ-ACC-001" { ... }
    requirement "REQ-ACC-002" { ... }
    
    requirement "REQ-BRAKE-001" { ... }
    requirement "REQ-BRAKE-002" { ... }
}
```

### 4. Complete Traceability

```arc
// Ensure every requirement is traced
requirement "REQ-001" { ... }

component "LC-001" { ... }

trace "LC-001" satisfies "REQ-001" {
    rationale: "Clear explanation of how requirement is satisfied"
}
```

### 5. Safety Documentation

```arc
// Always document safety-critical elements
requirement "REQ-SAFE-001" {
    description: "Detect brake failure within 50ms"
    safety_level: "ASIL_D"
    verification_method: "Test"
    rationale: "Critical for safe vehicle operation"
}
```

---

## Complete Example

```arc
// ========================================
// Adaptive Cruise Control (ACC) System
// ========================================

// 1. OPERATIONAL ANALYSIS
operational_analysis "ACC Operations" {
    actor "Driver" {
        id: "ACT-DRIVER"
        description: "Vehicle operator"
    }
}

// 2. SYSTEM ANALYSIS (Requirements)
system_analysis "ACC Requirements" {
    requirement "REQ-ACC-001" {
        description: "System shall maintain safe following distance"
        priority: "Critical"
        safety_level: "ASIL_B"
        verification_method: "Test"
    }
    
    requirement "REQ-ACC-002" {
        description: "Response time shall be less than 100ms"
        priority: "High"
        type: "Performance"
        derived_from: ["REQ-ACC-001"]
    }
}

// 3. LOGICAL ARCHITECTURE
logical_architecture "ACC Architecture" {
    component "Radar Sensor" {
        id: "LC-RADAR"
        type: "Logical"
        
        function "Measure Distance" {
            id: "LF-MEASURE"
            outputs: ["distance_data", "relative_speed"]
            execution_time: "10ms"
        }
    }
    
    component "ACC Controller" {
        id: "LC-CTRL"
        type: "Logical"
        safety_level: "ASIL_B"
        
        function "Calculate Target Speed" {
            id: "LF-CALC"
            inputs: ["distance_data", "relative_speed", "set_speed"]
            outputs: ["target_speed"]
            execution_time: "20ms"
        }
        
        function "Control Throttle" {
            id: "LF-THROTTLE"
            inputs: ["target_speed", "current_speed"]
            outputs: ["throttle_command"]
        }
    }
}

// 4. PHYSICAL ARCHITECTURE
physical_architecture "ACC Hardware" {
    node "Front ECU" {
        id: "PN-FRONT-ECU"
        processor: "ARM Cortex-M7 @ 400MHz"
        memory: "2MB Flash, 512KB RAM"
        os: "AUTOSAR"
        
        deploys "LC-RADAR"
        deploys "LC-CTRL"
    }
}

// 5. EPBS (Product Structure)
epbs "ACC Product" {
    configuration_item "Front Radar Unit" {
        id: "CI-RADAR"
        part_number: "77GHz-RADAR-001"
        supplier: "Bosch"
        
        implements "PN-FRONT-ECU"
    }
}

// 6. TRACEABILITY
trace "LC-RADAR" satisfies "REQ-ACC-001" {
    rationale: "Radar provides distance measurement capability"
}

trace "LC-CTRL" satisfies "REQ-ACC-001" {
    rationale: "Controller maintains safe distance"
}

trace "LF-CALC" satisfies "REQ-ACC-002" {
    rationale: "Calculation completes within time constraint"
}

// 7. SAFETY ANALYSIS
hazard "HAZ-ACC-001" {
    description: "Unintended acceleration"
    severity: "S3"
    exposure: "E4"
    controllability: "C2"
    asil: "ASIL_C"
    mitigation: ["REQ-ACC-SAFE-001"]
}
```

---

## Grammar Summary (EBNF)

```ebnf
model ::= (analysis | architecture | trace)*

analysis ::= 
    | system_analysis STRING "{" requirement* "}"
    | operational_analysis STRING "{" actor* "}"

architecture ::=
    | logical_architecture STRING "{" component* "}"
    | physical_architecture STRING "{" node* "}"
    | epbs STRING "{" configuration_item* "}"

requirement ::= "requirement" STRING "{" attribute* "}"

component ::= "component" STRING "{" 
    "id:" STRING
    "type:" STRING
    attribute*
    function*
    component*
"}"

function ::= "function" STRING "{"
    "id:" STRING
    attribute*
"}"

trace ::= "trace" STRING trace_type STRING "{" attribute* "}"

trace_type ::= "satisfies" | "implements" | "deploys" | "derives_from"

attribute ::= IDENTIFIER ":" value

value ::= STRING | NUMBER | BOOLEAN | array

array ::= "[" value ("," value)* "]"
```

---

## Next Steps

- **Quick Start**: [QUICKSTART.md](QUICKSTART.md) - Get started in 5 minutes
- **Safety Standards**: [SAFETY_STANDARDS.md](SAFETY_STANDARDS.md) - ISO 26262, DO-178C
- **Examples**: See `examples/` directory for real-world models

---

**Version**: 1.0.0  
**Last Updated**: 2025-10-18  
**Complete**: Yes  
**Tested**: Yes
