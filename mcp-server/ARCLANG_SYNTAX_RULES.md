<!-- GENERATED from src/arclang_mcp/resources/syntax_guide.py and reference_model.arc.
     Do not edit: a test fails when this file and the server disagree.
     Regenerate with: python -m arclang_mcp.resources > ARCLANG_SYNTAX_RULES.md -->

# ArcLang syntax rules

For the compiler this server runs (the `arclang://metamodel` resource prints
the language version). The compiler is STRICT: any construct
outside the grammar is a compile error with its line and column. Nothing is
skipped silently. Do not guess: check with `arclang_validate` after every
change (`as_json: true` names the rule and the fix of each finding, and
`arclang_rules` says what a rule means), and read the `arclang://metamodel`
resource (or call `arclang_metamodel` with a `kind`) to know which attributes
an element kind accepts and with which type.

## Rules that are most often broken

1. Requirement ids are STRINGS, followed by the title:
   `req "REQ-001" "Title" { description: "..." }`. An id such as `REQ-001`
   without quotes is an error: `-` is not part of an identifier.
2. Names are identifiers (`Driver`, dotted `Vehicle.Monitor`) or strings
   (`"Brake controller"`). Anything with a space or a dash needs quotes.
3. Requirements live in `requirements <kind> { }` (kind: `stakeholder`,
   `system`, `safety` or any identifier) or in a `system_analysis` block.
4. Layers: `operational_analysis <name> { }`, `system_analysis <name> { }`,
   `architecture logical { }` (or `logical_architecture <name> { }`),
   `architecture physical { }` (or `physical_architecture <name> { }`),
   `epbs <name> { }`. `arclang_transition` declares on the next layer what
   realizes the elements of this one (oa-to-sa, sa-to-la, la-to-pa).
5. Give every element you will trace or reference an `id: "..."`. The id is
   its stable identity: renaming an element with an id changes nothing else.
   Ports take one too: `port out threat { id: "LP-001" data_type: "ThreatLevel" }`.
6. Traces are top-level: `trace "LC-001" satisfies "REQ-001" { rationale: "..." }`.
   Relations: satisfies, implements, validates, realizes, refines. A trace to
   an unknown element is an error. Verification is not a trace: a `test_case`
   lists the requirements it covers in `verifies: [...]`.
7. Enumerations are spelled as the metamodel spells them: safety levels are
   `"ASIL-D"`, `"DAL-A"`, `"SIL-2"`, `"QM"`; priorities are `"Critical"`,
   `"High"`, `"Medium"`, `"Low"`. A value outside the enumeration is a
   `metamodel.type` finding and a gate blocker.
8. Quantities are a number and a unit of the unit table, UNQUOTED:
   `latency: 25 ms`, `bandwidth: 2 Mbps`, `ram: 512 MB`, `load: 40 %`.
   A bare number (`latency: 25`) is a `metamodel.unit` warning; an unknown
   unit is an error; a quantity of the wrong dimension for a typed attribute
   (`power: 3 ms`) is a `metamodel.type` finding and a gate blocker.
9. Function ports are `port in <name>` or `port out <name>`; component ports
   may also be `inout`; physical node ports have no direction. Exchanges name
   the ports: `component_exchange X { from_port: "LC-001.threat" to_port: "LC-002.threat" }`,
   `functional_exchange X { from: "SF-001.out" to: "SF-002.in" }`. Prefer the
   named form: the arrow form `"SF-001.p" -> "SF-002.p"` takes the first port's
   name as its own id.
10. Multiplicity (logical components and physical nodes): `multiplicity: 4`,
    or `"0..1"`, `"1..*"`, `"*"`.
11. Reusable definitions are top-level `type "Name" { attributes }` blocks,
    `type "B" extends "A" { }` specializing one; `is: "Name"` on an element
    gives it the type's attributes. Writing one of them differently on the
    element is an error.
12. A `constraint "Name" { assert: <expression> }` compares the model's
    values: `"LF-001".latency + "LF-002".latency <= 50 ms`,
    `sum("FC-001", latency) <= "FC-001".latency_budget`. One that is false is
    a `constraint.violated` finding and a gate blocker.
13. Modes and states: `state_machine Name { initial: "A" mode A { } state B { }
    transition A -> B { trigger: "LF-001" guard: "..." } }`. A `mode` is a
    chosen behavior, a `state` an undergone one. The ends of a transition
    must be declared; a `trigger` that names no declared element is a warning.
