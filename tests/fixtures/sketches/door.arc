// The model of door.ino, as an AI proposed it and `arclang prove` proved it.
model "Door lock" { version: "1.0.0" }

state_machine "Door lock" {
    variables: [
        "in button: Boolean",
        "in ajar: Boolean",
        "prev: Boolean = false",
        "age: Integer = 0",
        "alarm: Boolean = false"
    ]
    initial: "Closed"
    state Closed {}
    state Open {}
    transition Closed -> Open {
        guard: "button and not prev"
        action: "prev := not (not button); age := 50; alarm := false"
    }
    transition Closed -> Closed {
        guard: "not (button and not prev)"
        action: "prev := not (not button); alarm := not (not ajar)"
    }
    transition Open -> Closed {
        guard: "(button and not prev) or age >= 3000"
        action: "prev := not (not button); age := 0; alarm := not (not ajar)"
    }
    transition Open -> Open {
        guard: "not ((button and not prev) or age >= 3000)"
        action: "prev := not (not button); age := age + 50; alarm := false"
    }
}

correspondence "Door sketch" {
    machine: "Door lock"
    source: "door.ino"
    period: 50 ms
    reads: ["digitalRead(BUTTON) is button", "digitalRead(SENSOR) is ajar"]
    writes: ["digitalWrite(LOCK) is in Open", "digitalWrite(ALARM) is alarm"]
    state: [
        "open_ == 0 is in Closed",
        "open_ == 1 is in Open",
        "held == 1 is prev",
        "open_ * (millis() - opened) is age"
    ]
    invariant: [
        "open_ == 0 || open_ == 1",
        "held == 0 || held == 1",
        "open_ == 0 || millis() - opened < 3050"
    ]
}
