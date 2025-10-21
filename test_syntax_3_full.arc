// Alternative Syntax 3: top-level blocks
model "Adaptive Cruise Control System" {
    version: "1.0.0"
    author: "System Architect"
}

requirements {
    requirement "REQ-ACC-001" {
        description: "System shall maintain safe distance"
        priority: "Critical"
    }
}

logical_architecture {
    component "ACC_Controller" {
        id: "LC-ACC-001"
        description: "Main controller"
    }
}

traceability {
    trace "REQ-ACC-001" -> "ACC_Controller"
}
