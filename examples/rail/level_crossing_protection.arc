// Railway level-crossing protection system — a classic rail-signalling
// safety case: detect an approaching train, close the barriers, prove the
// crossing is protected before the train arrives, fall back to a safe state
// on any failure. Integrity levels follow IEC 61508 / EN 50129.
//
//   arclang check examples/rail/level_crossing_protection.arc
//   arclang gate --standard iec61508 examples/rail/level_crossing_protection.arc

model LevelCrossingProtection {
    name: "Level Crossing Protection System"
    description: "Automatic protection of a road crossing on a single-track line"
    version: "2.0"
}

requirements safety {
    req "REQ-LX-001" "Train detection" {
        description: "The system shall detect an approaching train at least 30 s before it reaches the crossing"
        priority: "High"
        safety_level: "SIL-4"
    }
    req "REQ-LX-002" "Barrier closure" {
        description: "Barriers shall be fully closed at least 10 s before the train reaches the crossing"
        priority: "High"
        safety_level: "SIL-4"
    }
    req "REQ-LX-003" "Crossing clear proving" {
        description: "The system shall prove the crossing area is free of obstacles before authorizing train passage"
        priority: "High"
        safety_level: "SIL-4"
    }
    req "REQ-LX-004" "Fail-safe state" {
        description: "On any internal failure, barriers shall close and the protecting signal shall show stop"
        priority: "High"
        safety_level: "SIL-4"
    }
    req "REQ-LX-005" "Road user warning" {
        description: "Road lights and bells shall warn road users at least 5 s before the barriers start to lower"
        priority: "Medium"
        safety_level: "SIL-2"
    }
}

operational_analysis "Level Crossing Operations" {
    actor "Train Driver" {
        id: "OA-ACT-001"
        description: "Drives the approaching train and obeys the protecting signal"
    }
    actor "Road User" {
        id: "OA-ACT-002"
        description: "Vehicles and pedestrians crossing the track"
    }
    actor "Signaller" {
        id: "OA-ACT-003"
        description: "Supervises the line and handles degraded modes"
    }
    entity "Level crossing" {
        id: "OA-LX"
        activity "Announce the train" { id: "OA-ANNOUNCE" description: "Warn the road before the train arrives" }
        activity "Close the road" { id: "OA-CLOSE" description: "Stop road traffic with barriers" }
        activity "Prove the crossing clear" { id: "OA-PROVE" description: "Confirm nobody is trapped between the barriers" }
        activity "Authorize the passage" { id: "OA-AUTH" description: "Show a proceed aspect to the train" }
    }
    operational_capability "Protect the crossing" {
        id: "OC-001"
        description: "Road traffic is stopped and proven clear before a train passes"
        involves: ["OA-ACT-001", "OA-ACT-002", "OA-LX"]
    }
    interaction "Movement authority" { from: "OA-AUTH" to: "OA-ACT-001" }
    interaction "Road warning" { from: "OA-ANNOUNCE" to: "OA-ACT-002" }
    interaction "Degraded mode handling" { from: "OA-ACT-003" to: "OA-LX" }
}

system_analysis "Level Crossing System" {
    actor "Train Driver" { id: "SA-ACT-001" }
    actor "Signaller" { id: "SA-ACT-003" }
    capability "Protected passage" {
        id: "CAP-001"
        realizes: "OC-001"
        involves: ["SF-DETECT", "SF-WARN", "SF-CLOSE", "SF-PROVE", "SF-AUTH"]
    }
    function "Detect the approaching train" { id: "SF-DETECT" latency: 500 ms port out "approach" }
    function "Warn the road users" { id: "SF-WARN" latency: 200 ms port in "approach" port out "warned" }
    function "Close the barriers" { id: "SF-CLOSE" latency: 8 s port in "warned" port out "closed" }
    function "Prove the crossing clear" { id: "SF-PROVE" latency: 300 ms port in "closed" port out "clear" }
    function "Authorize the train passage" { id: "SF-AUTH" latency: 200 ms port in "clear" }
    function "Fall back to the safe state" { id: "SF-SAFE" description: "Barriers down, signal at stop, on any fault" }
    functional_exchange "train approach" { id: "FE-LX-001" from: "SF-DETECT.approach" to: "SF-WARN.approach" }
    functional_exchange "road warned" { id: "FE-LX-002" from: "SF-WARN.warned" to: "SF-CLOSE.warned" }
    functional_exchange "barriers closed" { id: "FE-LX-003" from: "SF-CLOSE.closed" to: "SF-PROVE.closed" }
    functional_exchange "crossing clear" { id: "FE-LX-004" from: "SF-PROVE.clear" to: "SF-AUTH.clear" }
    functional_chain "Protection cycle" {
        id: "FC-001"
        involves: ["SF-DETECT", "SF-WARN", "SF-CLOSE", "SF-PROVE", "SF-AUTH"]
        latency_budget: 20 s
    }
}

