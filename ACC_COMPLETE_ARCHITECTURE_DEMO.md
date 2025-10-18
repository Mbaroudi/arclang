# Complete ACC Architecture - Capella-Style Visualization Demo

## Overview

This document demonstrates ArcViz's **complete Capella-style system architecture visualization** with components, connectors, and full traceability.

## Files

- **Source**: `examples/automotive/acc_complete_architecture.arc`
- **Visualization**: `acc_complete_with_components.html`
- **Requirements View**: `acc_complete_architecture.html` (original)

## Architecture Contents

### System Layers

The ACC system is organized into **4 architectural layers**:

```
┌─────────────────────────────────────────────────────────────┐
│  Perception Layer                                            │
│  ├── LC-001: Long Range Radar                               │
│  └── LC-002: Forward Camera                                 │
├─────────────────────────────────────────────────────────────┤
│  Processing Layer                                            │
│  ├── LC-003: Sensor Fusion                                  │
│  └── LC-004: Target Selection                               │
├─────────────────────────────────────────────────────────────┤
│  Control Layer                                               │
│  ├── LC-005: Longitudinal Controller                        │
│  └── LC-006: Actuator Command                               │
├─────────────────────────────────────────────────────────────┤
│  Safety & Monitoring Layer                                   │
│  ├── LC-007: Safety Monitor                                 │
│  ├── LC-008: Driver Interface (HMI)                         │
│  └── LC-009: Override Manager                               │
└─────────────────────────────────────────────────────────────┘
```

### Components (9 Total)

#### 1. **LC-001: Long Range Radar**
- **Category**: Perception
- **Functions**: 3
  - LF-001: Transmit RF Signal → `rf_signal: RadarWaveform`
  - LF-002: Receive Echoes → `raw_echoes: SignalArray`
  - LF-003: Process Radar Data → `radar_targets: TargetList`

#### 2. **LC-002: Forward Camera**
- **Category**: Perception
- **Functions**: 3
  - LF-004: Capture Image → `raw_image: ImageFrame`
  - LF-005: Detect Objects → `detected_objects: ObjectList`
  - LF-006: Detect Lanes → `lane_data: LaneInfo`

#### 3. **LC-003: Sensor Fusion**
- **Category**: Processing
- **Functions**: 3
  - LF-007: Correlate Detections
  - LF-008: Track Objects
  - LF-009: Predict Trajectories

#### 4. **LC-004: Target Selection**
- **Category**: Processing
- **Functions**: 2
  - LF-010: Identify Lead Vehicle
  - LF-011: Detect Cut-In

#### 5. **LC-005: Longitudinal Controller**
- **Category**: Control
- **Functions**: 3
  - LF-012: Calculate Time Gap
  - LF-013: Determine Desired Speed
  - LF-014: Compute Acceleration

#### 6. **LC-006: Actuator Command**
- **Category**: Control
- **Functions**: 2
  - LF-015: Map to Throttle
  - LF-016: Map to Brake

#### 7. **LC-007: Safety Monitor**
- **Category**: Safety
- **Functions**: 3
  - LF-017: Check Sensor Health
  - LF-018: Enforce Limits
  - LF-019: Detect Faults

#### 8. **LC-008: Driver Interface**
- **Category**: HMI
- **Functions**: 3
  - LF-020: Read Driver Inputs
  - LF-021: Display Status
  - LF-022: Issue Warnings

#### 9. **LC-009: Override Manager**
- **Category**: Safety
- **Functions**: 3
  - LF-023: Detect Brake Pedal
  - LF-024: Detect Accelerator Pedal
  - LF-025: Override Control

### Data Flow Connectors

The visualization shows **8 primary data flow paths**:

```
Radar (LC-001) ───────────┐
                          ▼
Camera (LC-002) ─────► Sensor Fusion (LC-003)
                          │
                          ▼
                     Target Selection (LC-004)
                          │
                          ▼
                  Longitudinal Controller (LC-005)
                          │
                          ▼
                   Actuator Command (LC-006)
                          │
         ┌────────────────┼────────────────┐
         ▼                ▼                ▼
  Safety Monitor   Driver Interface   Override Manager
    (LC-007)          (LC-008)           (LC-009)
```

### Requirements Coverage (7 Requirements)

1. **SYS-ACC-001** (ASIL-B) - 2-second following distance
2. **SYS-ACC-002** (ASIL-B) - Cut-in detection < 500ms
3. **SYS-ACC-003** (ASIL-B) - Max deceleration 3.5 m/s²
4. **SYS-ACC-004** (ASIL-C) - Brake override < 100ms
5. **SYS-ACC-005** (ASIL-A) - Speed range 30-180 km/h
6. **SYS-ACC-006** (ASIL-A) - Visual & audible warnings
7. **SYS-ACC-007** (ASIL-B) - Continuous self-diagnostics

## Visual Features

### Capella-Style Elements

