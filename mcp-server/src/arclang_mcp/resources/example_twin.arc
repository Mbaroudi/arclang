// A case that simulates the physics, and a twin that drives the same physics with measurements.
model Heating {
    requirements system {
        req "REQ-COMFORT" "Comfort" { description: "The room stays at 20 °C or more" }
    }
    architecture physical {
        node "Outdoor" { id: "PN-OUTDOOR" }
        node "Living space" { id: "PN-SPACE" }
    }
    test_case "TC-DESIGN-DAY" {
        verifies: ["REQ-COMFORT"]
        simulate: { model: "HeatingPhysics.StandardDay" stop_time: 24 h sources: ["physics/HeatingPhysics.mo"] }
        expects: ["REQ-COMFORT: min(\"PN-SPACE\".room.T) >= 20 °C"]
    }
}

twin "January cold day" {
    id: "TW-JANUARY"
    verifies: ["REQ-COMFORT"]
    simulate: { model: "HeatingPhysics.StandardDay" sources: ["physics/HeatingPhysics.mo"] }
    data: { file: "data/house-2026-01-14.csv" time: "timestamp" }
    drives: ["\"PN-OUTDOOR\".T <- outdoor_c °C"]
    compares: ["\"PN-SPACE\".room.T ~ indoor_c °C within 0.5 K for 99 %"]
    expects: ["REQ-COMFORT: min(\"PN-SPACE\".room.T) >= 20 °C"]
}
