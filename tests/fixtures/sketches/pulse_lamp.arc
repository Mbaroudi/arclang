// The model of pulse_lamp.ino: the lamp changes at each pulse.
model "Pulse lamp" {
    version: "1.0.0"
}

state_machine "Lamp" {
    variables: [
        "led: Boolean = false"
    ]
    initial: "Dark"
    state Dark {}
    state Lit {}
    transition Dark -> Lit { trigger: "pulse" action: "led := true" }
    transition Lit -> Dark { trigger: "pulse" action: "led := false" }
}

correspondence "Pulse lamp sketch" {
    machine: "Lamp"
    source: "pulse_lamp.ino"
    period: 10 ms
    int: 16
    events: ["RISING on SENSOR is pulse"]
    writes: ["digitalWrite(LAMP) is led"]
    state: [
        "lit == 0 is in Dark",
        "lit == 1 is in Lit",
        "lit == 1 is led"
    ]
    invariant: ["lit == 0 || lit == 1", "pulses == 0"]
}
