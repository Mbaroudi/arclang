// Fly-by-wire flight control system: pilot inputs become control surface
// deflections through two dissimilar flight control computers. The model
// follows the Arcadia levels and carries what a DO-178C / ARP4754A review
// asks for: DAL on every element, a functional hazard assessment, test
// cases on every requirement, and an ICD on every link.
//
//   arclang check examples/aerospace/flight_control_system.arc
//   arclang gate --standard do178c examples/aerospace/flight_control_system.arc

model FlightControlSystem {
    name: "Primary Flight Control System"
    description: "Fly-by-wire control of ailerons and elevators with a dual-channel computer"
    version: "2.0"
}

requirements system {
    req "SYS-FCS-001" "Pilot input latency" {
        description: "The flight control system shall command the control surfaces within 50 ms of a pilot input"
        priority: "Critical"
        safety_level: "DAL-A"
        verification_method: "test"
    }
    req "SYS-FCS-002" "Redundant control paths" {
        description: "The system shall provide two independent control paths for every primary control surface"
        priority: "Critical"
        safety_level: "DAL-A"
        rationale: "Continued safe flight and landing after any single failure (ARP4754A)"
        verification_method: "analysis"
    }
    req "SYS-FCS-003" "Deflection accuracy" {
        description: "Control surface deflection shall be within 0.5 degrees of the commanded position"
        priority: "High"
        safety_level: "DAL-B"
        verification_method: "analysis"
    }
    req "SYS-FCS-004" "Channel switchover" {
        description: "The backup channel shall take over within 100 ms of a detected primary channel failure"
        priority: "Critical"
        safety_level: "DAL-A"
        verification_method: "test"
    }
}

operational_analysis "Aircraft Flight Control" {
    actor "Pilot" {
        id: "ACT-001"
        description: "Flies the aircraft through the side-stick and monitors the flight control status"
    }
    actor "ATC" {
        id: "ACT-002"
        description: "Air traffic control: provides guidance and coordinates traffic"
    }
    entity "Aircraft" {
        id: "OA-AIRCRAFT"
        activity "Control Aircraft Attitude" {
            id: "OA-001"
            description: "Hold and change pitch and roll as the pilot commands"
        }
        activity "Monitor Flight Control Status" {
            id: "OA-002"
            description: "Keep the crew aware of the state of the control channels"
        }
    }
    operational_capability "Safe Flight Operations" {
        id: "OC-001"
        description: "Fly the aircraft safely through takeoff, cruise and landing"
        phases: ["takeoff", "cruise", "landing"]
        involves: ["ACT-001", "ACT-002", "OA-AIRCRAFT"]
    }
    interaction "Side-stick inputs" { from: "ACT-001" to: "OA-001" }
    interaction "Channel status" { from: "OA-002" to: "ACT-001" }
    interaction "Clearances" { from: "ACT-002" to: "ACT-001" }
}

system_analysis "Flight Control System" {
    actor "Pilot" { id: "SA-PILOT" }
    capability "Attitude control" {
        id: "CAP-001"
        realizes: "OC-001"
        involves: ["SF-001", "SF-002", "SF-003", "SF-004"]
    }
    function "Process Pilot Commands" {
        id: "SF-001"
        description: "Read and validate the side-stick and pedal inputs"
        safety_level: "DAL-A"
        latency: 10 ms
        port out "commands"
    }
    function "Compute Control Laws" {
        id: "SF-002"
        description: "Turn the validated commands into surface deflection orders"
        safety_level: "DAL-A"
        latency: 20 ms
        port in "commands"
        port out "orders"
    }
    function "Drive Control Surfaces" {
        id: "SF-003"
        description: "Position the actuators on the ordered deflection"
        safety_level: "DAL-A"
        latency: 15 ms
        port in "orders"
    }
    function "Monitor Channel Health" {
        id: "SF-004"
        description: "Detect a failed channel and hand control to the other one"
        safety_level: "DAL-A"
    }
    functional_exchange "validated commands" { id: "FE-001" from: "SF-001.commands" to: "SF-002.commands" }
    functional_exchange "deflection orders" { id: "FE-002" from: "SF-002.orders" to: "SF-003.orders" }
    functional_chain "Pilot input to surface" {
        id: "FC-001"
        involves: ["SF-001", "SF-002", "SF-003"]
        latency_budget: 50 ms
    }
    system_component "Flight Control Computer" {
        id: "SC-101"
        description: "The system under design: a dual-channel fly-by-wire computer"
        redundancy: "Dual"
    }
}

