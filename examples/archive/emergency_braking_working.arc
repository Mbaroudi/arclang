// Emergency Braking System - Working Model with Current ArcLang Syntax
// This uses the actual parser-supported syntax for all diagram types

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
        safety_level: "ASIL_D"
    }
    
    actor "Leading Vehicle" {
        id: "OA-ACT-003"
        description: "Vehicle ahead in same lane"
        category: "External"
    }
    
    actor "Pedestrian" {
        id: "OA-ACT-004"
        description: "Vulnerable road user"
        category: "External"
    }
    
    operational_activity "Monitor Environment" {
        id: "OA-01"
        description: "Driver monitors road and traffic"
        performed_by: "OA-ACT-001"
    }
    
    operational_activity "Control Vehicle" {
        id: "OA-02"
        description: "Driver controls speed and braking"
        performed_by: "OA-ACT-001"
    }
    
    operational_activity "Scan Environment" {
        id: "OA-03"
        description: "System scans for obstacles"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_C"
    }
    
    operational_activity "Detect Collision Risk" {
        id: "OA-04"
        description: "System analyzes collision probability"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_D"
    }
    
    operational_activity "Warn Driver" {
        id: "OA-05"
        description: "System alerts driver"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_B"
    }
    
    operational_activity "Apply Emergency Brake" {
        id: "OA-06"
        description: "System applies automatic braking"
        performed_by: "OA-ACT-002"
        safety_level: "ASIL_D"
    }
    
    operational_activity "Drive Ahead" {
        id: "OA-07"
        description: "Leading vehicle moves in traffic"
        performed_by: "OA-ACT-003"
    }
    
    operational_activity "Cross Road" {
        id: "OA-08"
        description: "Pedestrian crosses vehicle path"
        performed_by: "OA-ACT-004"
    }
}

system_analysis "Emergency Braking System Analysis" {
    
    requirement "STK-001" {
        description: "The system shall prevent rear-end collisions"
        priority: "Critical"
        safety_level: "ASIL_D"
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
        description: "Fuse data from at least 2 independent sensors"
        priority: "Critical"
        safety_level: "ASIL_D"
    }
    
    requirement "SYS-004" {
        description: "Modulate braking force from 0% to 100%"
        priority: "High"
        safety_level: "ASIL_C"
    }
}

