// ===========================================================================
// COMPLETE MBSE EXAMPLE: Automotive Emergency Braking System
// ===========================================================================
// 
// This example demonstrates ALL 5 Arcadia layers with proper methodology:
// 1. OA (Operational Analysis) - What users need
// 2. SA (System Analysis) - What the system must do
// 3. LA (Logical Architecture) - How the system is structured (logical)
// 4. PA (Physical Architecture) - How it's deployed (hardware/software)
// 5. EPBS (End Product Breakdown Structure) - Physical products
//
// Compliance: ISO 26262 ASIL-D (Automotive Safety)
// ===========================================================================

model EmergencyBrakingSystem {
  name: "Automotive Emergency Braking System (AEB)"
  description: "Complete MBSE model with all Arcadia layers"
  version: "2.0.0"
  author: "Malek Baroudi"
  safety_standard: "ISO 26262 ASIL-D"
}

// ===========================================================================
// LAYER 1: OPERATIONAL ANALYSIS (OA)
// What do users/operators need to do?
// ===========================================================================

operational_analysis "Emergency Braking - Operational View" {
  operational_capability "Avoid Collisions" {
    id: "OC-001"
    description: "Ability to avoid collisions with obstacles and vulnerable road users"
  }

  // External actors (people/systems interacting with the vehicle)
  actor "Driver" {
    id: "OA-ACT-001"
    description: "Vehicle driver"
    type: "human"
  }
  
  actor "Pedestrian" {
    id: "OA-ACT-002"
    description: "Vulnerable road user"
    type: "human"
  }
  
  actor "TrafficEnvironment" {
    id: "OA-ACT-003"
    description: "Road conditions, weather, other vehicles"
    type: "environment"
  }
  
  // Operational entities (conceptual groupings)
  entity Vehicle {
    description: "The vehicle being controlled"
    
    // Operational activities (what happens operationally)
    activity MonitorEnvironment {
      description: "Continuously observe surroundings"
    }
    
    activity DetectThreat {
      description: "Identify potential collision hazards"
    }
    
    activity AlertDriver {
      description: "Warn driver of imminent danger"
    }
    
    activity ApplyBrakes {
      description: "Execute emergency braking"
    }
  }
  
  // Operational interactions (how activities interact)
  interaction DriverCommands {
    from: Driver
    to: Vehicle.MonitorEnvironment
    description: "Driver provides driving inputs"
  }
  
  interaction EnvironmentData {
    from: TrafficEnvironment
    to: Vehicle.MonitorEnvironment
    description: "Environmental conditions"
  }
  
  interaction ThreatDetection {
    from: Vehicle.MonitorEnvironment
    to: Vehicle.DetectThreat
    description: "Sensor data for analysis"
  }
  
  interaction DriverWarning {
    from: Vehicle.AlertDriver
    to: Driver
    description: "Visual/audio warning"
  }
  
  interaction BrakingAction {
    from: Vehicle.ApplyBrakes
    to: Vehicle
    description: "Physical braking force"
  }
  
  interaction PedestrianProtection {
    from: Vehicle.ApplyBrakes
    to: Pedestrian
    description: "Collision avoidance"
  }
}

// ===========================================================================
// LAYER 2: SYSTEM ANALYSIS (SA)
// What must the system do to satisfy operational needs?
// ===========================================================================

