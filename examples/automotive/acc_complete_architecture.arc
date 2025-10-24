// Complete Adaptive Cruise Control System Architecture
// Capella-style with full component interactions, ports, and interfaces

operational_analysis "ACC Operational Context" {
    actor "Driver" {
        id: "ACT-001"
        description: "Vehicle driver who controls ACC system"
        concerns: ["Set desired speed", "Override control", "Monitor system status"]
    }
    
    actor "Lead Vehicle" {
        id: "ACT-002"
        description: "Vehicle being followed"
        concerns: ["Position", "Speed", "Relative distance"]
    }
    
    actor "Environment" {
        id: "ACT-003"
        description: "Road conditions and traffic"
        concerns: ["Weather", "Road geometry", "Traffic density"]
    }
}

system_analysis "ACC System Requirements" {
    requirement "SYS-ACC-001" {
        id: "SYS-ACC-001"
        description: "The ACC system shall maintain minimum 2-second following distance at all speeds"
        category: "Functional Safety"
        priority: "Critical"
        safety_level: "ASIL_B"
        verification_method: "Test"
        standard: "ISO 26262"
    }
    
    requirement "SYS-ACC-002" {
        id: "SYS-ACC-002"
        description: "The system shall detect cut-in vehicles within 500ms"
        category: "Performance"
        priority: "High"
        safety_level: "ASIL_B"
        verification_method: "Test"
        performance_metric: "Detection latency < 500ms"
    }
    
    requirement "SYS-ACC-003" {
        id: "SYS-ACC-003"
        description: "Maximum deceleration shall not exceed 3.5 m/s²"
        category: "Performance"
        priority: "High"
        safety_level: "ASIL_B"
        rationale: "Prevent passenger discomfort and maintain stability"
    }
    
    requirement "SYS-ACC-004" {
        id: "SYS-ACC-004"
        description: "Driver brake input shall immediately override ACC control"
        category: "Safety Override"
        priority: "Critical"
        safety_level: "ASIL_C"
        verification_method: "Test"
        response_time: "< 100ms"
    }
    
    requirement "SYS-ACC-005" {
        id: "SYS-ACC-005"
        description: "System shall operate in speed range 30-180 km/h"
        category: "Operational Range"
        priority: "Medium"
        safety_level: "ASIL_A"
    }
    
    requirement "SYS-ACC-006" {
        id: "SYS-ACC-006"
        description: "System shall provide visual and audible warnings"
        category: "Human Machine Interface"
        priority: "High"
        safety_level: "ASIL_A"
    }
    
    requirement "SYS-ACC-007" {
        id: "SYS-ACC-007"
        description: "System shall perform continuous self-diagnostics"
        category: "Diagnostic"
        priority: "Critical"
        safety_level: "ASIL_B"
    }
}

logical_architecture "ACC Logical Architecture" {
    // Sensor Layer
    component "Long Range Radar" {
        id: "LC-001"
        type: "Logical"
        category: "Perception"
        description: "77 GHz radar for forward vehicle detection"
        
        function "Transmit RF Signal" {
            id: "LF-001"
            description: "Generate and transmit radar pulses"
            power: "20dBm"
        }
        
        function "Receive Echoes" {
            id: "LF-002"
            description: "Capture reflected radar signals"
        }
        
        function "Process Radar Data" {
            id: "LF-003"
            description: "Extract target information from echoes"
        }
    }
    
    component "Forward Camera" {
        id: "LC-002"
        type: "Logical"
        category: "Perception"
        description: "Vision sensor for lane and object detection"
        
        function "Capture Image" {
            id: "LF-004"
            description: "Acquire camera frames"
            frame_rate: "30 Hz"
        }
        
        function "Detect Objects" {
            id: "LF-005"
            description: "Identify vehicles and obstacles"
        }
        
        function "Detect Lanes" {
            id: "LF-006"
            description: "Identify lane markings"
        }
    }
    
    // Fusion & Processing Layer
    component "Sensor Fusion" {
        id: "LC-003"
        type: "Logical"
        category: "Processing"
        description: "Combines radar and camera data for robust detection"
        
        function "Correlate Detections" {
            id: "LF-007"
            description: "Match radar and camera targets"
        }
        
        function "Track Objects" {
            id: "LF-008"
            description: "Maintain consistent object tracks over time"
        }
        
        function "Predict Trajectories" {
            id: "LF-009"
            description: "Estimate future positions of tracked objects"
        }
    }
    
    component "Target Selection" {
        id: "LC-004"
        type: "Logical"
        category: "Processing"
        description: "Determines which vehicle to follow"
        
        function "Identify Lead Vehicle" {
            id: "LF-010"
            description: "Select most relevant target in ego lane"
        }
        
        function "Detect Cut-In" {
            id: "LF-011"
            description: "Identify vehicles entering ego lane"
        }
    }
    
    // Control Layer
    component "Longitudinal Controller" {
        id: "LC-005"
        type: "Logical"
        category: "Control"
        description: "Calculates desired acceleration/deceleration"
        
        function "Calculate Time Gap" {
            id: "LF-012"
            description: "Compute time to lead vehicle"
        }
        
        function "Determine Desired Speed" {
            id: "LF-013"
            description: "Calculate target speed based on lead vehicle"
        }
        
        function "Compute Acceleration" {
            id: "LF-014"
            description: "Calculate required acceleration to achieve desired speed"
        }
    }
    
    component "Actuator Command" {
        id: "LC-006"
        type: "Logical"
        category: "Control"
        description: "Converts acceleration command to throttle/brake signals"
        
        function "Map to Throttle" {
            id: "LF-015"
            description: "Convert positive acceleration to throttle position"
        }
        
        function "Map to Brake" {
            id: "LF-016"
            description: "Convert negative acceleration to brake pressure"
        }
    }
    
    // Safety & Monitoring Layer
    component "Safety Monitor" {
        id: "LC-007"
        type: "Logical"
        category: "Safety"
        description: "Monitors system health and enforces safety limits"
        
        function "Check Sensor Health" {
            id: "LF-017"
            description: "Verify sensor data validity"
        }
        
        function "Enforce Limits" {
            id: "LF-018"
            description: "Limit acceleration/deceleration to safe values"
        }
        
        function "Detect Faults" {
            id: "LF-019"
            description: "Identify system malfunctions"
        }
    }
    
    component "Driver Interface" {
        id: "LC-008"
        type: "Logical"
        category: "HMI"
        description: "Handles driver inputs and provides feedback"
        
        function "Read Driver Inputs" {
            id: "LF-020"
            description: "Capture driver commands (set, resume, cancel)"
        }
        
        function "Display Status" {
            id: "LF-021"
            description: "Show ACC status on instrument cluster"
        }
        
        function "Issue Warnings" {
            id: "LF-022"
            description: "Alert driver of critical conditions"
        }
    }
    
    component "Override Manager" {
        id: "LC-009"
        type: "Logical"
        category: "Safety"
        description: "Handles driver override conditions"
        
        function "Detect Brake Pedal" {
            id: "LF-023"
            description: "Monitor brake pedal input"
        }
        
        function "Detect Accelerator Pedal" {
            id: "LF-024"
            description: "Monitor accelerator pedal input"
        }
        
        function "Override Control" {
            id: "LF-025"
            description: "Disable ACC when driver overrides"
        }
    }
}

