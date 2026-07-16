operational_analysis "Emergency Braking Operational Context" {
    actor "Driver" {
        id: "OA-ACT-001"
        description: "Human operator controlling the vehicle"
        category: "Human"
    }
    
    actor "Vehicle System" {
        id: "OA-ACT-002"
        description: "Autonomous emergency braking system"
        category: "System"
    }
    
    actor "Road Environment" {
        id: "OA-ACT-003"
        description: "Physical environment with obstacles"
        category: "External"
    }
    
    actor "Other Vehicles" {
        id: "OA-ACT-004"
        description: "Surrounding traffic"
        category: "External"
    }
    
    actor "Pedestrians" {
        id: "OA-ACT-005"
        description: "Vulnerable road users"
        category: "External"
    }
    
    operational_activity "Monitor Environment" {
        id: "OA-01"
        description: "Driver monitors road and traffic continuously"
        performed_by: "OA-ACT-001"
    }
    
    operational_activity "Control Vehicle" {
        id: "OA-02"
        description: "Driver controls speed and braking"
        performed_by: "OA-ACT-001"
    }
    
    operational_activity "Detect Collision Risk" {
        id: "OA-03"
        description: "System detects potential collisions"
        performed_by: "OA-ACT-002"
    }
    
    operational_activity "Alert Driver" {
        id: "OA-04"
        description: "System provides collision warnings"
        performed_by: "OA-ACT-002"
    }
    
    operational_activity "Apply Emergency Brake" {
        id: "OA-05"
        description: "System automatically brakes"
        performed_by: "OA-ACT-002"
    }
    
    operational_activity "Present Hazards" {
        id: "OA-06"
        description: "Environment creates obstacles"
        performed_by: "OA-ACT-003"
    }
    
    operational_activity "Navigate Traffic" {
        id: "OA-07"
        description: "Other vehicles move dynamically"
        performed_by: "OA-ACT-004"
    }
    
    operational_activity "Cross Road" {
        id: "OA-08"
        description: "Pedestrians enter vehicle path"
        performed_by: "OA-ACT-005"
    }
    
    // Operational Exchanges (Actor <-> System interactions)
    interaction "OA-01" -> "OA-03" {
        label: "Environment Data"
        description: "Driver observation informs system detection"
    }
    
    interaction "OA-03" -> "OA-04" {
        label: "Collision Warning"
        description: "System alerts driver of risk"
    }
    
    interaction "OA-04" -> "OA-02" {
        label: "Driver Response"
        description: "Driver reacts to warning"
    }
    
    interaction "OA-03" -> "OA-05" {
        label: "Auto Brake Trigger"
        description: "System initiates emergency braking"
    }
    
    interaction "OA-06" -> "OA-03" {
        label: "Hazard Detection"
        description: "Environment hazards detected by system"
    }
    
    interaction "OA-07" -> "OA-03" {
        label: "Traffic Info"
        description: "Other vehicle movements detected"
    }
    
    interaction "OA-08" -> "OA-03" {
        label: "Pedestrian Alert"
        description: "Pedestrian crossing detected"
    }
}