system_analysis SA_EmergencyBraking {
  name: "AEB System - Functional View"
  
  // External actors at system level
  actor Driver {
    description: "Vehicle operator"
  }
  
  actor Vehicle {
    description: "Vehicle platform"
  }
  
  // Mission and capability: why the system exists (Arcadia SA)
  mission SafeAutonomousBraking {
    id: "MIS-001"
    description: "Prevent collisions through autonomous emergency braking"
  }

  capability EmergencyBraking {
    id: "CAP-001"
    description: "Detect an imminent collision and brake autonomously"
    mission: "MIS-001"
    realizes: "OC-001"
    involves: ["AcquireSensorData", "FuseData", "AssessThreat", "ComputeBrakingForce"]
  }

  // The dataflow path that exemplifies the capability (Arcadia Functional Chain)
  functional_chain EmergencyBrakingChain {
    id: "FC-001"
    description: "Sensor acquisition to braking command"
    involves: ["AcquireSensorData", "FuseData", "TrackObjects", "AssessThreat", "ComputeBrakingForce"]
  }

  // System functions (what the system does)
  function AcquireSensorData {
    description: "Collect data from radar and camera"
    inputs: ["radar_signal", "camera_image"]
    outputs: ["raw_sensor_data"]
    color: "#ADD8E6"  // Capella System Function blue
    safety_level: "ASIL-D"
  }
  
  function FuseData {
    description: "Combine multiple sensor inputs"
    inputs: ["raw_sensor_data"]
    outputs: ["fused_data"]
    color: "#ADD8E6"
    safety_level: "ASIL-D"
  }
  
  function TrackObjects {
    description: "Track detected objects over time"
    inputs: ["fused_data"]
    outputs: ["tracked_objects"]
    color: "#ADD8E6"
    safety_level: "ASIL-D"
  }
  
  function AssessThreat {
    description: "Determine collision risk"
    inputs: ["tracked_objects", "vehicle_speed"]
    outputs: ["threat_level"]
    color: "#ADD8E6"
    safety_level: "ASIL-D"
  }
  
  function GenerateWarning {
    description: "Create driver warning signals"
    inputs: ["threat_level"]
    outputs: ["warning_signal"]
    color: "#ADD8E6"
    safety_level: "ASIL-B"
  }
  
  function ComputeBrakingForce {
    description: "Calculate required braking force"
    inputs: ["threat_level", "vehicle_speed"]
    outputs: ["brake_command"]
    color: "#ADD8E6"
    safety_level: "ASIL-D"
  }
  
  function MonitorVehicleState {
    description: "Track vehicle speed, steering, etc."
    inputs: ["vehicle_can_bus"]
    outputs: ["vehicle_speed", "vehicle_state"]
    color: "#ADD8E6"
    safety_level: "ASIL-B"
  }
  
  // Functional exchanges (data flows between functions)
  functional_exchange SensorToFusion {
    from: AcquireSensorData
    to: FuseData
    exchange_item: "raw_sensor_data"
    kind: "DATA"
  }
  
  functional_exchange FusionToTracking {
    from: FuseData
    to: TrackObjects
    exchange_item: "fused_data"
    kind: "DATA"
  }
  
  functional_exchange TrackingToThreat {
    from: TrackObjects
    to: AssessThreat
    exchange_item: "tracked_objects"
    kind: "DATA"
  }
  
  functional_exchange ThreatToWarning {
    from: AssessThreat
    to: GenerateWarning
    exchange_item: "threat_level"
    kind: "EVENT"
  }
  
  functional_exchange ThreatToBraking {
    from: AssessThreat
    to: ComputeBrakingForce
    exchange_item: "threat_level"
    kind: "DATA"
  }
  
  functional_exchange VehicleToThreat {
    from: MonitorVehicleState
    to: AssessThreat
    exchange_item: "vehicle_speed"
    kind: "DATA"
  }
  
  functional_exchange VehicleToBraking {
    from: MonitorVehicleState
    to: ComputeBrakingForce
    exchange_item: "vehicle_speed"
    kind: "DATA"
  }
}

// ===========================================================================
// LAYER 3: LOGICAL ARCHITECTURE (LA)
// How is the system structured (component-based)?
// ===========================================================================

