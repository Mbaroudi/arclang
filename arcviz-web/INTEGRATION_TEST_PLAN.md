# Integration Test Plan - Feature Validation

**Purpose**: Validate that all 4 integrated features render correctly in actual diagrams

---

## Test 1: Safety Colors Integration

### Test Case 1.1: Logical Component with ASIL_D
**File**: `test_safety_asil_d.arc`

```arc
LogicalArchitecture SafetyTest {
  Component BrakeController {
    id = "LC-001"
    name = "Emergency Brake Controller"
    component_type = "Logical"
    safety_level = "ASIL_D"
    interfaces_out = ["IBrakeCommand"]
    interfaces_in = ["ICollisionData"]
  }
  
  Component SensorFusion {
    id = "LC-002"
    name = "Sensor Fusion Unit"
    component_type = "Logical"
    safety_level = "ASIL_C"
    interfaces_out = ["IFusedData"]
    interfaces_in = ["ISensorRaw"]
  }
  
  Component DisplayUnit {
    id = "LC-003"
    name = "Driver Display"
    component_type = "Logical"
    interfaces_in = ["IWarningData"]
  }
}
```

**Expected Result**:
- LC-001: Red border, 6px width (ASIL_D)
- LC-002: Orange border, 5px width (ASIL_C)
- LC-003: Black border, 2px width (no safety level)

---

### Test Case 1.2: Physical Node with DAL_A
**File**: `test_safety_dal_a.arc`

```arc
PhysicalArchitecture AerospaceSystem {
  PhysicalNode FlightControlECU {
    id = "PN-001"
    name = "Flight Control Computer"
    node_type = "Hardware"
    safety_level = "DAL_A"
  }
  
  PhysicalNode NavigationSoftware {
    id = "PN-002"
    name = "Navigation Software"
    node_type = "Software"
    dal = "DAL_B"
  }
  
  PhysicalNode DisplayDriver {
    id = "PN-003"
    name = "Display Driver"
    node_type = "Software"
  }
}
```

**Expected Result**:
- PN-001: Red border, 6px width, 3D cube appearance (DAL_A)
- PN-002: Orange border, 5px width (DAL_B)
- PN-003: Black border, 2px width (no safety level)

---

## Test 2: Traceability Styles Integration

### Test Case 2.1: Function-to-Component Allocation
**File**: `test_traceability_allocation.arc`

```arc
SystemAnalysis EmergencyBraking {
  Function DetectObstacle {
    id = "SF-001"
    name = "Detect Obstacle"
    category = "Perception"
  }
  
  Function AssessRisk {
    id = "SF-002"
    name = "Assess Collision Risk"
    category = "Decision"
  }
}

LogicalArchitecture EmergencyBraking {
  Component PerceptionUnit {
    id = "LC-001"
    name = "Perception Unit"
    allocated_functions = ["SF-001"]
  }
  
  Component DecisionController {
    id = "LC-002"
    name = "Decision Controller"
    allocated_functions = ["SF-002"]
  }
}
```

**Expected Result**:
- SF-001 → LC-001: Orange solid line, 2.5px, arrow marker, "allocated to" label
- SF-002 → LC-002: Orange solid line, 2.5px, arrow marker, "allocated to" label
- Labels have orange border and fill

---

### Test Case 2.2: Mission-to-Capability Realization
**File**: `test_traceability_realizes.arc`

```arc
Mission AutoBraking {
  id = "M1"
  name = "Automatic Emergency Braking"
  capabilities = ["C1", "C2"]
}

Mission AdaptiveCruise {
  id = "M2"
  name = "Adaptive Cruise Control"
  capabilities = ["C2", "C3"]
}

CapabilityRealization EmergencyStop {
  id = "C1"
  name = "Emergency Stop Capability"
  realized_capability_id = "C1"
}

CapabilityRealization SpeedControl {
  id = "C2"
  name = "Speed Control Capability"
  realized_capability_id = "C2"
}

CapabilityRealization DistanceMaintain {
  id = "C3"
  name = "Distance Maintenance"
  realized_capability_id = "C3"
}
```

**Expected Result**:
- M1 → C1: Gray-blue dashed line (8,4), 2px, "realizes" label
- M1 → C2: Gray-blue dashed line (8,4), 2px, "realizes" label
- M2 → C2: Gray-blue dashed line (8,4), 2px, "realizes" label
- M2 → C3: Gray-blue dashed line (8,4), 2px, "realizes" label
- Labels have gray-blue border

