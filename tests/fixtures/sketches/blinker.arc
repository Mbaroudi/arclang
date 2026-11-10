// The model of blinker.ino, as an AI proposed it and `arclang prove` proved it.
model "Blinker" { version: "1.0.0" }

state_machine "Blinking lamp" {
    variables: [
        "in button: Boolean",
        "led: Boolean = false"
    ]
    initial: "Dark"
    state Dark {}
    state Lit { entry: "led := true" }
    transition Dark -> Lit { guard: "button" timing: 500 ms }
    transition Dark -> Dark { guard: "not button" }
    transition Lit -> Dark { guard: "button" timing: 500 ms action: "led := false" }
    transition Lit -> Dark { guard: "not button" action: "led := false" }
}

correspondence "Blinker sketch" {
    machine: "Blinking lamp"
    source: "blinker.ino"
    period: 20 ms
    reads: ["digitalRead(SWITCH) is button"]
    writes: ["digitalWrite(LAMP) is led"]
    state: [
        "lit == 0 is in Dark",
        "lit == 1 is in Lit",
        "millis() - changed is time in Dark",
        "millis() - changed is time in Lit",
        "lit == 1 is led"
    ]
    invariant: ["lit == 0 || lit == 1"]
}
