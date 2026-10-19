// ===========================================================================
// Automotive Emergency Braking System (AEB) — the flagship example
// ===========================================================================
//
// Every Arcadia level with the Arcadia method applied:
//   1. Operational analysis   — what the driver and the road need
//   2. System analysis        — the functions, their dataflow, the chain budget
//   3. Logical architecture   — the components that perform the functions
//   4. Physical architecture  — the ECUs, the buses, the deployments
// plus the transverse views: requirements, HARA, modes, scenarios, data
// model, test cases, traces, declared diagrams.
//
// Compliance: ISO 26262, ASIL-D on the braking path.
//
//   arclang check examples/complete_emergency_braking_simple.arc
//   arclang gate  examples/complete_emergency_braking_simple.arc
//   arclang workbench examples/complete_emergency_braking_simple.arc
// ===========================================================================

model EmergencyBrakingSystem {
    name: "Automotive Emergency Braking System (AEB)"
    description: "Detects an imminent collision, warns the driver and brakes autonomously"
    version: "2.1.0"
    author: "Malek Baroudi"
    safety_standard: "ISO 26262 ASIL-D"
}

// ===========================================================================
// 1. Operational analysis: what do the users need?
// ===========================================================================

operational_analysis "Emergency Braking - Operational View" {
    operational_capability "Avoid Collisions" {
        id: "OC-001"
        description: "Avoid collisions with obstacles and vulnerable road users"
        involves: ["OA-ACT-001", "OA-ACT-002", "OA-ACT-003", "Vehicle"]
    }

    // External actors: the people and the world around the vehicle
    actor "Driver" {
        id: "OA-ACT-001"
        description: "Drives the vehicle and keeps the authority over the braking"
        type: "human"
    }
    actor "Pedestrian" {
        id: "OA-ACT-002"
        description: "Vulnerable road user the vehicle must not hit"
        type: "human"
    }
    actor "TrafficEnvironment" {
        id: "OA-ACT-003"
        description: "Road conditions, weather, other vehicles"
        type: "environment"
    }

    // The operational entity and what happens in it
    entity Vehicle {
        description: "The vehicle being controlled"
        activity MonitorEnvironment { description: "Continuously observe the surroundings" }
        activity DetectThreat { description: "Identify a potential collision" }
        activity AlertDriver { description: "Warn the driver of the imminent danger" }
        activity ApplyBrakes { description: "Brake hard enough to avoid or mitigate the collision" }
    }

    // Operational interactions
    interaction DriverCommands { from: Driver to: Vehicle.MonitorEnvironment description: "Driving inputs" }
    interaction EnvironmentData { from: TrafficEnvironment to: Vehicle.MonitorEnvironment description: "Environmental conditions" }
    interaction ThreatDetection { from: Vehicle.MonitorEnvironment to: Vehicle.DetectThreat description: "Observations to analyse" }
    interaction DriverWarning { from: Vehicle.AlertDriver to: Driver description: "Visual and audio warning" }
    interaction BrakingAction { from: Vehicle.ApplyBrakes to: Vehicle description: "Braking force" }
    interaction PedestrianProtection { from: Vehicle.ApplyBrakes to: Pedestrian description: "Collision avoided or mitigated" }
}

// ===========================================================================
// 2. System analysis: what must the system do?
// ===========================================================================

