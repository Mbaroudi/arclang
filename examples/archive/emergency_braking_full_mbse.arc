// ============================================================================
// OPERATIONAL ARCHITECTURE (OA Layer)
// Defines the operational context with actors and activities
// ============================================================================

operational_analysis "Emergency Braking Operational Context" {
    actor "Driver" {
        id: "OA-ACT-001"
        description: "Human operator controlling the vehicle"
        category: "Human"
        type: "Operational"
    }
    
    actor "Vehicle System" {
        id: "OA-ACT-002"
        description: "Autonomous emergency braking system"
        category: "System"
        type: "Operational"
    }
    
    actor "Road Environment" {
        id: "OA-ACT-003"
        description: "Physical environment including road, weather, obstacles"
        category: "External"
        type: "Operational"
    }
    
    actor "Other Vehicles" {
        id: "OA-ACT-004"
        description: "Surrounding traffic participants"
        category: "External"
        type: "Operational"
    }
    
    actor "Pedestrians" {
        id: "OA-ACT-005"
        description: "Vulnerable road users"
        category: "External"
        type: "Operational"
    }
    
    operational_activity "Monitor Driving Environment" {
        id: "OA-01"
        description: "Driver continuously monitors road conditions and traffic"
        performed_by: "OA-ACT-001"
        category: "Perception"
    }
    
    operational_activity "Control Vehicle Speed" {
        id: "OA-02"
        description: "Driver controls acceleration and braking"
        performed_by: "OA-ACT-001"
        category: "Control"
    }
    
    operational_activity "Detect Potential Collision" {
        id: "OA-03"
        description: "System detects objects and calculates collision risk"
        performed_by: "OA-ACT-002"
        category: "Perception"
    }
    
    operational_activity "Alert Driver" {
        id: "OA-04"
        description: "System provides warnings to driver"
        performed_by: "OA-ACT-002"
        category: "Communication"
    }
    
    operational_activity "Apply Emergency Braking" {
        id: "OA-05"
        description: "System automatically applies brakes to prevent collision"
        performed_by: "OA-ACT-002"
        category: "Control"
    }
    
    operational_activity "Present Road Hazards" {
        id: "OA-06"
        description: "Environment presents obstacles, weather conditions"
        performed_by: "OA-ACT-003"
        category: "Context"
    }
    
    operational_activity "Navigate in Traffic" {
        id: "OA-07"
        description: "Other vehicles move creating dynamic scenarios"
        performed_by: "OA-ACT-004"
        category: "Context"
    }
    
    operational_activity "Cross Road" {
        id: "OA-08"
        description: "Pedestrians cross vehicle path"
        performed_by: "OA-ACT-005"
        category: "Context"
    }
}

// ============================================================================
// SYSTEM ANALYSIS (SA Layer)
// Requirements, capabilities, and system functions
// ============================================================================

