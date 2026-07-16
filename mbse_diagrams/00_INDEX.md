# Emergency Braking System - MBSE Diagrams Portfolio

**Professional Capella-style diagrams for Emergency Braking System**

Generated: October 29, 2025

---

## 📊 Diagram Inventory

### 1. **Operational Analysis** ✅ (Dimensions 1-2)
*Operational activities and entities*
- File: `01_operational.html`
- Status: Generated (previous session)

### 2. **System Analysis - Functional** ✅ (Dimension 3)
*System functions and data flows*
- File: `02_functional.html`
- Status: Generated (previous session)

### 3. **Logical Architecture** ✅ (Dimension 4)
*Logical components with function allocation*
- **File: `03_logical_professional.html`** ⭐
- Features:
  - Component hierarchy with nested sub-components
  - Function allocation indicators
  - Professional pastel colors
  - Ports and interfaces
  - Stereotypes (<<sensor>>, <<controller>>, <<actuator>>)
- Size: 17KB, 2600×1800px

### 4. **Physical Architecture** ✅ (Dimension 5)
*Physical components and deployment*
- **File: `04_physical_professional.html`** ⭐
- Features:
  - Hardware and software components
  - Component allocation chains
  - Real-world component types
  - Professional styling
- Size: 17KB, 2600×1800px

### 5. **Dataflow Diagram** ✅ NEW!
*Information dependencies between functions*
- **File: `05_dataflow_professional.html`** ⭐
- Features:
  - 9 functions, 11 functional exchanges
  - Semantic color coding by category
  - Data rates and protocols (1 Hz - 100 Hz)
  - Bidirectional feedback loop (SF-006 ↔ SF-007)
  - Intelligent legend positioning
  - Geometric midpoint edge labels
- Size: 16KB, 2840×524px

### 6. **Function Tree Diagram** ✅ NEW!
*Hierarchical function breakdown*
- **File: `06_function_tree_interactive.html`** ⭐
- Features:
  - 9 parent functions, 30 sub-functions
  - **Interactive fold/unfold** with JavaScript
  - Vertical hierarchical layout (Capella style)
  - Orthogonal connections
  - Click ⊞/⊟ to expand/collapse
  - Capella pastel colors
- Size: 16KB, 1400×2800px

### 7. **Sequence Diagram - Functional Scenario** ✅ NEW!
*Function-to-function message flow*
- **File: `07_sequence_functional.svg`** ⭐
- Features:
  - 8 function lifelines
  - 10 messages (sync, async, return)
  - Self-calls (calculateTTC, displayAlert)
  - Activation bars
  - Timeline layout
- Size: 8.8KB, 2440×880px
- Scenario: Emergency braking activation sequence

### 8. **Sequence Diagram - Exchange Scenario** ✅ NEW!
*Component/Actor exchange interactions*
- **File: `08_sequence_exchange.svg`** ⭐
- Features:
  - 7 participants (5 components, 2 actors)
  - 12 component exchanges (CE-001 to CE-008)
  - Combined fragment: ALT (collision imminent vs safe)
  - Actor interactions (Driver, Brake ECU)
  - CAN message exchanges
- Size: 9.2KB, 2120×1000px
- Scenario: Component-level emergency braking

### 9. **Sequence Diagram - Interface Scenario** ✅ NEW!
*Interface operations between components*
- **File: `09_sequence_interface.svg`** ⭐
- Features:
  - 5 component lifelines
  - 12 interface operation calls
  - Interface notation (ISensor, IPerception, IBrake, IHMI)
  - Combined fragment: LOOP (every 50ms)
  - Timing constraint (< 100ms)
  - Component stereotypes
- Size: 8.4KB, 1480×1000px
- Scenario: Interface-level operations

---

## 🎯 Capella Diagram Types Coverage

