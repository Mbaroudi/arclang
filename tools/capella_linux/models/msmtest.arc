model T { name: "MSM test" }
system_analysis "SA" {
    id: "ARCH-SA"
    function "Sense" { id: "SF-Sense" }
    system_component "Controller" { id: "SC-Ctrl" allocated_functions: ["SF-Sense"] }
}
state_machine "Modes" {
    id: "SM-1"
    owner: "SC-Ctrl"
    initial: "Off"
    state "Off" { id: "ST-Off" }
    state "Running" {
        id: "ST-Run"
        initial: "Idle"
        state "Idle" { id: "ST-Idle" }
        state "Busy" { id: "ST-Busy" }
        state "Done" { id: "ST-Done" kind: "final" }
        transition "Idle" -> "Busy" { trigger: "go" }
        transition "Busy" -> "Done" { trigger: "finish" guard: "ready" }
    }
    state "Check" { id: "ST-Check" kind: "choice" }
    transition "Off" -> "Running" { trigger: "power" }
    transition "Running" -> "Check" { trigger: "stop" }
    transition "Check" -> "Off" { trigger: "" guard: "cold" }
    transition "Check" -> "Running" { trigger: "" guard: "warm" }
    transition "Off" -> "Off" { trigger: "tick" }
}
state_machine "Two regions" {
    id: "SM-2"
    owner: "SC-Ctrl"
    region "Power" { id: "RG-1" initial: "On" state "On" { id: "ST-On" } state "Off2" { id: "ST-Off2" } transition "On" -> "Off2" { trigger: "cut" } transition "Off2" -> "On" { trigger: "restore" } }
    region "Link" { initial: "Linked" state "Linked" { id: "ST-Linked" } state "Lost" { id: "ST-Lost" } transition "Linked" -> "Lost" { trigger: "drop" } }
}
