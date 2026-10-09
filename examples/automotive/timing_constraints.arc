// Typed quantities, user-defined types and constraints: the compiler checks
// dimensions and type conformance, the production gate checks the assertions.
//
//   arclang check examples/automotive/timing_constraints.arc
//   arclang gate  examples/automotive/timing_constraints.arc
model LaneKeepingTiming {
    metadata {
        version: "1.0"
        description: "Lane keeping assist: end-to-end timing and bus load constraints"
    }
}

// A reusable definition: every periodic task must state its worst case.
type "Periodic task" {
    required: ["wcet", "latency"]
    period: 20 ms
}

type "ECU" {
    required: ["ram"]
    voltage: 12 V
}

// Specialization: a safety ECU is an ECU with an integrity level.
type "Safety ECU" extends "ECU" {
    safety_level: "ASIL-B"
}

type "CAN FD bus" {
    protocol: "CAN FD"
    bandwidth: 2 Mbps
}

operational_analysis "Driving" {
    actor "Driver" { id: "ACT-DRV" }
}

system_analysis "Lane Keeping Assist" {
    requirement "REQ-LKA-001" {
        description: "Steering correction shall be commanded within 80 ms of lane departure detection"
        priority: "Critical"
        safety_level: "ASIL-B"
    }

    function "Detect lane markings" {
        id: "SF-DET"
        is: "Periodic task"
        period: 33 ms
        latency: 30 ms
        wcet: 0.035 s
        port out lane { data_type: "LaneModel" }
    }
    function "Estimate departure" {
        id: "SF-EST"
        is: "Periodic task"
        latency: 15 ms
        wcet: 18 ms
        port in lane { data_type: "LaneModel" }
        port out departure { data_type: "DepartureEstimate" }
    }
    function "Command steering" {
        id: "SF-CMD"
        is: "Periodic task"
        latency: 10 ms
        wcet: 12 ms
        port in departure { data_type: "DepartureEstimate" }
    }

    functional_exchange "SF-DET.lane" -> "SF-EST.lane" { label: "lane model" }
    functional_exchange "SF-EST.departure" -> "SF-CMD.departure" { label: "departure estimate" }

    functional_chain "Lane departure reaction" {
        id: "FC-LKA"
        involves: ["SF-DET", "SF-EST", "SF-CMD"]
        latency_budget: 80 ms
    }
}

logical_architecture "LKA Logical" {
    component "Camera" {
        id: "LC-CAM"
        safety_level: "ASIL-B"
        function "Capture frame" { frame_period: 33 ms }
        port out lane { interface: "LaneModel" }
    }
    component "LKA Controller" {
        id: "LC-CTL"
        safety_level: "ASIL-B"
        function "Plan correction"
        port in lane { interface: "LaneModel" }
    }
    component_exchange "LaneFlow" { from_port: "LC-CAM.lane" to_port: "LC-CTL.lane" }
}

physical_architecture "LKA Physical" {
    node "Camera ECU" { id: "PN-CAM" is: "ECU" ram: 512 MB deploys "LC-CAM" }
    node "Chassis ECU" { id: "PN-CHS" is: "Safety ECU" ram: 64 MB deploys "LC-CTL" }
    link "ChassisCAN" {
        from: "PN-CAM"
        to: "PN-CHS"
        is: "CAN FD bus"
        load: 600 kbps
    }
}

// End-to-end latency must fit the budget with a 20 % margin.
constraint "Reaction time margin" {
    id: "CST-LKA-001"
    description: "Sum of function latencies stays within 80 % of the chain budget"
    assert: sum("FC-LKA", latency) <= "FC-LKA".latency_budget * 0.8
}

// Worst case must still fit the raw budget.
constraint "Worst-case reaction time" {
    id: "CST-LKA-002"
    assert: sum("FC-LKA", wcet) <= "FC-LKA".latency_budget
}

// The slowest stage must complete within one camera frame.
constraint "Stage fits a camera frame" {
    id: "CST-LKA-003"
    assert: max("FC-LKA", wcet) <= "Capture frame".frame_period * 1.1
}

// Bus load stays under 40 % of the available bandwidth.
constraint "CAN bus load" {
    id: "CST-LKA-004"
    assert: ChassisCAN.load / ChassisCAN.bandwidth <= 0.4
}

// Every stage finishes within its own period (inherited or redefined).
constraint "Detection meets its period" {
    id: "CST-LKA-005"
    assert: "SF-DET".wcet <= "SF-DET".period * 1.1
}

test_case "TC-LKA-001" {
    verifies: ["REQ-LKA-001"]
    method: "test"
    description: "HIL measurement of departure-to-command latency"
}

safety_analysis {
    hazard "Late steering correction" {
        description: "Correction commanded too late to keep the lane"
        severity: "S2"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-B"
        mitigated_by: ["REQ-LKA-001"]
    }
}

trace "LC-CTL" satisfies "REQ-LKA-001" { rationale: "The controller commands the correction" }
