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
