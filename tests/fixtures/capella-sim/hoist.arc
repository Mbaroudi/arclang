model "Cargo hoist" {
    name: "Cargo hoist"
    description: "Imported from Capella (hoist.capella)"
}


requirements system {
    req "REQ-LIGHT" "Light duty" {
        id: "REQ-LIGHT"
        capella_id: "bf67226b-19db-52d0-bd6e-776a85fea33e"
        description: "With 100 kg in the cabin, the hoist holds the landing for 10 minutes and keeps 20 % of its charge"
    }
    req "REQ-HEAVY" "Heavy duty" {
        id: "REQ-HEAVY"
        capella_id: "b108399d-009a-5c4b-9efa-7f2ac947e629"
        description: "With 200 kg in the cabin, the hoist holds the landing for 10 minutes and keeps 20 % of its charge"
    }
    req "REQ-HOLD" "Landing hold" {
        id: "REQ-HOLD"
        capella_id: "157b8f75-a4d4-5496-92dd-30817018e2dc"
        description: "The cabin overshoots the 20 m landing by less than 2 m and is within 1 m of it at the end"
    }
}

operational_analysis "Operational Analysis" {
    id: "6df5b210-4682-5015-a3fe-33fdca2ddd6f"
}

system_analysis "Hoist needs" {
    id: "3e531fe0-c412-50c1-9f5b-1c3e4566a555"
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
    system_component "System" {
        id: "9f40e22a-0c08-50f4-8f58-0df6f771fa9d"
    }
}

logical_architecture "Hoist" {
    id: "bd265e9d-9857-5b96-a151-0e1c2e5d01ba"
    component "Hoist" {
        id: "LC-HOIST"
        component "Energy" {
            id: "LC-ENERGY"
            component "Small pack" {
                id: "LC-PACK-S"
                energy: 40 Wh
            }
            component "Large pack" {
                id: "LC-PACK-L"
                energy: 60 Wh
            }
        }
        component "Lifting" {
            id: "LC-LIFT"
        }
        component "Control" {
            id: "LC-CTRL"
        }
    }
}

physical_architecture "Hoist hardware" {
    id: "01b43765-3288-5211-81bf-c1ead2bf74f6"
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
        load: 200 kg
        mass: 150 kg
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

trace "BC-HOLD" satisfies "REQ-HOLD"
trace "PN-BATT" satisfies "REQ-LIGHT"
trace "PN-BATT" satisfies "REQ-HEAVY"

