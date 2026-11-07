operational_analysis "Emergency Braking Operational Context" {
    actor "Driver" {
        id: "OA-ACT-001"
        description: "Human vehicle operator"
    }
    
    actor "Vehicle System" {
        id: "OA-ACT-002"
        description: "Emergency braking system"
        safety_level: "ASIL_D"
    }
    
    operational_activity "Monitor Environment" {
        id: "OA-01"
        description: "Driver monitors road and traffic"
        performed_by: "OA-ACT-001"
    }
    
    operational_activity "Detect Collision Risk" {
        id: "OA-02"
        description: "System detects collision threats"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_D"
    }
    
    operational_activity "Apply Emergency Brake" {
        id: "OA-03"
        description: "System applies automatic braking"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_D"
    }
}

system_analysis "Emergency Braking Requirements" {
    requirement "STK-001" {
        description: "The system shall prevent rear-end collisions"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
    
    requirement "SYS-001" {
        description: "Detect obstacles at 1-200m range"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
    
    requirement "SYS-002" {
        description: "Initiate braking within 200ms"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
}

logical_architecture "Emergency Braking Logical Architecture" {
    component "Sensor Fusion Controller" {
        id: "LC-001"
        type: "Logical"
        description: "Multi-sensor data fusion"
        
        function "Fuse Sensor Data" {
            id: "LF-001"
        }
        
        function "Assess Collision Risk" {
            id: "LF-002"
        }
    }
    
    component "Braking Decision Controller" {
        id: "LC-002"
        type: "Logical"
        description: "Braking strategy decision"
        
        function "Decide Braking" {
            id: "LF-003"
        }
    }
    
    component "Brake Actuator Controller" {
        id: "LC-003"
        type: "Logical"
        description: "Brake control unit"
        
        function "Actuate Brakes" {
            id: "LF-004"
        }
    }
}

physical_architecture "Emergency Braking Physical Architecture" {
    node "Emergency Brake ECU" {
        id: "PA-001"
        processor: "Renesas RH850"
        
        deploys "LC-001"
        deploys "LC-002"
    }
    
    node "Brake Actuator ECU" {
        id: "PA-002"
        processor: "Infineon AURIX"
        
        deploys "LC-003"
    }
}

epbs "Emergency Braking Product Breakdown" {
    system "Emergency Braking System" {
        id: "EPBS-001"
        
        subsystem "Sensor Suite" {
            id: "SS-001"
            
            item "Radar Unit" {
                id: "ITEM-001"
                part_number: "RADAR-77GHZ"
            }
            
            item "Camera Unit" {
                id: "ITEM-002"
                part_number: "CAM-MONO"
            }
        }
        
        subsystem "Control Subsystem" {
            id: "SS-002"
            
            item "Main ECU" {
                id: "ITEM-003"
                part_number: "ECU-MAIN"
            }
        }
    }
}

safety_analysis {
    hazard "Failed to Brake" {
        id: "HAZ-001"
        description: "System fails to brake"
        severity: "S3"
        asil: "ASIL_D"
    }
    
    fmea "Sensor Fusion FMEA" {
        target: "Sensor Fusion"
        failure_mode: "False detection"
        effects: "Unnecessary braking"
        severity: "S2"
        rpn: 12
    }
}

trace "LC-001" satisfies "SYS-001" {
    rationale: "Sensor Fusion provides detection"
}

trace "LC-002" satisfies "SYS-002" {
    rationale: "Decision Controller meets timing"
}
