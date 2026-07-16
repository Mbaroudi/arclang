model "ACC Test" {
  metadata {
    name: "Test ACC"
    version: "1.0"
  }

  operational_analysis "ACC Operations" {
    actor "Driver" {
      id: "A-001"
      description: "Human driver"
    }
  }

  system_analysis "ACC Requirements" {
    requirement "Maintain Speed" {
      id: "REQ-001"
      description: "Maintain set speed"
    }
  }

  logical_architecture "ACC System" {
    component "Long Range Radar" {
      id: "LC-001"
      description: "Front radar sensor"
      
      function "Transmit RF Signal" {
        id: "LF-001"
        description: "Generate radar pulses"
      }
      
      function "Receive Reflected Signal" {
        id: "LF-002"
        description: "Detect reflected signals"
      }
    }
    
    component "Control Unit" {
      id: "LC-002"
      description: "Main controller"
      
      function "Process Radar Data" {
        id: "LF-003"
        description: "Analyze sensor data"
      }
    }
  }
}