// Traceability: Requirements to Components
trace "LC-001" satisfies "SYS-ACC-002" {
    rationale: "Radar provides primary detection for cut-in vehicles"
}

trace "LC-002" satisfies "SYS-ACC-002" {
    rationale: "Camera provides complementary detection for cut-in scenarios"
}

trace "LC-003" satisfies "SYS-ACC-001" {
    rationale: "Sensor fusion ensures robust target tracking for distance maintenance"
}

trace "LC-004" satisfies "SYS-ACC-001" {
    rationale: "Target selection identifies correct lead vehicle for following"
}

trace "LC-005" satisfies "SYS-ACC-001" {
    rationale: "Longitudinal controller maintains 2-second time gap"
}

trace "LC-005" satisfies "SYS-ACC-003" {
    rationale: "Controller enforces maximum deceleration limit"
}

trace "LC-006" satisfies "SYS-ACC-003" {
    rationale: "Actuator command limits brake pressure to prevent excessive deceleration"
}

trace "LC-007" satisfies "SYS-ACC-007" {
    rationale: "Safety monitor performs continuous diagnostics"
}

trace "LC-008" satisfies "SYS-ACC-006" {
    rationale: "Driver interface provides visual and audible warnings"
}

trace "LC-009" satisfies "SYS-ACC-004" {
    rationale: "Override manager detects brake input and disables ACC"
}

trace "LC-005" satisfies "SYS-ACC-005" {
    rationale: "Controller operates within specified speed range"
}

// Component to Component Data Flow
trace "LC-001" implements "LC-003" {
    rationale: "Radar provides target data to sensor fusion"
}

trace "LC-002" implements "LC-003" {
    rationale: "Camera provides object and lane data to sensor fusion"
}

trace "LC-003" implements "LC-004" {
    rationale: "Fused tracks feed target selection algorithm"
}

trace "LC-004" implements "LC-005" {
    rationale: "Selected lead vehicle drives longitudinal controller"
}

trace "LC-005" implements "LC-006" {
    rationale: "Acceleration command sent to actuator command mapper"
}

trace "LC-007" implements "LC-005" {
    rationale: "Safety monitor validates and limits controller outputs"
}

trace "LC-008" implements "LC-005" {
    rationale: "Driver commands affect controller behavior"
}

trace "LC-009" implements "LC-005" {
    rationale: "Override manager can disable controller"
}

trace "LC-009" implements "LC-006" {
    rationale: "Override manager can block actuator commands"
}

// Functional decomposition traces
trace "LF-001" implements "LF-002" {
    rationale: "Transmitted signal creates echoes that are received"
}

trace "LF-002" implements "LF-003" {
    rationale: "Raw echoes are processed to extract targets"
}

trace "LF-003" implements "LF-007" {
    rationale: "Radar targets feed into fusion algorithm"
}

trace "LF-005" implements "LF-007" {
    rationale: "Camera objects feed into fusion algorithm"
}

trace "LF-007" implements "LF-008" {
    rationale: "Correlated detections are tracked over time"
}

trace "LF-008" implements "LF-009" {
    rationale: "Current tracks used to predict future trajectories"
}

trace "LF-008" implements "LF-010" {
    rationale: "Tracked objects used for lead vehicle identification"
}

trace "LF-010" implements "LF-012" {
    rationale: "Lead vehicle used to calculate time gap"
}

trace "LF-012" implements "LF-013" {
    rationale: "Time gap influences desired speed calculation"
}

trace "LF-013" implements "LF-014" {
    rationale: "Desired speed drives acceleration computation"
}

trace "LF-014" implements "LF-015" {
    rationale: "Positive acceleration maps to throttle"
}

trace "LF-014" implements "LF-016" {
    rationale: "Negative acceleration maps to brake pressure"
}