logical_architecture "Flight Control Logical Architecture" {
    component "Primary Flight Computer" {
        id: "LC-001"
        safety_level: "DAL-A"
        port in "stick"
        port out "orders"
        port out "health"
        function "Command Processing" { id: "LF-001" realizes: "SF-001" }
        function "Control Law Computation" { id: "LF-002" realizes: "SF-002" }
    }
    component "Backup Flight Computer" {
        id: "LC-002"
        safety_level: "DAL-A"
        description: "Dissimilar hardware and software; monitors the primary and takes over on failure"
        port in "health"
        port out "orders"
        function "Monitor Primary" { id: "LF-003" realizes: "SF-004" }
        function "Takeover Control" { id: "LF-004" realizes: "SF-002" }
    }
    component "Actuator Control" {
        id: "LC-003"
        safety_level: "DAL-A"
        port in "orders"
        port in "backup_orders"
        function "Command Actuators" { id: "LF-005" realizes: "SF-003" }
        function "Monitor Position Feedback" { id: "LF-006" }
    }
    interface "Command Interface" {
        id: "LI-001"
        from: "LC-001"
        to: "LC-003"
        interface_type: "Data"
    }
    interface "Cross-Channel Datalink" {
        id: "LI-002"
        from: "LC-001"
        to: "LC-002"
        interface_type: "Data"
    }
    component_exchange "Surface orders" { from_port: "LC-001.orders" to_port: "LC-003.orders" }
    component_exchange "Backup surface orders" { from_port: "LC-002.orders" to_port: "LC-003.backup_orders" }
    component_exchange "Channel health" { from_port: "LC-001.health" to_port: "LC-002.health" }
}

physical_architecture "Flight Control Physical Architecture" {
    node "FCC-A" {
        id: "PN-001"
        safety_level: "DAL-A"
        processor: "PowerPC MPC7448"
        ram: 256 MB
        deploys "LC-001"
    }
    node "FCC-B" {
        id: "PN-002"
        safety_level: "DAL-A"
        processor: "ARM Cortex-R52 (dissimilar to FCC-A)"
        ram: 256 MB
        deploys "LC-002"
    }
    node "Remote Terminal Unit" {
        id: "PN-003"
        safety_level: "DAL-A"
        processor: "ARM Cortex-R5"
        deploys "LC-003"
    }
    physical_link "ARINC 664 Network" {
        id: "PL-001"
        from: "PN-001"
        to: "PN-003"
        protocol: "ARINC 664 Part 7 (AFDX)"
        bandwidth: 100 Mbps
    }
    physical_link "Backup ARINC 664 Network" {
        id: "PL-002"
        from: "PN-002"
        to: "PN-003"
        protocol: "ARINC 664 Part 7 (AFDX)"
        bandwidth: 100 Mbps
    }
    physical_link "Cross-Channel Link" {
        id: "PL-003"
        from: "PN-001"
        to: "PN-002"
        protocol: "RS-422 discrete and serial"
        voltage: 28 V
    }
}

epbs "Flight Control EPBS" {
    system "Flight Control System" {
        id: "EPBS-001"
        subsystem "Computing Subsystem" {
            id: "EPBS-101"
            item "FCC-A Hardware" {
                id: "EPBS-1001"
                part_number: "FCC-A-001-Rev-C"
                supplier: "Collins Aerospace"
                certification: "DO-254 Level A"
            }
            item "FCC-B Hardware" {
                id: "EPBS-1002"
                part_number: "FCC-B-001-Rev-C"
                supplier: "Collins Aerospace"
                certification: "DO-254 Level A"
            }
            item "Flight Control Software" {
                id: "EPBS-1003"
                version: "v3.2.1"
                supplier: "Internal"
                certification: "DO-178C Level A"
                sloc: 45000
            }
        }
        subsystem "Actuation Subsystem" {
            id: "EPBS-102"
            item "Remote Terminal Unit" {
                id: "EPBS-1004"
                part_number: "RTU-001-Rev-B"
                supplier: "Parker Aerospace"
                certification: "DO-254 Level B"
            }
            item "Aileron Actuator" { id: "EPBS-1005" part_number: "ACT-AIL-001" supplier: "Moog" quantity: 4 }
            item "Elevator Actuator" { id: "EPBS-1006" part_number: "ACT-ELEV-001" supplier: "Moog" quantity: 2 }
        }
        subsystem "Network Subsystem" {
            id: "EPBS-103"
            item "AFDX Switch" { id: "EPBS-1007" part_number: "SW-AFDX-100" supplier: "DDC" ports: 24 }
            item "Network Cables" { id: "EPBS-1008" specification: "ARINC 664 Part 7" }
        }
    }
}

