// A small braking model with gaps on purpose: what the tables show.
model Braking { name: "Braking" }

system_analysis "SA" {
    requirement "REQ-1" { title: "Detect obstacles" description: "Detect obstacles at 150 m" priority: "High" }
    requirement "REQ-2" { title: "Brake in time" description: "Brake within 100 ms" priority: "High" }
    requirement "REQ-3" { title: "Warn the driver" description: "Warn before braking" priority: "Medium" }
    requirement "REQ-4" { title: "Log faults" description: "Log every fault" priority: "Low" }
}

logical_architecture "LA" {
    component "Radar" {
        id: "LC-RADAR"
        port out "targets" { protocol: "CAN FD" }
        function "Sense" { id: "LF-SENSE" }
    }
    component "Controller" {
        id: "LC-CTRL"
        port in "targets"
        port out "command" { protocol: "CAN" }
        function "Decide" { id: "LF-DECIDE" }
    }
    component "Brake" {
        id: "LC-BRAKE"
        port in "command"
        function "Actuate" { id: "LF-ACT" }
    }
    component "Logger" { id: "LC-LOG" }
    function "Archive" { id: "LF-ARCHIVE" }
    component_exchange "Targets" { from_port: "LC-RADAR.targets" to_port: "LC-CTRL.targets" exchange_item: "EI-TARGETS" latency: 10 ms }
    component_exchange "Command" { from_port: "LC-CTRL.command" to_port: "LC-BRAKE.command" }
}

physical_architecture "PA" {
    node "Radar ECU" { id: "PN-RADAR" deploys "LC-RADAR" }
    node "Brake ECU" { id: "PN-BRAKE" deploys "LC-CTRL" deploys "LC-BRAKE" }
    link "CAN" { id: "PL-CAN" from: "PN-RADAR" to: "PN-BRAKE" protocol: "CAN FD" bandwidth: 5 Mbps }
    physical_exchange "Frames" { from: "PN-RADAR" to: "PN-BRAKE" via: "CAN" message_type: "RadarTargets" }
}

exchange_item "Targets" { id: "EI-TARGETS" mechanism: "FLOW" }

trace "LC-RADAR" satisfies "REQ-1"
trace "LC-CTRL" satisfies "REQ-2"
trace "LC-BRAKE" satisfies "REQ-2"
trace "LC-CTRL" satisfies "REQ-3"

test_case "Range test" { id: "TC-1" verifies: ["REQ-1"] method: "test" verdict: "pass" }
test_case "Timing test" { id: "TC-2" verifies: ["REQ-2"] method: "test" verdict: "fail" }
test_case "Warning demo" { id: "TC-3" verifies: ["REQ-3"] method: "demonstration" }

table "Requirement coverage" {
    kind: requirements_coverage
    scope: "SA"
    columns: ["id", "name", "text", "priority", "satisfied_by", "verified_by", "verdict", "gate", "status"]
}
table "Satisfaction" {
    id: "TBL-SAT"
    kind: traceability_matrix
    rows: ["Requirement"]
    columns: ["LogicalComponent"]
    kinds: ["satisfies"]
}
table "Allocation" { kind: allocation_matrix scope: "LA" }
table "Interfaces" { kind: interface_n2 scope: "LA" }
table "ICD" { kind: icd scope: "LA" }
table "Open requirements" { kind: requirements_coverage filter: "is:gap" description: "What the gate still blocks on" }
table "Components" { kind: elements row_type: "LogicalComponent" columns: ["id", "name", "description"] }
