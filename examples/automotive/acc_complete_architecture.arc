// Adaptive Cruise Control, the logical architecture in depth: nine
// components, twenty-five logical functions and the dataflow between them,
// written as functional exchanges between the logical functions and as
// component exchanges between the ports. The model that follows the function
// from the operational need to the ECUs is adaptive_cruise_control.arc.
//
//   arclang check examples/automotive/acc_complete_architecture.arc
//   arclang gate  examples/automotive/acc_complete_architecture.arc

model AccLogicalArchitecture {
    name: "Adaptive Cruise Control - Logical Architecture"
    version: "2.0"
}

operational_analysis "ACC Operational Context" {
    actor "Driver" {
        id: "ACT-001"
        description: "Sets the desired speed, overrides the control, monitors the system status"
    }
    actor "Lead Vehicle" {
        id: "ACT-002"
        description: "The vehicle being followed: position, speed, relative distance"
    }
    actor "Environment" {
        id: "ACT-003"
        description: "Weather, road geometry, traffic density"
    }
    entity "Ego Vehicle" {
        id: "OA-VEH"
        activity "Follow the lead vehicle" { id: "OA-FOLLOW" }
    }
    operational_capability "Follow traffic at a safe distance" {
        id: "OC-001"
        involves: ["ACT-001", "ACT-002", "ACT-003", "OA-VEH"]
    }
}

system_analysis "ACC System Requirements" {
    requirement "SYS-ACC-001" {
        description: "The ACC shall keep a time gap of at least 2 s to the lead vehicle at every speed"
        category: "Functional safety"
        priority: "Critical"
        safety_level: "ASIL-B"
        verification_method: "test"
    }
    requirement "SYS-ACC-002" {
        description: "A vehicle cutting into the ego lane shall be detected within 500 ms"
        category: "Performance"
        priority: "High"
        safety_level: "ASIL-B"
        verification_method: "test"
    }
    requirement "SYS-ACC-003" {
        description: "The ACC shall not command a deceleration above 3.5 m/s2"
        category: "Performance"
        priority: "High"
        safety_level: "ASIL-B"
        rationale: "Passenger comfort and vehicle stability"
        verification_method: "test"
    }
    requirement "SYS-ACC-004" {
        description: "A brake pedal input by the driver shall override the ACC within 100 ms"
        category: "Safety override"
        priority: "Critical"
        safety_level: "ASIL-C"
        verification_method: "test"
    }
    requirement "SYS-ACC-005" {
        description: "The ACC shall operate between 30 km/h and 180 km/h"
        category: "Operational range"
        priority: "Medium"
        safety_level: "ASIL-A"
        verification_method: "test"
    }
    requirement "SYS-ACC-006" {
        description: "The ACC shall warn the driver visually and audibly"
        category: "Human machine interface"
        priority: "High"
        safety_level: "ASIL-A"
        verification_method: "demonstration"
    }
    requirement "SYS-ACC-007" {
        description: "The ACC shall run continuous self-diagnostics"
        category: "Diagnostic"
        priority: "Critical"
        safety_level: "ASIL-B"
        verification_method: "test"
    }
    capability "Adaptive cruise" { id: "CAP-001" realizes: "OC-001" involves: ["SF-001", "SF-002", "SF-003"] }
    function "Perceive the traffic" { id: "SF-001" }
    function "Decide the acceleration" { id: "SF-002" }
    function "Actuate and supervise" { id: "SF-003" }
}

