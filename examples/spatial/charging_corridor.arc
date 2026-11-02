// A motorway charging corridor modeled in space: three charging hubs on
// the A6 between Auxerre and Beaune, the fibre that links them along the
// motorway, and the supervision center in Lyon. Each hub stands where it is
// (WGS 84), its cabinet has a size and its equipment a place in it, and the
// links have a route, a length and a latency the compiler checks against
// the distances and the speed of light.
//
//   arclang check examples/spatial/charging_corridor.arc
//   arclang export examples/spatial/charging_corridor.arc -f geojson -o corridor.geojson   (QGIS, kepler.gl, GitHub)
//   arclang export examples/spatial/charging_corridor.arc -f step -o corridor.step         (CATIA, SOLIDWORKS, FreeCAD)

model ChargingCorridor {
  name: "A6 Charging Corridor"
  description: "Three 600 kW charging hubs every 60 km at most on the A6, supervised from Lyon over the motorway fibre"
  version: "1.0"
}

// ---- Requirements ---------------------------------------------------------

requirements corridor {
  requirement "REQ-COR-001" {
    description: "A charging hub shall stand at most every 60 km along the corridor, in each direction"
    category: "functional"  priority: "Critical"  source: "Regulation (EU) 2023/1804 (AFIR), article 3"  verification_method: "analysis"
  }
  requirement "REQ-COR-002" {
    description: "Each hub shall deliver at least 600 kW in total, with at least one point of 150 kW"
    category: "performance"  priority: "High"  source: "AFIR, article 3"  verification_method: "test"
  }
  requirement "REQ-COR-003" {
    description: "The supervision center shall see the state of every charging point within 1 s, over a link of at most 80 km between amplifiers"
    category: "performance"  priority: "High"  verification_method: "analysis"
  }
  requirement "REQ-COR-004" {
    description: "The cabinet of a hub shall fit on a 6 m by 2.5 m concrete slab"
    category: "constraint"  priority: "Medium"  verification_method: "inspection"
  }
}

// ---- Operational analysis -------------------------------------------------

operational_analysis "Operational Analysis" {
  actor "EV driver" { id: "OA-DRV" description: "Travels the A6 and stops to charge" }
  entity "Motorway operator" {
    id: "OE-MOT"
    description: "Operates the A6 and its service areas, owns the motorway fibre"
    location: { lat: 47.3220 deg lon: 5.0415 deg }
    activity "Host the charging hubs" { id: "OAC-HOST" }
  }
  entity "Charge point operator" {
    id: "OE-CPO"
    description: "Runs the hubs from its supervision center"
    location: { lat: 45.7640 deg lon: 4.8357 deg alt: 170 m }
    activity "Supervise the corridor" { id: "OAC-SUP" }
    activity "Deliver energy" { id: "OAC-DELIVER" }
  }
  operational_capability "Cross France without range anxiety" {
    id: "OC-001"
    involves: ["OA-DRV", "OE-MOT", "OE-CPO", "OAC-HOST", "OAC-SUP", "OAC-DELIVER"]
  }
  interaction "Stop to charge" { id: "OI-STOP" from: "OA-DRV" to: "OAC-DELIVER" }
  interaction "Site availability" { id: "OI-SITE" from: "OAC-HOST" to: "OAC-SUP" }
}

// ---- System analysis ------------------------------------------------------

system_analysis "System Analysis" {
  actor "EV driver" { id: "SA-DRV" }
  actor "Grid operator" { id: "SA-GRID" location: { lat: 47.3220 deg lon: 5.0415 deg } }

  capability "Charge on the corridor" {
    id: "CAP-001"  realizes: "OC-001"
    involves: ["SF-CHARGE", "SF-SUPERVISE"]
  }
  function "Deliver charging power" { id: "SF-CHARGE" realizes: "OAC-DELIVER" }
  function "Supervise the hubs" { id: "SF-SUPERVISE" realizes: "OAC-SUP" }
  functional_exchange "Hub state" { id: "SFE-STATE" from: "SF-CHARGE" to: "SF-SUPERVISE" latency: 1 s }
}

// ---- Logical architecture -------------------------------------------------

logical_architecture "Logical Architecture" {
  component "Charging hub" {
    id: "LC-HUB"
    function "Convert and deliver power" { id: "LF-CONV" realizes: "SF-CHARGE" }
    port out "state"
  }
  component "Supervision" {
    id: "LC-SUP"
    function "Collect hub states" { id: "LF-SUP" realizes: "SF-SUPERVISE" }
    port in "state"
  }
  component_exchange "Hub state" { id: "LCE-STATE" from_port: "LC-HUB.state" to_port: "LC-SUP.state" }
}

// ---- Physical architecture ------------------------------------------------
// Positions are in the owner's frame (x east, y north, z up, from the
// south-west corner of the slab); a hub without a position is placed in CAD
// by its distance from the first located hub.

