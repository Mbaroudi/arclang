// Adaptive Cruise Control, the complete model: radar and camera perception,
// sensor fusion, target selection, longitudinal control, driver override and
// safety monitoring, followed from the operational need down to the ECUs and
// the product breakdown. Integrity levels follow ISO 26262.
//
//   arclang check examples/automotive/adaptive_cruise_control.arc
//   arclang gate  examples/automotive/adaptive_cruise_control.arc
//   arclang workbench examples/automotive/adaptive_cruise_control.arc

model AdaptiveCruiseControl {
    name: "Adaptive Cruise Control"
    description: "Keeps the set speed and a safe time gap to the vehicle ahead"
    version: "2.0"
}

// ---------------------------------------------------------------------------
// Requirements
// ---------------------------------------------------------------------------

requirements system {
    req "SYS-ACC-001" "Following distance" {
        description: "The ACC shall keep a time gap of at least 2 s to the lead vehicle at every speed"
        category: "Functional safety"
        priority: "Critical"
        safety_level: "ASIL-B"
        verification_method: "test"
    }
    req "SYS-ACC-002" "Cut-in detection" {
        description: "A vehicle cutting into the ego lane shall be detected within 500 ms"
        category: "Performance"
        priority: "High"
        safety_level: "ASIL-B"
        verification_method: "test"
    }
    req "SYS-ACC-003" "Deceleration limit" {
        description: "The ACC shall not command a deceleration above 3.5 m/s2"
        category: "Performance"
        priority: "High"
        safety_level: "ASIL-B"
        rationale: "Passenger comfort and vehicle stability; stronger braking is left to the emergency braking function"
        verification_method: "test"
    }
    req "SYS-ACC-004" "Driver override" {
        description: "A brake pedal input by the driver shall override the ACC within 100 ms"
        category: "Safety override"
        priority: "Critical"
        safety_level: "ASIL-C"
        verification_method: "test"
    }
    req "SYS-ACC-005" "Speed range" {
        description: "The ACC shall operate between 30 km/h and 180 km/h"
        category: "Operational range"
        priority: "Medium"
        safety_level: "ASIL-A"
        verification_method: "test"
    }
    req "SYS-ACC-006" "Driver warnings" {
        description: "The ACC shall warn the driver visually and audibly when it cannot hold the time gap"
        category: "Human machine interface"
        priority: "High"
        safety_level: "ASIL-A"
        verification_method: "demonstration"
    }
    req "SYS-ACC-007" "Self-diagnostics" {
        description: "The ACC shall monitor its sensors and controller continuously and fall back to a safe state on a fault"
        category: "Diagnostic"
        priority: "Critical"
        safety_level: "ASIL-B"
        verification_method: "test"
    }
}

// ---------------------------------------------------------------------------
// Operational analysis: who needs what
// ---------------------------------------------------------------------------

operational_analysis "Adaptive Cruise Control Operations" {
    actor "Driver" {
        id: "ACT-001"
        description: "Sets the cruise speed and the time gap, keeps the authority to brake and to accelerate"
    }
    actor "Lead Vehicle" {
        id: "ACT-003"
        description: "The vehicle ahead in the ego lane, followed by the ACC"
    }
    actor "Traffic Environment" {
        id: "ACT-004"
        description: "Road geometry, weather and the other vehicles that cut in and out"
    }
    entity "Ego Vehicle" {
        id: "ACT-002"
        description: "The host vehicle under ACC control"
        activity "Monitor Traffic" {
            id: "OA-001"
            description: "Watch the vehicles ahead and the lane"
        }
        activity "Adjust Speed" {
            id: "OA-002"
            description: "Speed up or slow down to hold the set speed and the time gap"
        }
        activity "Inform the Driver" {
            id: "OA-003"
            description: "Show the ACC state and warn when the driver must take over"
        }
    }
    operational_capability "Maintain Safe Following Distance" {
        id: "OC-001"
        description: "Adjust the speed automatically to keep a safe distance to the lead vehicle"
        involves: ["ACT-001", "ACT-002", "ACT-003"]
    }
    interaction "Set speed and gap" { from: "ACT-001" to: "OA-002" }
    interaction "Lead vehicle motion" { from: "ACT-003" to: "OA-001" }
    interaction "Cut-in traffic" { from: "ACT-004" to: "OA-001" }
    interaction "ACC status and warnings" { from: "OA-003" to: "ACT-001" }
}

