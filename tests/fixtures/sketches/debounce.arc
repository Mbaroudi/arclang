// The model of debounce.ino: a change of the button counts once it has held for 50 ms.
model "Debounce" {
    version: "1.0.0"
}

state_machine "Debounced button" {
    variables: [
        "in button: Boolean",
        "led: Boolean = false"
    ]
    initial: "Low"
    state Low {}
    state Rising {}
    state High { entry: "led := true" }
    state Falling {}
    transition Low -> Rising { guard: "button" }
    transition Rising -> High { guard: "button" timing: 50 ms }
    transition Rising -> Low { guard: "not button" }
    transition High -> Falling { guard: "not button" }
    transition Falling -> Low { guard: "not button" timing: 50 ms action: "led := false" }
    transition Falling -> High { guard: "button" }
}

correspondence "Debounce sketch" {
    machine: "Debounced button"
    source: "debounce.ino"
    period: 10 ms
    reads: ["digitalRead(BUTTON) is button"]
    writes: ["digitalWrite(LED) is led"]
    state: [
        "stable == 0 && counting == 0 is in Low",
        "stable == 0 && counting == 1 is in Rising",
        "stable == 1 && counting == 0 is in High",
        "stable == 1 && counting == 1 is in Falling",
        "millis() - since is time in Rising",
        "millis() - since is time in Falling",
        "stable == 1 is led"
    ]
    invariant: ["stable == 0 || stable == 1", "counting == 0 || counting == 1"]
}
