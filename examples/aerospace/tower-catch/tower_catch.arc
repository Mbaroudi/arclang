// Catching a returning heavy vehicle with a tower's arms, verified by simulation.
// The vehicle falls flat (belly flop) until the drag balances its weight, turns
// upright (flip), relights its engines when the braking they must give reaches
// their lowest thrust, follows the thrust that brings its speed to zero at the
// height of the arms, and is caught by two fittings on two arms that share its
// weight. The physics is hand-written in physics/CatchPhysics.mo and
// `arclang verify` runs it in OpenModelica: the design descent, and a vehicle
// 15 % heavier than the design. The phases are a state machine you can run step
// by step in the debugger (F5, or ArcLang: Run…).
//
// The numbers are round and chosen for the example. They are not those of any
// real vehicle or tower.

model "Tower catch" {
    description: "A heavy vehicle returns from space and is caught by the arms of its launch tower: its descent, its landing burn and the load on its catch fittings, checked by simulation"
}

// --- Operational analysis: who needs what ----------------------------------

operational_analysis "Return to the launch site" {
    actor "Flight director" { id: "OA-FD" description: "Clears the vehicle to return and the tower to catch it" }
    actor "Tower crew" { id: "OA-CREW" description: "Prepares the arms, makes the vehicle safe after the catch" }
    actor "Range safety" { id: "OA-RANGE" description: "Keeps people out of the zone under the descent" }
    entity "Launch site" {
        id: "OA-SITE"
        activity "Bring the vehicle home" { id: "OA-RETURN" description: "From the end of the flight to the last 2 km above the tower" }
        activity "Stop it at the arms" { id: "OA-STOP" description: "Brake the vehicle to a halt at the height of the arms" }
        activity "Hold it" { id: "OA-HOLD" description: "Carry its weight on the arms and make it safe" }
    }
}

// --- Requirements ----------------------------------------------------------

system_analysis "Catch needs" {
    requirement "Speed at the arms" {
        id: "REQ-CONTACT"
        description: "When the catch fittings reach the arms, the vehicle descends at 3 m/s at most"
    }
    requirement "Load on the fittings" {
        id: "REQ-FITTING"
        description: "The load each catch fitting carries, at the contact and after, stays at or below 1200 kN"
    }
    requirement "Weight shared" {
        id: "REQ-SHARE"
        description: "Once the vehicle hangs on the arms, the two fittings share its weight: each carries between 450 and 550 kN"
    }
    requirement "Landing propellant" {
        id: "REQ-PROP"
        description: "The landing burn uses at most 7 t of propellant, of the 8 t kept for it: a margin of 1 t"
    }
    requirement "Heavier vehicle" {
        id: "REQ-HEAVY"
        description: "A vehicle 15 % heavier than the design is caught within the same limits on speed and on load"
    }
    requirement "Engines never over the limit" {
        id: "REQ-THRUST"
        description: "The landing burn never asks the engines for more than their maximum thrust of 2600 kN"
    }

    function "Estimate the state" { id: "SF-ESTIMATE" latency: 10 ms port out "state" }
    function "Compute the thrust" {
        id: "SF-GUIDE"
        latency: 10 ms
        port in "state"
        port out "thrust command"
    }
    function "Throttle the engines" { id: "SF-THROTTLE" latency: 50 ms port in "thrust command" }
    functional_exchange "vehicle state" { from: "SF-ESTIMATE.state" to: "SF-GUIDE.state" }
    functional_exchange "thrust command" { from: "SF-GUIDE.thrust command" to: "SF-THROTTLE.thrust command" }
    functional_chain "Landing guidance loop" {
        id: "FC-GUIDANCE"
        involves: ["SF-ESTIMATE", "SF-GUIDE", "SF-THROTTLE"]
        latency_budget: 100 ms
    }
}

// --- Logical architecture --------------------------------------------------

logical_architecture "Vehicle and tower" {
    component "Vehicle" {
        id: "LC-VEHICLE"
        component "Engines" { id: "LC-ENGINES" function "Produce the thrust" { id: "LF-THRUST" } }
        component "Flight computer" { id: "LC-FC" function "Guide the landing" { id: "LF-GUIDE" } }
        component "Catch fittings" { id: "LC-FITTINGS" function "Carry the weight" { id: "LF-CARRY" } }
    }
    component "Tower" {
        id: "LC-TOWER"
        component "Catch arms" { id: "LC-ARMS" function "Catch and hold the vehicle" { id: "LF-CATCH" } }
    }
}

// --- Physical architecture: what is simulated ------------------------------

