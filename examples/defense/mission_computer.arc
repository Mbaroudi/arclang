// Tactical mission computer of a two-seat combat aircraft: radar and
// electronic-warfare data are fused into tracks, threats are ranked, and a
// weapon is released only after a dual-operator authorization. Airborne
// software levels follow DO-178C; the weapon release path is DAL-A.
//
//   arclang check examples/defense/mission_computer.arc
//   arclang gate --standard do178c examples/defense/mission_computer.arc

model MissionComputer {
    name: "Tactical Mission Computer"
    description: "Sensor fusion, threat evaluation and weapon control of a tactical aircraft"
    version: "2.0"
}

requirements system {
    req "SYS-MC-001" "Radar processing latency" {
        description: "The mission computer shall turn a radar detection into an updated track within 100 ms"
        priority: "Critical"
        safety_level: "DAL-A"
        verification_method: "test"
    }
    req "SYS-MC-002" "Track continuity" {
        description: "The system shall keep track continuity above 99.9 percent for every detected target"
        priority: "Critical"
        safety_level: "DAL-A"
        verification_method: "analysis"
    }
    req "SYS-MC-003" "Dual-operator weapon release" {
        description: "Weapon release shall require the confirmation of both the pilot and the weapons system officer"
        priority: "Critical"
        safety_level: "DAL-A"
        rationale: "No single operator action may release a weapon"
        verification_method: "test"
    }
    req "SYS-MC-004" "Tactical data encryption" {
        description: "All tactical data leaving the mission computer shall be encrypted with an NSA-approved algorithm"
        priority: "Critical"
        safety_level: "DAL-C"
        standard: "FIPS 140-3"
        verification_method: "inspection"
    }
    req "SYS-MC-005" "Contested environment" {
        description: "Threat warning shall keep working under jamming up to the specified level"
        priority: "High"
        safety_level: "DAL-B"
        verification_method: "test"
    }
    req "SYS-MC-006" "Mission data integrity" {
        description: "Stored mission data shall carry tamper detection and an audit trail"
        priority: "High"
        safety_level: "DAL-C"
        verification_method: "inspection"
    }
}

operational_analysis "Tactical Mission System" {
    actor "Pilot" {
        id: "ACT-001"
        description: "Flies the aircraft and gives the first weapon release consent"
    }
    actor "Weapons System Officer" {
        id: "ACT-002"
        description: "Operates the sensors, designates targets and gives the second consent"
    }
    actor "Ground Control" {
        id: "ACT-003"
        description: "Command and control: intelligence, coordination, engagement authority"
    }
    entity "Mission system" {
        id: "OA-MISSION"
        activity "Track Multiple Targets" {
            id: "OA-001"
            description: "Keep situational awareness over up to 20 simultaneous threats"
        }
        activity "Evaluate Threats" {
            id: "OA-003"
            description: "Rank the threats by their danger to the aircraft and the mission"
        }
        activity "Weapon Employment" {
            id: "OA-002"
            description: "Select and release the right ordnance once authorized"
        }
    }
    operational_capability "Air-to-Air Combat" {
        id: "OC-001"
        description: "Detect, track and engage airborne threats"
        involves: ["ACT-001", "ACT-002", "OA-MISSION"]
    }
    operational_capability "Air-to-Ground Strike" {
        id: "OC-002"
        description: "Identify and engage ground targets with precision, under engagement authority"
        involves: ["ACT-001", "ACT-002", "ACT-003", "OA-MISSION"]
    }
    interaction "Target designation" { from: "ACT-002" to: "OA-003" }
    interaction "Release consent" { from: "ACT-001" to: "OA-002" }
    interaction "Engagement authority" { from: "ACT-003" to: "OA-002" }
}