logical_architecture LA_EmergencyBraking {
  name: "AEB System - Component Structure"
  
  // Logical components (software/logical modules)
  component RadarSensor {
    description: "Front-facing radar sensor"
    type: "Sensor"
    color: "#6495ED"  // Capella Logical Component blue
    safety_level: "ASIL-D"
    
    // Provided interfaces (what this component offers)
    interface_out RadarDataProvider {
      name: "IRadarData"
      protocol: "CAN"
    }
  }
  
  component CameraSensor {
    description: "Front-facing camera"
    type: "Sensor"
    color: "#6495ED"
    safety_level: "ASIL-D"
    
    interface_out CameraDataProvider {
      name: "ICameraData"
      protocol: "Ethernet"
    }
  }
  
  component SensorFusion {
    description: "Multi-sensor data fusion"
    type: "Processing"
    color: "#6495ED"
    safety_level: "ASIL-D"
    
    // Required interfaces (what this component needs)
    interface_in RadarDataConsumer {
      name: "IRadarData"
    }
    
    interface_in CameraDataConsumer {
      name: "ICameraData"
    }
    
    interface_out FusedDataProvider {
      name: "IFusedData"
    }
    
    // Allocated functions (from SA layer)
    allocated_function: "FuseData"
  }
  
  component ObjectTracker {
    description: "Object tracking and prediction"
    type: "Processing"
    color: "#6495ED"
    safety_level: "ASIL-D"
    
    interface_in FusedDataConsumer {
      name: "IFusedData"
    }
    
    interface_out TrackedObjectsProvider {
      name: "ITrackedObjects"
    }
    
    allocated_function: "TrackObjects"
  }
  
  component ThreatAssessment {
    description: "Collision risk assessment"
    type: "Decision"
    color: "#6495ED"
    safety_level: "ASIL-D"
    
    interface_in TrackedObjectsConsumer {
      name: "ITrackedObjects"
    }
    
    interface_in VehicleStateConsumer {
      name: "IVehicleState"
    }
    
    interface_out ThreatLevelProvider {
      name: "IThreatLevel"
    }
    
    allocated_function: "AssessThreat"
  }
  
  component BrakeController {
    description: "Emergency brake control"
    type: "Controller"
    color: "#6495ED"
    safety_level: "ASIL-D"
    
    interface_in ThreatLevelConsumer {
      name: "IThreatLevel"
    }
    
    interface_in VehicleSpeedConsumer {
      name: "IVehicleSpeed"
    }
    
    interface_out BrakeCommandProvider {
      name: "IBrakeCommand"
    }
    
    allocated_function: "ComputeBrakingForce"
  }
  
  component DriverInterface {
    description: "HMI for warnings"
    type: "Interface"
    color: "#6495ED"
    safety_level: "QM"  // Quality Management (non-safety-critical)
    
    interface_in WarningConsumer {
      name: "IWarning"
    }
    
    allocated_function: "GenerateWarning"
  }
  
  component VehicleCANGateway {
    description: "CAN bus interface"
    type: "Gateway"
    color: "#6495ED"
    safety_level: "ASIL-B"
    
    interface_out VehicleStateProvider {
      name: "IVehicleState"
    }
    
    interface_out VehicleSpeedProvider {
      name: "IVehicleSpeed"
    }
    
    allocated_function: "MonitorVehicleState"
  }
  
  component BrakeActuator {
    description: "Physical brake actuator"
    type: "Actuator"
    color: "#6495ED"
    safety_level: "ASIL-D"
    
    interface_in BrakeCommandConsumer {
      name: "IBrakeCommand"
    }
  }
  
  // Component exchanges (connections between components)
  component_exchange "RadarToFusion" {
    from_port: "RadarSensor.RadarDataProvider"
    to_port: "SensorFusion.RadarDataConsumer"
    exchange_item: "radar_data"
    label: "Radar Data"
  }
  
  component_exchange "CameraToFusion" {
    from_port: "CameraSensor.CameraDataProvider"
    to_port: "SensorFusion.FusedDataConsumer"
    exchange_item: "camera_data"
    label: "Camera Data"
  }
  
  component_exchange "FusionToTracker" {
    from_port: "SensorFusion.FusedDataProvider"
    to_port: "ObjectTracker.FusedDataConsumer"
    exchange_item: "fused_sensor_data"
    label: "Fused Data"
  }
  
  component_exchange "TrackerToThreat" {
    from_port: "ObjectTracker.TrackedObjectsProvider"
    to_port: "ThreatAssessment.TrackedObjectsConsumer"
    exchange_item: "tracked_objects"
    label: "Tracked Objects"
  }
  
  component_exchange "VehicleStateToThreat" {
    from_port: "VehicleCANGateway.VehicleStateProvider"
    to_port: "ThreatAssessment.VehicleStateConsumer"
    exchange_item: "vehicle_state"
    label: "Vehicle State"
  }
  
  component_exchange "ThreatToBrake" {
    from_port: "ThreatAssessment.ThreatLevelProvider"
    to_port: "BrakeController.ThreatLevelConsumer"
    exchange_item: "threat_level"
    label: "Threat Level"
  }
  
  component_exchange "VehicleSpeedToBrake" {
    from_port: "VehicleCANGateway.VehicleSpeedProvider"
    to_port: "BrakeController.VehicleSpeedConsumer"
    exchange_item: "vehicle_speed"
    label: "Vehicle Speed"
  }
  
  component_exchange "BrakeToActuator" {
    from_port: "BrakeController.BrakeCommandProvider"
    to_port: "BrakeActuator.BrakeCommandConsumer"
    exchange_item: "brake_command"
    label: "Brake Command"
  }
}

