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
