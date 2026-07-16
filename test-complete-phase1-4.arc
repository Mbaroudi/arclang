// Comprehensive test with all Phase 1-4 features:
// - Interface notation (lollipops/sockets) ✅
// - Physical deployment (HW nodes + SW behavior_components) ✅
// - Component exchanges with protocols
// - Safety borders (ASIL levels)
// - Traceability

operational_analysis "Emergency Braking OA" {
    actor "Driver" {
        id: "OA-ACT-001"
    }
    
    actor "Pedestrian" {
        id: "OA-ACT-002"
    }
}

system_analysis "Emergency Braking SA" {
    system_function "DetectObstacle" {
        id: "SF-001"
        category: "System"
    }
    
    system_function "CalculateBraking" {
        id: "SF-002"
        category: "Control"
    }
    
    system_function "ApplyBrakes" {
        id: "SF-003"
        category: "System"
    }
}

logical_architecture "Emergency Braking LA" {
    component "Radar" {
        id: "LC-001"
        type: "Logical"
        description: "Obstacle detection radar"
        safety_level: "ASIL-D"
        
        provides "ObstacleData" {
            protocol: "CAN"
        }
    }
    
    component "BrakingController" {
        id: "LC-002"
        type: "Logical"
        description: "Emergency braking decision logic"
        safety_level: "ASIL-D"
        
        requires "SensorInput" {
            protocol: "CAN"
        }
        
        provides "BrakeCommand" {
            protocol: "CAN"
        }
    }
    
    component "BrakeActuator" {
        id: "LC-003"
        type: "Logical"
        description: "Hydraulic brake actuator"
        safety_level: "ASIL-D"
        
        requires "CommandInput" {
            protocol: "CAN"
        }
    }
    
}

physical_architecture "Emergency Braking PA" {
    node "ECU_Sensing" {
        type: "Hardware"
        processor: "ARM Cortex-M4"
        memory: "512KB"
        
        behavior_component "Radar_SW" {
            id: "BC-001"
            color: "#4472C4"
            allocated_functions: ["DetectObstacle"]
        }
    }
    
    node "ECU_Control" {
        type: "Hardware"
        processor: "ARM Cortex-M7"
        memory: "2MB"
        
        behavior_component "BrakingControl_SW" {
            id: "BC-002"
            color: "#70AD47"
            allocated_functions: ["CalculateBraking"]
        }
    }
    
    node "ECU_Actuation" {
        type: "Hardware"
        processor: "ARM Cortex-M3"
        memory: "256KB"
        
        behavior_component "BrakeActuator_SW" {
            id: "BC-003"
            color: "#ED7D31"
            allocated_functions: ["ApplyBrakes"]
        }
    }
}