// ===========================================================================
// LAYER 4: PHYSICAL ARCHITECTURE (PA)
// How is it deployed on hardware?
// ===========================================================================

physical_architecture PA_EmergencyBraking {
  name: "AEB System - Hardware Deployment"
  
  // Physical nodes (hardware)
  node FrontRadarECU {
    node_type: "ECU"
    description: "Radar sensor ECU"
    processor: "ARM Cortex-M4"
    memory: "512KB Flash, 128KB RAM"
    safety_level: "ASIL-D"
    
    // Behavioral components deployed on this hardware
    behavior_component RadarProcessing {
      name: "Radar Signal Processing"
      description: "Radar data acquisition and preprocessing"
      component_type: "Behavior"
      safety_level: "ASIL-D"
      allocated_component: "RadarSensor"
    }
  }
  
  node FrontCameraECU {
    node_type: "ECU"
    description: "Camera sensor ECU"
    processor: "ARM Cortex-A53"
    memory: "2GB DDR4"
    safety_level: "ASIL-D"
    
    behavior_component CameraProcessing {
      name: "Image Processing"
      description: "Camera image processing and object detection"
      component_type: "Behavior"
      safety_level: "ASIL-D"
      allocated_component: "CameraSensor"
    }
  }
  
  node CentralADASECU {
    node_type: "ECU"
    description: "Central ADAS processing unit"
    processor: "Intel Atom x5-E3940"
    memory: "8GB DDR4, 64GB eMMC"
    safety_level: "ASIL-D"
    
    behavior_component FusionModule {
      name: "Sensor Fusion"
      component_type: "Behavior"
      safety_level: "ASIL-D"
      allocated_component: "SensorFusion"
    }
    
    behavior_component TrackingModule {
      name: "Object Tracking"
      component_type: "Behavior"
      safety_level: "ASIL-D"
      allocated_component: "ObjectTracker"
    }
    
    behavior_component ThreatModule {
      name: "Threat Assessment"
      component_type: "Behavior"
      safety_level: "ASIL-D"
      allocated_component: "ThreatAssessment"
    }
    
    behavior_component BrakingModule {
      name: "Brake Control"
      component_type: "Behavior"
      safety_level: "ASIL-D"
      allocated_component: "BrakeController"
    }
  }
  
  node BrakeECU {
    node_type: "ECU"
    description: "Electronic Brake Control Unit"
    processor: "Infineon AURIX TC397"
    memory: "8MB Flash, 3MB RAM"
    safety_level: "ASIL-D"
    
    behavior_component BrakeActuation {
      name: "Brake Actuator Control"
      component_type: "Behavior"
      safety_level: "ASIL-D"
      allocated_component: "BrakeActuator"
    }
  }
  
  node InstrumentCluster {
    node_type: "ECU"
    description: "Driver display and HMI"
    processor: "NXP i.MX 8M"
    memory: "4GB DDR4"
    safety_level: "QM"
    
    behavior_component HMI {
      name: "Warning Display"
      component_type: "Behavior"
      safety_level: "QM"
      allocated_component: "DriverInterface"
    }
  }
  
  node CANGatewayECU {
    node_type: "Gateway"
    description: "Vehicle CAN gateway"
    processor: "Renesas RH850"
    memory: "2MB Flash"
    safety_level: "ASIL-B"
    
    behavior_component CANInterface {
      name: "CAN Bus Interface"
      component_type: "Behavior"
      safety_level: "ASIL-B"
      allocated_component: "VehicleCANGateway"
    }
  }
  
  // Physical links (hardware connections)
  link CANBus {
    protocol: "CAN FD"
    from: "FrontRadarECU"
    to: "CentralADASECU"
    bandwidth: "5 Mbps"
  }
  
  link EthernetLink {
    protocol: "Automotive Ethernet"
    from: "FrontCameraECU"
    to: "CentralADASECU"
    bandwidth: "1 Gbps"
  }
  
  link BrakeCAN {
    protocol: "CAN FD"
    from: "CentralADASECU"
    to: "BrakeECU"
    bandwidth: "5 Mbps"
  }
  
  link VehicleCAN {
    protocol: "CAN"
    from: "CANGatewayECU"
    to: "CentralADASECU"
    bandwidth: "500 Kbps"
  }
  
  link DisplayLink {
    protocol: "LVDS"
    from: "CentralADASECU"
    to: "InstrumentCluster"
    bandwidth: "135 MHz"
  }
  
  // Physical exchanges (messages on physical links)
  physical_exchange RadarMessages {
    from: "FrontRadarECU"
    to: "CentralADASECU"
    via: "CANBus"
    message_type: "RadarTargets"
    frequency: "50 Hz"
  }
  
  physical_exchange CameraMessages {
    from: "FrontCameraECU"
    to: "CentralADASECU"
    via: "EthernetLink"
    message_type: "CameraFrame"
    frequency: "30 Hz"
  }
  
  physical_exchange BrakeCommands {
    from: "CentralADASECU"
    to: "BrakeECU"
    via: "BrakeCAN"
    message_type: "BrakeRequest"
    frequency: "100 Hz"
  }
  
  physical_exchange VehicleStatus {
    from: "CANGatewayECU"
    to: "CentralADASECU"
    via: "VehicleCAN"
    message_type: "VehicleSpeed"
    frequency: "100 Hz"
  }
}