14. Scenarios: `scenario Name { participants: ["LC-001", "LC-002"]
    message "LC-001" -> "LC-002" "label" { type: "async" exchange: "X" } }`.
    Participants must be declared elements, and every message end a participant.
15. Data model, top-level: `class Name { id: "..." field: "type" }`,
    `enumeration Name { values: [A, B] }`, `data_type Name { base: "float" unit: "N" }`,
    `exchange_item Name { mechanism: "EVENT" elements: ["EN-001"] }`
    (mechanism: EVENT, FLOW, OPERATION, DATA or SHARED_DATA). An exchange
    names what it carries with `exchange_item: "Name"`.
16. Physical architecture: a `node` `deploys "LC-001"`; a
    `link "Name" { from: "PN-001" to: "PN-002" }` joins two nodes; a
    `physical_path "Name" { involves: ["Link"] }` orders links by name.
17. The production gate (`arclang_gate`) needs: every requirement satisfied by
    a trace and verified by a `test_case` (`verifies: ["REQ-001"]`) whose
    outcome is known and passes, every
    constraint true, no `metamodel.type` finding and, when safety levels are
    declared, a `safety_analysis` with `hazard` entries that are
    `mitigated_by` requirements (for `do178c`, each hazard also states its
    failure condition). The outcome of a case is computed (`simulate:`,
    `execute:`, `asserts:`) or recorded from a test run elsewhere
    (`verdict: "pass"`, `"fail"`); a case that says nothing verifies nothing.
    Never record a verdict for a test nobody ran: leave the case without one
    and tell the engineer what remains to run.

## Verification that runs

A `test_case` computes its verdict instead of recording one when it says how
to run. `execute: { machine: ... step: ... stop_time: ... inputs: [...] }`
runs a state machine: each input is `"<instant>: <event>"` (or
`"<instant>: <variable> = <value>"`), and each line of `expects:` names the
requirement it checks, then an expression on the run: `at(<instant>,
"<Machine>".<State or variable>)`, with `min`, `max`, `final`, `mean` for a
simulated quantity. The same case is run on the model, as C on this computer
(`arclang_sil`), on the target's processor (`arclang_pil`) and in logic
(`arclang_hdl`, `arclang_soc`); `arclang_realize` says which of those stages
each requirement has, and climbs the missing ones with `run: true`.

An `io_binding` is the state machine as a task of a target: its `period`,
its inputs (`"<event> <- gpio 4 rising"`) and its outputs
(`"<State> -> gpio 20"`). `arclang_schedule` says whether every task meets
its period.

```arc
// A case whose verdict is computed: the state machine is run, its expects: judged.
// The same case is run as C by arclang_sil and on the target by arclang_pil.
model Wiper {
    requirements system {
        req "REQ-WIPE" "Slow wiping" { description: "The wiper runs slow when the stalk asks for it" }
    }
    architecture logical { component "Wiper control" { id: "LC-WIPE" } }
    test_case "TC-WIPE" {
        verifies: ["REQ-WIPE"]
        method: "test"
        execute: {
            machine: "Wiper"
            step: 20 ms
            stop_time: 200 ms
            inputs: ["20 ms: stalk slow", "100 ms: stalk off"]
        }
        expects: [
            "REQ-WIPE: at(40 ms, \"Wiper\".Slow) == 1",
            "REQ-WIPE: at(120 ms, \"Wiper\".Slow) == 0"
        ]
    }
}

state_machine "Wiper" {
    initial: "Off"
    state Off {}
    state Slow {}
    transition Off -> Slow { trigger: "stalk slow" }
    transition Slow -> Off { trigger: "stalk off" }
}

io_binding "Wiper task" {
    machine: "Wiper"
    target: "zephyr/litex_vexriscv"
    period: 20 ms
    inputs: ["stalk slow <- gpio 4 rising", "stalk off <- gpio 6 rising"]
    outputs: ["Slow -> gpio 20"]
}

trace "LC-WIPE" satisfies "REQ-WIPE" { rationale: "Runs the wiper" }
```

`simulate: { model: "<Package.Class>" stop_time: ... sources: [...] }` runs
the physics in OpenModelica instead (`arclang_verify`): a hand-written
Modelica class, or the package `arclang_export` writes with `format:
modelica`. A `twin` drives the same physics with measured data: `data:` is
the file and its time column, `drives:` imposes a column on a variable
(`variable <- column unit`), `compares:` holds the model to a measurement
(`variable ~ column unit within tolerance [for N %]`), and `expects:` checks
the requirements on that run. `arclang_twin` runs the twins.

