model "Battery thermal loop" {}

// A second wired model for the drawing tests: electrical, thermal, mechanical
// and signal connectors, a node with two behavior components and a cycle.
physical_architecture "Thermal hardware" {
    node "Cell stack" {
        id: "PN-CELLS"
        port "plus" { physics: electrical }
        port "minus" { physics: electrical pole: negative }
        port "heat" { physics: thermal flange: b }
        port out "temperature" { physics: signal }
    }
    node "Charger" {
        id: "PN-CHARGER"
        port "dc_p" { physics: electrical }
        port "dc_n" { physics: electrical pole: negative }
        port in "limit" { physics: signal }
    }
    node "Coolant pump" {
        id: "PN-PUMP"
        port "shaft" { physics: mechanical_rotational }
        port "coolant" { physics: thermal }
        port in "speed_cmd" { physics: signal }
    }
    node "Motor" {
        id: "PN-MOTOR"
        port "shaft" { physics: mechanical_rotational flange: b }
        port in "enable" { physics: signal }
    }
    node "Thermal controller" {
        id: "PN-CTRL"
        port in "temperature" { physics: signal }
        port out "speed_cmd" { physics: signal }
        port out "enable" { physics: signal }
        port out "limit" { physics: signal }
        behavior_component "Estimator" {
            id: "BC-EST"
            port in "temperature" { physics: signal }
            port out "margin" { physics: signal }
        }
        behavior_component "Regulator" {
            id: "BC-REG"
            port in "margin" { physics: signal }
            port out "speed_cmd" { physics: signal }
            port out "enable" { physics: signal }
            port out "limit" { physics: signal }
        }
    }
    physical_link "DC plus" { id: "PL-DCP" from: "PN-CHARGER.dc_p" to: "PN-CELLS.plus" protocol: "DC" }
    physical_link "DC minus" { id: "PL-DCN" from: "PN-CHARGER.dc_n" to: "PN-CELLS.minus" protocol: "DC" }
    physical_link "Heat" { id: "PL-HEAT" from: "PN-CELLS.heat" to: "PN-PUMP.coolant" protocol: "Coolant" }
    physical_link "Pump drive" { id: "PL-DRIVE" from: "PN-MOTOR.shaft" to: "PN-PUMP.shaft" protocol: "Shaft" }
    physical_exchange "Temperature" { id: "PX-TEMP" from: "PN-CELLS.temperature" to: "PN-CTRL.temperature" message_type: "T" frequency: 10 Hz }
    physical_exchange "Pump speed" { id: "PX-SPEED" from: "PN-CTRL.speed_cmd" to: "PN-PUMP.speed_cmd" message_type: "S" frequency: 10 Hz }
    physical_exchange "Enable" { id: "PX-EN" from: "PN-CTRL.enable" to: "PN-MOTOR.enable" message_type: "E" frequency: 10 Hz }
    physical_exchange "Limit" { id: "PX-LIM" from: "PN-CTRL.limit" to: "PN-CHARGER.limit" message_type: "L" frequency: 10 Hz }
    physical_link "In" { id: "PL-IN" from: "PN-CTRL.temperature" to: "BC-EST.temperature" protocol: "x" }
    physical_link "Margin" { id: "PL-MARGIN" from: "BC-EST.margin" to: "BC-REG.margin" protocol: "x" }
    physical_link "Out1" { id: "PL-O1" from: "BC-REG.speed_cmd" to: "PN-CTRL.speed_cmd" protocol: "x" }
    physical_link "Out2" { id: "PL-O2" from: "BC-REG.enable" to: "PN-CTRL.enable" protocol: "x" }
    physical_link "Out3" { id: "PL-O3" from: "BC-REG.limit" to: "PN-CTRL.limit" protocol: "x" }
}
