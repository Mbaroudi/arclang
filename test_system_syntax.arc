// ASIL-B Adaptive Cruise Control System
// Safety-critical automotive system following ISO 26262

system "AdaptiveCruiseControlSystem" {
    id: "SYS-ACC-001"
    description: "ASIL-B compliant adaptive cruise control system with emergency braking capability"
    safety_level: "ASIL_B"
    standard: "ISO26262"
    
    // Safety Requirements
    requirements {
        requirement "REQ-ACC-001" {
            id: "REQ-ACC-001"
            description: "The adaptive cruise control system shall maintain a safe following distance behind the preceding vehicle based on current speed and road conditions"
            priority: "Critical"
            safety_level: "ASIL_B"
            type: "Safety"
            verification_method: "Test"
        }
        
        requirement "REQ-ACC-002" {
            id: "REQ-ACC-002"
            description: "The system shall automatically reduce vehicle speed when a slower moving vehicle is detected within the forward detection range"
            priority: "Critical"
            safety_level: "ASIL_B"
            type: "Safety"
            verification_method: "Test"
        }
    }
    
    // Physical Architecture
    physical_architecture {
        component "ForwardRadarSensor" {
            id: "PC-RAD-001"
            type: "Physical"
            description: "77GHz radar sensor for detecting vehicles ahead and measuring distance and relative velocity"
            safety_level: "ASIL_B"
            supplier: "Continental"
            part_number: "ARS408-21"
            
            function "ProcessRadarData" {
                id: "PF-RAD-001"
                inputs: [
                    "radar_raw_signal",
                    "ego_vehicle_speed"
                ]
                outputs: [
                    "target_vehicle_distance",
                    "target_vehicle_velocity"
                ]
                execution_time: "20ms"
                safety_level: "ASIL_B"
            }
        }
    }
    
    // Logical Architecture
    logical_architecture {
        component "AdaptiveCruiseController" {
            id: "LC-ACC-001"
            type: "Logical"
            description: "Main ACC logic for maintaining safe following distance and speed control"
            safety_level: "ASIL_B"
            
            function "CalculateTargetSpeed" {
                id: "LF-ACC-001"
                inputs: [
                    "target_distance",
                    "current_speed"
                ]
                outputs: [
                    "target_acceleration"
                ]
                execution_time: "10ms"
                safety_level: "ASIL_B"
            }
        }
    }
    
    // Traceability
    traces {
        trace "REQ-ACC-001" -> "AdaptiveCruiseController"
        trace "REQ-ACC-002" -> "AdaptiveCruiseController.CalculateTargetSpeed"
    }
}
