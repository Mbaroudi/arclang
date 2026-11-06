// The source of capella/hoist.capella (arclang export -f capella -o capella/hoist),
// whose import is hoist.arc. A test fixture of tests/capella_to_simulation.rs.
model "Cargo hoist" {
    name: "Cargo hoist"
    description: "A battery winch lifting a cabin to a landing and holding it there"
}

requirements system {
    req "REQ-LIGHT" "Light duty" {
        id: "REQ-LIGHT"
        description: "With 100 kg in the cabin, the hoist holds the landing for 10 minutes and keeps 20 % of its charge"
    }
    req "REQ-HEAVY" "Heavy duty" {
        id: "REQ-HEAVY"
        description: "With 200 kg in the cabin, the hoist holds the landing for 10 minutes and keeps 20 % of its charge"
    }
    req "REQ-HOLD" "Landing hold" {
        id: "REQ-HOLD"
        description: "The cabin overshoots the 20 m landing by less than 2 m and is within 1 m of it at the end"
    }
}

system_analysis "Hoist needs" {
    function "Measure height" {
        id: "SF-HEIGHT"
        port out "height" { id: "SF-HEIGHT.height" }
    }
    function "Hold the landing" {
        id: "SF-HOLD"
        port in "height" { id: "SF-HOLD.height" }
        port out "force" { id: "SF-HOLD.force" }
    }
    functional_exchange "height" { id: "FE-HEIGHT" from: "SF-HEIGHT.height" to: "SF-HOLD.height" }
}

logical_architecture "Hoist" {
    component "Hoist" {
        id: "LC-HOIST"
        component "Energy" {
            id: "LC-ENERGY"
            component "Small pack" { id: "LC-PACK-S" energy: 40 Wh }
            component "Large pack" { id: "LC-PACK-L" energy: 60 Wh }
        }
        component "Lifting" { id: "LC-LIFT" }
        component "Control" { id: "LC-CTRL" }
    }
}

physical_architecture "Hoist hardware" {
    node "Battery" {
        id: "PN-BATT"
        voltage: 48 V
        port "p" { id: "PN-BATT.p" }
        port "n" { id: "PN-BATT.n" }
    }
    node "Winch" {
        id: "PN-WINCH"
        efficiency: 80 %
        max_force: 6000 N
        standby: 50 W
        port "p" { id: "PN-WINCH.p" }
        port "n" { id: "PN-WINCH.n" }
        port "cable" { id: "PN-WINCH.cable" }
        port "bus" { id: "PN-WINCH.bus" }
        behavior_component "Winch drive" {
            id: "BC-DRIVE"
            port in "force_cmd" { id: "BC-DRIVE.force_cmd" }
        }
    }
    node "Cabin" {
        id: "PN-CABIN"
        mass: 150 kg
        load: 200 kg
        port "cable" { id: "PN-CABIN.cable" }
        port "bus" { id: "PN-CABIN.bus" }
        behavior_component "Height sensor" {
            id: "BC-SENSE"
            port out "height" { id: "BC-SENSE.height" }
        }
    }
    node "Controller" {
        id: "PN-CTRL"
        port "sensor" { id: "PN-CTRL.sensor" }
        port "drive" { id: "PN-CTRL.drive" }
        behavior_component "Height hold" {
            id: "BC-HOLD"
            target_height: 20 m
            port in "height" { id: "BC-HOLD.height" }
            port out "force_cmd" { id: "BC-HOLD.force_cmd" }
        }
    }
    physical_link "Power plus" { id: "PL-PWR-P" from: "PN-BATT.p" to: "PN-WINCH.p" protocol: "48 V DC" }
    physical_link "Power minus" { id: "PL-PWR-N" from: "PN-BATT.n" to: "PN-WINCH.n" protocol: "48 V DC" }
    physical_link "Cable" { id: "PL-CABLE" from: "PN-WINCH.cable" to: "PN-CABIN.cable" protocol: "Steel rope" }
    physical_link "Sensor bus" { id: "PL-SENS" from: "PN-CABIN.bus" to: "PN-CTRL.sensor" protocol: "CAN" }
    physical_link "Drive bus" { id: "PL-DRIVE" from: "PN-CTRL.drive" to: "PN-WINCH.bus" protocol: "CAN" }
    physical_exchange "Height" { id: "PX-HEIGHT" from: "BC-SENSE.height" to: "BC-HOLD.height" via: "Sensor bus" frequency: 50 Hz message_type: "Height" }
    physical_exchange "Force command" { id: "PX-FORCE" from: "BC-HOLD.force_cmd" to: "BC-DRIVE.force_cmd" via: "Drive bus" frequency: 100 Hz message_type: "Force" }
}

trace "PN-BATT" satisfies "REQ-LIGHT"
trace "PN-BATT" satisfies "REQ-HEAVY"
trace "BC-HOLD" satisfies "REQ-HOLD"
