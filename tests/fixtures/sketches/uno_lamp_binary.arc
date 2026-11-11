// The lamp of uno_lamp.arc, proven against the binary flashed on the board (uno_lamp.hex)
// and not against its sketch: the machine code is read, no source is.
// The addresses are those of the binary: its functions, and the bytes of its memory
// that hold its state (two for an int, four for a long, low byte first).
model "Uno lamp, as flashed" { version: "1.0.0" }

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

correspondence "Uno lamp binary" {
    machine: "Lamp toggled by a button"
    binary: "uno_lamp.hex"
    period: 10 ms
    functions: [
        "loop at 0x0102",
        "pinMode at 0x043e",
        "digitalRead at 0x0514",
        "digitalWrite at 0x04b6",
        "millis at 0x029c",
        "delay at 0x02fe"
    ]
    reads: ["digitalRead(2) is button"]
    writes: ["digitalWrite(13) is led"]
    state: [
        "word(0x0106) == 0 && word(0x0104) == 0 is in Up",
        "word(0x0106) == 0 && word(0x0104) == 1 is in Arming",
        "word(0x0106) == 1 && word(0x0104) == 0 is in Down",
        "word(0x0106) == 1 && word(0x0104) == 1 is in Releasing",
        "millis() - dword(0x0100) is time in Arming",
        "millis() - dword(0x0100) is time in Releasing",
        "word(0x0108) == 1 is led"
    ]
    invariant: [
        "word(0x0106) == 0 || word(0x0106) == 1",
        "word(0x0104) == 0 || word(0x0104) == 1",
        "word(0x0108) == 0 || word(0x0108) == 1"
    ]
}