system_analysis "Mission Computer System" {
    actor "Pilot" { id: "SA-PILOT" }
    actor "Weapons System Officer" { id: "SA-WSO" }
    capability "Engage a tracked threat" {
        id: "CAP-001"
        realizes: "OC-001"
        involves: ["SF-001", "SF-002", "SF-003", "SF-004"]
    }
    function "Sensor Management" {
        id: "SF-001"
        description: "Control and coordinate the onboard sensors"
        safety_level: "DAL-A"
        latency: 20 ms
        port out "detections"
    }
    function "Track Management" {
        id: "SF-002"
        description: "Correlate detections and maintain the tracks"
        safety_level: "DAL-A"
        latency: 50 ms
        port in "detections"
        port out "tracks"
    }
    function "Threat Evaluation" {
        id: "SF-003"
        description: "Assess the threat level of each track and prioritize the responses"
        safety_level: "DAL-B"
        latency: 20 ms
        port in "tracks"
        port out "threats"
    }
    function "Weapon Control" {
        id: "SF-004"
        description: "Manage the weapon inventory and release a weapon on authorization"
        safety_level: "DAL-A"
        port in "threats"
    }
    function "Protect Tactical Data" {
        id: "SF-005"
        description: "Encrypt the data links and protect the mission data at rest"
        safety_level: "DAL-C"
    }
    functional_exchange "detections" { id: "FE-001" from: "SF-001.detections" to: "SF-002.detections" }
    functional_exchange "tracks" { id: "FE-002" from: "SF-002.tracks" to: "SF-003.tracks" }
    functional_exchange "prioritized threats" { id: "FE-003" from: "SF-003.threats" to: "SF-004.threats" }
    functional_chain "Detection to track" {
        id: "FC-001"
        involves: ["SF-001", "SF-002"]
        latency_budget: 100 ms
    }
}

logical_architecture "Mission Computer Logical Architecture" {
    component "Radar Processor" {
        id: "LC-001"
        safety_level: "DAL-A"
        description: "AESA radar signal processor"
        port out "radar_detections"
        function "Beam Steering" { id: "LF-001" realizes: "SF-001" }
        function "Target Detection" { id: "LF-002" realizes: "SF-001" }
        function "Doppler Processing" { id: "LF-003" }
    }
    component "Electronic Warfare Processor" {
        id: "LC-002"
        safety_level: "DAL-B"
        description: "Radar warning receiver and countermeasures"
        port out "emitters"
        function "RWR Processing" { id: "LF-004" realizes: "SF-001" }
        function "Countermeasures Control" { id: "LF-005" }
    }
    component "Track Fusion" {
        id: "LC-003"
        safety_level: "DAL-A"
        description: "Multi-sensor track correlation and fusion"
        port in "radar_detections"
        port out "tracks"
        function "Track Association" { id: "LF-006" realizes: "SF-002" }
        function "Track Prediction" { id: "LF-007" realizes: "SF-002" }
        function "Track Quality Assessment" { id: "LF-008" }
    }
    component "Threat Manager" {
        id: "LC-004"
        safety_level: "DAL-B"
        description: "Threat identification and prioritization"
        port in "emitters"
        port in "tracks"
        port out "threats"
        function "Identify Emitters" { id: "LF-009" realizes: "SF-003" }
        function "Calculate Threat Priority" { id: "LF-010" realizes: "SF-003" }
    }
    component "Weapon Manager" {
        id: "LC-005"
        safety_level: "DAL-A"
        description: "Weapon inventory and employment control"
        port in "threats"
        port out "release"
        function "Weapon Inventory" { id: "LF-011" }
        function "Target-Weapon Pairing" { id: "LF-012" realizes: "SF-004" }
        function "Authorization Check" { id: "LF-013" realizes: "SF-004" }
        function "Generate Release Command" { id: "LF-014" realizes: "SF-004" }
    }
    component "Cryptographic Module" {
        id: "LC-006"
        safety_level: "DAL-C"
        description: "Type 1 encryption of the data links and of the mission data"
        certification: "NSA Type 1"
        port in "release"
        function "Encrypt Data" { id: "LF-015" realizes: "SF-005" }
        function "Key Management" { id: "LF-016" }
    }
    component_exchange "Radar detections" { from_port: "LC-001.radar_detections" to_port: "LC-003.radar_detections" }
    component_exchange "Emitter reports" { from_port: "LC-002.emitters" to_port: "LC-004.emitters" }
    component_exchange "Fused tracks" { from_port: "LC-003.tracks" to_port: "LC-004.tracks" }
    component_exchange "Prioritized threats" { from_port: "LC-004.threats" to_port: "LC-005.threats" }
    component_exchange "Release command" { from_port: "LC-005.release" to_port: "LC-006.release" }
}

