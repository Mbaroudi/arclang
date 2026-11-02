// ===========================================================================
// Automotive Emergency Braking System: the five Arcadia levels
// ===========================================================================
//
// The same AEB as complete_emergency_braking_simple.arc, written as the
// Arcadia method unfolds it, with the fifth level the flagship leaves out:
//   1. OA   - Operational analysis: what the users need
//   2. SA   - System analysis: what the system must do
//   3. LA   - Logical architecture: how the system is structured
//   4. PA   - Physical architecture: how it is deployed
//   5. EPBS - End product breakdown structure: what is bought and built
//
// Compliance: ISO 26262, ASIL-D on the braking path.
//
//   arclang check examples/complete_emergency_braking_mbse.arc
//   arclang gate  examples/complete_emergency_braking_mbse.arc
// ===========================================================================

model EmergencyBrakingSystemEpbs {
    name: "Automotive Emergency Braking System (AEB), five levels"
    description: "AEB model down to the end product breakdown structure"
    version: "2.1.0"
    author: "Malek Baroudi"
    safety_standard: "ISO 26262 ASIL-D"
}

// ===========================================================================
// 1. Operational analysis
// ===========================================================================

operational_analysis OA_EmergencyBraking {
    name: "Emergency Braking - Operational View"

    actor Driver { description: "Vehicle driver" type: "human" }
    actor Pedestrian { description: "Vulnerable road user" type: "human" }
    actor TrafficEnvironment { description: "Road conditions, weather, other vehicles" type: "environment" }

    entity Vehicle {
        description: "The vehicle being controlled"
        activity MonitorEnvironment { description: "Continuously observe the surroundings" }
        activity DetectThreat { description: "Identify a potential collision" }
        activity AlertDriver { description: "Warn the driver of the imminent danger" }
        activity ApplyBrakes { description: "Execute the emergency braking" }
    }

    operational_capability "Avoid Collisions" {
        id: "OC-001"
        involves: ["Driver", "Pedestrian", "TrafficEnvironment", "Vehicle"]
    }

    interaction DriverCommands { from: Driver to: Vehicle.MonitorEnvironment description: "Driving inputs" }
    interaction EnvironmentData { from: TrafficEnvironment to: Vehicle.MonitorEnvironment description: "Environmental conditions" }
    interaction ThreatDetection { from: Vehicle.MonitorEnvironment to: Vehicle.DetectThreat description: "Observations to analyze" }
    interaction DriverWarning { from: Vehicle.AlertDriver to: Driver description: "Visual and audio warning" }
    interaction BrakingAction { from: Vehicle.ApplyBrakes to: Vehicle description: "Braking force" }
    interaction PedestrianProtection { from: Vehicle.ApplyBrakes to: Pedestrian description: "Collision avoided" }
}

// ===========================================================================
// 2. System analysis
// ===========================================================================

system_analysis SA_EmergencyBraking {
    name: "AEB System - Functional View"

    actor Driver { description: "Vehicle operator" }
    actor Vehicle { description: "Vehicle platform" }

    capability EmergencyBraking {
        id: "CAP-001"
        realizes: "OC-001"
        involves: ["AcquireSensorData", "FuseData", "TrackObjects", "AssessThreat", "ComputeBrakingForce"]
    }
    functional_chain EmergencyBrakingChain {
        id: "FC-001"
        latency_budget: 100 ms
        involves: ["AcquireSensorData", "FuseData", "TrackObjects", "AssessThreat", "ComputeBrakingForce"]
    }

    function AcquireSensorData { latency: 20 ms description: "Collect the radar and camera data" safety_level: "ASIL-D" port out "raw_sensor_data" }
    function FuseData { latency: 25 ms description: "Combine the sensor inputs" safety_level: "ASIL-D" port in "raw_sensor_data" port out "fused_data" }
    function TrackObjects { latency: 15 ms description: "Track the detected objects over time" safety_level: "ASIL-D" port in "fused_data" port out "tracked_objects" }
    function AssessThreat { latency: 20 ms description: "Determine the collision risk" safety_level: "ASIL-D" port in "tracked_objects" port in "vehicle_speed" port out "threat_level" }
    function GenerateWarning { description: "Create the driver warning signals" safety_level: "ASIL-B" port in "threat_level" }
    function ComputeBrakingForce { latency: 10 ms description: "Calculate the braking force" safety_level: "ASIL-D" port in "threat_level" port in "vehicle_speed" port out "brake_command" }
    function MonitorVehicleState { description: "Track the vehicle speed, steering and pedals" safety_level: "ASIL-B" port out "vehicle_speed" }

    functional_exchange SensorToFusion { from: "AcquireSensorData.raw_sensor_data" to: "FuseData.raw_sensor_data" }
    functional_exchange FusionToTracking { from: "FuseData.fused_data" to: "TrackObjects.fused_data" }
    functional_exchange TrackingToThreat { from: "TrackObjects.tracked_objects" to: "AssessThreat.tracked_objects" }
    functional_exchange ThreatToWarning { from: "AssessThreat.threat_level" to: "GenerateWarning.threat_level" }
    functional_exchange ThreatToBraking { from: "AssessThreat.threat_level" to: "ComputeBrakingForce.threat_level" }
    functional_exchange VehicleToThreat { from: "MonitorVehicleState.vehicle_speed" to: "AssessThreat.vehicle_speed" }
    functional_exchange VehicleToBraking { from: "MonitorVehicleState.vehicle_speed" to: "ComputeBrakingForce.vehicle_speed" }
}

