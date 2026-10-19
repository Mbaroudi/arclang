// Adaptive Cruise Control as it comes out of a Capella import, after the
// first edit pass: the importer brings the requirements, the logical
// components and the traces; the engineer adds the operational need, the
// functions the components perform and the test cases. The complete model
// is adaptive_cruise_control.arc.
//
//   arclang check examples/automotive/acc_from_capella.arc
//   arclang gate  examples/automotive/acc_from_capella.arc

model AccFromCapella {
    name: "Adaptive Cruise Control (Capella import)"
    version: "1.1"
}

operational_analysis "Driving in traffic" {
    actor "Driver" { id: "OA-DRV" }
    operational_capability "Keep a safe following distance" { id: "OC-001" involves: ["OA-DRV"] }
}

system_analysis "Imported System" {
    requirement "SYS-ACC-002" {
        priority: "Critical"
        description: "Driver brake input shall override ACC"
    }
    requirement "SYS-ACC-003" {
        priority: "High"
        description: "System shall detect cut-in vehicles within 500 ms"
    }
    requirement "SYS-ACC-001" {
        priority: "Critical"
        description: "The ACC system shall maintain a minimum 2 s following distance"
    }
    capability "Adaptive cruise" { id: "CAP-001" realizes: "OC-001" involves: ["SF-001", "SF-002"] }
    function "Sense the traffic ahead" { id: "SF-001" port out "targets" }
    function "Plan the speed" { id: "SF-002" port in "targets" }
    functional_exchange "targets" { id: "FE-001" from: "SF-001.targets" to: "SF-002.targets" }
}

logical_architecture "Imported Architecture" {
    component "ACC Controller" {
        type: "Logical"
        id: "LC-004"
        port in "tracks"
        function "Compute the speed profile" { id: "LF-004" realizes: "SF-002" }
    }
    component "Sensor Fusion" {
        id: "LC-003"
        type: "Logical"
        port in "radar"
        port in "camera"
        port out "tracks"
        function "Fuse the detections" { id: "LF-003" realizes: "SF-001" }
    }
    component "Radar Sensor" {
        type: "Logical"
        id: "LC-001"
        port out "radar"
        function "Detect objects" { id: "LF-001" realizes: "SF-001" }
    }
    component "Vision Camera" {
        id: "LC-002"
        type: "Logical"
        port out "camera"
        function "Classify objects" { id: "LF-002" realizes: "SF-001" }
    }
    component_exchange "Radar targets" { from_port: "LC-001.radar" to_port: "LC-003.radar" }
    component_exchange "Camera objects" { from_port: "LC-002.camera" to_port: "LC-003.camera" }
    component_exchange "Object tracks" { from_port: "LC-003.tracks" to_port: "LC-004.tracks" }
}

physical_architecture "Imported Deployment" {
    node "Sensor ECU" { id: "PN-001" deploys "LC-001" deploys "LC-002" }
    node "ADAS ECU" { id: "PN-002" deploys "LC-003" deploys "LC-004" }
    physical_link "Sensor CAN FD" { id: "PL-001" from: "PN-001" to: "PN-002" protocol: "CAN FD" bandwidth: 2 Mbps }
}

test_case "TC-ACC-001" { verifies: ["SYS-ACC-001"] method: "test" description: "Time gap measured behind a lead vehicle" }
test_case "TC-ACC-002" { verifies: ["SYS-ACC-002"] method: "demonstration" description: "Brake pedal override demonstrated" }
test_case "TC-ACC-003" { verifies: ["SYS-ACC-003"] method: "test" description: "Cut-in detection latency on the HIL bench" }

trace "LC-001" satisfies "SYS-ACC-001" {
}

trace "LC-002" satisfies "SYS-ACC-003" {
}

trace "LC-003" satisfies "SYS-ACC-003" {
}

trace "LC-004" satisfies "SYS-ACC-002" {
}
