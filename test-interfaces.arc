// Simple test file with interfaces

logical_architecture "Test Interfaces" {
    component "Sensor" {
        id: "LC-001"
        type: "Logical"
        
        provides "DataOut" {
            protocol: "CAN"
        }
    }
    
    component "Controller" {
        id: "LC-002"
        type: "Logical"
        
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
        
        requires "CommandIn" {
            protocol: "CAN"
        }
    }
    
    connect "Sensor"."DataOut" to "Controller"."DataIn"
    connect "Controller"."CommandOut" to "Actuator"."CommandIn"
}