system_analysis SA_EmergencyBraking {
    name: "AEB System - Functional View"

    actor Driver { description: "Vehicle operator" }
    actor Vehicle { description: "Vehicle platform: brakes, bus, cluster" }

    // Mission and capability: why the system exists
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

    // The dataflow path that exemplifies the capability, with its budget
    functional_chain EmergencyBrakingChain {
        id: "FC-001"
        description: "Sensor acquisition to braking command"
        latency_budget: 100 ms
        involves: ["AcquireSensorData", "FuseData", "TrackObjects", "AssessThreat", "ComputeBrakingForce"]
    }

    // System functions
    function AcquireSensorData {
        latency: 20 ms
        description: "Collect the radar and camera data"
        safety_level: "ASIL-D"
        port in "radar_signal"
        port in "camera_image"
        port out "raw_sensor_data"
        function "Read radar" { description: "Range, azimuth and relative speed of the echoes" }
        function "Read camera" { description: "Frames of the forward camera" }
    }
    function FuseData {
        latency: 25 ms
        description: "Combine the radar and camera detections"
        safety_level: "ASIL-D"
        port in "raw_sensor_data"
        port out "fused_data"
    }
    function TrackObjects {
        latency: 15 ms
        description: "Track the detected objects over time"
        safety_level: "ASIL-D"
        port in "fused_data"
        port out "tracked_objects"
    }
    function AssessThreat {
        latency: 20 ms
        description: "Compute the time to collision and the threat level"
        safety_level: "ASIL-D"
        port in "tracked_objects"
        port in "vehicle_speed"
        port out "threat_level"
    }
    function GenerateWarning {
        description: "Create the driver warning signals"
        safety_level: "ASIL-B"
        port in "threat_level"
        port out "warning_signal"
    }
    function ComputeBrakingForce {
        latency: 10 ms
        description: "Calculate the braking force to apply"
        safety_level: "ASIL-D"
        port in "threat_level"
        port in "vehicle_speed"
        port out "brake_command"
    }
    function MonitorVehicleState {
        description: "Track the vehicle speed, the pedals and the steering"
        safety_level: "ASIL-B"
        port in "vehicle_can_bus"
        port out "vehicle_speed"
    }
    function MonitorSystemHealth {
        description: "Check the sensors and the controller, fall back to a safe state on a fault"
        safety_level: "ASIL-D"
    }

    // Functional exchanges: the dataflow between the functions
    functional_exchange SensorToFusion { from: "AcquireSensorData.raw_sensor_data" to: "FuseData.raw_sensor_data" exchange_item: "raw_sensor_data" }
    functional_exchange FusionToTracking { from: "FuseData.fused_data" to: "TrackObjects.fused_data" exchange_item: "fused_data" }
    functional_exchange TrackingToThreat { from: "TrackObjects.tracked_objects" to: "AssessThreat.tracked_objects" exchange_item: "tracked_objects" }
    functional_exchange ThreatToWarning { from: "AssessThreat.threat_level" to: "GenerateWarning.threat_level" exchange_item: "threat_level" }
    functional_exchange ThreatToBraking { from: "AssessThreat.threat_level" to: "ComputeBrakingForce.threat_level" exchange_item: "threat_level" }
    functional_exchange VehicleToThreat { from: "MonitorVehicleState.vehicle_speed" to: "AssessThreat.vehicle_speed" exchange_item: "vehicle_speed" }
    functional_exchange VehicleToBraking { from: "MonitorVehicleState.vehicle_speed" to: "ComputeBrakingForce.vehicle_speed" exchange_item: "vehicle_speed" }
}

// ===========================================================================
// 3. Logical architecture: how is the system structured?
// ===========================================================================

