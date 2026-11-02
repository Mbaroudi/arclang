model AdaptiveCruiseControl {
    metadata {
        version: "1.0"
        description: "ACC System"
        domain: "automotive"
        safety_standard: "iso26262"
        safety_level: "ASIL_B"
    }
}

requirements stakeholder {
    req "STK-001" "Driver Speed Control" {
        description: "The driver shall be able to set and maintain a desired cruising speed"
        priority: High
    }
    
    req "STK-002" "Automatic Distance Maintenance" {
        description: "The system shall automatically maintain safe following distance from vehicles ahead"
        priority: Critical
    }
    
    req "STK-003" "Driver Override Capability" {
        description: "The driver shall be able to override ACC at any time using brake or accelerator"
        priority: Critical
    }
}

requirements system {
    req "SYS-001" "Speed Range Operation" {
        description: "The ACC system shall operate in the speed range of 30-180 km/h"
        priority: High
    }
    
    req "SYS-002" "Target Speed Setting" {
        description: "The system shall allow speed setting in 5 km/h increments"
        priority: Medium
    }
    
    req "SYS-003" "Radar Detection Range" {
        description: "The forward radar shall detect vehicles up to 200 meters ahead"
        priority: Critical
        safety_level: ASIL_B
    }
}

requirements safety {
    req "SAF-001" "Sensor Redundancy" {
        description: "Critical sensing functions shall have redundant sensors with failure detection"
        priority: Critical
        safety_level: ASIL_B
    }
    
    req "SAF-002" "Watchdog Monitoring" {
        description: "ACC controller shall be monitored by independent watchdog with 100ms timeout"
        priority: Critical
        safety_level: ASIL_B
    }
    
    req "SAF-003" "Fail-Safe State" {
        description: "Upon critical failure detection, system shall transition to safe state within 200ms"
        priority: Critical
        safety_level: ASIL_B
    }
}

architecture logical {
    component "ACCController" {
        id: "LC-001"
        description: "Main adaptive cruise control logic and coordination"
        safety_level: ASIL_B
    }
    
    component "RadarSensor" {
        id: "LC-002"
        description: "77 GHz radar for detecting vehicles and obstacles ahead"
        safety_level: ASIL_B
    }
    
    component "CameraSensor" {
        id: "LC-003"
        description: "Vision sensor for lane detection and target verification"
        safety_level: ASIL_B
    }
    
    component "SpeedSensor" {
        id: "LC-004"
        description: "Wheel speed sensors for accurate vehicle speed"
        safety_level: ASIL_B
    }
    
    component "BrakeActuator" {
        id: "LC-005"
        description: "Electronic brake system interface"
        safety_level: ASIL_B
    }
    
    component "ThrottleActuator" {
        id: "LC-006"
        description: "Electronic throttle control interface"
        safety_level: ASIL_B
    }
    
    component "DriverInterface" {
        id: "LC-007"
        description: "Driver human-machine interface for ACC control and feedback"
    }
    
    component "PedalMonitor" {
        id: "LC-008"
        description: "Monitors brake and accelerator pedal positions"
        safety_level: ASIL_B
    }
    
    component "SafetyMonitor" {
        id: "LC-009"
        description: "Independent safety monitor with watchdog functionality"
        safety_level: ASIL_B
    }
    
    component "WarningSystem" {
        id: "LC-010"
        description: "Manages visual and audible driver warnings"
        safety_level: ASIL_B
    }
    
    connection "SensorToController1" {
        from: "LC-002"
        to: "LC-001"
    }
    
    connection "SensorToController2" {
        from: "LC-003"
        to: "LC-001"
    }
    
    connection "SensorToController3" {
        from: "LC-004"
        to: "LC-001"
    }
    
    connection "ControllerToBrake" {
        from: "LC-001"
        to: "LC-005"
    }
    
    connection "ControllerToThrottle" {
        from: "LC-001"
        to: "LC-006"
    }
    
    connection "DriverToController" {
        from: "LC-007"
        to: "LC-001"
    }
    
    connection "PedalToController" {
        from: "LC-008"
        to: "LC-001"
    }
    
    connection "SafetyToController" {
        from: "LC-009"
        to: "LC-001"
    }
    
    connection "ControllerToWarning" {
        from: "LC-001"
        to: "LC-010"
    }
}
