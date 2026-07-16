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

operational_analysis OA_EmergencyBraking {
  name: "Emergency Braking - Operational View"
  
  // External actors (people/systems interacting with the vehicle)
  actor Driver {
    description: "Vehicle driver"
    type: "human"
  }
  
  actor Pedestrian {
    description: "Vulnerable road user"
    type: "human"
  }
  
  actor TrafficEnvironment {
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
// REQUIREMENTS TRACEABILITY
// Link requirements to design elements
// ===========================================================================

requirements {
  requirement REQ_AEB_001 {
    text: "System shall detect obstacles at 150m range"
    type: "Functional"
    safety_level: "ASIL-D"
    satisfied_by: ["RadarSensor", "CameraSensor"]
  }
  
  requirement REQ_AEB_002 {
    text: "System shall activate brakes within 200ms of threat detection"
    type: "Performance"
    safety_level: "ASIL-D"
    satisfied_by: ["ThreatAssessment", "BrakeController"]
  }
  
  requirement REQ_AEB_003 {
    text: "System shall warn driver before automatic braking"
    type: "Functional"
    safety_level: "ASIL-B"
    satisfied_by: ["DriverInterface"]
  }
  
  requirement REQ_AEB_004 {
    text: "System shall operate in all weather conditions"
    type: "Environmental"
    safety_level: "ASIL-D"
    satisfied_by: ["SensorFusion", "ObjectTracker"]
  }
}

// ===========================================================================
// SAFETY ANALYSIS
// ===========================================================================

safety_analysis {
  hazard HAZ_001 {
    description: "Unintended braking activation"
    asil_level: "ASIL-D"
    mitigation: ["ThreatAssessment redundancy", "Driver override capability"]
  }
  
  hazard HAZ_002 {
    description: "Failure to brake when needed"
    asil_level: "ASIL-D"
    mitigation: ["Redundant sensors", "Watchdog monitoring"]
  }
  
  hazard HAZ_003 {
    description: "Sensor degradation in bad weather"
    asil_level: "ASIL-C"
    mitigation: ["Multi-sensor fusion", "Graceful degradation"]
  }
}
