# ArcLang Syntax Guide

**Current Version**: 1.0.0  
**Date**: October 20, 2025  
**Status**: ✅ **WORKING SYNTAX DOCUMENTED**

---

## Summary

✅ **Currently Working Syntax** - Use this syntax for all models  
❌ **Not Supported** - Alternative syntaxes that require parser updates

---

## ✅ WORKING SYNTAX

### Model Declaration

```arc
model ModelName {
    // Content here
}
```

**Rules**:
- Use `model` keyword
- Model name is **identifier** (no quotes, no spaces)
- Use PascalCase: `AdaptiveCruiseControl`, `VehicleSystem`

### Metadata Block

```arc
model AdaptiveCruiseControl {
    metadata {
        version: "1.0.0"
        author: "System Architect"
        description: "ACC system for automotive"
        safety_standard: "ISO 26262"
    }
}
```

**Supported attributes**:
- `version`, `author`, `description`
- Any custom attributes with string values

### Requirements

```arc
requirements stakeholder {
    req STK-001 "Requirement Title" {
        description: "Full requirement text"
        priority: Critical
        safety_level: ASIL_B
        rationale: "Why this requirement exists"
    }
}

requirements system {
    req SYS-001 "System Requirement" {
        description: "System-level requirement"
        traces: [STK-001]
        verification: "Test method"
    }
}

requirements safety {
    req SAF-001 "Safety Requirement" {
        description: "Safety-critical requirement"
        safety_level: ASIL_B
    }
}
```

**Rules**:
- Use `requirements` keyword followed by type: `stakeholder`, `system`, or `safety`
- Each requirement uses `req ID "Title" { }`
- ID format: `PREFIX-NNN` (e.g., `STK-001`, `SYS-042`)
- Title in quotes
- Common attributes:
  - `description`: string (required)
  - `priority`: identifier (Critical, High, Medium, Low)
  - `safety_level`: identifier (ASIL_A, ASIL_B, ASIL_C, ASIL_D)
  - `rationale`: string
  - `traces`: array of IDs
  - `verification`: string

### Architecture - Logical

```arc
architecture logical {
    component SensingSubsystem "Forward Sensing Subsystem" {
        description: "Detects and tracks objects"
        safety_level: ASIL_B
        
        provides interface IObjectDetection {
            description: "Object detection data"
            signals: [
                "ObjectDistance: Real (m)",
                "ObjectSpeed: Real (m/s)",
                "DetectionConfidence: Integer (0-100%)"
            ]
        }
        
        requires interface IVehicleSpeed {
            description: "Current vehicle speed"
            signals: ["EgoSpeed: Real (m/s)"]
        }
    }
    
    component RadarSensor "77GHz Radar" {
        description: "Long-range radar sensor"
        safety_level: ASIL_B
        parent: SensingSubsystem
        
        provides interface IRadarData {
            signals: [
                "TargetRange: Real (m)",
                "TargetVelocity: Real (m/s)"
            ]
        }
    }
    
    // Connections
    connect SensingSubsystem.IObjectDetection -> ControllerSubsystem
    connect ControllerSubsystem.IVehicleCommands -> ActuationSubsystem
}
```

**Rules**:
- Use `architecture logical { }`
- Components use `component Name "Display Name" { }`
- **Interfaces**:
  - `provides interface InterfaceName { }` - what component offers
  - `requires interface InterfaceName { }` - what component needs
  - `signals: [...]` - array of signal descriptions
- **Connections**:
  - `connect ComponentA.InterfaceX -> ComponentB`
  - `connect ComponentA -> ComponentB` (implicit interface)
- **Parent-child**: Use `parent: ParentComponentName`

### Architecture - Physical

