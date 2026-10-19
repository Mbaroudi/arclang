// Emergency braking, the logical architecture in depth: a three-sensor
// perception chain (radar, camera, lidar) feeding fusion, tracking, risk
// assessment and brake control, with the degraded operation that keeps the
// function alive when one sensor fails. The complete AEB model, with its
// physical architecture and its HARA, is complete_emergency_braking_simple.arc.
//
//   arclang check examples/emergency_braking_logical.arc
//   arclang gate  examples/emergency_braking_logical.arc

model EmergencyBrakingLogical {
    name: "Emergency Braking - Logical Architecture"
    version: "2.0"
}

requirements system {
    req "SYS-001" "Obstacle detection" {
        description: "The system shall detect obstacles up to 150 m ahead"
        priority: "Critical"
        safety_level: "ASIL-D"
    }
    req "SYS-002" "Response time" {
        description: "Braking shall start within 100 ms of the obstacle detection"
        priority: "Critical"
        safety_level: "ASIL-D"
    }
    req "SYS-004" "Sensor fusion" {
        description: "The system shall fuse the data of three sensor types: radar, camera and lidar"
        priority: "High"
        safety_level: "ASIL-C"
    }
    req "SYS-005" "Degraded operation" {
        description: "The system shall keep the basic braking function with one sensor failed"
        priority: "High"
        safety_level: "ASIL-C"
    }
    req "SYS-006" "Driver override" {
        description: "The driver shall be able to override the system at any time"
        priority: "High"
        safety_level: "ASIL-B"
    }
}

operational_analysis "Emergency Braking Context" {
    actor "Driver" { id: "ACT-001" description: "Keeps the authority over the braking" }
    actor "Other Vehicles" { id: "ACT-004" description: "The traffic the obstacles come from" }
    entity "Vehicle" {
        id: "ACT-002"
        activity "Detect Obstacles" { id: "OA-003" }
        activity "Brake for the obstacle" { id: "OA-006" }
    }
    operational_capability "Avoid rear-end collisions" { id: "OC-001" involves: ["ACT-001", "ACT-002", "ACT-004"] }
}

system_analysis "Emergency Braking Functions" {
    capability "Autonomous emergency braking" { id: "CAP-001" realizes: "OC-001" involves: ["SF-001", "SF-002", "SF-004", "SF-005", "SF-007"] }
    function "Acquire Sensor Data" { id: "SF-001" latency: 20 ms port out "raw" }
    function "Process Sensor Data" { id: "SF-002" latency: 25 ms description: "Validate, align and fuse" port in "raw" port out "fused" }
    function "Track Objects" { id: "SF-004" latency: 15 ms port in "fused" port out "tracks" }
    function "Assess Situation" { id: "SF-005" latency: 20 ms description: "Time to collision and risk" port in "tracks" port out "risk" }
    function "Execute Response" { id: "SF-007" latency: 10 ms port in "risk" }
    function "Monitor System" { id: "SF-008" description: "Component status and fault handling" }
    functional_exchange "raw data" { id: "FE-001" from: "SF-001.raw" to: "SF-002.raw" }
    functional_exchange "fused data" { id: "FE-002" from: "SF-002.fused" to: "SF-004.fused" }
    functional_exchange "tracks" { id: "FE-003" from: "SF-004.tracks" to: "SF-005.tracks" }
    functional_exchange "risk" { id: "FE-004" from: "SF-005.risk" to: "SF-007.risk" }
    functional_chain "Detection to braking" { id: "FC-001" involves: ["SF-001", "SF-002", "SF-004", "SF-005", "SF-007"] latency_budget: 100 ms }
}

