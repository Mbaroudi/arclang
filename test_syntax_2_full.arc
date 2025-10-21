// Alternative Syntax 2: system keyword
system "AdaptiveCruiseControlSystem" {
    id: "SYS-ACC-001"
    description: "ASIL-B compliant ACC system"
    safety_level: "ASIL_B"
    
    requirements {
        requirement "REQ-ACC-001" {
            description: "The ACC system shall maintain safe following distance"
            priority: "Critical"
            safety_level: "ASIL_B"
        }
    }
    
    physical_architecture {
        component "RadarSensor" {
            id: "PC-RAD-001"
            description: "77GHz radar sensor"
        }
    }
}