physical_architecture "Physical Architecture" {
  node "Hub Venoy" {
    id: "PN-VEN"
    description: "Aire de Venoy, A6 kilometre 165"
    location: { lat: 47.7950 deg lon: 3.6610 deg alt: 180 m }
    shape: { box: [6 m, 2.5 m, 2.6 m] }
    port out "fibre"
    deploys "LC-HUB"
    behavior_component "Power cabinet" {
      id: "PB-VEN-PWR"
      position: { x: 0.3 m y: 0.3 m z: 0 m }
      shape: { box: [3.2 m, 1.2 m, 2.2 m] }
    }
    behavior_component "Charge controller" {
      id: "PB-VEN-CTL"
      position: { x: 4.2 m y: 0.6 m z: 0.8 m }
      orientation: { yaw: 90 deg }
      shape: { box: [0.8 m, 0.6 m, 1.2 m] }
    }
    behavior_component "Transformer" {
      id: "PB-VEN-TX"
      position: { x: 5.3 m y: 1.8 m z: 0 m }
      shape: { cylinder: [1.1 m, 1.8 m] }
    }
  }
  node "Hub Maison-Dieu" {
    id: "PN-MDI"
    description: "Aire de la Maison-Dieu, A6 kilometre 202"
    location: { lat: 47.5370 deg lon: 3.9350 deg alt: 290 m }
    shape: { box: [6 m, 2.5 m, 2.6 m] }
    port in "fibre"
    port out "relay"
    deploys "LC-HUB"
  }
  node "Hub Pouilly" {
    id: "PN-POU"
    description: "Aire de Pouilly-en-Auxois, A6 kilometre 255"
    location: { lat: 47.2650 deg lon: 4.5560 deg alt: 390 m }
    shape: { box: [6 m, 2.5 m, 2.6 m] }
    port in "relay"
    port out "uplink"
    deploys "LC-HUB"
  }
  node "Supervision center" {
    id: "PN-SUP"
    description: "Lyon, Part-Dieu"
    location: { lat: 45.7640 deg lon: 4.8357 deg alt: 170 m }
    port in "uplink"
    deploys "LC-SUP"
  }

  physical_link "Fibre Venoy - Maison-Dieu" {
    id: "PL-VEN-MDI"
    from: "PN-VEN.fibre"  to: "PN-MDI.fibre"
    protocol: "10GBASE-LR"
    route: [{ lat: 47.7950 deg lon: 3.6610 deg }, { lat: 47.6800 deg lon: 3.7700 deg }, { lat: 47.5370 deg lon: 3.9350 deg }]
    length: 36 km
    max_length: 80 km
    latency: 0.5 ms
  }
  physical_link "Fibre Maison-Dieu - Pouilly" {
    id: "PL-MDI-POU"
    from: "PN-MDI.relay"  to: "PN-POU.relay"
    protocol: "10GBASE-LR"
    route: [{ lat: 47.5370 deg lon: 3.9350 deg }, { lat: 47.4300 deg lon: 4.1800 deg }, { lat: 47.2650 deg lon: 4.5560 deg }]
    length: 58 km
    max_length: 80 km
    latency: 0.5 ms
  }
  physical_link "Uplink Pouilly - Lyon" {
    id: "PL-POU-SUP"
    from: "PN-POU.uplink"  to: "PN-SUP.uplink"
    protocol: "MPLS over leased line"
    length: 210 km
    latency: 4 ms
  }
}

// ---- EPBS -----------------------------------------------------------------

epbs "Corridor EPBS" {
  system "Charging hub" {
    id: "CI-HUB"
    shape: { box: [6 m, 2.5 m, 2.6 m] }
    subsystem "Power train" {
      id: "CI-POWER"
      position: { x: 0.3 m y: 0.3 m }
      item "Power cabinet" { id: "CI-PWR" part_number: "PWR-600-C" mass: 1800 kg shape: { box: [3.2 m, 1.2 m, 2.2 m] } }
      item "Transformer" { id: "CI-TX" part_number: "TX-630-20K" mass: 2100 kg position: { x: 5 m y: 1.5 m } shape: { cylinder: [1.1 m, 1.8 m] } }
    }
  }
}

// ---- Traceability ---------------------------------------------------------

trace "PN-VEN" satisfies "REQ-COR-001"
trace "PN-MDI" satisfies "REQ-COR-001"
trace "PN-POU" satisfies "REQ-COR-001"
trace "LC-HUB" satisfies "REQ-COR-002"
trace "LC-SUP" satisfies "REQ-COR-003"
trace "PL-VEN-MDI" satisfies "REQ-COR-003"
trace "PB-VEN-PWR" satisfies "REQ-COR-004"

test_case "TC-COR-001" { id: "TC-COR-001" verifies: ["REQ-COR-001"] method: "analysis" description: "Distances between hubs measured on the GeoJSON export" }
test_case "TC-COR-002" { id: "TC-COR-002" verifies: ["REQ-COR-002"] method: "test" }
test_case "TC-COR-003" { id: "TC-COR-003" verifies: ["REQ-COR-003"] method: "analysis" description: "Link lengths and latencies checked by the compiler" }
test_case "TC-COR-004" { id: "TC-COR-004" verifies: ["REQ-COR-004"] method: "inspection" description: "Cabinet footprint read on the STEP export" }