system_analysis "Emergency Braking Functional Analysis" {
    
    function "Acquire Radar Data" {
        id: "SF-001"
        description: "Process 77GHz radar for object detection"
        ports_in: ["radarSignals"]
        ports_out: ["radarObjects"]
        safety_level: "ASIL_B"
    }
    
    function "Acquire Camera Data" {
        id: "SF-002"
        description: "Process camera images for object detection"
        ports_in: ["cameraImage"]
        ports_out: ["cameraObjects"]
        safety_level: "ASIL_B"
    }
    
    function "Acquire Lidar Data" {
        id: "SF-003"
        description: "Process lidar point cloud"
        ports_in: ["lidarCloud"]
        ports_out: ["lidarObjects"]
        safety_level: "ASIL_B"
    }
    
    function "Fuse Sensor Data" {
        id: "SF-004"
        description: "Combine multi-sensor data into unified model"
        ports_in: ["radarObjects", "cameraObjects", "lidarObjects"]
        ports_out: ["fusedObjects", "confidence"]
        safety_level: "ASIL_D"
    }
    
    function "Assess Collision Risk" {
        id: "SF-005"
        description: "Calculate time-to-collision and threat level"
        ports_in: ["fusedObjects", "vehicleSpeed"]
        ports_out: ["threatLevel", "timeToCollision"]
        safety_level: "ASIL_D"
    }
    
    function "Generate Warning" {
        id: "SF-006"
        description: "Create visual and audible warnings"
        ports_in: ["threatLevel"]
        ports_out: ["warningSignal"]
        safety_level: "ASIL_B"
    }
    
    function "Decide Braking" {
        id: "SF-007"
        description: "Determine braking strategy"
        ports_in: ["threatLevel", "timeToCollision"]
        ports_out: ["brakingCommand"]
        safety_level: "ASIL_D"
    }
    
    function "Actuate Brakes" {
        id: "SF-008"
        description: "Control brake actuators"
        ports_in: ["brakingCommand"]
        ports_out: ["brakeForce"]
        safety_level: "ASIL_D"
    }
    
    function "Monitor Health" {
        id: "SF-009"
        description: "Diagnose sensor and actuator health"
        ports_in: ["sensorStatus"]
        ports_out: ["systemHealth"]
        safety_level: "ASIL_D"
    }
    
    function "Log Events" {
        id: "SF-010"
        description: "Record all emergency braking events"
        ports_in: ["brakingCommand", "threatLevel"]
        ports_out: ["eventLog"]
        safety_level: "ASIL_C"
    }
    
    exchange "Radar to Fusion" {
        from: SF-001
        to: SF-004
        exchange_item: "radarObjects"
        exchange_item_kind: FLOW
    }
    
    exchange "Camera to Fusion" {
        from: SF-002
        to: SF-004
        exchange_item: "cameraObjects"
        exchange_item_kind: FLOW
    }
    
    exchange "Lidar to Fusion" {
        from: SF-003
        to: SF-004
        exchange_item: "lidarObjects"
        exchange_item_kind: FLOW
    }
    
    exchange "Fused Data" {
        from: SF-004
        to: SF-005
        exchange_item: "fusedObjects"
        exchange_item_kind: FLOW
    }
    
    exchange "Threat to Warning" {
        from: SF-005
        to: SF-006
        exchange_item: "threatLevel"
        exchange_item_kind: DATA
    }
    
    exchange "Threat to Decision" {
        from: SF-005
        to: SF-007
        exchange_item: "threatLevel"
        exchange_item_kind: DATA
    }
    
    exchange "Braking Command" {
        from: SF-007
        to: SF-008
        exchange_item: "brakingCommand"
        exchange_item_kind: OPERATION
    }
    
    exchange "Health Status" {
        from: SF-009
        to: SF-004
        exchange_item: "systemHealth"
        exchange_item_kind: DATA
    }
    
    exchange "Event Data" {
        from: SF-007
        to: SF-010
        exchange_item: "brakingCommand"
        exchange_item_kind: DATA
    }
}

