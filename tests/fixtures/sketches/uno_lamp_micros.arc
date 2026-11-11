// The model of uno_lamp_micros.ino: the model of uno_lamp.ino, the time read on the finer clock.
model "Uno lamp" { version: "1.0.0" }

state_machine "Lamp toggled by a button" {
    variables: [
        "in button: Boolean",
        "led: Boolean = false"
    ]
    initial: "Up"
    state Up {}
    state Arming {}
    state Down {}
    state Releasing {}
    transition Up -> Arming { guard: "not button" }
    transition Arming -> Down { guard: "not button" timing: 50 ms action: "led := not led" }
    transition Arming -> Up { guard: "button" }
    transition Down -> Releasing { guard: "button" }
    transition Releasing -> Up { guard: "button" timing: 50 ms }
    transition Releasing -> Down { guard: "not button" }
}

correspondence "Uno lamp sketch" {
    machine: "Lamp toggled by a button"
    source: "uno_lamp_micros.ino"
    period: 10 ms
    int: 16
    reads: ["digitalRead(BUTTON) is button"]
    writes: ["digitalWrite(LAMP) is led"]
    state: [
        "pressed == 0 && counting == 0 is in Up",
        "pressed == 0 && counting == 1 is in Arming",
        "pressed == 1 && counting == 0 is in Down",
        "pressed == 1 && counting == 1 is in Releasing",
        "(micros() - since) / 1000 is time in Arming",
        "(micros() - since) / 1000 is time in Releasing",
        "lit == 1 is led"
    ]
    invariant: [
        "pressed == 0 || pressed == 1",
        "counting == 0 || counting == 1",
        "lit == 0 || lit == 1"
    ]
}
