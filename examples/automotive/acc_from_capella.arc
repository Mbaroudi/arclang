system_analysis "Imported System" {
    requirement "SYS-ACC-002" {
        priority: "Critical"
        description: "Driver brake input shall override ACC"
    }

    requirement "SYS-ACC-003" {
        priority: "High"
        description: "System shall detect cut-in vehicles within 500ms"
    }

    requirement "SYS-ACC-001" {
        priority: "Critical"
        description: "The ACC system shall maintain minimum 2-second following distance"
    }

}

logical_architecture "Imported Architecture" {
    component "ACC Controller" {
        type: "Logical"
        id: "LC-004"
    }

    component "Sensor Fusion" {
        id: "LC-003"
        type: "Logical"
    }

    component "Radar Sensor" {
        type: "Logical"
        id: "LC-001"
    }

    component "Vision Camera" {
        id: "LC-002"
        type: "Logical"
    }

}

trace "LC-001" satisfies "SYS-ACC-001" {
}

trace "LC-003" satisfies "SYS-ACC-002" {
}

trace "LC-004" satisfies "SYS-ACC-003" {
}