system_analysis "Emergency Braking System Requirements" {
    // Safety-Critical Requirements
    requirement "Obstacle Detection Range" {
        id: "SYS-REQ-001"
        description: "System shall detect obstacles up to 150m ahead at highway speeds"
        priority: "Critical"
        safety_level: "ASIL_D"
        verification_method: "Test"
        rationale: "Required for safe stopping distance at 130 km/h"
    }
    
    requirement "Response Time Constraint" {
        id: "SYS-REQ-002"
        description: "System shall initiate braking within 100ms of confirmed threat"
        priority: "Critical"
        safety_level: "ASIL_D"
        verification_method: "Test"
        rationale: "Critical timing for collision avoidance"
    }
    
    requirement "False Positive Rate" {
        id: "SYS-REQ-003"
        description: "False positive rate shall be less than 1 per 10^9 operating hours"
        priority: "Critical"
        safety_level: "ASIL_D"
        verification_method: "Analysis"
        rationale: "Prevent unnecessary emergency braking events"
    }
    
    requirement "Multi-Sensor Fusion" {
        id: "SYS-REQ-004"
        description: "System shall fuse data from minimum 3 independent sensor types"
        priority: "High"
        safety_level: "ASIL_C"
        verification_method: "Test"
        rationale: "Redundancy and improved detection accuracy"
    }
    
    requirement "Degraded Mode Operation" {
        id: "SYS-REQ-005"
        description: "System shall maintain basic functionality with one sensor failure"
        priority: "High"
        safety_level: "ASIL_C"
        verification_method: "Test"
        rationale: "Graceful degradation for availability"
    }
    
    requirement "Driver Override Capability" {
        id: "SYS-REQ-006"
        description: "Driver shall be able to override automatic braking at any time"
        priority: "High"
        safety_level: "ASIL_B"
        verification_method: "Test"
        rationale: "Driver maintains ultimate authority"
    }
    
    requirement "Continuous Health Monitoring" {
        id: "SYS-REQ-007"
        description: "System shall monitor and report status every 50ms"
        priority: "Medium"
        safety_level: "ASIL_B"
        verification_method: "Review"
        rationale: "Early fault detection"
    }
    
    requirement "All-Weather Performance" {
        id: "SYS-REQ-008"
        description: "System shall operate in rain, fog, snow with degraded performance notice"
        priority: "High"
        safety_level: "ASIL_C"
        verification_method: "Test"
        rationale: "Real-world operating conditions"
    }
    
    // System Functions
    system_function "Acquire Sensor Data" {
        id: "SF-001"
        description: "Collect data from all sensors"
        category: "Input"
        allocated_to: "Sensor Interface Layer"
        
        function "Read Radar Data" {
            id: "SF-001-01"
            description: "Acquire 77GHz radar returns"
        }
        
        function "Read Camera Images" {
            id: "SF-001-02"
            description: "Capture stereo camera frames at 30Hz"
        }
        
        function "Read Lidar Points" {
            id: "SF-001-03"
            description: "Acquire 3D point cloud data"
        }
        
        function "Read Vehicle Data" {
            id: "SF-001-04"
            description: "Read speed, steering angle, brake status"
        }
    }
    
    system_function "Process and Fuse Sensor Data" {
        id: "SF-002"
        description: "Validate, align, and fuse multi-sensor data"
        category: "Processing"
        allocated_to: "Fusion Layer"
        
        function "Validate Sensor Data" {
            id: "SF-002-01"
            description: "Check data quality and plausibility"
        }
        
        function "Temporal Alignment" {
            id: "SF-002-02"
            description: "Synchronize sensor data to common timestamp"
        }
        
        function "Spatial Registration" {
            id: "SF-002-03"
            description: "Transform data to vehicle coordinate frame"
        }
        
        function "Multi-Sensor Fusion" {
            id: "SF-002-04"
            description: "Combine sensor data using Kalman filtering"
        }
    }
    
    system_function "Detect and Classify Objects" {
        id: "SF-003"
        description: "Identify objects in environment"
        category: "Perception"
        allocated_to: "Perception Layer"
        
        function "Segment Objects" {
            id: "SF-003-01"
            description: "Separate objects from background"
        }
        
        function "Classify Object Type" {
            id: "SF-003-02"
            description: "Determine if vehicle, pedestrian, cyclist, static"
        }
        
        function "Estimate Object Dimensions" {
            id: "SF-003-03"
            description: "Calculate bounding box and size"
        }
    }
    
    system_function "Track Object Trajectories" {
        id: "SF-004"
        description: "Maintain temporal consistency of objects"
        category: "Perception"
        allocated_to: "Tracking Layer"
        
        function "Associate Detections" {
            id: "SF-004-01"
            description: "Match detections to existing tracks"
        }
        
        function "Update Track States" {
            id: "SF-004-02"
            description: "Update position, velocity estimates"
        }
        
        function "Predict Future Motion" {
            id: "SF-004-03"
            description: "Forecast object trajectories 3 seconds ahead"
        }
    }
    
    system_function "Assess Collision Risk" {
        id: "SF-005"
        description: "Evaluate threat level for each object"
        category: "Decision"
        allocated_to: "Risk Assessment Layer"
        
        function "Calculate Time-to-Collision" {
            id: "SF-005-01"
            description: "Compute TTC for each tracked object"
        }
        
        function "Evaluate Threat Level" {
            id: "SF-005-02"
            description: "Classify as no-risk, warning, critical"
        }
        
        function "Determine Intervention Need" {
            id: "SF-005-03"
            description: "Decide if automatic braking required"
        }
    }
    
    system_function "Plan Braking Response" {
        id: "SF-006"
        description: "Calculate optimal braking strategy"
        category: "Planning"
        allocated_to: "Planning Layer"
        
        function "Select Braking Strategy" {
            id: "SF-006-01"
            description: "Choose partial or full braking"
        }
        
        function "Calculate Deceleration Profile" {
            id: "SF-006-02"
            description: "Compute required brake force over time"
        }
        
        function "Check Feasibility" {
            id: "SF-006-03"
            description: "Verify vehicle dynamics constraints"
        }
    }
    
    system_function "Execute Braking Command" {
        id: "SF-007"
        description: "Apply brakes and monitor execution"
        category: "Control"
        allocated_to: "Actuation Layer"
        
        function "Send Brake Commands" {
            id: "SF-007-01"
            description: "Interface with brake ECU"
        }
        
        function "Monitor Brake Response" {
            id: "SF-007-02"
            description: "Verify actual vs commanded deceleration"
        }
        
        function "Handle Override" {
            id: "SF-007-03"
            description: "Respond to driver intervention"
        }
    }
    
    system_function "Monitor System Health" {
        id: "SF-008"
        description: "Continuous safety monitoring"
        category: "Safety"
        allocated_to: "Safety Monitor"
        
        function "Check Sensor Status" {
            id: "SF-008-01"
            description: "Verify all sensors operational"
        }
        
        function "Diagnose Faults" {
            id: "SF-008-02"
            description: "Detect and classify failures"
        }
        
        function "Report System Status" {
            id: "SF-008-03"
            description: "Provide status to driver and vehicle"
        }
    }
}