// ===========================================================================

// ===========================================================================
// SAFETY REQUIREMENTS & TRACEABILITY
// ISO 26262 requirements satisfied by the logical architecture
// ===========================================================================

requirements safety {
  req "REQ-AEB-001" "Obstacle detection range" {
    description: "The system shall detect obstacles at a range of at least 150 m"
    safety_level: "ASIL-D"
    priority: "High"
  }
  req "REQ-AEB-002" "Sensor fusion latency" {
    description: "Radar and camera data shall be fused within 50 ms"
    safety_level: "ASIL-D"
    priority: "High"
  }
  req "REQ-AEB-003" "Collision risk assessment" {
    description: "The system shall compute time-to-collision for every tracked object"
    safety_level: "ASIL-D"
    priority: "High"
  }
  req "REQ-AEB-004" "Emergency braking activation" {
    description: "Full braking force shall be commanded within 100 ms of critical threat detection"
    safety_level: "ASIL-D"
    priority: "High"
  }
  req "REQ-AEB-005" "Driver warning" {
    description: "The driver shall be warned before autonomous braking is applied"
    safety_level: "ASIL-B"
    priority: "Medium"
  }
}

trace "RadarSensor" satisfies "REQ-AEB-001" { rationale: "150 m long-range radar detection" }
trace "CameraSensor" satisfies "REQ-AEB-001" { rationale: "Visual obstacle confirmation" }
trace "SensorFusion" satisfies "REQ-AEB-002" { rationale: "Fuses radar and camera within the latency budget" }
trace "ThreatAssessment" satisfies "REQ-AEB-003" { rationale: "Computes time-to-collision per tracked object" }
trace "BrakeController" satisfies "REQ-AEB-004" { rationale: "Commands full braking force on critical threat" }
trace "DriverInterface" satisfies "REQ-AEB-005" { rationale: "Issues visual and audio warnings" }

