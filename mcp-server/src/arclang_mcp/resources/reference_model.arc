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

    // Verification: the gate needs one test case per requirement.
    test_case "TC-001" {
        description: "Target cut-in at 50 km/h"
        verifies: ["REQ-BRK-001"]
        method: test
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
