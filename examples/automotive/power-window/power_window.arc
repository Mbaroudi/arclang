// ============================================================================
// POWER WINDOW CONTROLLER - pseudo-states and regions (docs/embedded-c.md)
// ============================================================================
// A window lift whose active mode holds two orthogonal regions (the motion
// and the anti-pinch monitor), with a choice on the position, a junction on
// the key, a fork that resumes both regions from their history (deep for
// the motion, shallow for the monitor), a join that stops on both, a
// transition across the regions, and a terminate on a fatal fault. The
// generated C is proven against the model by `arclang sil`.
//
//   arclang check  examples/automotive/power-window/power_window.arc
//   arclang export examples/automotive/power-window/power_window.arc -f c -o build/window
//   arclang sil    examples/automotive/power-window/power_window.arc
// ============================================================================

model PowerWindow {
    metadata {
        version: "1.0.0"
        description: "Window lift controller: motion, anti-pinch, resume"
        domain: "automotive"
    }
}

requirements system {
    req "SYS-PW-001" "Auto direction from the position" {
        description: "An auto request shall move the window up below half-open and down otherwise"
        priority: "High"
    }
    req "SYS-PW-002" "Resume after sleep" {
        description: "Waking shall resume the motion and the anti-pinch monitor where they were"
        priority: "High"
    }
    req "SYS-PW-003" "Pinch stops the motion" {
        description: "Releasing a pinch shall stop the motion and re-arm the monitor"
        priority: "Critical"
    }
    req "SYS-PW-004" "Out-of-range position is a fault" {
        description: "A key with an out-of-range position shall go to fault; a fatal fault stops the controller"
        priority: "Critical"
    }
}

state_machine "Window" {
    variables: ["in position: Integer", "out commands: Integer = 0", "out pinches: Integer = 0"]
    initial: "Off"
    state Off {}
    state Active {
        region "Motion" {
            initial: "Stopped"
            state Stopped {}
            state Moving {
                initial: "Up"
                state Up { entry: "commands := commands + 1" }
                state Down { entry: "commands := commands + 1" }
                transition Up -> Down { trigger: "reverse" }
                transition Down -> Up { trigger: "reverse" }
            }
            state Resume { kind: "deep_history" }
            state Decide { kind: "choice" }
            transition Stopped -> Decide { trigger: "auto" }
            transition Decide -> Up { guard: "position < 50" }
            transition Decide -> Down { guard: "else" }
            transition Moving -> Stopped { trigger: "stop" }
        }
        region "Safety" {
            initial: "Watching"
            state Watching {}
            state Pinched { entry: "pinches := pinches + 1" }
            state Monitor { kind: "shallow_history" }
            transition Watching -> Pinched { trigger: "obstacle" }
        }
        transition Pinched -> Stopped { trigger: "release" }
    }
    state Fault {}
    state Gate { kind: "junction" }
    state Wake { kind: "fork" }
    state Both { kind: "join" }
    state Dead { kind: "terminate" }
    transition Off -> Gate { trigger: "key" }
    transition Gate -> Active { guard: "position <= 100" }
    transition Gate -> Fault { guard: "else" }
    transition Active -> Off { trigger: "sleep" }
    transition Off -> Wake { trigger: "wake" }
    transition Wake -> Resume {}
    transition Wake -> Monitor {}
    transition Pinched -> Both { trigger: "shutdown" }
    transition Stopped -> Both { trigger: "shutdown" }
    transition Both -> Off {}
    transition Fault -> Dead { trigger: "fatal" }
}

test_case "TC-PW-001" {
    verifies: ["SYS-PW-001", "SYS-PW-002", "SYS-PW-003", "SYS-PW-004"]
    method: "test"
    description: "Key, auto up, reverse, pinch, sleep, wake from history, release, shutdown on both, fault, terminate"
    execute: {
        machine: "Window"
        step: 10 ms
        stop_time: 150 ms
        inputs: [
            "0 s: position = 10",
            "10 ms: key", "20 ms: auto", "30 ms: reverse", "40 ms: obstacle",
            "50 ms: sleep", "60 ms: wake", "70 ms: release", "80 ms: obstacle",
            "90 ms: shutdown", "100 ms: position = 150", "110 ms: key",
            "120 ms: fatal", "130 ms: key"
        ]
    }
    expects: [
        "at(10 ms, \"Window\".Active.Stopped) == 1",
        "at(10 ms, \"Window\".Active.Watching) == 1",
        "SYS-PW-001: at(20 ms, \"Window\".Active.Moving.Up) == 1",
        "at(30 ms, \"Window\".Active.Moving.Down) == 1",
        "at(40 ms, \"Window\".Active.Pinched) == 1",
        "at(50 ms, \"Window\".Off) == 1",
        "SYS-PW-002: at(60 ms, \"Window\".Active.Moving.Down) == 1",
        "SYS-PW-002: at(60 ms, \"Window\".Active.Pinched) == 1",
        "SYS-PW-002: at(60 ms, \"Window\".commands) == 3",
        "SYS-PW-003: at(70 ms, \"Window\".Active.Stopped) == 1",
        "SYS-PW-003: at(70 ms, \"Window\".Active.Watching) == 1",
        "at(90 ms, \"Window\".Off) == 1",
        "SYS-PW-004: at(110 ms, \"Window\".Fault) == 1",
        "SYS-PW-004: final(\"Window\".Off) == 0",
        "SYS-PW-004: final(\"Window\".Fault) == 0",
        "final(\"Window\".pinches) == 3"
    ]
}

test_case "TC-PW-002" {
    verifies: ["SYS-PW-001"]
    method: "test"
    description: "Above half-open, auto goes down; waking before any sleep takes the regions' initial states"
    execute: {
        machine: "Window"
        stop_time: 120 ms
        inputs: ["0 s: position = 80", "10 ms: wake", "20 ms: auto", "30 ms: stop"]
    }
    expects: [
        "at(10 ms, \"Window\".Active.Stopped) == 1",
        "at(10 ms, \"Window\".Active.Watching) == 1",
        "SYS-PW-001: at(20 ms, \"Window\".Active.Moving.Down) == 1",
        "at(30 ms, \"Window\".Active.Stopped) == 1",
        "final(\"Window\".commands) == 1"
    ]
}
