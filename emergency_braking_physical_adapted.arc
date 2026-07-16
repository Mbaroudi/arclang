logical_architecture "Emergency Braking Physical Deployment" {
    
    component "ADAS ECU" {
        id: "PA-001"
        type: "Physical"
        description: "ADAS ECU - NXP S32V234, 4x ARM Cortex-A53"
        safety_level: "ASIL_D"
        color: "#2E75B6"
        
        component "Main Processor" {
            id: "PA-001-01"
            type: "Physical"
            description: "Application cores"
            color: "#5B9BD5"
        }
        
        component "Safety Core" {
            id: "PA-001-02"
            type: "Physical"
            description: "Lockstep safety core"
            color: "#5B9BD5"
        }
    }
    
    component "Front Radar" {
        id: "PA-002"
        type: "Physical"
        description: "77GHz radar - Continental ARS540"
        safety_level: "ASIL_B"
        color: "#70AD47"
    }
    
    component "Stereo Camera" {
        id: "PA-003"
        type: "Physical"
        description: "Dual camera - Mobileye EyeQ4"
        safety_level: "ASIL_B"
        color: "#70AD47"
    }
    
    component "Lidar Sensor" {
        id: "PA-004"
        type: "Physical"
        description: "3D lidar - Velodyne VLP-16"
        safety_level: "ASIL_B"
        color: "#70AD47"
    }
    
    component "CAN Gateway" {
        id: "PA-005"
        type: "Physical"
        description: "Vehicle network - CAN-FD 5Mbps"
        safety_level: "ASIL_D"
        color: "#FFB900"
    }
    
    component "Brake Module" {
        id: "PA-006"
        type: "Physical"
        description: "Electronic brake - Bosch iBooster"
        safety_level: "ASIL_D"
        color: "#ED7D31"
    }
    
    component "Instrument Cluster" {
        id: "PA-007"
        type: "Physical"
        description: "12.3 inch driver display"
        safety_level: "QM"
        color: "#A5A5A5"
    }
    
    component "Audio Warning" {
        id: "PA-008"
        type: "Physical"
        description: "Acoustic alert - 80dB"
        safety_level: "QM"
        color: "#ED7D31"
    }
    
    component "Power Supply" {
        id: "PA-009"
        type: "Physical"
        description: "12V regulated - 50W"
        safety_level: "ASIL_B"
        color: "#FFC000"
    }
    
    connection "PA-002" -> "PA-001" {
        description: "Radar CAN-FD"
    }
    
    connection "PA-003" -> "PA-001" {
        description: "Camera GMSL"
    }
    
    connection "PA-004" -> "PA-001" {
        description: "Lidar Ethernet"
    }
    
    connection "PA-001" -> "PA-005" {
        description: "Vehicle CAN"
    }
    
    connection "PA-001" -> "PA-006" {
        description: "Brake FlexRay"
    }
    
    connection "PA-001" -> "PA-007" {
        description: "Display LVDS"
    }
    
    connection "PA-001" -> "PA-008" {
        description: "Audio trigger"
    }
    
    connection "PA-009" -> "PA-001" {
        description: "12V power"
    }
    
    connection "PA-009" -> "PA-002" {
        description: "Sensor power"
    }
    
    connection "PA-009" -> "PA-003" {
        description: "Camera power"
    }
    
    connection "PA-009" -> "PA-004" {
        description: "Lidar power"
    }
}