```arc
architecture physical {
    component RadarECU "Radar Electronic Control Unit" {
        description: "77GHz radar processing unit"
        implements: [RadarSensor]
        properties: {
            "Processor": "Infineon AURIX TC397",
            "PowerConsumption": "8W",
            "OperatingTemp": "-40°C to 85°C"
        }
    }
    
    component ACCMainECU "ACC Main Controller ECU" {
        description: "Main ACC control unit"
        implements: [
            SpeedController,
            DistanceController,
            ControllerSubsystem
        ]
        properties: {
            "Processor": "Renesas RH850",
            "Memory": "4MB Flash, 512KB RAM"
        }
    }
    
    // Physical connections
    connect RadarECU -> ACCMainECU via "CAN Bus (500 kbps)"
}
```

**Rules**:
- Use `architecture physical { }`
- `implements: [LogicalComponents]` - maps to logical architecture
- `properties: { }` - key-value pairs for physical properties
- Connections can use `via "protocol"` for communication method

### Scenarios

```arc
scenarios {
    scenario NormalFollowing "Following Lead Vehicle" {
        description: "ACC maintains safe distance"
        precondition: "ACC active, lead vehicle detected"
        steps: [
            "RadarSensor detects vehicle 80m ahead",
            "DistanceController commands deceleration",
            "Vehicle maintains 2.0s time gap"
        ]
        postcondition: "Safe time gap maintained"
        traces: [SYS-001, SYS-003]
    }
}
```

**Rules**:
- Use `scenarios { }` block
- Each scenario: `scenario ID "Title" { }`
- Attributes:
  - `description`: string
  - `precondition`: string
  - `postcondition`: string
  - `steps`: array of strings
  - `traces`: array of requirement IDs

### Traceability

```arc
traceability {
    // Requirement to requirements
    trace STK-001 -> [SYS-001, SYS-005]
    
    // Requirement to components
    trace SYS-001 -> [SpeedController, ControllerSubsystem]
    
    // Component to component (implementation)
    trace RadarSensor -> [RadarECU]
}
```

**Rules**:
- Use `traceability { }` block
- Format: `trace SourceID -> [TargetID1, TargetID2]`
- Can trace:
  - Requirement → Requirements
  - Requirement → Components
  - Logical → Physical components

### Data Types

**Strings**:
```arc
version: "1.0.0"
description: "System with 99.5% accuracy"
temp_range: "-40°C to 85°C"
```
- Quoted with double quotes
- Can contain: decimals, special chars (°, %, ±, etc.)
- Spaces allowed

**Numbers**:
```arc
speed: 120.5
distance: 1_000_000
deceleration: 0.3
```
- Decimals supported: `3.14`, `0.5`
- Underscores for readability: `1_000_000`
- Negative numbers: `-40`, `-12.5`

**Identifiers**:
```arc
priority: Critical
safety_level: ASIL_B
status: OK
```
- No quotes
- Letters, digits, underscores: `ASIL_B`, `value_123`

**Arrays**:
```arc
signals: ["Signal1", "Signal2", "Signal3"]
traces: [REQ-001, REQ-002]
implements: [Component1, Component2]
```
- Square brackets: `[...]`
- Comma-separated
- Can contain strings or identifiers

**Objects** (for properties):
```arc
properties: {
    "Key1": "Value1",
    "Key2": "Value2"
}
```
- Curly braces: `{ }`
- Key-value pairs
- Keys in quotes, values in quotes

---

## ❌ NOT SUPPORTED (Yet)

### Model with String Name
```arc
❌ model "Adaptive Cruise Control System" { }
✅ model AdaptiveCruiseControl { }
```

### System Keyword
```arc
❌ system "SystemName" { }
✅ model SystemName { }
```

### Requirement (Singular)
```arc
❌ requirement "REQ-001" { }
✅ req REQ-001 "Title" { }
```

### Architecture Without Type
```arc
❌ logical_architecture { }
✅ architecture logical { }
```

### Architecture with Identifier Name
```arc
❌ logical_architecture ACCArchitecture { }
✅ architecture logical { }
```