// ===========================================================================
// 3. Logical architecture
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
        description: "Object tracking and prediction"
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
    }
    component BrakeController {
        description: "Emergency brake control"
        type: "Controller"
        safety_level: "ASIL-D"
        port in ThreatLevelConsumer { interface: "IThreatLevel" }
        port in VehicleSpeedConsumer { interface: "IVehicleSpeed" }
        port out BrakeCommandProvider { interface: "IBrakeCommand" }
        function "Compute braking command" { realizes: "ComputeBrakingForce" }
    }
    component DriverInterface {
        description: "HMI for the warnings"
        type: "Interface"
        safety_level: "QM"
        port in WarningConsumer { interface: "IWarning" }
        function "Warn driver" { realizes: "GenerateWarning" }
    }
    component VehicleCANGateway {
        description: "CAN bus interface"
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

    component_exchange "RadarToFusion" { from_port: "RadarSensor.RadarDataProvider" to_port: "SensorFusion.RadarDataConsumer" label: "Radar Data" }
    component_exchange "CameraToFusion" { from_port: "CameraSensor.CameraDataProvider" to_port: "SensorFusion.CameraDataConsumer" label: "Camera Data" }
    component_exchange "FusionToTracker" { from_port: "SensorFusion.FusedDataProvider" to_port: "ObjectTracker.FusedDataConsumer" label: "Fused Data" }
    component_exchange "TrackerToThreat" { from_port: "ObjectTracker.TrackedObjectsProvider" to_port: "ThreatAssessment.TrackedObjectsConsumer" label: "Tracked Objects" }
    component_exchange "VehicleStateToThreat" { from_port: "VehicleCANGateway.VehicleStateProvider" to_port: "ThreatAssessment.VehicleStateConsumer" label: "Vehicle State" }
    component_exchange "ThreatToBrake" { from_port: "ThreatAssessment.ThreatLevelProvider" to_port: "BrakeController.ThreatLevelConsumer" label: "Threat Level" }
    component_exchange "ThreatToDriver" { from_port: "ThreatAssessment.WarningProvider" to_port: "DriverInterface.WarningConsumer" label: "Warning" }
    component_exchange "VehicleSpeedToBrake" { from_port: "VehicleCANGateway.VehicleSpeedProvider" to_port: "BrakeController.VehicleSpeedConsumer" label: "Vehicle Speed" }
    component_exchange "BrakeToActuator" { from_port: "BrakeController.BrakeCommandProvider" to_port: "BrakeActuator.BrakeCommandConsumer" label: "Brake Command" }
}

// ===========================================================================
// 4. Physical architecture
// ===========================================================================