```arc
// A case that simulates the physics, and a twin that drives the same physics with measurements.
model Heating {
    requirements system {
        req "REQ-COMFORT" "Comfort" { description: "The room stays at 20 °C or more" }
    }
    architecture physical {
        node "Outdoor" { id: "PN-OUTDOOR" }
        node "Living space" { id: "PN-SPACE" }
    }
    test_case "TC-DESIGN-DAY" {
        verifies: ["REQ-COMFORT"]
        simulate: { model: "HeatingPhysics.StandardDay" stop_time: 24 h sources: ["physics/HeatingPhysics.mo"] }
        expects: ["REQ-COMFORT: min(\"PN-SPACE\".room.T) >= 20 °C"]
    }
}

twin "January cold day" {
    id: "TW-JANUARY"
    verifies: ["REQ-COMFORT"]
    simulate: { model: "HeatingPhysics.StandardDay" sources: ["physics/HeatingPhysics.mo"] }
    data: { file: "data/house-2026-01-14.csv" time: "timestamp" }
    drives: ["\"PN-OUTDOOR\".T <- outdoor_c °C"]
    compares: ["\"PN-SPACE\".room.T ~ indoor_c °C within 0.5 K for 99 %"]
    expects: ["REQ-COMFORT: min(\"PN-SPACE\".room.T) >= 20 °C"]
}
```

Never write `verdict: "pass"` on a case that says `execute:` or `simulate:`:
its verdict is the run's, and it has none until the run is made.

## Code written by hand

A `correspondence` says that a piece of code written by hand (an Arduino
sketch) IS a state machine of the model: `source:` is the file beside the
model, `period:` the time of one `loop()`, `reads:` and `writes:` tie each
pin to a variable or a state, `state:` ties the sketch's variables to the
machine (`<C expression> is in <State>`, `... is <variable>`, `... is time
in <State>`), `invariant:` bounds the sketch's variables. It claims nothing
by itself: `arclang_prove` proves it or refutes it with a case, and
`arclang_observe` measures a real board against it.

```arc
// Code written by hand (toggle.ino) and the model that says what it is: arclang_prove decides.
model "Toggle" {
    version: "1.0.0"
}

state_machine "Lamp" {
    variables: ["in button: Boolean", "led: Boolean = false"]
    initial: "OffReleased"
    state OffReleased {}
    state OnHeld { entry: "led := true" }
    state OnReleased {}
    state OffHeld { entry: "led := false" }
    transition OffReleased -> OnHeld { guard: "button" }
    transition OnHeld -> OnReleased { guard: "not button" }
    transition OnReleased -> OffHeld { guard: "button" }
    transition OffHeld -> OffReleased { guard: "not button" }
}

correspondence "Toggle sketch" {
    machine: "Lamp"
    source: "toggle.ino"
    period: 10 ms
    reads: ["digitalRead(BUTTON) is button"]
    writes: ["digitalWrite(LED) is led"]
    state: [
        "lit == 0 && was == 0 is in OffReleased",
        "lit == 1 && was == 1 is in OnHeld",
        "lit == 1 && was == 0 is in OnReleased",
        "lit == 0 && was == 1 is in OffHeld",
        "lit == 1 is led"
    ]
    invariant: ["lit == 0 || lit == 1", "was == 0 || was == 1"]
}
```

## Levels of the V, responsibility, documents, risks

A `test_case` says the `level:` of the V it belongs to (`component`,
`subsystem`, `system`, `validation`) and a requirement its plan,
`verification_levels: [...]`: the requirement is verified when a case of
EACH planned level verifies it, and the gate blocks on a level no case
verifies at. A case that states no level is at none.

`organization` and `document` are top-level elements; any element names the
organization that answers for it (`responsible:`) and the documents that
describe it (`documented_by: [...]`), by id or by name. A `risk` is a risk
of the project, not a hazard of the product: `likelihood` and `severity`
(low, medium, high), what it `bears_on` and what it is `mitigated_by`; its
level is computed, never written. All of them are followed by
`arclang_impact` and shown by `arclang_table`.

