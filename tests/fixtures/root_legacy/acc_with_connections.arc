logical_architecture "ACC System with Connections" {
    component "RadarSensor" {
        id: "LC-001"
        description: "77GHz radar for distance measurement"
    }

    component "ACCController" {
        id: "LC-002"
        description: "Main ACC control logic"
    }

    component "BrakeActuator" {
        id: "LC-003"
        description: "Brake system interface"
    }

    component "ThrottleActuator" {
        id: "LC-004"
        description: "Throttle control interface"
    }

    component "SpeedSensor" {
        id: "LC-005"
        description: "Vehicle speed measurement"
    }

}

system_analysis "ACC Requirements" {
    requirement "SYS-001" {
        description: "System shall maintain 2s following distance"
        priority: "Critical"
    }

    requirement "SYS-002" {
        description: "System shall detect vehicles up to 200m"
        priority: "Critical"
    }
}

trace "LC-001" satisfies "SYS-002" {}
trace "LC-002" satisfies "SYS-001" {}
