// Emergency Braking System - Complete MBSE Model for All Diagram Types
// This model includes all necessary elements for proper diagram generation
// with arrows, connections, and proper layout

model EmergencyBrakingComplete {

// ============================================================================
// OPERATIONAL ANALYSIS - Actors, Activities, and Interactions
// ============================================================================

operational_analysis "Emergency Braking Operational Context" {
    
    // Actors
    actor "Driver" {
        id: "OA_ACT_001"
        description: "Human operator controlling the vehicle"
        category: "Human"
        safety_level: ASIL_B
    }
    
    actor "Vehicle System" {
        id: "OA_ACT_002"
        description: "Autonomous emergency braking system"
        category: "System"
        safety_level: ASIL_D
    }
    
    actor "Leading Vehicle" {
        id: "OA_ACT_003"
        description: "Vehicle ahead in same lane"
        category: "External"
    }
    
    actor "Pedestrian" {
        id: "OA_ACT_004"
        description: "Vulnerable road user"
        category: "External"
    }
    
    // Operational Activities
    operational_activity "Monitor Road" {
        id: "OA_01"
        description: "Driver continuously monitors road and traffic"
        performed_by: "OA_ACT_001"
    }
    
    operational_activity "Control Speed" {
        id: "OA_02"
        description: "Driver controls vehicle acceleration and braking"
        performed_by: "OA_ACT_001"
    }
    
    operational_activity "Scan Environment" {
        id: "OA_03"
        description: "System scans for obstacles using sensors"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_C
    }
    
    operational_activity "Detect Collision Risk" {
        id: "OA_04"
        description: "System analyzes collision probability"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_D
    }
    
    operational_activity "Warn Driver" {
        id: "OA_05"
        description: "System alerts driver of danger"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_B
    }
    
    operational_activity "Apply Emergency Brake" {
        id: "OA_06"
        description: "System automatically applies maximum braking"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_D
    }
    
    operational_activity "Drive Ahead" {
        id: "OA_07"
        description: "Leading vehicle moves in traffic"
        performed_by: "OA_ACT_003"
    }
    
    operational_activity "Cross Road" {
        id: "OA_08"
        description: "Pedestrian crosses vehicle path"
        performed_by: "OA_ACT_004"
    }
    
    // Operational Interactions (arrows between actors)
    operational_interaction "Driver Input" {
        id: "OI_01"
        from: "OA_ACT_001"
        to: "OA_ACT_002"
        exchange_item_kind: OPERATION
        description: "Driver provides manual control inputs"
    }
    
    operational_interaction "Visual Warning" {
        id: "OI_02"
        from: "OA_ACT_002"
        to: "OA_ACT_001"
        exchange_item_kind: EVENT
        description: "System displays collision warning"
    }
    
    operational_interaction "Distance Data" {
        id: "OI_03"
        from: "OA_ACT_003"
        to: "OA_ACT_002"
        exchange_item_kind: FLOW
        description: "Relative distance and speed"
    }
    
    operational_interaction "Pedestrian Position" {
        id: "OI_04"
        from: "OA_ACT_004"
        to: "OA_ACT_002"
        exchange_item_kind: FLOW
        description: "Pedestrian location and trajectory"
    }
    
    operational_interaction "Brake Command" {
        id: "OI_05"
        from: "OA_ACT_002"
        to: "OA_ACT_001"
        exchange_item_kind: OPERATION
        description: "Emergency braking initiated"
    }
}

// ============================================================================
// SYSTEM ANALYSIS - Functions and Functional Exchanges
// ============================================================================

architecture functional {
    
    function "Detect Obstacles" {
        id: "SF_001"
        description: "Fuse sensor data to detect obstacles"
        ports_in: ["radarData", "cameraData", "lidarData"]
        ports_out: ["objectList"]
        safety_level: ASIL_D
    }
    
    function "Assess Collision Risk" {
        id: "SF_002"
        description: "Calculate time-to-collision and risk level"
        ports_in: ["objectList", "vehicleSpeed"]
        ports_out: ["riskLevel", "timeToCollision"]
        safety_level: ASIL_D
    }
    
    function "Generate Warning" {
        id: "SF_003"
        description: "Create visual and audible warnings"
        ports_in: ["riskLevel"]
        ports_out: ["warningSignal"]
        safety_level: ASIL_B
    }
    
    function "Decide Braking" {
        id: "SF_004"
        description: "Determine braking strategy"
        ports_in: ["riskLevel", "timeToCollision", "driverOverride"]
        ports_out: ["brakingCommand"]
        safety_level: ASIL_D
    }
    
    function "Actuate Brakes" {
        id: "SF_005"
        description: "Control brake actuators"
        ports_in: ["brakingCommand"]
        ports_out: ["brakeForce"]
        safety_level: ASIL_D
    }
    
    function "Monitor System Health" {
        id: "SF_006"
        description: "Diagnose sensor and actuator health"
        ports_in: ["sensorStatus"]
        ports_out: ["systemHealth"]
        safety_level: ASIL_D
    }
    
    // Functional Exchanges (data flows between functions)
    exchange "Object Data" {
        from: SF_001
        to: SF_002
        exchange_item: "objectList"
        exchange_item_kind: FLOW
    }
    
    exchange "Risk Assessment" {
        from: SF_002
        to: SF_003
        exchange_item: "riskLevel"
        exchange_item_kind: DATA
    }
    
    exchange "Risk to Decision" {
        from: SF_002
        to: SF_004
        exchange_item: "riskLevel"
        exchange_item_kind: DATA
    }
    
    exchange "Braking Command" {
        from: SF_004
        to: SF_005
        exchange_item: "brakingCommand"
        exchange_item_kind: OPERATION
    }
}

// ============================================================================
// LOGICAL ARCHITECTURE - Components and Interfaces
// ============================================================================

architecture logical {
    
    component "Sensor Fusion Controller" {
        id: "LC_001"
        description: "Multi-sensor data fusion processor"
        allocated_functions: [SF_001, SF_002]
        safety_level: ASIL_D
        
        provides interface "IEnvironmentModel" {
            signals: [
                "objectList: Array<Object>",
                "riskLevel: RiskLevel",
                "timeToCollision: Real"
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
            signals: ["lidarPointCloud", "lidarStatus"]
            protocol: "Ethernet"
        }
    }
    
    component "Braking Decision Controller" {
        id: "LC_002"
        description: "Braking strategy decision logic"
        allocated_functions: [SF_004]
        safety_level: ASIL_D
        
        provides interface "IBrakingCommand" {
            signals: [
                "brakingForce: Real",
                "brakingMode: Mode"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["riskLevel", "timeToCollision"]
            protocol: "CAN FD"
        }
    }
    
    component "Warning Interface" {
        id: "LC_003"
        description: "Driver warning system"
        allocated_functions: [SF_003]
        safety_level: ASIL_B
        
        provides interface "IWarning" {
            signals: ["visualAlert", "audioAlert"]
            protocol: "LIN"
        }
    }
    
    component "Brake Actuator Controller" {
        id: "LC_004"
        description: "Brake-by-wire control unit"
        allocated_functions: [SF_005]
        safety_level: ASIL_D
        
        provides interface "IBrakeStatus" {
            signals: ["actualBrakeForce", "actuatorHealth"]
            protocol: "CAN FD"
        }
        
        requires interface "IBrakingCommand" {
            signals: ["brakingForce"]
            protocol: "CAN FD"
        }
    }
    
    component "Health Monitor" {
        id: "LC_005"
        description: "System health monitoring"
        allocated_functions: [SF_006]
        safety_level: ASIL_D
        
        provides interface "ISystemHealth" {
            signals: ["systemMode", "faultCode"]
            protocol: "CAN FD"
        }
    }
    
    // Component Connections
    connect "LC_001" -> "LC_002" via "IEnvironmentModel"
    connect "LC_001" -> "LC_003" via "IEnvironmentModel"
    connect "LC_002" -> "LC_004" via "IBrakingCommand"
}

// ============================================================================
// PHYSICAL ARCHITECTURE - Hardware Nodes and Networks
// ============================================================================

architecture physical {
    
    node "Emergency Brake ECU" {
        id: "PA_001"
        type: "ECU"
        description: "Main processing unit"
        implements: ["LC_001", "LC_002"]
        
        properties: {
            "processor": "Renesas RH850",
            "memory": "8MB Flash + 1MB RAM",
            "power": "15W"
        }
        
        safety_level: ASIL_D
    }
    
    node "Radar ECU" {
        id: "PA_002"
        type: "ECU"
        description: "77GHz radar processor"
        
        properties: {
            "processor": "Infineon AURIX",
            "power": "8W"
        }
        
        safety_level: ASIL_B
    }
    
    node "Camera ECU" {
        id: "PA_003"
        type: "ECU"
        description: "Vision processing unit"
        
        properties: {
            "processor": "Mobileye EyeQ5",
            "power": "12W"
        }
        
        safety_level: ASIL_B
    }
    
    node "Lidar ECU" {
        id: "PA_004"
        type: "ECU"
        description: "Lidar point cloud processing"
        
        properties: {
            "processor": "NVIDIA Orin",
            "power": "20W"
        }
        
        safety_level: ASIL_B
    }
    
    node "Brake Actuator ECU" {
        id: "PA_005"
        type: "ECU"
        description: "Brake actuator control"
        implements: ["LC_004"]
        
        properties: {
            "processor": "Infineon AURIX",
            "power": "5W"
        }
        
        safety_level: ASIL_D
    }
    
    node "Instrument Cluster" {
        id: "PA_006"
        type: "Display"
        description: "Driver warning display"
        implements: ["LC_003"]
        
        properties: {
            "display": "12.3 inch TFT",
            "power": "10W"
        }
    }
    
    // Networks
    network "High-Speed CAN" {
        id: "NET_001"
        protocol: "CAN FD"
        speed: "2 Mbps"
        nodes: ["PA_001", "PA_002", "PA_005"]
    }
    
    network "Ethernet Backbone" {
        id: "NET_002"
        protocol: "Automotive Ethernet"
        speed: "1000 Mbps"
        nodes: ["PA_001", "PA_003", "PA_004"]
    }
    
    network "LIN Bus" {
        id: "NET_003"
        protocol: "LIN"
        speed: "19.2 kbps"
        nodes: ["PA_001", "PA_006"]
    }
    
    // Physical Links
    link "PA_002" -> "PA_001" via "NET_001" {
        description: "Radar data to main ECU"
        bandwidth: "1 Mbps"
        latency: "5 ms"
    }
    
    link "PA_003" -> "PA_001" via "NET_002" {
        description: "Camera data to main ECU"
        bandwidth: "100 Mbps"
        latency: "10 ms"
    }
    
    link "PA_004" -> "PA_001" via "NET_002" {
        description: "Lidar data to main ECU"
        bandwidth: "200 Mbps"
        latency: "15 ms"
    }
    
    link "PA_001" -> "PA_005" via "NET_001" {
        description: "Braking commands to actuator"
        bandwidth: "500 kbps"
        latency: "2 ms"
        safety_level: ASIL_D
    }
    
    link "PA_001" -> "PA_006" via "NET_003" {
        description: "Warning signals to display"
        bandwidth: "19.2 kbps"
        latency: "50 ms"
    }
}

// ============================================================================
// REQUIREMENTS - Traceability
// ============================================================================

requirements stakeholder {
    req STK_001 "Collision Prevention" {
        description: "The system shall prevent rear-end collisions"
        priority: Critical
        safety_level: ASIL_D
    }
    
    req STK_002 "User Comfort" {
        description: "The system shall provide smooth braking"
        priority: High
    }
}

requirements system {
    req SYS_001 "Detection Range" {
        description: "Detect obstacles at 1-200m range"
        priority: Critical
        safety_level: ASIL_D
        traces: [STK_001]
    }
    
    req SYS_002 "Response Time" {
        description: "Initiate braking within 200ms"
        priority: Critical
        safety_level: ASIL_D
        traces: [STK_001]
    }
    
    req SYS_003 "Sensor Fusion" {
        description: "Fuse data from 3 independent sensors"
        priority: Critical
        safety_level: ASIL_D
        traces: [STK_001]
    }
}

// Traceability Links
traceability {
    trace SYS_001 -> [SF_001, SF_002]
    trace SYS_002 -> [SF_004, SF_005]
    trace SYS_003 -> [SF_001]
    
    trace SF_001 -> [LC_001]
    trace SF_002 -> [LC_001]
    trace SF_004 -> [LC_002]
    trace SF_005 -> [LC_004]
    
    trace LC_001 -> [PA_001]
    trace LC_002 -> [PA_001]
    trace LC_004 -> [PA_005]
}

}
