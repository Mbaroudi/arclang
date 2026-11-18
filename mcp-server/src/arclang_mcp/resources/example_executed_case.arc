// A case whose verdict is computed: the state machine is run, its expects: judged.
// The same case is run as C by arclang_sil and on the target by arclang_pil.
model Wiper {
    requirements system {
        req "REQ-WIPE" "Slow wiping" { description: "The wiper runs slow when the stalk asks for it" }
    }
    architecture logical { component "Wiper control" { id: "LC-WIPE" } }
    test_case "TC-WIPE" {
        verifies: ["REQ-WIPE"]
        method: "test"
        execute: {
            machine: "Wiper"
            step: 20 ms
            stop_time: 200 ms
            inputs: ["20 ms: stalk slow", "100 ms: stalk off"]
        }
        expects: [
            "REQ-WIPE: at(40 ms, \"Wiper\".Slow) == 1",
            "REQ-WIPE: at(120 ms, \"Wiper\".Slow) == 0"
        ]
    }
}

state_machine "Wiper" {
    initial: "Off"
    state Off {}
    state Slow {}
    transition Off -> Slow { trigger: "stalk slow" }
    transition Slow -> Off { trigger: "stalk off" }
}

io_binding "Wiper task" {
    machine: "Wiper"
    target: "zephyr/litex_vexriscv"
    period: 20 ms
    inputs: ["stalk slow <- gpio 4 rising", "stalk off <- gpio 6 rising"]
    outputs: ["Slow -> gpio 20"]
}

trace "LC-WIPE" satisfies "REQ-WIPE" { rationale: "Runs the wiper" }
