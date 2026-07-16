system_analysis "Emergency Braking Requirements" {
    requirement "Obstacle Detection" {
        id: "SYS-001"
        description: "System shall detect obstacles up to 150m ahead"
        priority: "Critical"
        safety_level: "ASIL_D"
        verification_method: "Test"
    }
    
    requirement "Response Time" {
        id: "SYS-002"
        description: "System shall initiate braking within 100ms of obstacle detection"
        priority: "Critical"
        safety_level: "ASIL_D"
        verification_method: "Test"
    }
    
    requirement "False Positive Rate" {
        id: "SYS-003"
        description: "False positive rate shall be less than 1 in 10^9 operating hours"
        priority: "Critical"
        safety_level: "ASIL_D"
        verification_method: "Analysis"
    }
    
    requirement "Sensor Fusion" {
        id: "SYS-004"
        description: "System shall fuse data from at least 3 sensor types"
        priority: "High"
        safety_level: "ASIL_C"
        verification_method: "Test"
    }
    
    requirement "Degraded Operation" {
        id: "SYS-005"
        description: "System shall maintain basic functionality with one sensor failure"
        priority: "High"
        safety_level: "ASIL_C"
        verification_method: "Test"
    }
    
    requirement "Driver Override" {
        id: "SYS-006"
        description: "Driver shall be able to override system at any time"
        priority: "High"
        safety_level: "ASIL_B"
        verification_method: "Test"
    }
    
    requirement "System Status" {
        id: "SYS-007"
        description: "System shall continuously monitor and report its status"
        priority: "Medium"
        safety_level: "ASIL_B"
        verification_method: "Review"
    }
}

logical_architecture "Emergency Braking Architecture" {
    component "Main Controller" {
        id: "LC-001"
        type: "Logical"
        description: "Central control unit"
        safety_level: "ASIL_D"
        
        function "Manage System" {
            id: "LF-001"
            description: "Overall system management"
        }
        
        function "Monitor Health" {
            id: "LF-002"
            description: "System health monitoring"
        }
        
        component "Supervisor" {
            id: "LC-001-01"
            type: "Logical"
            description: "Supervision logic"
            safety_level: "ASIL_D"
            
            function "Check Safety" {
                id: "LF-003"
                description: "Safety checking"
            }
        }
    }
    
    component "Sensor Fusion Unit" {
        id: "LC-002"
        type: "Logical"
        description: "Multi-sensor fusion processor"
        safety_level: "ASIL_C"
        
        function "Fuse Data" {
            id: "LF-004"
            description: "Sensor data fusion"
        }
        
        component "Data Validator" {
            id: "LC-002-01"
            type: "Logical"
            description: "Data validation logic"
            
            function "Validate Input" {
                id: "LF-005"
                description: "Input validation"
            }
        }
    }
    
    component "Radar Interface" {
        id: "LC-003"
        type: "Logical"
        description: "Radar sensor interface"
        safety_level: "ASIL_C"
        
        function "Process Radar" {
            id: "LF-006"
            description: "Radar data processing"
        }
    }
    
    component "Camera Interface" {
        id: "LC-004"
        type: "Logical"
        description: "Camera sensor interface"
        safety_level: "ASIL_C"
        
        function "Process Image" {
            id: "LF-007"
            description: "Image processing"
        }
    }
    
    component "Lidar Interface" {
        id: "LC-005"
        type: "Logical"
        description: "Lidar sensor interface"
        safety_level: "ASIL_C"
        
        function "Process Lidar" {
            id: "LF-008"
            description: "Lidar data processing"
        }
    }
    
    component "Object Tracker" {
        id: "LC-006"
        type: "Logical"
        description: "Object tracking unit"
        safety_level: "ASIL_C"
        
        function "Track Objects" {
            id: "LF-009"
            description: "Object tracking"
        }
        
        component "Trajectory Predictor" {
            id: "LC-006-01"
            type: "Logical"
            description: "Trajectory prediction"
            
            function "Predict Motion" {
                id: "LF-010"
                description: "Motion prediction"
            }
        }
    }
    
    component "Risk Assessor" {
        id: "LC-007"
        type: "Logical"
        description: "Risk assessment unit"
        safety_level: "ASIL_D"
        
        function "Assess Risk" {
            id: "LF-011"
            description: "Risk assessment"
        }
    }
    
    component "Brake Controller" {
        id: "LC-008"
        type: "Logical"
        description: "Brake control unit"
        safety_level: "ASIL_D"
        
        function "Control Brakes" {
            id: "LF-012"
            description: "Brake actuation"
        }
    }
    
    connection "LC-003" -> "LC-002" {
        description: "Radar data to fusion"
    }
    
    connection "LC-004" -> "LC-002" {
        description: "Camera data to fusion"
    }
    
    connection "LC-005" -> "LC-002" {
        description: "Lidar data to fusion"
    }
    
    connection "LC-002" -> "LC-006" {
        description: "Fused data to tracker"
    }
    
    connection "LC-006" -> "LC-007" {
        description: "Tracking to risk assessment"
    }
    
    connection "LC-007" -> "LC-008" {
        description: "Risk assessment to brake control"
    }
    
    connection "LC-001" -> "LC-008" {
        description: "Main control to brake actuation"
    }
}

trace "LC-001" satisfies "SYS-001" { rationale: "Main controller implements detection requirement" }
trace "LC-002" satisfies "SYS-004" { rationale: "Fusion unit implements sensor fusion requirement" }
trace "LC-007" satisfies "SYS-002" { rationale: "Risk assessor implements timing requirement" }
trace "LC-008" satisfies "SYS-006" { rationale: "Brake controller implements override requirement" }
trace "LC-001" satisfies "SYS-007" { rationale: "Main controller implements monitoring requirement" }
