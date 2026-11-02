// An electric car line, with what Arcadia gains from SysML v2: port and
// interface definitions, variants and configurations, typed allocations,
// views filtered by tags, and analyses tied to requirements.

model "EV product line" {
    description: "Two batteries, two chargers, one architecture"
}

// --- Reusable definitions --------------------------------------------------

type "HV connector" {
    interface: "HVIL 800 V"
    port inout "pole" { multiplicity: 2 }
}
type "HV power" {
    protocol: "DC 800 V"
}
type "Software deployment" {
    from_kind: "PhysicalNode"
    to_kind: "LogicalComponent"
    partition: "ASIL-C partition"
}

// --- Requirements ----------------------------------------------------------

system_analysis "Vehicle needs" {
    requirement "Range" { id: "REQ-RANGE" description: "Enough energy on board for the range of the line" }
    requirement "Isolation" { id: "REQ-ISO" description: "High voltage is monitored for isolation faults" }
}

// --- Architecture ----------------------------------------------------------

logical_architecture "EV" {
    component "Vehicle" {
        id: "LC-EV"
        energy: 60 kWh
        component "Battery" {
            id: "LC-BATT"
            variation: true
            tags: ["Safety"]
            port inout "hv" { is: "HV connector" }
            component "Standard pack" { id: "LC-PACK-STD" variant: true energy: 64 kWh }
            component "Long range pack" { id: "LC-PACK-LR" variant: true energy: 82 kWh }
        }
        component "Charger" {
            id: "LC-CHG"
            variation: true
            port inout "hv" { is: "HV connector" }
            component "11 kW onboard charger" { id: "LC-OBC-11" variant: true }
            component "22 kW onboard charger" { id: "LC-OBC-22" variant: true }
        }
        component "Battery management" { id: "LC-BMS" tags: ["Safety"] }
        component "Infotainment" { id: "LC-INFO" }
    }
    component_exchange "Charging power" {
        is: "HV power"
        from_port: "LC-CHG.hv.pole"
        to_port: "LC-BATT.hv.pole"
    }
}

physical_architecture "EV hardware" {
    node "Battery ECU" { id: "PN-BECU" }
}

configuration "City" { id: "CFG-CITY" selects: ["LC-PACK-STD", "LC-OBC-11"] }
configuration "Touring" { id: "CFG-TOURING" selects: ["LC-PACK-LR", "LC-OBC-22"] }

trace "PN-BECU" allocates "LC-BMS" { is: "Software deployment" }
trace "LC-BATT" satisfies "REQ-RANGE"
trace "LC-BMS" satisfies "REQ-ISO"

// --- Analyses and tests ----------------------------------------------------

constraint "Smallest pack covers the range" {
    id: "CST-RANGE"
    assert: "LC-PACK-STD".energy >= "LC-EV".energy
}

test_case "Range analysis" {
    id: "TC-RANGE"
    verifies: ["REQ-RANGE"]
    method: "analysis"
    asserts: ["CST-RANGE"]
}
test_case "Isolation fault injection" {
    id: "TC-ISO"
    verifies: ["REQ-ISO"]
    method: "test"
    verdict: "pass"
}

// --- Views -----------------------------------------------------------------

diagram "Touring car" { view: lab configuration: "CFG-TOURING" shows: ["LC-EV"] }
diagram "Safety relevant" { view: lab under: "LC-EV" tagged: ["Safety"] }