safety_analysis {
    hazard "Loss of Flight Control" {
        id: "HAZ-001"
        description: "No control path left to a primary surface"
        condition: "Catastrophic"
        dal: "DAL-A"
        mitigated_by: ["SYS-FCS-002", "SYS-FCS-004"]
    }
    hazard "Delayed surface response" {
        id: "HAZ-002"
        description: "Surface responds late to a pilot input during landing"
        condition: "Catastrophic"
        dal: "DAL-A"
        mitigated_by: ["SYS-FCS-001"]
    }
    hazard "Inaccurate surface deflection" {
        id: "HAZ-003"
        description: "Surface position differs from the commanded one"
        condition: "Major"
        dal: "DAL-C"
        mitigated_by: ["SYS-FCS-003"]
    }
    fmea "Primary computer processor fault" {
        id: "FMEA-001"
        failure_mode: "Processor halt"
        effect: "Loss of the primary control channel"
        cause: "Single-event upset or hardware fault"
        severity: "Major"
        occurrence: "Remote"
        detection: "Built-in test within 50 ms, cross-channel monitor"
        rpn: 48
    }
}

state_machine "Control channel modes" {
    initial: "Primary"
    mode Primary { description: "FCC-A commands the surfaces, FCC-B monitors" }
    mode Backup { description: "FCC-B commands the surfaces after a primary failure" }
    state Direct { description: "Both computers lost: mechanical direct law" }
    transition Primary -> Backup { trigger: "LF-003" guard: "primary channel failed" timing: 100 ms }
    transition Backup -> Direct { trigger: "LF-003" guard: "backup channel failed" }
}

scenario "Primary channel failure" {
    participants: ["LC-001", "LC-002", "LC-003"]
    message "LC-001" -> "LC-003" "surface orders"
    message "LC-001" -> "LC-002" "channel health"
    message "LC-002" -> "LC-003" "backup surface orders" { timing: 100 ms }
}

test_case "TC-FCS-001" { verifies: ["SYS-FCS-001"] method: "test" description: "Iron bird: stick step to surface movement latency" }
test_case "TC-FCS-002" { verifies: ["SYS-FCS-002"] method: "analysis" description: "Common-mode analysis of the two channels" }
test_case "TC-FCS-003" { verifies: ["SYS-FCS-003"] method: "analysis" description: "Actuator loop accuracy analysis over the flight envelope" }
test_case "TC-FCS-004" { verifies: ["SYS-FCS-004"] method: "test" description: "Injected primary failure, switchover time measured" }

constraint "Pilot input chain within budget" {
    id: "CST-FCS-001"
    assert: sum("FC-001", latency) <= "FC-001".latency_budget
}

trace "LC-001" satisfies "SYS-FCS-001" { rationale: "Command processing and control laws run within one 20 ms frame" }
trace "LC-001" satisfies "SYS-FCS-002" { rationale: "First control path" }
trace "LC-002" satisfies "SYS-FCS-002" { rationale: "Second, dissimilar control path" }
trace "LC-002" satisfies "SYS-FCS-004" { rationale: "The backup monitors the primary and takes over" }
trace "LC-003" satisfies "SYS-FCS-003" { rationale: "Closed-loop position control with feedback monitoring" }
trace "SC-101" satisfies "SYS-FCS-002" { rationale: "The system is specified as a dual-channel computer" }
trace "LC-001" realizes "SC-101"
trace "LC-002" realizes "SC-101"
trace "SF-001" realizes "OA-001"
trace "SF-004" realizes "OA-002"

diagram "Dual channel" {
    view: lab
    shows: ["LC-001", "LC-002", "LC-003"]
}
