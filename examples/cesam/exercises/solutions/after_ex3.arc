// Cumulative solution of the CESAM tutorial after exercise 3: every mission has a use case, every use case a scenario.
// Start from 01_start.arc; compare with `diff`.

model SmartLock {
    name: "Smart door lock"
    description: "Opens the front door for the residents and the people they let in, keeps everyone else out"
    version: "1.0"
    method: "cesam"
}

// ---------------------------------------------------------------------------
// Operational vision (why?): stakeholders, needs, lifecycle, use cases
// ---------------------------------------------------------------------------

requirements stakeholder {
    stakeholder "Resident" { id: "STK-RES" category: "user" interest: "Enters without looking for a key" influence: High }
    stakeholder "Guest" { id: "STK-GST" category: "user" interest: "Is let in when expected" influence: Medium }
    stakeholder "Installer" { id: "STK-INS" category: "maintainer" interest: "Fits the lock in a standard door in under an hour" influence: Medium }
    stakeholder "Insurer" { id: "STK-INSR" category: "authority" interest: "The door resists as well as a certified mechanical lock" influence: High }

    concern "Keyless entry" { id: "NEED-1" stakeholders: ["STK-RES"] }
    concern "Temporary access" { id: "NEED-2" stakeholders: ["STK-RES", "STK-GST"] }
    concern "Burglary resistance" { id: "NEED-3" stakeholders: ["STK-INSR", "STK-RES"] }
    concern "Easy installation" { id: "NEED-4" stakeholders: ["STK-INS"] }

    req "STK-REQ-1" "Open with a phone" { priority: High frames: ["NEED-1"] description: "A resident opens the door with a registered phone, without taking it out of the pocket" }
    req "STK-REQ-2" "Grant a guest code" { priority: Medium frames: ["NEED-2"] description: "A resident grants a guest a code valid for a time window" }
    req "STK-REQ-3" "Resist like a certified lock" { priority: High frames: ["NEED-3"] description: "The locked door resists attacks as a EN 1303 grade 6 cylinder" }
    req "STK-REQ-4" "Fit a standard door" { priority: Medium frames: ["NEED-4"] description: "The lock fits a standard European door without drilling" }
}

operational_analysis "Front door" {
    actor "Resident" { id: "OA-RES" }
    actor "Guest" { id: "OA-GST" }
    actor "Intruder" { id: "OA-INT" }
    entity "Home" { id: "OA-HOME" }
    interaction "Asks to enter" { id: "OE-1" from: "OA-RES" to: "OA-HOME" data_type: "Access request" }
    interaction "Lets a guest in" { id: "OE-2" from: "OA-RES" to: "OA-GST" data_type: "Access request" }
    interaction "Forces the door" { id: "OE-3" from: "OA-INT" to: "OA-HOME" }
}

system_analysis "Smart lock" {
    actor "Resident" { id: "SA-RES" }
    actor "Guest" { id: "SA-GST" }
    actor "Phone" { id: "SA-PHN" }

    mission "Give the right people access to the home" { id: "MIS-1" }
    mission "Keep everyone else out" { id: "MIS-2" }
    capability "Enter with a phone" { id: "UC-1" mission: "MIS-1" involves: ["SA-RES", "SA-PHN"] }
    capability "Enter with a guest code" { id: "UC-2" mission: "MIS-1" involves: ["SA-GST"] }
    capability "Refuse an intruder" { id: "UC-3" mission: "MIS-2" involves: ["SA-PHN"] }

    function "Authenticate user" { id: "SF-1" latency: 800 ms port in "credential" port out "verdict" }
    function "Command bolt" { id: "SF-2" latency: 400 ms port in "verdict" port out "bolt_command" }
    function "Manage codes" { id: "SF-3" port out "codes" }
    function "Report lock state" { id: "SF-4" port in "bolt_state" }

    functional_exchange "verdict" { id: "FE-1" from: "SF-1.verdict" to: "SF-2.verdict" }

    functional_chain "Phone entry" {
        id: "FC-1"
        capability: "UC-1"
        involves: ["SF-1", "SF-2"]
        latency_budget: 1500 ms
    }
}

requirements functional {
    req "FUN-1" "Authenticate within one second" { priority: High description: "The lock decides on a credential within 1 s of receiving it" }
}

requirements {
    req "FUN-2" "Guest codes expire" { priority: Medium description: "A guest code is refused outside its time window" }
}

// ---------------------------------------------------------------------------
// Constructional vision (how?): the product and its interfaces
// ---------------------------------------------------------------------------

logical_architecture "Smart lock structure" {
    component "Radio module" {
        id: "LC-1"
        port out "credential"
        function "Receive credential" { id: "LF-1" realizes: "SF-1" }
    }
    component "Controller" {
        id: "LC-2"
        port in "credential"
        port out "bolt_command"
        function "Check credential" { id: "LF-2" realizes: "SF-1" }
        function "Store codes" { id: "LF-3" realizes: "SF-3" }
        function "Publish state" { id: "LF-5" realizes: "SF-4" }
    }
    component "Motor drive" {
        id: "LC-3"
        port in "bolt_command"
        function "Drive bolt" { id: "LF-4" realizes: "SF-2" }
    }
    component_exchange "Credential" { from_port: "LC-1.credential" to_port: "LC-2.credential" protocol: "SPI" }
    component_exchange "Bolt command" { from_port: "LC-2.bolt_command" to_port: "LC-3.bolt_command" protocol: "PWM" }
}