// ============================================================================
// LOGICAL ARCHITECTURE (LA Layer)
// Logical components, interfaces, and data flows
// ============================================================================

logical_architecture "Emergency Braking Logical Architecture" {
    
    component "Application Controller" {
        id: "LA-001"
        type: "Logical"
        description: "Top-level application coordination and mode management"
        safety_level: "ASIL_D"
        
        function "Manage System State" {
            id: "LF-001"
            description: "Handle initialization, operational, degraded, fault states"
        }
        
        function "Coordinate Processing Pipeline" {
            id: "LF-002"
            description: "Orchestrate sensor-to-actuation data flow"
        }
        
        component "Safety Supervisor" {
            id: "LA-001-01"
            type: "Logical"
            description: "Independent safety monitoring"
            safety_level: "ASIL_D"
            
            function "Monitor Watchdog" {
                id: "LF-003"
                description: "Verify system responsiveness"
            }
            
            function "Validate Outputs" {
                id: "LF-004"
                description: "Check output plausibility"
            }
        }
    }
    
    component "Multi-Sensor Fusion Unit" {
        id: "LA-002"
        type: "Logical"
        description: "Central sensor data processing and fusion"
        safety_level: "ASIL_C"
        
        function "Fuse Sensor Data" {
            id: "LF-005"
            description: "Multi-modal sensor fusion"
        }
        
        function "Generate Environment Model" {
            id: "LF-006"
            description: "Create unified scene representation"
        }
        
        component "Data Validator" {
            id: "LA-002-01"
            type: "Logical"
            description: "Input data quality assurance"
            
            function "Check Data Integrity" {
                id: "LF-007"
                description: "CRC and plausibility checks"
            }
            
            function "Flag Anomalies" {
                id: "LF-008"
                description: "Detect out-of-range values"
            }
        }
    }
    
    component "Radar Sensor Interface" {
        id: "LA-003"
        type: "Logical"
        description: "77GHz radar data acquisition and preprocessing"
        safety_level: "ASIL_C"
        
        function "Process Radar Returns" {
            id: "LF-009"
            description: "Convert raw radar to object detections"
        }
        
        function "Filter Clutter" {
            id: "LF-010"
            description: "Remove false detections"
        }
    }
    
    component "Camera Sensor Interface" {
        id: "LA-004"
        type: "Logical"
        description: "Stereo vision processing"
        safety_level: "ASIL_C"
        
        function "Process Images" {
            id: "LF-011"
            description: "Image rectification and feature extraction"
        }
        
        function "Compute Disparity" {
            id: "LF-012"
            description: "Stereo depth calculation"
        }
    }
    
    component "Lidar Sensor Interface" {
        id: "LA-005"
        type: "Logical"
        description: "3D point cloud processing"
        safety_level: "ASIL_C"
        
        function "Process Point Cloud" {
            id: "LF-013"
            description: "Filter and downsample lidar data"
        }
        
        function "Ground Segmentation" {
            id: "LF-014"
            description: "Separate ground from objects"
        }
    }
    
    component "Vehicle Data Interface" {
        id: "LA-006"
        type: "Logical"
        description: "CAN bus interface for vehicle signals"
        safety_level: "ASIL_B"
        
        function "Read Vehicle Signals" {
            id: "LF-015"
            description: "Extract speed, steering, brake status"
        }
    }
    
    component "Object Detection & Tracking" {
        id: "LA-007"
        type: "Logical"
        description: "Multi-object tracking with prediction"
        safety_level: "ASIL_C"
        
        function "Detect Objects" {
            id: "LF-016"
            description: "Identify and classify objects"
        }
        
        function "Track Objects" {
            id: "LF-017"
            description: "Maintain object trajectories"
        }
        
        component "Trajectory Predictor" {
            id: "LA-007-01"
            type: "Logical"
            description: "Future motion prediction"
            
            function "Predict Motion" {
                id: "LF-018"
                description: "Forecast 3-second trajectories"
            }
            
            function "Compute Uncertainty" {
                id: "LF-019"
                description: "Estimate prediction confidence"
            }
        }
    }
    
    component "Risk Assessment Unit" {
        id: "LA-008"
        type: "Logical"
        description: "Collision risk evaluation"
        safety_level: "ASIL_D"
        
        function "Calculate TTC" {
            id: "LF-020"
            description: "Time-to-collision computation"
        }
        
        function "Assess Risk Level" {
            id: "LF-021"
            description: "Classify threat severity"
        }
        
        function "Trigger Decision" {
            id: "LF-022"
            description: "Determine intervention need"
        }
    }
    
    component "Braking Strategy Planner" {
        id: "LA-009"
        type: "Logical"
        description: "Optimal braking strategy calculation"
        safety_level: "ASIL_D"
        
        function "Plan Deceleration" {
            id: "LF-023"
            description: "Calculate brake profile"
        }
        
        function "Optimize Comfort" {
            id: "LF-024"
            description: "Balance safety and passenger comfort"
        }
    }
    
    component "Brake Actuator Controller" {
        id: "LA-010"
        type: "Logical"
        description: "Brake system interface and control"
        safety_level: "ASIL_D"
        
        function "Command Brakes" {
            id: "LF-025"
            description: "Send brake requests to ECU"
        }
        
        function "Monitor Execution" {
            id: "LF-026"
            description: "Verify brake application"
        }
    }
    
    component "HMI Controller" {
        id: "LA-011"
        type: "Logical"
        description: "Driver interface and warnings"
        safety_level: "ASIL_B"
        
        function "Display Warnings" {
            id: "LF-027"
            description: "Show visual alerts"
        }
        
        function "Emit Audio Alerts" {
            id: "LF-028"
            description: "Provide audible warnings"
        }
    }
    
    // Logical Connections (Data Flows)
    connection "LA-003" -> "LA-002" {
        description: "Radar detections to fusion"
        data_type: "RadarObjectList"
        frequency: "20Hz"
    }
    
    connection "LA-004" -> "LA-002" {
        description: "Camera detections to fusion"
        data_type: "VisionObjectList"
        frequency: "30Hz"
    }
    
    connection "LA-005" -> "LA-002" {
        description: "Lidar point cloud to fusion"
        data_type: "PointCloud"
        frequency: "10Hz"
    }
    
    connection "LA-006" -> "LA-002" {
        description: "Vehicle state data"
        data_type: "VehicleState"
        frequency: "100Hz"
    }
    
    connection "LA-002" -> "LA-007" {
        description: "Fused sensor data"
        data_type: "FusedObjectList"
        frequency: "20Hz"
    }
    
    connection "LA-007" -> "LA-008" {
        description: "Tracked objects with predictions"
        data_type: "TrackedObjectList"
        frequency: "20Hz"
    }
    
    connection "LA-008" -> "LA-009" {
        description: "Risk assessment results"
        data_type: "ThreatAssessment"
        frequency: "20Hz"
    }
    
    connection "LA-009" -> "LA-010" {
        description: "Braking command"
        data_type: "BrakeRequest"
        frequency: "50Hz"
    }
    
    connection "LA-008" -> "LA-011" {
        description: "Warning trigger"
        data_type: "WarningLevel"
        frequency: "20Hz"
    }
    
    connection "LA-001" -> "LA-010" {
        description: "System enable/disable"
        data_type: "SystemControl"
        frequency: "10Hz"
    }
    
    connection "LA-002" -> "LA-001" {
        description: "Fusion status"
        data_type: "ComponentHealth"
        frequency: "10Hz"
    }
    
    connection "LA-008" -> "LA-001" {
        description: "Risk assessment status"
        data_type: "ComponentHealth"
        frequency: "10Hz"
    }
}

