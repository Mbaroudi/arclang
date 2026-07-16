operational_analysis "Vehicle System" {
    actor "Driver" {
        id: "ACT-001"
        description: "Person operating the vehicle"
    }
    
    actor "Vehicle" {
        id: "ACT-002"
        description: "The vehicle being driven"
    }
}

system_analysis "Advanced Cruise Control" {
    requirement "SYS-001" {
        description: "System shall maintain set speed within ±2 km/h"
        priority: Critical
        safety_level: ASIL_D
    }
    
    requirement "SYS-002" {
        description: "System shall detect obstacles within 150m range"
        priority: Critical
        safety_level: ASIL_D
    }
}

logical_architecture "ACC Control System" {
    component "User Interface" {
        id: "LA-001"
        type: "Software"
        level: "LA"
        safety_level: ASIL_B
        
        interface_in "Driver Commands" {
            protocol: "CAN"
        }
        
        interface_out "System Status" {
            protocol: "CAN"
        }
    }
    
    component "Speed Controller" {
        id: "LA-002"
        type: "Software"
        level: "LA"
        safety_level: ASIL_D
        
        interface_in "Target Speed" {
            protocol: "Internal"
        }
        
        interface_in "Current Speed" {
            protocol: "CAN"
        }
        
        interface_out "Throttle Command" {
            protocol: "CAN"
        }
        
        interface_out "Brake Command" {
            protocol: "CAN"
        }
    }
    
    component "Radar System" {
        id: "LA-003"
        type: "Hardware"
        level: "LA"
        safety_level: ASIL_D
        
        interface_in "North Sensor Data" {
            protocol: "SPI"
        }
        
        interface_out "Distance Measurement" {
            protocol: "CAN"
        }
        
        interface_out "South Debug Port" {
            protocol: "UART"
        }
    }
    
    component "Safety Monitor" {
        id: "LA-004"
        type: "Software"
        level: "LA"
        safety_level: ASIL_D
        
        interface_in "System State" {
            protocol: "Internal"
        }
        
        interface_out "Safety Alert" {
            protocol: "CAN"
        }
    }
    
    component "Vehicle Network" {
        id: "LA-005"
        type: "Hardware"
        level: "LA"
        
        interface_in "CAN Bus Data" {
            protocol: "CAN"
        }
        
        interface_out "Network Status" {
            protocol: "CAN"
        }
    }
}

physical_architecture "ECU Deployment" {
    component "Central ECU" {
        id: "PA-001"
        type: "Hardware"
        level: "PA"
        
        interface_in "Power Supply" {
            protocol: "Power"
        }
        
        interface_out "Vehicle Bus" {
            protocol: "CAN"
        }
    }
    
    component "Sensor ECU" {
        id: "PA-002"
        type: "Hardware"
        level: "PA"
        
        interface_in "Sensor Input" {
            protocol: "SPI"
        }
        
        interface_out "Data Output" {
            protocol: "CAN"
        }
    }
}

trace "SYS-001" -> "LA-002" {
    trace_type: "satisfies"
    rationale: "Speed controller implements speed maintenance requirement"
}

trace "SYS-002" -> "LA-003" {
    trace_type: "satisfies"
    rationale: "Radar system provides obstacle detection capability"
}

trace "LA-001" -> "PA-001" {
    trace_type: "allocated_to"
    rationale: "UI functions deployed to central ECU"
}

trace "LA-002" -> "PA-001" {
    trace_type: "allocated_to"
    rationale: "Speed control algorithms run on central ECU"
}

trace "LA-003" -> "PA-002" {
    trace_type: "allocated_to"
    rationale: "Radar processing runs on dedicated sensor ECU"
}
