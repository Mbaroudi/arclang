model "Drone endurance" {
    name: "Drone endurance"
    description: "Imported from Capella (drone.capella)"
}


requirements system {
    req "Hover endurance" "Hover endurance" {
        id: "Hover endurance"
        capella_id: "7f073dd4-60a2-5d51-ba6f-bfc2c514691f"
        description: "With a 2 kg payload at 20 °C, the drone hovers at 50 m for 25 minutes and lands with 20 % of its charge"
        attr_applies_to: "CFG-LONG"
    }
    req "Survey endurance" "Survey endurance" {
        id: "Survey endurance"
        capella_id: "c3b04451-b2e1-5053-9c22-4df5e8f13a22"
        description: "With a 1 kg camera at 20 °C, the survey drone hovers at 50 m for 20 minutes and lands with 20 % of its charge"
        attr_applies_to: "CFG-SURVEY"
    }
    req "Altitude hold" "Altitude hold" {
        id: "Altitude hold"
        capella_id: "1083ad76-6403-54b1-90f4-429e0081f077"
        description: "The climb to 50 m overshoots by less than 5 m, and the drone is still within 5 m of 50 m at the end of the mission"
    }
    req "Operating envelope" "Operating envelope" {
        id: "Operating envelope"
        capella_id: "d30499ac-3e16-509b-bdd7-e3641ec0bcc1"
        description: "From 0 °C to 35 °C and with payloads up to 2 kg, the extended pack keeps the endurance and the reserve of REQ-ENDURANCE"
        attr_applies_to: "CFG-LONG"
    }
}

operational_analysis "Operational Analysis" {
    id: "7193e227-8353-58e1-9f7e-027d9dd91c4e"
}

system_analysis "Mission needs" {
    id: "dd71f676-3a6d-5778-9c1a-9fb65f86efb0"
    function "Measure altitude" {
        id: "SF-ALT"
        latency: 2 ms
        port out "altitude" { id: "b302d388-dc34-5a60-8cd1-a1be1a0e9c57" }
    }
    function "Hold altitude" {
        id: "SF-HOLD"
        latency: 1 ms
        port in "altitude" { id: "483e2800-e770-555f-91a6-7820abe9c40a" }
        port out "thrust" { id: "f1e0f5f3-50ff-5271-bf05-6ae6cdcf5c2f" }
    }
    function "Produce thrust" {
        id: "SF-THRUST"
        latency: 4 ms
        port in "thrust" { id: "73282513-03e1-51a0-b155-c69a93e6c531" }
    }
    functional_exchange "altitude" { id: "altitude" from: "SF-ALT.altitude" to: "SF-HOLD.altitude" }
    functional_exchange "thrust demand" { id: "thrust demand" from: "SF-HOLD.thrust" to: "SF-THRUST.thrust" }
    system_component "System" {
        id: "8934af35-efe1-5c28-bb2e-b695b289de23"
    }
    functional_chain "Altitude loop" {
        id: "FC-ALT"
        latency_budget: 10 ms
        involves: ["SF-ALT", "SF-HOLD", "SF-THRUST"]
    }
}

logical_architecture "Drone" {
    id: "7d4ae01f-fa85-5232-a205-fe45edc9c902"
    component "Air vehicle" {
        id: "LC-UAV"
        component "Energy" {
            id: "LC-ENERGY"
            pv_variation: true
            component "Standard pack" {
                id: "LC-PACK-STD"
                energy: 444 Wh
                mass: 2.6 kg
                pv_variant: true
            }
            component "Extended pack" {
                id: "LC-PACK-EXT"
                energy: 666 Wh
                mass: 3.9 kg
                pv_variant: true
            }
        }
        component "Propulsion" {
            id: "LC-PROP"
        }
        component "Structure" {
            id: "LC-STRUCT"
        }
        component "Flight control" {
            id: "LC-FCS"
        }
    }
}

