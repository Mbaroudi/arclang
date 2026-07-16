// Simple test file with correct ArcLang syntax for interfaces

logical_architecture "Test Interfaces" {
    component "Sensor" {
        id: "LC-001"
        type: "Logical"
        description: "Data sensor"
        
        provides "DataOut" {
            protocol: "CAN"
        }
    }
    
    component "Controller" {
        id: "LC-002"
        type: "Logical"
        description: "Control processor"
        
        requires "DataIn" {
            protocol: "CAN"
        }
        
        provides "CommandOut" {
            protocol: "CAN"
        }
    }
    
    component "Actuator" {
        id: "LC-003"
        type: "Logical"
        description: "Execution actuator"
        
        requires "CommandIn" {
            protocol: "CAN"
        }
    }
}
