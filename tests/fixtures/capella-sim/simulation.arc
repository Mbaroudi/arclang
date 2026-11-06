// What simulation adds to the hoist imported from Capella (hoist.arc, never
// edited): the physics of the ports, each element's executable model
// (physics/HoistPhysics.mo), two configurations, and cases flying missions
// written in files of their own. A test fixture of tests/capella_to_simulation.rs.

import "hoist.arc"

type "Power plus" { physics: electrical }
type "Power minus" { physics: electrical pole: negative }
type "Signal" { physics: signal }

refine "PN-BATT" {
    model: "HoistPhysics.Pack"
    deploys "LC-ENERGY"
    port "p" { is: "Power plus" }
    port "n" { is: "Power minus" }
}

refine "PN-WINCH" {
    model: "HoistPhysics.Winch"
    deploys "LC-LIFT"
    port "p" { is: "Power plus" }
    port "n" { is: "Power minus" }
    port "cable" { physics: mechanical_translational flange: b }
}

refine "BC-DRIVE" {
    model: "HoistPhysics.Drive"
    port "force_cmd" { is: "Signal" }
}

refine "PN-CABIN" {
    model: "HoistPhysics.Cabin"
    port "cable" { physics: mechanical_translational }
}

refine "BC-SENSE" {
    model: "HoistPhysics.HeightSensor"
    port "height" { is: "Signal" }
}

refine "PN-CTRL" { deploys "LC-CTRL" }

refine "BC-HOLD" {
    model: "HoistPhysics.HeightHold"
    port "height" { is: "Signal" }
    port "force_cmd" { is: "Signal" }
}

refine "LC-ENERGY" { variation: true }
refine "LC-PACK-S" { variant: true }
refine "LC-PACK-L" { variant: true }

configuration "Light duty" { id: "CFG-LIGHT" selects: ["LC-PACK-S"] }
configuration "Heavy duty" { id: "CFG-HEAVY" selects: ["LC-PACK-L"] }

test_case "Light duty, small pack" {
    id: "TC-LIGHT"
    verifies: ["REQ-LIGHT", "REQ-HOLD"]
    method: "test"
    simulate: { configuration: "CFG-LIGHT" mission: "missions/light.mission" stop_time: 10 min sources: ["physics/HoistPhysics.mo"] }
    expects: ["REQ-LIGHT: min(\"PN-BATT\".soc) >= 0.2", "REQ-HOLD: max(\"PN-CABIN\".height) <= 22 m", "REQ-HOLD: final(\"PN-CABIN\".height) >= 19 m"]
}

test_case "Heavy duty, large pack" {
    id: "TC-HEAVY"
    verifies: ["REQ-HEAVY", "REQ-HOLD"]
    method: "test"
    simulate: { configuration: "CFG-HEAVY" mission: "missions/heavy.mission" stop_time: 10 min sources: ["physics/HoistPhysics.mo"] }
    expects: ["REQ-HEAVY: min(\"PN-BATT\".soc) >= 0.2", "REQ-HOLD: max(\"PN-CABIN\".height) <= 22 m", "REQ-HOLD: final(\"PN-CABIN\".height) >= 19 m"]
}

test_case "Every load, large pack" {
    id: "TC-ENVELOPE"
    verifies: ["REQ-HEAVY"]
    method: "test"
    simulate: { configuration: "CFG-HEAVY" mission: "missions/envelope.mission" stop_time: 10 min sources: ["physics/HoistPhysics.mo"] }
    expects: ["min(\"PN-BATT\".soc) >= 0.2"]
}