### Nested Function Blocks
```arc
❌ component "Controller" {
    function "ProcessData" {
        inputs: ["a", "b"]
        outputs: ["c"]
    }
}

✅ component Controller "Main Controller" {
    description: "Processes data"
    // Functions described in attributes or separate
}
```

### Port Blocks
```arc
❌ port "radar_data_in" {
    type: "input"
    data_type: "RadarData"
}

✅ provides interface IRadarData {
    signals: ["data_field: Type"]
}
```

### Flow Blocks
```arc
❌ flow "radar_to_controller" {
    from: "Radar.out"
    to: "Controller.in"
}

✅ connect Radar.IRadarOut -> Controller
```

### Data Flows Section
```arc
❌ data_flows { }
✅ Use connect statements in architecture blocks
```

### Safety Section
```arc
❌ safety {
    hazard "HAZ-001" { }
}

✅ requirements safety {
    req SAF-001 "Safety requirement" { }
}
```

### Validation Section
```arc
❌ validation {
    test_case "TC-001" { }
}

✅ Use scenarios or external test documentation
```

---

## 📝 COMPLETE EXAMPLE

```arc
model AdaptiveCruiseControl {
    metadata {
        version: "1.0.0"
        author: "System Architect"
        description: "ASIL-B compliant ACC system"
        safety_standard: "ISO 26262"
    }

    requirements stakeholder {
        req STK-001 "Adaptive Speed Control" {
            description: "System shall maintain vehicle speed ±2 km/h"
            priority: Critical
            safety_level: ASIL_B
            rationale: "Core ACC functionality"
        }
        
        req STK-002 "Safe Distance Maintenance" {
            description: "System shall maintain safe following distance"
            priority: Critical
            safety_level: ASIL_B
        }
    }

    requirements system {
        req SYS-001 "Target Speed Control" {
            description: "Control vehicle speed to match target ±2 km/h"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-001]
            verification: "Vehicle speed test"
        }
        
        req SYS-002 "Distance Sensing" {
            description: "Detect vehicles up to 200m with 0.5m accuracy"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-002]
            verification: "Radar sensor accuracy test"
        }
    }

    requirements safety {
        req SAF-001 "Sensor Redundancy" {
            description: "Forward sensing uses redundant sensors"
            priority: Critical
            safety_level: ASIL_B
            traces: [SYS-002]
        }
    }

    architecture logical {
        component SensingSubsystem "Forward Sensing Subsystem" {
            description: "Detects and tracks objects"
            safety_level: ASIL_B
            
            provides interface IObjectDetection {
                description: "Provides detected object data"
                signals: [
                    "ObjectDistance: Real (m)",
                    "ObjectRelativeSpeed: Real (m/s)",
                    "DetectionConfidence: Integer (0-100%)",
                    "SensorStatus: Enum {OK, DEGRADED, FAILED}"
                ]
            }
            
            requires interface IVehicleSpeed {
                description: "Current vehicle speed"
                signals: ["EgoSpeed: Real (m/s)"]
            }
        }

        component RadarSensor "77GHz FMCW Radar" {
            description: "Long-range forward radar sensor"
            safety_level: ASIL_B
            parent: SensingSubsystem
            
            provides interface IRadarData {
                signals: [
                    "TargetRange: Real (m)",
                    "TargetVelocity: Real (m/s)",
                    "TargetAzimuth: Real (degrees)"
                ]
            }
        }

        component ControllerSubsystem "ACC Control Subsystem" {
            description: "Main adaptive cruise control logic"
            safety_level: ASIL_B
            
            requires interface IObjectDetection
            
            provides interface IVehicleCommands {
                description: "Actuator commands for speed control"
                signals: [
                    "TargetAcceleration: Real (m/s²)",
                    "ThrottleCommand: Real (0-100%)",
                    "BrakeCommand: Real (0-100%)"
                ]
            }
        }

        component ActuationSubsystem "Vehicle Actuation Interface" {
            description: "Interfaces with powertrain and brake systems"
            safety_level: ASIL_B
            
            requires interface IVehicleCommands
        }

        connect SensingSubsystem.IObjectDetection -> ControllerSubsystem
        connect ControllerSubsystem.IVehicleCommands -> ActuationSubsystem
    }

    architecture physical {
        component RadarECU "Radar Electronic Control Unit" {
            description: "77GHz radar processing unit"
            implements: [RadarSensor]
            properties: {
                "Processor": "Infineon AURIX TC397",
                "PowerConsumption": "8W",
                "OperatingTemp": "-40°C to 85°C",
                "CANBusSpeed": "500 kbps"
            }
        }

        component ACCMainECU "ACC Main Controller ECU" {
            description: "Main ACC control unit"
            implements: [ControllerSubsystem]
            properties: {
                "Processor": "Renesas RH850 F1KM",
                "Memory": "4MB Flash, 512KB RAM",
                "PowerConsumption": "5W",
                "Redundancy": "Dual-core lockstep"
            }
        }

        connect RadarECU -> ACCMainECU via "CAN Bus (500 kbps)"
    }

    scenarios {
        scenario NormalFollowing "Following Lead Vehicle" {
            description: "ACC maintains safe distance behind slower vehicle"
            precondition: "ACC active, lead vehicle detected, ego speed > 30 km/h"
            steps: [
                "RadarSensor detects vehicle 80m ahead at 80 km/h",
                "DistanceController commands 0.15g deceleration",
                "Vehicle decelerates smoothly to maintain 2.0s time gap"
            ]
            postcondition: "Safe 2.0s time gap maintained"
            traces: [SYS-001, SYS-002]
        }
    }

    traceability {
        trace STK-001 -> [SYS-001]
        trace STK-002 -> [SYS-002]
        trace SYS-001 -> [ControllerSubsystem]
        trace SYS-002 -> [RadarSensor, SensingSubsystem]
        trace SAF-001 -> [RadarSensor]
        trace RadarSensor -> [RadarECU]
        trace ControllerSubsystem -> [ACCMainECU]
    }
}
```

