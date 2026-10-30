// Heating a house with a heat pump on the coldest day, verified by
// simulation. The physical architecture is exported to Modelica
// (`arclang export -f modelica`), the physics is hand-written in
// physics/HeatingPhysics.mo, and `arclang verify` runs the design day in
// OpenModelica: the outdoor air follows its daily cycle, the walls lose
// heat to it, the heat pump's capacity and efficiency fall as it gets
// colder, and the thermostat holds 21 °C. The 9 kW unit keeps the house
// comfortable through the night; the 6 kW unit does not.

model "Heat pump heating" {
    description: "A house, its weather, two heat pump sizes, comfort and energy checked by simulation over the design day"
}

// --- Port definitions ------------------------------------------------------

type "Heat" { physics: thermal }
type "Signal" { physics: signal }

// --- Requirements ----------------------------------------------------------

system_analysis "Comfort needs" {
    requirement "Comfort on the design day" {
        id: "REQ-COMFORT"
        description: "On the -7 °C design day, the living space stays between 20 °C and 22 °C around the clock"
    }
    requirement "Energy on the design day" {
        id: "REQ-ENERGY"
        description: "The heat pump uses at most 45 kWh of electricity over the design day"
    }
    requirement "Climate envelope" {
        id: "REQ-CLIMATE"
        description: "For design days from -10 °C to 0 °C, with internal gains from 300 W to 900 W, the living space stays between 20 °C and 22 °C"
    }

    function "Measure room temperature" { id: "SF-ROOM" latency: 10 s port out "temperature" }
    function "Control comfort" {
        id: "SF-COMFORT"
        latency: 1 s
        port in "temperature"
        port out "demand"
    }
    function "Produce heat" { id: "SF-HEAT" latency: 30 s port in "demand" }
    functional_exchange "room temperature" { from: "SF-ROOM.temperature" to: "SF-COMFORT.temperature" }
    functional_exchange "heat demand" { from: "SF-COMFORT.demand" to: "SF-HEAT.demand" }
    functional_chain "Comfort loop" {
        id: "FC-COMFORT"
        involves: ["SF-ROOM", "SF-COMFORT", "SF-HEAT"]
        latency_budget: 1 min
    }
}

// --- Logical architecture: the product line --------------------------------

logical_architecture "Heating" {
    component "House" {
        id: "LC-HOUSE"
        component "Living space" { id: "LC-SPACE" }
        component "Heat generation" {
            id: "LC-GEN"
            variation: true
            component "Compact unit" { id: "LC-HP-6" variant: true heating_capacity: 6 kW }
            component "Standard unit" { id: "LC-HP-9" variant: true heating_capacity: 9 kW }
        }
        component "Room control" { id: "LC-CTRL" }
    }
    component "Climate" { id: "LC-CLIMATE" }
}

// --- Physical architecture: what is simulated ------------------------------