---

## Test 3: Exchange Item Visualization

### Test Case 3.1: All Exchange Types in Dataflow
**File**: `test_exchange_items_all_types.arc`

```arc
SystemAnalysis ExchangeTypesDemo {
  Function EventSource {
    id = "SF-001"
    name = "Event Source"
  }
  
  Function EventHandler {
    id = "SF-002"
    name = "Event Handler"
  }
  
  Function DataProducer {
    id = "SF-003"
    name = "Data Producer"
  }
  
  Function DataConsumer {
    id = "SF-004"
    name = "Data Consumer"
  }
  
  Function ServiceProvider {
    id = "SF-005"
    name = "Service Provider"
  }
  
  Function ServiceClient {
    id = "SF-006"
    name = "Service Client"
  }
  
  FunctionalExchange AlertEvent {
    from = "SF-001"
    to = "SF-002"
    exchange_item_kind = "EVENT"
    label = "Alert"
    data = "AlertSignal"
  }
  
  FunctionalExchange SensorStream {
    from = "SF-003"
    to = "SF-004"
    exchange_item_kind = "FLOW"
    label = "SensorData"
    data = "ContinuousStream"
  }
  
  FunctionalExchange ServiceCall {
    from = "SF-006"
    to = "SF-005"
    exchange_item_kind = "OPERATION"
    label = "Calculate"
    data = "CalculateResult(input)"
  }
  
  FunctionalExchange DataPacket {
    from = "SF-003"
    to = "SF-002"
    exchange_item_kind = "DATA"
    label = "Telemetry"
    data = "TelemetryPacket"
  }
  
  FunctionalExchange SharedAccess {
    from = "SF-001"
    to = "SF-004"
    exchange_item_kind = "SHARED_DATA"
    label = "ConfigDB"
    data = "SharedConfiguration"
  }
}
```

