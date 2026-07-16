# Quick Reference - Integrated MBSE Features

**ArcViz Platform - Feature Integration v1.0**

---

## 🔴 Safety Colors

### Automotive (ISO 26262)
```arc
Component BrakeECU {
  safety_level = "ASIL_D"  // Red 6px border
  safety_level = "ASIL_C"  // Orange 5px border
  safety_level = "ASIL_B"  // Orange 4px border
  safety_level = "ASIL_A"  // Yellow 3px border
  safety_level = "QM"      // Gray 2px border
}
```

### Aerospace (DO-178C)
```arc
Component FlightControl {
  dal = "DAL_A"  // Red 6px border (most critical)
  dal = "DAL_B"  // Orange 5px border
  dal = "DAL_C"  // Orange 4px border
  dal = "DAL_D"  // Yellow 3px border
  dal = "DAL_E"  // Gray 2px border
}
```

### Industrial (IEC 61508)
```arc
Component SafetyPLC {
  sil = "SIL_4"  // Red 6px border (highest)
  sil = "SIL_3"  // Orange 5px border
  sil = "SIL_2"  // Orange 4px border
  sil = "SIL_1"  // Yellow 3px border
  sil = "SIL_0"  // Gray 2px border
}
```

**Visual**: Color-coded borders on components and physical nodes

---

## 🔗 Traceability Links

### Active Link Types

#### allocates (Function → Component)
```arc
Component Controller {
  allocated_functions = ["SF-001", "SF-002"]
}
// Renders: Orange solid line, 2.5px width
```

#### realizes (Mission → Capability)
```arc
Mission AutoBraking {
  capabilities = ["C1"]
}
CapabilityRealization EmergencyStop {
  id = "C1"
  realized_capability_id = "C1"
}
// Renders: Gray-blue dashed line (8,4), 2px width
```

### Future Link Types (Implemented, Not Yet Integrated)
- `implements` - Component → Requirement (blue dashed)
- `satisfies` - Architecture → Requirement (green dashed)
- `derives` - Requirement → Requirement (brown dashed)
- `justifies` - Decision → Requirement (pink dashed)
- `verifies` - Test → Requirement (cyan dashed)
- `refines` - SA→LA, LA→PA (purple dotted)
- `traces` - Generic traceability (gray dashed)

---

## ⚡ Exchange Items

### All 6 Types

#### EVENT - Signal with no data
```arc
FunctionalExchange CollisionAlert {
  from_port = "Sensor.out"
  to_port = "Controller.in"
  exchange_item_kind = "EVENT"
  label = "Alert"
}
// Visual: ⚡ Red (#FF6B6B), dashed (5,5), open arrow
```

#### FLOW - Continuous stream
```arc
FunctionalExchange SensorStream {
  exchange_item_kind = "FLOW"
  label = "ContinuousData"
}
// Visual: ⟿ Cyan (#4ECDC4), solid thick (3px), filled arrow
```

#### OPERATION - Request/response
```arc
FunctionalExchange ServiceCall {
  exchange_item_kind = "OPERATION"
  label = "Calculate"
}
// Visual: ↔ Teal (#95E1D3), double line, standard arrow
```

#### DATA - Structured package (DEFAULT)
```arc
FunctionalExchange DataPacket {
  exchange_item_kind = "DATA"
  label = "Telemetry"
}
// Visual: 📦 Blue (#5B9BD5), solid (2px), standard arrow
```

#### SHARED_DATA - Repository access
```arc
FunctionalExchange SharedConfig {
  exchange_item_kind = "SHARED_DATA"
  label = "ConfigDB"
}
// Visual: 🗄 Purple (#9B59B6), dotted (2,3), diamond arrow
```

#### UNSET - Generic/unknown
```arc
FunctionalExchange Generic {
  exchange_item_kind = "UNSET"
  label = "Unknown"
}
// Visual: → Gray (#95A5A6), thin solid, standard arrow
```

**Note**: If `exchange_item_kind` not specified, defaults to DATA

---

## ○⌒ Interface Notation

### Provided Interfaces (Lollipops)
```arc
Component DataProducer {
  interfaces_out = [
    "IData",
    "IStatus"
  ]
}
// Visual: White circles (○) on RIGHT side, 20px line + 6px radius
```

### Required Interfaces (Sockets)
```arc
Component DataConsumer {
  interfaces_in = [
    "IData",
    "IConfig"
  ]
}
// Visual: Semicircular arcs (⌒) on LEFT side, 20px line
```

