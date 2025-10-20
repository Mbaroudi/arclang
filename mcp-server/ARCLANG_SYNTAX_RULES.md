# ArcLang Syntax Rules for AI Clients

**MANDATORY**: All AI clients MUST follow these exact syntax rules when generating ArcLang models.

---

## ✅ CORRECT SYNTAX

### Model Declaration
```arc
model ModelName {
    // Content - ModelName is IDENTIFIER, no quotes, no spaces
}
```

### Metadata
```arc
metadata {
    version: "1.0.0"
    author: "Name"
    description: "Text"
}
```

### Requirements
```arc
requirements stakeholder {
    req REQ-ID "Title" {
        description: "Requirement text"
        priority: Critical
        safety_level: ASIL_B
    }
}

requirements system {
    req SYS-ID "Title" {
        description: "System requirement"
        traces: [REQ-ID]
    }
}

requirements safety {
    req SAF-ID "Title" {
        description: "Safety requirement"
        safety_level: ASIL_B
    }
}
```

### Architecture - Logical
```arc
architecture logical {
    component ComponentName "Display Name" {
        description: "Component description"
        safety_level: ASIL_B
        
        provides interface IInterfaceName {
            description: "Interface description"
            signals: ["Signal1: Type", "Signal2: Type"]
        }
        
        requires interface IOtherInterface {
            signals: ["Signal3: Type"]
        }
    }
    
    connect ComponentA.IInterface -> ComponentB
}
```

### Architecture - Physical
```arc
architecture physical {
    component ECUName "ECU Display Name" {
        description: "Physical component"
        implements: [LogicalComponent1, LogicalComponent2]
        properties: {
            "Key1": "Value1",
            "Key2": "Value2"
        }
    }
    
    connect ECUA -> ECUB via "CAN Bus (500 kbps)"
}
```

### Scenarios
```arc
scenarios {
    scenario ScenarioID "Scenario Title" {
        description: "What happens"
        precondition: "Initial state"
        steps: [
            "Step 1",
            "Step 2"
        ]
        postcondition: "Final state"
        traces: [REQ-ID1, REQ-ID2]
    }
}
```

### Traceability
```arc
traceability {
    trace SOURCE-ID -> [TARGET-ID1, TARGET-ID2]
    trace REQ-001 -> [Component1, Component2]
    trace LogicalComp -> [PhysicalECU]
}
```

---

## ❌ INCORRECT SYNTAX (DO NOT USE)

### ❌ Model with String
```arc
❌ model "Adaptive Cruise Control" { }
✅ model AdaptiveCruiseControl { }
```

### ❌ System Keyword
```arc
❌ system "SystemName" { }
✅ model SystemName { }
```

### ❌ Requirement (Singular)
```arc
❌ requirement "REQ-001" { }
✅ req REQ-001 "Title" { }
```

### ❌ Logical Architecture Without Type
```arc
❌ logical_architecture { }
❌ logical_architecture ArchName { }
✅ architecture logical { }
```

### ❌ Nested Function Blocks
```arc
❌ component "Name" {
    function "FuncName" {
        inputs: ["a", "b"]
    }
}
✅ component Name "Display" {
    description: "Functions: FuncName processes a, b"
}
```

### ❌ Port Blocks
```arc
❌ port "name" {
    type: "input"
}
✅ provides interface IName {
    signals: ["data: Type"]
}
```

### ❌ Top-Level Blocks Without Model
```arc
❌ requirements { }
❌ logical_architecture { }
✅ model Name {
    requirements stakeholder { }
    architecture logical { }
}
```

---

## 📋 NAMING CONVENTIONS

### Identifiers (No Quotes)
- Model names: `AdaptiveCruiseControl`, `VehicleSystem`
- Component names: `SensingSubsystem`, `RadarSensor`
- Interface names: `IObjectDetection`, `IRadarData`
- Requirement prefixes: `STK`, `SYS`, `SAF`

### Strings (With Quotes)
- Display names: `"Forward Sensing Subsystem"`
- Descriptions: `"Detects vehicles ahead"`
- Requirement titles: `"Distance Regulation"`
- Requirement IDs in req: `req REQ-001 "Title"`

### Values
- Safety levels: `ASIL_A`, `ASIL_B`, `ASIL_C`, `ASIL_D` (no quotes)
- Priorities: `Critical`, `High`, `Medium`, `Low` (no quotes)
- Numbers: `1.0`, `100`, `1_000_000`
- Technical strings: `"±2 km/h"`, `"-40°C to 85°C"`, `"ISO 26262"`

---

## 🔒 MANDATORY RULES

1. **Always use `model` keyword**, never `system`
2. **Model names are identifiers**, not strings
3. **Use `architecture logical`**, not `logical_architecture`
4. **Requirements need subtypes**: `stakeholder`, `system`, or `safety`
5. **Use `req ID "Title"`**, not `requirement "ID"`
6. **Interfaces use `provides`/`requires`**, not `port`
7. **All blocks must be inside `model { }`**
8. **Component names are identifiers**, display names are strings

---

## 📖 COMPLETE EXAMPLE

```arc
model AdaptiveCruiseControl {
    metadata {
        version: "1.0.0"
        author: "System Architect"
        safety_standard: "ISO 26262"
    }

    requirements stakeholder {
        req STK-001 "Speed Control" {
            description: "System shall maintain target speed"
            priority: Critical
            safety_level: ASIL_B
        }
    }

    requirements system {
        req SYS-001 "Speed Accuracy" {
            description: "Control speed within ±2 km/h"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-001]
        }
    }

    architecture logical {
        component SensingSubsystem "Forward Sensing" {
            description: "Detects vehicles ahead"
            safety_level: ASIL_B
            
            provides interface IObjectDetection {
                signals: [
                    "ObjectDistance: Real (m)",
                    "ObjectSpeed: Real (m/s)"
                ]
            }
        }

        component ControllerSubsystem "ACC Controller" {
            description: "Main control logic"
            safety_level: ASIL_B
            
            requires interface IObjectDetection
            
            provides interface IVehicleCommands {
                signals: ["Acceleration: Real (m/s²)"]
            }
        }

        connect SensingSubsystem.IObjectDetection -> ControllerSubsystem
    }

    architecture physical {
        component RadarECU "Radar ECU" {
            description: "77GHz radar processing"
            implements: [SensingSubsystem]
            properties: {
                "Processor": "Infineon AURIX TC397",
                "PowerConsumption": "8W"
            }
        }

        component ACCMainECU "Main ACC ECU" {
            implements: [ControllerSubsystem]
            properties: {
                "Processor": "Renesas RH850",
                "Memory": "4MB Flash"
            }
        }

        connect RadarECU -> ACCMainECU via "CAN Bus (500 kbps)"
    }

    scenarios {
        scenario NormalFollowing "Following Lead Vehicle" {
            description: "Maintain safe distance"
            precondition: "ACC active, vehicle detected"
            steps: [
                "Radar detects vehicle 80m ahead",
                "Controller calculates distance",
                "System maintains 2s gap"
            ]
            postcondition: "Safe gap maintained"
            traces: [SYS-001]
        }
    }

    traceability {
        trace STK-001 -> [SYS-001]
        trace SYS-001 -> [ControllerSubsystem]
        trace SensingSubsystem -> [RadarECU]
    }
}
```

---

## ⚠️ VALIDATION

All generated models will be validated by the ArcLang compiler. **Syntax errors will be rejected**.

To validate: `arclang check model.arc`

**Follow these rules exactly. No exceptions.**
