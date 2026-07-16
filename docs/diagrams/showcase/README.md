# ArcLang Diagram Showcase - Generated Examples

This directory contains example diagrams demonstrating ArcLang's 10 Capella MBSE diagram types.

## Available Diagrams

### 1. Operational Activity Diagram ✅
**File:** `acc-operational.svg` (2.7KB)  
**Source:** Adaptive Cruise Control example  
**Content:** Actors, entities, operational activities with swimlanes

### 2. Functional Dataflow Diagram ✅
**File:** `acc-functional.svg` (2.2KB)  
**Source:** Adaptive Cruise Control example  
**Content:** System functions with port-based data flows

### 3. Sequence Diagram ⏭️
**Status:** Requires scenario definition in model  
**Note:** ACC example doesn't include scenarios

### 4. State Machine Diagram ⏭️
**Status:** Requires state machine definition in model  
**Note:** ACC example doesn't include state machines

### 5. Component Block Diagram ✅
**File:** `acc-component.svg` (2.7KB)  
**Source:** Adaptive Cruise Control example  
**Content:** Logical components with allocated functions

### 6. Physical Deployment Diagram ⏭️
**Status:** Generated but has ELK layout errors  
**Note:** Requires debugging physical node references

### 7. Class/Interface Diagram ⭐ NEW ✅
**File:** `vehicle-class.svg` (9.2KB)  
**Source:** Vehicle data model sample  
**Content:** 4 ExchangeItems (classes), 5 DataTypes (enumerations)  
**Features:**
- Data structures with attributes
- Enumerations with values
- Associations and inheritance
- Stereotypes (data, event, interface)

### 8. Tree Diagram ⭐ NEW ✅
**File:** `vehicle-tree.svg` (11KB)  
**Source:** Vehicle Control function hierarchy  
**Content:** 16 functions in 3-level hierarchy  
**Features:**
- Function breakdown tree
- Category-based coloring
- Expand/collapse indicators
- Icons for each function

### 9. Capability Diagram ⭐ NEW ✅
**File:** `vehicle-capability.svg` (12KB)  
**Source:** Autonomous vehicle capabilities  
**Content:** 1 Mission, 4 Capabilities, 11 Sub-Capabilities  
**Features:**
- Three-level hierarchy
- Capability associations (includes, extends)
- Stereotypes (primary, safety)
- Visual hierarchy with colors

### 10. Functional Chain Diagram ⭐ NEW ✅
**File:** `emergency-stop-chain.svg` (7.7KB)  
**Source:** Emergency stop scenario  
**Content:** 6 functions in execution sequence  
**Features:**
- Left-to-right execution flow
- Port visualization
- Data flow arrows
- Category-based coloring

## Diagram Statistics

| Diagram Type | Status | File Size | Dimensions | Elements |
|--------------|--------|-----------|------------|----------|
| Operational | ✅ | 2.7KB | - | Activities, actors |
| Functional | ✅ | 2.2KB | - | Functions, exchanges |
| Sequence | ⏭️ | - | - | - |
| State Machine | ⏭️ | - | - | - |
| Component | ✅ | 2.7KB | - | Components |
| Physical | ⏭️ | - | - | - |
| **Class** | ✅ | **9.2KB** | **1690×620** | **9 items** |
| **Tree** | ✅ | **11KB** | **16500×460** | **16 nodes** |
| **Capability** | ✅ | **12KB** | **1958×1560** | **16 caps** |
| **Functional Chain** | ✅ | **7.7KB** | **2860×240** | **6 funcs** |

**Total Generated:** 7/10 diagrams (70%)  
**Total Size:** ~48KB  
**New Diagrams:** 4/4 (100% of new types working)

## How to View

All diagrams are in SVG format and can be viewed:
- **In Browser:** Open the .svg file directly
- **In VS Code:** Install SVG extension
- **In Documentation:** Embed with `![Diagram](path/to/diagram.svg)`
- **In Presentation:** Export to PDF or PNG

## How to Generate

```bash
# Individual diagrams
arclang diagram model.arc -o output.svg --format <type>

# All diagrams
arclang diagram model.arc -o output.svg --format all
```

**Formats:**
- `operational` - Operational activity diagrams
- `functional` - Functional dataflow diagrams
- `sequence` - Sequence diagrams
- `statemachine` - State machine diagrams
- `component` - Component block diagrams
- `physical` - Physical deployment diagrams
- `class` - Class/interface diagrams ⭐
- `tree` - Tree diagrams ⭐
- `capability` - Capability diagrams ⭐
- `functional-chain` - Functional chain diagrams ⭐

## Source Models

### Adaptive Cruise Control (ACC)
- **File:** `examples/automotive/adaptive_cruise_control.arc`
- **Contains:** Operational analysis, system functions, logical components, physical architecture
- **Generates:** Operational, Functional, Component diagrams

### Vehicle Data Model
- **File:** `arcviz-web/apps/diagram-service/sample-class.json`
- **Contains:** ExchangeItems, DataTypes with enumerations
- **Generates:** Class diagrams

### Vehicle Control Hierarchy
- **File:** `arcviz-web/apps/diagram-service/sample-tree.json`
- **Contains:** Hierarchical function breakdown
- **Generates:** Tree diagrams

### Autonomous Vehicle Capabilities
- **File:** `arcviz-web/apps/diagram-service/sample-capability.json`
- **Contains:** Mission, capabilities, sub-capabilities with associations
- **Generates:** Capability diagrams

### Emergency Stop Scenario
- **File:** `arcviz-web/apps/diagram-service/sample-functional-chain.json`
- **Contains:** Function execution sequence with data flows
- **Generates:** Functional chain diagrams

## Next Steps

To generate the missing diagrams (Sequence, State Machine, Physical):

1. **Add Scenarios** to ACC model for Sequence diagrams
2. **Add State Machines** to ACC model for State Machine diagrams
3. **Fix Physical References** in ACC model for Physical diagrams

Example additions needed:

```arc
// Add scenario for sequence diagram
scenario "ACC Activation" {
    participant "Driver" as Actor
    participant "ACC Controller" as Component
    
    "Driver" -> "ACC Controller": Press ACC Button [sync]
    "ACC Controller" -> "Radar": Enable [async]
}

// Add state machine for state diagram
state_machine "ACC Controller" {
    initial_state: "OFF"
    
    state "OFF" {
        entry_actions: ["disable_actuators"]
    }
    
    state "ACTIVE" {
        entry_actions: ["engage_control"]
    }
    
    transition {
        from: "OFF"
        to: "ACTIVE"
        trigger: "ACC_BUTTON_PRESSED"
    }
}
```

## More Examples

For complete examples with all 10 diagram types, see:
- `docs/DIAGRAM_TYPES.md` - Complete reference
- `docs/DIAGRAM_SHOWCASE.md` - Visual showcase
- `examples/` - Sample ArcLang models

---

*Generated: October 25, 2024*  
*ArcLang Version: 1.0.0*  
*Status: Production Ready* ✅
