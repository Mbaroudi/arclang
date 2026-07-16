// Emergency Braking System - Correct Working Syntax
// Compatible with current ArcLang parser for all diagram types

operational_analysis "Emergency Braking Operational Context" {
    
    actor "Driver" {
        id: "OA-ACT-001"
        description: "Human operator controlling the vehicle"
        category: "Human"
    }
    
    actor "Vehicle System" {
        id: "OA-ACT-002"
        description: "Autonomous emergency braking system with sensor fusion"
        category: "System"
        safety_level: "ASIL_D"
    }
    
    actor "Leading Vehicle" {
        id: "OA-ACT-003"
        description: "Vehicle ahead in same lane"
        category: "External"
    }
    
    actor "Pedestrian" {
        id: "OA-ACT-004"
        description: "Vulnerable road user crossing vehicle path"
        category: "External"
    }
    
    actor "Road Environment" {
        id: "OA-ACT-005"
        description: "Physical environment with obstacles and weather"
        category: "External"
    }
    
    operational_activity "Monitor Environment" {
        id: "OA-01"
        description: "Driver monitors road and traffic continuously"
        performed_by: "OA-ACT-001"
    }
    
    operational_activity "Control Vehicle" {
        id: "OA-02"
        description: "Driver controls acceleration and braking"
        performed_by: "OA-ACT-001"
    }
    
    operational_activity "Scan Environment" {
        id: "OA-03"
        description: "System scans 360 degrees using multiple sensors"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_C"
    }
    
    operational_activity "Detect Collision Risk" {
        id: "OA-04"
        description: "System identifies objects and calculates collision probability"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_D"
    }
    
    operational_activity "Warn Driver" {
        id: "OA-05"
        description: "System provides visual and audible collision warnings"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_B"
    }
    
    operational_activity "Apply Emergency Brake" {
        id: "OA-06"
        description: "System automatically applies maximum safe braking force"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_D"
    }
    
    operational_activity "Drive Ahead" {
        id: "OA-07"
        description: "Leading vehicle travels ahead, may brake suddenly"
        performed_by: "OA-ACT-003"
    }
    
    operational_activity "Cross Road" {
        id: "OA-08"
        description: "Pedestrian enters roadway in front of vehicle"
        performed_by: "OA-ACT-004"
    }
    
    operational_activity "Present Hazards" {
        id: "OA-09"
        description: "Environment presents obstacles, poor visibility, slippery roads"
        performed_by: "OA-ACT-005"
    }
}

