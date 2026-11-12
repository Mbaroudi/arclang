// The lamp of uno_lamp_micros.arc, proven against the binary its sketch compiles to
// (uno_lamp_micros.hex): the machine code is read, no source is. It times its press with micros().
model "Uno lamp timed with micros, as flashed" { version: "1.0.0" }

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

correspondence "Uno lamp micros binary" {
    machine: "Lamp toggled by a button"
    binary: "uno_lamp_micros.hex"
    period: 10 ms
    functions: [
        "loop at 0x0102",
        "micros at 0x029e",
        "delay at 0x02e8",
        "pinMode at 0x0428",
        "digitalWrite at 0x04a0",
        "digitalRead at 0x04fe"
    ]
    reads: ["digitalRead(2) is button"]
    writes: ["digitalWrite(13) is led"]
    state: [
        "word(0x0106) == 0 && word(0x0104) == 0 is in Up",
        "word(0x0106) == 0 && word(0x0104) == 1 is in Arming",
        "word(0x0106) == 1 && word(0x0104) == 0 is in Down",
        "word(0x0106) == 1 && word(0x0104) == 1 is in Releasing",
        "(micros() - dword(0x0100)) / 1000 is time in Arming",
        "(micros() - dword(0x0100)) / 1000 is time in Releasing",
        "word(0x0108) == 1 is led"
    ]
    invariant: [
        "word(0x0106) == 0 || word(0x0106) == 1",
        "word(0x0104) == 0 || word(0x0104) == 1",
        "word(0x0108) == 0 || word(0x0108) == 1"
    ]
}
