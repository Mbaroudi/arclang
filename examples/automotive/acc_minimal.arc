operational_analysis "Adaptive Cruise Control Operations" {
    actor "Driver" {
        id: "ACT-001"
        description: "Vehicle operator"
    }
}

system_analysis "ACC System" {
    requirement "SYS-ACC-001" {
        description: "The ACC system shall maintain minimum 2-second following distance"
        priority: "Critical"
        safety_level: "ASIL_B"
    }
    
    requirement "SYS-ACC-002" {
        description: "Driver brake input shall override ACC"
        priority: "Critical"
        safety_level: "ASIL_C"
    }
    
    requirement "SYS-ACC-003" {
        description: "System shall detect cut-in vehicles within 500ms"
        priority: "High"
        safety_level: "ASIL_B"
    }
}

logical_architecture "ACC Logical Architecture" {
    component "Radar Sensor" {
        id: "LC-001"
        type: "Logical"
        description: "77 GHz long-range radar"
        
        function "Detect Objects" {
            id: "LF-001"
        }
    }
    
    component "Vision Camera" {
        id: "LC-002"
        type: "Logical"
        description: "Forward-facing camera"
        
        function "Classify Objects" {
            id: "LF-003"
        }
    }
    
    component "Sensor Fusion" {
        id: "LC-003"
        type: "Logical"
        safety_level: "ASIL_B"
        
        function "Fuse Detections" {
            id: "LF-005"
        }
    }
    
    component "ACC Controller" {
        id: "LC-004"
        type: "Logical"
        safety_level: "ASIL_B"
        
        function "Compute Speed Profile" {
            id: "LF-008"
        }
    }
}

physical_architecture "ACC Physical Architecture" {
    node "Radar ECU" {
        id: "PN-001"
        processor: "Infineon AURIX TC397"
        
        deploys "LC-001"
    }
    
    node "Camera ECU" {
        id: "PN-002"
        processor: "Mobileye EyeQ5"
        
        deploys "LC-002"
    }
    
    node "ADAS ECU" {
        id: "PN-003"
        processor: "NVIDIA Xavier"
        
        deploys "LC-003"
        deploys "LC-004"
    }
}

epbs "ACC Product Breakdown" {
    system "Adaptive Cruise Control" {
        id: "SYS-001"
        
        subsystem "Sensing Subsystem" {
            id: "SS-001"
            
            item "Radar Sensor Unit" {
                id: "ITEM-001"
                part_number: "77GHZ-RADAR-001"
            }
            
            item "Camera Module" {
                id: "ITEM-002"
                part_number: "CAM-FWD-001"
            }
        }
        
        subsystem "Control Subsystem" {
            id: "SS-002"
            
            item "ADAS Controller" {
                id: "ITEM-003"
                part_number: "ADAS-ECU-001"
            }
        }
    }
}

safety_analysis {
    hazard "Unintended Acceleration" {
        id: "HAZ-001"
        description: "Vehicle accelerates when it should decelerate"
        severity: "S3"
        asil: "ASIL_C"
    }
    
    fmea "Sensor Fusion FMEA" {
        target: "Sensor Fusion"
        failure_mode: "False target detection"
        effects: "Unnecessary braking"
        severity: "S2"
        rpn: 36
    }
}

trace "LC-001" satisfies "SYS-ACC-001" {
    rationale: "Radar provides target detection for distance control"
}

trace "LC-003" satisfies "SYS-ACC-002" {
    rationale: "Sensor fusion enables reliable tracking"
}

trace "LC-004" satisfies "SYS-ACC-003" {
    rationale: "Controller computes safe following profile"
}