system_analysis "Emergency Braking System Requirements" {
    
    requirement "STK-001" {
        description: "The system shall prevent rear-end collisions in all driving conditions"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
    
    requirement "STK-002" {
        description: "The system shall alert driver before automatic braking"
        priority: "High"
        safety_level: "ASIL_B"
    }
    
    requirement "SYS-001" {
        description: "Detect obstacles at 1-200m range with accuracy ≤ 0.5m"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
    
    requirement "SYS-002" {
        description: "Initiate braking within 200ms of threat detection"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
    
    requirement "SYS-003" {
        description: "Fuse data from at least 2 independent sensors (radar + camera minimum)"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
    
    requirement "SYS-004" {
        description: "Modulate braking force from 0 to 100 percent in 10 percent increments"
        priority: "High"
        safety_level: "ASIL_C"
    }
    
    requirement "SAFE-001" {
        description: "Upon sensor failure, system shall enter safe state with degraded capability"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
}

logical_architecture "Emergency Braking Logical Architecture" {
    
    component "Sensor Fusion Controller" {
        id: "LC-001"
        type: "Logical"
        description: "Multi-sensor data fusion processor with Extended Kalman Filter"
        safety_level: "ASIL_D"
        
        function "Acquire Radar Data" {
            id: "LF-001"
            description: "Process 77GHz radar returns"
        }
        
        function "Acquire Camera Data" {
            id: "LF-002"
            description: "Process camera images with CNN"
        }
        
        function "Acquire Lidar Data" {
            id: "LF-003"
            description: "Process lidar point cloud"
        }
        
        function "Fuse Sensor Data" {
            id: "LF-004"
            description: "Combine multi-sensor data into unified environment model"
        }
        
        function "Assess Collision Risk" {
            id: "LF-005"
            description: "Calculate time-to-collision and threat level"
        }
        
        provides interface "IEnvironmentModel" {
            signals: [
                "fusedObjects: Array",
                "threatLevel: Enum",
                "timeToCollision: Real",
                "confidence: Real"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "IRadarData" {
            signals: ["radarObjects", "radarStatus"]
            protocol: "CAN FD"
        }
        
        requires interface "ICameraData" {
            signals: ["cameraObjects", "cameraStatus"]
            protocol: "Automotive Ethernet"
        }
        
        requires interface "ILidarData" {
            signals: ["lidarObjects", "lidarStatus"]
            protocol: "Automotive Ethernet"
        }
    }
    
    component "Braking Decision Controller" {
        id: "LC-002"
        type: "Logical"
        description: "Braking strategy decision logic with safety validation"
        safety_level: "ASIL_D"
        
        function "Decide Braking Strategy" {
            id: "LF-006"
            description: "Determine optimal braking force and timing"
        }
        
        provides interface "IBrakingCommand" {
            signals: [
                "brakingForce: Real",
                "brakingMode: Enum",
                "emergencyBrake: Boolean"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["threatLevel", "timeToCollision"]
            protocol: "CAN FD"
        }
        
        requires interface "IDriverInput" {
            signals: ["driverOverride: Boolean"]
            protocol: "CAN"
        }
    }
    
    component "Warning Interface" {
        id: "LC-003"
        type: "Logical"
        description: "Driver warning system with visual and audible alerts"
        safety_level: "ASIL_B"
        
        function "Generate Driver Warning" {
            id: "LF-007"
            description: "Create visual and audible warnings"
        }
        
        provides interface "IWarning" {
            signals: [
                "visualAlert: AlertLevel",
                "audioAlert: AudioPattern",
                "hapticsAlert: VibrationPattern"
            ]
            protocol: "LIN"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["threatLevel", "timeToCollision"]
            protocol: "CAN FD"
        }
    }
    
    component "Brake Actuator Controller" {
        id: "LC-004"
        type: "Logical"
        description: "Hydraulic brake-by-wire control unit"
        safety_level: "ASIL_D"
        
        function "Actuate Brakes" {
            id: "LF-008"
            description: "Control hydraulic brake actuators with ABS integration"
        }
        
        provides interface "IBrakeStatus" {
            signals: [
                "actualBrakeForce: Real",
                "brakePressure: Real",
                "actuatorHealth: HealthStatus"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "IBrakingCommand" {
            signals: ["brakingForce", "brakingMode"]
            protocol: "CAN FD"
        }
    }
    
    component "Health Monitor" {
        id: "LC-005"
        type: "Logical"
        description: "System health monitoring and diagnostics"
        safety_level: "ASIL_D"
        
        function "Monitor System Health" {
            id: "LF-009"
            description: "Diagnose sensor and actuator health, trigger degradation"
        }
        
        provides interface "ISystemHealth" {
            signals: [
                "systemMode: SystemMode",
                "faultCodes: Array",
                "degradationLevel: DegradationLevel"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "ISensorHealth" {
            signals: ["radarHealth", "cameraHealth", "lidarHealth"]
            protocol: "CAN FD"
        }
    }
    
    component "Event Recorder" {
        id: "LC-006"
        type: "Logical"
        description: "Safety event logging (black box)"
        safety_level: "ASIL_C"
        
        function "Log Safety Events" {
            id: "LF-010"
            description: "Record all emergency braking events"
        }
        
        provides interface "IEventLog" {
            signals: ["logEntries: Array"]
            protocol: "Diagnostic"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["threatLevel", "timeToCollision"]
            protocol: "CAN FD"
        }
        
        requires interface "IBrakingCommand" {
            signals: ["brakingForce"]
            protocol: "CAN FD"
        }
    }
    
}

physical_architecture "Emergency Braking Physical Architecture" {
    
    node "Emergency Brake ECU" {
        id: "PA-001"
        type: "ECU"
        description: "Central processing unit running sensor fusion and decision logic"
        processor: "Renesas RH850 F1KM-R7F7016643"
        cores: "Dual-core lockstep @ 320MHz"
        memory: "8MB Flash + 1MB RAM with ECC"
        power: "15W nominal, 20W peak"
        operating_temp: "-40°C to +125°C"
        safety_level: "ASIL_D"
        
        deploys "LC-001"
        deploys "LC-002"
        deploys "LC-005"
        deploys "LC-006"
    }
    
    node "Radar ECU" {
        id: "PA-002"
        type: "ECU"
        description: "77GHz radar signal processing"
        processor: "Infineon AURIX TC39x"
        memory: "4MB Flash + 512KB RAM"
        power: "8W"
        operating_temp: "-40°C to +85°C"
        safety_level: "ASIL_B"
    }
    
    node "Camera ECU" {
        id: "PA-003"
        type: "ECU"
        description: "Vision processing with Mobileye EyeQ5"
        processor: "Mobileye EyeQ5 + ARM Cortex-A76"
        memory: "8GB LPDDR4 + 16GB eMMC"
        power: "12W"
        camera_sensor: "2MP RCCB, 120° HFOV"
        safety_level: "ASIL_B"
    }
    
    node "Lidar ECU" {
        id: "PA-004"
        type: "ECU"
        description: "Lidar point cloud processing"
        processor: "NVIDIA Orin Nano"
        memory: "8GB unified memory"
        power: "15W"
        lidar_type: "905nm, 200m range, 10Hz"
        safety_level: "ASIL_B"
    }
    
    node "Brake Actuator ECU" {
        id: "PA-005"
        type: "ECU"
        description: "Brake-by-wire control unit"
        processor: "Infineon AURIX TC38x"
        memory: "2MB Flash + 256KB RAM"
        power: "5W"
        actuator_type: "Electric booster (iBooster)"
        safety_level: "ASIL_D"
        
        deploys "LC-004"
    }
    
    node "Instrument Cluster" {
        id: "PA-006"
        type: "Display"
        description: "Driver information and warning display"
        display_size: "12.3 inch TFT LCD"
        resolution: "1920x720"
        power: "10W"
        
        deploys "LC-003"
    }
    
    network "High-Speed CAN" {
        id: "NET-001"
        protocol: "CAN FD"
        speed: "2 Mbps"
        topology: "Dual redundant bus"
        safety_level: "ASIL_D"
        nodes: ["PA-001", "PA-002", "PA-005", "PA-006"]
    }
    
    network "Ethernet Backbone" {
        id: "NET-002"
        protocol: "Automotive Ethernet 100BASE-T1"
        speed: "100 Mbps"
        topology: "Star with switch"
        nodes: ["PA-001", "PA-003", "PA-004"]
    }
    
    network "Diagnostic Network" {
        id: "NET-003"
        protocol: "CAN"
        speed: "500 kbps"
        usage: "Diagnostics and service"
        nodes: ["PA-001"]
    }
    
    link "PA-002" -> "PA-001" via "NET-001" {
        description: "Radar object list to main ECU"
        bandwidth: "1 Mbps"
        latency: "5 ms"
        message_cycle: "50 ms"
        safety_level: "ASIL_B"
    }
    
    link "PA-003" -> "PA-001" via "NET-002" {
        description: "Camera object list to main ECU"
        bandwidth: "10 Mbps"
        latency: "10 ms"
        message_cycle: "50 ms"
        safety_level: "ASIL_B"
    }
    
    link "PA-004" -> "PA-001" via "NET-002" {
        description: "Lidar point cloud to main ECU"
        bandwidth: "20 Mbps"
        latency: "15 ms"
        message_cycle: "100 ms"
        safety_level: "ASIL_B"
    }
    
    link "PA-001" -> "PA-005" via "NET-001" {
        description: "Braking command to actuator"
        bandwidth: "500 kbps"
        latency: "2 ms"
        message_cycle: "10 ms"
        safety_level: "ASIL_D"
    }
    
    link "PA-001" -> "PA-006" via "NET-001" {
        description: "Warning signals to instrument cluster"
        bandwidth: "100 kbps"
        latency: "20 ms"
        message_cycle: "100 ms"
    }
}

epbs "Emergency Braking Product Breakdown" {
    
    system "Emergency Braking System" {
        id: "EPBS-001"
        part_number: "EBS-2000-SF"
        version: "2.0.0"
        cost: "850 USD"
        
        subsystem "Sensor Suite" {
            id: "EPBS-SS-001"
            cost: "450 USD"
            
            item "77GHz Radar Unit" {
                id: "EPBS-ITEM-001"
                part_number: "RADAR-77GHZ-ARS540"
                supplier: "Continental AG"
                cost: "180 USD"
                mass: "0.6 kg"
            }
            
            item "Front Camera Unit" {
                id: "EPBS-ITEM-002"
                part_number: "CAM-MONO-EYEQ5"
                supplier: "Mobileye"
                cost: "200 USD"
                mass: "0.5 kg"
            }
            
            item "Lidar Unit" {
                id: "EPBS-ITEM-003"
                part_number: "LIDAR-905NM-LUMINAR"
                supplier: "Luminar Technologies"
                cost: "70 USD"
                mass: "0.4 kg"
            }
        }
        
        subsystem "Control Subsystem" {
            id: "EPBS-SS-002"
            cost: "200 USD"
            
            item "Main Control ECU" {
                id: "EPBS-ITEM-004"
                part_number: "ECU-EB-MAIN-V2"
                supplier: "In-house"
                cost: "200 USD"
                mass: "0.8 kg"
                certification: "ISO 26262 ASIL-D"
            }
        }
        
        subsystem "Actuation Subsystem" {
            id: "EPBS-SS-003"
            cost: "200 USD"
            
            item "Brake Actuator Unit" {
                id: "EPBS-ITEM-005"
                part_number: "BRAKE-ACT-IBOOSTER"
                supplier: "Bosch"
                cost: "200 USD"
                mass: "0.9 kg"
            }
        }
    }
}

safety_analysis {
    
    hazard "Unintended Braking" {
        id: "HAZ-001"
        description: "System applies brakes when no threat exists"
        severity: "S2"
        exposure: "E4"
        controllability: "C1"
        asil: "ASIL_B"
    }
    
    hazard "Failed to Brake" {
        id: "HAZ-002"
        description: "System fails to brake when collision is imminent"
        severity: "S3"
        exposure: "E4"
        controllability: "C3"
        asil: "ASIL_D"
    }
    
    fmea "Sensor Fusion FMEA" {
        target: "Sensor Fusion Controller"
        failure_mode: "False positive detection"
        effects: "Unnecessary emergency braking"
        severity: "S2"
        occurrence: "O2"
        detection: "D3"
        rpn: 12
        mitigation: "Triple sensor redundancy with voting"
    }
    
    fmea "Brake Actuator FMEA" {
        target: "Brake Actuator Controller"
        failure_mode: "Actuator does not respond"
        effects: "No braking applied"
        severity: "S3"
        occurrence: "O1"
        detection: "D2"
        rpn: 6
        mitigation: "Dual redundant actuators + watchdog"
    }
}

trace "LC-001" satisfies "SYS-001" {
    rationale: "Sensor Fusion Controller provides 1-200m detection range"
}

trace "LC-001" satisfies "SYS-003" {
    rationale: "Fuses radar, camera, and lidar data"
}

trace "LC-002" satisfies "SYS-002" {
    rationale: "Braking Decision Controller meets 200ms latency requirement"
}

trace "LC-004" satisfies "SYS-004" {
    rationale: "Brake Actuator Controller modulates force 0-100%"
}

trace "LC-003" satisfies "STK-002" {
    rationale: "Warning Interface alerts driver before braking"
}

trace "LC-005" satisfies "SAFE-001" {
    rationale: "Health Monitor triggers safe degradation mode"
}