| Diagram Type | Status | Files |
|--------------|--------|-------|
| **Operational Activity Breakdown (OAB)** | ✅ | 01_operational.html |
| **System Dataflow (SDFB)** | ✅ | 05_dataflow_professional.html |
| **Functional Breakdown (FBD)** | ✅ | 02_functional.html |
| **Functional Tree** | ✅ | 06_function_tree_interactive.html |
| **Logical Architecture (LAB)** | ✅ | 03_logical_professional.html |
| **Physical Architecture (PAB)** | ✅ | 04_physical_professional.html |
| **Sequence - Functional Scenario (ES)** | ✅ | 07_sequence_functional.svg |
| **Sequence - Exchange Scenario (ES)** | ✅ | 08_sequence_exchange.svg |
| **Sequence - Interface Scenario (IS)** | ✅ | 09_sequence_interface.svg |

---

## 🔧 Technical Features Implemented

### Visual Quality
- ✅ Professional Capella pastel colors
- ✅ Semantic color coding by category/type
- ✅ Component stereotypes (<<sensor>>, <<controller>>, etc.)
- ✅ Multiline text wrapping
- ✅ Intelligent layout algorithms (ELK hierarchical)

### Interactivity
- ✅ **Tree diagram fold/unfold** (JavaScript)
- ✅ Click handlers for expand/collapse
- ✅ State management

### Layout & Positioning
- ✅ Intelligent legend positioning (no overlap)
- ✅ Bidirectional edge separation
- ✅ Geometric midpoint calculation for labels
- ✅ Orthogonal edge routing
- ✅ Vertical hierarchical tree layout

### Sequence Diagrams
- ✅ Lifelines (functions, components, actors)
- ✅ Message types (sync, async, return)
- ✅ Activation bars
- ✅ Combined fragments (ALT, LOOP, OPT, PAR)
- ✅ Self-calls
- ✅ Timing constraints

---

## 📁 File Sizes

| File | Size | Dimensions |
|------|------|------------|
| 03_logical_professional.html | 17KB | 2600×1800px |
| 04_physical_professional.html | 17KB | 2600×1800px |
| 05_dataflow_professional.html | 16KB | 2840×524px |
| 06_function_tree_interactive.html | 16KB | 1400×2800px |
| 07_sequence_functional.svg | 8.8KB | 2440×880px |
| 08_sequence_exchange.svg | 9.2KB | 2120×1000px |
| 09_sequence_interface.svg | 8.4KB | 1480×1000px |

---

## 🎨 System Overview

**Emergency Braking System** - A complete MBSE model demonstrating:

### Functions (9 top-level)
1. **SF-001**: Sensor Data Acquisition (Input)
2. **SF-002**: Multi-Sensor Fusion (Processing)
3. **SF-003**: Object Detection & Classification (Perception)
4. **SF-004**: Multi-Object Tracking (Perception)
5. **SF-005**: Collision Risk Assessment (Decision)
6. **SF-006**: Braking Strategy Planning (Planning)
7. **SF-007**: Brake Actuation (Control)
8. **SF-008**: Driver Warning (HMI)
9. **SF-009**: System Health Monitoring (Safety)

### Components (7 logical)
1. **LC-001**: Sensor Array Controller
2. **LC-002**: Perception Unit
3. **LC-003**: Decision Controller
4. **LC-004**: Braking Controller
5. **LC-005**: HMI Controller
6. **LC-006**: Safety Monitor
7. **LC-007**: System Manager

### Data Flow
- 11 functional exchanges
- Data rates: 1 Hz to 100 Hz
- Protocols: CAN, FlexRay, Ethernet
- Bidirectional feedback loop

---

## 🚀 Next Steps (Optional Enhancements)

- [ ] Mode/State machines on sequence diagrams
- [ ] Capability diagrams
- [ ] Requirements traceability matrix
- [ ] Class diagrams for data types
- [ ] State machine diagrams
- [ ] Mission/Capability diagrams

---

**Generated with ArcViz Diagram Service**  
Professional MBSE diagrams following Capella methodology
