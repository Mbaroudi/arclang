system_analysis "Emergency Braking Dataflow" {
    
    // High-level functions
    system_function "Sensor Data Acquisition" {
        id: "SF-001"
        category: "Input"
        color: "#70AD47"
    }
    
    system_function "Multi-Sensor Fusion" {
        id: "SF-002"
        category: "Processing"
        color: "#4472C4"
    }
    
    system_function "Object Detection & Classification" {
        id: "SF-003"
        category: "Perception"
        color: "#9E7BB5"
    }
    
    system_function "Multi-Object Tracking" {
        id: "SF-004"
        category: "Perception"
        color: "#9E7BB5"
    }
    
    system_function "Collision Risk Assessment" {
        id: "SF-005"
        category: "Decision"
        color: "#ED7D31"
    }
    
    system_function "Braking Strategy Planning" {
        id: "SF-006"
        category: "Planning"
        color: "#FFC000"
    }
    
    system_function "Brake Actuation" {
        id: "SF-007"
        category: "Control"
        color: "#C00000"
    }
    
    system_function "Driver Warning" {
        id: "SF-008"
        category: "HMI"
        color: "#5B9BD5"
    }
    
    system_function "System Health Monitoring" {
        id: "SF-009"
        category: "Safety"
        color: "#43682B"
    }
    
    // Functional exchanges (data flows)
    functional_exchange {
        from: "SF-001"
        to: "SF-002"
        data: "Raw Sensor Data"
        description: "Radar, Camera, Lidar streams"
        rate: "20-100 Hz"
        protocol: "CAN/Ethernet"
    }
    
    functional_exchange {
        from: "SF-002"
        to: "SF-003"
        data: "Fused Sensor Data"
        description: "Synchronized, validated sensor data"
        rate: "20 Hz"
        protocol: "Internal"
    }
    
    functional_exchange {
        from: "SF-003"
        to: "SF-004"
        data: "Object Detections"
        description: "Classified objects with positions"
        rate: "20 Hz"
        protocol: "Internal"
    }
    
    functional_exchange {
        from: "SF-004"
        to: "SF-005"
        data: "Tracked Objects"
        description: "Object tracks with trajectories"
        rate: "20 Hz"
        protocol: "Internal"
    }
    
    functional_exchange {
        from: "SF-005"
        to: "SF-006"
        data: "Threat Assessment"
        description: "Collision risk level (Safe/Warning/Critical)"
        rate: "20 Hz"
        protocol: "Internal"
    }
    
    functional_exchange {
        from: "SF-006"
        to: "SF-007"
        data: "Brake Command"
        description: "Target deceleration profile"
        rate: "100 Hz"
        protocol: "CAN"
    }
    
    functional_exchange {
        from: "SF-005"
        to: "SF-008"
        data: "Warning Signal"
        description: "Driver alert level"
        rate: "10 Hz"
        protocol: "Internal"
    }
    
    functional_exchange {
        from: "SF-007"
        to: "SF-006"
        data: "Actual Deceleration"
        description: "Feedback from brake system"
        rate: "100 Hz"
        protocol: "CAN"
    }
    
    functional_exchange {
        from: "SF-001"
        to: "SF-009"
        data: "Sensor Status"
        description: "Health and availability info"
        rate: "1 Hz"
        protocol: "Internal"
    }
    
    functional_exchange {
        from: "SF-009"
        to: "SF-008"
        data: "System Status"
        description: "Overall system health"
        rate: "1 Hz"
        protocol: "Internal"
    }
    
    functional_exchange {
        from: "SF-002"
        to: "SF-009"
        data: "Data Quality"
        description: "Sensor fusion confidence"
        rate: "1 Hz"
        protocol: "Internal"
    }
    
    // Functional chain: Critical braking path
    functional_chain "Emergency Braking Chain" {
        id: "FC-001"
        description: "Critical path from sensing to braking"
        functions: ["SF-001", "SF-002", "SF-003", "SF-004", "SF-005", "SF-006", "SF-007"]
        color: "#C00000"
        priority: "Critical"
    }
    
    // Functional chain: Driver notification
    functional_chain "Warning Chain" {
        id: "FC-002"
        description: "Driver warning notification path"
        functions: ["SF-005", "SF-008"]
        color: "#FFC000"
        priority: "High"
    }
}
