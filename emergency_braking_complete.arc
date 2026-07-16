operational_analysis "Emergency Braking System Context" {
    actor "Driver" {
        id: "ACT-001"
        description: "Vehicle operator responsible for overall vehicle control"
        category: "Human"
    }
    
    actor "Vehicle" {
        id: "ACT-002"
        description: "Host vehicle system"
        category: "System"
    }
    
    actor "Environment" {
        id: "ACT-003"
        description: "External environment including road conditions and obstacles"
        category: "External"
    }
    
    actor "Other Vehicles" {
        id: "ACT-004"
        description: "Surrounding traffic participants"
        category: "External"
    }
    
    operational_activity "Monitor Traffic" {
        id: "OA-001"
        description: "Driver monitors surrounding traffic conditions"
        performed_by: "ACT-001"
    }
    
    operational_activity "Control Vehicle" {
        id: "OA-002"
        description: "Driver controls vehicle acceleration and braking"
        performed_by: "ACT-001"
    }
    
    operational_activity "Detect Obstacles" {
        id: "OA-003"
        description: "Vehicle sensors detect obstacles and hazards"
        performed_by: "ACT-002"
    }
    
    operational_activity "Present Road Conditions" {
        id: "OA-004"
        description: "Environment presents various road conditions"
        performed_by: "ACT-003"
    }
    
    operational_activity "Generate Traffic Situation" {
        id: "OA-005"
        description: "Other vehicles create dynamic traffic situations"
        performed_by: "ACT-004"
    }
}

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

    system_function "Acquire Sensor Data" {
        id: "SF-001"
        description: "Collect and validate sensor inputs"
        category: "Input"
        
        function "Read Radar" {
            id: "SF-001-01"
            description: "Acquire radar sensor data"
        }
        
        function "Read Camera" {
            id: "SF-001-02"
            description: "Acquire camera image data"
        }
        
        function "Read Lidar" {
            id: "SF-001-03"
            description: "Acquire lidar point cloud data"
        }
    }
    
    system_function "Process Sensor Data" {
        id: "SF-002"
        description: "Process and fuse sensor data"
        category: "Processing"
        
        function "Validate Data" {
            id: "SF-002-01"
            description: "Validate sensor data quality"
        }
        
        function "Align Data" {
            id: "SF-002-02"
            description: "Temporal and spatial alignment"
        }
        
        function "Fuse Data" {
            id: "SF-002-03"
            description: "Multi-sensor data fusion"
        }
    }
    
    system_function "Detect Objects" {
        id: "SF-003"
        description: "Detect and classify objects"
        category: "Analysis"
        
        function "Segment Objects" {
            id: "SF-003-01"
            description: "Segment detected objects"
        }
        
        function "Classify Objects" {
            id: "SF-003-02"
            description: "Classify object types"
        }
    }
    
    system_function "Track Objects" {
        id: "SF-004"
        description: "Track object trajectories"
        category: "Analysis"
        
        function "Update Tracks" {
            id: "SF-004-01"
            description: "Update object tracking"
        }
        
        function "Predict Motion" {
            id: "SF-004-02"
            description: "Predict object trajectories"
        }
    }
    
    system_function "Assess Situation" {
        id: "SF-005"
        description: "Assess collision risk"
        category: "Decision"
        
        function "Calculate TTC" {
            id: "SF-005-01"
            description: "Calculate time-to-collision"
        }
        
        function "Evaluate Risk" {
            id: "SF-005-02"
            description: "Evaluate collision risk"
        }
    }
    
    system_function "Plan Response" {
        id: "SF-006"
        description: "Plan braking response"
        category: "Planning"
        
        function "Select Strategy" {
            id: "SF-006-01"
            description: "Select braking strategy"
        }
        
        function "Calculate Profile" {
            id: "SF-006-02"
            description: "Calculate braking profile"
        }
    }
    
    system_function "Execute Response" {
        id: "SF-007"
        description: "Execute braking command"
        category: "Output"
        
        function "Command Brakes" {
            id: "SF-007-01"
            description: "Send brake commands"
        }
        
        function "Monitor Execution" {
            id: "SF-007-02"
            description: "Monitor command execution"
        }
    }
    
    system_function "Monitor System" {
        id: "SF-008"
        description: "Monitor system health"
        category: "Monitoring"
        
        function "Check Status" {
            id: "SF-008-01"
            description: "Check component status"
        }
        
        function "Handle Faults" {
            id: "SF-008-02"
            description: "Handle fault conditions"
        }
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
}

trace "ACT-001" implements "ACT-002" { rationale: "Driver controls vehicle" }
trace "ACT-002" implements "ACT-003" { rationale: "Vehicle interacts with environment" }
trace "ACT-002" implements "ACT-004" { rationale: "Vehicle interacts with other vehicles" }

trace "OA-001" implements "OA-002" { rationale: "Monitoring leads to control" }
trace "OA-003" implements "OA-004" { rationale: "Detection influenced by conditions" }
trace "OA-003" implements "OA-005" { rationale: "Detection includes other vehicles" }

trace "SF-001" implements "SF-002" { rationale: "Data acquisition to processing" }
trace "SF-002" implements "SF-003" { rationale: "Processing to detection" }
trace "SF-003" implements "SF-004" { rationale: "Detection to tracking" }
trace "SF-004" implements "SF-005" { rationale: "Tracking to assessment" }
trace "SF-005" implements "SF-006" { rationale: "Assessment to planning" }
trace "SF-006" implements "SF-007" { rationale: "Planning to execution" }
trace "SF-007" implements "SF-008" { rationale: "Execution monitoring" }

trace "LC-003" implements "LC-002" { rationale: "Radar data to fusion" }
trace "LC-004" implements "LC-002" { rationale: "Camera data to fusion" }
trace "LC-005" implements "LC-002" { rationale: "Lidar data to fusion" }
trace "LC-002" implements "LC-006" { rationale: "Fused data to tracker" }
trace "LC-006" implements "LC-007" { rationale: "Tracking to risk assessment" }
trace "LC-007" implements "LC-008" { rationale: "Risk assessment to brake control" }
trace "LC-001" implements "LC-008" { rationale: "Main control to brake actuation" }

trace "LF-006" implements "LF-004" { rationale: "Radar to fusion" }
trace "LF-007" implements "LF-004" { rationale: "Camera to fusion" }
trace "LF-008" implements "LF-004" { rationale: "Lidar to fusion" }
trace "LF-004" implements "LF-009" { rationale: "Fusion to tracking" }
trace "LF-009" implements "LF-011" { rationale: "Tracking to risk" }
trace "LF-011" implements "LF-012" { rationale: "Risk to braking" }

trace "LC-001" satisfies "SYS-001" { rationale: "Main controller implements detection requirement" }
trace "LC-002" satisfies "SYS-004" { rationale: "Fusion unit implements sensor fusion requirement" }
trace "LC-007" satisfies "SYS-002" { rationale: "Risk assessor implements timing requirement" }
trace "LC-008" satisfies "SYS-006" { rationale: "Brake controller implements override requirement" }
trace "LC-001" satisfies "SYS-007" { rationale: "Main controller implements monitoring requirement" }

trace "LF-004" satisfies "SF-002" { rationale: "Fusion function implements system fusion" }
trace "LF-009" satisfies "SF-004" { rationale: "Tracking function implements system tracking" }
trace "LF-011" satisfies "SF-005" { rationale: "Risk function implements system assessment" }
trace "LF-012" satisfies "SF-007" { rationale: "Brake function implements system response" }
trace "LF-002" satisfies "SF-008" { rationale: "Monitor function implements system monitoring" }