physical_architecture PA_EmergencyBraking {
    name: "AEB System - Hardware Deployment"

    node FrontRadarECU {
        description: "Radar sensor ECU"
        processor: "ARM Cortex-M4"
        memory: "512 KB Flash, 128 KB RAM"
        safety_level: "ASIL-D"
        behavior_component RadarProcessing { name: "Radar Signal Processing" safety_level: "ASIL-D" allocated_component: "RadarSensor" }
    }
    node FrontCameraECU {
        description: "Camera sensor ECU"
        processor: "ARM Cortex-A53"
        ram: 2 GB
        safety_level: "ASIL-D"
        behavior_component CameraProcessing { name: "Image Processing" safety_level: "ASIL-D" allocated_component: "CameraSensor" }
    }
    node CentralADASECU {
        description: "Central ADAS processing unit"
        processor: "Intel Atom x5-E3940"
        ram: 8 GB
        storage: 64 GB
        safety_level: "ASIL-D"
        behavior_component FusionModule { name: "Sensor Fusion" safety_level: "ASIL-D" allocated_component: "SensorFusion" }
        behavior_component TrackingModule { name: "Object Tracking" safety_level: "ASIL-D" allocated_component: "ObjectTracker" }
        behavior_component ThreatModule { name: "Threat Assessment" safety_level: "ASIL-D" allocated_component: "ThreatAssessment" }
        behavior_component BrakingModule { name: "Brake Control" safety_level: "ASIL-D" allocated_component: "BrakeController" }
    }
    node BrakeECU {
        description: "Electronic brake control unit"
        processor: "Infineon AURIX TC397"
        memory: "8 MB Flash, 3 MB RAM"
        safety_level: "ASIL-D"
        behavior_component BrakeActuation { name: "Brake Actuator Control" safety_level: "ASIL-D" allocated_component: "BrakeActuator" }
    }
    node InstrumentCluster {
        description: "Driver display and HMI"
        processor: "NXP i.MX 8M"
        ram: 4 GB
        safety_level: "QM"
        behavior_component HMI { name: "Warning Display" safety_level: "QM" allocated_component: "DriverInterface" }
    }
    node CANGatewayECU {
        description: "Vehicle CAN gateway"
        processor: "Renesas RH850"
        flash: 2 MB
        safety_level: "ASIL-B"
        behavior_component CANInterface { name: "CAN Bus Interface" safety_level: "ASIL-B" allocated_component: "VehicleCANGateway" }
    }

    link CANBus { protocol: "CAN FD" from: "FrontRadarECU" to: "CentralADASECU" bandwidth: 5 Mbps }
    link EthernetLink { protocol: "Automotive Ethernet" from: "FrontCameraECU" to: "CentralADASECU" bandwidth: 1 Gbps }
    link BrakeCAN { protocol: "CAN FD" from: "CentralADASECU" to: "BrakeECU" bandwidth: 5 Mbps }
    link VehicleCAN { protocol: "CAN" from: "CANGatewayECU" to: "CentralADASECU" bandwidth: 500 kbps }
    link DisplayLink { protocol: "LVDS" from: "CentralADASECU" to: "InstrumentCluster" bandwidth: 3.2 Gbps }

    physical_exchange RadarMessages { from: "FrontRadarECU" to: "CentralADASECU" via: "CANBus" message_type: "RadarTargets" frequency: "50 Hz" }
    physical_exchange CameraMessages { from: "FrontCameraECU" to: "CentralADASECU" via: "EthernetLink" message_type: "CameraFrame" frequency: "30 Hz" }
    physical_exchange BrakeCommands { from: "CentralADASECU" to: "BrakeECU" via: "BrakeCAN" message_type: "BrakeRequest" frequency: "100 Hz" }
    physical_exchange VehicleStatus { from: "CANGatewayECU" to: "CentralADASECU" via: "VehicleCAN" message_type: "VehicleSpeed" frequency: "100 Hz" }

    deployment RadarSensor -> FrontRadarECU
    deployment CameraSensor -> FrontCameraECU
    deployment SensorFusion -> CentralADASECU
    deployment ObjectTracker -> CentralADASECU
    deployment ThreatAssessment -> CentralADASECU
    deployment BrakeController -> CentralADASECU
    deployment BrakeActuator -> BrakeECU
    deployment DriverInterface -> InstrumentCluster
    deployment VehicleCANGateway -> CANGatewayECU
}

// ===========================================================================
// 5. End product breakdown structure
// ===========================================================================