logical_architecture "Emergency Braking Architecture" {
    component "Radar Interface" {
        id: "LC-003"
        description: "Radar sensor interface"
        safety_level: "ASIL-C"
        port out "RadarData" { protocol: "CAN FD" }
        function "Process Radar" { id: "LF-006" realizes: "SF-001" }
    }
    component "Camera Interface" {
        id: "LC-004"
        description: "Camera sensor interface"
        safety_level: "ASIL-C"
        port out "ImageData" { protocol: "Ethernet" }
        function "Process Image" { id: "LF-007" realizes: "SF-001" }
    }
    component "Lidar Interface" {
        id: "LC-005"
        description: "Lidar sensor interface"
        safety_level: "ASIL-C"
        port out "PointCloud" { protocol: "Ethernet" }
        function "Process Lidar" { id: "LF-008" realizes: "SF-001" }
    }
    component "Sensor Fusion Unit" {
        id: "LC-002"
        description: "Multi-sensor fusion; keeps working on two sensors out of three"
        safety_level: "ASIL-C"
        port in "RadarData"
        port in "ImageData"
        port in "PointCloud"
        port out "FusedData"
        function "Validate and align the data" { id: "LF-003" realizes: "SF-002" }
        function "Fuse Data" { id: "LF-004" realizes: "SF-002" }
        function "Degrade on a sensor loss" { id: "LF-005" description: "Reweight the remaining sensors" }
    }
    component "Object Tracker" {
        id: "LC-006"
        description: "Object tracking unit"
        safety_level: "ASIL-C"
        port in "FusedData"
        port out "TrackData"
        function "Track Objects" { id: "LF-009" realizes: "SF-004" }
    }
    component "Risk Assessor" {
        id: "LC-007"
        description: "Risk assessment unit"
        safety_level: "ASIL-D"
        port in "Tracks"
        port out "RiskLevel"
        function "Assess Risk" { id: "LF-011" realizes: "SF-005" }
    }
    component "Brake Controller" {
        id: "LC-008"
        description: "Brake control unit; the driver's pedals win over the request"
        safety_level: "ASIL-D"
        port in "BrakeCommand"
        port in "Supervision"
        port out "BrakeOutput" { protocol: "CAN FD" }
        function "Control Brakes" { id: "LF-012" realizes: "SF-007" }
        function "Yield to the driver" { id: "LF-013" }
    }
    component "Main Controller" {
        id: "LC-001"
        description: "Supervision: health of every unit, fault handling"
        safety_level: "ASIL-D"
        port in "Health"
        port out "Supervision"
        function "Manage System" { id: "LF-001" realizes: "SF-008" }
        function "Monitor Health" { id: "LF-002" realizes: "SF-008" }
    }
    component_exchange "radar data" { from_port: "LC-003.RadarData" to_port: "LC-002.RadarData" }
    component_exchange "image data" { from_port: "LC-004.ImageData" to_port: "LC-002.ImageData" }
    component_exchange "point cloud" { from_port: "LC-005.PointCloud" to_port: "LC-002.PointCloud" }
    component_exchange "fused data to tracker" { from_port: "LC-002.FusedData" to_port: "LC-006.FusedData" }
    component_exchange "tracks" { from_port: "LC-006.TrackData" to_port: "LC-007.Tracks" }
    component_exchange "brake command" { from_port: "LC-007.RiskLevel" to_port: "LC-008.BrakeCommand" }
    component_exchange "supervision" { from_port: "LC-001.Supervision" to_port: "LC-008.Supervision" }
    component_exchange "health" { from_port: "LC-002.FusedData" to_port: "LC-001.Health" }
}

physical_architecture "Emergency Braking Deployment" {
    node "Sensor cluster" { id: "PN-001" deploys "LC-003" deploys "LC-004" deploys "LC-005" }
    node "Central ADAS ECU" { id: "PN-002" safety_level: "ASIL-D" deploys "LC-002" deploys "LC-006" deploys "LC-007" deploys "LC-001" }
    node "Brake ECU" { id: "PN-003" safety_level: "ASIL-D" deploys "LC-008" }
    physical_link "Sensor network" { id: "PL-001" from: "PN-001" to: "PN-002" protocol: "Automotive Ethernet and CAN FD" bandwidth: 1 Gbps }
    physical_link "Brake CAN FD" { id: "PL-002" from: "PN-002" to: "PN-003" protocol: "CAN FD" bandwidth: 5 Mbps }
}

safety_analysis {
    hazard "No braking on an imminent collision" {
        id: "HAZ-001"
        severity: "S3"
        exposure: "E3"
        controllability: "C3"
        asil: "ASIL-C"
        mitigated_by: ["SYS-001", "SYS-004", "SYS-005"]
    }
    hazard "Unintended full braking" {
        id: "HAZ-002"
        severity: "S3"
        exposure: "E4"
        controllability: "C3"
        asil: "ASIL-D"
        mitigated_by: ["SYS-002", "SYS-006"]
    }
}

state_machine "Perception modes" {
    initial: "Nominal"
    mode Nominal { description: "Three sensors fused" }
    mode Degraded { description: "One sensor lost, basic braking kept" }
    state Off { description: "Two sensors lost, function unavailable" }
    transition Nominal -> Degraded { trigger: "LF-005" guard: "one sensor failed" }
    transition Degraded -> Off { trigger: "LF-002" guard: "second sensor failed" }
    transition Degraded -> Nominal { trigger: "LF-002" guard: "sensor recovered" }
}

test_case "TC-001" { verifies: ["SYS-001"] method: "test" description: "Detection range on the proving ground" }
test_case "TC-002" { verifies: ["SYS-002"] method: "test" description: "Detection to brake pressure latency on the HIL bench" }
test_case "TC-004" { verifies: ["SYS-004"] method: "analysis" description: "Fusion design review: three sensor types" }
test_case "TC-005" { verifies: ["SYS-005"] method: "test" description: "Each sensor disconnected in turn: braking still delivered" }
test_case "TC-006" { verifies: ["SYS-006"] method: "demonstration" description: "Pedal input during autonomous braking" }

constraint "Detection to braking within budget" { id: "CST-001" assert: sum("FC-001", latency) <= "FC-001".latency_budget }

trace "LC-003" satisfies "SYS-001" { rationale: "Radar range of 150 m" }
trace "LC-004" satisfies "SYS-004" { rationale: "Second sensor type" }
trace "LC-005" satisfies "SYS-004" { rationale: "Third sensor type" }
trace "LC-002" satisfies "SYS-004" { rationale: "Fusion of the three sensors" }
trace "LC-002" satisfies "SYS-005" { rationale: "Reweighting on a sensor loss" }
trace "LC-006" satisfies "SYS-002" { rationale: "Tracks ready within the budget" }
trace "LC-007" satisfies "SYS-002" { rationale: "Risk assessed within the budget" }
trace "LC-008" satisfies "SYS-006" { rationale: "Pedals override the brake request" }
trace "LC-001" satisfies "SYS-005" { rationale: "Fault handling keeps the function alive" }
trace "SF-001" realizes "OA-003"
trace "SF-007" realizes "OA-006"
