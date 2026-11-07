// A requirement verified by a physical simulation. The physical
// architecture is exported to Modelica (`arclang export -f modelica`), the
// physics is hand-written in physics/EVPhysics.mo, and `arclang verify`
// simulates each configuration in OpenModelica: the long range pack keeps
// its reserve over 300 km at -10 °C, the standard pack does not. Each run
// ends when the 300 km are driven or when the pack is empty, whichever comes
// first; `stop_time` is only a bound. The envelope case sweeps the long range
// pack over the temperatures and speeds the requirement names.
//
// The behavior runs step by step in the debugger (F5, or ArcLang: Run…):
// the drive modes (a state machine), a winter highway trip (a scenario) and
// the speed control loop (a functional chain).

model "EV range simulation" {
    description: "One drivetrain, two battery packs, a range requirement checked by simulation"
}

// --- Port definitions ------------------------------------------------------

type "HV plus" { physics: electrical }
type "HV minus" { physics: electrical pole: negative }
type "Signal" { physics: signal }

// --- Requirements ----------------------------------------------------------

system_analysis "Vehicle needs" {
    requirement "Winter highway range" {
        id: "REQ-RANGE"
        description: "At -10 °C and 100 km/h, the vehicle drives 300 km and keeps 10 % of its charge in reserve"
    }
    requirement "Range envelope" {
        id: "REQ-RANGE-ENVELOPE"
        description: "From -20 °C to 20 °C and from 90 km/h to 110 km/h, the long range vehicle drives 300 km and keeps 10 % of its charge"
    }

    // What the vehicle does on the trip: what the debugger runs.
    function "Measure vehicle speed" {
        id: "SF-SPEED"
        latency: 5 ms
        port out "speed"
    }
    function "Watch state of charge" {
        id: "SF-SOC"
        port out "soc"
    }
    function "Regulate speed" {
        id: "SF-REG"
        latency: 10 ms
        port in "speed"
        port in "soc"
        port out "torque"
    }
    function "Deliver torque" {
        id: "SF-TORQUE"
        latency: 15 ms
        port in "torque"
    }
    functional_exchange "speed" { from: "SF-SPEED.speed" to: "SF-REG.speed" }
    functional_exchange "state of charge" { from: "SF-SOC.soc" to: "SF-REG.soc" }
    functional_exchange "torque request" { from: "SF-REG.torque" to: "SF-TORQUE.torque" }
    functional_chain "Speed control loop" {
        id: "FC-SPEED"
        involves: ["SF-SPEED", "SF-REG", "SF-TORQUE"]
        latency_budget: 50 ms
    }
}

// --- Logical architecture: the product line --------------------------------

logical_architecture "EV" {
    component "Vehicle" {
        id: "LC-EV"
        component "Battery" {
            id: "LC-BATT"
            variation: true
            component "Standard pack" { id: "LC-PACK-STD" variant: true energy: 64 kWh }
            component "Long range pack" { id: "LC-PACK-LR" variant: true energy: 82 kWh }
        }
        component "Drive" { id: "LC-DRIVE" }
        component "Speed control" { id: "LC-SPD" }
        component "Body" { id: "LC-BODY" }
    }
}

// --- Physical architecture: what is simulated ------------------------------

physical_architecture "EV hardware" {
    node "Battery pack" {
        id: "PN-BATT"
        voltage: 400 V
        auxiliary_load: 2.5 kW
        deploys "LC-BATT"
        port "hv_p" { is: "HV plus" }
        port "hv_n" { is: "HV minus" }
    }
    node "Drive unit" {
        id: "PN-DRIVE"
        efficiency: 90 %
        gear_ratio: 9.0
        deploys "LC-DRIVE"
        port "hv_p" { is: "HV plus" }
        port "hv_n" { is: "HV minus" }
        port "shaft" { physics: mechanical_rotational flange: b }
        port in "torque_cmd" { is: "Signal" }
    }
    node "Vehicle body" {
        id: "PN-BODY"
        mass: 2100 kg
        wheel_radius: 0.34 m
        drag_area: 0.62 m²
        rolling_resistance: 0.011
        ambient: -10 °C
        deploys "LC-BODY"
        port "wheel" { physics: mechanical_rotational }
        port out "speed" { is: "Signal" }
    }
    node "Vehicle control unit" {
        id: "PN-VCU"
        port in "speed" { is: "Signal" }
        port out "torque_cmd" { is: "Signal" }
        behavior_component "Speed controller" {
            id: "BC-SPD"
            allocated_component: "LC-SPD"
            target_speed: 100 km/h
            port in "speed" { is: "Signal" }
            port out "torque_cmd" { is: "Signal" }
        }
    }
    physical_link "HV plus line" { id: "PL-HVP" from: "PN-BATT.hv_p" to: "PN-DRIVE.hv_p" protocol: "HV DC 400 V" }
    physical_link "HV minus line" { id: "PL-HVN" from: "PN-BATT.hv_n" to: "PN-DRIVE.hv_n" protocol: "HV DC 400 V" }
    physical_link "Half shafts" { id: "PL-SHAFT" from: "PN-DRIVE.shaft" to: "PN-BODY.wheel" protocol: "Mechanical" }
    physical_link "Body CAN" { id: "PL-CAN-BODY" from: "PN-BODY" to: "PN-VCU" protocol: "CAN FD" }
    physical_link "Drive CAN" { id: "PL-CAN-DRIVE" from: "PN-VCU" to: "PN-DRIVE" protocol: "CAN FD" }
    physical_exchange "Vehicle speed" { id: "PX-SPEED" from: "PN-BODY.speed" to: "PN-VCU.speed" via: "Body CAN" message_type: "VehicleSpeed" frequency: 100 Hz }
    physical_exchange "Torque request" { id: "PX-TORQUE" from: "PN-VCU.torque_cmd" to: "PN-DRIVE.torque_cmd" via: "Drive CAN" message_type: "TorqueRequest" frequency: 100 Hz }
    // Inside the control unit: the node's ports relay the controller's.
    physical_link "Speed to controller" { id: "PL-SPEED-IN" from: "PN-VCU.speed" to: "BC-SPD.speed" protocol: "On-chip" }
    physical_link "Torque from controller" { id: "PL-TORQUE-OUT" from: "BC-SPD.torque_cmd" to: "PN-VCU.torque_cmd" protocol: "On-chip" }
}

