// Simple operational test
operational_analysis "Simple Test" {
    actor "Driver" {
        id: "OA-ACT-001"
        description: "Human driver"
    }
    
    actor "System" {
        id: "OA-ACT-002"
        description: "Vehicle system"
    }
    
    operational_activity "Monitor" {
        id: "OA-01"
        performed_by: "OA-ACT-001"
    }
    
    operational_activity "Control" {
        id: "OA-02"
        performed_by: "OA-ACT-002"
    }
    
    exchange "OA-01" -> "OA-02" {
        description: "Driver input"
        data_type: "Command"
    }
}