physical_architecture "Catch system" {
    node "Vehicle" {
        id: "PN-VEHICLE"
        dry_mass: 100000 kg
        landing_propellant: 8000 kg
        belly_area: 450 m2
        upright_area: 60 m2
        deploys "LC-VEHICLE"
    }
    node "Engines" {
        id: "PN-ENGINES"
        thrust_max: 2600 kN
        thrust_min: 1500 kN
        specific_impulse: 330 s
        deploys "LC-ENGINES"
    }
    node "Flight computer" {
        id: "PN-FC"
        deploys "LC-FC"
        behavior_component "Landing guidance" {
            id: "BC-GUIDANCE"
            relight_at_thrust: 1500 kN
            cut_below_speed: 1 m/s
        }
    }
    node "Catch arms" {
        id: "PN-ARMS"
        catch_height: 146 m
        stiffness: "2 MN/m"
        damping: "200 kN.s/m"
        deploys "LC-ARMS"
    }
    node "Catch fittings" {
        id: "PN-FITTINGS"
        count: 2
        deploys "LC-FITTINGS"
    }
    node "Atmosphere" {
        id: "PN-AIR"
        density_at_ground: "1.2 kg/m3"
        scale_height: 8500 m
    }
}

// --- Behavior: the phases of the descent -----------------------------------

state_machine "Phases of the descent" {
    id: "SM-PHASES"
    initial: "Belly flop"
    mode "Belly flop" { description: "Falling flat: the drag grows until it balances the weight, at about 67 m/s" }
    mode "Flip" { description: "Turning upright in 6 s: the area to the wind falls from 450 to 60 m2" }
    mode "Landing burn" { description: "Engines on, following the thrust that stops the vehicle at the arms" }
    mode "Coast to the arms" { description: "Engines cut below 1 m/s: the last metres under gravity" }
    mode "Caught" { description: "The fittings hang on the arms and share the weight" }
    transition "Belly flop" -> "Flip" { trigger: "SF-ESTIMATE" guard: "altitude at or below 1100 m" }
    transition "Flip" -> "Landing burn" { trigger: "SF-GUIDE" guard: "flip done and the braking asked reaches the lowest thrust" }
    transition "Landing burn" -> "Coast to the arms" { trigger: "SF-GUIDE" guard: "speed below 1 m/s" }
    transition "Coast to the arms" -> "Caught" { trigger: "SF-ESTIMATE" guard: "fittings at the height of the arms" }
}

trace "PN-ARMS" satisfies "REQ-CONTACT"
trace "BC-GUIDANCE" satisfies "REQ-CONTACT"
trace "PN-ARMS" satisfies "REQ-FITTING"
trace "PN-FITTINGS" satisfies "REQ-FITTING"
trace "PN-FITTINGS" satisfies "REQ-SHARE"
trace "PN-ENGINES" satisfies "REQ-PROP"
trace "BC-GUIDANCE" satisfies "REQ-PROP"
trace "PN-ENGINES" satisfies "REQ-HEAVY"
trace "PN-VEHICLE" satisfies "REQ-HEAVY"
trace "PN-ENGINES" satisfies "REQ-THRUST"
trace "LC-VEHICLE" satisfies "REQ-HEAVY"
trace "LC-ENGINES" satisfies "REQ-THRUST"
trace "LC-FC" satisfies "REQ-CONTACT"
trace "LC-FITTINGS" satisfies "REQ-SHARE"
trace "LC-TOWER" satisfies "REQ-FITTING"
trace "LC-ARMS" satisfies "REQ-FITTING"
trace "SF-GUIDE" realizes "OA-STOP"
trace "SF-THROTTLE" realizes "OA-STOP"

// --- Verification by simulation --------------------------------------------

test_case "Design descent" {
    id: "TC-DESIGN"
    verifies: ["REQ-CONTACT", "REQ-FITTING", "REQ-SHARE", "REQ-PROP", "REQ-THRUST"]
    method: "analysis"
    simulate: { model: "CatchPhysics.Descent" stop_time: 60 s sources: ["physics/CatchPhysics.mo"] }
    expects: ["REQ-CONTACT: final(v_contact) <= 3 m/s",
              "REQ-FITTING: max(F_fitting) <= 1200 kN",
              "REQ-SHARE: at(60 s, F_fitting) >= 450 kN",
              "REQ-SHARE: at(60 s, F_fitting) <= 550 kN",
              "REQ-PROP: final(propellant_used) <= 7000 kg",
              "REQ-THRUST: max(T) <= 2600 kN"]
}
test_case "Vehicle 15 % heavier" {
    id: "TC-HEAVY"
    verifies: ["REQ-HEAVY"]
    method: "analysis"
    simulate: { model: "CatchPhysics.HeavyVehicle" stop_time: 60 s sources: ["physics/CatchPhysics.mo"] }
    expects: ["final(v_contact) <= 3 m/s", "max(F_fitting) <= 1200 kN"]
}
