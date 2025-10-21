model TestSystem {
    metadata {
        version: "1.0"
    }
}

requirements stakeholder {
    req "REQ001" "Main Requirement" {
        description: "System shall work"
    }
}

architecture logical {
    component "ComponentA" {
        id: "COMP001"
    }
    
    component "ComponentB" {
        id: "COMP002"
    }
    
    component "ComponentC" {
        id: "COMP003"
    }
    
    connection "ConnAB" {
        from: "COMP001"
        to: "COMP002"
    }
    
    connection "ConnBC" {
        from: "COMP002"
        to: "COMP003"
    }
}
