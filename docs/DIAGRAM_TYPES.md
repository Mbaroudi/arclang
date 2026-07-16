# ArcLang Diagram Types Reference

ArcLang supports all 10 Capella MBSE diagram types for comprehensive system modeling and visualization.

## Table of Contents

1. [Operational Activity Diagrams](#1-operational-activity-diagrams)
2. [Functional Dataflow Diagrams](#2-functional-dataflow-diagrams)
3. [Sequence Diagrams](#3-sequence-diagrams)
4. [State Machine Diagrams](#4-state-machine-diagrams)
5. [Component Block Diagrams](#5-component-block-diagrams)
6. [Physical Deployment Diagrams](#6-physical-deployment-diagrams)
7. [Class/Interface Diagrams](#7-classinterface-diagrams)
8. [Tree Diagrams](#8-tree-diagrams)
9. [Capability Diagrams](#9-capability-diagrams)
10. [Functional Chain Diagrams](#10-functional-chain-diagrams)

---

## 1. Operational Activity Diagrams

**Purpose:** Model high-level operational scenarios showing actors, entities, and activities in swimlanes.

**Use Cases:**
- Stakeholder interaction modeling
- Business process workflows
- Operational concept definition
- Mission scenarios

**CLI Command:**
```bash
arclang diagram model.arc -o operational.svg --format operational
```

**Key Elements:**
- Actors and entities (swimlanes)
- Operational activities (yellow boxes)
- Operational exchanges (data flows between activities)
- Capability associations

**Example:**
```arc
OperationalAnalysis "Vehicle Operation" {
    Actor "Driver" {
        Activity "Request Start" -> "Vehicle System"
        Activity "Monitor Status"
    }
    
    Entity "Vehicle System" {
        Activity "Process Command"
        Activity "Report Status" -> "Driver"
    }
}
```

---

## 2. Functional Dataflow Diagrams

**Purpose:** Show system functions and data flows between them, with port-based interfaces.

**Use Cases:**
- System requirements decomposition
- Functional architecture
- Data flow analysis
- Interface definition

**CLI Command:**
```bash
arclang diagram model.arc -o functional.svg --format functional
```

**Key Elements:**
- System functions (green boxes with ports)
- Function ports (IN/OUT/INOUT)
- Functional exchanges (port-to-port data flows)
- External actors

**Example:**
```arc
SystemFunction "Process Sensor Data" {
    ports {
        in raw_data: SensorData
        out processed_data: FilteredData
    }
}

SystemFunction "Make Decision" {
    ports {
        in processed_data: FilteredData
        out command: ControlCommand
    }
}

Exchange: "Process Sensor Data".out -> "Make Decision".in
```

---

## 3. Sequence Diagrams

**Purpose:** Show message exchanges between components over time, including timing constraints.

**Use Cases:**
- Behavioral scenarios
- Protocol definitions
- Timing analysis
- Component interaction modeling

**CLI Command:**
```bash
arclang diagram model.arc -o sequence.svg --format sequence
```

**Key Elements:**
- Participants (lifelines)
- Messages (synchronous, asynchronous, return)
- Combined fragments (PAR, OPT, LOOP, ALT)
- Timing constraints
- Activation boxes

**Example:**
```arc
Scenario "Emergency Stop" {
    Participant "Sensor" as Actor
    Participant "Controller" as Component
    Participant "Actuator" as Component
    
    "Sensor" -> "Controller": DetectHazard [sync]
    activate "Controller"
    "Controller" -> "Actuator": EmergencyBrake [async]
    "Actuator" -> "Controller": BrakeApplied [return]
    deactivate "Controller"
    
    timing: DetectHazard to BrakeApplied < 100ms
}
```

---

## 4. State Machine Diagrams

**Purpose:** Model component behavior as states and transitions with guards and actions.

**Use Cases:**
- Mode management
- Behavioral modeling
- Safety state analysis
- Control logic

**CLI Command:**
```bash
arclang diagram model.arc -o statemachine.svg --format statemachine
```

**Key Elements:**
- States (regular, composite, initial, final)
- Transitions with triggers, guards, actions
- Entry/exit actions
- Sub-states
- Choice/junction pseudo-states

**Example:**
```arc
StateMachine "Vehicle Controller" {
    initial -> "Idle"
    
    State "Idle" {
        entry: reset_systems()
        on Start [battery_ok] -> "Active" / initialize()
    }
    
    State "Active" {
        on EmergencyStop -> "SafeStop"
        on Stop -> "Idle"
        
        SubState "Cruising"
        SubState "Braking"
    }
    
    State "SafeStop" {
        entry: emergency_brake()
        exit: log_event()
        on Reset [systems_ok] -> "Idle"
    }
}
```

---

## 5. Component Block Diagrams

**Purpose:** Show logical component architecture with allocated functions and interfaces.

**Use Cases:**
- System decomposition
- Component interface definition
- Function allocation
- Architecture modeling

**CLI Command:**
```bash
arclang diagram model.arc -o component.svg --format component
```

**Key Elements:**
- Logical components (nested blue boxes)
- Component ports
- Component exchanges
- Allocated functions
- Interfaces (provided/required)

**Example:**
```arc
LogicalComponent "Vehicle Controller" {
    type: Logical
    
    SubComponent "Sensor Manager" {
        allocates: ["Process Sensor Data", "Validate Data"]
        ports {
            out sensor_feed: SensorData
        }
    }
    
    SubComponent "Decision Engine" {
        allocates: ["Make Decision", "Plan Actions"]
        ports {
            in sensor_feed: SensorData
            out commands: ControlCommand
        }
    }
    
    Exchange: "Sensor Manager".sensor_feed -> "Decision Engine".sensor_feed
}
```

---

## 6. Physical Deployment Diagrams

**Purpose:** Map logical components to physical hardware nodes with deployment relationships.

**Use Cases:**
- Hardware architecture
- Deployment planning
- Network topology
- Resource allocation

**CLI Command:**
```bash
arclang diagram model.arc -o physical.svg --format physical
```

**Key Elements:**
- Physical nodes (hardware/software/systems)
- Behavior components (software)
- Hardware components (processors, sensors, actuators)
- Physical links (networks, buses)
- Deployments (component → node mapping)

**Example:**
```arc
PhysicalArchitecture "Vehicle Hardware" {
    PhysicalNode "Main ECU" {
        type: Hardware
        processor: "ARM Cortex-A72"
        memory: "4GB"
        
        deploys: ["Vehicle Controller", "Sensor Manager"]
        
        HardwareComponent "CAN Bus Interface"
    }
    
    PhysicalNode "Sensor Cluster" {
        type: Hardware
        
        HardwareComponent "LIDAR"
        HardwareComponent "Camera"
    }
    
    PhysicalLink: "Main ECU" <-> "Sensor Cluster" {
        protocol: "CAN FD"
        bandwidth: "2 Mbps"
    }
}
```

---

## 7. Class/Interface Diagrams

**Purpose:** Define bit-precise data structures, enumerations, and interface protocols.

**Use Cases:**
- Data modeling
- API/interface definition
- Message format specification
- Type system definition

**CLI Command:**
```bash
arclang diagram model.arc -o class.svg --format class
```

**Key Elements:**
- Exchange items (classes with attributes)
- Data types (enumerations, primitives)
- Interface definitions (protocols/formats)
- Associations (references)
- Generalizations (inheritance)

**Example:**
```arc
DataType VehicleStatus {
    enumeration [IDLE = 0, ACTIVE = 1, FAULT = 2]
}

ExchangeItem VehicleState {
    stereotype: "data"
    attributes {
        speed: Double = 0.0
        heading: Double
        status: VehicleStatus = IDLE
        timestamp: Long
    }
}

ExchangeItem Command {
    stereotype: "event"
    attributes {
        type: CommandType
        priority: Integer = 5
    }
}

InterfaceDefinition VehicleAPI {
    protocol: "REST"
    format: "JSON"
}
```

---

## 8. Tree Diagrams

**Purpose:** Visualize hierarchical breakdown of functions or components with fold/unfold capability.

**Use Cases:**
- Function decomposition
- Component hierarchy
- Work breakdown structure
- Architecture exploration

**CLI Command:**
```bash
arclang diagram model.arc -o tree.svg --format tree
```

**Key Elements:**
- Tree nodes (functions or components)
- Parent-child relationships
- Expand/collapse indicators (⊞/⊟)
- Category colors
- Depth levels

**Example:**
```arc
SystemFunction "Vehicle Control" {
    category: System
    icon: "🚗"
    
    SubFunction "Navigation" {
        category: Control
        icon: "🧭"
        
        SubFunction "Route Planning" { icon: "📍" }
        SubFunction "Path Tracking" { icon: "📏" }
        SubFunction "Obstacle Avoidance" { icon: "🚧" }
    }
    
    SubFunction "Motion Control" {
        category: System
        
        SubFunction "Throttle Control"
        SubFunction "Steering Control"
        SubFunction "Brake Control"
    }
    
    SubFunction "Safety Monitor" {
        category: Management
        icon: "🛡️"
        
        SubFunction "Collision Detection"
        SubFunction "Emergency Stop"
    }
}
```

---

## 9. Capability Diagrams

**Purpose:** Model system capabilities at mission, capability, and sub-capability levels with relationships.

**Use Cases:**
- Requirements analysis
- Capability-based planning
- System-of-systems modeling
- Operational concept definition

**CLI Command:**
```bash
arclang diagram model.arc -o capability.svg --format capability
```

**Key Elements:**
- Missions (top-level, orange)
- Capabilities (mid-level, blue)
- Sub-capabilities (bottom-level, green)
- Capability associations (includes, extends, generalizes)
- Stereotypes

**Example:**
```arc
Mission "Autonomous Vehicle Operation" {
    Capability "Navigate to Destination" {
        stereotype: "primary"
        
        SubCapability "Plan Route"
        SubCapability "Follow Route"
        SubCapability "Adapt to Traffic"
    }
    
    Capability "Detect and Avoid Obstacles" {
        stereotype: "safety"
        
        SubCapability "Sense Environment"
        SubCapability "Classify Objects"
        SubCapability "Calculate Avoidance Maneuver"
    }
    
    Capability "Operate Within Safety Constraints" {
        stereotype: "safety"
        
        SubCapability "Monitor System Health"
        SubCapability "Execute Emergency Stop"
        SubCapability "Report Faults"
    }
}

// Capability relationships
"Navigate to Destination" includes "Detect and Avoid Obstacles"
"Detect and Avoid Obstacles" extends "Operate Within Safety Constraints"
```

---

## 10. Functional Chain Diagrams

**Purpose:** Show execution scenarios as sequences of function invocations with data flow.

**Use Cases:**
- Scenario modeling
- Execution flow analysis
- Performance analysis
- Integration testing

**CLI Command:**
```bash
arclang diagram model.arc -o functional-chain.svg --format functional-chain
```

**Key Elements:**
- Function nodes with ports (left-to-right flow)
- Functional exchanges (blue arrows with data types)
- Execution order
- Category colors
- Port indicators

**Example:**
```arc
FunctionalChain "Emergency Stop Scenario" {
    Function "Detect Hazard" {
        category: Environmental
        icon: "⚠️"
        ports {
            in sensor_data: SensorData
            out hazard_detected: HazardEvent
        }
    }
    
    Function "Assess Risk" {
        category: Management
        icon: "🔍"
        ports {
            in hazard: HazardEvent
            out risk_level: RiskLevel
        }
    }
    
    Function "Decide Action" {
        category: Control
        icon: "🎯"
        ports {
            in risk: RiskLevel
            out action_command: ActionCommand
        }
    }
    
    Function "Execute Emergency Stop" {
        category: System
        icon: "🛑"
        ports {
            in action: ActionCommand
            out brake_command: BrakeCommand
        }
    }
    
    // Execution flow
    "Detect Hazard" -> "Assess Risk": HazardEvent
    "Assess Risk" -> "Decide Action": RiskLevel
    "Decide Action" -> "Execute Emergency Stop": ActionCommand
}
```

---

## Generating All Diagrams

To generate all 10 diagram types at once:

```bash
arclang diagram model.arc -o output.svg --format all
```

This will create:
- `output_operational.svg`
- `output_functional.svg`
- `output_sequence.svg`
- `output_statemachine.svg`
- `output_component.svg`
- `output_physical.svg`
- `output_class.svg`
- `output_tree.svg`
- `output_capability.svg`
- `output_functional-chain.svg`

---

## Diagram Type Comparison

| Diagram Type | View Level | Primary Focus | Key Notation |
|--------------|------------|---------------|--------------|
| Operational Activity | Business | Stakeholder operations | Swimlanes, activities |
| Functional Dataflow | System | Function interfaces | Ports, data flows |
| Sequence | Behavioral | Time-ordered interactions | Lifelines, messages |
| State Machine | Behavioral | Mode/state behavior | States, transitions |
| Component Block | Logical | Component architecture | Nested boxes, interfaces |
| Physical Deployment | Physical | Hardware mapping | Nodes, links |
| Class/Interface | Data | Data structures | Classes, attributes |
| Tree | Structural | Hierarchy breakdown | Tree nodes, levels |
| Capability | Requirements | System capabilities | Mission/capability/sub |
| Functional Chain | Scenario | Execution sequences | Function flow, data |

---

## Best Practices

### 1. **Model Consistency**
- Use the same naming conventions across all diagram types
- Reference the same functions/components in multiple views
- Keep data type definitions consistent with exchange items

### 2. **Traceability**
- Link capabilities to functions (capability → functional)
- Map functions to components (functional → logical)
- Deploy components to nodes (logical → physical)
- Define data structures used in exchanges (class)

### 3. **Diagram Selection**
- Start with **operational** for stakeholder analysis
- Use **capability** for requirements breakdown
- Develop **functional** for system decomposition
- Create **sequence** and **state machine** for behavior
- Design **component** for logical architecture
- Plan **physical** for deployment
- Define **class** for data contracts
- Visualize **tree** for hierarchy exploration
- Model **functional chain** for scenarios

### 4. **Integration**
- All diagrams share the same underlying model
- Changes propagate across related diagrams
- Export to JSON preserves full model fidelity
- Import from Capella maintains compatibility

---

## Diagram Output Formats

Currently supported: **SVG** (Scalable Vector Graphics)

SVG benefits:
- Vector format (infinite zoom)
- Embeddable in web pages
- Viewable in all modern browsers
- Editable in tools like Inkscape, Illustrator
- Small file size with high quality

---

## Integration with Other Tools

### Capella Import/Export
```bash
# Import from Capella
arclang import model.capella -o model.arc

# Generate diagrams from imported model
arclang diagram model.arc -o diagrams.svg --format all
```

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Generate Architecture Diagrams
  run: |
    arclang diagram src/architecture.arc -o docs/diagrams.svg --format all
```

### Documentation Generation
```bash
# Include diagrams in documentation
arclang diagram model.arc -o docs/operational.svg --format operational
arclang diagram model.arc -o docs/functional.svg --format functional
# ... embed SVGs in Markdown/HTML docs
```

---

## Troubleshooting

### Diagram Generation Fails
1. Ensure diagram-service is installed: `cd arcviz-web/apps/diagram-service && npm install`
2. Verify Node.js is available: `node --version`
3. Check output directory exists and is writable
4. Ensure model compiles without errors

### Empty Diagrams
- Some diagram types require specific model elements
- Example: Sequence diagrams need `Scenario` blocks
- Example: State machines need `StateMachine` blocks
- Use `--format all` to see which diagrams have content

### Layout Issues
- Tree diagrams work best with 3-5 levels of depth
- Component diagrams may overlap with deep nesting
- Adjust spacing in renderer settings if needed

---

## Next Steps

1. Review [ArcLang Language Guide](./LANGUAGE_GUIDE.md) for syntax
2. Explore [Example Models](../examples/) for complete systems
3. Read [Capella Method Guide](./CAPELLA_METHOD.md) for methodology
4. Check [API Reference](./API_REFERENCE.md) for programmatic usage

---

## References

- **Capella MBSE Tool**: https://www.eclipse.org/capella/
- **Arcadia Method**: https://www.eclipse.org/capella/arcadia.html
- **ArcLang Repository**: https://github.com/yourusername/arclang
- **Diagram Service**: `/arcviz-web/apps/diagram-service/`
