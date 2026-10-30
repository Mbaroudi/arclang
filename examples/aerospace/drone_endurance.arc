// Hover endurance of an inspection drone, verified by simulation. The
// physical architecture is exported to Modelica (`arclang export -f
// modelica`), the physics is hand-written in physics/DronePhysics.mo, and
// `arclang verify` flies each configuration in OpenModelica: the flight
// controller climbs to 50 m and holds it, the rotors draw the power momentum
// theory gives for the weight they carry, and the battery's own weight
// hangs on the frame. Each configuration has its mission: the long
// endurance drone hovers 25 minutes with 2 kg, the survey drone 20 minutes
// with 1 kg; each requirement applies to its configuration only, and the
// gate wants a passing case in each.

model "Drone endurance" {
    description: "A quadrotor carrying a 2 kg sensor, two battery packs, endurance and altitude hold checked by simulation"
}

// --- Port definitions ------------------------------------------------------

type "Power plus" { physics: electrical }
type "Power minus" { physics: electrical pole: negative }
type "Load path" { physics: mechanical_translational }
type "Signal" { physics: signal }

// --- Requirements ----------------------------------------------------------

system_analysis "Mission needs" {
    requirement "Hover endurance" {
        id: "REQ-ENDURANCE"
        applies_to: ["CFG-LONG"]
        description: "With a 2 kg payload at 20 °C, the drone hovers at 50 m for 25 minutes and lands with 20 % of its charge"
    }
    requirement "Survey endurance" {
        id: "REQ-SURVEY"
        applies_to: ["CFG-SURVEY"]
        description: "With a 1 kg camera at 20 °C, the survey drone hovers at 50 m for 20 minutes and lands with 20 % of its charge"
    }
    requirement "Altitude hold" {
        id: "REQ-ALT-HOLD"
        description: "The climb to 50 m overshoots by less than 5 m, and the drone is still within 5 m of 50 m at the end of the mission"
    }
    requirement "Operating envelope" {
        id: "REQ-ENVELOPE"
        applies_to: ["CFG-LONG"]
        description: "From 0 °C to 35 °C and with payloads up to 2 kg, the extended pack keeps the endurance and the reserve of REQ-ENDURANCE"
    }

    function "Measure altitude" { id: "SF-ALT" latency: 2 ms port out "altitude" }
    function "Hold altitude" {
        id: "SF-HOLD"
        latency: 1 ms
        port in "altitude"
        port out "thrust"
    }
    function "Produce thrust" { id: "SF-THRUST" latency: 4 ms port in "thrust" }
    functional_exchange "altitude" { from: "SF-ALT.altitude" to: "SF-HOLD.altitude" }
    functional_exchange "thrust demand" { from: "SF-HOLD.thrust" to: "SF-THRUST.thrust" }
    functional_chain "Altitude loop" {
        id: "FC-ALT"
        involves: ["SF-ALT", "SF-HOLD", "SF-THRUST"]
        latency_budget: 10 ms
    }
}

// --- Logical architecture: the product line --------------------------------

logical_architecture "Drone" {
    component "Air vehicle" {
        id: "LC-UAV"
        component "Energy" {
            id: "LC-ENERGY"
            variation: true
            component "Standard pack" { id: "LC-PACK-STD" variant: true energy: 444 Wh mass: 2.6 kg }
            component "Extended pack" { id: "LC-PACK-EXT" variant: true energy: 666 Wh mass: 3.9 kg }
        }
        component "Propulsion" { id: "LC-PROP" }
        component "Structure" { id: "LC-STRUCT" }
        component "Flight control" { id: "LC-FCS" }
    }
}

// --- Physical architecture: what is simulated ------------------------------

