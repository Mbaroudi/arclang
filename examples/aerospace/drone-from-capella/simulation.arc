// What simulation adds to the drone's architecture, which is authored in
// Capella (capella/drone.capella) and read with
//   arclang import capella/drone.capella -f capella -o drone.arc
// drone.arc is never edited: a new import replaces it, and everything
// below still applies, by the ids Capella gave the elements.
//
// Here: the physics of the ports, the executable model of each element
// (physics/DronePhysics.mo), the product line Capella has no notion of, and
// the cases, each flying a mission written in its own file (missions/).
// ArcLang rolls the element models up along the architecture into one
// executable model per configuration (DroneEndurance.Executable), runs it
// in OpenModelica and judges the requirements.

import "drone.arc"

// --- The physics of the connections ----------------------------------------

type "Power plus" { physics: electrical }
type "Power minus" { physics: electrical pole: negative }
type "Load path" { physics: mechanical_translational }
type "Signal" { physics: signal }

// --- Each element: its connectors and its executable model -----------------

refine "PN-BATT" {
    model: "DronePhysics.Pack"
    deploys "LC-ENERGY"
    port "p" { is: "Power plus" }
    port "n" { is: "Power minus" }
    port "mount" { is: "Load path" }
}

refine "PN-PROP" {
    // The rotors breathe the air of the frame's ambient temperature.
    model: "DronePhysics.Rotors(rho=airframe.rho)"
    deploys "LC-PROP"
    port "p" { is: "Power plus" }
    port "n" { is: "Power minus" }
    port "lift" { physics: mechanical_translational flange: b }
}

refine "BC-ESC" {
    model: "DronePhysics.MotorDrive"
    port "thrust_cmd" { is: "Signal" }
}

refine "PN-FRAME" {
    model: "DronePhysics.Frame"
    deploys "LC-STRUCT"
    port "lift" { is: "Load path" }
    port "mount" { physics: mechanical_translational flange: b }
}

refine "BC-BARO" {
    model: "DronePhysics.Altimeter"
    port "altitude" { is: "Signal" }
}

refine "PN-FC" { deploys "LC-FCS" }

refine "BC-ALT" {
    model: "DronePhysics.AltitudeLoop"
    port "altitude" { is: "Signal" }
    port "thrust_cmd" { is: "Signal" }
}

// --- The product line: two packs, two configurations -----------------------

refine "LC-ENERGY" { variation: true }
refine "LC-PACK-STD" { variant: true }
refine "LC-PACK-EXT" { variant: true }

configuration "Survey" { id: "CFG-SURVEY" selects: ["LC-PACK-STD"] }
configuration "Long endurance" { id: "CFG-LONG" selects: ["LC-PACK-EXT"] }

// --- The cases: a configuration, a mission, what the requirements need ------

test_case "Survey mission, standard pack" {
    id: "TC-SURVEY"
    verifies: ["REQ-SURVEY", "REQ-ALT-HOLD"]
    method: "test"
    simulate: { configuration: "CFG-SURVEY" mission: "missions/survey.mission" stop_time: 20 min sources: ["physics/DronePhysics.mo"] }
    expects: ["REQ-SURVEY: final(\"PN-PROP\".elapsed) >= 20 min", "REQ-SURVEY: min(\"PN-BATT\".soc) >= 0.2", "REQ-ALT-HOLD: max(\"PN-FRAME\".height) <= 55 m", "REQ-ALT-HOLD: final(\"PN-FRAME\".height) >= 45 m"]
}

test_case "Hover endurance, extended pack" {
    id: "TC-ENDURANCE"
    verifies: ["REQ-ENDURANCE", "REQ-ALT-HOLD"]
    method: "test"
    simulate: { configuration: "CFG-LONG" mission: "missions/long_hover.mission" stop_time: 25 min sources: ["physics/DronePhysics.mo"] }
    expects: ["REQ-ENDURANCE: final(\"PN-PROP\".elapsed) >= 25 min", "REQ-ENDURANCE: min(\"PN-BATT\".soc) >= 0.2", "REQ-ALT-HOLD: max(\"PN-FRAME\".height) <= 55 m", "REQ-ALT-HOLD: final(\"PN-FRAME\".height) >= 45 m"]
}

test_case "Operating envelope, extended pack" {
    id: "TC-ENVELOPE"
    verifies: ["REQ-ENVELOPE"]
    method: "test"
    simulate: { configuration: "CFG-LONG" mission: "missions/envelope.mission" stop_time: 25 min sources: ["physics/DronePhysics.mo"] }
    expects: ["final(\"PN-PROP\".elapsed) >= 25 min", "min(\"PN-BATT\".soc) >= 0.2"]
}