physical_architecture "Mission Computer Physical Architecture" {
    node "Mission Processor 1" {
        id: "PN-001"
        safety_level: "DAL-A"
        processor: "Multi-core PowerPC, VPX 3U"
        deploys "LC-003"
        deploys "LC-004"
        deploys "LC-005"
    }
    node "Mission Processor 2" {
        id: "PN-002"
        safety_level: "DAL-A"
        description: "Hot standby of processor 1"
        processor: "Multi-core PowerPC, VPX 3U"
        redundancy: "Hot standby"
    }
    node "Radar Signal Processor" {
        id: "PN-003"
        safety_level: "DAL-A"
        processor: "FPGA signal processing board"
        deploys "LC-001"
    }
    node "EW Processor" {
        id: "PN-004"
        safety_level: "DAL-B"
        deploys "LC-002"
    }
    node "Crypto Appliance" {
        id: "PN-005"
        safety_level: "DAL-C"
        deploys "LC-006"
    }
    physical_link "Radar backplane lane" { id: "PL-001" from: "PN-003" to: "PN-001" protocol: "VPX PCIe backplane" bandwidth: 8 Gbps }
    physical_link "EW backplane lane" { id: "PL-002" from: "PN-004" to: "PN-001" protocol: "VPX PCIe backplane" bandwidth: 8 Gbps }
    physical_link "Crypto backplane lane" { id: "PL-004" from: "PN-001" to: "PN-005" protocol: "VPX PCIe backplane" bandwidth: 8 Gbps }
    physical_link "Cross-Channel Link" { id: "PL-003" from: "PN-001" to: "PN-002" protocol: "Gigabit Ethernet, redundant" bandwidth: 1 Gbps }
}

epbs "Mission Computer EPBS" {
    system "Tactical Mission System" {
        id: "EPBS-001"
        subsystem "Mission Processing Subsystem" {
            id: "EPBS-101"
            item "Mission Processor Card" { id: "EPBS-1001" part_number: "MPC-001-Rev-F" supplier: "Raytheon" certification: "DO-254 Level A" quantity: 2 }
            item "Mission Software" { id: "EPBS-1003" version: "v7.3.2" certification: "DO-178C Level A" sloc: 850000 language: "Ada 2012" }
        }
        subsystem "Sensor Processing Subsystem" {
            id: "EPBS-102"
            item "Radar Signal Processor" { id: "EPBS-1004" part_number: "RSP-FPGA-001" supplier: "Northrop Grumman" }
            item "EW Processor" { id: "EPBS-1005" part_number: "EWP-001" supplier: "BAE Systems" }
        }
        subsystem "Security Subsystem" {
            id: "EPBS-103"
            item "Type 1 Crypto Module" { id: "EPBS-1006" part_number: "KIV-77" supplier: "General Dynamics" certification: "NSA Type 1" }
        }
        subsystem "Interconnect Subsystem" {
            id: "EPBS-104"
            item "VPX Backplane" { id: "EPBS-1007" part_number: "VPX-BP-14U" specification: "VITA 46" slots: 14 }
            item "1760 Interface Card" { id: "EPBS-1008" part_number: "MS1760-001" channels: 8 }
        }
    }
}

