// Direct liquid cooling of an AI training hall, verified by simulation.
// Sixteen GPU racks of 120 kW give most of their heat to cold plates; a
// coolant distribution unit (CDU) carries it from the racks' loop to the
// facility water; dry coolers give it to the outside air, a chiller helps
// on hot afternoons. The physical architecture is exported to Modelica
// (`arclang export -f modelica`), the physics is hand-written in
// physics/CoolingPhysics.mo, and `arclang verify` runs each scenario in
// OpenModelica: the design summer day, a CDU pump failing in the middle of
// a training run, a heat wave, and the climate and load envelope.
//
// The behaviour runs step by step in the debugger (F5, or ArcLang: Run…):
// the cooling modes (a state machine), a pump failure during a training
// run (a scenario) and the leak isolation chain (a functional chain).

model "AI liquid cooling" {
    description: "Direct-to-chip liquid cooling of an AI training hall: racks, CDU, facility water, dry coolers and chiller, checked by simulation"
}

// --- Port definitions ------------------------------------------------------

type "Heat" { physics: thermal }
type "Signal" { physics: signal }

// --- Operational analysis: who needs what ----------------------------------

operational_analysis "AI hall operations" {
    actor "Hall operator" { id: "OA-OPERATOR" description: "Runs the facility, answers alarms, plans maintenance" }
    actor "Training scheduler" { id: "OA-SCHEDULER" description: "Places training jobs on the GPU racks and can cap their power" }
    actor "Utility" { id: "OA-UTILITY" description: "Supplies the electricity the hall and its cooling use" }
    entity "AI training hall" {
        id: "OA-HALL"
        activity "Run training jobs" { id: "OA-TRAIN" description: "Keep thousands of GPUs at full power for days" }
        activity "Remove the heat" { id: "OA-COOL" description: "Carry 2 MW from the chips to the outside air" }
        activity "Contain a leak" { id: "OA-LEAK" description: "Stop coolant reaching the electronics" }
    }
}

// --- Requirements ----------------------------------------------------------

system_analysis "Cooling needs" {
    requirement "Chip temperature" {
        id: "REQ-GPU"
        description: "On the design summer day (35 °C), every GPU cold plate stays at or below 85 °C at full training load"
    }
    requirement "No throttling" {
        id: "REQ-THROTTLE"
        description: "The cooling never makes the scheduler cap the GPUs' power on the design summer day"
    }
    requirement "Coolant supply" {
        id: "REQ-SUPPLY"
        description: "The coolant reaches the cold plates between 20 °C (no condensation) and 40 °C"
    }
    requirement "Cooling efficiency" {
        id: "REQ-PUE"
        description: "Over the design summer day, the cooling uses at most 10 % of the energy the IT uses (partial PUE at most 1.10)"
    }
    requirement "Pump redundancy" {
        id: "REQ-N1"
        description: "A CDU pump failing at full load leaves every GPU cold plate at or below 85 °C, without throttling"
    }
    requirement "Climate and load envelope" {
        id: "REQ-ENVELOPE"
        description: "From 30 °C to 45 °C outside and from 100 kW to 132 kW per rack, every GPU cold plate stays at or below 85 °C without throttling"
    }
    requirement "Throttling risk" {
        id: "REQ-RISK"
        description: "Over the summer peaks the site sees (35 °C, give or take 3 K) and racks from 110 kW to 132 kW, at least 95 % of the hottest days pass without throttling"
    }
    requirement "Leak isolation" {
        id: "REQ-LEAK"
        description: "A coolant leak in a rack is isolated within 10 s of reaching the floor sensor"
    }

    function "Measure coolant supply" { id: "SF-MEASURE" latency: 1 s port out "temperature" }
    function "Control the supply temperature" {
        id: "SF-CONTROL"
        latency: 1 s
        port in "temperature"
        port out "command"
    }
    function "Reject heat outside" { id: "SF-REJECT" latency: 60 s port in "command" }
    functional_exchange "supply temperature" { from: "SF-MEASURE.temperature" to: "SF-CONTROL.temperature" }
    functional_exchange "cooling command" { from: "SF-CONTROL.command" to: "SF-REJECT.command" }
    functional_chain "Supply temperature loop" {
        id: "FC-SUPPLY"
        involves: ["SF-MEASURE", "SF-CONTROL", "SF-REJECT"]
        latency_budget: 2 min
    }

    function "Detect a leak" { id: "SF-LEAK-DETECT" latency: 2 s port out "alarm" }
    function "Decide isolation" {
        id: "SF-LEAK-DECIDE"
        latency: 500 ms
        port in "alarm"
        port out "close"
    }
    function "Close the rack valves" { id: "SF-LEAK-CLOSE" latency: 6 s port in "close" }
    functional_exchange "leak alarm" { from: "SF-LEAK-DETECT.alarm" to: "SF-LEAK-DECIDE.alarm" }
    functional_exchange "valve command" { from: "SF-LEAK-DECIDE.close" to: "SF-LEAK-CLOSE.close" }
    functional_chain "Leak isolation" {
        id: "FC-LEAK"
        involves: ["SF-LEAK-DETECT", "SF-LEAK-DECIDE", "SF-LEAK-CLOSE"]
        latency_budget: 10 s
    }
}

