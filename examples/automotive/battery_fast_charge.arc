// A fast charge kept cool, verified by simulation. The physical
// architecture is exported to Modelica (`arclang export -f modelica`), the
// physics is hand-written in physics/FastChargePhysics.mo, and
// `arclang verify` charges the pack from 10 % to 80 % in OpenModelica: the
// cells heat by their internal resistance, the cooling loop takes the heat
// away, and the battery management unit lowers the charge power when the
// cells get too hot. With a liquid chiller the charge is short and cool;
// with air cooling the cells stay below their limit only because the power
// is cut, and the charge takes too long.

model "Battery fast charge" {
    description: "A 75 kWh pack on a 150 kW charger, two ways to cool it, two requirements checked by simulation"
}

// --- Port definitions ------------------------------------------------------

type "DC plus" { physics: electrical }
type "DC minus" { physics: electrical pole: negative }
type "Heat" { physics: thermal }
type "Signal" { physics: signal }

// --- Requirements ----------------------------------------------------------

system_analysis "Charging needs" {
    requirement "Cells kept below their limit" {
        id: "REQ-CELL-TEMP"
        description: "During a fast charge at 35 °C ambient, right after a highway drive, the cells stay below 55 °C"
    }
    requirement "Fast charge time" {
        id: "REQ-CHARGE-TIME"
        description: "At 35 °C ambient, the pack charges from 10 % to 80 % in 30 minutes at most"
    }
    requirement "Hot climate envelope" {
        id: "REQ-HOT-ENVELOPE"
        description: "From 25 °C to 40 °C ambient, and from 100 kW to 150 kW chargers, the chilled pack charges from 10 % to 80 % in 35 minutes, its cells below 55 °C"
    }

    function "Measure cell temperature" { id: "SF-TEMP" latency: 100 ms port out "temperature" }
    function "Regulate cooling" {
        id: "SF-COOL"
        latency: 50 ms
        port in "temperature"
        port out "pump speed"
        port out "power limit"
    }
    function "Deliver charge power" { id: "SF-CHARGE" latency: 500 ms port in "power limit" }
    function "Remove heat" { id: "SF-HEAT" port in "pump speed" }
    functional_exchange "cell temperature" { from: "SF-TEMP.temperature" to: "SF-COOL.temperature" }
    functional_exchange "charge limit" { from: "SF-COOL.power limit" to: "SF-CHARGE.power limit" }
    functional_exchange "cooling demand" { from: "SF-COOL.pump speed" to: "SF-HEAT.pump speed" }
    functional_chain "Thermal derating loop" {
        id: "FC-DERATE"
        involves: ["SF-TEMP", "SF-COOL", "SF-CHARGE"]
        latency_budget: 1 s
    }
}

// --- Logical architecture: the product line --------------------------------

logical_architecture "Charging" {
    component "Vehicle battery system" {
        id: "LC-BATTSYS"
        component "Cells" { id: "LC-CELLS" }
        component "Thermal management" {
            id: "LC-THERMAL"
            variation: true
            component "Air cooling" { id: "LC-AIR" variant: true cooling_conductance: 60 W/K coolant_inlet: 35 °C }
            component "Liquid chiller" { id: "LC-CHILLER" variant: true cooling_conductance: 500 W/K coolant_inlet: 18 °C }
        }
        component "Battery management" { id: "LC-BMS" }
    }
    component "Charging station" { id: "LC-STATION" }
}

// --- Physical architecture: what is simulated ------------------------------