// ---------------------------------------------------------------------------
// System analysis: what the system does
// ---------------------------------------------------------------------------

system_analysis "ACC System" {
    actor "Driver" { id: "SA-DRV" }
    actor "Lead Vehicle" { id: "SA-LEAD" }
    mission "Comfortable and safe highway driving" { id: "MIS-001" }
    capability "Adaptive cruise" {
        id: "CAP-001"
        mission: "MIS-001"
        realizes: "OC-001"
        involves: ["SF-001", "SF-002", "SF-003", "SF-004"]
    }
    function "Sense Environment" {
        id: "SF-001"
        description: "Detect and track the vehicles ahead"
        safety_level: "ASIL-B"
        latency: 60 ms
        port out "objects"
    }
    function "Select Target" {
        id: "SF-002"
        description: "Choose the lead vehicle in the ego lane, notice a cut-in"
        safety_level: "ASIL-B"
        latency: 20 ms
        port in "objects"
        port out "lead"
    }
    function "Determine Target Speed" {
        id: "SF-003"
        description: "Compute the speed and acceleration that keep the time gap"
        safety_level: "ASIL-B"
        latency: 20 ms
        port in "lead"
        port in "driver"
        port out "accel"
    }
    function "Control Vehicle Speed" {
        id: "SF-004"
        description: "Actuate throttle and brakes to reach the target speed"
        safety_level: "ASIL-C"
        latency: 20 ms
        port in "accel"
    }
    function "Handle Driver Inputs" {
        id: "SF-005"
        description: "Read set, resume, cancel and the pedals; give the driver the authority"
        safety_level: "ASIL-C"
        latency: 10 ms
        port out "driver"
    }
    function "Monitor System Health" {
        id: "SF-006"
        description: "Check the sensors and the controller, degrade or switch off on a fault"
        safety_level: "ASIL-B"
    }
    function "Inform the Driver" {
        id: "SF-007"
        description: "Show the ACC state and warn when a takeover is needed"
        safety_level: "ASIL-A"
    }
    functional_exchange "detected objects" { id: "FE-001" from: "SF-001.objects" to: "SF-002.objects" }
    functional_exchange "lead vehicle" { id: "FE-002" from: "SF-002.lead" to: "SF-003.lead" }
    functional_exchange "driver inputs" { id: "FE-003" from: "SF-005.driver" to: "SF-003.driver" }
    functional_exchange "acceleration demand" { id: "FE-004" from: "SF-003.accel" to: "SF-004.accel" }
    functional_chain "Cut-in reaction" {
        id: "FC-001"
        description: "From the first sight of the cut-in vehicle to the brake demand"
        involves: ["SF-001", "SF-002", "SF-003", "SF-004"]
        latency_budget: 500 ms
    }
    functional_chain "Override path" {
        id: "FC-002"
        description: "From the pedal to the released actuators"
        involves: ["SF-005", "SF-004"]
        latency_budget: 100 ms
    }
}

// ---------------------------------------------------------------------------
// Logical architecture: how the system is structured
// ---------------------------------------------------------------------------

