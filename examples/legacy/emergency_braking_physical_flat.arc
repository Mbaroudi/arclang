physical_architecture "Emergency Braking Physical Deployment" {
    
    node "ADAS ECU" {
        id: "PA-001"
        description: "Main computing platform - NXP S32V234, 4-core ARM, 4GB RAM, ASIL-D"
    }
    
    node "Front Radar Sensor" {
        id: "PA-002"
        description: "77GHz long-range radar - Continental ARS540, 250m range, ASIL-B"
    }
    
    node "Stereo Camera Unit" {
        id: "PA-003"
        description: "Dual camera system - Mobileye EyeQ4, 1080p@30fps, ASIL-B"
    }
    
    node "Lidar Sensor" {
        id: "PA-004"
        description: "3D scanning lidar - Velodyne VLP-16, 100m range, 360° FOV, ASIL-B"
    }
    
    node "CAN Gateway" {
        id: "PA-005"
        description: "Vehicle network interface - CAN-FD 5Mbps, ASIL-D"
    }
    
    node "Brake Control Module" {
        id: "PA-006"
        description: "Electronic brake system - Bosch iBooster electro-hydraulic, ASIL-D"
    }
    
    node "Instrument Cluster" {
        id: "PA-007"
        description: "Driver display - 12.3 inch TFT LCD 1920x720, QM"
    }
    
    node "Audio Warning System" {
        id: "PA-008"
        description: "Acoustic alerts - 80dB @ 1m, 500-2000Hz, QM"
    }
    
    node "Power Distribution Unit" {
        id: "PA-009"
        description: "Power supply - 12V input, 5V/3.3V outputs, 50W capacity, ASIL-B"
    }
    
    connection "PA-002" -> "PA-001" {
        description: "Radar data via CAN-FD 5Mbps"
    }
    
    connection "PA-003" -> "PA-001" {
        description: "Video stream via GMSL 3Gbps"
    }
    
    connection "PA-004" -> "PA-001" {
        description: "Point cloud via Ethernet 100Mbps"
    }
    
    connection "PA-001" -> "PA-005" {
        description: "Vehicle data via CAN-FD"
    }
    
    connection "PA-001" -> "PA-006" {
        description: "Brake commands via FlexRay 10Mbps"
    }
    
    connection "PA-001" -> "PA-007" {
        description: "Display output via LVDS 500Mbps"
    }
    
    connection "PA-001" -> "PA-008" {
        description: "Audio trigger via digital I/O"
    }
    
    connection "PA-009" -> "PA-001" {
        description: "12V 2A power supply"
    }
    
    connection "PA-009" -> "PA-002" {
        description: "12V sensor power"
    }
    
    connection "PA-009" -> "PA-003" {
        description: "12V camera power"
    }
    
    connection "PA-009" -> "PA-004" {
        description: "12V lidar power"
    }
}
