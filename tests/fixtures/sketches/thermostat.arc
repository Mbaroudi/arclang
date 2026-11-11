// The model of thermostat.ino, and how the two relate.
model "Thermostat" {
    version: "1.0.0"
}

state_machine "Thermostat" {
    variables: [
        "in temperature: Integer",
        "heater: Boolean = false"
    ]
    initial: "Idle"
    state Idle { entry: "heater := false" }
    state Heating { entry: "heater := true" }
    transition Idle -> Heating { guard: "temperature < 400" }
    transition Heating -> Idle { guard: "temperature > 450" }
}

correspondence "Thermostat sketch" {
    machine: "Thermostat"
    source: "thermostat.ino"
    period: 100 ms
    reads: ["analogRead(SENSOR) is temperature"]
    writes: ["digitalWrite(HEATER) is heater"]
    state: ["mode == 1 is in Heating", "mode == 0 is in Idle"]
}