// ===========================================================================
// MODES & STATES + SCENARIO (Arcadia transverse concepts)
// ===========================================================================

state_machine AEBOperatingModes {
  initial: "Standby"
  mode Standby { description: "System armed, monitoring" }
  mode Warning { description: "Threat detected, driver warned" }
  mode EmergencyBraking { description: "Autonomous braking active" }
  state Failed { description: "Undergone: sensor or actuator failure" }

  transition Standby -> Warning { trigger: "AssessThreat" }
  transition Warning -> EmergencyBraking { trigger: "ComputeBrakingForce" }
  transition EmergencyBraking -> Standby { trigger: "MonitorVehicleState" }
  transition Standby -> Failed { trigger: "MonitorVehicleState" guard: "sensor_fault" }
}

scenario EmergencyStop {
  participants: ["RadarSensor", "SensorFusion", "ThreatAssessment", "BrakeController", "BrakeActuator"]
  message RadarSensor -> SensorFusion "raw radar targets"
  message SensorFusion -> ThreatAssessment "fused object list"
  message ThreatAssessment -> BrakeController "critical threat" { type: "async" }
  message BrakeController -> BrakeActuator "full braking force"
}

// ===========================================================================
// DATA MODEL (Arcadia: Class, Enumeration, Exchange Items)
// ===========================================================================

class RadarFrame {
  id: "CL-001"
  range_m: "float"
  azimuth_deg: "float"
  velocity_mps: "float"
}

enumeration ThreatLevel {
  id: "EN-001"
  values: ["None", "Low", "Critical"]
}

data_type BrakingForce {
  id: "DT-001"
  base: "float"
  unit: "N"
}

exchange_item RadarTargets {
  id: "EI-001"
  mechanism: "FLOW"
  elements: ["CL-001"]
}

exchange_item ThreatAssessed {
  id: "EI-002"
  mechanism: "EVENT"
  elements: ["EN-001"]
}

exchange_item raw_sensor_data { id: "EI-003" mechanism: "FLOW" elements: ["CL-001"] }
exchange_item fused_data { id: "EI-004" mechanism: "FLOW" }
exchange_item tracked_objects { id: "EI-005" mechanism: "FLOW" }
exchange_item threat_level { id: "EI-006" mechanism: "EVENT" elements: ["EN-001"] }
exchange_item vehicle_speed { id: "EI-007" mechanism: "FLOW" }

// Vertical traceability: system functions realize operational activities
trace "AcquireSensorData" realizes "OA-Mon" { rationale: "Sensor acquisition realizes environment monitoring" }
trace "AssessThreat" realizes "OA-Det" { rationale: "Threat assessment realizes threat detection" }
trace "ComputeBrakingForce" realizes "OA-App" { rationale: "Braking computation realizes brake application" }
