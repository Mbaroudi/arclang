// Emergency Braking System - Complete Working Model for All Diagram Types
// Compatible with current ArcLang parser (no operational_interaction yet)
// Shows proper connections through functional exchanges and component interfaces

model EmergencyBrakingComplete {

// ============================================================================
// REQUIREMENTS - Foundation for traceability
// ============================================================================

requirements stakeholder {
    req STK_001 "Collision Prevention" {
        description: "The system shall prevent rear-end collisions in all driving conditions"
        priority: Critical
        safety_level: ASIL_D
        rationale: "Primary safety goal for occupant protection"
    }
    
    req STK_002 "Driver Awareness" {
        description: "The system shall alert the driver before automatic braking"
        priority: High
        safety_level: ASIL_B
    }
    
    req STK_003 "System Availability" {
        description: "The system shall maintain 99.9% availability during vehicle operation"
        priority: High
    }
}

requirements system {
    req SYS_001 "Detection Range" {
        description: "Detect obstacles at ranges from 1m to 200m with accuracy ≤ 0.5m"
        priority: Critical
        safety_level: ASIL_D
        traces: [STK_001]
        verification: "Sensor range and accuracy test"
    }
    
    req SYS_002 "Response Time" {
        description: "Initiate braking within 200ms of threat detection"
        priority: Critical
        safety_level: ASIL_D
        traces: [STK_001]
        verification: "End-to-end latency test"
    }
    
    req SYS_003 "Sensor Fusion" {
        description: "Fuse data from at least 2 independent sensors (radar + camera minimum)"
        priority: Critical
        safety_level: ASIL_D
        traces: [STK_001]
        verification: "Sensor failure mode test"
    }
    
    req SYS_004 "Braking Force Control" {
        description: "Modulate braking force from 0% to 100% in 10% increments"
        priority: High
        safety_level: ASIL_C
        traces: [STK_001, STK_002]
    }
    
    req SYS_005 "Driver Warning" {
        description: "Provide visual and audible warning at least 1 second before braking"
        priority: High
        safety_level: ASIL_B
        traces: [STK_002]
    }
}

requirements safety {
    req SAFE_001 "Fail-Safe Mode" {
        description: "Upon critical sensor failure, system shall enter safe state with degraded capability"
        safety_level: ASIL_D
        priority: Critical
        traces: [STK_003]
        fmea_ref: "FMEA-001-Sensor-Failure"
    }
    
    req SAFE_002 "Watchdog Monitoring" {
        description: "All safety-critical functions shall be monitored by independent watchdog with timeout ≤ 50ms"
        safety_level: ASIL_D
        priority: Critical
        traces: [STK_001]
    }
    
    req SAFE_003 "Graceful Degradation" {
        description: "System shall degrade from ASIL-D (3 sensors) to ASIL-C (2 sensors) to ASIL-B (1 sensor)"
        safety_level: ASIL_D
        priority: Critical
        traces: [STK_003, SAFE_001]
    }
}

// ============================================================================
// OPERATIONAL ANALYSIS - What the system does in operational context
// ============================================================================

operational_analysis "Emergency Braking Operational Context" {
    
    // Human Actor
    actor "Driver" {
        id: "OA_ACT_001"
        description: "Human operator controlling the vehicle"
        category: "Human"
        safety_level: ASIL_B
        responsibilities: "Monitor environment, control vehicle, respond to warnings"
    }
    
    // System Actor
    actor "Vehicle System" {
        id: "OA_ACT_002"
        description: "Autonomous emergency braking system with sensor fusion"
        category: "System"
        safety_level: ASIL_D
        responsibilities: "Detect threats, warn driver, apply emergency braking"
    }
    
    // External Actors
    actor "Leading Vehicle" {
        id: "OA_ACT_003"
        description: "Vehicle ahead in same lane"
        category: "External"
        characteristics: "Dynamic, unpredictable behavior (braking, lane changes)"
    }
    
    actor "Pedestrian" {
        id: "OA_ACT_004"
        description: "Vulnerable road user crossing or approaching vehicle path"
        category: "External"
        characteristics: "Highly vulnerable, unpredictable trajectory"
    }
    
    actor "Road Environment" {
        id: "OA_ACT_005"
        description: "Physical environment including road conditions and fixed obstacles"
        category: "External"
        characteristics: "Static obstacles, weather conditions, road surface"
    }
    
    // Driver Activities
    operational_activity "Monitor Driving Environment" {
        id: "OA_01"
        description: "Driver continuously monitors road, traffic, and hazards"
        performed_by: "OA_ACT_001"
        frequency: "Continuous"
        criticality: "High"
    }
    
    operational_activity "Control Vehicle Speed" {
        id: "OA_02"
        description: "Driver manages acceleration, deceleration, and braking"
        performed_by: "OA_ACT_001"
        frequency: "Continuous"
        criticality: "Critical"
    }
    
    operational_activity "Respond to Warnings" {
        id: "OA_03"
        description: "Driver reacts to system collision warnings"
        performed_by: "OA_ACT_001"
        response_time: "< 1.5 seconds"
        criticality: "Critical"
    }
    
    // System Activities
    operational_activity "Scan Environment Continuously" {
        id: "OA_04"
        description: "System scans 360° environment using multiple sensors"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_C
        frequency: "50ms cycle"
        coverage: "200m range, 120° horizontal FOV"
    }
    
    operational_activity "Detect Collision Threats" {
        id: "OA_05"
        description: "System identifies objects and calculates collision risk"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_D
        response_time: "< 40ms"
        algorithm: "Multi-sensor fusion with Extended Kalman Filter"
    }
    
    operational_activity "Assess Threat Level" {
        id: "OA_06"
        description: "System evaluates time-to-collision and threat severity"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_D
        output: "Threat level (None, Low, Medium, Critical)"
    }
    
    operational_activity "Warn Driver of Danger" {
        id: "OA_07"
        description: "System provides visual and audible collision warning"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_B
        warning_time: "≥ 1 second before braking"
        modalities: "Visual (HUD + cluster), Audible (3-tone alert)"
    }
    
    operational_activity "Apply Emergency Braking" {
        id: "OA_08"
        description: "System automatically applies maximum safe braking force"
        performed_by: "OA_ACT_002"
        safety_level: ASIL_D
        max_deceleration: "10 m/s²"
        modulation: "ABS + stability control"
    }
    
    // External Actor Activities
    operational_activity "Drive Ahead in Lane" {
        id: "OA_09"
        description: "Leading vehicle travels ahead, may brake suddenly"
        performed_by: "OA_ACT_003"
        variability: "High (unpredictable braking)"
    }
    
    operational_activity "Cross Vehicle Path" {
        id: "OA_10"
        description: "Pedestrian enters roadway in front of vehicle"
        performed_by: "OA_ACT_004"
        criticality: "Critical"
        typical_speed: "0.5 - 2 m/s"
    }
    
    operational_activity "Present Environmental Hazards" {
        id: "OA_11"
        description: "Environment presents fixed obstacles, poor visibility, slippery roads"
        performed_by: "OA_ACT_005"
        factors: "Weather, road surface, obstacles"
    }
    
    // Traceability to requirements
    trace STK_001 -> [OA_05, OA_06, OA_08]
    trace STK_002 -> [OA_07]
}

// ============================================================================
// SYSTEM ANALYSIS - Functional architecture (black-box functions)
// ============================================================================

architecture functional {
    
    function "Acquire Radar Data" {
        id: "SF_001"
        description: "Process 77GHz radar returns to detect objects"
        ports_in: ["radarRawSignals"]
        ports_out: ["radarObjectList", "radarConfidence"]
        safety_level: ASIL_B
        update_rate: "50ms"
    }
    
    function "Acquire Camera Data" {
        id: "SF_002"
        description: "Process camera images for object detection and classification"
        ports_in: ["cameraRawImage"]
        ports_out: ["cameraObjectList", "cameraConfidence"]
        safety_level: ASIL_B
        update_rate: "50ms"
        algorithm: "CNN-based object detection"
    }
    
    function "Acquire Lidar Data" {
        id: "SF_003"
        description: "Process lidar point cloud for precise 3D object localization"
        ports_in: ["lidarPointCloud"]
        ports_out: ["lidarObjectList", "lidarConfidence"]
        safety_level: ASIL_B
        update_rate: "100ms"
    }
    
    function "Fuse Sensor Data" {
        id: "SF_004"
        description: "Combine multi-sensor data into unified environment model"
        ports_in: ["radarObjectList", "cameraObjectList", "lidarObjectList"]
        ports_out: ["fusedObjectList", "fusionConfidence"]
        safety_level: ASIL_D
        algorithm: "Extended Kalman Filter with track-to-track fusion"
    }
    
    function "Assess Collision Risk" {
        id: "SF_005"
        description: "Calculate time-to-collision and threat level for each object"
        ports_in: ["fusedObjectList", "vehicleSpeed", "vehicleAcceleration"]
        ports_out: ["threatLevel", "timeToCollision", "targetObject"]
        safety_level: ASIL_D
        decision_logic: "TTC < 2.5s → Critical, TTC < 4s → Medium"
    }
    
    function "Generate Driver Warning" {
        id: "SF_006"
        description: "Create visual and audible warnings for driver"
        ports_in: ["threatLevel", "timeToCollision"]
        ports_out: ["visualWarning", "audioWarning"]
        safety_level: ASIL_B
        warning_stages: "Pre-warning (4s), Final warning (2s)"
    }
    
    function "Decide Braking Strategy" {
        id: "SF_007"
        description: "Determine optimal braking force and timing"
        ports_in: ["threatLevel", "timeToCollision", "driverOverride", "roadConditions"]
        ports_out: ["brakingCommand", "brakingMode"]
        safety_level: ASIL_D
        modes: "Gentle, Moderate, Emergency"
    }
    
    function "Actuate Brakes" {
        id: "SF_008"
        description: "Control hydraulic/electric brake actuators"
        ports_in: ["brakingCommand"]
        ports_out: ["actualBrakeForce", "brakePressure"]
        safety_level: ASIL_D
        control_frequency: "100Hz"
    }
    
    function "Monitor System Health" {
        id: "SF_009"
        description: "Diagnose sensor and actuator health, trigger degradation"
        ports_in: ["radarStatus", "cameraStatus", "lidarStatus", "actuatorStatus"]
        ports_out: ["systemMode", "faultCodes", "degradationLevel"]
        safety_level: ASIL_D
        diagnostics: "Continuous self-test, watchdog monitoring"
    }
    
    function "Log Safety Events" {
        id: "SF_010"
        description: "Record all emergency braking events for analysis"
        ports_in: ["threatLevel", "brakingCommand", "actualBrakeForce"]
        ports_out: ["eventLog"]
        safety_level: ASIL_C
        retention: "1000 events or 1 year"
    }
    
    // Functional Exchanges (the arrows showing data flow!)
    exchange "Radar Objects" {
        from: SF_001
        to: SF_004
        exchange_item: "radarObjectList"
        exchange_item_kind: FLOW
        bandwidth: "10 KB/s"
    }
    
    exchange "Camera Objects" {
        from: SF_002
        to: SF_004
        exchange_item: "cameraObjectList"
        exchange_item_kind: FLOW
        bandwidth: "50 KB/s"
    }
    
    exchange "Lidar Objects" {
        from: SF_003
        to: SF_004
        exchange_item: "lidarObjectList"
        exchange_item_kind: FLOW
        bandwidth: "100 KB/s"
    }
    
    exchange "Fused Environment Model" {
        from: SF_004
        to: SF_005
        exchange_item: "fusedObjectList"
        exchange_item_kind: FLOW
        bandwidth: "20 KB/s"
    }
    
    exchange "Threat Assessment" {
        from: SF_005
        to: SF_006
        exchange_item: "threatLevel"
        exchange_item_kind: DATA
        update_rate: "50ms"
    }
    
    exchange "Threat to Decision" {
        from: SF_005
        to: SF_007
        exchange_item: "threatLevel"
        exchange_item_kind: DATA
        criticality: "High"
    }
    
    exchange "Braking Command" {
        from: SF_007
        to: SF_008
        exchange_item: "brakingCommand"
        exchange_item_kind: OPERATION
        safety_level: ASIL_D
    }
    
    exchange "Health Status" {
        from: SF_009
        to: SF_004
        exchange_item: "systemMode"
        exchange_item_kind: DATA
        purpose: "Trigger degradation if sensors fail"
    }
    
    exchange "Event Data" {
        from: SF_007
        to: SF_010
        exchange_item: "brakingCommand"
        exchange_item_kind: DATA
        purpose: "Log all braking events"
    }
    
    // Traceability
    trace SYS_001 -> [SF_001, SF_002, SF_003, SF_004]
    trace SYS_002 -> [SF_005, SF_007, SF_008]
    trace SYS_003 -> [SF_004]
    trace SYS_005 -> [SF_006]
}

// ============================================================================
// LOGICAL ARCHITECTURE - Components that implement functions
// ============================================================================

architecture logical {
    
    component "Sensor Fusion Controller" {
        id: "LC_001"
        description: "Central processing unit for multi-sensor data fusion"
        allocated_functions: [SF_004, SF_005, SF_009]
        safety_level: ASIL_D
        redundancy: "Triple Modular Redundancy (TMR)"
        
        provides interface "IEnvironmentModel" {
            signals: [
                "fusedObjectList: Array<Object>",
                "threatLevel: ThreatLevel (None|Low|Medium|Critical)",
                "timeToCollision: Real [seconds]",
                "confidence: Real [0.0-1.0]"
            ]
            protocol: "CAN FD"
            bandwidth: "2 Mbps"
        }
        
        requires interface "IRadarData" {
            signals: [
                "radarObjectList: Array<RadarObject>",
                "radarStatus: HealthStatus"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "ICameraData" {
            signals: [
                "cameraObjectList: Array<CameraObject>",
                "cameraStatus: HealthStatus"
            ]
            protocol: "Automotive Ethernet"
        }
        
        requires interface "ILidarData" {
            signals: [
                "lidarObjectList: Array<LidarObject>",
                "lidarStatus: HealthStatus"
            ]
            protocol: "Automotive Ethernet"
        }
    }
    
    component "Braking Decision Controller" {
        id: "LC_002"
        description: "Decision logic for braking strategy and timing"
        allocated_functions: [SF_007]
        safety_level: ASIL_D
        redundancy: "Dual-core lockstep"
        
        provides interface "IBrakingCommand" {
            signals: [
                "brakingForce: Real [0.0-1.0]",
                "brakingMode: Mode (Gentle|Moderate|Emergency)",
                "emergencyBrake: Boolean"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["threatLevel", "timeToCollision"]
            protocol: "CAN FD"
        }
        
        requires interface "IDriverInput" {
            signals: ["driverOverride: Boolean", "manualBraking: Real"]
            protocol: "CAN"
        }
    }
    
    component "Driver Warning Interface" {
        id: "LC_003"
        description: "Human-machine interface for collision warnings"
        allocated_functions: [SF_006]
        safety_level: ASIL_B
        
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
        id: "LC_004"
        description: "Hydraulic/electric brake control unit"
        allocated_functions: [SF_008]
        safety_level: ASIL_D
        redundancy: "Dual redundant actuators"
        
        provides interface "IBrakeStatus" {
            signals: [
                "actualBrakeForce: Real",
                "brakePressure: Real [bar]",
                "actuatorHealth: HealthStatus"
            ]
            protocol: "CAN FD"
        }
        
        requires interface "IBrakingCommand" {
            signals: ["brakingForce", "brakingMode"]
            protocol: "CAN FD"
        }
    }
    
    component "Event Data Recorder" {
        id: "LC_005"
        description: "Safety event logging (black box)"
        allocated_functions: [SF_010]
        safety_level: ASIL_C
        storage: "Non-volatile memory, 1000 events"
        
        provides interface "IEventLog" {
            signals: ["logEntries: Array<Event>"]
            protocol: "Diagnostic (UDS)"
        }
        
        requires interface "IEnvironmentModel" {
            signals: ["threatLevel", "timeToCollision"]
            protocol: "CAN FD"
        }
        
        requires interface "IBrakingCommand" {
            signals: ["brakingForce", "brakingMode"]
            protocol: "CAN FD"
        }
    }
    
    // Component Connections (shows interface connections)
    connect "LC_001" -> "LC_002" via "IEnvironmentModel"
    connect "LC_001" -> "LC_003" via "IEnvironmentModel"
    connect "LC_002" -> "LC_004" via "IBrakingCommand"
    connect "LC_001" -> "LC_005" via "IEnvironmentModel"
    connect "LC_002" -> "LC_005" via "IBrakingCommand"
    
    // Traceability
    trace SF_004 -> [LC_001]
    trace SF_005 -> [LC_001]
    trace SF_006 -> [LC_003]
    trace SF_007 -> [LC_002]
    trace SF_008 -> [LC_004]
    trace SF_009 -> [LC_001]
    trace SF_010 -> [LC_005]
}

// ============================================================================
// PHYSICAL ARCHITECTURE - ECUs, networks, and deployment
// ============================================================================

architecture physical {
    
    node "Emergency Brake ECU" {
        id: "PA_001"
        type: "ECU"
        description: "Central processing unit running sensor fusion and decision logic"
        implements: ["LC_001", "LC_002", "LC_005"]
        safety_level: ASIL_D
        
        properties: {
            "processor": "Renesas RH850/F1KM-R7F7016643",
            "cores": "Dual-core lockstep @ 320MHz",
            "memory": "8MB Flash + 1MB RAM (ECC)",
            "power": "15W nominal, 20W peak",
            "operating_temp": "-40°C to +125°C",
            "certification": "ISO 26262 ASIL-D, AEC-Q100"
        }
        
        redundancy: {
            architecture: "Dual-core lockstep",
            memory_protection: "MPU + ECC on all RAM",
            watchdog: "Independent ASIL-D watchdog < 50ms",
            self_test: "Power-on self-test (POST) + periodic BIST"
        }
    }
    
    node "Radar ECU" {
        id: "PA_002"
        type: "ECU"
        description: "77GHz radar signal processing"
        implements: []
        allocated_functions: [SF_001]
        safety_level: ASIL_B
        
        properties: {
            "processor": "Infineon AURIX TC39x",
            "memory": "4MB Flash + 512KB RAM",
            "power": "8W",
            "operating_temp": "-40°C to +85°C"
        }
    }
    
    node "Camera ECU" {
        id: "PA_003"
        type: "ECU"
        description: "Vision processing with Mobileye EyeQ5"
        implements: []
        allocated_functions: [SF_002]
        safety_level: ASIL_B
        
        properties: {
            "processor": "Mobileye EyeQ5 + ARM Cortex-A76",
            "memory": "8GB LPDDR4 + 16GB eMMC",
            "power": "12W",
            "camera": "2MP RCCB sensor, 120° HFOV"
        }
    }
    
    node "Lidar ECU" {
        id: "PA_004"
        type: "ECU"
        description: "Lidar point cloud processing"
        implements: []
        allocated_functions: [SF_003]
        safety_level: ASIL_B
        
        properties: {
            "processor": "NVIDIA Orin Nano",
            "memory": "8GB unified memory",
            "power": "15W",
            "lidar": "905nm, 200m range, 10Hz"
        }
    }
    
    node "Brake Actuator ECU" {
        id: "PA_005"
        type: "ECU"
        description: "Brake-by-wire control unit"
        implements: ["LC_004"]
        allocated_functions: [SF_008]
        safety_level: ASIL_D
        
        properties: {
            "processor": "Infineon AURIX TC38x",
            "memory": "2MB Flash + 256KB RAM",
            "power": "5W",
            "actuator": "Electric booster iBooster"
        }
    }
    
    node "Instrument Cluster" {
        id: "PA_006"
        type: "Display"
        description: "Driver information and warning display"
        implements: ["LC_003"]
        allocated_functions: [SF_006]
        
        properties: {
            "display": "12.3 inch TFT LCD",
            "resolution": "1920x720",
            "power": "10W"
        }
    }
    
    // Networks
    network "High-Speed CAN" {
        id: "NET_001"
        protocol: "CAN FD"
        speed: "2 Mbps"
        nodes: ["PA_001", "PA_002", "PA_005"]
        redundancy: "Dual CAN bus"
        safety_level: ASIL_D
    }
    
    network "Ethernet Backbone" {
        id: "NET_002"
        protocol: "Automotive Ethernet (100BASE-T1)"
        speed: "100 Mbps"
        nodes: ["PA_001", "PA_003", "PA_004"]
        topology: "Star with switch"
    }
    
    network "LIN Bus" {
        id: "NET_003"
        protocol: "LIN 2.2"
        speed: "19.2 kbps"
        nodes: ["PA_001", "PA_006"]
    }
    
    // Physical Links (network connections with timing)
    link "PA_002" -> "PA_001" via "NET_001" {
        description: "Radar object list to main ECU"
        bandwidth: "1 Mbps"
        latency: "5 ms"
        message_size: "256 bytes"
        safety_level: ASIL_B
    }
    
    link "PA_003" -> "PA_001" via "NET_002" {
        description: "Camera object list to main ECU"
        bandwidth: "10 Mbps"
        latency: "10 ms"
        message_size: "10 KB"
        safety_level: ASIL_B
    }
    
    link "PA_004" -> "PA_001" via "NET_002" {
        description: "Lidar point cloud to main ECU"
        bandwidth: "20 Mbps"
        latency: "15 ms"
        message_size: "50 KB"
        safety_level: ASIL_B
    }
    
    link "PA_001" -> "PA_005" via "NET_001" {
        description: "Braking command to actuator"
        bandwidth: "500 kbps"
        latency: "2 ms"
        message_size: "16 bytes"
        safety_level: ASIL_D
        criticality: "Safety-critical path"
    }
    
    link "PA_001" -> "PA_006" via "NET_003" {
        description: "Warning signals to instrument cluster"
        bandwidth: "10 kbps"
        latency: "50 ms"
        message_size: "8 bytes"
    }
    
    // Traceability
    trace LC_001 -> [PA_001]
    trace LC_002 -> [PA_001]
    trace LC_003 -> [PA_006]
    trace LC_004 -> [PA_005]
    trace LC_005 -> [PA_001]
}

// ============================================================================
// TRACEABILITY MATRIX - End-to-end traces
// ============================================================================

traceability {
    // Requirements to Functions
    trace STK_001 -> [SYS_001, SYS_002, SYS_003]
    trace STK_002 -> [SYS_005]
    trace STK_003 -> [SAFE_001, SAFE_003]
    
    trace SYS_001 -> [SF_001, SF_002, SF_003, SF_004]
    trace SYS_002 -> [SF_005, SF_007, SF_008]
    trace SYS_003 -> [SF_004]
    trace SYS_005 -> [SF_006]
    
    // Functions to Components
    trace SF_001 -> [PA_002]
    trace SF_002 -> [PA_003]
    trace SF_003 -> [PA_004]
    trace SF_004 -> [LC_001, PA_001]
    trace SF_005 -> [LC_001, PA_001]
    trace SF_006 -> [LC_003, PA_006]
    trace SF_007 -> [LC_002, PA_001]
    trace SF_008 -> [LC_004, PA_005]
    trace SF_009 -> [LC_001, PA_001]
    trace SF_010 -> [LC_005, PA_001]
}

}
