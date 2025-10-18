# ArcLang Language Specification

This document provides a complete reference for the ArcLang language syntax and semantics.

## Table of Contents

1. [Overview](#overview)
2. [Lexical Structure](#lexical-structure)
3. [Operational Analysis](#operational-analysis)
4. [System Analysis](#system-analysis)
5. [Logical Architecture](#logical-architecture)
6. [Physical Architecture](#physical-architecture)
7. [EPBS](#epbs)
8. [Safety Analysis](#safety-analysis)
9. [Traceability](#traceability)
10. [Type System](#type-system)

## Overview

ArcLang follows the Arcadia methodology's five-level architecture:

- **Operational Analysis (OA)**: Actors, capabilities, activities, exchanges
- **System Analysis (SA)**: Requirements, system functions, capabilities
- **Logical Architecture (LA)**: Logical components, functions, interfaces
- **Physical Architecture (PA)**: Physical nodes, deployment, communication
- **EPBS**: End products, subsystems, hardware/software items

## Lexical Structure

### Comments

```arclang
// Single-line comment

/* 
   Multi-line comment
   spanning multiple lines
*/
```

### Identifiers

Identifiers must start with a letter or underscore, followed by letters, digits, or underscores:

```arclang
valid_identifier
ValidIdentifier123
_private
```

### Keywords

Reserved keywords:
```
operational_analysis, actor, operational_capability, operational_activity, operational_exchange
system_analysis, requirement, system_function, system_capability, system_component
logical_architecture, component, function, interface, exchange
physical_architecture, node, physical_link, deploys
epbs, system, subsystem, item
safety_analysis, hazard, fmea, fta, standard, dal, asil, sil
trace, traces, satisfies, implements, refines, derives_from, verifies, allocates
from, to, type, description, priority, safety_level, inputs, outputs, behavior
Critical, High, Medium, Low
ASIL_A, ASIL_B, ASIL_C, ASIL_D
DAL_A, DAL_B, DAL_C, DAL_D, DAL_E
SIL_1, SIL_2, SIL_3, SIL_4
```

### Literals

```arclang
// String literals
"Hello, World!"
"Multi-line\nstring"

// Numeric literals
42
3.14
1.5e-10

// Boolean literals
true
false
```

## Operational Analysis

### Actor Definition

```arclang
operational_analysis "System Name" {
    actor "Actor Name" {
        id: "ACT-001"
        description: "Actor description"
        classification: "PUBLIC" | "SECRET" | "TOP_SECRET"
        interactions: ["interaction1", "interaction2"]
    }
}
```

### Operational Capability

```arclang
operational_capability "Capability Name" {
    id: "OC-001"
    description: "Capability description"
    involving: ["Actor1", "Actor2"]
    phases: ["phase1", "phase2"]
    scenarios: ["scenario1", "scenario2"]
}
```

### Operational Activity

```arclang
operational_activity "Activity Name" {
    id: "OA-001"
    description: "Activity description"
    performed_by: "Actor Name"
    frequency: "Continuous" | "Periodic" | "Event-driven"
    safety_critical: true | false
    trigger: "event_name"
}
```

### Operational Exchange

```arclang
operational_exchange "Exchange Name" from "Source" to "Target" {
    id: "OE-001"
    data: ["data1", "data2"]
    protocol: "Protocol Name"
    rate: "10Hz"
}
```

## System Analysis

### Requirement Definition

```arclang
system_analysis "System Name" {
    requirement "REQ-ID" {
        description: "Requirement text"
        priority: Critical | High | Medium | Low
        safety_level: ASIL_A | ASIL_B | ASIL_C | ASIL_D | 
                     DAL_A | DAL_B | DAL_C | DAL_D | DAL_E |
                     SIL_1 | SIL_2 | SIL_3 | SIL_4
        traces: ["OC-001", "OA-001"]
        verification_method: "Test" | "Analysis" | "Inspection" | "Demonstration"
        rationale: "Justification text"
        standard: "ISO 26262" | "DO-178C" | "IEC 61508"
        derived_from: ["parent_req1", "parent_req2"]
        allocated_to: "Component Name"
    }
}
```

### System Function

```arclang
system_function "Function Name" {
    id: "SF-001"
    description: "Function description"
    inputs: ["input1", "input2"]
    outputs: ["output1", "output2"]
    safety_level: ASIL_B
    execution_time: "10ms"
    wcet: "15ms"  // Worst-case execution time
}
```

### System Capability

```arclang
system_capability "Capability Name" {
    id: "SC-001"
    description: "Capability description"
    satisfies: ["OC-001"]
    includes: ["sub_capability1", "sub_capability2"]
}
```

### System Component

```arclang
system_component "Component Name" {
    id: "SC-101"
    description: "Component description"
    implements: ["SF-001", "SF-002"]
    safety_level: ASIL_C
    redundancy: "Single" | "Dual" | "Triple"
}
```

## Logical Architecture

### Component Definition

```arclang
logical_architecture "Architecture Name" {
    component "Component Name" {
        id: "LC-001"
        type: Logical | Behavioral | Node
        description: "Component description"
        safety_level: DAL_A
        
        function "Function Name" {
            id: "LF-001"
            inputs: ["param1: Type1", "param2: Type2"]
            outputs: ["result: ResultType"]
            behavior: {
                // Pseudo-code or algorithm description
                validate_input(param1)
                result = process(param1, param2)
                return result
            }
            wcet: "5ms"
            implements: "SF-001"
            satisfies: "SYS-REQ-001"
        }
        
        ports: [
            {
                id: "P-001"
                name: "InputPort"
                direction: In | Out | InOut
                data_type: "DataType"
            }
        ]
    }
}
```

### Interface Definition

```arclang
interface "Interface Name" {
    id: "LI-001"
    from: "Component1"
    to: "Component2"
    type: Data | Control | Physical
    protocol: "CAN" | "Ethernet" | "Custom"
    
    exchange "Exchange Name" {
        message_id: "0x200"
        rate: "20Hz"
        latency: "max 10ms"
        data: ["field1", "field2"]
    }
}
```

### Trace Definitions

```arclang
trace "LC-001" satisfies "SYS-REQ-001" {
    rationale: "Component implements the requirement"
}

trace "LF-001" implements "SF-001" {
    rationale: "Function realizes system function"
}

trace "LC-001" allocates "SF-002" {
    rationale: "Function allocated to this component"
}
```

## Physical Architecture

### Node Definition

```arclang
physical_architecture "Physical Architecture Name" {
    node "Node Name" {
        id: "PN-001"
        description: "Hardware node description"
        processor: "ARM Cortex-A53"
        cores: 4
        memory: "8GB RAM"
        storage: "128GB SSD"
        safety_level: ASIL_D
        supplier: "Company Name"
        part_number: "PN-12345"
        
        deploys "LC-001" {
            partition: "Partition Name"
            criticality: "ASIL_D"
            memory_protection: "MMU-enforced"
            isolation: "Strong spatial and temporal"
            mode: "Active" | "Hot Standby" | "Cold Standby"
        }
    }
}
```

### Physical Link

```arclang
physical_link "Link Name" {
    id: "PL-001"
    topology: "Point-to-Point" | "Bus" | "Star" | "Ring"
    protocol: "CAN FD" | "Ethernet" | "FlexRay"
    bandwidth: "1Gbps"
    baudrate: "500kbps"
    redundancy: "Single" | "Dual" | "Triple"
    connects: ["PN-001", "PN-002", "PN-003"]
    
    realizes "LI-001"  // Maps to logical interface
}
```

## EPBS

### End Product Breakdown Structure

```arclang
epbs "Product Name" {
    system "System Name" {
        id: "EPBS-001"
        classification: "PUBLIC" | "SECRET"
        
        subsystem "Subsystem Name" {
            id: "EPBS-101"
            
            item "Item Name" {
                id: "EPBS-1001"
                part_number: "PN-12345-Rev-A"
                supplier: "Supplier Name"
                implements: "PN-001"
                certification: "DO-254 Level A"
                unit_cost: "$1,000"
                quantity: 2
                lead_time: "12 weeks"
                
                // For software items
                version: "v1.2.3"
                sloc: 50000
                language: "C++" | "Ada" | "Rust"
                
                // For hardware items
                asil: "ASIL_B"
                dal: "DAL_A"
                tempest_certified: true
            }
        }
    }
}
```

## Safety Analysis

### Safety Analysis Block

```arclang
safety_analysis {
    standard: ISO_26262 | DO_178C | IEC_61508
    asil: ASIL_A | ASIL_B | ASIL_C | ASIL_D
    dal: DAL_A | DAL_B | DAL_C | DAL_D | DAL_E
    sil: SIL_1 | SIL_2 | SIL_3 | SIL_4
    classification: "SECRET"
    
    hazard "Hazard Name" {
        id: "HAZ-001"
        description: "Hazard description"
        severity: Catastrophic | Hazardous | Major | Minor | NoEffect
        likelihood: Frequent | Probable | Remote | ExtremelyRemote | ExtremelyImprobable
        
        // ISO 26262 specific
        severity: S0 | S1 | S2 | S3
        exposure: E0 | E1 | E2 | E3 | E4
        controllability: C0 | C1 | C2 | C3
        asil: ASIL_A | ASIL_B | ASIL_C | ASIL_D
        
        causes: [
            "Cause 1",
            "Cause 2"
        ]
        
        mitigations: [
            "Mitigation 1",
            "Mitigation 2"
        ]
    }
    
    fmea "FMEA Name" {
        component: "Component Name"
        failure_mode: "Failure description"
        effects: "Effect description"
        severity: Catastrophic | Hazardous | Major | Minor
        occurrence: Frequent | Probable | Occasional | Remote | Improbable
        detection: "Detection method"
        rpn: 48  // Risk Priority Number
        actions: [
            "Corrective action 1",
            "Corrective action 2"
        ]
    }
    
    fta "Fault Tree Name" {
        top_event: "Top level failure"
        gates: [
            AND {
                "Event 1"
                "Event 2" OR {
                    "Subevent A"
                    "Subevent B"
                }
            }
        ]
        minimal_cut_sets: 3
        probability: 1.2e-8
    }
    
    security_analysis {
        threat: "Threat description"
        attack_vector: "Attack method"
        likelihood: "High" | "Medium" | "Low"
        impact: "Catastrophic" | "Major" | "Minor"
        countermeasures: [
            "Countermeasure 1",
            "Countermeasure 2"
        ]
        certification: "Common Criteria EAL 6+"
    }
}
```

## Traceability

### Trace Types

```arclang
// Requirement satisfaction
trace "Component" satisfies "Requirement" {
    rationale: "Explanation"
}

// Functional implementation
trace "LogicalFunction" implements "SystemFunction" {
    rationale: "Explanation"
}

// Refinement relationship
trace "DetailedRequirement" refines "HighLevelRequirement" {
    rationale: "Explanation"
}

// Derivation
trace "DerivedReq" derives_from "ParentReq" {
    rationale: "Explanation"
}

// Verification
trace "TestCase" verifies "Requirement" {
    rationale: "Explanation"
}

// Allocation
trace "Component" allocates "Function" {
    rationale: "Explanation"
}
```

### Inline Traces

```arclang
requirement "REQ-001" {
    description: "System requirement"
    traces: ["OC-001", "OA-002"]  // Traces to operational level
    allocated_to: "Component1"
}

component "Component1" {
    function "Function1" {
        implements: "SF-001"  // Implements system function
        satisfies: "REQ-001"  // Satisfies requirement
    }
}
```

## Type System

### Built-in Types

```arclang
// Primitive types
Boolean
Integer
Float
String

// Structured types
struct Point {
    x: Float
    y: Float
}

enum Status {
    Active,
    Inactive,
    Error
}

// Array types
Integer[]
String[10]

// Optional types
Optional<Integer>
```

### Safety-Critical Types

```arclang
SafetyLevel = ASIL_A | ASIL_B | ASIL_C | ASIL_D | 
              DAL_A | DAL_B | DAL_C | DAL_D | DAL_E |
              SIL_1 | SIL_2 | SIL_3 | SIL_4

Priority = Critical | High | Medium | Low

Severity = Catastrophic | Hazardous | Major | Minor | NoEffect
```

## Attributes

### Common Attributes

All elements support these common attributes:

```arclang
element "Name" {
    id: "UNIQUE-ID"
    description: "Description text"
    version: "1.0.0"
    author: "Author Name"
    created: "2024-01-01"
    modified: "2024-01-15"
    status: "Draft" | "UnderReview" | "Approved" | "Obsolete"
    classification: "PUBLIC" | "CONFIDENTIAL" | "SECRET" | "TOP_SECRET"
    tags: ["tag1", "tag2"]
}
```

## Modules and Imports

```arclang
// Import from another file
import "common/types.arc"
import "requirements/system_requirements.arc"

// Namespace declaration
namespace Vehicle.Powertrain {
    component "Engine" {
        // ...
    }
}

// Using namespaced elements
trace "Vehicle.Powertrain.Engine" satisfies "SYS-REQ-001"
```

## Best Practices

1. **Use meaningful IDs**: Follow a consistent naming scheme (e.g., `REQ-SYS-001`, `LC-CTRL-001`)
2. **Document rationale**: Always provide rationale for traces and safety decisions
3. **Maintain traceability**: Ensure every requirement traces to operational needs and implementation
4. **Version control**: Use Git for version control and semantic merge
5. **Incremental compilation**: Enable incremental compilation for large projects
6. **Safety annotations**: Always specify safety levels for critical elements
7. **Modular design**: Split large models into multiple files using imports

## Grammar Summary

```ebnf
model ::= level_block*

level_block ::= operational_analysis
              | system_analysis
              | logical_architecture
              | physical_architecture
              | epbs
              | safety_analysis

operational_analysis ::= "operational_analysis" string_literal "{" oa_element* "}"

oa_element ::= actor | operational_capability | operational_activity | operational_exchange

actor ::= "actor" string_literal "{" attribute* "}"

requirement ::= "requirement" string_literal "{" attribute* "}"

component ::= "component" string_literal "{" (attribute | function | port)* "}"

function ::= "function" string_literal "{" attribute* "}"

trace ::= "trace" string_literal trace_type string_literal "{" attribute* "}"

trace_type ::= "satisfies" | "implements" | "refines" | "derives_from" | "verifies" | "allocates"

attribute ::= identifier ":" value

value ::= string_literal | numeric_literal | boolean_literal | identifier | array
```

## Examples

See the [Examples](examples.md) section for complete, real-world examples of ArcLang models.
