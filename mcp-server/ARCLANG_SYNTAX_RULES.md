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

### Architecture - Operational (Operational Analysis)
```arc
operational_analysis "Context Title" {
    actor "Actor Name" {
        id: "OA-ACT-001"
        description: "Actor description"
        category: "Human"  // or "System", "External"
        safety_level: ASIL_C  // Optional - for safety-critical actors
    }
    
    operational_activity "Activity Name" {
        id: "OA-01"
        description: "Activity description"
        performed_by: "OA-ACT-001"
        safety_level: ASIL_D  // Optional - for safety-critical activities
    }
    
    operational_interaction "Interaction Name" {
        id: "OI-01"
        from: "OA-ACT-001"
        to: "OA-ACT-002"
        exchange_item_kind: EVENT  // EVENT, FLOW, OPERATION, DATA, SHARED_DATA
        description: "Interaction description"
    }
}
```

**Note**: For MBSE Capella compliance, actors and activities can have `safety_level` attributes (ASIL_D, ASIL_C, ASIL_B, ASIL_A, QM) which will render as colored borders in operational diagrams.

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

### ❌ Operational Architecture Errors
```arc
❌ architecture operational { actor Driver { } }  // Wrong! Use operational_analysis
❌ operational_analysis {  // Missing title string
    actor Driver { }
}
❌ actor Driver {  // Actor cannot be at model level, must be in operational_analysis
    description: "Driver"
}
❌ entity Driver {  // Wrong! Use "actor", not "entity"
    id: "OA-001"
}

✅ operational_analysis "Context Title" {
    actor "Driver" {
        id: "OA-ACT-001"
        description: "Human operator"
        category: "Human"
        safety_level: ASIL_B  // Optional for safety-critical actors
    }
    
    operational_activity "Monitor Road" {
        id: "OA-01"
        description: "Driver monitors traffic"
        performed_by: "OA-ACT-001"
        safety_level: ASIL_C  // Optional
    }
    
    operational_interaction "Visual Feedback" {
        id: "OI-01"
        from: "OA-ACT-001"
        to: "OA-ACT-002"
        exchange_item_kind: EVENT  // EVENT, FLOW, OPERATION, DATA, SHARED_DATA
    }
}
```

**Critical**: Operational architectures MUST use:
- `operational_analysis "Title"` (not `architecture operational`)
- `actor "Name"` inside operational_analysis (with quotes)
- `operational_activity "Name"` (with quotes)
- `operational_interaction "Name"` with `exchange_item_kind`

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

---

## 🎨 MBSE CAPELLA FEATURES (MANDATORY FOR PROFESSIONAL DIAGRAMS)

### Safety Colors (ISO 26262, DO-178C, IEC 61508)

**CRITICAL**: All safety-critical components MUST include safety level metadata for proper visualization.

#### Automotive (ISO 26262)
```arc
component BrakeController "Brake Controller" {
    description: "Emergency brake control"
    safety_level: ASIL_D  // Red 6px border (most critical)
    // Or: ASIL_C (Orange 5px), ASIL_B (Orange 4px), ASIL_A (Yellow 3px), QM (Gray 2px)
}
```

#### Aerospace (DO-178C)
```arc
component FlightControl "Flight Control Computer" {
    description: "Primary flight control"
    dal: DAL_A  // Red 6px border (most critical)
    // Or: DAL_B (Orange 5px), DAL_C (Orange 4px), DAL_D (Yellow 3px), DAL_E (Gray 2px)
}
```

#### Industrial (IEC 61508)
```arc
component SafetyPLC "Safety PLC" {
    description: "Emergency shutdown controller"
    sil: SIL_4  // Red 6px border (highest)
    // Or: SIL_3 (Orange 5px), SIL_2 (Orange 4px), SIL_1 (Yellow 3px), SIL_0 (Gray 2px)
}
```

**Applies to**: Components (LA), Physical Nodes (PA), Operational Activities (OA), Process Diagrams (OPD), Breakdown Trees (EPBS), Sequence Messages, State Machines

---

### Exchange Item Types (Functional Exchanges)

**CRITICAL**: All functional exchanges MUST specify exchange_item_kind for semantic clarity.

