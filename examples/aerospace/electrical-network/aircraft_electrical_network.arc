// Loss of the generator on a 28 V DC aircraft network, verified by
// simulation. The physical architecture is exported to Modelica
// (`arclang export -f modelica`), the physics is hand-written in
// physics/ElectricalNetworkPhysics.mo, and `arclang verify` runs the
// failure in OpenModelica: the generator feeds the bus and charges the
// battery, then fails after ten minutes; the bus power control unit sheds
// the non-essential loads and the battery alone carries the essential ones.
// The battery's voltage sags as it empties and as it gets colder. The
// 40 Ah battery keeps the essential bus above 24 V for the 30 minutes the
// requirement asks; the 25 Ah battery does not. The envelope case sweeps the
// 40 Ah battery over cold and heavier loads, and finds where it is not
// enough: at -20 °C past 400 W, and at 700 W whatever the temperature.
// Each expectation names the requirement it checks, so a sagging bus does
// not hide that the load shedding works.

model "Aircraft electrical network" {
    description: "A 28 V DC network: generator, battery, essential and non-essential loads, load shedding, two battery sizes checked by simulation"
}

// --- Port definitions ------------------------------------------------------

type "DC plus" { physics: electrical }
type "DC minus" { physics: electrical pole: negative }
type "Discrete" { physics: signal }

// --- Requirements ----------------------------------------------------------

system_analysis "Electrical needs" {
    requirement "Essential power after a generator loss" {
        id: "REQ-ESS-30MIN"
        description: "After the loss of the generator, the essential bus stays at 24 V or more for 30 minutes, on the battery alone"
    }
    requirement "Load shedding" {
        id: "REQ-SHED"
        description: "The non-essential loads are shed when the generator is lost, and draw no power from the battery"
    }
    requirement "Cold and heavy envelope" {
        id: "REQ-ESS-ENVELOPE"
        description: "With the 40 Ah battery, from -20 °C to 20 °C and with 400 W to 700 W of essential loads, REQ-ESS-30MIN holds"
    }

    function "Generate power" { id: "SF-GEN" latency: 20 ms port out "status" }
    function "Control the buses" {
        id: "SF-BPCU"
        latency: 10 ms
        port in "status"
        port out "shed"
    }
    function "Supply non-essential loads" { id: "SF-NESS" latency: 50 ms port in "shed" }
    functional_exchange "generator status" { from: "SF-GEN.status" to: "SF-BPCU.status" }
    functional_exchange "shed order" { from: "SF-BPCU.shed" to: "SF-NESS.shed" }
    functional_chain "Load shedding chain" {
        id: "FC-SHED"
        involves: ["SF-GEN", "SF-BPCU", "SF-NESS"]
        latency_budget: 100 ms
    }
}

// --- Logical architecture: the product line --------------------------------

logical_architecture "Electrical system" {
    component "Electrical power" {
        id: "LC-EPS"
        component "Generation" { id: "LC-GEN" }
        component "Storage" {
            id: "LC-STORAGE"
            variation: true
            component "25 Ah battery" { id: "LC-BAT-25" variant: true capacity: 25 Ah }
            component "40 Ah battery" { id: "LC-BAT-40" variant: true capacity: 40 Ah }
        }
        component "Distribution" { id: "LC-DIST" }
        component "Essential consumers" { id: "LC-ESS" }
        component "Cabin consumers" { id: "LC-NESS" }
    }
}

// --- Physical architecture: what is simulated ------------------------------