```arc
// Levels of the V, who answers for an element, what documents it, and a risk of the project.
model Braking {
    requirements system {
        requirement "Stops before the obstacle" {
            id: "REQ-STOP"
            verification_levels: ["system", "validation"]
            responsible: "ORG-CHASSIS"
            documented_by: ["DOC-SRS"]
        }
    }
    architecture logical {
        component "Brake controller" { id: "LC-BRAKE" responsible: "Chassis team" }
    }
    test_case "Braking on the bench" { id: "TC-BENCH" verifies: ["REQ-STOP"] level: "system" verdict: "pass" }
    test_case "Braking on the track" { id: "TC-TRACK" verifies: ["REQ-STOP"] level: "validation" method: "demonstration" verdict: "pass" }
}

organization "Chassis team" { id: "ORG-CHASSIS" }
document "System requirements specification" { id: "DOC-SRS" category: "specification" }
risk "Brake supplier delay" {
    id: "RISK-1"
    likelihood: "medium"
    severity: "high"
    bears_on: ["LC-BRAKE"]
    mitigated_by: ["REQ-STOP"]
}

trace "LC-BRAKE" satisfies "REQ-STOP" { rationale: "Commands the braking" }
```

## Editing a model: precise requests, not a rewrite

Do not rewrite a file to change one thing: the compiler edits the text
itself and keeps comments, order and layout. Call `arclang_workbench` first
to learn the exact ids (it lists every element as `id | kind | name | owner |
attributes`, and one `element` with its relationships and traces; the ids it
lists are accepted even when the text writes none). Then call `arclang_edit`
with a list of requests, applied in order on one text:

    {"op": "add", "keyword": "function", "name": "Log braking", "container": "LC-002"}
    {"op": "set", "element": "Log braking", "key": "description", "value": ""Logs each braking""}
    {"op": "add_trace", "from": "LC-002", "kind": "satisfies", "to": "REQ-BRK-001"}

The other ops are `unset`, `rename` (the identity is kept), `add_raw` (one
declaration as source: an exchange, a message, a transition), `remove`,
`move` (a function reallocated, a component nested elsewhere),
`remove_trace`, `wrap_fragment` and `wrap_invariant` (messages of a scenario
in a fragment, or under a state asserted on a lifeline), `reconnect` (a
relationship to another source or target) and `duplicate` /
`duplicate_many` (copies with ids made unique). The whole list is refused, and nothing
changes, when any request names an unknown element or would leave a model
that does not compile; the answer names the request and the reason. Nothing
is written without `write: true`: the answer is the diff.

## A model that compiles and passes the gate

