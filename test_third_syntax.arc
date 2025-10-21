model "Adaptive Cruise Control System" {
    version: "1.0.0"
    author: "System Architect"
    safety_standard: "ISO26262"
    asil_level: "ASIL_B"
}

// ========================================
// REQUIREMENTS
// ========================================

requirements {
    requirement "REQ-ACC-001" {
        description: "The adaptive cruise control system shall maintain a following distance from the vehicle ahead"
        priority: "Critical"
        safety_level: "ASIL_B"
        type: "Safety"
        verification_method: "Test"
    }

    requirement "REQ-SENS-001" {
        description: "The system shall detect vehicles ahead using radar and camera sensor fusion"
        priority: "Critical"
        safety_level: "ASIL_B"
        type: "Performance"
        verification_method: "Test"
    }
}

// ========================================
// LOGICAL ARCHITECTURE
// ========================================

logical_architecture {
    component "ACC_Controller" {
        id: "LC-ACC-001"
        type: "Logical"
        description: "Main adaptive cruise control controller"
        safety_level: "ASIL_B"

        port "radar_data_in" {
            type: "input"
            data_type: "RadarData"
            rate: "20Hz"
        }

        port "camera_data_in" {
            type: "input"
            data_type: "CameraData"
            rate: "30Hz"
        }

        function "Sensor_Fusion" {
            id: "LF-FUSION-001"
            inputs: ["radar_data_in", "camera_data_in"]
            outputs: ["fused_target_distance", "fused_target_velocity"]
            execution_time: "8ms"
            safety_level: "ASIL_B"
        }

        traces ["REQ-ACC-001", "REQ-SENS-001"]
    }
}