logical_architecture "Emergency Braking Logical Architecture" {
    
    component "Sensor Fusion Controller" {
        id: "LC-001"
        type: "Logical"
        description: "Multi-sensor data fusion processor"
        safety_level: "ASIL_D"
        
        function "Fuse Sensor Data" {
            id: "SF-004"
        }
        
        function "Assess Collision Risk" {
            id: "SF-005"
        }
        
        function "Monitor Health" {
            id: "SF-009"
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
            protocol: "Ethernet"
        }
        
        requires interface "ILidarData" {
            signals: ["lidarObjects", "lidarStatus"]
            protocol: "Ethernet"
        }
    }
    
    component "Braking Decision Controller" {
        id: "LC-002"
        type: "Logical"
        description: "Braking strategy decision logic"
        safety_level: "ASIL_D"
        
        function "Decide Braking" {
            id: "SF-007"
        }
        
        provides interface "IBrakingCommand" {
            signals: [
                "brakingForce: Real",
                "brakingMode: Enum"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["threatLevel", "timeToCollision"]
            protocol: "CAN FD"
        }
    }
    
    component "Warning Interface" {
        id: "LC-003"
        type: "Logical"
        description: "Driver warning system"
        safety_level: "ASIL_B"
        
        function "Generate Warning" {
            id: "SF-006"
        }
        
        provides interface "IWarning" {
            signals: ["visualAlert", "audioAlert"]
            protocol: "LIN"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["threatLevel"]
            protocol: "CAN FD"
        }
    }
    
    component "Brake Actuator Controller" {
        id: "LC-004"
        type: "Logical"
        description: "Brake control unit"
        safety_level: "ASIL_D"
        
        function "Actuate Brakes" {
            id: "SF-008"
        }
        
        provides interface "IBrakeStatus" {
            signals: ["actualBrakeForce", "brakePressure"]
            protocol: "CAN FD"
        }
        
        requires interface "IBrakingCommand" {
            signals: ["brakingForce"]
            protocol: "CAN FD"
        }
    }
    
    component "Event Recorder" {
        id: "LC-005"
        type: "Logical"
        description: "Safety event logging"
        safety_level: "ASIL_C"
        
        function "Log Events" {
            id: "SF-010"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["threatLevel"]
            protocol: "CAN FD"
        }
        
        requires interface "IBrakingCommand" {
            signals: ["brakingForce"]
            protocol: "CAN FD"
        }
    }
    
    connect "LC-001" -> "LC-002" via "IEnvironmentModel"
    connect "LC-001" -> "LC-003" via "IEnvironmentModel"
    connect "LC-002" -> "LC-004" via "IBrakingCommand"
    connect "LC-001" -> "LC-005" via "IEnvironmentModel"
    connect "LC-002" -> "LC-005" via "IBrakingCommand"
}

physical_architecture "Emergency Braking Physical Architecture" {
    
    node "Emergency Brake ECU" {
        id: "PA-001"
        type: "ECU"
        description: "Central processing unit"
        processor: "Renesas RH850 F1KM"
        memory: "8MB Flash + 1MB RAM"
        power: "15W"
        safety_level: "ASIL_D"
        
        deploys "LC-001"
        deploys "LC-002"
        deploys "LC-005"
    }
    
    node "Radar ECU" {
        id: "PA-002"
        type: "ECU"
        description: "77GHz radar processor"
        processor: "Infineon AURIX TC39x"
        memory: "4MB Flash"
        power: "8W"
        safety_level: "ASIL_B"
    }
    
    node "Camera ECU" {
        id: "PA-003"
        type: "ECU"
        description: "Vision processing"
        processor: "Mobileye EyeQ5"
        memory: "8GB LPDDR4"
        power: "12W"
        safety_level: "ASIL_B"
    }
    
    node "Lidar ECU" {
        id: "PA-004"
        type: "ECU"
        description: "Lidar processing"
        processor: "NVIDIA Orin Nano"
        memory: "8GB"
        power: "15W"
        safety_level: "ASIL_B"
    }
    
    node "Brake Actuator ECU" {
        id: "PA-005"
        type: "ECU"
        description: "Brake control"
        processor: "Infineon AURIX TC38x"
        memory: "2MB Flash"
        power: "5W"
        safety_level: "ASIL_D"
        
        deploys "LC-004"
    }
    
    node "Instrument Cluster" {
        id: "PA-006"
        type: "Display"
        description: "Driver display"
        display: "12.3 inch TFT"
        power: "10W"
        
        deploys "LC-003"
    }
    
    network "High-Speed CAN" {
        id: "NET-001"
        protocol: "CAN FD"
        speed: "2 Mbps"
        nodes: ["PA-001", "PA-002", "PA-005"]
    }
    
    network "Ethernet Backbone" {
        id: "NET-002"
        protocol: "Automotive Ethernet"
        speed: "100 Mbps"
        nodes: ["PA-001", "PA-003", "PA-004"]
    }
    
    network "LIN Bus" {
        id: "NET-003"
        protocol: "LIN 2.2"
        speed: "19.2 kbps"
        nodes: ["PA-001", "PA-006"]
    }
    
    link "PA-002" -> "PA-001" via "NET-001" {
        description: "Radar data to main ECU"
        bandwidth: "1 Mbps"
        latency: "5 ms"
    }
    
    link "PA-003" -> "PA-001" via "NET-002" {
        description: "Camera data to main ECU"
        bandwidth: "10 Mbps"
        latency: "10 ms"
    }
    
    link "PA-004" -> "PA-001" via "NET-002" {
        description: "Lidar data to main ECU"
        bandwidth: "20 Mbps"
        latency: "15 ms"
    }
    
    link "PA-001" -> "PA-005" via "NET-001" {
        description: "Braking commands"
        bandwidth: "500 kbps"
        latency: "2 ms"
        safety_level: "ASIL_D"
    }
    
    link "PA-001" -> "PA-006" via "NET-003" {
        description: "Warning signals"
        bandwidth: "10 kbps"
        latency: "50 ms"
    }
}
