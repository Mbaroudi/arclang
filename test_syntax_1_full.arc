// Alternative Syntax 1: logical_architecture without string
model AdaptiveCruiseControl {
  metadata {
    version: "1.0.0"
    domain: "Automotive"
    safety_standard: "ISO 26262"
    asil_level: "ASIL_B"
  }

  requirements SystemRequirements {
    requirement FR_001 {
      id: "FR_001"
      title: "Distance Regulation"
      description: "The ACC system shall maintain safe following distance"
      priority: "Critical"
      safety_level: "ASIL_B"
    }
  }

  logical_architecture ACCLogicalArchitecture {
    component ACC_Controller {
      id: "LC-ACC-001"
      type: "Logical"
      description: "Main adaptive cruise control controller"
      safety_level: "ASIL_B"
    }
  }
}
