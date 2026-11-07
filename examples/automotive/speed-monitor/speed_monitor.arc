// A speed monitor: data flow between machines (docs/embedded-c.md, "Data
// flow between machines").
//
// The wheel speed is measured with noise; a data-flow block filters it (a
// machine with equations and no state), in the programmable logic, in fixed
// point; the overspeed warning, a state machine on the processor, reads the
// filtered speed through a wire (`in speed: ... from "Wheel speed
// filter".speed`) and turns the warning on above 130 km/h, off below
// 125 km/h. The wire converts m/s to km/h as an assignment converts, and the
// block's range gives the warning's input its fixed point.
//
//   arclang sil speed_monitor.arc      the C of both, back to back with the model
//   arclang hdl speed_monitor.arc      the block as VHDL, bit-exact with its fixed-point C
//   arclang cosim speed_monitor.arc    both as one FMU in OMSimulator (docs/cosimulation.md)

model SpeedMonitor {
    metadata {
        version: "1.0.0"
        description: "A filter block in the logic feeding a warning on the processor"
    }
}

requirements system {
    req "SYS-SPD-001" "Filtered speed" {
        description: "The speed the warning reads shall be the measured wheel speed through a first-order filter, limited to 0 .. 250 km/h"
        priority: "High"
    }
    req "SYS-SPD-002" "Overspeed warning" {
        description: "The warning shall come on within 300 ms of the wheel speed going above 130 km/h, and go off once the filtered speed is below 125 km/h"
        priority: "Critical"
    }
}

physical_architecture "Speed monitor hardware" {
    node "Monitor SoC" {
        id: "PN-SPD-SOC"
        processor: "VexRiscv (LiteX)"
        behavior_component "Warning logic" { id: "PC-SPD-WARN" }
    }
    node "Monitor fabric" {
        id: "PN-SPD-FABRIC"
        kind: "fpga"
        device: "LFE5U-25F-6BG381C"
        clock: 25 MHz
        behavior_component "Speed filter" { id: "PC-SPD-FILTER" }
    }
}

// The data-flow block: no state, its equations every step.
state_machine "Wheel speed filter" {
    id: "SM-SPD-FILTER"
    owner: "PC-SPD-FILTER"
    variables: [
        "in raw: Real [km/h] range 0 km/h .. 300 km/h",
        "filtered: Real [km/h] range 0 km/h .. 300 km/h = 0 km/h",
        "out speed: Real [m/s] range 0 m/s .. 70 m/s = 0 m/s"
    ]
    equations: [
        "filtered = filtered + 0.25 * (raw - filtered)",
        "speed = saturate(filtered, 0 km/h, 250 km/h)"
    ]
}

state_machine "Overspeed warning" {
    id: "SM-SPD-WARN"
    owner: "PC-SPD-WARN"
    variables: [
        "in speed: Real [km/h] from \"Wheel speed filter\".speed",
        "const on_above: Real [km/h] = 130 km/h",
        "const off_below: Real [km/h] = 125 km/h"
    ]
    initial: "Normal"
    state Normal {}
    state Warning {}
    transition Normal -> Warning { guard: "speed > on_above" }
    transition Warning -> Normal { guard: "speed < off_below" }
}

test_case "TC-SPD-001" {
    verifies: ["SYS-SPD-001", "SYS-SPD-002"]
    method: "test"
    description: "A step to 140 km/h: the filtered speed rises, the warning comes on in time"
    execute: {
        machine: "Overspeed warning"
        step: 10 ms
        stop_time: 400 ms
        inputs: ["0 s: \"Wheel speed filter\".raw = 140 km/h"]
    }
    expects: [
        "SYS-SPD-001: at(0 s, \"Overspeed warning\".speed) == 35 km/h",
        "SYS-SPD-001: at(10 ms, \"Overspeed warning\".speed) < 100 km/h",
        "SYS-SPD-002: at(100 ms, \"Overspeed warning\".Warning) == 1",
        "SYS-SPD-002: at(300 ms, \"Overspeed warning\".Warning) == 1"
    ]
}

test_case "TC-SPD-002" {
    verifies: ["SYS-SPD-002"]
    method: "test"
    description: "Above the limit, then back to 120 km/h: the warning goes off"
    execute: {
        machine: "Overspeed warning"
        step: 10 ms
        stop_time: 1 s
        inputs: ["0 s: \"Wheel speed filter\".raw = 140 km/h", "500 ms: \"Wheel speed filter\".raw = 120 km/h"]
    }
    expects: [
        "SYS-SPD-002: at(490 ms, \"Overspeed warning\".Warning) == 1",
        "SYS-SPD-002: final(\"Overspeed warning\".Normal) == 1"
    ]
}

test_case "TC-SPD-003" {
    verifies: ["SYS-SPD-001"]
    method: "test"
    description: "The block alone: it settles on the measured speed and holds the limit"
    execute: {
        machine: "Wheel speed filter"
        step: 10 ms
        stop_time: 600 ms
        inputs: ["0 s: raw = 140 km/h", "300 ms: raw = 290 km/h"]
    }
    expects: [
        "SYS-SPD-001: at(290 ms, \"Wheel speed filter\".speed) > 139.9 km/h",
        "SYS-SPD-001: final(\"Wheel speed filter\".speed) <= 250 km/h",
        "SYS-SPD-001: final(\"Wheel speed filter\".speed) > 249.9 km/h"
    ]
}
