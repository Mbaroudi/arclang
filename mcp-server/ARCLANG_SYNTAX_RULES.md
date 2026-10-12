<!-- GENERATED from src/arclang_mcp/resources/syntax_guide.py and reference_model.arc.
     Do not edit: a test fails when this file and the server disagree.
     Regenerate with: python -m arclang_mcp.resources > ARCLANG_SYNTAX_RULES.md -->

# ArcLang syntax rules (language 4.0.0)

The compiler is STRICT: any construct outside the grammar is a compile error
with its line and column. Nothing is skipped silently. Do not guess: check
with `arclang_validate` after every change, and read the `arclang://metamodel`
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
   `epbs <name> { }`.
5. Give every element you will trace or reference an `id: "..."`. The id is
   its stable identity: renaming an element with an id changes nothing else.
6. Traces are top-level: `trace "LC-001" satisfies "REQ-001" { rationale: "..." }`.
   Relations: satisfies, implements, validates, realizes, refines. A trace to
   an unknown element is an error. Verification is not a trace: a `test_case`
   lists the requirements it covers in `verifies: [...]`.
7. Enumerations are spelled as the metamodel spells them: safety levels are
   `"ASIL-D"`, `"DAL-A"`, `"SIL-2"`, `"QM"`; priorities are `"Critical"`,
   `"High"`, `"Medium"`, `"Low"`. A value outside the enumeration is a
   `metamodel:` warning and a gate blocker.
8. Quantities are a number and a unit of the unit table: `25 ms`, `100 Mbps`,
   `512 MB`, `40 %`. An unknown unit is an error; a quantity of the wrong
   dimension for a typed attribute (`power: 3 ms`) is a `metamodel:` warning
   and a gate blocker.
9. Function ports are `port in <name>` or `port out <name>`; component ports
   may also be `inout`; physical node ports have no direction.
10. Multiplicity (logical components and physical nodes): `multiplicity: 4`,
    or `"0..1"`, `"1..*"`, `"*"`.
11. The production gate (`arclang_gate`) needs: every requirement satisfied by
    a trace and verified by a `test_case` (`verifies: ["REQ-001"]`), and, when
    safety levels are declared, a `safety_analysis` with `hazard` entries that
    are `mitigated_by` requirements.

## A model that compiles and passes the gate

```arc
// Reference model served by the ArcLang MCP server.
// The test suite compiles it and runs the production gate on it, so every
// construct shown here is accepted by the compiler this server talks to.
model EmergencyBraking {
    version: "4.0.0"

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
    }

    // Logical Architecture: how the system is organised.
    architecture logical {
        component "ThreatAssessor" {
            id: "LC-001"
            safety_level: "ASIL-D"
            port out threat { data_type: "ThreatLevel" }
            function "Assess threat" { id: "LF-001" latency: 20 ms }
        }

        component "BrakeController" {
            id: "LC-002"
            safety_level: "ASIL-D"
            port in threat { data_type: "ThreatLevel" }
            function "Compute braking force" { id: "LF-002" latency: 10 ms }
        }

        component_exchange ThreatToBrake {
            from_port: "LC-001.threat"
            to_port: "LC-002.threat"
        }
    }

    // Physical Architecture: what it runs on.
    architecture physical {
        node "BrakeECU" {
            id: "PN-001"
            deploys "LC-001"
            deploys "LC-002"
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

    // Verification: the gate needs one test case per requirement.
    test_case "TC-001" {
        description: "Target cut-in at 50 km/h"
        verifies: ["REQ-BRK-001"]
        method: test
    }
}

// Traces: element, relation, element. The rationale says why.
trace "LC-002" satisfies "REQ-BRK-001" { rationale: "Computes and commands the braking force" }
trace "LC-001" satisfies "REQ-BRK-001" { rationale: "Detects the imminent collision" }
trace "LF-001" realizes "SF-001" { rationale: "Logical realisation of the threat assessment" }
```