safety_analysis {
    hazard "Unintended Weapon Release" {
        id: "HAZ-001"
        description: "A weapon is released without the authorization of both operators"
        condition: "Catastrophic"
        dal: "DAL-A"
        mitigated_by: ["SYS-MC-003"]
    }
    hazard "Loss of Track Continuity" {
        id: "HAZ-002"
        description: "The crew loses the picture of the threats around the aircraft"
        condition: "Hazardous"
        dal: "DAL-B"
        mitigated_by: ["SYS-MC-001", "SYS-MC-002"]
    }
    hazard "Threat warning lost under jamming" {
        id: "HAZ-003"
        description: "The radar warning receiver is blinded by a jammer"
        condition: "Major"
        dal: "DAL-C"
        mitigated_by: ["SYS-MC-005"]
    }
    hazard "Tactical data compromised" {
        id: "HAZ-004"
        description: "Tactical or mission data is read or altered by an adversary"
        condition: "Major"
        dal: "DAL-C"
        mitigated_by: ["SYS-MC-004", "SYS-MC-006"]
    }
    fmea "Mission processor halt" {
        id: "FMEA-001"
        failure_mode: "Processor halt"
        effect: "Loss of the primary mission functions until switchover"
        cause: "Hardware fault or software exception"
        severity: "Major"
        occurrence: "Remote (1e-6 per hour)"
        detection: "Built-in test within 100 ms, heartbeat on the cross-channel link"
        rpn: 54
    }
}

state_machine "Weapon release states" {
    initial: "Safe"
    mode Safe { description: "Master arm off, no release possible" }
    mode Armed { description: "Master arm on, target paired" }
    mode Consented { description: "Pilot consent given, waiting for the WSO" }
    mode Released { description: "Release command issued" }
    transition Safe -> Armed { trigger: "LF-012" }
    transition Armed -> Consented { trigger: "LF-013" guard: "pilot consent" }
    transition Consented -> Released { trigger: "LF-014" guard: "WSO consent within 5 s" }
    transition Consented -> Armed { trigger: "LF-013" guard: "consent timed out" timing: 5 s }
    transition Armed -> Safe { trigger: "LF-013" guard: "master arm off" }
}

scenario "Engagement" {
    participants: ["LC-001", "LC-003", "LC-004", "LC-005", "LC-006"]
    message "LC-001" -> "LC-003" "radar detections"
    message "LC-003" -> "LC-004" "fused tracks" { timing: 100 ms }
    message "LC-004" -> "LC-005" "prioritized threats"
    message "LC-005" -> "LC-006" "release command" { type: "async" }
}

constraint "Detection to track within budget" {
    id: "CST-MC-001"
    assert: sum("FC-001", latency) <= "FC-001".latency_budget
}

test_case "TC-MC-001" { verifies: ["SYS-MC-001"] method: "test" description: "Radar detection to track update latency on the integration rig" }
test_case "TC-MC-002" { verifies: ["SYS-MC-002"] method: "analysis" description: "Track continuity statistics over the recorded flight test data" }
test_case "TC-MC-003" { verifies: ["SYS-MC-003"] method: "test" description: "Release attempted with one consent only: no release" }
test_case "TC-MC-004" { verifies: ["SYS-MC-004"] method: "inspection" description: "Cryptographic module certificate and algorithm suite inspected" }
test_case "TC-MC-005" { verifies: ["SYS-MC-005"] method: "test" description: "Threat warning under jamming in the anechoic chamber" }
test_case "TC-MC-006" { verifies: ["SYS-MC-006"] method: "inspection" description: "Audit trail and tamper seals of the mission data store inspected" }

trace "LC-001" satisfies "SYS-MC-001" { rationale: "Detections leave the radar processor within one scan" }
trace "LC-003" satisfies "SYS-MC-001" { rationale: "Track fusion updates the track within the budget" }
trace "LC-003" satisfies "SYS-MC-002" { rationale: "Track prediction coasts a track through sensor gaps" }
trace "LC-002" satisfies "SYS-MC-005" { rationale: "The radar warning receiver sorts emitters under jamming" }
trace "LC-004" satisfies "SYS-MC-005" { rationale: "Emitter identification keeps the threat picture under jamming" }
trace "LC-005" satisfies "SYS-MC-003" { rationale: "The authorization check needs both consents" }
trace "LC-006" satisfies "SYS-MC-004" { rationale: "Type 1 encryption of every tactical data link" }
trace "LC-006" satisfies "SYS-MC-006" { rationale: "Keys and integrity checks of the mission data store" }
trace "SF-002" realizes "OA-001"
trace "SF-003" realizes "OA-003"
trace "SF-004" realizes "OA-002"

diagram "Engagement chain" {
    view: lab
    shows: ["LC-001", "LC-003", "LC-004", "LC-005"]
}