```arc
// Reference model served by the ArcLang MCP server.
// The test suite compiles it and runs the production gate on it, so every
// construct shown here is accepted by the compiler this server talks to.
model EmergencyBraking {
    version: "4.1.0"

    // Operational Analysis: who does what, before any system exists.
    operational_analysis "Emergency braking in traffic" {
        actor Driver { description: "Vehicle operator" }

        entity Vehicle {
            activity MonitorEnvironment { description: "Observe the road ahead" }
        }

        interaction DriverCommands {
            from: Driver
            to: Vehicle.MonitorEnvironment
        }
    }

    // Requirements: the id is a STRING, then the title, then the attributes.
    requirements safety {
        req "REQ-BRK-001" "Emergency braking activation" {
            description: "The system shall brake when a collision is imminent"
            priority: "Critical"
            safety_level: "ASIL-D"
            // The verification plan: the levels of the V a case must verify it at.
            verification_levels: ["system"]
            documented_by: ["DOC-001"]
        }
    }

    // System Analysis: what the system must do.
    system_analysis Braking {
        function AssessThreat {
            id: "SF-001"
            safety_level: "ASIL-D"
            port in tracked_objects { data_type: "ObjectList" }
            port out threat_level { data_type: "ThreatLevel" }
        }

        function CommandBrakes {
            id: "SF-002"
            safety_level: "ASIL-D"
            port in threat_level { data_type: "ThreatLevel" }
        }

        functional_exchange ThreatFlow {
            from: "SF-001.threat_level"
            to: "SF-002.threat_level"
            exchange_item: "ThreatAssessed"
        }
    }

    // Logical Architecture: how the system is organized.
    architecture logical {
        component "ThreatAssessor" {
            id: "LC-001"
            safety_level: "ASIL-D"
            port out threat { id: "LP-001" data_type: "ThreatLevel" }
            function "Assess threat" { id: "LF-001" latency: 20 ms }
        }

        component "BrakeController" {
            id: "LC-002"
            is: "Periodic task"
            safety_level: "ASIL-D"
            responsible: "ORG-001"
            port in threat { id: "LP-002" data_type: "ThreatLevel" }
            function "Compute braking force" { id: "LF-002" latency: 10 ms }
        }

        component_exchange ThreatToBrake {
            from_port: "LC-001.threat"
            to_port: "LC-002.threat"
            exchange_item: "ThreatAssessed"
        }
    }

    // Physical Architecture: what it runs on.
    architecture physical {
        node "SensorECU" {
            id: "PN-001"
            ram: 512 MB
            deploys "LC-001"
        }

        node "BrakeECU" {
            id: "PN-002"
            is: "Safety ECU"
            ram: 64 MB
            deploys "LC-002"
        }

        link "ChassisCAN" {
            id: "PL-001"
            from: "PN-001"
            to: "PN-002"
            protocol: "CAN FD"
            bandwidth: 2 Mbps
        }

        physical_path "ThreatPath" {
            id: "PP-001"
            involves: ["ChassisCAN"]
        }
    }

    // Safety analysis: the gate needs a hazard when ASIL levels are declared.
    safety_analysis {
        hazard "H-001" {
            description: "No braking when a collision is imminent"
            severity: "S3"
            exposure: "E4"
            controllability: "C3"
            asil: "ASIL-D"
            mitigated_by: ["REQ-BRK-001"]
        }
    }

    // Verification: the gate needs, per requirement, a test case whose outcome
    // is known: computed (simulate:, execute:, asserts:) or, as here, recorded
    // from a test run elsewhere (verdict:).
    test_case "TC-001" {
        description: "Target cut-in at 50 km/h"
        verifies: ["REQ-BRK-001"]
        method: test
        level: "system"
        verdict: "pass"
    }
}

// Reusable definitions: `is: "Periodic task"` gives an element these attributes.
type "Periodic task" {
    period: 20 ms
}

type "ECU" {
    voltage: 12 V
}

type "Safety ECU" extends "ECU" {
    safety_level: "ASIL-D"
}

// Data model: what the exchanges carry.
enumeration ThreatLevel {
    id: "EN-001"
    values: ["None", "Warning", "Critical"]
}

data_type BrakingForce {
    id: "DT-001"
    base: "float"
    unit: "N"
}

class ObjectList {
    id: "CL-001"
    count: "int"
    nearest_range: "float"
}

exchange_item ThreatAssessed {
    id: "EI-001"
    mechanism: "EVENT"
    elements: ["EN-001"]
}

// Modes and states (Arcadia transverse): mode = chosen, state = undergone.
// Who answers for an element (responsible:), what documents it (documented_by:),
// and a risk of the project: its level is computed from likelihood and severity.
organization "Chassis team" {
    id: "ORG-001"
}

document "Braking system requirements" {
    id: "DOC-001"
    category: "specification"
}

risk "Radar supplier delay" {
    id: "RISK-001"
    likelihood: "medium"
    severity: "high"
    bears_on: ["LC-001"]
    mitigated_by: ["REQ-BRK-001"]
}

state_machine BrakingModes {
    initial: "Standby"
    mode Standby { description: "Armed, monitoring" }
    mode Braking { description: "Autonomous braking active" }
    state Failed { description: "Sensor or actuator failure" }
    transition Standby -> Braking { trigger: "LF-001" }
    transition Braking -> Standby { trigger: "LF-002" }
    transition Standby -> Failed { trigger: "SF-001" guard: "sensor_fault" }
}

// Scenario: the messages between participants, in order.
scenario EmergencyStop {
    participants: ["LC-001", "LC-002"]
    message "LC-001" -> "LC-002" "critical threat" { type: "async" exchange: "ThreatToBrake" }
}

// Constraint: an assertion on the model's values, checked by the gate.
constraint "Reaction time" {
    id: "CST-001"
    description: "Assessment and braking computation fit a 50 ms budget"
    assert: "LF-001".latency + "LF-002".latency <= 50 ms
}

// Traces: element, relation, element. The rationale says why.
trace "LC-002" satisfies "REQ-BRK-001" { rationale: "Computes and commands the braking force" }
trace "LC-001" satisfies "REQ-BRK-001" { rationale: "Detects the imminent collision" }
trace "LF-001" realizes "SF-001" { rationale: "Logical realization of the threat assessment" }
trace "LF-002" realizes "SF-002" { rationale: "Logical realization of the braking command" }
```
