// A lamp (pin 13) toggled at each press of a button (pin 2, pressed = LOW), that prints its state on the
// serial port each time it changes. Proven against the binary the Arduino IDE builds of it
// (uno_serial_lto.hex, link-time optimisation): the cycle is placed, no function is named; the serial port
// is run on the model of the chip (its interrupt empties the transmit buffer), and what it prints is not
// looked at. The bytes: `lit` at 0x0116 and `was` at 0x0118 (two bytes each, low byte first).
model "Uno serial lamp, as the Arduino IDE builds it" { version: "1.0.0" }

state_machine "Serial lamp" {
    variables: [
        "in button: Boolean",
        "led: Boolean = false"
    ]
    initial: "Up"
    state Up {}
    state Down {}
    transition Up -> Down { guard: "not button" action: "led := not led" }
    transition Down -> Up { guard: "button" }
}

correspondence "Uno serial lamp optimised binary" {
    machine: "Serial lamp"
    binary: "uno_serial_lto.hex"
    period: 10 ms
    tick: 1000 us
    functions: ["cycle at 0x06ce"]
    reads: ["digitalRead(2) is button"]
    writes: ["digitalWrite(13) is led"]
    state: [
        "word(0x0118) == 0 is in Up",
        "word(0x0118) == 1 is in Down",
        "word(0x0116) == 1 is led"
    ]
    invariant: [
        "word(0x0118) == 0 || word(0x0118) == 1",
        "word(0x0116) == 0 || word(0x0116) == 1"
    ]
}
