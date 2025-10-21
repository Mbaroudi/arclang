model Test {
    metadata {
        version: "1.0"
    }
}

requirements stakeholder {
    req "REQ-001" "Test Requirement" {
        description: "A test requirement"
    }
}

architecture logical {
    component "TestComponent" {
        id: "COMP-001"
    }
}