physical_architecture "Drone hardware" {
    id: "59d347a5-e873-58d6-9ade-bc46cc20b6ce"
    node "Drone" {
        id: "PN-DRONE"
    }
    node "Battery" {
        id: "PN-BATT"
        avionics_load: 25 W
        voltage: 44.4 V
        port "p" { id: "95d54554-93fe-5553-b86b-042253408b65" }
        port "n" { id: "0e96ec52-7edc-5e3d-9014-f7736c3e6bb6" }
        port "mount" { id: "e95dd4e7-da8f-5c00-95bb-f48d9da516d2" }
    }
    node "Propulsion" {
        id: "PN-PROP"
        efficiency: 85 %
        figure_of_merit: 70 %
        max_thrust: 250 N
        rotor_radius: 0.27 m
        rotors: 4
        port "p" { id: "0fd25fa4-e71b-557d-95ae-09c9dd571851" }
        port "n" { id: "745f1965-c1ed-573e-b95b-975badea5a2d" }
        port "lift" { id: "3d71259f-e6c4-51b4-ae7a-f4dff2b1d942" }
        port "PP d91f9819" { id: "d91f9819-5da0-58f7-b7f7-fc6d94cf50ab" }
        behavior_component "Motor drive" {
            id: "BC-ESC"
            port in "thrust_cmd" { id: "1855e003-8c54-5f86-99be-6a48e797c9a1" }
        }
    }
    node "Airframe" {
        id: "PN-FRAME"
        ambient: 20 °C
        mass: 4.5 kg
        payload: 2 kg
        port "lift" { id: "2b8c4114-c172-5c70-9a37-3e762a91789e" }
        port "mount" { id: "68aa6254-c03c-5680-a9ef-adc6abc5d09c" }
        port "PP a01f7d02" { id: "a01f7d02-fc49-50cf-a6ec-f1f50785cfcc" }
        behavior_component "Altimeter" {
            id: "BC-BARO"
            port out "altitude" { id: "8c7adb35-84e9-584f-938b-1f43ffcecc56" }
        }
    }
    node "Flight controller" {
        id: "PN-FC"
        port "PP e61b6041" { id: "e61b6041-6412-59c5-9f7f-1220dd3f1f2b" }
        port "PP f46d227c" { id: "f46d227c-ff36-5875-9c90-89989149a956" }
        behavior_component "Altitude hold" {
            id: "BC-ALT"
            target_altitude: 50 m
            port in "altitude" { id: "7e1b847d-b679-5eb3-bd88-17a55e1f1960" }
            port out "thrust_cmd" { id: "7894812a-df73-5cc0-8acf-d7e1ca625543" }
        }
    }
    physical_link "Power harness plus" { id: "PL-PWR-P" from: "PN-BATT.p" to: "PN-PROP.p" }
    physical_link "Power harness minus" { id: "PL-PWR-N" from: "PN-BATT.n" to: "PN-PROP.n" }
    physical_link "Motor arms" { id: "PL-ARMS" from: "PN-PROP.lift" to: "PN-FRAME.lift" }
    physical_link "Battery tray" { id: "PL-TRAY" from: "PN-FRAME.mount" to: "PN-BATT.mount" }
    physical_link "Sensor bus" { id: "PL-SENS" from: "PN-FRAME.PP a01f7d02" to: "PN-FC.PP e61b6041" }
    physical_link "ESC bus" { id: "PL-ESC" from: "PN-FC.PP f46d227c" to: "PN-PROP.PP d91f9819" }
    physical_exchange "Altitude" { id: "PX-ALT" from: "BC-BARO.altitude" to: "BC-ALT.altitude" }
    physical_exchange "Thrust command" { id: "PX-THRUST" from: "BC-ALT.thrust_cmd" to: "BC-ESC.thrust_cmd" }
}


diagram "[LAB] Drone" {
    view: lab
    shows: ["LC-UAV", "LC-ENERGY", "LC-PACK-STD", "LC-PACK-EXT", "LC-PROP", "LC-STRUCT", "LC-FCS"]
}

diagram "[PAB] Drone hardware" {
    view: pab
    shows: ["PN-BATT", "PN-PROP", "PN-FRAME", "PN-FC", "BC-ALT", "BC-BARO", "BC-ESC"]
}

