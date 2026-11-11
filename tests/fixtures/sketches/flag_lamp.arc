// The model of flag_lamp.ino: the lamp changes once in a step that had a pulse, or several.
model "Flag lamp" {
    version: "1.0.0"
}

state_machine "Lamp" {
    variables: [
        "armed: Boolean = false",
        "led: Boolean = false"
    ]
    initial: "Dark"
    state Dark {}
    state Lit {}
    transition Dark -> Dark { trigger: "pulse" action: "armed := true" }
    transition Lit -> Lit { trigger: "pulse" action: "armed := true" }
    transition Dark -> Lit { guard: "armed" action: "led := true; armed := false" }
    transition Lit -> Dark { guard: "armed" action: "led := false; armed := false" }
}

correspondence "Flag lamp sketch" {
    machine: "Lamp"
    source: "flag_lamp.ino"
    period: 10 ms
    int: 16
    events: ["RISING on SENSOR is pulse"]
    writes: ["digitalWrite(LAMP) is led"]
    state: [
        "lit == 0 is in Dark",
        "lit == 1 is in Lit",
        "lit == 1 is led",
        "pulsed is armed"
    ]
    invariant: ["lit == 0 || lit == 1", "pulsed == 0"]
}