```arc
architecture functional {
    function DetectCollision {
        id: SF-001
        ports_out: ["riskAlert"]
    }
    
    function PlanBraking {
        id: SF-002
        ports_in: ["riskData"]
    }
    
    // EVENT - Signal with no data (red dashed)
    exchange AlertExchange {
        from: SF-001.riskAlert
        to: SF-002.riskData
        exchange_item_kind: EVENT  // ⚡ Red dashed line
        label: "CollisionAlert"
    }
    
    // FLOW - Continuous stream (cyan thick)
    exchange SensorStream {
        from: SF-001.sensorOut
        to: SF-002.sensorIn
        exchange_item_kind: FLOW  // ⟿ Cyan thick line
        label: "ContinuousData"
    }
    
    // OPERATION - Request/response (teal double)
    exchange ServiceCall {
        from: SF-001.serviceOut
        to: SF-002.serviceIn
        exchange_item_kind: OPERATION  // ↔ Teal double arrow
        label: "Calculate"
    }
    
    // DATA - Structured package (blue solid, DEFAULT)
    exchange DataPacket {
        from: SF-001.dataOut
        to: SF-002.dataIn
        exchange_item_kind: DATA  // 📦 Blue solid line
        label: "Telemetry"
    }
    
    // SHARED_DATA - Repository access (purple dotted)
    exchange ConfigAccess {
        from: SF-001.configOut
        to: SF-002.configIn
        exchange_item_kind: SHARED_DATA  // 🗄 Purple dotted
        label: "ConfigDB"
    }
    
    // UNSET - Generic/unknown (gray thin)
    exchange GenericExchange {
        from: SF-001.out
        to: SF-002.in
        exchange_item_kind: UNSET  // → Gray thin
        label: "Unknown"
    }
}
```

**Applies to**: Functional Exchanges (SA), Component Exchanges (LA), Dataflow Diagrams

---

### Interface Notation (UML/SysML Ball-and-Socket)

**CRITICAL**: Use provides/requires for proper UML/SysML interface notation.

```arc
architecture logical {
    component DataProducer "Data Producer" {
        description: "Produces sensor data"
        
        // Provided interfaces (lollipops - white circles on RIGHT side)
        provides interface IData {
            signals: ["rawData: ByteArray", "timestamp: Time"]
        }
        
        provides interface IStatus {
            signals: ["status: Enum", "health: Boolean"]
        }
    }
    
    component DataConsumer "Data Consumer" {
        description: "Consumes sensor data"
        
        // Required interfaces (sockets - semicircles on LEFT side)
        requires interface IData
        requires interface IConfig {
            signals: ["mode: Enum", "threshold: Real"]
        }
    }
    
    // Connection creates ball-and-socket link
    connect DataProducer.IData -> DataConsumer
}
```

**Visual Result**: 
- DataProducer: 2 lollipops (○) on right side
- DataConsumer: 2 sockets (⌒) on left side

**Applies to**: Component Architecture (LA) only

---

### Traceability Links (Cross-Layer)

**CRITICAL**: Use explicit traceability for proper architectural consistency.

#### Allocation Links (Function → Component)
```arc
architecture logical {
    component Controller "ACC Controller" {
        description: "Main control logic"
        safety_level: ASIL_D
        allocated_functions: [SF-001, SF-002]  // Orange solid allocation lines
    }
}
```

#### Realization Links (Mission → Capability)
```arc
missions {
    mission AutoBraking "Automatic Emergency Braking" {
        capabilities: [CAP-001]  // Gray-blue dashed realization lines
    }
}

capabilities {
    capability EmergencyStop {
        id: CAP-001
        realized_capability_id: CAP-001
    }
}
```

**Future Link Types** (implemented, not yet integrated):
- `implements`: Component → Requirement (blue dashed)
- `satisfies`: Architecture → Requirement (green dashed)
- `derives`: Requirement → Requirement (brown dashed)
- `justifies`: Decision → Requirement (pink dashed)
- `verifies`: Test → Requirement (cyan dashed)
- `refines`: SA→LA, LA→PA (purple dotted)
- `traces`: Generic traceability (gray dashed)

**Applies to**: Allocation Diagrams, Missions-Capabilities Diagrams

---

## 📊 COMPLETE MBSE EXAMPLE WITH ALL FEATURES