logical_architecture "ACC Logical Architecture" {
    // Perception
    component "Long Range Radar" {
        id: "LC-001"
        safety_level: "ASIL-B"
        description: "77 GHz radar for forward vehicle detection, range and relative speed"
        port out "radar_targets"
        function "Transmit RF Signal" { id: "LF-001" }
        function "Process Echoes" { id: "LF-002" realizes: "SF-001" description: "Extract targets from the received echoes" }
    }
    component "Forward Camera" {
        id: "LC-002"
        safety_level: "ASIL-B"
        description: "Vision sensor for object classification and lane detection"
        port out "camera_objects"
        function "Capture Image" { id: "LF-003" period: 33 ms }
        function "Detect Objects" { id: "LF-004" realizes: "SF-001" description: "Identify vehicles and obstacles" }
        function "Detect Lanes" { id: "LF-005" description: "Identify the lane markings" }
    }
    // Fusion and target selection
    component "Sensor Fusion" {
        id: "LC-003"
        safety_level: "ASIL-B"
        description: "Combines radar and camera data into consistent object tracks"
        port in "radar_targets"
        port in "camera_objects"
        port out "tracks"
        function "Correlate Detections" { id: "LF-006" realizes: "SF-001" description: "Match radar and camera targets" }
        function "Track Objects" { id: "LF-007" realizes: "SF-001" description: "Keep the tracks consistent over time" }
        function "Predict Trajectories" { id: "LF-008" description: "Estimate where each track goes next" }
    }
    component "Target Selection" {
        id: "LC-004"
        safety_level: "ASIL-B"
        description: "Decides which vehicle to follow"
        port in "tracks"
        port out "lead"
        function "Identify Lead Vehicle" { id: "LF-009" realizes: "SF-002" description: "Select the most relevant target in the ego lane" }
        function "Detect Cut-In" { id: "LF-010" realizes: "SF-002" description: "Notice a vehicle entering the ego lane" }
    }
    // Control
    component "Longitudinal Controller" {
        id: "LC-005"
        safety_level: "ASIL-B"
        description: "Computes the acceleration that holds the set speed and the time gap"
        port in "lead"
        port in "driver"
        port in "limits"
        port out "accel"
        port out "status"
        function "Calculate Time Gap" { id: "LF-011" realizes: "SF-003" }
        function "Determine Desired Speed" { id: "LF-012" realizes: "SF-003" }
        function "Compute Acceleration" { id: "LF-013" realizes: "SF-003" description: "Bounded by the limits of the safety monitor" }
    }
    component "Actuator Command" {
        id: "LC-006"
        safety_level: "ASIL-C"
        description: "Maps the acceleration demand to throttle and brake requests on the vehicle bus"
        port in "accel"
        port in "inhibit"
        port out "vehicle_bus"
        function "Map to Throttle" { id: "LF-014" realizes: "SF-004" }
        function "Map to Brake" { id: "LF-015" realizes: "SF-004" description: "Brake pressure limited to 3.5 m/s2" }
    }
    // Safety and override
    component "Safety Monitor" {
        id: "LC-007"
        safety_level: "ASIL-B"
        description: "Watches the sensor health and the controller, enforces the acceleration limits"
        port in "tracks"
        port out "limits"
        function "Check Sensor Health" { id: "LF-016" realizes: "SF-006" }
        function "Enforce Limits" { id: "LF-017" realizes: "SF-006" description: "Clamp acceleration and deceleration to the safe envelope" }
        function "Detect Faults" { id: "LF-018" realizes: "SF-006" }
    }
    component "Override Manager" {
        id: "LC-009"
        safety_level: "ASIL-C"
        description: "Gives the driver the authority: a pedal input disables the ACC"
        port in "pedals"
        port out "inhibit"
        port out "driver"
        function "Detect Brake Pedal" { id: "LF-019" realizes: "SF-005" latency: 10 ms }
        function "Detect Accelerator Pedal" { id: "LF-020" realizes: "SF-005" }
        function "Override Control" { id: "LF-021" realizes: "SF-005" latency: 20 ms description: "Inhibit the actuator command while the driver intervenes" }
    }
    // Driver interface
    component "Driver Interface" {
        id: "LC-008"
        safety_level: "ASIL-A"
        description: "Driver inputs (set, resume, cancel) and the ACC display in the cluster"
        port in "status"
        port out "pedals"
        port out "driver"
        function "Read Driver Inputs" { id: "LF-022" realizes: "SF-005" }
        function "Display Status" { id: "LF-023" realizes: "SF-007" }
        function "Issue Warnings" { id: "LF-024" realizes: "SF-007" description: "Takeover request when the time gap cannot be held" }
    }

    component_exchange "Radar targets" { from_port: "LC-001.radar_targets" to_port: "LC-003.radar_targets" protocol: "CAN FD" period: 50 ms }
    component_exchange "Camera objects" { from_port: "LC-002.camera_objects" to_port: "LC-003.camera_objects" protocol: "Automotive Ethernet" period: 33 ms }
    component_exchange "Object tracks" { from_port: "LC-003.tracks" to_port: "LC-004.tracks" }
    component_exchange "Tracks for monitoring" { from_port: "LC-003.tracks" to_port: "LC-007.tracks" }
    component_exchange "Lead vehicle" { from_port: "LC-004.lead" to_port: "LC-005.lead" }
    component_exchange "Driver settings" { from_port: "LC-008.driver" to_port: "LC-005.driver" }
    component_exchange "Pedal signals" { from_port: "LC-008.pedals" to_port: "LC-009.pedals" }
    component_exchange "Acceleration limits" { from_port: "LC-007.limits" to_port: "LC-005.limits" }
    component_exchange "Acceleration demand" { from_port: "LC-005.accel" to_port: "LC-006.accel" }
    component_exchange "Override inhibit" { from_port: "LC-009.inhibit" to_port: "LC-006.inhibit" }
    component_exchange "ACC status" { from_port: "LC-005.status" to_port: "LC-008.status" }
}

