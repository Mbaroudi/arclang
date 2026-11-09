// {{NAME}}: a model following CESAM (see docs/cesam.md).
//
// One section per CESAM vision; every element is a placeholder to rename and
// complete. The grid shows what each view holds and what is missing:
//
//   arclang table {{FILE}} --kind cesam_grid -f md
//   arclang check {{FILE}} --lint
//   arclang diagram {{FILE}} -o views.html -f html --view cesam

model {{NAME}} {
    name: "{{NAME}}"
    description: "What the system is for, in one sentence"
    version: "0.1"
    method: "cesam"
}

// ---------------------------------------------------------------------------
// Operational vision (why?): stakeholders and needs, lifecycle, use cases,
// environment, operational scenarios
// ---------------------------------------------------------------------------

requirements stakeholder {
    stakeholder "User" { id: "STK-1" category: "user" interest: "What the user expects of the system" influence: High }
    concern "Main need" { id: "NEED-1" stakeholders: ["STK-1"] }
    req "STK-REQ-1" "Stakeholder requirement" { priority: High frames: ["NEED-1"] description: "What the system shall do for the user" }
}

operational_analysis "Environment" {
    actor "User" { id: "OA-USR" }
    entity "Surroundings" { id: "OA-ENV" }
    interaction "Uses the system" { id: "OE-1" from: "OA-USR" to: "OA-ENV" data_type: "Request" }
}

system_analysis "{{NAME}} system" {
    actor "User" { id: "SA-USR" }
    actor "Operator" { id: "SA-OPR" }

    mission "Main mission" { id: "MIS-1" }
    capability "Main use case" { id: "UC-1" mission: "MIS-1" involves: ["SA-USR"] }

    function "Main function" { id: "SF-1" latency: 100 ms port out "result" }
    function "Second function" { id: "SF-2" latency: 100 ms port in "result" }
    functional_exchange "result" { id: "FE-1" from: "SF-1.result" to: "SF-2.result" }

    functional_chain "Main chain" { id: "FC-1" capability: "UC-1" involves: ["SF-1", "SF-2"] latency_budget: 500 ms }
}

state_machine "Lifecycle" {
    kind: "lifecycle"
    initial: "Service"
    state Service { context: ["OA-USR", "OA-ENV"] }
    state Maintenance { context: ["STK-1"] }
    transition Service -> Maintenance { guard: "maintenance due" }
    transition Maintenance -> Service { guard: "maintained" }
}

scenario "Main use case" {
    capability: "UC-1"
    participants: ["SA-USR", "SA-OPR"]
    message "SA-USR" -> "SA-OPR" "asks"
    message "SA-OPR" -> "SA-USR" "answers"
}

exchange_item "Request" { id: "EI-1" mechanism: "EVENT" }

// ---------------------------------------------------------------------------
// Functional vision (what?): functional requirements, modes, breakdown and
// interactions above, functional scenarios
// ---------------------------------------------------------------------------

requirements functional {
    req "FUN-1" "Functional requirement" { priority: High description: "What a function shall do" }
}

trace "SF-1" satisfies "FUN-1" {}
trace "UC-1" satisfies "STK-REQ-1" {}

state_machine "Modes" {
    kind: "functional_modes"
    initial: "Nominal"
    mode Nominal { during: ["Lifecycle.Service"] }
    mode Degraded { during: ["Lifecycle.Service", "Lifecycle.Maintenance"] }
    transition Nominal -> Degraded { guard: "fault" }
    transition Degraded -> Nominal { guard: "recovered" }
}

scenario "Main function at work" {
    participants: ["SF-1", "SF-2"]
    message "SF-1" -> "SF-2" "result"
}

class "Result" { id: "CL-1" value: "float" }

// ---------------------------------------------------------------------------
// Constructional vision (how?): technical requirements, configurations,
// product breakdown, constructional interactions and scenarios
// ---------------------------------------------------------------------------

logical_architecture "{{NAME}} structure" {
    component "First component" {
        id: "LC-1"
        port out "result"
        function "Compute" { id: "LF-1" realizes: "SF-1" }
    }
    component "Second component" {
        id: "LC-2"
        port in "result"
        function "Use" { id: "LF-2" realizes: "SF-2" }
    }
    component_exchange "Result link" { from_port: "LC-1.result" to_port: "LC-2.result" protocol: "CAN" }
}

requirements technical {
    req "TEC-1" "Technical requirement" { priority: Medium description: "What a component shall be" }
}

trace "LC-1" satisfies "TEC-1" {}
trace "LC-2" satisfies "TEC-1" {}

state_machine "Configurations" {
    kind: "technical_configurations"
    initial: "Full"
    state Full { realizes: ["Modes.Nominal"] }
    state Reduced { realizes: ["Modes.Degraded"] }
    transition Full -> Reduced { guard: "component lost" }
    transition Reduced -> Full { guard: "component back" }
}

scenario "Components at work" {
    participants: ["LC-1", "LC-2"]
    message "LC-1" -> "LC-2" "result frame"
}

test_case "TC-1" { verifies: ["STK-REQ-1", "FUN-1", "TEC-1"] method: "test" description: "The main use case, end to end" verdict: "pass" }
