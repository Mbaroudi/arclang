// A lamp on a PWM pin (pin 9): pressing the button (pin 2, pressed = LOW) sets it to half brightness, releasing
// puts it out. Proven against the binary the Arduino IDE builds of it (uno_pwm_lto.hex): the cycle is placed, no
// function is named; the value of the pin is what the registers of its timer give.
model "Uno PWM lamp, as the Arduino IDE builds it" { version: "1.0.0" }

state_machine "PWM lamp" {
    variables: [
        "in level: Boolean",
        "duty: Integer = 0"
    ]
    initial: "On"
    state On {}
    transition On -> On { guard: "not level" action: "duty := 128" }
    transition On -> On { guard: "level" action: "duty := 0" }
}

correspondence "Uno PWM lamp optimised binary" {
    machine: "PWM lamp"
    binary: "uno_pwm_lto.hex"
    period: 10 ms
    tick: 1000 us
    functions: ["cycle at 0x03aa"]
    reads: ["digitalRead(2) is level"]
    writes: ["analogWrite(9) is duty"]
}
