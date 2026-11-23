model "Adaptive cruise control, followed in CARLA" {
    description: "A follower keeps its gap behind a lead vehicle that drives, slows and stops; a digital twin compares the model of the gap law with the vehicle the CARLA simulator drives"
}

system_analysis "Following a vehicle" {
    requirement "Gap to the lead vehicle" {
        id: "REQ-GAP"
        description: "Behind a lead vehicle that drives, slows and stops, the follower never comes closer than 4 m"
    }
    function "Measure the gap" { id: "SF-GAP" port out "gap" }
    function "Keep the gap" { id: "SF-KEEP" port in "gap" port out "acceleration" }
    function "Move the vehicle" { id: "SF-MOVE" port in "acceleration" }
    functional_exchange "gap and closing speed" { from: "SF-GAP.gap" to: "SF-KEEP.gap" }
    functional_exchange "acceleration asked" { from: "SF-KEEP.acceleration" to: "SF-MOVE.acceleration" }
}

logical_architecture "Follower" {
    component "Range sensor" { id: "LC-RANGE" }
    component "Gap controller" { id: "LC-GAP" }
    component "Vehicle" { id: "LC-VEHICLE" }
}

physical_architecture "Follower vehicle" {
    node "Radar" { id: "PN-RADAR" deploys "LC-RANGE" }
    node "Cruise control unit" {
        id: "PN-ACC"
        standstill_gap: 6 m
        time_gap: 1.5 s
        deploys "LC-GAP"
    }
    node "Powertrain and brakes" { id: "PN-DRIVE" deploys "LC-VEHICLE" }
}

trace "LC-RANGE" satisfies "REQ-GAP"
trace "LC-GAP" satisfies "REQ-GAP"
trace "LC-VEHICLE" satisfies "REQ-GAP"
trace "PN-ACC" satisfies "REQ-GAP"

// What CARLA measured, replayed: a run recorded by the live twin below. The
// data is a simulator's, not a road vehicle's (see the file's header).
twin "Follower in CARLA, a recorded run" {
    id: "TW-CARLA-RUN"
    verifies: ["REQ-GAP"]
    simulate: { model: "AccPhysics.Follow" sources: ["physics/AccPhysics.mo"] }
    data: { file: "data/carla-town04.csv" }
    drives: ["v_lead <- lead_speed m/s"]
    compares: [
        "v_ego ~ ego_speed m/s within 1.5 m/s for 95 %",
        "gap ~ gap m within 3 m for 95 %"
    ]
    expects: ["REQ-GAP: min(gap) >= 4 m"]
}

// The same twin on a running CARLA server: carla/bridge.py publishes each
// sample on a local MQTT broker (see the README).
twin "Follower in CARLA, live" {
    id: "TW-CARLA-LIVE"
    simulate: { model: "AccPhysics.Follow" sources: ["physics/AccPhysics.mo"] }
    data: { mqtt: "mqtt://localhost:1883" topics: ["carla/acc"] window: 10 min refresh: 15 s record: "data/carla-live.csv" }
    drives: ["v_lead <- lead_speed m/s"]
    compares: [
        "v_ego ~ ego_speed m/s within 1.5 m/s for 95 %",
        "gap ~ gap m within 3 m for 95 %"
    ]
    expects: ["REQ-GAP: min(gap) >= 4 m"]
}