physical_architecture "Charging hardware" {
    node "Cell stack" {
        id: "PN-CELLS"
        energy: 75 kWh
        voltage: 400 V
        internal_resistance: 40 mOhm
        mass: 420 kg
        ambient: 35 °C
        // Arriving from a highway drive: the cells are warmer than the air.
        warm_up: 8 K
        initial_soc: 10 %
        deploys "LC-CELLS"
        port "plus" { is: "DC plus" }
        port "minus" { is: "DC minus" }
        port "heat" { is: "Heat" }
        port out "temperature" { is: "Signal" }
    }
    node "Fast charger" {
        id: "PN-CHARGER"
        max_power: 150 kW
        deploys "LC-STATION"
        port "dc_p" { is: "DC plus" }
        port "dc_n" { is: "DC minus" }
        port in "limit" { is: "Signal" }
    }
    node "Cooling loop" {
        id: "PN-LOOP"
        deploys "LC-THERMAL"
        port "heat" { is: "Heat" }
        port in "pump_cmd" { is: "Signal" }
    }
    node "Battery management unit" {
        id: "PN-BMU"
        deploys "LC-BMS"
        port in "temperature" { is: "Signal" }
        port out "pump_cmd" { is: "Signal" }
        port out "limit" { is: "Signal" }
        behavior_component "Thermal regulator" {
            id: "BC-THERM"
            pump_on: 30 °C
            derate_start: 45 °C
            derate_full: 53 °C
            port in "temperature" { is: "Signal" }
            port out "pump_cmd" { is: "Signal" }
            port out "limit" { is: "Signal" }
        }
    }
    physical_link "DC plus cable" { id: "PL-DCP" from: "PN-CHARGER.dc_p" to: "PN-CELLS.plus" protocol: "CCS DC" }
    physical_link "DC minus cable" { id: "PL-DCN" from: "PN-CHARGER.dc_n" to: "PN-CELLS.minus" protocol: "CCS DC" }
    physical_link "Cold plate" { id: "PL-PLATE" from: "PN-CELLS.heat" to: "PN-LOOP.heat" protocol: "Coolant" }
    physical_link "Sensor wiring" { id: "PL-SENSE" from: "PN-CELLS" to: "PN-BMU" protocol: "Wired" }
    physical_link "Vehicle CAN" { id: "PL-CAN" from: "PN-BMU" to: "PN-LOOP" protocol: "CAN FD" }
    physical_link "Charge communication" { id: "PL-PLC" from: "PN-BMU" to: "PN-CHARGER" protocol: "ISO 15118" }
    physical_exchange "Cell temperature" { id: "PX-TEMP" from: "PN-CELLS.temperature" to: "PN-BMU.temperature" via: "Sensor wiring" message_type: "CellTemperature" frequency: 10 Hz }
    physical_exchange "Pump command" { id: "PX-PUMP" from: "PN-BMU.pump_cmd" to: "PN-LOOP.pump_cmd" via: "Vehicle CAN" message_type: "PumpSpeed" frequency: 10 Hz }
    physical_exchange "Power limit" { id: "PX-LIMIT" from: "PN-BMU.limit" to: "PN-CHARGER.limit" via: "Charge communication" message_type: "ChargeParameter" frequency: 10 Hz }
    physical_link "Temperature to regulator" { id: "PL-T-IN" from: "PN-BMU.temperature" to: "BC-THERM.temperature" protocol: "On-chip" }
    physical_link "Pump from regulator" { id: "PL-P-OUT" from: "BC-THERM.pump_cmd" to: "PN-BMU.pump_cmd" protocol: "On-chip" }
    physical_link "Limit from regulator" { id: "PL-L-OUT" from: "BC-THERM.limit" to: "PN-BMU.limit" protocol: "On-chip" }
}

configuration "Air cooled" { id: "CFG-AIR" selects: ["LC-AIR"] }
configuration "Chilled" { id: "CFG-CHILLED" selects: ["LC-CHILLER"] }

trace "PN-LOOP" satisfies "REQ-CELL-TEMP"
trace "BC-THERM" satisfies "REQ-CELL-TEMP"
trace "PN-CHARGER" satisfies "REQ-CHARGE-TIME"
trace "PN-LOOP" satisfies "REQ-HOT-ENVELOPE"

// --- Verification by simulation --------------------------------------------

test_case "Fast charge, air cooled" {
    id: "TC-CHARGE-AIR"
    verifies: ["REQ-CELL-TEMP", "REQ-CHARGE-TIME"]
    method: "analysis"
    simulate: { model: "FastChargePhysics.AirCooledCharge" configuration: "CFG-AIR" stop_time: 90 min sources: ["physics/FastChargePhysics.mo"] }
    expects: ["REQ-CELL-TEMP: max(\"PN-CELLS\".core.T) <= 55 °C", "REQ-CHARGE-TIME: final(\"PN-CELLS\".soc) >= 0.8", "REQ-CHARGE-TIME: final(\"PN-CHARGER\".elapsed) <= 30 min"]
}
test_case "Fast charge, chilled" {
    id: "TC-CHARGE-CHILLED"
    verifies: ["REQ-CELL-TEMP", "REQ-CHARGE-TIME"]
    method: "analysis"
    simulate: { model: "FastChargePhysics.ChilledCharge" configuration: "CFG-CHILLED" stop_time: 90 min sources: ["physics/FastChargePhysics.mo"] }
    expects: ["REQ-CELL-TEMP: max(\"PN-CELLS\".core.T) <= 55 °C", "REQ-CHARGE-TIME: final(\"PN-CELLS\".soc) >= 0.8", "REQ-CHARGE-TIME: final(\"PN-CHARGER\".elapsed) <= 30 min"]
}
test_case "Hot climate envelope, chilled" {
    id: "TC-HOT-ENVELOPE"
    verifies: ["REQ-HOT-ENVELOPE"]
    method: "analysis"
    simulate: { model: "FastChargePhysics.ChilledCharge" configuration: "CFG-CHILLED" stop_time: 90 min sources: ["physics/FastChargePhysics.mo"]
                sweep: ["\"PN-CELLS\".ambient = 25 °C .. 40 °C step 5 °C", "\"PN-CHARGER\".max_power = 100 kW, 150 kW"] }
    expects: ["max(\"PN-CELLS\".core.T) <= 55 °C", "final(\"PN-CELLS\".soc) >= 0.8", "final(\"PN-CHARGER\".elapsed) <= 35 min"]
}