// ---------------------------------------------------------------------------
// Physical architecture: where it runs
// ---------------------------------------------------------------------------

physical_architecture "ACC Physical Architecture" {
    node "Radar ECU" {
        id: "PN-001"
        safety_level: "ASIL-B"
        processor: "Infineon AURIX TC397"
        deploys "LC-001"
    }
    node "Camera ECU" {
        id: "PN-002"
        safety_level: "ASIL-B"
        processor: "Mobileye EyeQ5"
        deploys "LC-002"
    }
    node "ADAS ECU" {
        id: "PN-003"
        safety_level: "ASIL-C"
        processor: "NVIDIA Orin, lockstep safety island"
        ram: 16 GB
        deploys "LC-003"
        deploys "LC-004"
        deploys "LC-005"
        deploys "LC-007"
        deploys "LC-009"
    }
    node "Gateway ECU" {
        id: "PN-004"
        safety_level: "ASIL-C"
        processor: "NXP S32G"
        description: "Routes the throttle and brake requests to the engine and brake ECUs"
        deploys "LC-006"
    }
    node "Instrument Cluster" {
        id: "PN-005"
        safety_level: "ASIL-A"
        processor: "NXP i.MX 8"
        deploys "LC-008"
    }
    physical_link "Sensor CAN FD" { id: "PL-001" from: "PN-001" to: "PN-003" protocol: "CAN FD" bandwidth: 2 Mbps }
    physical_link "Camera Ethernet" { id: "PL-002" from: "PN-002" to: "PN-003" protocol: "100BASE-T1" bandwidth: 100 Mbps }
    physical_link "Vehicle CAN" { id: "PL-003" from: "PN-003" to: "PN-004" protocol: "CAN" bandwidth: 500 kbps }
    physical_link "Cluster CAN" { id: "PL-004" from: "PN-003" to: "PN-005" protocol: "CAN" bandwidth: 500 kbps }
    physical_exchange "Brake and throttle requests" { from: "PN-003" to: "PN-004" via: "Vehicle CAN" message_type: "ACC_Request" frequency: "100 Hz" }
}

