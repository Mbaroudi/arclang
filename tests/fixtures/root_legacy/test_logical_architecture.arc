// Logical Architecture - Emergency Braking System
logical_architecture "Emergency Braking Logical Architecture" {
    
    // Logical Components
    component "Sensor Fusion" {
        id: "LA-001"
        description: "Combines data from multiple sensors"
        color: "#A8D08D"
        safety_level: "ASIL_D"
    }
    
    component "Radar Sensor" {
        id: "LA-002"
        description: "77GHz long-range radar"
        color: "#A8D08D"
        safety_level: "ASIL_B"
    }
    
    component "Camera Sensor" {
        id: "LA-003"
        description: "Stereo vision camera"
        color: "#A8D08D"
        safety_level: "ASIL_B"
    }
    
    component "Lidar Sensor" {
        id: "LA-004"
        description: "3D point cloud sensor"
        color: "#A8D08D"
        safety_level: "ASIL_B"
    }
    
    component "Object Tracking" {
        id: "LA-005"
        description: "Tracks detected objects over time"
        color: "#A8D08D"
        safety_level: "ASIL_D"
    }
    
    component "Risk Assessment" {
        id: "LA-006"
        description: "Evaluates collision probability"
        color: "#A8D08D"
        safety_level: "ASIL_D"
    }
    
    component "Decision Logic" {
        id: "LA-007"
        description: "Decides when to brake"
        color: "#A8D08D"
        safety_level: "ASIL_D"
    }
    
    component "Brake Actuator" {
        id: "LA-008"
        description: "Controls braking system"
        color: "#A8D08D"
        safety_level: "ASIL_D"
    }
    
    component "HMI Display" {
        id: "LA-009"
        description: "Driver warning display"
        color: "#A8D08D"
        safety_level: "QM"
    }
    
    // Logical Connections (Component Exchanges)
    connection "LA-002" -> "LA-001" {
        label: "Radar Objects"
        data_type: "ObjectList"
        protocol: "CAN"
    }
    
    connection "LA-003" -> "LA-001" {
        label: "Vision Objects"
        data_type: "ObjectList"
        protocol: "Ethernet"
    }
    
    connection "LA-004" -> "LA-001" {
        label: "Point Cloud"
        data_type: "PointCloud"
        protocol: "Ethernet"
    }
    
    connection "LA-001" -> "LA-005" {
        label: "Fused Objects"
        data_type: "FusedObjectList"
    }
    
    connection "LA-005" -> "LA-006" {
        label: "Tracked Objects"
        data_type: "TrackedObjectList"
    }
    
    connection "LA-006" -> "LA-007" {
        label: "Risk Level"
        data_type: "RiskAssessment"
    }
    
    connection "LA-007" -> "LA-008" {
        label: "Brake Command"
        data_type: "BrakeRequest"
    }
    
    connection "LA-006" -> "LA-009" {
        label: "Warning Signal"
        data_type: "WarningLevel"
    }
}
