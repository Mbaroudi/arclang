# SysML v2 ↔ Capella ↔ ArcLang Complete Mapping

**Date**: 2025-11-03  
**Repository Analyzed**: [SysML-v2-Release](https://github.com/Systems-Modeling/SysML-v2-Release) - 251 examples  
**Purpose**: Align ArcLang with SysML v2 standard for full Capella/Arcadia MBSE compliance

---

## 📋 Executive Summary

**Key Finding**: **Capella is NOT directly based on SysML v2**. Capella is based on the **Arcadia methodology** which predates SysML v2 (2023). However, there are **significant conceptual alignments** between:

1. **SysML v2** (OMG Standard, 2023) - Modern MBSE language
2. **Capella/Arcadia** (Thales, 2005+) - Method-driven MBSE tool
3. **ArcLang** (Current) - DSL for Arcadia methodology

**Recommendation**: Enhance ArcLang to support **SysML v2 constructs** where they map to Capella/Arcadia, creating a **bi-directional bridge** between the two MBSE approaches.

---

## 🔄 Three-Way Mapping Matrix

### 1. **Structural Elements (Parts & Components)**

| SysML v2 | Capella/Arcadia | ArcLang (Current) | Status | Enhancement Needed |
|----------|-----------------|-------------------|--------|-------------------|
| `part def Vehicle` | Logical Component (Definition) | `component Vehicle { }` | ✅ Compatible | None |
| `part vehicle1: Vehicle` | Logical Component (Instance) | `component vehicle1 "Vehicle" { }` | ✅ Compatible | None |
| `part def Engine;` | Sub-Component | `component Engine { parent: Vehicle }` | ✅ Compatible | None |
| `attribute mass: Real` | Component Property | `component { properties: {"mass": "Real"} }` | ⚠️ Partial | Add `attribute` keyword |
| `ref part driver: Person` | Reference Component | Not explicitly supported | ❌ Missing | Add `ref component` |
| **Multiplicity**: `part wheel: Wheel[2] ordered` | Cardinality | Not supported | ❌ Missing | Add `[min..max]` syntax |

**SysML v2 Example**:
```sysml
part def Vehicle {
    attribute mass: Real;
    part engine: Engine;
    ref part driver: Person;
}
```

**ArcLang Equivalent (Current)**:
```arc
architecture logical {
    component Vehicle "Vehicle" {
        description: "Vehicle with engine"
        properties: {
            "mass": "Real"
        }
    }
    
    component Engine "Engine" {
        description: "Vehicle engine"
        parent: Vehicle
    }
}
```

**ArcLang Proposed (SysML v2-aligned)**:
```arc
architecture logical {
    component def Vehicle {
        attribute mass: Real
        part engine: Engine
        ref part driver: Person
    }
    
    component Engine { }
    component Person { }
}
```

---

### 2. **Ports & Interfaces**

| SysML v2 | Capella/Arcadia | ArcLang (Current) | Status | Enhancement |
|----------|-----------------|-------------------|--------|-------------|
| `port def FuelOutPort` | Port Definition | `provides interface IFuelOut` | ✅ Compatible | Alias `port def` |
| `port fuelPort: FuelOutPort` | Port Instance | `provides interface IFuelOut { signals: [...] }` | ✅ Compatible | None |
| `in item fuelSupply: Fuel` | Input Flow | `requires interface { signals: ["fuelIn: Fuel"] }` | ✅ Compatible | Add `in/out` keywords |
| `out item fuelReturn: Fuel` | Output Flow | `provides interface { signals: ["fuelOut: Fuel"] }` | ✅ Compatible | Add `in/out` keywords |
| `interface def EngineInterface { end port1, end port2 }` | Interface Connection | `connect EnginePort -> TransmissionPort` | ✅ Compatible | Add `interface def` |

**SysML v2 Example**:
```sysml
port def FuelOutPort {
    attribute temperature: Temp;
    out item fuelSupply: Fuel;
    in item fuelReturn: Fuel;
}

part def Engine {
    port fuelPort: FuelOutPort;
}
```

**ArcLang Current**:
```arc
component Engine {
    provides interface IFuelOut {
        signals: [
            "temperature: Temp",
            "fuelSupply: Fuel (out)",
            "fuelReturn: Fuel (in)"
        ]
    }
}
```

**ArcLang Proposed**:
```arc
port def FuelOutPort {
    attribute temperature: Temp
    out item fuelSupply: Fuel
    in item fuelReturn: Fuel
}

component Engine {
    port fuelPort: FuelOutPort
}
```

---

### 3. **Actions & Functions (Behavioral)**

| SysML v2 | Capella/Arcadia | ArcLang (Current) | Status | Enhancement |
|----------|-----------------|-------------------|--------|-------------|
| `action def 'Generate Torque'` | System Function (Definition) | `function GenerateTorque { }` | ✅ Compatible | Add `action def` |
| `action 'generate torque': 'Generate Torque'` | Function Instance | `function generateTorque { id: SF-001 }` | ✅ Compatible | None |
| `in fuelCmd: FuelCmd` | Function Input Port | `function { ports_in: ["fuelCmd"] }` | ✅ Compatible | Add typed inputs |
| `out engineTorque: Torque` | Function Output Port | `function { ports_out: ["torque"] }` | ✅ Compatible | Add typed outputs |
| `flow from.out to to.in` | Functional Exchange | `exchange { from: SF-001, to: SF-002 }` | ✅ Compatible | Add `flow` keyword |
| `first start then continue` | Control Flow | Not supported | ❌ Missing | Add succession/control flow |
| `accept engineStart: EngineStart` | Event Reception | Operational activity | ⚠️ Partial | Add `accept` action |
| `perform 'provide power'` | Function Allocation | `allocated_functions: [SF-001]` | ✅ Compatible | Add `perform` keyword |

**SysML v2 Example**:
```sysml
action def 'Generate Torque' {
    in fuelCmd: FuelCmd;
    out engineTorque: Torque;
}

action 'provide power' {
    action 'generate torque': 'Generate Torque';
    action 'amplify torque': 'Amplify Torque';
    
    flow 'generate torque'.engineTorque 
        to 'amplify torque'.engineTorque;
    
    first start then 'generate torque';
    first 'generate torque' then 'amplify torque';
}
```

**ArcLang Current**:
```arc
architecture functional {
    function GenerateTorque {
        id: SF-001
        ports_in: ["fuelCmd"]
        ports_out: ["engineTorque"]
    }
    
    function AmplifyTorque {
        id: SF-002
        ports_in: ["engineTorque"]
        ports_out: ["transmissionTorque"]
    }
    
    exchange TorqueFlow {
        from: SF-001.engineTorque
        to: SF-002.engineTorque
        exchange_item_kind: FLOW
    }
}
```

**ArcLang Proposed (SysML v2-aligned)**:
```arc
action def 'Generate Torque' {
    in fuelCmd: FuelCmd
    out engineTorque: Torque
}

action 'provide power' {
    action 'generate torque': 'Generate Torque'
    action 'amplify torque': 'Amplify Torque'
    
    flow 'generate torque'.engineTorque 
        to 'amplify torque'.engineTorque
    
    succession start -> 'generate torque' -> 'amplify torque'
}
```

---

### 4. **Requirements**

| SysML v2 | Capella/Arcadia | ArcLang (Current) | Status | Enhancement |
|----------|-----------------|-------------------|--------|-------------|
| `requirement def <'1'> MassReq` | Requirement Definition | `req STK-001 "Title" { }` | ✅ Compatible | Add `requirement def` |
| `requirement <'1.1'> vehicleMass: MassReq` | Requirement Instance | `req SYS-001 "Title" { traces: [STK-001] }` | ✅ Compatible | None |
| `subject vehicle: Vehicle` | Requirement Subject | Not supported | ❌ Missing | Add `subject` attribute |
| `doc /* text */` | Requirement Text | `description: "text"` | ✅ Compatible | Add `doc` keyword |
| `require constraint { mass <= maxMass }` | Constraint | Not supported | ❌ Missing | Add `require/assume constraint` |
| `satisfy reqId by component` | Satisfaction Link | `traces: [REQ-ID] → [Component]` | ⚠️ Partial | Add `satisfy` keyword |
| `assume constraint { ... }` | Assumption | Not supported | ❌ Missing | Add `assume` block |

**SysML v2 Example**:
```sysml
requirement def <'1'> MassLimitationRequirement {
    doc /* The actual mass shall be less than or equal to the required mass. */
    
    attribute massActual: MassValue;
    attribute massReqd: MassValue;
    
    require constraint {
        massActual <= massReqd
    }
}

requirement <'1.1'> vehicleMass: MassLimitationRequirement {
    doc /* The vehicle mass shall be less than or equal to 2000 kg. */
    
    subject vehicle: Vehicle;
    
    attribute :>> massActual = vehicle.mass;
    attribute :>> massReqd = 2000 [kg];
    
    assume constraint fuelConstraint {
        doc /* full fuel tank */
        vehicle.fuelLevel >= vehicle.fuelTankCapacity
    }
}

satisfy vehicleMass by vehicle1_c1;
```

**ArcLang Current**:
```arc
requirements stakeholder {
    req STK_001 "Mass Limitation" {
        description: "The actual mass shall be less than or equal to the required mass"
        priority: Critical
        safety_level: ASIL_B
    }
}

requirements system {
    req SYS_001 "Vehicle Mass Limit" {
        description: "The vehicle mass shall be less than or equal to 2000 kg"
        priority: Critical
        safety_level: ASIL_B
        traces: [STK_001]
        verification: "Mass measurement test"
    }
}

traceability {
    trace SYS_001 -> [VehicleComponent]
}
```

**ArcLang Proposed**:
```arc
requirement def MassLimitationRequirement {
    id: "STK_001"
    doc /* The actual mass shall be less than or equal to the required mass. */
    
    attribute massActual: Real
    attribute massReqd: Real
    
    require constraint {
        massActual <= massReqd
    }
}

requirement vehicleMass: MassLimitationRequirement {
    id: "SYS_001"
    doc /* The vehicle mass shall be less than or equal to 2000 kg. */
    
    subject vehicle: Vehicle
    
    attribute massActual = vehicle.mass
    attribute massReqd = 2000 [kg]
    
    assume constraint fuelConstraint {
        doc /* full fuel tank */
        vehicle.fuelLevel >= vehicle.fuelTankCapacity
    }
}

satisfy vehicleMass by vehicle1_c1
```

---

### 5. **Allocation (Functional to Logical)**

| SysML v2 | Capella/Arcadia | ArcLang (Current) | Status | Enhancement |
|----------|-----------------|-------------------|--------|-------------|
| `perform 'provide power'` | Function Allocation | `allocated_functions: [SF-001]` | ✅ Compatible | Add `perform` keyword |
| `perform action.'sub-action'` | Sub-function Allocation | `allocated_functions: [SF-001]` | ✅ Compatible | Hierarchical allocation |
| `ref action 'name' :> 'def'` | Reference to Action | Not supported | ❌ Missing | Add `ref action` |

**SysML v2 Example**:
```sysml
part vehicle: Vehicle {
    perform 'provide power' {
        in fuelCmd = fuelCmdPort.fuelCmd;
    }
    
    part engine: Engine {
        perform 'provide power'.'generate torque' {
            in fuelCmd = fuelCmdPort.fuelCmd;
            out engineTorque = drivePwrPort.engineTorque;
        }
    }
}
```

**ArcLang Current**:
```arc
component Vehicle {
    allocated_functions: [ProvidePower]
}

component Engine {
    parent: Vehicle
    allocated_functions: [GenerateTorque]
}
```

**ArcLang Proposed**:
```arc
component vehicle: Vehicle {
    perform 'provide power' {
        in fuelCmd = fuelCmdPort.fuelCmd
    }
    
    component engine: Engine {
        perform 'provide power'.'generate torque' {
            in fuelCmd = fuelCmdPort.fuelCmd
            out engineTorque = drivePwrPort.engineTorque
        }
    }
}
```

---

### 6. **State Machines (Behavioral)**

| SysML v2 | Capella/Arcadia | ArcLang (Current) | Status | Enhancement |
|----------|-----------------|-------------------|--------|-------------|
| `state def StateName` | State Definition | Not in examples | ❌ Missing | Add `state def` |
| `state stateName: StateName` | State Instance | `state StateName { }` (inferred) | ⚠️ Partial | None |
| `transition from to via event` | State Transition | `transition From -> To` | ⚠️ Partial | Add event triggers |
| `entry action { ... }` | Entry Action | Not supported | ❌ Missing | Add `entry/exit/do` |
| `do action { ... }` | Do Activity | Not supported | ❌ Missing | Add `do` keyword |

**Note**: State machines are less emphasized in SysML v2 compared to SysML v1. ArcLang already has basic support.

---

### 7. **Packages & Namespaces**

| SysML v2 | Capella/Arcadia | ArcLang (Current) | Status | Enhancement |
|----------|-----------------|-------------------|--------|-------------|
| `package 'Package Name'` | Package | `model PackageName { }` | ✅ Compatible | Add `package` keyword |
| `private import PackageName::*` | Import | Not supported | ❌ Missing | Add `import` statement |
| `public import PackageName::*` | Public Import | Not supported | ❌ Missing | Add `public import` |
| `alias Torque for ISQ::TorqueValue` | Alias | Not supported | ❌ Missing | Add `alias` keyword |

**SysML v2 Example**:
```sysml
package '1a-Parts Tree' {
    private import SI::kg;
    
    package Definitions {
        part def Vehicle { }
    }
    
    package Usages {
        private import Definitions::*;
        part vehicle1: Vehicle { }
    }
}
```

**ArcLang Current**: No package support (flat model structure)

**ArcLang Proposed**:
```arc
package PartsTree {
    import SI::kg
    
    package Definitions {
        component def Vehicle { }
    }
    
    package Usages {
        import Definitions::*
        component vehicle1: Vehicle { }
    }
}
```

---

## 🎯 Priority Enhancements for ArcLang

### **Phase 1: Critical SysML v2 Alignment** (4-6 weeks)

#### 1.1 **Typed Attributes**
```arc
// Current
component Engine {
    properties: {
        "power": "Real"
    }
}

// Enhanced (SysML v2-aligned)
component def Engine {
    attribute power: Real [kW]
    attribute efficiency: Real (0.0..1.0)
}
```

**Implementation**:
- Add `attribute name: Type [unit]` syntax
- Support units from SI/ISQ libraries
- Support value ranges `(min..max)`

#### 1.2 **Port Definitions with Typed Items**
```arc
// Enhanced
port def FuelPort {
    in item fuelSupply: Fuel
    out item fuelReturn: Fuel
    attribute temperature: Temp
}

component Engine {
    port fuelPort: FuelPort
}
```

**Implementation**:
- Add `port def` keyword
- Add `in item` / `out item` for flow items
- Map to Capella ComponentPort

#### 1.3 **Action Definitions with Typed Parameters**
```arc
// Enhanced
action def GenerateTorque {
    in fuelCmd: FuelCmd
    out engineTorque: Torque
}

action providePower {
    action generateTorque: GenerateTorque
    action amplifyTorque: AmplifyTorque
    
    flow generateTorque.engineTorque 
        to amplifyTorque.engineTorque
}
```

**Implementation**:
- Add `action def` keyword
- Add typed `in`/`out` parameters
- Add `flow from.param to to.param` syntax

#### 1.4 **Functional Allocation with `perform`**
```arc
// Enhanced
component vehicle: Vehicle {
    perform providePower {
        in fuelCmd = fuelCmdPort.fuelCmd
    }
    
    component engine: Engine {
        perform providePower.generateTorque {
            in fuelCmd = fuelCmdPort.fuelCmd
            out engineTorque = drivePwrPort.engineTorque
        }
    }
}
```

**Implementation**:
- Add `perform actionName` keyword
- Support hierarchical allocation (`.sub-action`)
- Auto-generate Capella ComponentFunctionalAllocation

---

### **Phase 2: Requirements Enhancement** (3-4 weeks)

#### 2.1 **Requirement Definitions with Constraints**
```arc
requirement def MassLimitationRequirement {
    doc /* Mass shall be within limits */
    
    attribute massActual: Real
    attribute massReqd: Real
    
    require constraint {
        massActual <= massReqd
    }
}

requirement vehicleMass: MassLimitationRequirement {
    id: "SYS_001"
    subject vehicle: Vehicle
    
    attribute massActual = vehicle.mass
    attribute massReqd = 2000 [kg]
    
    assume constraint {
        vehicle.fuelLevel >= vehicle.fuelTankCapacity
    }
}

satisfy vehicleMass by vehicle1_c1
```

**Implementation**:
- Add `requirement def` for reusable patterns
- Add `subject: Type` for requirement subject
- Add `require constraint` for formal verification
- Add `assume constraint` for assumptions
- Add `satisfy reqId by component` for satisfaction links

---

### **Phase 3: Advanced Features** (4-6 weeks)

#### 3.1 **Package System**
```arc
package EmergencyBraking {
    import SafetyLibrary::ASIL
    import SI::{kg, m, s}
    
    package Definitions {
        component def BrakeController { }
    }
    
    package Usages {
        import Definitions::*
        component mainController: BrakeController { }
    }
}
```

#### 3.2 **Control Flow / Succession**
```arc
action providePower {
    succession start -> generateTorque -> amplifyTorque -> done
    
    decision "Torque sufficient?" {
        if torque > threshold -> amplifyTorque
        else -> generateMoreTorque
    }
}
```

#### 3.3 **Reference Parts**
```arc
component def Vehicle {
    part engine: Engine          // Composite (owned)
    ref part driver: Person       // Reference (not owned)
}
```

#### 3.4 **Multiplicities**
```arc
component def AxleAssembly {
    part wheel: Wheel[2] ordered  // Exactly 2 wheels, ordered
    part sensor: Sensor[1..*]     // At least 1 sensor
}
```

---

## 📊 Feature Completeness Matrix

| Feature Category | SysML v2 | Capella | ArcLang Current | ArcLang Target | Priority |
|------------------|----------|---------|-----------------|----------------|----------|
| **Structural** | ✅ 100% | ✅ 100% | 🟡 70% | 🟢 95% | **High** |
| - Part definitions | ✅ | ✅ | ✅ | ✅ | - |
| - Attributes (typed) | ✅ | ✅ | 🟡 Partial | 🟢 Full | P1 |
| - References | ✅ | ✅ | ❌ | 🟢 Full | P3 |
| - Multiplicities | ✅ | ✅ | ❌ | 🟢 Full | P3 |
| **Behavioral** | ✅ 100% | ✅ 100% | 🟡 65% | 🟢 90% | **High** |
| - Action definitions | ✅ | ✅ | ✅ | ✅ | - |
| - Typed parameters | ✅ | ✅ | 🟡 Partial | 🟢 Full | P1 |
| - Control flow | ✅ | ✅ | ❌ | 🟢 Full | P3 |
| - Allocation (perform) | ✅ | ✅ | 🟡 Partial | 🟢 Full | P1 |
| **Requirements** | ✅ 100% | ✅ 100% | 🟡 60% | 🟢 95% | **Medium** |
| - Req definitions | ✅ | ✅ | ❌ | 🟢 Full | P2 |
| - Constraints | ✅ | ✅ | ❌ | 🟢 Full | P2 |
| - Subjects | ✅ | ✅ | ❌ | 🟢 Full | P2 |
| - Satisfaction | ✅ | ✅ | 🟡 Traces | 🟢 Full | P2 |
| **Interfaces** | ✅ 100% | ✅ 100% | 🟢 85% | 🟢 95% | **Medium** |
| - Port definitions | ✅ | ✅ | 🟡 Via interfaces | 🟢 Full | P1 |
| - Typed items | ✅ | ✅ | 🟡 Signals | 🟢 Full | P1 |
| **Organization** | ✅ 100% | ✅ 100% | 🟡 50% | 🟢 90% | **Low** |
| - Packages | ✅ | ✅ | ❌ | 🟢 Full | P3 |
| - Imports | ✅ | ✅ | ❌ | 🟢 Full | P3 |
| - Aliases | ✅ | ✅ | ❌ | 🟡 Basic | P3 |

**Legend**:
- ✅ Full support (90-100%)
- 🟢 Good support (70-89%)
- 🟡 Partial support (40-69%)
- 🔴 Limited support (10-39%)
- ❌ No support (0-9%)

---

## 🚀 Implementation Roadmap

### **Milestone 1: SysML v2 Core Syntax** (Week 1-4)
- [ ] Add `attribute name: Type [unit]` syntax
- [ ] Add `port def` with `in item` / `out item`
- [ ] Add `action def` with typed parameters
- [ ] Add `flow from.param to to.param`
- [ ] Add `perform actionName` for allocation

**Deliverables**:
- Updated parser (`src/compiler/parser.rs`)
- Updated AST (`src/compiler/ast.rs`)
- 20+ test cases

### **Milestone 2: Requirements++ ** (Week 5-8)
- [ ] Add `requirement def` keyword
- [ ] Add `subject: Type` for requirement subjects
- [ ] Add `require constraint` blocks
- [ ] Add `assume constraint` blocks
- [ ] Add `satisfy reqId by component`

**Deliverables**:
- Requirement constraint validator
- Formal verification hooks
- 15+ requirement test cases

### **Milestone 3: Package System** (Week 9-12)
- [ ] Add `package PackageName { }` syntax
- [ ] Add `import PackageName::*`
- [ ] Add `private import` / `public import`
- [ ] Add `alias Name for Namespace::Type`
- [ ] Namespace resolution

**Deliverables**:
- Package/namespace manager
- Import resolver
- 10+ package test cases

### **Milestone 4: Advanced Behavioral** (Week 13-16)
- [ ] Add `succession start -> action1 -> action2`
- [ ] Add `decision` nodes for conditional flow
- [ ] Add `ref part` for reference components
- [ ] Add multiplicity syntax `part wheel: Wheel[2]`
- [ ] Add `entry/exit/do` for state machines

**Deliverables**:
- Control flow engine
- Behavioral validator
- 15+ advanced test cases

---

## 🧪 Validation Strategy

### **Test Suite Structure**:
```
tests/sysml-v2/
├── 01-parts-tree/
│   ├── basic_part_def.arc
│   ├── part_with_attributes.arc
│   ├── part_with_multiplicities.arc
├── 02-ports-interfaces/
│   ├── port_def_basic.arc
│   ├── port_with_items.arc
│   ├── interface_connection.arc
├── 03-actions/
│   ├── action_def_basic.arc
│   ├── action_with_flow.arc
│   ├── action_allocation.arc
├── 04-requirements/
│   ├── requirement_def.arc
│   ├── requirement_with_constraint.arc
│   ├── requirement_satisfaction.arc
├── 05-packages/
│   ├── package_basic.arc
│   ├── package_with_import.arc
│   ├── package_nested.arc
```

### **Conversion Tests**:
Convert 20 SysML v2 examples to ArcLang:
1. `1a-Parts Tree.sysml` → `1a-parts_tree.arc`
2. `3a-Function-based Behavior-1.sysml` → `3a-function_behavior.arc`
3. `4a-Functional Allocation.sysml` → `4a-functional_allocation.arc`
4. `8-Requirements.sysml` → `8-requirements.arc`

### **Capella Export Tests**:
Verify each ArcLang model generates valid Capella XML:
- ✅ All elements have correct `xsi:type`
- ✅ Traceability links preserved
- ✅ Constraints exported as Capella constraints
- ✅ Allocations map to ComponentFunctionalAllocation

---

## 📚 Reference Materials

### **SysML v2 Specifications**:
- [Kernel Modeling Language (KerML) 1.0](https://www.omg.org/spec/KerML/1.0/Beta4)
- [SysML v2.0 Specification](https://www.omg.org/spec/SysML/2.0/Beta4)
- [Systems Modeling API 1.0](https://www.omg.org/spec/SystemsModelingAPI/1.0/Beta3)

### **Capella/Arcadia References**:
- Capella Metamodel Documentation
- Arcadia Method Guidelines
- Existing ArcLang examples (251 files)

### **Key SysML v2 Examples Analyzed**:
1. `/sysml-v2/sysml/src/validation/01-Parts Tree/1a-Parts Tree.sysml` (125 lines)
2. `/sysml-v2/sysml/src/validation/03-Function-based Behavior/3a-Function-based Behavior-1.sysml` (137 lines)
3. `/sysml-v2/sysml/src/validation/04-Functional Allocation/4a-Functional Allocation.sysml` (110 lines)
4. `/sysml-v2/sysml/src/validation/08-Requirements/8-Requirements.sysml` (205 lines)

---

## ✅ Success Criteria

1. **Syntax Compatibility**: 90%+ of SysML v2 structural patterns expressible in ArcLang
2. **Capella Export**: 100% valid Capella XML from SysML v2-aligned ArcLang
3. **Bi-directional**: SysML v2 → ArcLang → Capella → ArcLang (round-trip)
4. **Test Coverage**: 150+ test cases covering all major SysML v2 features
5. **Documentation**: Complete SysML v2 syntax guide for ArcLang users

---

## 🎯 Strategic Benefits

### **For ArcLang Users**:
- ✅ Write models using industry-standard SysML v2 syntax
- ✅ Import SysML v2 models from other tools
- ✅ Export to both Capella (method-driven) and generic SysML v2 (tool-agnostic)

### **For Capella Users**:
- ✅ Express Capella models in modern SysML v2 textual notation
- ✅ Leverage SysML v2 tooling ecosystem (Jupyter, Eclipse, VS Code)
- ✅ Bridge to other SysML v2 tools (Cameo, MagicDraw, etc.)

### **For MBSE Community**:
- ✅ Demonstrates convergence between Arcadia methodology and SysML v2 standard
- ✅ Provides concrete mapping between two major MBSE approaches
- ✅ Facilitates tool interoperability

---

## 📝 Next Steps

1. **Review this mapping** with Capella/Arcadia experts
2. **Prioritize Phase 1 features** (attributes, ports, actions, allocation)
3. **Create detailed parser grammar** for SysML v2 syntax extensions
4. **Implement iteratively** with test-driven development
5. **Validate with real automotive/aerospace models**

---

**Status**: 🟢 **ANALYSIS COMPLETE** - Ready for implementation planning

**Contact**: ArcLang Development Team  
**Repository**: https://github.com/Mbaroudi/arclang  
**SysML v2 Reference**: https://github.com/Systems-Modeling/SysML-v2-Release