physical_architecture "Electrical hardware" {
    node "Starter generator" {
        id: "PN-GEN"
        voltage: 28 V
        internal_resistance: 5 mOhm
        fails_at: 10 min
        deploys "LC-GEN"
        port "p" { is: "DC plus" }
        port "n" { is: "DC minus" }
        port out "online" { is: "Discrete" }
    }
    node "Main battery" {
        id: "PN-BAT"
        full_voltage: 27.6 V
        empty_voltage: 21 V
        internal_resistance: 25 mOhm
        temperature: 20 °C
        initial_soc: 80 %
        deploys "LC-STORAGE"
        port "p" { is: "DC plus" }
        port "n" { is: "DC minus" }
    }
    node "Essential loads" {
        id: "PN-ESS"
        power: 550 W
        deploys "LC-ESS"
        port "p" { is: "DC plus" }
        port "n" { is: "DC minus" }
    }
    node "Non-essential loads" {
        id: "PN-NESS"
        power: 2.4 kW
        deploys "LC-NESS"
        port "p" { is: "DC plus" }
        port "n" { is: "DC minus" }
        port in "shed" { is: "Discrete" }
    }
    node "Bus power control unit" {
        id: "PN-BPCU"
        deploys "LC-DIST"
        port in "online" { is: "Discrete" }
        port out "shed" { is: "Discrete" }
        behavior_component "Shedding logic" {
            id: "BC-SHED"
            port in "online" { is: "Discrete" }
            port out "shed" { is: "Discrete" }
        }
    }
    physical_link "Generator feeder" { id: "PL-GEN-P" from: "PN-GEN.p" to: "PN-BAT.p" protocol: "28 V DC" }
    physical_link "Essential feeder" { id: "PL-ESS-P" from: "PN-BAT.p" to: "PN-ESS.p" protocol: "28 V DC" }
    physical_link "Cabin feeder" { id: "PL-NESS-P" from: "PN-BAT.p" to: "PN-NESS.p" protocol: "28 V DC" }
    physical_link "Generator return" { id: "PL-GEN-N" from: "PN-GEN.n" to: "PN-BAT.n" protocol: "Structure return" }
    physical_link "Essential return" { id: "PL-ESS-N" from: "PN-BAT.n" to: "PN-ESS.n" protocol: "Structure return" }
    physical_link "Cabin return" { id: "PL-NESS-N" from: "PN-BAT.n" to: "PN-NESS.n" protocol: "Structure return" }
    physical_link "Generator control" { id: "PL-GCU" from: "PN-GEN" to: "PN-BPCU" protocol: "Discrete wiring" }
    physical_link "Shed relays" { id: "PL-RELAY" from: "PN-BPCU" to: "PN-NESS" protocol: "Discrete wiring" }
    physical_exchange "Generator online" { id: "PX-ONLINE" from: "PN-GEN.online" to: "PN-BPCU.online" via: "Generator control" message_type: "GeneratorStatus" frequency: 100 Hz }
    physical_exchange "Shed order" { id: "PX-SHED" from: "PN-BPCU.shed" to: "PN-NESS.shed" via: "Shed relays" message_type: "Shed" frequency: 100 Hz }
    physical_link "Status to the logic" { id: "PL-S-IN" from: "PN-BPCU.online" to: "BC-SHED.online" protocol: "On-board" }
    physical_link "Order from the logic" { id: "PL-S-OUT" from: "BC-SHED.shed" to: "PN-BPCU.shed" protocol: "On-board" }
}

configuration "Light battery" { id: "CFG-25AH" selects: ["LC-BAT-25"] }
configuration "Heavy battery" { id: "CFG-40AH" selects: ["LC-BAT-40"] }

trace "PN-BAT" satisfies "REQ-ESS-30MIN"
trace "BC-SHED" satisfies "REQ-SHED"
trace "PN-BAT" satisfies "REQ-ESS-ENVELOPE"

// --- Verification by simulation --------------------------------------------

// The generator fails at 10 min; the run ends 30 min later.
test_case "Generator loss, 25 Ah battery" {
    id: "TC-LOSS-25"
    verifies: ["REQ-ESS-30MIN", "REQ-SHED"]
    method: "analysis"
    simulate: { model: "ElectricalNetworkPhysics.LightBattery" configuration: "CFG-25AH" stop_time: 40 min sources: ["physics/ElectricalNetworkPhysics.mo"] }
    expects: ["REQ-ESS-30MIN: min(\"PN-ESS\".voltage) >= 24 V", "REQ-SHED: at(40 min, \"PN-NESS\".drawn) <= 1 W"]
}
test_case "Generator loss, 40 Ah battery" {
    id: "TC-LOSS-40"
    verifies: ["REQ-ESS-30MIN", "REQ-SHED"]
    method: "analysis"
    simulate: { model: "ElectricalNetworkPhysics.HeavyBattery" configuration: "CFG-40AH" stop_time: 40 min sources: ["physics/ElectricalNetworkPhysics.mo"] }
    expects: ["REQ-ESS-30MIN: min(\"PN-ESS\".voltage) >= 24 V", "REQ-SHED: at(40 min, \"PN-NESS\".drawn) <= 1 W"]
}
test_case "Cold and heavy envelope, 40 Ah battery" {
    id: "TC-LOSS-ENVELOPE"
    verifies: ["REQ-ESS-ENVELOPE"]
    method: "analysis"
    simulate: { model: "ElectricalNetworkPhysics.HeavyBattery" configuration: "CFG-40AH" stop_time: 40 min sources: ["physics/ElectricalNetworkPhysics.mo"]
                sweep: ["\"PN-BAT\".temperature = -20 °C, 0 °C, 20 °C", "\"PN-ESS\".power = 400 W .. 700 W step 150 W"] }
    expects: ["min(\"PN-ESS\".voltage) >= 24 V"]
}