// --- Behavior: what runs step by step --------------------------------------

state_machine "Drive modes" {
    id: "SM-DRIVE"
    initial: "Parked"
    mode Parked { description: "Ready, no torque" }
    mode Driving { description: "Cruising at the target speed" }
    mode Reserve { description: "Below 10 % of charge: power limited to reach a charger" }
    mode Stranded { description: "Battery empty: the vehicle stops" }
    transition Parked -> Driving { trigger: "SF-REG" guard: "driver starts the trip" }
    transition Driving -> Reserve { trigger: "SF-SOC" guard: "state of charge below 10 %" }
    transition Driving -> Parked { trigger: "SF-REG" guard: "destination reached" }
    transition Reserve -> Parked { trigger: "SF-REG" guard: "destination reached" }
    transition Reserve -> Stranded { trigger: "SF-SOC" guard: "battery empty" }
}

scenario "Winter highway trip" {
    id: "SC-TRIP"
    participants: ["LC-BODY", "LC-SPD", "LC-DRIVE", "LC-BATT"]
    state "LC-SPD" "Driving"
    message "LC-BODY" -> "LC-SPD" "speed 100 km/h"
    // One pass per 100 km: three to drive the 300 km of the requirement.
    fragment loop "3 times" {
        message "LC-SPD" -> "LC-DRIVE" "torque request"
        message "LC-DRIVE" -> "LC-BATT" "draw power"
        message "LC-BATT" -> "LC-SPD" "state of charge"
    }
    fragment alt {
        operand "state of charge above 10 %" {
            message "LC-SPD" -> "LC-DRIVE" "keep 100 km/h"
        }
        operand "state of charge below 10 %" {
            message "LC-SPD" -> "LC-DRIVE" "limit power: reserve mode"
            state "LC-SPD" "Reserve"
        }
    }
}

configuration "City" { id: "CFG-CITY" selects: ["LC-PACK-STD"] }
configuration "Touring" { id: "CFG-TOURING" selects: ["LC-PACK-LR"] }

trace "PN-BATT" satisfies "REQ-RANGE"
trace "PN-BATT" satisfies "REQ-RANGE-ENVELOPE"

// --- Verification by simulation --------------------------------------------

test_case "Winter range, standard pack" {
    id: "TC-RANGE-STD"
    verifies: ["REQ-RANGE"]
    method: "analysis"
    simulate: { model: "EVPhysics.CityRange" configuration: "CFG-CITY" stop_time: 5 h sources: ["physics/EVPhysics.mo"] }
    expects: ["final(vehicleBody.distance) >= 300 km", "min(\"PN-BATT\".soc) >= 0.1"]
}
test_case "Winter range, long range pack" {
    id: "TC-RANGE-LR"
    verifies: ["REQ-RANGE"]
    method: "analysis"
    simulate: { model: "EVPhysics.TouringRange" configuration: "CFG-TOURING" stop_time: 5 h sources: ["physics/EVPhysics.mo"] }
    expects: ["final(vehicleBody.distance) >= 300 km", "min(\"PN-BATT\".soc) >= 0.1"]
}
test_case "Range envelope, long range pack" {
    id: "TC-RANGE-ENVELOPE"
    verifies: ["REQ-RANGE-ENVELOPE"]
    method: "analysis"
    simulate: { model: "EVPhysics.TouringRange" configuration: "CFG-TOURING" stop_time: 5 h sources: ["physics/EVPhysics.mo"]
                sweep: ["vehicleBody.ambient = -20 °C, 0 °C, 20 °C", "\"PN-VCU\".speedController.target_speed = 90 km/h .. 110 km/h step 10 km/h"] }
    expects: ["final(vehicleBody.distance) >= 300 km", "min(\"PN-BATT\".soc) >= 0.1"]
}