architecture logical {
    component "Train Detector" {
        id: "LC-LX-001"
        safety_level: "SIL-4"
        port out "TrainApproach"
        function "Detect approaching train" { id: "LF-DETECT" realizes: "SF-DETECT" latency: 500 ms }
    }
    component "Crossing Controller" {
        id: "LC-LX-002"
        safety_level: "SIL-4"
        port in "TrainApproach"
        port in "CrossingStatus"
        port out "BarrierCommand"
        port out "SignalCommand"
        port out "WarningCommand"
        function "Sequence the protection cycle" { id: "LF-SEQ" realizes: "SF-CLOSE" latency: 200 ms }
        function "Authorize the passage" { id: "LF-AUTH" realizes: "SF-AUTH" }
        function "Enter fail-safe on fault" { id: "LF-SAFE" realizes: "SF-SAFE" }
    }
    component "Road Warning Unit" {
        id: "LC-LX-006"
        safety_level: "SIL-2"
        port in "WarningCommand"
        function "Flash the lights and ring the bells" { id: "LF-WARN" realizes: "SF-WARN" }
    }
    component "Barrier Machine" {
        id: "LC-LX-003"
        safety_level: "SIL-4"
        port in "BarrierCommand"
        function "Drive the barriers" { id: "LF-BARRIER" realizes: "SF-CLOSE" }
    }
    component "Obstacle Detector" {
        id: "LC-LX-004"
        safety_level: "SIL-4"
        port out "CrossingStatus"
        function "Prove crossing clear" { id: "LF-PROVE" realizes: "SF-PROVE" latency: 300 ms }
    }
    component "Protecting Signal" {
        id: "LC-LX-005"
        safety_level: "SIL-4"
        port in "SignalCommand"
        function "Display movement authority" { id: "LF-SIGNAL" realizes: "SF-AUTH" }
    }
    component_exchange "train approach" { from_port: "LC-LX-001.TrainApproach" to_port: "LC-LX-002.TrainApproach" }
    component_exchange "crossing status" { from_port: "LC-LX-004.CrossingStatus" to_port: "LC-LX-002.CrossingStatus" }
    component_exchange "warning command" { from_port: "LC-LX-002.WarningCommand" to_port: "LC-LX-006.WarningCommand" }
    component_exchange "barrier command" { from_port: "LC-LX-002.BarrierCommand" to_port: "LC-LX-003.BarrierCommand" }
    component_exchange "signal command" { from_port: "LC-LX-002.SignalCommand" to_port: "LC-LX-005.SignalCommand" }
}

architecture physical {
    node "Axle counter head" { id: "PN-LX-001" safety_level: "SIL-4" deploys "LC-LX-001" }
    node "Crossing control cabinet" {
        id: "PN-LX-002"
        safety_level: "SIL-4"
        processor: "2-out-of-2 safety PLC"
        redundancy: "2oo2 with hardware comparator"
        deploys "LC-LX-002"
    }
    node "Roadside equipment" {
        id: "PN-LX-003"
        description: "Barrier machines, road lights, bells and the obstacle radar on the crossing"
        deploys "LC-LX-003"
        deploys "LC-LX-004"
        deploys "LC-LX-006"
    }
    node "Signal post" { id: "PN-LX-004" safety_level: "SIL-4" deploys "LC-LX-005" }
    physical_link "Track-side cable" { id: "PL-LX-001" from: "PN-LX-001" to: "PN-LX-002" protocol: "Axle counter telegram" }
    physical_link "Crossing field bus" { id: "PL-LX-002" from: "PN-LX-002" to: "PN-LX-003" protocol: "Safe serial bus" bandwidth: 115 kbps }
    physical_link "Signal cable" { id: "PL-LX-003" from: "PN-LX-002" to: "PN-LX-004" protocol: "Vital relay circuit" }
}

