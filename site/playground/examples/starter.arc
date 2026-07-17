// Welcome to ArcLang — MBSE as code for the Arcadia method.
// This starter model compiles clean AND passes the production gate.
// Try breaking it: remove a trace, or change the ASIL to see the gate react.

model StarterSystem {
  version: "1.0"
}

requirements safety {
  req "REQ-001" "Braking latency" {
    description: "The system shall brake within 100 ms of threat detection"
    safety_level: "ASIL-B"
    priority: "High"
  }
  req "REQ-002" "Driver override" {
    description: "The driver shall be able to override automatic braking"
    priority: "Medium"
  }
}

architecture logical {
  component "Threat Detector" {
    id: "LC-001"
    port in SensorData { }
    port out ThreatLevel { }
    function "Assess threat" { latency: "40 ms" }
  }
  component "Brake Controller" {
    id: "LC-002"
    port in ThreatLevel { }
    port out BrakeCommand { }
    function "Compute braking command" { latency: "30 ms" }
    function "Detect driver override"
  }
  component_exchange "threat level" {
    from_port: "LC-001"
    to_port: "LC-002"
  }
}

trace "LC-001" satisfies "REQ-001" { rationale: "detection path" }
trace "LC-002" satisfies "REQ-001" { rationale: "actuation path" }
trace "LC-002" satisfies "REQ-002" { rationale: "override monitoring" }

safety_analysis {
  hazard "HAZ-001" {
    description: "Failure to brake on detected threat"
    severity: "S3"
    exposure: "E4"
    controllability: "C1"
    asil: "ASIL-B"
    mitigated_by: ["REQ-001"]
  }
}

test_case "TC-001" {
  name: "Braking latency measured on bench"
  verifies: ["REQ-001"]
  method: "test"
}
test_case "TC-002" {
  name: "Override demonstrated in vehicle"
  verifies: ["REQ-002"]
  method: "demonstration"
}