---

## 🔧 USAGE

### Validate Syntax
```bash
arclang check model.arc
```

### Compile to Capella XML
```bash
arclang build model.arc -o output.capella.xml
```

### Export to HTML
```bash
arclang export model.arc -o diagram.html -f html
```

### Export to JSON
```bash
arclang export model.arc -o model.json -f json
```

---

## 📚 KEYWORDS REFERENCE

### Top-Level Keywords
- `model` - Model declaration
- `metadata` - Metadata block
- `requirements` - Requirements section
- `architecture` - Architecture section
- `scenarios` - Scenarios section
- `traceability` - Traceability section

### Requirements Keywords
- `req` - Requirement definition
- `stakeholder` - Stakeholder requirements type
- `system` - System requirements type
- `safety` - Safety requirements type

### Architecture Keywords
- `logical` - Logical architecture
- `physical` - Physical architecture
- `component` - Component definition
- `provides` - Provided interface
- `requires` - Required interface
- `interface` - Interface definition
- `connect` - Connection between components
- `via` - Communication protocol
- `parent` - Parent component reference
- `implements` - Physical implements logical
- `properties` - Component properties
- `signals` - Interface signals

### Scenario Keywords
- `scenario` - Scenario definition
- `precondition` - Initial conditions
- `postcondition` - End conditions
- `steps` - Scenario steps
- `traces` - Traced requirements

### Traceability Keywords
- `trace` - Traceability link

### Common Attributes
- `description` - Text description
- `version` - Version string
- `author` - Author name
- `safety_level` - ASIL level
- `priority` - Requirement priority
- `rationale` - Justification
- `verification` - Verification method

---

**Last Updated**: October 20, 2025  
**ArcLang Version**: 1.0.0  
**All syntax tested and verified**: ✅
