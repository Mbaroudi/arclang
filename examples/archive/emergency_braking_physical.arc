physical_architecture "Emergency Braking Physical Deployment" {
    
    component "ADAS ECU" {
        id: "PA-001"
        description: "Advanced Driver Assistance System ECU - NXP S32V234"
        safety_level: "ASIL_D"
        
        component "Main Processor" {
            id: "PA-001-01"
            description: "Application cores - 4x ARM Cortex-A53"
        }
        
        component "Safety Core" {
            id: "PA-001-02"
            description: "Lockstep safety monitoring core"
        }
    }
    
    component "Front Radar" {
        id: "PA-002"
        description: "77GHz radar - Continental ARS540, 250m range"
        safety_level: "ASIL_B"
    }
    
    component "Stereo Camera" {
        id: "PA-003"
        description: "Dual camera - Mobileye EyeQ4, 1080p @ 30fps"
        safety_level: "ASIL_B"
    }
    
    component "Lidar Sensor" {
        id: "PA-004"
        description: "3D lidar - Velodyne VLP-16, 100m range"
        safety_level: "ASIL_B"
    }
    
    component "CAN Gateway" {
        id: "PA-005"
        description: "Vehicle network interface - CAN-FD 5Mbps"
        safety_level: "ASIL_D"
    }
    
    component "Brake Module" {
        id: "PA-006"
        description: "Electronic brake - Bosch iBooster"
        safety_level: "ASIL_D"
    }
    
    component "Instrument Cluster" {
        id: "PA-007"
        description: "12.3 inch driver display"
        safety_level: "QM"
    }
    
    component "Audio Warning" {
        id: "PA-008"
        description: "Acoustic alert system - 80dB"
        safety_level: "QM"
    }
    
    component "Power Supply" {
        id: "PA-009"
        description: "12V regulated power - 50W capacity"
        safety_level: "ASIL_B"
    }
    
    connection "PA-002" -> "PA-001" {
        description: "Radar CAN-FD interface"
    }
    
    connection "PA-003" -> "PA-001" {
        description: "Camera GMSL video"
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
