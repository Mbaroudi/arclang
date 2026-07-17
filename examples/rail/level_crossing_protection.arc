// Railway level-crossing protection system — a classic rail-signalling
// safety case: detect an approaching train, close the barriers, prove the
// crossing is protected before the train arrives.

model LevelCrossingProtection {
  version: "1.0"
}

operational_analysis "Level Crossing Operations" {
  actor "Train Driver" {
    id: "OA-ACT-001"
    description: "Drives the approaching train"
  }
  actor "Road User" {
    id: "OA-ACT-002"
    description: "Vehicles and pedestrians crossing the track"
  }
  actor "Signaller" {
    id: "OA-ACT-003"
    description: "Supervises the line and handles degraded modes"
  }
  operational_capability "Protect the crossing" {
    id: "OC-001"
    description: "Road traffic is stopped and proven clear before a train passes"
  }
}

requirements safety {
  req "REQ-LX-001" "Train detection" {
    description: "The system shall detect an approaching train at least 30 s before it reaches the crossing"
    priority: "High"
  }
  req "REQ-LX-002" "Barrier closure" {
    description: "Barriers shall be fully closed at least 10 s before the train reaches the crossing"
    priority: "High"
  }
  req "REQ-LX-003" "Crossing clear proving" {
    description: "The system shall prove the crossing area is free of obstacles before authorising train passage"
    priority: "High"
  }
  req "REQ-LX-004" "Fail-safe state" {
    description: "On any internal failure, barriers shall close and the protecting signal shall show stop"
    priority: "High"
  }
}

architecture logical {
  component "Train Detector" {
    id: "LC-LX-001"
    port out TrainApproach { }
    function "Detect approaching train" { latency: "500 ms" }
  }
  component "Crossing Controller" {
    id: "LC-LX-002"
    port in TrainApproach { }
    port in CrossingStatus { }
    port out BarrierCommand { }
    port out SignalCommand { }
    function "Sequence the protection cycle" { latency: "200 ms" }
    function "Enter fail-safe on fault"
  }
  component "Barrier Machine" {
    id: "LC-LX-003"
    port in BarrierCommand { }
    function "Drive the barriers"
  }
  component "Obstacle Detector" {
    id: "LC-LX-004"
    port out CrossingStatus { }
    function "Prove crossing clear" { latency: "300 ms" }
  }
  component "Protecting Signal" {
    id: "LC-LX-005"
    port in SignalCommand { }
    function "Display movement authority"
  }
  component_exchange "train approach" {
    from_port: "LC-LX-001"
    to_port: "LC-LX-002"
  }
  component_exchange "crossing status" {
    from_port: "LC-LX-004"
    to_port: "LC-LX-002"
  }
  component_exchange "barrier command" {
    from_port: "LC-LX-002"
    to_port: "LC-LX-003"
  }
  component_exchange "signal command" {
    from_port: "LC-LX-002"
    to_port: "LC-LX-005"
  }
}

state_machine CrossingModes {
  initial: "Open"
  mode Open { }
  mode Closing { }
  mode Protected { }
  mode Failed { }
  transition Open -> Closing { trigger: "train approach" }
  transition Closing -> Protected { trigger: "crossing status" }
  transition Protected -> Open { trigger: "train approach" }
  transition Closing -> Failed { trigger: "Enter fail-safe on fault" }
  transition Protected -> Failed { trigger: "Enter fail-safe on fault" }
}

trace "LC-LX-001" satisfies "REQ-LX-001" { rationale: "detection chain" }
trace "LC-LX-002" satisfies "REQ-LX-002" { rationale: "protection sequencing" }
trace "LC-LX-003" satisfies "REQ-LX-002" { rationale: "barrier actuation" }
trace "LC-LX-004" satisfies "REQ-LX-003" { rationale: "clear proving" }
trace "LC-LX-002" satisfies "REQ-LX-004" { rationale: "fail-safe logic" }

test_case "TC-LX-001" {
  name: "Detection timing measured on site"
  verifies: ["REQ-LX-001"]
  method: "test"
}
test_case "TC-LX-002" {
  name: "Closure sequence timing analysis"
  verifies: ["REQ-LX-002"]
  method: "analysis"
}
test_case "TC-LX-003" {
  name: "Obstacle scenarios demonstrated"
  verifies: ["REQ-LX-003"]
  method: "demonstration"
}
test_case "TC-LX-004" {
  name: "Fault injection campaign"
  verifies: ["REQ-LX-004"]
  method: "test"
}