logical_architecture "ACC Logical Architecture" {
    // ---- Perception --------------------------------------------------------
    component "Long Range Radar" {
        id: "LC-001"
        safety_level: "ASIL-B"
        description: "77 GHz radar for forward vehicle detection"
        port out "radar_targets"
        function "Transmit RF Signal" { id: "LF-001" description: "Generate and transmit the radar pulses" power: 100 mW }
        function "Receive Echoes" { id: "LF-002" description: "Capture the reflected signals" }
        function "Process Radar Data" { id: "LF-003" realizes: "SF-001" description: "Extract the targets from the echoes" }
    }
    component "Forward Camera" {
        id: "LC-002"
        safety_level: "ASIL-B"
        description: "Vision sensor for lane and object detection"
        port out "camera_objects"
        function "Capture Image" { id: "LF-004" description: "Acquire the camera frames" frequency: 30 Hz }
        function "Detect Objects" { id: "LF-005" realizes: "SF-001" description: "Identify vehicles and obstacles" }
        function "Detect Lanes" { id: "LF-006" description: "Identify the lane markings" }
    }
    // ---- Fusion and target selection ---------------------------------------
    component "Sensor Fusion" {
        id: "LC-003"
        safety_level: "ASIL-B"
        description: "Combines radar and camera data for a robust detection"
        port in "radar_targets"
        port in "camera_objects"
        port out "tracks"
        function "Correlate Detections" { id: "LF-007" realizes: "SF-001" description: "Match the radar and camera targets" }
        function "Track Objects" { id: "LF-008" realizes: "SF-001" description: "Keep consistent object tracks over time" }
        function "Predict Trajectories" { id: "LF-009" description: "Estimate the future positions of the tracks" }
    }
    component "Target Selection" {
        id: "LC-004"
        safety_level: "ASIL-B"
        description: "Determines which vehicle to follow"
        port in "tracks"
        port out "lead"
        function "Identify Lead Vehicle" { id: "LF-010" realizes: "SF-002" description: "Select the most relevant target in the ego lane" }
        function "Detect Cut-In" { id: "LF-011" realizes: "SF-002" description: "Identify a vehicle entering the ego lane" }
    }
    // ---- Control -----------------------------------------------------------
    component "Longitudinal Controller" {
        id: "LC-005"
        safety_level: "ASIL-B"
        description: "Calculates the desired acceleration or deceleration"
        port in "lead"
        port in "driver_inputs"
        port in "limits"
        port in "override"
        port out "acceleration"
        function "Calculate Time Gap" { id: "LF-012" realizes: "SF-002" description: "Compute the time to the lead vehicle" }
        function "Determine Desired Speed" { id: "LF-013" realizes: "SF-002" description: "Target speed from the set speed and the lead vehicle" }
        function "Compute Acceleration" { id: "LF-014" realizes: "SF-002" description: "Acceleration needed to reach the desired speed" }
    }
    component "Actuator Command" {
        id: "LC-006"
        safety_level: "ASIL-C"
        description: "Converts the acceleration command to throttle and brake signals"
        port in "acceleration"
        port in "inhibit"
        port out "throttle"
        port out "brake"
        function "Map to Throttle" { id: "LF-015" realizes: "SF-003" description: "Positive acceleration to throttle position" }
        function "Map to Brake" { id: "LF-016" realizes: "SF-003" description: "Negative acceleration to brake pressure, limited to 3.5 m/s2" }
    }
    // ---- Safety and monitoring ---------------------------------------------
    component "Safety Monitor" {
        id: "LC-007"
        safety_level: "ASIL-B"
        description: "Monitors the system health and enforces the safety limits"
        port in "tracks"
        port out "limits"
        port out "faults"
        function "Check Sensor Health" { id: "LF-017" realizes: "SF-003" description: "Verify the validity of the sensor data" }
        function "Enforce Limits" { id: "LF-018" realizes: "SF-003" description: "Limit acceleration and deceleration to safe values" }
        function "Detect Faults" { id: "LF-019" realizes: "SF-003" description: "Identify the malfunctions" }
    }
    component "Driver Interface" {
        id: "LC-008"
        safety_level: "ASIL-A"
        description: "Handles the driver inputs and the feedback"
        port in "faults"
        port out "driver_inputs"
        port out "pedals"
        function "Read Driver Inputs" { id: "LF-020" realizes: "SF-002" description: "Capture set, resume and cancel" }
        function "Display Status" { id: "LF-021" description: "Show the ACC status in the cluster" }
        function "Issue Warnings" { id: "LF-022" description: "Alert the driver of the critical conditions" }
    }
    component "Override Manager" {
        id: "LC-009"
        safety_level: "ASIL-C"
        description: "Handles the driver override conditions"
        port in "pedals"
        port out "override"
        port out "inhibit"
        function "Detect Brake Pedal" { id: "LF-023" realizes: "SF-003" description: "Monitor the brake pedal" }
        function "Detect Accelerator Pedal" { id: "LF-024" description: "Monitor the accelerator pedal" }
        function "Override Control" { id: "LF-025" realizes: "SF-003" description: "Disable the ACC when the driver overrides" }
    }

    // ---- Component exchanges -----------------------------------------------
    component_exchange "Radar targets" { from_port: "LC-001.radar_targets" to_port: "LC-003.radar_targets" }
    component_exchange "Camera objects" { from_port: "LC-002.camera_objects" to_port: "LC-003.camera_objects" }
    component_exchange "Tracks to selection" { from_port: "LC-003.tracks" to_port: "LC-004.tracks" }
    component_exchange "Tracks to monitor" { from_port: "LC-003.tracks" to_port: "LC-007.tracks" }
    component_exchange "Lead vehicle" { from_port: "LC-004.lead" to_port: "LC-005.lead" }
    component_exchange "Driver inputs" { from_port: "LC-008.driver_inputs" to_port: "LC-005.driver_inputs" }
    component_exchange "Limits" { from_port: "LC-007.limits" to_port: "LC-005.limits" }
    component_exchange "Faults" { from_port: "LC-007.faults" to_port: "LC-008.faults" }
    component_exchange "Pedals" { from_port: "LC-008.pedals" to_port: "LC-009.pedals" }
    component_exchange "Override" { from_port: "LC-009.override" to_port: "LC-005.override" }
    component_exchange "Inhibit" { from_port: "LC-009.inhibit" to_port: "LC-006.inhibit" }
    component_exchange "Acceleration" { from_port: "LC-005.acceleration" to_port: "LC-006.acceleration" }

    // ---- Functional dataflow between the logical functions -----------------
    functional_exchange "echoes" { id: "LFE-001" from: "LF-001" to: "LF-002" }
    functional_exchange "raw echoes" { id: "LFE-002" from: "LF-002" to: "LF-003" }
    functional_exchange "radar targets" { id: "LFE-003" from: "LF-003" to: "LF-007" }
    functional_exchange "camera objects" { id: "LFE-004" from: "LF-005" to: "LF-007" }
    functional_exchange "correlated detections" { id: "LFE-005" from: "LF-007" to: "LF-008" }
    functional_exchange "tracks to prediction" { id: "LFE-006" from: "LF-008" to: "LF-009" }
    functional_exchange "tracks to selection" { id: "LFE-007" from: "LF-008" to: "LF-010" }
    functional_exchange "lead vehicle" { id: "LFE-008" from: "LF-010" to: "LF-012" }
    functional_exchange "time gap" { id: "LFE-009" from: "LF-012" to: "LF-013" }
    functional_exchange "desired speed" { id: "LFE-010" from: "LF-013" to: "LF-014" }
    functional_exchange "positive acceleration" { id: "LFE-011" from: "LF-014" to: "LF-015" }
    functional_exchange "negative acceleration" { id: "LFE-012" from: "LF-014" to: "LF-016" }
    functional_exchange "brake pedal" { id: "LFE-013" from: "LF-023" to: "LF-025" }
}