requirements technical {
    req "TEC-1" "Bolt actuation under 400 ms" { priority: High description: "The motor drive throws the bolt in less than 400 ms" }
    req "TEC-2" "Two years on four AA cells" { priority: Medium description: "The lock runs two years on four AA cells at ten openings a day" }
}

trace "UC-1" satisfies "STK-REQ-1" {}
trace "UC-2" satisfies "STK-REQ-2" {}
trace "LC-3" satisfies "STK-REQ-3" {}
trace "LC-1" satisfies "STK-REQ-4" {}
trace "LC-3" satisfies "TEC-1" {}
trace "LC-1" satisfies "TEC-2" {}
trace "LC-2" satisfies "FUN-1" {}

test_case "TC-1" { verifies: ["STK-REQ-1"] method: "demonstration" description: "A resident walks to the door with a paired phone in the pocket" verdict: "pass" }
test_case "TC-2" { verifies: ["STK-REQ-2", "FUN-2"] method: "test" description: "A guest code works inside its window and is refused outside it" verdict: "pass" }
test_case "TC-3" { verifies: ["STK-REQ-3"] method: "test" description: "Attack tests of EN 1303 grade 6 on the locked door" verdict: "pass" }
test_case "TC-4" { verifies: ["STK-REQ-4"] method: "demonstration" description: "An installer fits the lock in a standard door, timed" verdict: "pass" }
test_case "TC-5" { verifies: ["FUN-1", "TEC-1"] method: "test" description: "Time from credential to bolt thrown, 100 runs" verdict: "pass" }
test_case "TC-6" { verifies: ["TEC-2"] method: "analysis" description: "Energy budget of ten openings a day over two years" verdict: "pass" }

// ---------------------------------------------------------------------------
// States in the three visions: lifecycle, functional modes, configurations
// ---------------------------------------------------------------------------

state_machine "Lock lifecycle" {
    kind: "lifecycle"
    initial: "Installation"
    state Installation { context: ["STK-INS", "OA-HOME"] }
    state Service { context: ["OA-RES", "OA-GST", "OA-INT"] }
    state Maintenance {}
    transition Installation -> Service { guard: "commissioned" }
    transition Service -> Maintenance { guard: "battery low" }
    transition Maintenance -> Service { guard: "batteries replaced" }
}

state_machine "Lock modes" {
    kind: "functional_modes"
    initial: "Locked"
    mode Locked { during: ["Lock lifecycle.Service"] }
    mode Unlocked { during: ["Lock lifecycle.Service"] }
    mode Pairing { during: ["Lock lifecycle.Installation", "Lock lifecycle.Maintenance"] }
    transition Locked -> Unlocked { trigger: "SF-1" guard: "credential accepted" }
    transition Unlocked -> Locked { trigger: "SF-2" guard: "door closed" }
}

state_machine "Lock configurations" {
    kind: "technical_configurations"
    initial: "Nominal"
    state Nominal { realizes: ["Lock modes.Locked", "Lock modes.Unlocked"] }
    state "Radio only" { realizes: ["Lock lifecycle.Installation"] }
    transition Nominal -> "Radio only" { guard: "pairing button pressed" }
    transition "Radio only" -> Nominal { guard: "phone paired" }
}

// ---------------------------------------------------------------------------
// Scenarios in the three visions
// ---------------------------------------------------------------------------

scenario "Resident comes home" {
    capability: "UC-1"
    participants: ["SA-RES", "SA-PHN"]
    message "SA-RES" -> "SA-PHN" "approaches the door"
    message "SA-PHN" -> "SA-RES" "door unlocked"
}

scenario "Guest is let in" {
    capability: "UC-2"
    participants: ["SA-GST", "SA-RES"]
    message "SA-RES" -> "SA-GST" "sends a code"
    message "SA-GST" -> "SA-RES" "enters"
}

scenario "An intruder tries" {
    capability: "UC-3"
    participants: ["SA-PHN"]
    message "SA-PHN" -> "SA-PHN" "unknown credential refused"
}

scenario "Credential check" {
    participants: ["SF-1", "SF-2"]
    message "SF-1" -> "SF-2" "verdict"
}

scenario "Unlock on the boards" {
    participants: ["LC-1", "LC-2", "LC-3"]
    message "LC-1" -> "LC-2" "credential frame"
    message "LC-2" -> "LC-3" "throw bolt"
}

// ---------------------------------------------------------------------------
// Interfaces: what flows
// ---------------------------------------------------------------------------

exchange_item "Access request" {
    id: "EI-0"
    mechanism: "EVENT"
}

exchange_item "Credential token" {
    id: "EI-1"
    mechanism: "EVENT"
}

class "Guest code" {
    id: "CL-1"
    code: "string"
    valid_from: "datetime"
    valid_until: "datetime"
}