```arc
model EmergencyBrakingSystem {
    metadata {
        version: "2.0.0"
        author: "Safety Architect"
        safety_standard: "ISO 26262 ASIL-D"
    }

    requirements stakeholder {
        req STK-001 "Emergency Braking" {
            description: "System shall perform emergency braking"
            priority: Critical
            safety_level: ASIL_D
        }
    }

    requirements system {
        req SYS-001 "Collision Detection" {
            description: "Detect imminent collision within 100ms"
            priority: Critical
            safety_level: ASIL_D
            traces: [STK-001]
        }
    }

    // OPERATIONAL ANALYSIS - Safety-critical activities and actors
    operational_analysis "Emergency Braking Operational Context" {
        actor "Driver" {
            id: "OA-ACT-001"
            description: "Human operator controlling vehicle"
            category: "Human"
        }
        
        actor "Vehicle System" {
            id: "OA-ACT-002"
            description: "Emergency braking controller"
            category: "System"
            safety_level: ASIL_D  // Red border on safety-critical system actor
        }
        
        actor "Environment" {
            id: "OA-ACT-003"
            description: "Road obstacles and hazards"
            category: "External"
        }
        
        operational_activity "Monitor Environment" {
            id: "OA-01"
            description: "Continuously scan for collision risks"
            performed_by: "OA-ACT-002"
            safety_level: ASIL_C  // Orange border on activity
        }
        
        operational_activity "Execute Emergency Braking" {
            id: "OA-02"
            description: "Apply maximum braking force"
            performed_by: "OA-ACT-002"
            safety_level: ASIL_D  // Red border on critical activity
        }
        
        operational_activity "Override System" {
            id: "OA-03"
            description: "Driver overrides automatic braking"
            performed_by: "OA-ACT-001"
        }
        
        operational_interaction "Hazard Detection" {
            id: "OI-01"
            from: "OA-ACT-003"
            to: "OA-ACT-002"
            exchange_item_kind: FLOW  // Cyan continuous sensor data
            description: "Obstacle position and velocity"
        }
        
        operational_interaction "Driver Alert" {
            id: "OI-02"
            from: "OA-ACT-002"
            to: "OA-ACT-001"
            exchange_item_kind: EVENT  // Red dashed warning event
            description: "Collision warning alert"
        }
    }

    // SYSTEM ANALYSIS - Functional exchanges with types
    architecture functional {
        function DetectCollision {
            id: SF-001
            ports_out: ["riskAssessment"]
        }
        
        function PlanBraking {
            id: SF-002
            ports_in: ["riskData"]
            ports_out: ["brakeCommand"]
        }
        
        function ExecuteBrake {
            id: SF-003
            ports_in: ["brakeCmd"]
        }
        
        // Event exchange - critical alert
        exchange RiskAlert {
            from: SF-001.riskAssessment
            to: SF-002.riskData
            exchange_item_kind: EVENT  // ⚡ Red dashed
            label: "CollisionAlert"
        }
        
        // Flow exchange - continuous monitoring
        exchange BrakeCommand {
            from: SF-002.brakeCommand
            to: SF-003.brakeCmd
            exchange_item_kind: FLOW  // ⟿ Cyan thick
            label: "ContinuousBrakeCmd"
        }
    }

    // LOGICAL ARCHITECTURE - Safety colors + Interfaces + Allocation
    architecture logical {
        component PerceptionUnit "Perception Unit" {
            description: "Environmental sensing and analysis"
            safety_level: ASIL_C  // Orange 5px border
            allocated_functions: [SF-001]  // Orange allocation line to SF-001
            
            // Provided interface - lollipop
            provides interface IRiskData {
                signals: [
                    "collisionRisk: Enum",
                    "timeToCollision: Real (s)",
                    "targetDistance: Real (m)"
                ]
            }
            
            // Required interface - socket
            requires interface ISensorInput {
                signals: ["rawSensorData: ByteArray"]
            }
        }
        
        component BrakeController "Brake Controller" {
            description: "Main brake control logic"
            safety_level: ASIL_D  // Red 6px border
            allocated_functions: [SF-002, SF-003]  // Orange allocation lines
            
            // Required interface - socket
            requires interface IRiskData
            
            // Provided interface - lollipop
            provides interface IBrakeCommand {
                signals: ["brakeForce: Real (N)"]
            }
        }
        
        // Component exchange with type
        exchange RiskDataExchange {
            from: PerceptionUnit.IRiskData
            to: BrakeController
            exchange_item_kind: EVENT  // ⚡ Red dashed
            label: "CriticalRiskAlert"
        }
    }

    // PHYSICAL ARCHITECTURE - Safety-critical hardware
    architecture physical {
        component RadarECU "Radar ECU" {
            description: "77GHz radar processing unit"
            node_type: Hardware
            safety_level: ASIL_C  // Orange 5px border on 3D cube
            implements: [PerceptionUnit]
            properties: {
                "Processor": "Infineon AURIX TC397",
                "PowerConsumption": "8W",
                "OperatingTemp": "-40°C to 85°C"
            }
        }
        
        component BrakeECU "Brake ECU" {
            description: "Primary brake control ECU"
            node_type: Hardware
            safety_level: ASIL_D  // Red 6px border on 3D cube
            implements: [BrakeController]
            properties: {
                "Processor": "Renesas RH850",
                "Memory": "4MB Flash + 512KB RAM",
                "Redundancy": "Dual-core lockstep"
            }
        }
        
        connect RadarECU -> BrakeECU via "CAN-FD (5 Mbps)"
    }

    // BREAKDOWN TREE - Safety propagation
    breakdown logical_components {
        node BrakingSystem {
            id: BN-001
            label: "Emergency Braking System"
            safety_level: ASIL_D  // Red border on tree node
            children: [
                {
                    id: BN-002
                    label: "Perception Subsystem"
                    safety_level: ASIL_C  // Orange border
                },
                {
                    id: BN-003
                    label: "Control Subsystem"
                    safety_level: ASIL_D  // Red border
                }
            ]
        }
    }

    // SEQUENCE DIAGRAM - Safety-critical messages
    scenarios {
        sequence EmergencyBraking "Emergency Braking Sequence" {
            participants: [RadarECU, BrakeController, BrakeActuator]
            
            message DetectObstacle {
                from: RadarECU
                to: BrakeController
                message_type: Synchronous
                safety_level: ASIL_D  // Red thick arrow (6px)
                label: "ObstacleDetected(distance, speed)"
            }
            
            message ActivateBrakes {
                from: BrakeController
                to: BrakeActuator
                message_type: Synchronous
                safety_level: ASIL_D  // Red thick arrow (6px)
                label: "BrakeCommand(force)"
            }
        }
    }

    // STATE MACHINE - Safety states
    statemachine BrakeController {
        initial_state: Monitoring
        
        state Monitoring {
            safety_level: ASIL_B  // Orange border on state
            entry_actions: ["EnableSensors"]
            exit_actions: ["LogEvent"]
        }
        
        state EmergencyBraking {
            safety_level: ASIL_D  // Red border on state
            entry_actions: ["ActivateBrakes", "AlertDriver"]
            exit_actions: ["ReleaseBrakes"]
        }
        
        transition ToEmergency {
            from: Monitoring
            to: EmergencyBraking
            trigger: "CollisionImminent"
            guard: "distance < 5m"
        }
    }

    traceability {
        trace STK-001 -> [SYS-001]
        trace SYS-001 -> [SF-001, SF-002]
        trace SF-001 -> [PerceptionUnit]
        trace SF-002 -> [BrakeController]
        trace PerceptionUnit -> [RadarECU]
        trace BrakeController -> [BrakeECU]
    }
}
```