### Combined Example
```arc
Component Processor {
  interfaces_out = ["IProcessedData", "IQuality"]
  interfaces_in = ["IRawData", "ICalibration"]
}
// RIGHT side: 2 lollipops (provided)
// LEFT side: 2 sockets (required)
```

---

## 🎨 Complete Example

```arc
// System Analysis - Exchange Items
SystemAnalysis EmergencyBraking {
  Function DetectCollision {
    id = "SF-001"
    name = "Detect Collision Risk"
    ports_out = ["riskAssessment"]
  }
  
  Function PlanBraking {
    id = "SF-002"
    name = "Plan Braking Strategy"
    ports_in = ["riskData"]
  }
  
  FunctionalExchange RiskData {
    from_port = "SF-001.riskAssessment"
    to_port = "SF-002.riskData"
    exchange_item_kind = "FLOW"  // Cyan thick line ⟿
    label = "ContinuousRisk"
  }
}

// Logical Architecture - Safety + Interfaces + Traceability
LogicalArchitecture EmergencyBraking {
  Component PerceptionUnit {
    id = "LC-001"
    name = "Perception Unit"
    safety_level = "ASIL_C"  // Orange 5px border
    allocated_functions = ["SF-001"]  // Orange allocation line
    interfaces_out = ["IRiskData"]  // Lollipop ○
    interfaces_in = ["ISensorInput"]  // Socket ⌒
  }
  
  Component BrakeController {
    id = "LC-002"
    name = "Brake Controller"
    safety_level = "ASIL_D"  // Red 6px border
    allocated_functions = ["SF-002"]  // Orange allocation line
    interfaces_out = ["IBrakeCmd"]  // Lollipop ○
    interfaces_in = ["IRiskData"]  // Socket ⌒
  }
}

// Physical Architecture - Safety Colors
PhysicalArchitecture EmergencyBraking {
  PhysicalNode BrakeECU {
    id = "PN-001"
    name = "Brake ECU"
    node_type = "Hardware"
    safety_level = "ASIL_D"  // Red 6px on 3D cube
  }
}
```

**Result**:
- Functional exchanges show cyan flow arrows with ⟿ icon
- Components have orange/red safety borders
- Allocation links are orange solid lines
- Interfaces show lollipops and sockets
- Physical nodes have red 3D borders

---

## 📊 Where Each Feature Works

| Feature | Renderers | Diagram Types |
|---------|-----------|---------------|
| Safety Colors | component.ts, physical.ts | LAB, PAB |
| Traceability | allocation.ts, missions-capabilities.ts | Allocation, MCB |
| Exchange Items | dataflow.ts, functional.ts | SDFB, Functional |
| Interfaces | component.ts | LAB |

**LAB** = Logical Architecture Blank  
**PAB** = Physical Architecture Blank  
**SDFB** = System Dataflow Blank  
**MCB** = Missions & Capabilities Blank

---

## 🔍 How to Check If Features Are Working

### Safety Colors
Look for: Colored borders (red, orange, yellow) on component boxes and physical nodes

### Traceability
Look for: Colored/dashed lines between functions and components (orange) or missions and capabilities (gray-blue)

### Exchange Items
Look for: Icon prefixes (⚡⟿↔📦🗄) on edge labels and colored lines matching exchange type

### Interfaces
Look for: White circles (○) on right side and semicircular arcs (⌒) on left side of components

---

## ⚠️ Common Issues

### Safety colors not showing?
- Check: `safety_level`, `asil`, `dal`, or `sil` in component/node metadata
- Valid values: ASIL_A through ASIL_D, DAL_A through DAL_E, SIL_0 through SIL_4

### Exchange items all look the same?
- Check: `exchange_item_kind` field in FunctionalExchange
- Valid values: EVENT, FLOW, OPERATION, DATA, SHARED_DATA, UNSET
- Default: DATA (if not specified)

### Interfaces not visible?
- Check: `interfaces_out` and `interfaces_in` arrays in Component
- Ensure arrays are not empty
- Interfaces only work in component.ts (LAB diagrams)

### Traceability links missing?
- Check: `allocated_functions` array in Component (for allocation links)
- Check: Mission capabilities array and CapabilityRealization (for realizes links)
- Only works in allocation.ts and missions-capabilities.ts

---

## 📚 Documentation References

- **FEATURE_INTEGRATION_COMPLETE.md** - Complete usage guide
- **INTEGRATION_TEST_PLAN.md** - Test cases and validation
- **INTEGRATION_SUMMARY_EXECUTIVE.md** - Executive overview

---

**Version**: 1.0.0 (Feature Integration Release)  
**Last Updated**: October 31, 2025
