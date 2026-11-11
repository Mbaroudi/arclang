// The model of toggle.ino: the lamp changes on each press, not while the button is held.
model "Toggle" {
    version: "1.0.0"
}

state_machine "Lamp" {
    variables: [
        "in button: Boolean",
        "led: Boolean = false"
    ]
    initial: "OffReleased"
    state OffReleased {}
    state OnHeld { entry: "led := true" }
    state OnReleased {}
    state OffHeld { entry: "led := false" }
    transition OffReleased -> OnHeld { guard: "button" }
    transition OnHeld -> OnReleased { guard: "not button" }
    transition OnReleased -> OffHeld { guard: "button" }
    transition OffHeld -> OffReleased { guard: "not button" }
}

correspondence "Toggle sketch" {
    machine: "Lamp"
    source: "toggle.ino"
    period: 10 ms
    reads: ["digitalRead(BUTTON) is button"]
    writes: ["digitalWrite(LED) is led"]
    state: [
        "lit == 0 && was == 0 is in OffReleased",
        "lit == 1 && was == 1 is in OnHeld",
        "lit == 1 && was == 0 is in OnReleased",
        "lit == 0 && was == 1 is in OffHeld",
        "lit == 1 is led"
    ]
    invariant: ["lit == 0 || lit == 1", "was == 0 || was == 1"]
}