epbs "AEB Product Breakdown" {
    system "Emergency Braking System" {
        id: "EPBS-001"
        subsystem "Perception" {
            id: "EPBS-101"
            item "Front radar unit" { id: "EPBS-1001" part_number: "ARS540-2022" supplier: "Continental" }
            item "Front camera unit" { id: "EPBS-1002" part_number: "EQ5-2021" supplier: "Mobileye" }
        }
        subsystem "Decision" {
            id: "EPBS-102"
            item "Central ADAS ECU" { id: "EPBS-1003" part_number: "ADAS-ECU-003" }
            item "AEB software" { id: "EPBS-1004" part_number: "SW-AEB-2.1.0" certification: "ISO 26262 ASIL-D" }
        }
        subsystem "Actuation and HMI" {
            id: "EPBS-103"
            item "Brake ECU" { id: "EPBS-1005" part_number: "EBCU-397" supplier: "Bosch" }
            item "Instrument cluster" { id: "EPBS-1006" part_number: "IC-IMX8-02" }
            item "CAN gateway" { id: "EPBS-1007" part_number: "CGW-RH850" }
        }
    }
}

// ===========================================================================
// Requirements, traces, HARA, verification, modes
// ===========================================================================

requirements {
    requirement REQ_AEB_001 {
        description: "The system shall detect obstacles at a range of 150 m"
        category: "Functional"
        safety_level: "ASIL-D"
        priority: "High"
    }
    requirement REQ_AEB_002 {
        description: "The system shall activate the brakes within 200 ms of the threat detection"
        category: "Performance"
        safety_level: "ASIL-D"
        priority: "High"
    }
    requirement REQ_AEB_003 {
        description: "The system shall warn the driver before the automatic braking"
        category: "Functional"
        safety_level: "ASIL-B"
        priority: "Medium"
    }
    requirement REQ_AEB_004 {
        description: "The system shall keep its detection performance in rain, fog and at night"
        category: "Environmental"
        safety_level: "ASIL-D"
        priority: "High"
    }
}

trace "RadarSensor" satisfies "REQ_AEB_001"
trace "CameraSensor" satisfies "REQ_AEB_001"
trace "ThreatAssessment" satisfies "REQ_AEB_002"
trace "BrakeController" satisfies "REQ_AEB_002"
trace "BrakeActuator" satisfies "REQ_AEB_002"
trace "VehicleCANGateway" satisfies "REQ_AEB_002" { rationale: "Vehicle speed delivered within the budget" }
trace "DriverInterface" satisfies "REQ_AEB_003"
trace "SensorFusion" satisfies "REQ_AEB_004"
trace "ObjectTracker" satisfies "REQ_AEB_004"
trace "AcquireSensorData" realizes "MonitorEnvironment"
trace "AssessThreat" realizes "DetectThreat"
trace "GenerateWarning" realizes "AlertDriver"
trace "ComputeBrakingForce" realizes "ApplyBrakes"

safety_analysis {
    hazard HAZ_001 {
        description: "Unintended braking activation"
        severity: "S3"
        exposure: "E4"
        controllability: "C3"
        asil: "ASIL-D"
        mitigated_by: ["REQ_AEB_002", "REQ_AEB_003"]
    }
    hazard HAZ_002 {
        description: "Failure to brake when needed"
        severity: "S3"
        exposure: "E3"
        controllability: "C3"
        asil: "ASIL-C"
        mitigated_by: ["REQ_AEB_001", "REQ_AEB_002"]
    }
    hazard HAZ_003 {
        description: "Sensor degradation in bad weather"
        severity: "S3"
        exposure: "E2"
        controllability: "C3"
        asil: "ASIL-B"
        mitigated_by: ["REQ_AEB_004"]
    }
}

test_case "TC-001" { verifies: ["REQ_AEB_001"] method: "test" description: "Detection range on the proving ground" }
test_case "TC-002" { verifies: ["REQ_AEB_002"] method: "test" description: "Threat to brake pressure latency on the HIL bench" }
test_case "TC-003" { verifies: ["REQ_AEB_003"] method: "demonstration" description: "Warning precedes the braking" }
test_case "TC-004" { verifies: ["REQ_AEB_004"] method: "test" description: "Rain, fog and night campaigns on the test track" }

constraint "Braking chain within budget" { id: "CST-001" assert: sum("FC-001", latency) <= "FC-001".latency_budget }

state_machine AEBOperatingModes {
    initial: "Standby"
    mode Standby { description: "System armed, monitoring" }
    mode Warning { description: "Threat detected, driver warned" }
    mode EmergencyBraking { description: "Autonomous braking active" }
    transition Standby -> Warning { trigger: "AssessThreat" }
    transition Warning -> EmergencyBraking { trigger: "ComputeBrakingForce" }
    transition EmergencyBraking -> Standby { trigger: "MonitorVehicleState" }
}

diagram "Product breakdown" { view: pbs shows: ["EPBS-001"] }