physical_architecture "Drone hardware" {
    node "Battery" {
        id: "PN-BATT"
        voltage: 44.4 V
        avionics_load: 25 W
        deploys "LC-ENERGY"
        port "p" { is: "Power plus" }
        port "n" { is: "Power minus" }
        port "mount" { is: "Load path" }
    }
    node "Propulsion" {
        id: "PN-PROP"
        rotors: 4
        rotor_radius: 0.27 m
        figure_of_merit: 70 %
        efficiency: 85 %
        max_thrust: 250 N
        deploys "LC-PROP"
        port "p" { is: "Power plus" }
        port "n" { is: "Power minus" }
        port "lift" { physics: mechanical_translational flange: b }
        port in "thrust_cmd" { is: "Signal" }
    }
    node "Airframe" {
        id: "PN-FRAME"
        mass: 4.5 kg
        payload: 2 kg
        ambient: 20 °C
        deploys "LC-STRUCT"
        port "lift" { is: "Load path" }
        port "mount" { physics: mechanical_translational flange: b }
        port out "altitude" { is: "Signal" }
    }
    node "Flight controller" {
        id: "PN-FC"
        deploys "LC-FCS"
        port in "altitude" { is: "Signal" }
        port out "thrust_cmd" { is: "Signal" }
        behavior_component "Altitude hold" {
            id: "BC-ALT"
            target_altitude: 50 m
            port in "altitude" { is: "Signal" }
            port out "thrust_cmd" { is: "Signal" }
        }
    }
    physical_link "Power harness plus" { id: "PL-PWR-P" from: "PN-BATT.p" to: "PN-PROP.p" protocol: "48 V DC" }
    physical_link "Power harness minus" { id: "PL-PWR-N" from: "PN-BATT.n" to: "PN-PROP.n" protocol: "48 V DC" }
    physical_link "Motor arms" { id: "PL-ARMS" from: "PN-PROP.lift" to: "PN-FRAME.lift" protocol: "Structure" }
    physical_link "Battery tray" { id: "PL-TRAY" from: "PN-FRAME.mount" to: "PN-BATT.mount" protocol: "Structure" }
    physical_link "Sensor bus" { id: "PL-SENS" from: "PN-FRAME" to: "PN-FC" protocol: "I2C" }
    physical_link "ESC bus" { id: "PL-ESC" from: "PN-FC" to: "PN-PROP" protocol: "DShot" }
    physical_exchange "Altitude" { id: "PX-ALT" from: "PN-FRAME.altitude" to: "PN-FC.altitude" via: "Sensor bus" message_type: "Barometer" frequency: 50 Hz }
    physical_exchange "Thrust command" { id: "PX-THRUST" from: "PN-FC.thrust_cmd" to: "PN-PROP.thrust_cmd" via: "ESC bus" message_type: "Throttle" frequency: 400 Hz }
    physical_link "Altitude to the loop" { id: "PL-ALT-IN" from: "PN-FC.altitude" to: "BC-ALT.altitude" protocol: "On-chip" }
    physical_link "Thrust from the loop" { id: "PL-THR-OUT" from: "BC-ALT.thrust_cmd" to: "PN-FC.thrust_cmd" protocol: "On-chip" }
}

configuration "Survey" { id: "CFG-SURVEY" selects: ["LC-PACK-STD"] }
configuration "Long endurance" { id: "CFG-LONG" selects: ["LC-PACK-EXT"] }

trace "PN-BATT" satisfies "REQ-ENDURANCE"
trace "PN-BATT" satisfies "REQ-SURVEY"
trace "PN-PROP" satisfies "REQ-ENDURANCE"
trace "BC-ALT" satisfies "REQ-ALT-HOLD"
trace "PN-BATT" satisfies "REQ-ENVELOPE"

// --- Verification by simulation --------------------------------------------

// The survey mission carries a 1 kg camera: one point, set as a sweep of one value.
test_case "Survey mission, standard pack" {
    id: "TC-HOVER-STD"
    verifies: ["REQ-SURVEY", "REQ-ALT-HOLD"]
    method: "analysis"
    simulate: { model: "DronePhysics.SurveyHover" configuration: "CFG-SURVEY" stop_time: 20 min sources: ["physics/DronePhysics.mo"] sweep: ["\"PN-FRAME\".payload = 1 kg"] }
    expects: ["REQ-SURVEY: final(\"PN-PROP\".elapsed) >= 20 min", "REQ-SURVEY: min(\"PN-BATT\".soc) >= 0.2", "REQ-ALT-HOLD: max(\"PN-FRAME\".height) <= 55 m", "REQ-ALT-HOLD: final(\"PN-FRAME\".height) >= 45 m"]
}
test_case "Hover endurance, extended pack" {
    id: "TC-HOVER-EXT"
    verifies: ["REQ-ENDURANCE", "REQ-ALT-HOLD"]
    method: "analysis"
    simulate: { model: "DronePhysics.LongHover" configuration: "CFG-LONG" stop_time: 25 min sources: ["physics/DronePhysics.mo"] }
    expects: ["REQ-ENDURANCE: final(\"PN-PROP\".elapsed) >= 25 min", "REQ-ENDURANCE: min(\"PN-BATT\".soc) >= 0.2", "REQ-ALT-HOLD: max(\"PN-FRAME\".height) <= 55 m", "REQ-ALT-HOLD: final(\"PN-FRAME\".height) >= 45 m"]
}
test_case "Operating envelope, extended pack" {
    id: "TC-ENVELOPE"
    verifies: ["REQ-ENVELOPE"]
    method: "analysis"
    simulate: { model: "DronePhysics.LongHover" configuration: "CFG-LONG" stop_time: 25 min sources: ["physics/DronePhysics.mo"]
                sweep: ["\"PN-FRAME\".payload = 0 kg .. 2 kg step 1 kg", "\"PN-FRAME\".ambient = 0 °C, 20 °C, 35 °C"] }
    expects: ["final(\"PN-PROP\".elapsed) >= 25 min", "min(\"PN-BATT\".soc) >= 0.2"]
}