epbs "ACC Product Breakdown" {
    system "Adaptive Cruise Control System" {
        id: "EPBS-001"
        subsystem "Sensing Subsystem" {
            id: "EPBS-101"
            item "Long range radar" { id: "EPBS-1001" part_number: "ARS540-2022" supplier: "Continental" }
            item "Forward camera" { id: "EPBS-1002" part_number: "EQ5-2021" supplier: "Mobileye" }
        }
        subsystem "Processing Subsystem" {
            id: "EPBS-102"
            item "ADAS domain controller" { id: "EPBS-1003" part_number: "ADAS-CTRL-001" }
            item "ACC software stack" { id: "EPBS-1004" part_number: "SW-ACC-V2.5.0" }
        }
        subsystem "Communication Subsystem" {
            id: "EPBS-103"
            item "Central gateway" { id: "EPBS-1005" part_number: "CGW-001" }
            item "CAN FD transceiver" { id: "EPBS-1006" part_number: "TJA1463" supplier: "NXP" }
        }
    }
}

// ---------------------------------------------------------------------------
// Safety: HARA and FMEA
// ---------------------------------------------------------------------------

safety_analysis {
    hazard "Unintended Acceleration" {
        id: "HAZ-001"
        description: "The vehicle accelerates when it should hold or decelerate"
        severity: "S3"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-C"
        mitigated_by: ["SYS-ACC-004", "SYS-ACC-007"]
    }
    hazard "Loss of Target Tracking" {
        id: "HAZ-002"
        description: "The lead vehicle is lost and the ACC accelerates to the set speed"
        severity: "S2"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-B"
        mitigated_by: ["SYS-ACC-001", "SYS-ACC-002", "SYS-ACC-006"]
    }
    hazard "Excessive Deceleration" {
        id: "HAZ-003"
        description: "The ACC brakes harder than the following traffic expects"
        severity: "S2"
        exposure: "E3"
        controllability: "C2"
        asil: "ASIL-A"
        mitigated_by: ["SYS-ACC-003"]
    }
    fmea "False target in sensor fusion" {
        id: "FMEA-001"
        failure_mode: "False target detection"
        effect: "Unnecessary braking or acceleration"
        cause: "Radar ghost echo not rejected by the camera"
        severity: "S2"
        occurrence: "Occasional"
        detection: "Camera confirmation required before a track is released"
        rpn: 36
    }
}

// ---------------------------------------------------------------------------
// Modes, scenario, data
// ---------------------------------------------------------------------------

state_machine "ACC operating modes" {
    initial: "Off"
    mode Off { description: "ACC not engaged" }
    mode Standby { description: "Engaged, waiting for a set speed" }
    mode Cruising { description: "Holding the set speed, no lead vehicle" }
    mode Following { description: "Holding the time gap to the lead vehicle" }
    mode Overridden { description: "Driver pedal input, actuator command inhibited" }
    state Degraded { description: "Sensor or controller fault, ACC switched off with a takeover request" }
    transition Off -> Standby { trigger: "LF-022" guard: "main switch on" }
    transition Standby -> Cruising { trigger: "LF-022" guard: "set pressed above 30 km/h" }
    transition Cruising -> Following { trigger: "LF-009" guard: "lead vehicle in range" }
    transition Following -> Cruising { trigger: "LF-009" guard: "lead vehicle gone" }
    transition Cruising -> Overridden { trigger: "LF-021" }
    transition Following -> Overridden { trigger: "LF-021" }
    transition Overridden -> Standby { trigger: "LF-022" guard: "pedal released" }
    transition Following -> Degraded { trigger: "LF-018" }
    transition Cruising -> Degraded { trigger: "LF-018" }
    transition Degraded -> Off { trigger: "LF-022" guard: "main switch off" }
}

scenario "Cut-in vehicle" {
    participants: ["LC-001", "LC-003", "LC-004", "LC-005", "LC-006"]
    message "LC-001" -> "LC-003" "radar targets"
    message "LC-003" -> "LC-004" "object tracks"
    message "LC-004" -> "LC-005" "new lead vehicle" { type: "async" }
    message "LC-005" -> "LC-006" "deceleration demand" { timing: 500 ms }
}

scenario "Driver override" {
    participants: ["LC-008", "LC-009", "LC-006"]
    message "LC-008" -> "LC-009" "brake pedal pressed"
    message "LC-009" -> "LC-006" "inhibit" { timing: 100 ms }
}