// --- Logical architecture: the product line --------------------------------

logical_architecture "Liquid cooling" {
    component "AI hall" {
        id: "LC-HALL"
        component "GPU racks" { id: "LC-RACKS" }
        component "Coolant distribution" {
            id: "LC-CDU"
            variation: true
            component "CDU, two pumps" { id: "LC-CDU-N" variant: true pump_count: 2 }
            component "CDU, three pumps (N+1)" { id: "LC-CDU-N1" variant: true pump_count: 3 }
        }
        component "Facility water" { id: "LC-FWS" }
        component "Heat rejection" { id: "LC-REJECT" }
        component "Cooling control" { id: "LC-CONTROL" }
        component "Leak protection" { id: "LC-LEAK" }
    }
    component "Climate" { id: "LC-CLIMATE" }
}

// --- Physical architecture: what is simulated ------------------------------

physical_architecture "Cooling plant" {
    node "GPU racks" {
        id: "PN-RACKS"
        rack_count: 16
        rack_power: 120 kW
        // Cold plates take the GPUs', CPUs' and switches' heat; fans and
        // power supplies give the rest to the room air.
        liquid_fraction: 85 %
        gpu_power: 1000 W
        // Per GPU, from the chip case to the coolant.
        cold_plate_conductance: 33 W/K
        idle_load: 30 %
        deploys "LC-RACKS"
        port "cold_plates" { is: "Heat" }
        port in "coolant_flow" { is: "Signal" }
        port in "power_cap" { is: "Signal" }
        port out "gpu_temperature" { is: "Signal" }
    }
    node "Coolant distribution unit" {
        id: "PN-CDU"
        pump_flow: 22 kg/s
        needed_pumps: 2
        pump_power: 7.5 kW
        hx_conductance: 450 kW/K
        coolant_capacity: 8 MJ/K
        deploys "LC-CDU"
        port "secondary" { physics: thermal flange: b }
        port "primary" { is: "Heat" }
        port out "coolant_flow" { is: "Signal" }
        port out "supply_temperature" { is: "Signal" }
    }
    node "Facility water loop" {
        id: "PN-FWS"
        water_capacity: 40 MJ/K
        deploys "LC-FWS"
        port "from_cdu" { physics: thermal flange: b }
        port "to_coolers" { is: "Heat" }
    }
    node "Dry coolers and chiller" {
        id: "PN-REJECT"
        cooler_conductance: 280 kW/K
        fan_power: 45 kW
        chiller_capacity: 900 kW
        chiller_efficiency: 35 %
        deploys "LC-REJECT"
        port "water" { physics: thermal flange: b }
        port "air" { is: "Heat" }
        port in "command" { is: "Signal" }
    }
    node "Outdoor air" {
        id: "PN-OUTDOOR"
        design_temperature: 35 °C
        daily_swing: 6 K
        deploys "LC-CLIMATE"
        port "air" { physics: thermal flange: b }
    }
    node "Cooling controller" {
        id: "PN-BMS"
        deploys "LC-CONTROL"
        port in "supply_temperature" { is: "Signal" }
        port in "gpu_temperature" { is: "Signal" }
        port out "command" { is: "Signal" }
        port out "power_cap" { is: "Signal" }
        behavior_component "Supply temperature control" {
            id: "BC-COOL"
            setpoint: 32 °C
            cap_above: 83 °C
            port in "supply_temperature" { is: "Signal" }
            port in "gpu_temperature" { is: "Signal" }
            port out "command" { is: "Signal" }
            port out "power_cap" { is: "Signal" }
        }
    }
    physical_link "Rack manifolds" { id: "PL-MANIFOLD" from: "PN-RACKS.cold_plates" to: "PN-CDU.secondary" protocol: "Coolant PG25" }
    physical_link "Primary loop" { id: "PL-PRIMARY" from: "PN-CDU.primary" to: "PN-FWS.from_cdu" protocol: "Facility water" }
    physical_link "Cooler loop" { id: "PL-COOLERS" from: "PN-FWS.to_coolers" to: "PN-REJECT.water" protocol: "Facility water" }
    physical_link "Cooler coils" { id: "PL-COILS" from: "PN-REJECT.air" to: "PN-OUTDOOR.air" protocol: "Air" }
    physical_link "CDU network" { id: "PL-NET-CDU" from: "PN-CDU" to: "PN-BMS" protocol: "Modbus TCP" }
    physical_link "Rack network" { id: "PL-NET-RACKS" from: "PN-BMS" to: "PN-RACKS" protocol: "Redfish" }
    physical_link "Plant network" { id: "PL-NET-PLANT" from: "PN-BMS" to: "PN-REJECT" protocol: "BACnet" }
    physical_link "Flow to the racks" { id: "PL-FLOW" from: "PN-CDU" to: "PN-RACKS" protocol: "Modbus TCP" }
    physical_exchange "Coolant flow" { id: "PX-FLOW" from: "PN-CDU.coolant_flow" to: "PN-RACKS.coolant_flow" via: "Flow to the racks" message_type: "Flow" frequency: 1 Hz }
    physical_exchange "Supply temperature" { id: "PX-SUPPLY" from: "PN-CDU.supply_temperature" to: "PN-BMS.supply_temperature" via: "CDU network" message_type: "Temperature" frequency: 1 Hz }
    physical_exchange "GPU temperature" { id: "PX-GPU" from: "PN-RACKS.gpu_temperature" to: "PN-BMS.gpu_temperature" via: "Rack network" message_type: "Temperature" frequency: 1 Hz }
    physical_exchange "Cooling command" { id: "PX-COMMAND" from: "PN-BMS.command" to: "PN-REJECT.command" via: "Plant network" message_type: "Command" frequency: 1 Hz }
    physical_exchange "Power cap" { id: "PX-CAP" from: "PN-BMS.power_cap" to: "PN-RACKS.power_cap" via: "Rack network" message_type: "PowerCap" frequency: 1 Hz }
    physical_link "Supply to the controller" { id: "PL-S-IN" from: "PN-BMS.supply_temperature" to: "BC-COOL.supply_temperature" protocol: "On-chip" }
    physical_link "GPU to the controller" { id: "PL-G-IN" from: "PN-BMS.gpu_temperature" to: "BC-COOL.gpu_temperature" protocol: "On-chip" }
    physical_link "Command from the controller" { id: "PL-C-OUT" from: "BC-COOL.command" to: "PN-BMS.command" protocol: "On-chip" }
    physical_link "Cap from the controller" { id: "PL-P-OUT" from: "BC-COOL.power_cap" to: "PN-BMS.power_cap" protocol: "On-chip" }
}