safety_analysis {
    // Risk graph of IEC 61508-5 written in the gate's classes: consequence as
    // severity, frequency of exposure as exposure, possibility of avoidance as
    // controllability.
    hazard "Train enters an unprotected crossing" {
        id: "HAZ-LX-001"
        description: "Road traffic is still on the crossing when the train arrives"
        severity: "S3"
        exposure: "E4"
        controllability: "C3"
        mitigated_by: ["REQ-LX-001", "REQ-LX-002", "REQ-LX-003"]
    }
    hazard "Wrong-side failure of the controller" {
        id: "HAZ-LX-002"
        description: "An internal fault leaves the barriers open or the signal at proceed"
        severity: "S3"
        exposure: "E2"
        controllability: "C3"
        mitigated_by: ["REQ-LX-004"]
    }
    hazard "Road user struck by a lowering barrier" {
        id: "HAZ-LX-003"
        description: "A barrier lowers on a road user who had no warning"
        severity: "S2"
        exposure: "E3"
        controllability: "C2"
        mitigated_by: ["REQ-LX-005"]
    }
}

state_machine "Crossing modes" {
    initial: "Open"
    mode Open { description: "Road traffic flows, signal at stop for the train" }
    mode Warning { description: "Lights and bells on, barriers still up" }
    mode Closing { description: "Barriers lowering" }
    mode Protected { description: "Barriers down, crossing proven clear, signal at proceed" }
    state Failed { description: "Fail-safe: barriers down, signal at stop, signaller alerted" }
    transition Open -> Warning { trigger: "LF-DETECT" }
    transition Warning -> Closing { trigger: "LF-WARN" timing: 5 s }
    transition Closing -> Protected { trigger: "LF-PROVE" guard: "crossing clear" }
    transition Protected -> Open { trigger: "LF-DETECT" guard: "train has cleared the crossing" }
    transition Closing -> Failed { trigger: "LF-SAFE" }
    transition Protected -> Failed { trigger: "LF-SAFE" }
}

scenario "Normal protection cycle" {
    participants: ["LC-LX-001", "LC-LX-002", "LC-LX-006", "LC-LX-003", "LC-LX-004", "LC-LX-005"]
    message "LC-LX-001" -> "LC-LX-002" "train approach"
    message "LC-LX-002" -> "LC-LX-006" "warning command"
    message "LC-LX-002" -> "LC-LX-003" "barrier command" { timing: 5 s }
    message "LC-LX-004" -> "LC-LX-002" "crossing status"
    message "LC-LX-002" -> "LC-LX-005" "signal command"
}

trace "LC-LX-001" satisfies "REQ-LX-001" { rationale: "Axle counters placed 30 s of running time before the crossing" }
trace "LC-LX-002" satisfies "REQ-LX-002" { rationale: "The controller sequences warning, closure and proving" }
trace "LC-LX-003" satisfies "REQ-LX-002" { rationale: "Barrier actuation within the closure time" }
trace "LC-LX-004" satisfies "REQ-LX-003" { rationale: "Obstacle detection proves the crossing clear" }
trace "LC-LX-002" satisfies "REQ-LX-004" { rationale: "Fail-safe logic of the 2oo2 controller" }
trace "LC-LX-005" satisfies "REQ-LX-004" { rationale: "The signal falls back to stop when the command is lost" }
trace "LC-LX-006" satisfies "REQ-LX-005" { rationale: "Lights and bells run for 5 s before the barriers move" }
trace "SF-DETECT" realizes "OA-ANNOUNCE"
trace "SF-CLOSE" realizes "OA-CLOSE"
trace "SF-PROVE" realizes "OA-PROVE"
trace "SF-AUTH" realizes "OA-AUTH"

test_case "TC-LX-001" {
    description: "Detection timing measured on site with a test train"
    verifies: ["REQ-LX-001"]
    method: "test"
}
test_case "TC-LX-002" {
    description: "Closure sequence timing analysis against the line speed"
    verifies: ["REQ-LX-002"]
    method: "analysis"
}
test_case "TC-LX-003" {
    description: "Obstacle scenarios demonstrated with vehicles and pedestrians"
    verifies: ["REQ-LX-003"]
    method: "demonstration"
}
test_case "TC-LX-004" {
    description: "Fault injection campaign on the controller and the field bus"
    verifies: ["REQ-LX-004"]
    method: "test"
}
test_case "TC-LX-005" {
    description: "Warning time measured from first light to first barrier movement"
    verifies: ["REQ-LX-005"]
    method: "test"
}

constraint "Protection cycle within budget" {
    id: "CST-LX-001"
    assert: sum("FC-001", latency) <= "FC-001".latency_budget
}

diagram "Protection cycle" {
    view: lab
    shows: ["LC-LX-001", "LC-LX-002", "LC-LX-003", "LC-LX-004", "LC-LX-005", "LC-LX-006"]
}