physical_architecture "ACC Deployment" {
    node "Sensor ECUs" { id: "PN-001" safety_level: "ASIL-B" deploys "LC-001" deploys "LC-002" }
    node "ADAS ECU" { id: "PN-002" safety_level: "ASIL-C" deploys "LC-003" deploys "LC-004" deploys "LC-005" deploys "LC-006" deploys "LC-007" deploys "LC-009" }
    node "Instrument Cluster" { id: "PN-003" safety_level: "ASIL-A" deploys "LC-008" }
    physical_link "Sensor network" { id: "PL-001" from: "PN-001" to: "PN-002" protocol: "CAN FD and 100BASE-T1" bandwidth: 100 Mbps }
    physical_link "Cluster CAN" { id: "PL-002" from: "PN-002" to: "PN-003" protocol: "CAN" bandwidth: 500 kbps }
}

safety_analysis {
    hazard "Unintended acceleration" {
        id: "HAZ-001"
        severity: "S3"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-C"
        mitigated_by: ["SYS-ACC-004", "SYS-ACC-007"]
    }
    hazard "Excessive deceleration" {
        id: "HAZ-002"
        severity: "S2"
        exposure: "E3"
        controllability: "C2"
        asil: "ASIL-A"
        mitigated_by: ["SYS-ACC-003"]
    }
}

test_case "TC-ACC-001" { verifies: ["SYS-ACC-001"] method: "test" description: "Time gap measured on the track" }
test_case "TC-ACC-002" { verifies: ["SYS-ACC-002"] method: "test" description: "Cut-in detection latency on the HIL bench" }
test_case "TC-ACC-003" { verifies: ["SYS-ACC-003"] method: "test" description: "Deceleration logged over the approach scenarios" }
test_case "TC-ACC-004" { verifies: ["SYS-ACC-004"] method: "test" description: "Brake pedal to inhibited command latency" }
test_case "TC-ACC-005" { verifies: ["SYS-ACC-005"] method: "test" description: "Engagement attempted outside the speed range" }
test_case "TC-ACC-006" { verifies: ["SYS-ACC-006"] method: "demonstration" description: "Warnings shown and heard" }
test_case "TC-ACC-007" { verifies: ["SYS-ACC-007"] method: "test" description: "Fault injection on the sensors and the controller" }

// Requirements to components
trace "LC-001" satisfies "SYS-ACC-002" { rationale: "Radar provides the primary detection of the cut-in vehicles" }
trace "LC-002" satisfies "SYS-ACC-002" { rationale: "Camera provides the complementary detection" }
trace "LC-003" satisfies "SYS-ACC-001" { rationale: "Robust target tracking for the distance keeping" }
trace "LC-004" satisfies "SYS-ACC-001" { rationale: "The correct lead vehicle is followed" }
trace "LC-005" satisfies "SYS-ACC-001" { rationale: "The controller holds the 2 s time gap" }
trace "LC-005" satisfies "SYS-ACC-003" { rationale: "The controller enforces the deceleration limit" }
trace "LC-005" satisfies "SYS-ACC-005" { rationale: "The controller engages within the speed range" }
trace "LC-006" satisfies "SYS-ACC-003" { rationale: "Brake pressure limited to prevent excessive deceleration" }
trace "LC-007" satisfies "SYS-ACC-007" { rationale: "Continuous diagnostics" }
trace "LC-008" satisfies "SYS-ACC-006" { rationale: "Visual and audible warnings" }
trace "LC-009" satisfies "SYS-ACC-004" { rationale: "Brake input detected, ACC disabled" }
trace "SF-001" realizes "OA-FOLLOW"

diagram "Functional dataflow" {
    view: lab
    description: "The logical functions and the exchanges between them"
    shows: ["LC-001", "LC-002", "LC-003", "LC-004", "LC-005", "LC-006"]
}