physical_architecture "Heating plant" {
    node "Living space" {
        id: "PN-SPACE"
        // Air, furniture and the inner layer of the walls, in J/K.
        heat_capacity: 1.5e7
        // Walls, windows, roof and ventilation, in W/K.
        envelope_conductance: 200
        internal_gain: 600 W
        initial_temperature: 294.15 K
        deploys "LC-SPACE"
        port "heating" { is: "Heat" }
        port "envelope" { physics: thermal flange: b }
        port out "temperature" { is: "Signal" }
    }
    node "Outdoor air" {
        id: "PN-OUTDOOR"
        design_temperature: 266.15 K
        daily_swing: 4 K
        deploys "LC-CLIMATE"
        port "air" { is: "Heat" }
        port "source" { is: "Heat" }
    }
    node "Heat pump" {
        id: "PN-HP"
        carnot_efficiency: 45 %
        supply_temperature: 308.15 K
        deploys "LC-GEN"
        port "condenser" { physics: thermal flange: b }
        port "evaporator" { physics: thermal flange: b }
        port in "demand" { is: "Signal" }
    }
    node "Thermostat" {
        id: "PN-TSTAT"
        deploys "LC-CTRL"
        port in "temperature" { is: "Signal" }
        port out "demand" { is: "Signal" }
        behavior_component "Comfort controller" {
            id: "BC-COMFORT"
            setpoint: 294.15 K
            port in "temperature" { is: "Signal" }
            port out "demand" { is: "Signal" }
        }
    }
    physical_link "Floor heating" { id: "PL-FLOOR" from: "PN-HP.condenser" to: "PN-SPACE.heating" protocol: "Water circuit" }
    physical_link "Building envelope" { id: "PL-ENVELOPE" from: "PN-SPACE.envelope" to: "PN-OUTDOOR.air" protocol: "Conduction" }
    physical_link "Outdoor unit" { id: "PL-OUTDOOR-UNIT" from: "PN-HP.evaporator" to: "PN-OUTDOOR.source" protocol: "Air coil" }
    physical_link "Room sensor" { id: "PL-SENSOR" from: "PN-SPACE" to: "PN-TSTAT" protocol: "Wired" }
    physical_link "Heat pump bus" { id: "PL-MODBUS" from: "PN-TSTAT" to: "PN-HP" protocol: "Modbus" }
    physical_exchange "Room temperature" { id: "PX-ROOM" from: "PN-SPACE.temperature" to: "PN-TSTAT.temperature" via: "Room sensor" message_type: "Temperature" frequency: 1 Hz }
    physical_exchange "Heat demand" { id: "PX-DEMAND" from: "PN-TSTAT.demand" to: "PN-HP.demand" via: "Heat pump bus" message_type: "Demand" frequency: 1 Hz }
    physical_link "Temperature to the controller" { id: "PL-T-IN" from: "PN-TSTAT.temperature" to: "BC-COMFORT.temperature" protocol: "On-chip" }
    physical_link "Demand from the controller" { id: "PL-D-OUT" from: "BC-COMFORT.demand" to: "PN-TSTAT.demand" protocol: "On-chip" }
}

configuration "Compact" { id: "CFG-COMPACT" selects: ["LC-HP-6"] }
configuration "Standard" { id: "CFG-STANDARD" selects: ["LC-HP-9"] }

trace "PN-HP" satisfies "REQ-COMFORT"
trace "BC-COMFORT" satisfies "REQ-COMFORT"
trace "PN-HP" satisfies "REQ-ENERGY"
trace "PN-HP" satisfies "REQ-CLIMATE"

// --- Verification by simulation --------------------------------------------

test_case "Design day, compact unit" {
    id: "TC-DAY-COMPACT"
    verifies: ["REQ-COMFORT", "REQ-ENERGY"]
    method: "analysis"
    simulate: { model: "HeatingPhysics.CompactDay" configuration: "CFG-COMPACT" stop_time: 24 h sources: ["physics/HeatingPhysics.mo"] }
    expects: ["min(\"PN-SPACE\".room.T) >= 293.15 K", "max(\"PN-SPACE\".room.T) <= 295.15 K", "final(\"PN-HP\".consumed) <= 45 kWh"]
}
test_case "Design day, standard unit" {
    id: "TC-DAY-STANDARD"
    verifies: ["REQ-COMFORT", "REQ-ENERGY"]
    method: "analysis"
    simulate: { model: "HeatingPhysics.StandardDay" configuration: "CFG-STANDARD" stop_time: 24 h sources: ["physics/HeatingPhysics.mo"] }
    expects: ["min(\"PN-SPACE\".room.T) >= 293.15 K", "max(\"PN-SPACE\".room.T) <= 295.15 K", "final(\"PN-HP\".consumed) <= 45 kWh"]
}
test_case "Climate envelope, standard unit" {
    id: "TC-CLIMATE"
    verifies: ["REQ-CLIMATE"]
    method: "analysis"
    simulate: { model: "HeatingPhysics.StandardDay" configuration: "CFG-STANDARD" stop_time: 24 h sources: ["physics/HeatingPhysics.mo"]
                sweep: ["\"PN-OUTDOOR\".design_temperature = 263.15 K .. 273.15 K step 5 K", "\"PN-SPACE\".internal_gain = 300 W, 900 W"] }
    expects: ["min(\"PN-SPACE\".room.T) >= 293.15 K", "max(\"PN-SPACE\".room.T) <= 295.15 K"]
}
