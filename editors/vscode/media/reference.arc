// A small Arcadia model that covers the ArcLang constructs: compiled by the
// extension's tests, shown to the assistant as its syntax reference.

model LaneKeeping {
  name: "Lane Keeping Assist"
  description: "Keeps the vehicle in its lane and corrects a departure"
  version: "1.0"
}

type "Periodic task" {
  required: ["wcet"]
  period: 20 ms
}

requirements {
  requirement "REQ-LKA-001" {
    description: "Steering correction shall be commanded within 80 ms of lane departure detection"
    priority: "Critical"
    safety_level: "ASIL-B"
  }
  requirement "REQ-LKA-002" {
    description: "The driver shall be able to override the correction at any time"
    priority: "High"
    safety_level: "ASIL-B"
  }
}

operational_analysis "Operational Analysis" {
  actor "Driver" {
    id: "OA-DRV"
    description: "Drives the vehicle"
  }
  entity "Vehicle" {
    id: "OA-VEH"
    activity "Keep the lane" {
      id: "OA-KEEP"
    }
  }
  operational_capability "Stay in lane" {
    id: "OC-001"
    involves: ["OA-DRV", "OA-VEH"]
  }
  interaction "Steering input" {
    from: "OA-DRV"
    to: "OA-KEEP"
  }
}

system_analysis "System Analysis" {
  actor "Driver" {
    id: "SA-DRV"
  }
  capability "Lane keeping" {
    id: "CAP-001"
    realizes: "OC-001"
    involves: ["SF-DET", "SF-EST", "SF-CMD"]
  }
  function "Detect lane markings" {
    id: "SF-DET"
    is: "Periodic task"
    latency: 30 ms
    wcet: 25 ms
    port out "lane"
  }
  function "Estimate departure" {
    id: "SF-EST"
    is: "Periodic task"
    latency: 15 ms
    wcet: 12 ms
    port in "lane"
    port out "departure"
  }
  function "Command steering" {
    id: "SF-CMD"
    is: "Periodic task"
    latency: 10 ms
    wcet: 8 ms
    port in "departure"
  }
  functional_exchange "lane model" {
    from: "SF-DET.lane"
    to: "SF-EST.lane"
  }
  functional_exchange "departure estimate" {
    from: "SF-EST.departure"
    to: "SF-CMD.departure"
  }
  functional_chain "Lane departure reaction" {
    id: "FC-LKA"
    involves: ["SF-DET", "SF-EST", "SF-CMD"]
    latency_budget: 80 ms
  }
}

logical_architecture "Logical Architecture" {
  component "Camera" {
    id: "LC-CAM"
    safety_level: "ASIL-B"
    port out "lane"
    function "Capture frame" {
      id: "LF-CAP"
      realizes: "SF-DET"
    }
  }
  component "LKA Controller" {
    id: "LC-CTL"
    safety_level: "ASIL-B"
    port in "lane"
    port out "steer"
    function "Plan correction" {
      id: "LF-PLAN"
      realizes: "SF-EST"
    }
  }
  component "Steering Actuator" {
    id: "LC-ACT"
    safety_level: "ASIL-B"
    port in "steer"
    function "Apply steering torque" {
      id: "LF-ACT"
      realizes: "SF-CMD"
    }
  }
  component_exchange "LaneFlow" {
    from_port: "LC-CAM.lane"
    to_port: "LC-CTL.lane"
  }
  component_exchange "SteerCommand" {
    from_port: "LC-CTL.steer"
    to_port: "LC-ACT.steer"
  }
}

physical_architecture "Physical Architecture" {
  node "Camera ECU" {
    id: "PN-CAM"
    ram: 512 MB
    deploys "LC-CAM"
  }
  node "Chassis ECU" {
    id: "PN-CHS"
    ram: 64 MB
    deploys "LC-CTL"
    deploys "LC-ACT"
  }
  physical_link "ChassisCAN" {
    id: "PL-CAN"
    from: "PN-CAM"
    to: "PN-CHS"
    protocol: "CAN FD"
    bandwidth: 2 Mbps
  }
}

safety_analysis {
  hazard "Late steering correction" {
    id: "HAZ-001"
    description: "Correction commanded too late to keep the lane"
    severity: "S2"
    exposure: "E4"
    controllability: "C2"
    asil: "ASIL-B"
    mitigated_by: ["REQ-LKA-001"]
  }
}

state_machine "LKA modes" {
  initial: "Off"
  mode Off { description: "Not armed" }
  mode Armed { description: "Monitoring the lane" }
  mode Correcting { description: "Steering correction active" }
  transition Off -> Armed { trigger: "SF-DET" }
  transition Armed -> Correcting { trigger: "SF-EST" guard: "departure imminent" }
  transition Correcting -> Armed { trigger: "SF-CMD" }
  transition Correcting -> Off { trigger: "SF-CMD" guard: "driver override" }
}

scenario "Lane departure" {
  participants: ["LC-CAM", "LC-CTL", "LC-ACT"]
  message "LC-CAM" -> "LC-CTL" "lane model"
  message "LC-CTL" -> "LC-ACT" "steering correction"
}

exchange_item "LaneModel" {
  id: "EI-LANE"
  mechanism: "FLOW"
}

test_case "TC-LKA-001" {
  id: "TC-LKA-001"
  description: "HIL measurement of departure-to-command latency"
  method: "test"
  verifies: ["REQ-LKA-001"]
}
test_case "TC-LKA-002" {
  id: "TC-LKA-002"
  description: "Driver override demonstration on the track"
  method: "demonstration"
  verifies: ["REQ-LKA-002"]
}

constraint "Reaction time within budget" {
  id: "CST-001"
  assert: ("SF-DET".latency + "SF-EST".latency + "SF-CMD".latency) <= "FC-LKA".latency_budget
}

trace "LC-CTL" satisfies "REQ-LKA-001" { rationale: "Plans the correction within the chain budget" }
trace "LC-CTL" satisfies "REQ-LKA-002" { rationale: "Releases control on driver torque" }
trace "LC-CAM" satisfies "REQ-LKA-001" { rationale: "Delivers the lane model within its frame period" }
trace "LC-ACT" satisfies "REQ-LKA-002" { rationale: "Torque limited so the driver can always override" }
trace "SF-DET" realizes "OA-KEEP"

diagram "Lane keeping chain" {
  view: lab
  shows: ["LC-CAM", "LC-CTL", "LC-ACT"]
}