logical_architecture LA_EmergencyBraking {
    name: "AEB System - Component Structure"

    component RadarSensor {
        description: "Front-facing radar sensor"
        type: "Sensor"
        safety_level: "ASIL-D"
        port out RadarDataProvider { interface: "IRadarData" protocol: "CAN FD" }
        function "Acquire radar frames" { realizes: "AcquireSensorData" }
    }

    component CameraSensor {
        description: "Front-facing camera"
        type: "Sensor"
        safety_level: "ASIL-D"
        port out CameraDataProvider { interface: "ICameraData" protocol: "Ethernet" }
        function "Acquire camera frames" { realizes: "AcquireSensorData" }
    }

    component SensorFusion {
        description: "Multi-sensor data fusion"
        type: "Processing"
        safety_level: "ASIL-D"
        port in RadarDataConsumer { interface: "IRadarData" }
        port in CameraDataConsumer { interface: "ICameraData" }
        port out FusedDataProvider { interface: "IFusedData" }
        function "Fuse sensor data" { realizes: "FuseData" }
    }

    component ObjectTracker {
        description: "Object tracking and motion prediction"
        type: "Processing"
        safety_level: "ASIL-D"
        port in FusedDataConsumer { interface: "IFusedData" }
        port out TrackedObjectsProvider { interface: "ITrackedObjects" }
        function "Track objects" { realizes: "TrackObjects" }
    }

    component ThreatAssessment {
        description: "Collision risk assessment"
        type: "Decision"
        safety_level: "ASIL-D"
        port in TrackedObjectsConsumer { interface: "ITrackedObjects" }
        port in VehicleStateConsumer { interface: "IVehicleState" }
        port out ThreatLevelProvider { interface: "IThreatLevel" }
        port out WarningProvider { interface: "IWarning" }
        function "Assess collision threat" { realizes: "AssessThreat" }
        function "Request the warning" { realizes: "GenerateWarning" }
    }

    component BrakeController {
        description: "Emergency brake control"
        type: "Controller"
        safety_level: "ASIL-D"
        port in ThreatLevelConsumer { interface: "IThreatLevel" }
        port in VehicleSpeedConsumer { interface: "IVehicleSpeed" }
        port out BrakeCommandProvider { interface: "IBrakeCommand" }
        function "Compute braking command" { realizes: "ComputeBrakingForce" }
        function "Release on driver override" { description: "Accelerator kick-down cancels the autonomous braking" }
    }

    component DriverInterface {
        description: "HMI for the warnings"
        type: "Interface"
        safety_level: "QM" // quality management: not safety-critical
        port in WarningConsumer { interface: "IWarning" }
        function "Warn driver" { realizes: "GenerateWarning" }
    }

    component VehicleCANGateway {
        description: "Interface to the vehicle CAN bus: speed, pedals, steering"
        type: "Gateway"
        safety_level: "ASIL-B"
        port out VehicleStateProvider { interface: "IVehicleState" }
        port out VehicleSpeedProvider { interface: "IVehicleSpeed" }
        function "Relay vehicle state" { realizes: "MonitorVehicleState" }
    }

    component BrakeActuator {
        description: "Brake actuation on the hydraulic unit"
        type: "Actuator"
        safety_level: "ASIL-D"
        port in BrakeCommandConsumer { interface: "IBrakeCommand" }
        function "Apply braking force" { realizes: "ComputeBrakingForce" }
    }

    component SystemSupervisor {
        description: "Health monitoring of the sensors and the controller"
        type: "Supervisor"
        safety_level: "ASIL-D"
        port in FusedDataConsumer { interface: "IFusedData" }
        function "Monitor system health" { realizes: "MonitorSystemHealth" }
        function "Handle faults" { description: "Switch the AEB off and request a takeover" }
    }

    // Component exchanges: the connections between the components
    component_exchange "RadarToFusion" {
        from_port: "RadarSensor.RadarDataProvider"
        to_port: "SensorFusion.RadarDataConsumer"
        exchange_item: "radar_data"
        label: "Radar Data"
    }
    component_exchange "CameraToFusion" {
        from_port: "CameraSensor.CameraDataProvider"
        to_port: "SensorFusion.CameraDataConsumer"
        exchange_item: "camera_data"
        label: "Camera Data"
    }
    component_exchange "FusionToTracker" {
        from_port: "SensorFusion.FusedDataProvider"
        to_port: "ObjectTracker.FusedDataConsumer"
        exchange_item: "fused_data"
        label: "Fused Data"
    }
    component_exchange "FusionToSupervisor" {
        from_port: "SensorFusion.FusedDataProvider"
        to_port: "SystemSupervisor.FusedDataConsumer"
        exchange_item: "fused_data"
        label: "Fused Data (monitoring)"
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
    component_exchange "ThreatToDriver" {
        from_port: "ThreatAssessment.WarningProvider"
        to_port: "DriverInterface.WarningConsumer"
        exchange_item: "warning_signal"
        label: "Warning"
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
// 4. Physical architecture: where is it deployed?
// ===========================================================================

physical_architecture PA_EmergencyBraking {
    name: "AEB System - Hardware Deployment"

    node FrontRadarECU {
        node_type: "ECU"
        description: "Radar sensor ECU"
        processor: "ARM Cortex-M4"
        memory: "512 KB Flash, 128 KB RAM"
        safety_level: "ASIL-D"
        behavior_component RadarProcessing {
            name: "Radar Signal Processing"
            description: "Radar data acquisition and preprocessing"
            safety_level: "ASIL-D"
            allocated_component: "RadarSensor"
        }
    }

    node FrontCameraECU {
        node_type: "ECU"
        description: "Camera sensor ECU"
        processor: "ARM Cortex-A53"
        ram: 2 GB
        safety_level: "ASIL-D"
        behavior_component CameraProcessing {
            name: "Image Processing"
            description: "Camera image processing and object detection"
            safety_level: "ASIL-D"
            allocated_component: "CameraSensor"
        }
    }

    node CentralADASECU {
        node_type: "ECU"
        description: "Central ADAS processing unit"
        processor: "Intel Atom x5-E3940"
        ram: 8 GB
        storage: 64 GB
        safety_level: "ASIL-D"
        behavior_component FusionModule { name: "Sensor Fusion" safety_level: "ASIL-D" allocated_component: "SensorFusion" }
        behavior_component TrackingModule { name: "Object Tracking" safety_level: "ASIL-D" allocated_component: "ObjectTracker" }
        behavior_component ThreatModule { name: "Threat Assessment" safety_level: "ASIL-D" allocated_component: "ThreatAssessment" }
        behavior_component BrakingModule { name: "Brake Control" safety_level: "ASIL-D" allocated_component: "BrakeController" }
        behavior_component SupervisorModule { name: "System Supervisor" safety_level: "ASIL-D" allocated_component: "SystemSupervisor" }
    }

    node BrakeECU {
        node_type: "ECU"
        description: "Electronic brake control unit"
        processor: "Infineon AURIX TC397"
        memory: "8 MB Flash, 3 MB RAM"
        safety_level: "ASIL-D"
        behavior_component BrakeActuation { name: "Brake Actuator Control" safety_level: "ASIL-D" allocated_component: "BrakeActuator" }
    }

    node InstrumentCluster {
        node_type: "ECU"
        description: "Driver display and HMI"
        processor: "NXP i.MX 8M"
        ram: 4 GB
        safety_level: "QM"
        behavior_component HMI { name: "Warning Display" safety_level: "QM" allocated_component: "DriverInterface" }
    }

    node CANGatewayECU {
        node_type: "Gateway"
        description: "Vehicle CAN gateway"
        processor: "Renesas RH850"
        flash: 2 MB
        safety_level: "ASIL-B"
        behavior_component CANInterface { name: "CAN Bus Interface" safety_level: "ASIL-B" allocated_component: "VehicleCANGateway" }
    }

    // Physical links
    link CANBus { protocol: "CAN FD" from: "FrontRadarECU" to: "CentralADASECU" bandwidth: 5 Mbps }
    link EthernetLink { protocol: "Automotive Ethernet" from: "FrontCameraECU" to: "CentralADASECU" bandwidth: 1 Gbps }
    link BrakeCAN { protocol: "CAN FD" from: "CentralADASECU" to: "BrakeECU" bandwidth: 5 Mbps }
    link VehicleCAN { protocol: "CAN" from: "CANGatewayECU" to: "CentralADASECU" bandwidth: 500 kbps }
    link DisplayLink { protocol: "LVDS" from: "CentralADASECU" to: "InstrumentCluster" bandwidth: 3.2 Gbps }

    // Physical exchanges: the messages on the links
    physical_exchange RadarMessages { from: "FrontRadarECU" to: "CentralADASECU" via: "CANBus" message_type: "RadarTargets" frequency: "50 Hz" }
    physical_exchange CameraMessages { from: "FrontCameraECU" to: "CentralADASECU" via: "EthernetLink" message_type: "CameraFrame" frequency: "30 Hz" }
    physical_exchange BrakeCommands { from: "CentralADASECU" to: "BrakeECU" via: "BrakeCAN" message_type: "BrakeRequest" frequency: "100 Hz" }
    physical_exchange VehicleStatus { from: "CANGatewayECU" to: "CentralADASECU" via: "VehicleCAN" message_type: "VehicleSpeed" frequency: "100 Hz" }

    // Deployment: which logical component runs on which ECU
    deployment RadarSensor -> FrontRadarECU
    deployment CameraSensor -> FrontCameraECU
    deployment SensorFusion -> CentralADASECU
    deployment ObjectTracker -> CentralADASECU
    deployment ThreatAssessment -> CentralADASECU
    deployment BrakeController -> CentralADASECU
    deployment SystemSupervisor -> CentralADASECU
    deployment BrakeActuator -> BrakeECU
    deployment DriverInterface -> InstrumentCluster
    deployment VehicleCANGateway -> CANGatewayECU
}

// ===========================================================================
// Safety requirements (ISO 26262) and their traces
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
        description: "The system shall compute the time to collision of every tracked object"
        safety_level: "ASIL-D"
        priority: "High"
    }
    req "REQ-AEB-004" "Emergency braking activation" {
        description: "Full braking force shall be commanded within 100 ms of a critical threat detection"
        safety_level: "ASIL-D"
        priority: "High"
    }
    req "REQ-AEB-005" "Driver warning" {
        description: "The driver shall be warned before autonomous braking is applied"
        safety_level: "ASIL-B"
        priority: "Medium"
    }
    req "REQ-AEB-006" "Driver override" {
        description: "An accelerator kick-down by the driver shall cancel the autonomous braking"
        safety_level: "ASIL-B"
        priority: "High"
    }
    req "REQ-AEB-007" "Health monitoring" {
        description: "A sensor or controller fault shall switch the AEB off and request a driver takeover within 200 ms"
        safety_level: "ASIL-D"
        priority: "High"
    }
}

trace "RadarSensor" satisfies "REQ-AEB-001" { rationale: "150 m long-range radar detection" }
trace "CameraSensor" satisfies "REQ-AEB-001" { rationale: "Visual obstacle confirmation" }
trace "SensorFusion" satisfies "REQ-AEB-002" { rationale: "Fuses radar and camera within the latency budget" }
trace "ObjectTracker" satisfies "REQ-AEB-003" { rationale: "Consistent tracks are the input of the time to collision" }
trace "ThreatAssessment" satisfies "REQ-AEB-003" { rationale: "Computes the time to collision of every tracked object" }
trace "BrakeController" satisfies "REQ-AEB-004" { rationale: "Commands full braking force on a critical threat" }
trace "BrakeActuator" satisfies "REQ-AEB-004" { rationale: "Builds up the full brake pressure within the budget" }
trace "DriverInterface" satisfies "REQ-AEB-005" { rationale: "Issues the visual and audio warnings" }
trace "VehicleCANGateway" satisfies "REQ-AEB-006" { rationale: "Relays the accelerator pedal state to the brake controller" }
trace "BrakeController" satisfies "REQ-AEB-006" { rationale: "Releases the braking on the kick-down" }
trace "SystemSupervisor" satisfies "REQ-AEB-007" { rationale: "Monitors the sensors and the controller, requests the takeover" }

// ===========================================================================
// Modes and states, scenario
// ===========================================================================

state_machine AEBOperatingModes {
    initial: "Standby"
    mode Standby { description: "System armed, monitoring" }
    mode Warning { description: "Threat detected, driver warned" }
    mode EmergencyBraking { description: "Autonomous braking active" }
    state Failed { description: "Undergone: sensor or actuator failure, driver takeover requested" }

    transition Standby -> Warning { trigger: "AssessThreat" }
    transition Warning -> EmergencyBraking { trigger: "ComputeBrakingForce" }
    transition EmergencyBraking -> Standby { trigger: "MonitorVehicleState" guard: "vehicle stopped or threat gone" }
    transition EmergencyBraking -> Standby { trigger: "Release on driver override" guard: "accelerator kick-down" }
    transition Standby -> Failed { trigger: "Handle faults" guard: "sensor fault" }
}

scenario EmergencyStop {
    participants: ["RadarSensor", "SensorFusion", "ThreatAssessment", "DriverInterface", "BrakeController", "BrakeActuator"]
    message RadarSensor -> SensorFusion "raw radar targets"
    message SensorFusion -> ThreatAssessment "fused object list"
    message ThreatAssessment -> DriverInterface "collision warning" { type: "async" }
    message ThreatAssessment -> BrakeController "critical threat" { type: "async" }
    message BrakeController -> BrakeActuator "full braking force" { timing: 100 ms }
}

// ===========================================================================
// Data model: classes, enumerations, exchange items
// ===========================================================================

class RadarFrame {
    id: "CL-001"
    range_m: "float"
    azimuth_deg: "float"
    velocity_mps: "float"
}
class TrackedObject {
    id: "CL-002"
    track_id: "int"
    time_to_collision_s: "float"
    threat: "ThreatLevel"
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

exchange_item RadarTargets { id: "EI-001" mechanism: "FLOW" elements: ["CL-001"] }
exchange_item ThreatAssessed { id: "EI-002" mechanism: "EVENT" elements: ["EN-001"] }
exchange_item raw_sensor_data { id: "EI-003" mechanism: "FLOW" elements: ["CL-001"] }
exchange_item fused_data { id: "EI-004" mechanism: "FLOW" }
exchange_item tracked_objects { id: "EI-005" mechanism: "FLOW" elements: ["CL-002"] }
exchange_item threat_level { id: "EI-006" mechanism: "EVENT" elements: ["EN-001"] }
exchange_item vehicle_speed { id: "EI-007" mechanism: "FLOW" }
exchange_item radar_data { id: "EI-008" mechanism: "FLOW" elements: ["CL-001"] }
exchange_item camera_data { id: "EI-009" mechanism: "FLOW" }
exchange_item vehicle_state { id: "EI-010" mechanism: "FLOW" }
exchange_item warning_signal { id: "EI-011" mechanism: "EVENT" }
exchange_item brake_command { id: "EI-012" mechanism: "FLOW" elements: ["DT-001"] }

// Vertical traceability: system functions realize operational activities
trace "AcquireSensorData" realizes "MonitorEnvironment" { rationale: "Sensor acquisition realizes environment monitoring" }
trace "AssessThreat" realizes "DetectThreat" { rationale: "Threat assessment realizes threat detection" }
trace "GenerateWarning" realizes "AlertDriver" { rationale: "The warning realizes the driver alert" }
trace "ComputeBrakingForce" realizes "ApplyBrakes" { rationale: "Braking computation realizes brake application" }

// ===========================================================================
// Hazard analysis (ISO 26262 HARA) and verification
// ===========================================================================

safety_analysis {
    hazard "Unintended full braking on highway" {
        id: "HAZ-001"
        severity: "S3"
        exposure: "E4"
        controllability: "C3"
        asil: "ASIL-D"
        mitigated_by: ["REQ-AEB-003", "REQ-AEB-004", "REQ-AEB-006"]
    }
    hazard "No braking on imminent collision" {
        id: "HAZ-002"
        severity: "S3"
        exposure: "E3"
        controllability: "C3"
        asil: "ASIL-C"
        mitigated_by: ["REQ-AEB-001", "REQ-AEB-002", "REQ-AEB-007"]
    }
    hazard "Late driver warning" {
        id: "HAZ-003"
        severity: "S2"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-B"
        mitigated_by: ["REQ-AEB-005"]
    }
}

test_case "Radar range verification" {
    id: "TC-001"
    method: "test"
    verifies: ["REQ-AEB-001"]
    description: "Verify 150 m obstacle detection on the proving ground"
}
test_case "Fusion latency measurement" {
    id: "TC-002"
    method: "test"
    verifies: ["REQ-AEB-002"]
    description: "Measure the radar and camera fusion latency under full bus load"
}
test_case "TTC computation analysis" {
    id: "TC-003"
    method: "analysis"
    verifies: ["REQ-AEB-003"]
    description: "Worst-case execution time analysis of the TTC algorithm"
}
test_case "Emergency braking activation test" {
    id: "TC-004"
    method: "test"
    verifies: ["REQ-AEB-004"]
    description: "Full braking force commanded within 100 ms on the target dummy"
}
test_case "Driver warning demonstration" {
    id: "TC-005"
    method: "demonstration"
    verifies: ["REQ-AEB-005"]
    description: "Visual and audio warning precedes the autonomous braking"
}
test_case "Driver override demonstration" {
    id: "TC-006"
    method: "demonstration"
    verifies: ["REQ-AEB-006"]
    description: "Kick-down during autonomous braking releases the brakes"
}
test_case "Fault injection campaign" {
    id: "TC-007"
    method: "test"
    verifies: ["REQ-AEB-007"]
    description: "Radar and controller faults injected on the HIL bench, takeover requested within 200 ms"
}

constraint "Braking chain within budget" {
    id: "CST-001"
    assert: sum("FC-001", latency) <= "FC-001".latency_budget
}

// ===========================================================================
// Diagrams: named views cut from a level, as Capella diagrams show what was
// put on them. The whole-level views stay available.
// ===========================================================================

diagram "Braking chain" {
    view: lab
    description: "From sensor fusion to the brake actuator"
    shows: [SensorFusion, ThreatAssessment, BrakeController, BrakeActuator]
}

diagram "Around the brake ECU" {
    view: pab
    shows: [BrakeECU]
    context: true
}
