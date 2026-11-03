model G { name: "MSM geometry" }
logical_architecture "LA" {
    id: "ARCH-LA"
    component "Controller" { id: "LC-CTRL" }
}
state_machine "Nested" {
    id: "SM-NESTED"
    owner: "LC-CTRL"
    initial: "Off"
    mode "Off" { id: "M-OFF" }
    mode "Running" {
        id: "M-RUN"
        initial: "Idle"
        mode "Idle" { id: "M-IDLE" }
        mode "Busy" {
            id: "M-BUSY"
            initial: "Warm-up"
            mode "Warm-up" { id: "M-WARM" }
            mode "Hot" { id: "M-HOT" }
            transition "Warm-up" -> "Hot" { trigger: "heated" }
            transition "Hot" -> "Warm-up" { trigger: "cooled" }
        }
        transition "Idle" -> "Busy" { trigger: "go" }
        transition "Busy" -> "Idle" { trigger: "done" }
    }
    mode "Waiting for the operator's confirmation before restart" { id: "M-WAIT" }
    transition "Off" -> "Running" { trigger: "power" }
    transition "Running" -> "Waiting for the operator's confirmation before restart" { trigger: "fault" }
    transition "Waiting for the operator's confirmation before restart" -> "Off" { trigger: "confirmed" }
}
state_machine "Lanes" {
    id: "SM-LANES"
    owner: "LC-CTRL"
    region "Power" { id: "RG-POWER" initial: "On" mode "On" { id: "M-ON" } mode "Saving" { id: "M-SAVE" } transition "On" -> "Saving" { trigger: "idle" } transition "Saving" -> "On" { trigger: "wake" } }
    region "Link" { id: "RG-LINK" initial: "Linked" mode "Linked" { id: "M-LINKED" } mode "Lost" { id: "M-LOST" } transition "Linked" -> "Lost" { trigger: "drop" } }
    region "Display" { id: "RG-DISPLAY" initial: "Dark" mode "Dark" { id: "M-DARK" } mode "Lit" { id: "M-LIT" } transition "Dark" -> "Lit" { trigger: "touch" } }
}
state_machine "Orthogonal" {
    id: "SM-ORTHO"
    owner: "LC-CTRL"
    initial: "Dual"
    mode "Dual" {
        id: "M-DUAL"
        initial: "A1"
        mode "A1" { id: "M-A1" }
        mode "A2" { id: "M-A2" }
        mode "B1" { id: "M-B1" }
        mode "B2" { id: "M-B2" }
        transition "A1" -> "A2" { trigger: "a" }
        transition "B1" -> "B2" { trigger: "b" }
    }
    mode "Triple" {
        id: "M-TRIPLE"
        initial: "X1"
        mode "X1" { id: "M-X1" }
        mode "Y1" { id: "M-Y1" }
        mode "Y2" { id: "M-Y2" }
        mode "Z1" { id: "M-Z1" }
        transition "Y1" -> "Y2" { trigger: "y" }
    }
    transition "Dual" -> "Triple" { trigger: "next" }
    transition "Triple" -> "Dual" { trigger: "back" }
}