// --- Behaviour: what runs step by step --------------------------------------

state_machine "Cooling modes" {
    id: "SM-COOLING"
    initial: "Free cooling"
    mode "Free cooling" { description: "Dry coolers alone hold the supply at 32 °C" }
    mode "Chiller assist" { description: "Hot afternoon: the chiller trims what the dry coolers cannot reject" }
    mode "Degraded" { description: "A CDU pump is lost: the spare runs, or the flow drops" }
    mode "Power capping" { description: "A GPU passes 83 °C: the scheduler caps the GPUs' power" }
    transition "Free cooling" -> "Chiller assist" { trigger: "SF-CONTROL" guard: "dry coolers at full speed and the supply above 32 °C" }
    transition "Chiller assist" -> "Free cooling" { trigger: "SF-CONTROL" guard: "the chiller idle for 15 min" }
    transition "Free cooling" -> "Degraded" { trigger: "SF-MEASURE" guard: "a CDU pump stops" }
    transition "Chiller assist" -> "Degraded" { trigger: "SF-MEASURE" guard: "a CDU pump stops" }
    transition "Degraded" -> "Power capping" { trigger: "SF-CONTROL" guard: "a GPU cold plate above 83 °C" }
    transition "Power capping" -> "Degraded" { trigger: "SF-CONTROL" guard: "every GPU cold plate below 80 °C" }
    transition "Degraded" -> "Free cooling" { trigger: "SF-MEASURE" guard: "the pump repaired" }
}

scenario "Pump failure during a training run" {
    id: "SC-PUMP"
    participants: ["LC-RACKS", "LC-CDU", "LC-CONTROL", "LC-REJECT"]
    message "LC-CDU" -> "LC-CONTROL" "pump 1 stopped"
    fragment alt {
        operand "a spare pump (N+1)" {
            message "LC-CONTROL" -> "LC-CDU" "start the spare pump"
            message "LC-CDU" -> "LC-CONTROL" "flow restored"
        }
        operand "no spare pump" {
            message "LC-CDU" -> "LC-CONTROL" "flow at half"
            message "LC-CONTROL" -> "LC-REJECT" "colder facility water"
            message "LC-RACKS" -> "LC-CONTROL" "GPU cold plate above 83 °C"
            message "LC-CONTROL" -> "LC-RACKS" "cap the GPUs' power"
        }
    }
}