**Visual Result**:
- ✅ **Red/Orange safety borders** on all safety-critical elements across ALL diagrams
- ✅ **Colored exchange arrows** with icons (⚡⟿↔📦🗄) showing data flow semantics
- ✅ **Ball-and-socket interfaces** (○⌒) on component boundaries
- ✅ **Orange allocation lines** connecting functions to components
- ✅ **Complete safety traceability** from operations → functions → components → hardware → behavior

---

## 🎯 MBSE FEATURE CHECKLIST FOR AI CLIENTS

When generating ArcLang models, **ALWAYS include**:

### ✅ Safety Colors
- [ ] Add `safety_level: ASIL_X` to safety-critical components (LA)
- [ ] Add `safety_level: ASIL_X` to safety-critical physical nodes (PA)
- [ ] Add `safety_level: ASIL_X` to safety-critical activities (OA)
- [ ] Add `safety_level: ASIL_X` to safety-critical messages (Sequence)
- [ ] Add `safety_level: ASIL_X` to safety-critical states (State Machine)
- [ ] Use appropriate standard: `ASIL_X` (automotive), `dal: DAL_X` (aerospace), `sil: SIL_X` (industrial)

### ✅ Exchange Item Types
- [ ] Add `exchange_item_kind: TYPE` to ALL functional exchanges
- [ ] Use EVENT for alerts/signals
- [ ] Use FLOW for continuous data streams
- [ ] Use OPERATION for service calls
- [ ] Use DATA for structured packets (default)
- [ ] Use SHARED_DATA for shared repositories
- [ ] Use UNSET for generic/unknown

### ✅ Interface Notation
- [ ] Use `provides interface IName` for provided interfaces
- [ ] Use `requires interface IName` for required interfaces
- [ ] Define signals in interface blocks
- [ ] Use IName (capital I prefix) for interface names

### ✅ Traceability
- [ ] Add `allocated_functions: [...]` to components
- [ ] Add `implements: [...]` to physical nodes
- [ ] Add `traces: [...]` to requirements
- [ ] Use explicit traceability block at end

### ✅ Complete Model Structure
- [ ] All elements inside `model { }` block
- [ ] Metadata with version and safety_standard
- [ ] Requirements with safety_level
- [ ] Multiple architecture views (operational, functional, logical, physical)
- [ ] Scenarios with traces
- [ ] Traceability block

---

## ⚠️ CRITICAL VALIDATION RULES

**The following will cause diagram rendering failures:**

1. ❌ **Missing exchange_item_kind** → Generic gray lines instead of semantic colors
2. ❌ **Missing safety_level** → No safety borders on critical components
3. ❌ **Using port instead of provides/requires** → No UML ball-and-socket notation
4. ❌ **Missing allocated_functions** → No allocation lines between functions and components
5. ❌ **String identifiers** → Parser errors
6. ❌ **Missing model wrapper** → Top-level blocks rejected

**Generate complete, professional MBSE models with all features enabled.**
