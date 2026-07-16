system_analysis "Emergency Braking System Analysis" {
    // External Actors (MUST be on periphery per Capella Spec 4.2)
    actor "Driver" {
        id: "SA-Actor-01"
        description: "Vehicle driver"
        color: "#FFFF99"
    }
    
    actor "Vehicle Sensors" {
        id: "SA-Actor-02"
        description: "Sensor hardware interface"
        color: "#FFFF99"
    }
    
    actor "Brake System" {
        id: "SA-Actor-03"
        description: "Physical braking system"
        color: "#FFFF99"
    }
    
    // System Functions (MUST be inside boundary per Capella Spec 4.2)
    system_function "Detect Obstacles" {
        id: "SF-001"
        description: "Process sensor data to detect obstacles"
        color: "#ADD8E6"
        
        port "sensor_data" { direction: In }
        port "obstacles" { direction: Out }
    }
    
    system_function "Calculate Risk" {
        id: "SF-002"
        description: "Assess collision risk"
        color: "#ADD8E6"
        
        port "obstacles" { direction: In }
        port "risk_level" { direction: Out }
    }
    
    system_function "Decide Action" {
        id: "SF-003"
        description: "Determine braking action"
        color: "#ADD8E6"
        safety_level: "ASIL_D"
        
        port "risk_level" { direction: In }
        port "driver_override" { direction: In }
        port "brake_command" { direction: Out }
        port "warning" { direction: Out }
    }
    
    system_function "Apply Brakes" {
        id: "SF-004"
        description: "Execute brake command"
        color: "#ADD8E6"
        safety_level: "ASIL_D"
        
        port "brake_command" { direction: In }
        port "brake_signal" { direction: Out }
    }
    
    // Functional Exchanges (connections between functions and actors)
    functional_exchange "SA-Actor-02.sensor_out" -> "SF-001.sensor_data" {
        label: "Sensor Data"
        data_type: "SensorArray"
    }
    
    functional_exchange "SF-001.obstacles" -> "SF-002.obstacles" {
        label: "Detected Obstacles"
        data_type: "ObjectList"
    }
    
    functional_exchange "SF-002.risk_level" -> "SF-003.risk_level" {
        label: "Risk Assessment"
        data_type: "RiskLevel"
    }
    
    functional_exchange "SA-Actor-01.driver_input" -> "SF-003.driver_override" {
        label: "Driver Override"
        data_type: "Boolean"
    }
    
    functional_exchange "SF-003.brake_command" -> "SF-004.brake_command" {
        label: "Brake Command"
        data_type: "BrakeRequest"
    }
    
    functional_exchange "SF-003.warning" -> "SA-Actor-01.warning_in" {
        label: "Warning Signal"
        data_type: "WarningLevel"
    }
    
    functional_exchange "SF-004.brake_signal" -> "SA-Actor-03.brake_in" {
        label: "Brake Signal"
        data_type: "BrakeSignal"
    }
}