**Expected Result**:
- AlertEvent: Red (#FF6B6B), dashed (5,5), open arrow, "⚡ Alert (event)"
- SensorStream: Cyan (#4ECDC4), solid thick (3px), filled arrow, "⟿ SensorData (flow)"
- ServiceCall: Teal (#95E1D3), double line, standard arrow, "↔ Calculate()"
- DataPacket: Blue (#5B9BD5), solid (2px), standard arrow, "📦 Telemetry"
- SharedAccess: Purple (#9B59B6), dotted (2,3), diamond arrow, "🗄 ConfigDB (shared)"

---

### Test Case 3.2: Functional Exchange with Event Type
**File**: `test_exchange_functional.arc`

```arc
SystemAnalysis EmergencyBraking {
  Function RadarSensor {
    id = "SF-001"
    name = "Radar Sensor"
    category = "Input"
    ports_out = ["obstacleDetected"]
  }
  
  Function CollisionDetector {
    id = "SF-002"
    name = "Collision Detector"
    category = "Processing"
    ports_in = ["sensorEvent"]
    ports_out = ["collisionAlert"]
  }
  
  Function WarningSystem {
    id = "SF-003"
    name = "Warning System"
    category = "HMI"
    ports_in = ["alertEvent"]
  }
  
  FunctionalExchange ObstacleDetected {
    from_port = "SF-001.obstacleDetected"
    to_port = "SF-002.sensorEvent"
    exchange_item_kind = "EVENT"
    label = "ObstacleDetected"
    data_type = "ObstacleEvent"
  }
  
  FunctionalExchange CollisionAlert {
    from_port = "SF-002.collisionAlert"
    to_port = "SF-003.alertEvent"
    exchange_item_kind = "EVENT"
    label = "CollisionImminent"
    data_type = "AlertEvent"
  }
}
```

**Expected Result**:
- Both exchanges render with red dashed lines
- Labels show "⚡ ObstacleDetected (event)" and "⚡ CollisionImminent (event)"
- Open arrow heads
- Red label borders

---

## Test 4: Interface Notation Integration

### Test Case 4.1: Component with Multiple Interfaces
**File**: `test_interfaces_multiple.arc`

```arc
LogicalArchitecture InterfaceDemo {
  Component SensorArray {
    id = "LC-001"
    name = "Multi-Sensor Array"
    component_type = "Logical"
    interfaces_out = [
      "ISensorData",
      "IStatus",
      "IDiagnostics"
    ]
    interfaces_in = [
      "IConfiguration",
      "IPowerControl"
    ]
  }
  
  Component DataProcessor {
    id = "LC-002"
    name = "Data Processing Unit"
    component_type = "Logical"
    interfaces_out = [
      "IProcessedData",
      "IQuality"
    ]
    interfaces_in = [
      "ISensorData",
      "ICalibration"
    ]
  }
  
  ComponentExchange SensorToProcessor {
    from_port = "LC-001.ISensorData"
    to_port = "LC-002.ISensorData"
    label = "RawSensorData"
  }
}
```

**Expected Result**:
- LC-001 RIGHT side: 3 lollipops (white circles) for ISensorData, IStatus, IDiagnostics
- LC-001 LEFT side: 2 sockets (arcs) for IConfiguration, IPowerControl
- LC-002 RIGHT side: 2 lollipops for IProcessedData, IQuality
- LC-002 LEFT side: 2 sockets for ISensorData, ICalibration
- All interfaces evenly spaced on component boundaries
- Interface labels positioned next to symbols
- Connection line between matching interfaces

---

## Test 5: Combined Features Test

### Test Case 5.1: Safety + Interfaces + Exchanges
**File**: `test_combined_all_features.arc`

```arc
SystemAnalysis EmergencyBrakingComplete {
  Function DetectCollision {
    id = "SF-001"
    name = "Detect Collision Risk"
    category = "Perception"
    ports_out = ["riskAssessment"]
  }
  
  Function PlanBraking {
    id = "SF-002"
    name = "Plan Braking Strategy"
    category = "Planning"
    ports_in = ["riskData"]
    ports_out = ["brakeCommand"]
  }
  
  FunctionalExchange RiskData {
    from_port = "SF-001.riskAssessment"
    to_port = "SF-002.riskData"
    exchange_item_kind = "FLOW"
    label = "ContinuousRiskAssessment"
    data_type = "RiskLevel"
  }
}

LogicalArchitecture EmergencyBrakingComplete {
  Component PerceptionUnit {
    id = "LC-001"
    name = "Perception Unit"
    component_type = "Logical"
    safety_level = "ASIL_C"
    allocated_functions = ["SF-001"]
    interfaces_out = ["IRiskData"]
    interfaces_in = ["ISensorInput"]
  }
  
  Component BrakeController {
    id = "LC-002"
    name = "Brake Controller"
    component_type = "Logical"
    safety_level = "ASIL_D"
    allocated_functions = ["SF-002"]
    interfaces_out = ["IBrakeCommand"]
    interfaces_in = ["IRiskData"]
  }
  
  ComponentExchange RiskDataExchange {
    from_port = "LC-001.IRiskData"
    to_port = "LC-002.IRiskData"
    label = "RiskAssessment"
  }
}

PhysicalArchitecture EmergencyBrakingComplete {
  PhysicalNode PerceptionECU {
    id = "PN-001"
    name = "Perception ECU"
    node_type = "Hardware"
    safety_level = "ASIL_C"
    behavior_components = [
      {name = "PerceptionSW", type = "Software"}
    ]
  }
  
  PhysicalNode BrakeECU {
    id = "PN-002"
    name = "Brake ECU"
    node_type = "Hardware"
    safety_level = "ASIL_D"
    behavior_components = [
      {name = "BrakeControlSW", type = "Software"}
    ]
  }
}
```

**Expected Result - System Analysis (Dataflow)**:
- SF-001 → SF-002: Cyan thick line (FLOW), "⟿ ContinuousRiskAssessment (flow)"

**Expected Result - Logical Architecture (Component)**:
- LC-001: Orange border 5px (ASIL_C), lollipop for IRiskData, socket for ISensorInput
- LC-002: Red border 6px (ASIL_D), lollipop for IBrakeCommand, socket for IRiskData
- Allocation: Orange solid lines from SF-001 to LC-001, SF-002 to LC-002

**Expected Result - Physical Architecture**:
- PN-001: Orange border 5px (ASIL_C), 3D cube appearance
- PN-002: Red border 6px (ASIL_D), 3D cube appearance

---

## Test 6: Negative Tests (Edge Cases)

### Test Case 6.1: Component Without Safety Level
```arc
LogicalArchitecture NoSafety {
  Component RegularComponent {
    id = "LC-001"
    name = "Regular Component"
    interfaces_out = ["IData"]
  }
}
```

**Expected Result**: Black border, 2px width (default styling)

---

### Test Case 6.2: Exchange Without Explicit Type
```arc
SystemAnalysis DefaultExchange {
  Function Source {
    id = "SF-001"
    name = "Source"
  }
  
  Function Target {
    id = "SF-002"
    name = "Target"
  }
  
  FunctionalExchange Unspecified {
    from = "SF-001"
    to = "SF-002"
    label = "GenericData"
  }
}
```

**Expected Result**: Blue line (DATA type default), "📦 GenericData" label

---

### Test Case 6.3: Component Without Interfaces
```arc
LogicalArchitecture NoInterfaces {
  Component SimpleComponent {
    id = "LC-001"
    name = "Simple Component"
  }
}
```

**Expected Result**: Component box rendered, no lollipops or sockets

---

## Validation Checklist

Use this checklist when running tests:

### Safety Colors
- [ ] ASIL_D renders with red 6px border
- [ ] ASIL_C renders with orange 5px border
- [ ] ASIL_B renders with orange 4px border
- [ ] ASIL_A renders with yellow 3px border
- [ ] DAL_A renders with red 6px border
- [ ] SIL_4 renders with red 6px border
- [ ] Components without safety level use default black 2px border
- [ ] Safety borders appear on both logical components and physical nodes
- [ ] 3D cube styling preserved on physical nodes with safety colors

### Traceability
- [ ] Function→Component allocation uses orange solid lines
- [ ] Mission→Capability realization uses gray dashed lines
- [ ] Custom arrow markers render correctly
- [ ] Labels have color-coordinated borders
- [ ] Label text includes relationship type (e.g., "allocated to", "realizes")

### Exchange Items
- [ ] EVENT: Red dashed line, open arrow, ⚡ icon
- [ ] FLOW: Cyan thick line, filled arrow, ⟿ icon
- [ ] OPERATION: Double line, standard arrow, ↔ icon
- [ ] DATA: Blue solid line, standard arrow, 📦 icon
- [ ] SHARED_DATA: Purple dotted line, diamond arrow, 🗄 icon
- [ ] UNSET/default: Gray line, standard arrow
- [ ] Labels include icon prefix and type suffix
- [ ] Works in both dataflow.ts and functional.ts renderers

### Interface Notation
- [ ] Provided interfaces render as lollipops (white circles on RIGHT)
- [ ] Required interfaces render as sockets (arcs on LEFT)
- [ ] Interfaces evenly spaced on component boundaries
- [ ] Interface labels positioned correctly
- [ ] Line length is 20px, circle radius is 6px
- [ ] Multiple interfaces don't overlap

### Combined Features
- [ ] Safety colors and interfaces work together on same component
- [ ] Safety colors don't interfere with traceability styles
- [ ] Exchange items render correctly in presence of safety colors
- [ ] All features work in multi-layer diagrams (OA→SA→LA→PA)

---

## Automated Test Script

Create and run this test:

```bash
#!/bin/bash
# test_all_integrations.sh

echo "Testing Safety Colors..."
arclang compile test_safety_asil_d.arc --output safety_asil_d.html
arclang compile test_safety_dal_a.arc --output safety_dal_a.html

echo "Testing Traceability Styles..."
arclang compile test_traceability_allocation.arc --output traceability_alloc.html
arclang compile test_traceability_realizes.arc --output traceability_realizes.html

echo "Testing Exchange Items..."
arclang compile test_exchange_items_all_types.arc --output exchange_all.html
arclang compile test_exchange_functional.arc --output exchange_functional.html

echo "Testing Interface Notation..."
arclang compile test_interfaces_multiple.arc --output interfaces_multiple.html

echo "Testing Combined Features..."
arclang compile test_combined_all_features.arc --output combined_all.html

echo "✅ All integration tests compiled successfully!"
echo "Open HTML files in browser to visually validate renderings."
```

---

## Manual Validation Steps

1. **Open each generated HTML file** in a web browser
2. **Visually inspect** that:
   - Colors match expected values
   - Line styles (solid/dashed/dotted) are correct
   - Arrow types match specifications
   - Labels include correct icons and text
   - Borders have correct widths
   - Interface symbols are properly positioned
3. **Take screenshots** of successful validations
4. **Document any discrepancies** in issue tracker

---

## Success Criteria

✅ **PASS**: All visual elements match expected results from test cases  
⚠️ **PARTIAL**: Some features work, minor visual issues  
❌ **FAIL**: Features don't render or have major visual errors

**Target**: 100% PASS rate across all test cases
