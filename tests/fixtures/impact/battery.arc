// A small model for the change-impact analysis: one link of every kind.
model T { name: "T" }

type "Sensor unit" { mass: 2 kg }

system_analysis "SA" {
    requirement "Range" { id: "REQ-R" description: "The range shall be met" safety_level: "ASIL-D" }
    requirement "Log" { id: "REQ-L" description: "Faults shall be logged" }
    functional_chain "Chain" { id: "FC-1" latency_budget: 50 ms involves: ["SF-A", "SF-B"] }
    function "Acquire" { id: "SF-A" latency: 20 ms port out "o" }
    function "Process" { id: "SF-B" latency: 20 ms port in "i" }
    functional_exchange "AtoB" { from: "SF-A.o" to: "SF-B.i" }
}

logical_architecture "LA" {
    component "Battery" {
        id: "LC-BATT"
        is: "Sensor unit"
        function "Acquire frames" { id: "LF-A" realizes: "SF-A" }
    }
    component "Other" { id: "LC-OTHER" is: "Sensor unit" }
    component "Logger" { id: "LC-LOG" }
}

physical_architecture "PA" {
    node "ECU" { id: "PN-1" deploys "LC-BATT" }
}

trace "LC-BATT" satisfies "REQ-R" { rationale: "the pack sets the range" }
trace "LC-LOG" satisfies "REQ-L" { rationale: "logs the faults" }

test_case "Range test" { id: "TC-R" verifies: ["REQ-R"] method: "test" verdict: "pass" }
test_case "Log test" { id: "TC-L" verifies: ["REQ-L"] method: "test" }

safety_analysis {
    hazard "Loss of range" {
        id: "HAZ-1"
        severity: "S3"
        exposure: "E4"
        controllability: "C3"
        asil: "ASIL-D"
        mitigated_by: ["REQ-R"]
    }
}

constraint "Chain budget" {
    id: "CST-1"
    assert: sum("FC-1", latency) <= "FC-1".latency_budget
}

diagram "Battery view" { view: lab shows: ["LC-BATT"] }

table "Coverage" { kind: requirements_coverage scope: "SA" }