// ============================================================================
// PHYSICAL ARCHITECTURE (PA Layer)
// Hardware deployment and physical components
// ============================================================================

physical_architecture "Emergency Braking Physical Deployment" {
    
    component "ADAS ECU" {
        id: "PA-001"
        type: "ECU"
        description: "Advanced Driver Assistance System Electronic Control Unit"
        hardware_spec: "NXP S32V234, 4-core ARM Cortex-A53, 4GB RAM"
        power: "12V, 15W typical, 25W peak"
        safety_level: "ASIL_D"
        
        component "Main Processor" {
            id: "PA-001-01"
            type: "Processor"
            description: "Application processing cores"
        }
        
        component "Safety Core" {
            id: "PA-001-02"
            type: "Processor"
            description: "Lockstep safety monitoring core"
        }
    }
    
    component "Front Radar Sensor" {
        id: "PA-002"
        type: "Sensor"
        description: "Long-range 77GHz radar"
        hardware_spec: "Continental ARS540, 250m range, ±45° FOV"
        mounting: "Front bumper center"
        safety_level: "ASIL_B"
    }
    
    component "Stereo Camera" {
        id: "PA-003"
        type: "Sensor"
        description: "Dual camera for depth perception"
        hardware_spec: "Mobileye EyeQ4, 1920x1080 @ 30fps"
        mounting: "Windshield, behind rearview mirror"
        safety_level: "ASIL_B"
    }
    
    component "Lidar Sensor" {
        id: "PA-004"
        type: "Sensor"
        description: "3D scanning lidar"
        hardware_spec: "Velodyne VLP-16, 100m range, 360° horizontal"
        mounting: "Roof center"
        safety_level: "ASIL_B"
    }
    
    component "CAN Gateway" {
        id: "PA-005"
        type: "Gateway"
        description: "Vehicle network interface"
        hardware_spec: "CAN-FD, 5Mbps"
        safety_level: "ASIL_D"
    }
    
    component "Brake Control Module" {
        id: "PA-006"
        type: "Actuator"
        description: "Electronic brake system"
        hardware_spec: "Bosch iBooster, electro-hydraulic"
        safety_level: "ASIL_D"
    }
    
    component "Instrument Cluster" {
        id: "PA-007"
        type: "Display"
        description: "Driver information display"
        hardware_spec: "12.3\" TFT LCD, 1920x720"
        safety_level: "QM"
    }
    
    component "Audio Warning Module" {
        id: "PA-008"
        type: "Actuator"
        description: "Acoustic alert system"
        hardware_spec: "80dB @ 1m, frequency 500-2000Hz"
        safety_level: "QM"
    }
    
    component "Power Supply Unit" {
        id: "PA-009"
        type: "Power"
        description: "Regulated power distribution"
        hardware_spec: "12V input, 5V/3.3V outputs, 50W capacity"
        safety_level: "ASIL_B"
    }
    
    // Physical Connections (Wiring/Bus)
    connection "PA-002" -> "PA-001" {
        description: "Radar data interface"
        protocol: "CAN-FD"
        bandwidth: "5Mbps"
        cable: "Shielded twisted pair"
    }
    
    connection "PA-003" -> "PA-001" {
        description: "Camera video stream"
        protocol: "GMSL"
        bandwidth: "3Gbps"
        cable: "Coaxial"
    }
    
    connection "PA-004" -> "PA-001" {
        description: "Lidar data stream"
        protocol: "Ethernet"
        bandwidth: "100Mbps"
        cable: "Cat6 shielded"
    }
    
    connection "PA-001" -> "PA-005" {
        description: "Vehicle CAN interface"
        protocol: "CAN-FD"
        bandwidth: "5Mbps"
        cable: "Automotive CAN bus"
    }
    
    connection "PA-001" -> "PA-006" {
        description: "Brake control commands"
        protocol: "FlexRay"
        bandwidth: "10Mbps"
        cable: "Automotive FlexRay"
    }
    
    connection "PA-001" -> "PA-007" {
        description: "Display warnings"
        protocol: "LVDS"
        bandwidth: "500Mbps"
        cable: "LVDS cable"
    }
    
    connection "PA-001" -> "PA-008" {
        description: "Audio alert trigger"
        protocol: "Digital I/O"
        bandwidth: "1Kbps"
        cable: "Single wire"
    }
    
    connection "PA-009" -> "PA-001" {
        description: "Power supply"
        voltage: "12V"
        current: "2A max"
        cable: "Power harness"
    }
}