configuration "Two pumps" { id: "CFG-N" selects: ["LC-CDU-N"] }
configuration "N+1 pumps" { id: "CFG-N1" selects: ["LC-CDU-N1"] }

trace "PN-CDU" satisfies "REQ-GPU"
trace "PN-REJECT" satisfies "REQ-GPU"
trace "BC-COOL" satisfies "REQ-SUPPLY"
trace "BC-COOL" satisfies "REQ-THROTTLE"
trace "PN-REJECT" satisfies "REQ-PUE"
trace "PN-CDU" satisfies "REQ-N1"
trace "PN-REJECT" satisfies "REQ-ENVELOPE"
trace "PN-REJECT" satisfies "REQ-RISK"
trace "LC-LEAK" satisfies "REQ-LEAK"
trace "SF-REJECT" realizes "OA-COOL"
trace "SF-LEAK-CLOSE" realizes "OA-LEAK"

// --- Verification by simulation --------------------------------------------

test_case "Design summer day, N+1 pumps" {
    id: "TC-SUMMER"
    verifies: ["REQ-GPU", "REQ-THROTTLE", "REQ-SUPPLY", "REQ-PUE"]
    method: "analysis"
    simulate: { model: "CoolingPhysics.SummerDay" configuration: "CFG-N1" stop_time: 24 h sources: ["physics/CoolingPhysics.mo"] }
    expects: ["REQ-GPU: max(\"PN-RACKS\".gpu_temperature) <= 85 °C",
              "REQ-THROTTLE: min(\"PN-RACKS\".cap) >= 0.99",
              "REQ-SUPPLY: max(\"PN-CDU\".supply_temperature) <= 40 °C",
              "REQ-SUPPLY: min(\"PN-CDU\".supply_temperature) >= 20 °C",
              "REQ-PUE: final(ppue) <= 1.1"]
}
test_case "Pump failure at full load, two pumps" {
    id: "TC-PUMP-N"
    verifies: ["REQ-N1"]
    method: "analysis"
    simulate: { model: "CoolingPhysics.PumpFailureTwoPumps" configuration: "CFG-N" stop_time: 24 h sources: ["physics/CoolingPhysics.mo"] }
    expects: ["max(\"PN-RACKS\".gpu_temperature) <= 85 °C", "min(\"PN-RACKS\".cap) >= 0.99"]
}
test_case "Pump failure at full load, N+1 pumps" {
    id: "TC-PUMP-N1"
    verifies: ["REQ-N1"]
    method: "analysis"
    simulate: { model: "CoolingPhysics.PumpFailure" configuration: "CFG-N1" stop_time: 24 h sources: ["physics/CoolingPhysics.mo"] }
    expects: ["max(\"PN-RACKS\".gpu_temperature) <= 85 °C", "min(\"PN-RACKS\".cap) >= 0.99"]
}
test_case "Climate and load envelope, N+1 pumps" {
    id: "TC-ENVELOPE"
    verifies: ["REQ-ENVELOPE"]
    method: "analysis"
    simulate: { model: "CoolingPhysics.SummerDay" configuration: "CFG-N1" stop_time: 24 h sources: ["physics/CoolingPhysics.mo"]
                sweep: ["\"PN-OUTDOOR\".design_temperature = 30 °C .. 45 °C step 5 °C", "\"PN-RACKS\".rack_power = 100 kW, 120 kW, 132 kW"] }
    expects: ["max(\"PN-RACKS\".gpu_temperature) <= 85 °C", "min(\"PN-RACKS\".cap) >= 0.99"]
}
test_case "Throttling risk over the summers, N+1 pumps" {
    id: "TC-RISK"
    verifies: ["REQ-RISK"]
    method: "analysis"
    simulate: { model: "CoolingPhysics.SummerDay" configuration: "CFG-N1" stop_time: 24 h sources: ["physics/CoolingPhysics.mo"]
                uncertain: ["\"PN-OUTDOOR\".design_temperature ~ normal(308.15 K, 3 K)", "\"PN-RACKS\".rack_power ~ uniform(110 kW, 132 kW)"]
                samples: 60 seed: 11 probability: 0.95 }
    expects: ["min(\"PN-RACKS\".cap) >= 0.99"]
}
test_case "Leak isolation" {
    id: "TC-LEAK"
    verifies: ["REQ-LEAK"]
    method: "analysis"
    description: "The leak isolation chain is checked against its 10 s budget by the compiler (FC-LEAK)"
}
