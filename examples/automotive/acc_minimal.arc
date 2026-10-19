// Adaptive Cruise Control, the starter: one capability followed through the
// four Arcadia levels, with the requirements, the hazard and the test cases
// a design review asks for. The complete model is adaptive_cruise_control.arc.
//
//   arclang check examples/automotive/acc_minimal.arc
//   arclang gate  examples/automotive/acc_minimal.arc

model AccMinimal {
    name: "Adaptive Cruise Control (starter)"
    version: "2.0"
}

requirements {
    requirement "SYS-ACC-001" {
        description: "The ACC shall keep a time gap of at least 2 s to the lead vehicle"
        priority: "Critical"
        safety_level: "ASIL-B"
    }
    requirement "SYS-ACC-002" {
        description: "A vehicle cutting into the lane shall be detected within 500 ms"
        priority: "High"
        safety_level: "ASIL-B"
    }
    requirement "SYS-ACC-003" {
        description: "A brake pedal input by the driver shall override the ACC at once"
        priority: "Critical"
        safety_level: "ASIL-C"
    }
}

operational_analysis "Driving in traffic" {
    actor "Driver" { id: "OA-DRV" description: "Sets the cruise speed and keeps the authority to brake" }
    entity "Ego vehicle" {
        id: "OA-VEH"
        activity "Follow the lead vehicle" { id: "OA-FOLLOW" }
    }
    operational_capability "Keep a safe following distance" {
        id: "OC-001"
        involves: ["OA-DRV", "OA-VEH"]
    }
}

system_analysis "ACC System" {
    actor "Driver" { id: "SA-DRV" }
    capability "Adaptive cruise" { id: "CAP-001" realizes: "OC-001" involves: ["SF-SENSE", "SF-PLAN", "SF-ACT"] }
    function "Sense the traffic ahead" { id: "SF-SENSE" latency: 50 ms port out "targets" }
    function "Plan the speed" { id: "SF-PLAN" latency: 20 ms port in "targets" port out "accel" }
    function "Actuate throttle and brake" { id: "SF-ACT" latency: 10 ms port in "accel" }
    functional_exchange "targets" { from: "SF-SENSE.targets" to: "SF-PLAN.targets" }
    functional_exchange "acceleration demand" { from: "SF-PLAN.accel" to: "SF-ACT.accel" }
    functional_chain "Cut-in reaction" { id: "FC-001" involves: ["SF-SENSE", "SF-PLAN", "SF-ACT"] latency_budget: 500 ms }
}

logical_architecture "ACC Logical Architecture" {
    component "Radar Sensor" {
        id: "LC-001"
        safety_level: "ASIL-B"
        port out "targets"
        function "Detect objects" { id: "LF-001" realizes: "SF-SENSE" }
    }
    component "ACC Controller" {
        id: "LC-002"
        safety_level: "ASIL-B"
        port in "targets"
        port in "pedals"
        port out "accel"
        function "Compute the speed profile" { id: "LF-002" realizes: "SF-PLAN" }
        function "Release control on brake pedal" { id: "LF-003" }
    }
    component "Powertrain Interface" {
        id: "LC-003"
        safety_level: "ASIL-C"
        port in "accel"
        function "Command throttle and brake" { id: "LF-004" realizes: "SF-ACT" }
    }
    component_exchange "Targets" { from_port: "LC-001.targets" to_port: "LC-002.targets" }
    component_exchange "Acceleration" { from_port: "LC-002.accel" to_port: "LC-003.accel" }
}

physical_architecture "ACC Physical Architecture" {
    node "Radar ECU" { id: "PN-001" processor: "Infineon AURIX TC397" deploys "LC-001" }
    node "ADAS ECU" { id: "PN-002" processor: "NVIDIA Orin" deploys "LC-002" deploys "LC-003" }
    physical_link "Sensor CAN FD" { id: "PL-001" from: "PN-001" to: "PN-002" protocol: "CAN FD" bandwidth: 2 Mbps }
}

safety_analysis {
    hazard "Unintended acceleration" {
        id: "HAZ-001"
        description: "The vehicle accelerates towards the lead vehicle"
        severity: "S3"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-C"
        mitigated_by: ["SYS-ACC-001", "SYS-ACC-003"]
    }
}

test_case "TC-ACC-001" { verifies: ["SYS-ACC-001"] method: "test" description: "Time gap measured on the track behind a lead vehicle" }
test_case "TC-ACC-002" { verifies: ["SYS-ACC-002"] method: "test" description: "Cut-in detection latency on the HIL bench" }
test_case "TC-ACC-003" { verifies: ["SYS-ACC-003"] method: "demonstration" description: "Brake pedal override demonstrated at every speed" }

trace "LC-001" satisfies "SYS-ACC-002" { rationale: "The radar sees the cut-in vehicle within one cycle" }
trace "LC-002" satisfies "SYS-ACC-001" { rationale: "The controller plans the time gap" }
trace "LC-002" satisfies "SYS-ACC-003" { rationale: "The controller releases control on the pedal signal" }
trace "LC-003" satisfies "SYS-ACC-003" { rationale: "The brake request of the driver wins over the ACC demand" }
trace "SF-SENSE" realizes "OA-FOLLOW"
