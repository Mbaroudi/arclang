// The model of dimmer.ino: one step brighter at each press of `up`, out while `off` is held,
// and a heartbeat that changes at every step.
model "Dimmer" {
    version: "1.0.0"
}

state_machine "Dimmer" {
    variables: [
        "in up: Boolean",
        "in off: Boolean",
        "duty: Integer = 0",
        "beat: Boolean = false"
    ]
    initial: "Released"
    state Released {}
    state Held {}
    transition Released -> Held { guard: "up and not off" action: "duty := min(duty + 85, 255); beat := not beat" }
    transition Released -> Held { guard: "up and off" action: "duty := 0; beat := not beat" }
    transition Released -> Released { guard: "not up and off" action: "duty := 0; beat := not beat" }
    transition Released -> Released { guard: "not up and not off" action: "beat := not beat" }
    transition Held -> Held { guard: "up and off" action: "duty := 0; beat := not beat" }
    transition Held -> Held { guard: "up and not off" action: "beat := not beat" }
    transition Held -> Released { guard: "not up and off" action: "duty := 0; beat := not beat" }
    transition Held -> Released { guard: "not up and not off" action: "beat := not beat" }
}

correspondence "Dimmer sketch" {
    machine: "Dimmer"
    source: "dimmer.ino"
    period: 20 ms
    int: 16
    reads: ["digitalRead(UP) is up", "digitalRead(OFF) is off"]
    writes: ["analogWrite(LAMP) is duty", "digitalWrite(BEAT) is beat"]
    state: [
        "was == 0 is in Released",
        "was == 1 is in Held",
        "level is duty"
    ]
    invariant: ["was == 0 || was == 1", "level == 0 || level == 85 || level == 170 || level == 255"]
}
