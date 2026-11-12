// The lamp of uno_lamp.arc, proven against the binary the Arduino IDE builds of its sketch
// (uno_lamp_lto.hex, link-time optimisation): setup(), loop() and the Arduino functions are folded
// into main, so no function is named: the correspondence places the cycle of main, and the whole of
// the machine code is run on a model of the chip. `tick` is how often a wait reads the timer: the
// proof holds for that step. The addresses are those of the binary (two bytes for an int, four for
// a long, low byte first).
model "Uno lamp, as the Arduino IDE builds it" { version: "1.0.0" }

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

correspondence "Uno lamp optimised binary" {
    machine: "Lamp toggled by a button"
    binary: "uno_lamp_lto.hex"
    period: 10 ms
    tick: 1000 us
    functions: ["cycle at 0x0390"]
    reads: ["digitalRead(2) is button"]
    writes: ["digitalWrite(13) is led"]
    state: [
        "word(0x0108) == 0 && word(0x0106) == 0 is in Up",
        "word(0x0108) == 0 && word(0x0106) == 1 is in Arming",
        "word(0x0108) == 1 && word(0x0106) == 0 is in Down",
        "word(0x0108) == 1 && word(0x0106) == 1 is in Releasing",
        "millis() - dword(0x0102) is time in Arming",
        "millis() - dword(0x0102) is time in Releasing",
        "word(0x0100) == 1 is led"
    ]
    invariant: [
        "word(0x0108) == 0 || word(0x0108) == 1",
        "word(0x0106) == 0 || word(0x0106) == 1",
        "word(0x0100) == 0 || word(0x0100) == 1"
    ]
}
