// ============================================================================
// BODY CONTROLLER - two machines on one processor, a task each (docs/embedded-c.md)
// ============================================================================
// A door module whose processor runs two controllers at two rates: the
// wiper (every 20 ms: the stalk, the rain sensor, the park position) and
// the interior light (every 100 ms: the doors, the dimming). Each is an
// io_binding on the same target: the application runs one task per
// binding, at its period and its priority (the shortest period the
// highest, rate monotonic, unless `priority:` says otherwise). The step
// times measured on the target by `arclang partition` feed
// `arclang schedule`, which says whether every task meets its period.
//
//   arclang check     examples/automotive/body-controller/body_controller.arc
//   arclang sil       examples/automotive/body-controller/body_controller.arc
//   arclang export    examples/automotive/body-controller/body_controller.arc -f zephyr -o build/body
//   arclang partition examples/automotive/body-controller/body_controller.arc
//   arclang schedule  examples/automotive/body-controller/body_controller.arc
// ============================================================================

model BodyController {
    metadata {
        version: "1.0.0"
        description: "Door module: wiper and interior light as two tasks"
        domain: "automotive"
    }
}

requirements system {
    req "SYS-BC-001" "Wiper follows the stalk" {
        description: "The wiper shall run at the speed the stalk asks for and park when it is released"
        priority: "High"
    }
    req "SYS-BC-002" "Light fades out" {
        description: "The interior light shall go on when a door opens and fade out within 2 s of the last door closing"
        priority: "Medium"
    }
    req "SYS-BC-003" "Every task keeps its period" {
        description: "On the door module's processor, the wiper task shall keep its 20 ms period and the light task its 100 ms period"
        priority: "High"
    }
}

physical_architecture "Door module" {
    node "Door module processor" {
        id: "PN-BC-CPU"
        processor: "VexRiscv (LiteX)"
        deploys "Wiper"
        deploys "Interior light"
    }
}

state_machine "Wiper" {
    variables: ["in rain: Integer range 0 .. 100", "out speed: Integer = 0", "out sweeps: Integer = 0"]
    initial: "Parked"
    state Parked { entry: "speed := 0" }
    state Slow { entry: "speed := 1" do: "sweep" }
    state Fast { entry: "speed := 2" do: "sweep" }
    state Parking { entry: "speed := 1" }
    transition Parked -> Slow { trigger: "stalk slow" }
    transition Parked -> Fast { trigger: "stalk fast" }
    transition Parked -> Slow { trigger: "rain seen" guard: "rain > 40" action: "sweeps := sweeps + 1" }
    transition Slow -> Fast { trigger: "stalk fast" }
    transition Fast -> Slow { trigger: "stalk slow" }
    transition Slow -> Parking { trigger: "stalk off" }
    transition Fast -> Parking { trigger: "stalk off" }
    transition Parking -> Parked { trigger: "park position" }
}

state_machine "Interior light" {
    variables: ["out level: Integer = 0", "out fades: Integer = 0"]
    initial: "Off"
    state Off { entry: "level := 0" }
    state On { entry: "level := 100" }
    state Fading { entry: "level := 50" exit: "fades := fades + 1" }
    transition Off -> On { trigger: "door opened" }
    transition On -> Fading { trigger: "doors closed" }
    transition Fading -> On { trigger: "door opened" }
    transition Fading -> Off { timing: 2 s }
}

io_binding "Wiper task" {
    machine: "Wiper"
    target: "zephyr/litex_vexriscv"
    period: 20 ms
    watchdog: true
    inputs: [
        "stalk slow <- gpio 4 rising",
        "stalk fast <- gpio 5 rising",
        "stalk off <- gpio 6 rising",
        "park position <- gpio 7 rising",
        "rain seen <- uart 0 0x52"
    ]
    outputs: [
        "Slow -> gpio 20",
        "Fast -> gpio 21"
    ]
}

io_binding "Light task" {
    machine: "Interior light"
    target: "zephyr/litex_vexriscv"
    period: 100 ms
    inputs: [
        "door opened <- gpio 8 rising",
        "doors closed <- gpio 8 falling"
    ]
    outputs: [
        "On -> gpio 22",
        "Fading -> pwm 0 500"
    ]
}

test_case "TC-BC-001" {
    verifies: ["SYS-BC-001"]
    method: "test"
    description: "Slow, fast, off, parked"
    execute: {
        machine: "Wiper"
        step: 20 ms
        stop_time: 400 ms
        inputs: ["0 s: rain = 10", "20 ms: stalk slow", "100 ms: stalk fast", "200 ms: stalk off", "300 ms: park position"]
    }
    expects: [
        "SYS-BC-001: at(40 ms, \"Wiper\".Slow) == 1",
        "SYS-BC-001: at(120 ms, \"Wiper\".speed) == 2",
        "SYS-BC-001: at(220 ms, \"Wiper\".Parking) == 1",
        "SYS-BC-001: final(\"Wiper\".Parked) == 1",
        "final(\"Wiper\".speed) == 0"
    ]
}

test_case "TC-BC-002" {
    verifies: ["SYS-BC-002"]
    method: "test"
    description: "A door opens, the doors close, the light fades out within 2 s"
    execute: {
        machine: "Interior light"
        step: 100 ms
        stop_time: 3 s
        inputs: ["100 ms: door opened", "500 ms: doors closed"]
    }
    expects: [
        "SYS-BC-002: at(200 ms, \"Interior light\".level) == 100",
        "SYS-BC-002: at(600 ms, \"Interior light\".Fading) == 1",
        "SYS-BC-002: at(2.6 s, \"Interior light\".Off) == 1",
        "final(\"Interior light\".fades) == 1"
    ]
}
