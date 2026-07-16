operational_analysis "Adaptive Cruise Control Operations" {
    actor "Driver" {
        id: "ACT-001"
        description: "Vehicle operator"
        interactions: ["set_cruise_speed", "monitor_distance", "override_control"]
    }
    
    actor "Vehicle" {
        id: "ACT-002"
        description: "Host vehicle being controlled"
    }
    
    actor "Target Vehicle" {
        id: "ACT-003"
        description: "Leading vehicle being tracked"
    }
    
    operational_capability "Maintain Safe Following Distance" {
        id: "OC-001"
        description: "Automatically adjust speed to maintain safe distance from lead vehicle"
    }
    
    operational_activity "Monitor Traffic" {
        id: "OA-001"
        description: "Continuously monitor vehicles ahead"
    }
    
    operational_activity "Adjust Speed" {
        id: "OA-002"
        description: "Increase or decrease vehicle speed as needed"
    }
}

system_analysis "ACC System" {
    requirement "SYS-ACC-001" {
        description: "The ACC system shall maintain minimum 2-second following distance at all speeds"
        priority: "Critical"
        safety_level: "ASIL_B"
        traces: ["OC-001"]
        verification_method: "Test"
    }
    
    requirement "SYS-ACC-002" {
        description: "The system shall detect cut-in vehicles within 500ms"
        priority: "High"
        safety_level: "ASIL_B"
    }
    
    requirement "SYS-ACC-003" {
        description: "Maximum deceleration shall not exceed 3.5 m/s²"
        priority: "High"
        safety_level: "ASIL_B"
    }
    
    requirement "SYS-ACC-004" {
        description: "Driver brake input shall immediately override ACC control"
        priority: "Critical"
        safety_level: "ASIL_C"
    }
    
    requirement "SYS-ACC-005" {
        description: "System shall operate in speed range 30-180 km/h"
        priority: "Medium"
        safety_level: "ASIL_A"
    }
    
    system_function "Sense Environment" {
        id: "SF-001"
        description: "Detect and track vehicles in front"
        safety_level: "ASIL_B"
    }
    
    system_function "Determine Target Speed" {
        id: "SF-002"
        description: "Calculate desired vehicle speed based on traffic"
        safety_level: "ASIL_B"
    }
    
    system_function "Control Vehicle Speed" {
        id: "SF-003"
        description: "Actuate throttle and brakes to achieve target speed"
        safety_level: "ASIL_C"
    }
}

logical_architecture "ACC Logical Architecture" {
    component "Radar Sensor" {
        id: "LC-001"
        component_type: "Logical"
        description: "77 GHz long-range radar"
        
        function "Transmit RF Signal" {
            id: "LF-001"
        }
        
        function "Process Echoes" {
            id: "LF-002"
        }
    }
    
    component "Vision Camera" {
        id: "LC-002"
        component_type: "Logical"
        description: "Forward-facing camera for object classification"
        
        function "Capture Images" {
            id: "LF-003"
        }
        
        function "Detect Objects" {
            id: "LF-004"
        }
    }
    
    component "Sensor Fusion" {
        id: "LC-003"
        component_type: "Logical"
        description: "Combine radar and camera data"
        safety_level: "ASIL_B"
        
        function "Fuse Detections" {
            id: "LF-005"
        }
        
        function "Select Target" {
            id: "LF-006"
        }
    }
    
    component "ACC Controller" {
        id: "LC-004"
        component_type: "Logical"
        description: "Main ACC control logic"
        safety_level: "ASIL_B"
        
        function "Calculate Following Distance" {
            id: "LF-007"
            formula: "time_gap * ego_speed + minimum_distance"
        }
        
        function "Compute Speed Profile" {
            id: "LF-008"
        }
    }
    
    component "Actuator Interface" {
        id: "LC-005"
        component_type: "Logical"
        description: "Interface to vehicle actuators"
        safety_level: "ASIL_C"
        
        function "Convert to CAN Commands" {
            id: "LF-009"
        }
        
        function "Monitor Driver Override" {
            id: "LF-010"
            sat: "SYS-ACC-004"
        }
    }
    
    interface "Radar Data Bus" {
        id: "LI-001"
        from: "LC-001"
        to: "LC-003"
        interface_type: "Data"
    }
    
    interface "Camera Data Bus" {
        id: "LI-002"
        from: "LC-002"
        to: "LC-003"
        interface_type: "Data"
    }
    
    interface "Control Bus" {
        id: "LI-003"
        from: "LC-004"
        to: "LC-005"
        interface_type: "Data"
    }
}

physical_architecture "ACC Physical Architecture" {
    node "Radar ECU" {
        id: "PN-001"
        processor: "Aurix TC397"
        
        deploys "LC-001"
    }
    
    node "Camera ECU" {
        id: "PN-002"
        processor: "EyeQ5 SoC"
        
        deploys "LC-002"
    }
    
    node "ADAS ECU" {
        id: "PN-003"
        processor: "NVIDIA Xavier"
        
        deploys "LC-003"
        deploys "LC-004"
    }
    
    node "Gateway ECU" {
        id: "PN-004"
        
        deploys "LC-005"
    }
    
    physical_link "CAN FD Bus" {
        id: "PL-001"
        topology: "CAN FD"
        bandwidth: "2Mbps"
        connects: ["PN-001", "PN-003"]
        realizes: ["LI-001"]
    }
    
    physical_link "Automotive Ethernet" {
        id: "PL-002"
        topology: "100BASE-T1"
        bandwidth: "100Mbps"
        connects: ["PN-002", "PN-003"]
        realizes: ["LI-002"]
    }
    
    physical_link "Vehicle CAN" {
        id: "PL-003"
        topology: "CAN"
        bandwidth: "500kbps"
        connects: ["PN-003", "PN-004", "Engine ECU", "Brake ECU"]
        realizes: ["LI-003"]
    }
}

epbs "ACC EPBS" {
    system "Adaptive Cruise Control System" {
        id: "EPBS-001"
        
        subsystem "Sensing Subsystem" {
            id: "EPBS-101"
            
            item "Continental ARS540 Radar" {
                id: "EPBS-1001"
                part_number: "ARS540-2022"
            }
            
            item "Mobileye EyeQ5 Camera" {
                id: "EPBS-1002"
                part_number: "EQ5-2021"
            }
        }
        
        subsystem "Processing Subsystem" {
            id: "EPBS-102"
            
            item "ADAS Domain Controller" {
                id: "EPBS-1003"
                part_number: "ADAS-CTRL-001"
            }
            
            item "ACC Software Stack" {
                id: "EPBS-1004"
                part_number: "SW-ACC-V2.5.0"
            }
        }
        
        subsystem "Communication Subsystem" {
            id: "EPBS-103"
            
            item "Central Gateway" {
                id: "EPBS-1005"
                part_number: "CGW-001"
            }
            
            item "CAN FD Transceiver" {
                id: "EPBS-1006"
                part_number: "TJA1463"
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
    
    hazard "Loss of Target Tracking" {
        id: "HAZ-002"
        description: "System loses track of lead vehicle"
        severity: "S2"
        asil: "ASIL_B"
    }
    
    fmea "ACC FMEA" {
        target: "Sensor Fusion"
        failure_mode: "False target detection"
        effects: "Unnecessary braking or acceleration"
        severity: "S2"
        rpn: 36
    }
}

trace "LC-003" satisfies "SYS-ACC-001" {
    rationale: "Sensor fusion provides target detection for distance control"
}

trace "LF-006" satisfies "SYS-ACC-002" {
    rationale: "Target selection detects cut-in vehicles"
}