class "ObjectTrack" {
    id: "CL-001"
    range: "float"
    relative_speed: "float"
    lane: "LaneAssignment"
}
enumeration "LaneAssignment" {
    id: "EN-001"
    values: ["EgoLane", "LeftLane", "RightLane", "Unknown"]
}
data_type "Acceleration" { id: "DT-001" base: "float" unit: "m/s" }
exchange_item "Tracks" { id: "EI-001" mechanism: "FLOW" elements: ["CL-001"] }
exchange_item "AccelerationDemand" { id: "EI-002" mechanism: "FLOW" }

// ---------------------------------------------------------------------------
// Verification and constraints
// ---------------------------------------------------------------------------

test_case "TC-ACC-001" { verifies: ["SYS-ACC-001"] method: "test" description: "Time gap measured on the track behind a lead vehicle at 50, 100 and 130 km/h" }
test_case "TC-ACC-002" { verifies: ["SYS-ACC-002"] method: "test" description: "Cut-in detection latency on the HIL bench with recorded scenes" }
test_case "TC-ACC-003" { verifies: ["SYS-ACC-003"] method: "test" description: "Deceleration logged over the approach scenarios" }
test_case "TC-ACC-004" { verifies: ["SYS-ACC-004"] method: "test" description: "Brake pedal to inhibited command latency on the HIL bench" }
test_case "TC-ACC-005" { verifies: ["SYS-ACC-005"] method: "test" description: "Engagement attempted below 30 km/h and above 180 km/h" }
test_case "TC-ACC-006" { verifies: ["SYS-ACC-006"] method: "demonstration" description: "Takeover request shown and heard when the gap cannot be held" }
test_case "TC-ACC-007" { verifies: ["SYS-ACC-007"] method: "test" description: "Fault injection on the radar and the controller, safe state reached" }

constraint "Cut-in reaction within budget" {
    id: "CST-ACC-001"
    assert: sum("FC-001", latency) <= "FC-001".latency_budget
}
constraint "Override within budget" {
    id: "CST-ACC-002"
    assert: sum("FC-002", latency) <= "FC-002".latency_budget
}

// ---------------------------------------------------------------------------
// Traceability
// ---------------------------------------------------------------------------

trace "LC-001" satisfies "SYS-ACC-002" { rationale: "The radar sees the cut-in vehicle within one cycle" }
trace "LC-002" satisfies "SYS-ACC-002" { rationale: "The camera confirms the cut-in from the lane change" }
trace "LC-003" satisfies "SYS-ACC-001" { rationale: "Consistent tracks are the input of the time gap" }
trace "LC-004" satisfies "SYS-ACC-001" { rationale: "The right lead vehicle is followed" }
trace "LC-004" satisfies "SYS-ACC-002" { rationale: "Cut-in detection selects the new lead vehicle" }
trace "LC-005" satisfies "SYS-ACC-001" { rationale: "The controller holds the 2 s time gap" }
trace "LC-005" satisfies "SYS-ACC-005" { rationale: "The controller engages only within the speed range" }
trace "LC-006" satisfies "SYS-ACC-003" { rationale: "Brake pressure mapping is limited to 3.5 m/s2" }
trace "LC-007" satisfies "SYS-ACC-003" { rationale: "The safety monitor clamps the acceleration demand" }
trace "LC-007" satisfies "SYS-ACC-007" { rationale: "Continuous sensor and controller diagnostics" }
trace "LC-008" satisfies "SYS-ACC-006" { rationale: "Visual and audible takeover request" }
trace "LC-009" satisfies "SYS-ACC-004" { rationale: "The pedal inhibits the actuator command" }
trace "SF-001" realizes "OA-001"
trace "SF-004" realizes "OA-002"
trace "SF-007" realizes "OA-003"

diagram "Perception to control" {
    view: lab
    description: "The cut-in reaction chain through the logical components"
    shows: ["LC-001", "LC-002", "LC-003", "LC-004", "LC-005", "LC-006"]
}
diagram "Around the ADAS ECU" {
    view: pab
    shows: ["PN-003"]
    context: true
}