✅ **Component Boxes**
- Blue gradient fill (#e3f2fd)
- Strong border (3px, #1976d2)
- Rounded corners (8px radius)
- Professional drop shadows

✅ **Layer Organization**
- Clear layer labels
- Top-down data flow
- Logical grouping

✅ **Function Visualization**
- Orange function boxes inside components
- Input/output port indicators
- Compact representation

✅ **Connectors**
- Blue arrows (#1976d2)
- 2px stroke width
- Directional arrowheads
- Clean routing

✅ **Typography**
- Segoe UI for labels
- Consolas monospace for IDs
- Professional hierarchy

### Interactive Features

🔍 **Zoom In/Out** - Mouse wheel or buttons
🖱️ **Pan & Drag** - Click and drag to navigate
↻ **Reset View** - Return to initial position
💾 **Export SVG** - Save standalone vector graphic

## Comparison with Capella Studio

| Feature | ArcViz | Capella Studio |
|---------|--------|----------------|
| **Component Boxes** | ✅ Auto-generated | ❌ Manual drawing |
| **Layered Architecture** | ✅ Code-defined | ❌ Manual layout |
| **Data Flow Connectors** | ✅ From traces | ❌ Manual connections |
| **Port Definitions** | ✅ In ArcLang code | ✅ In model |
| **Function Allocation** | ✅ Nested in components | ✅ Graphical nesting |
| **Traceability** | ✅ Automatic | ❌ Manual linking |
| **Generation Time** | ✅ < 1 second | ❌ Hours of modeling |
| **File Format** | ✅ Single HTML | ❌ Eclipse workspace |
| **Version Control** | ✅ Text-based | ❌ Binary XML |
| **CI/CD Integration** | ✅ CLI command | ❌ GUI-based |

## How It Works

### 1. Define Architecture in ArcLang

```arc
logical_architecture "ACC Logical Architecture" {
    component "Long Range Radar" {
        id: "LC-001"
        type: "Logical"
        category: "Perception"
        
        function "Process Radar Data" {
            id: "LF-003"
            inputs: ["raw_echoes: SignalArray"]
            outputs: ["radar_targets: TargetList"]
        }
    }
}

// Component data flow
trace "LC-001" implements "LC-003" {
    rationale: "Radar provides target data to sensor fusion"
}
```

### 2. Generate Visualization

```bash
arclang export examples/automotive/acc_complete_architecture.arc \
    -o acc_complete_with_components.html \
    -f arc-viz
```

### 3. Open in Browser

```bash
open acc_complete_with_components.html
```

## Key Innovations

### 1. **Automatic Layout**
- Components placed by layer
- No manual positioning required
- Consistent spacing

### 2. **Trace-Driven Connectors**
- Data flow extracted from `trace` statements
- Automatic arrow routing
- Maintains semantic consistency

### 3. **Dual View Support**
- Requirements view (categories, priorities, ASIL)
- Architecture view (components, connectors, layers)
- Automatically selects appropriate view

### 4. **Port-Based Communication**
- Functions declare inputs/outputs
- Typed data interfaces
- Clear data dependencies

### 5. **Safety Annotations**
- ASIL badges on requirements
- Safety components clearly marked
- Override paths visible

## Use Cases

### ✅ ISO 26262 Compliance
- Component-level safety requirements
- Traceability to system requirements
- ASIL decomposition visualization

### ✅ Design Reviews
- Architecture overview in minutes
- Share HTML file via email
- Interactive exploration during meetings

### ✅ Documentation Generation
- Embed in Confluence/wikis
- PDF export capability
- Always up-to-date with code

### ✅ Impact Analysis
- Visualize data dependencies
- Trace requirement changes
- Identify affected components

### ✅ New Engineer Onboarding
- Visual system overview
- Clear layer responsibilities
- Interactive learning

## Future Enhancements

### Phase 1 (Current) ✅
- [x] Component boxes with IDs
- [x] Layered architecture
- [x] Basic connectors
- [x] Function indicators

### Phase 2 (Next)
- [ ] Detailed port visualization
- [ ] Data type annotations on connectors
- [ ] Component state machines
- [ ] Timing annotations

### Phase 3 (Future)
- [ ] Physical architecture view
- [ ] Deployment diagrams
- [ ] Resource allocation
- [ ] Performance metrics

## Conclusion

**ArcViz now provides Capella-quality system architecture visualizations** with:

✅ Professional Capella-style component boxes and styling
✅ Layered architectural organization
✅ Component-to-component connectors with arrows
✅ Function and port representation
✅ Complete traceability from requirements to implementation
✅ Single-command generation from text-based models
✅ Interactive HTML output with zoom/pan controls
✅ No external dependencies or complex tool setup

This demonstrates that **ArcLang + ArcViz** can serve as a **lightweight, code-first alternative to Eclipse Capella** for systems engineering, while maintaining professional visualization quality and full MBSE capabilities.

---

**Generated with**: ArcLang v1.0.0 + ArcViz
**Date**: 2025-10-18
**Example**: Adaptive Cruise Control (ACC) Complete Architecture
