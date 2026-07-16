// Test physical architecture with behavior_component nesting

physical_architecture "Deployment Test" {
    node "ECU_Controller" {
        type: "Hardware"
        processor: "ARM Cortex-M4"
        
        behavior_component "Sensor_SW" {
            id: "BC-001"
            color: "#4472C4"
            allocated_functions: ["ReadSensor", "FilterData"]
        }
        
        behavior_component "Control_SW" {
            id: "BC-002"
            color: "#70AD47"
            allocated_functions: ["ProcessControl", "DecisionLogic"]
        }
    }
    
    node "ECU_Actuator" {
        type: "Hardware"
        processor: "ARM Cortex-M3"
        
        behavior_component "Actuator_SW" {
            id: "BC-003"
            color: "#ED7D31"
            allocated_functions: ["DriveActuator"]
        }
    }
}
