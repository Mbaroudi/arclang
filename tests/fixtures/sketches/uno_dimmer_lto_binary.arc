// The dimmer of uno_dimmer.ino (a press on UP takes the lamp one step brighter, 85 at a time, OFF puts it
// out, a heartbeat LED changes at every loop, the level is printed on the serial port), proven against the
// binary the Arduino IDE builds of it (uno_dimmer_lto.hex, link-time optimisation).
// The cycle is placed, no function is named. The pins: 2 (UP) and 3 (OFF) are read, 13 (the heartbeat) is
// written as a level, 9 (the lamp) as a PWM. The bytes: `was` at 0x0122 and `level` at 0x0124 (two bytes
// each, low byte first). What the code prints on the serial port is not looked at.
model "Uno dimmer, as the Arduino IDE builds it" { version: "1.0.0" }

state_machine "Dimmer" {
    variables: [
        "in up: Boolean",
        "in off: Boolean",
        "level: Integer = 0",
        "beat: Boolean = false"
    ]
    initial: "Released"
    state Released {}
    state Held {}
    transition Released -> Held { guard: "up and not off" action: "level := min(level + 85, 255); beat := not beat" }
    transition Released -> Held { guard: "up and off" action: "level := 0; beat := not beat" }
    transition Released -> Released { guard: "not up and off" action: "level := 0; beat := not beat" }
    transition Released -> Released { guard: "not up and not off" action: "beat := not beat" }
    transition Held -> Held { guard: "up and off" action: "level := 0; beat := not beat" }
    transition Held -> Held { guard: "up and not off" action: "beat := not beat" }
    transition Held -> Released { guard: "not up and off" action: "level := 0; beat := not beat" }
    transition Held -> Released { guard: "not up and not off" action: "beat := not beat" }
}

correspondence "Uno dimmer optimised binary" {
    machine: "Dimmer"
    binary: "uno_dimmer_lto.hex"
    period: 20 ms
    tick: 1000 us
    functions: ["cycle at 0x07b6"]
    reads: ["digitalRead(2) is up", "digitalRead(3) is off"]
    writes: ["digitalWrite(13) is beat", "analogWrite(9) is level"]
    state: [
        "word(0x0122) == 0 is in Released",
        "word(0x0122) == 1 is in Held",
        "word(0x0124) is level"
    ]
    invariant: [
        "word(0x0122) == 0 || word(0x0122) == 1",
        "word(0x0124) <= 255"
    ]
}
