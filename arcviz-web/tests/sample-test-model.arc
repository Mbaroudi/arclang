// Comprehensive Test Model for All 7 Dimensions
// Contains all elements needed for complete testing

architecture "Automotive Driver Assistance System" {
  
  // DIMENSION 1: Metamodel Intelligence - Rich component structure
  layer operational {
    actor "Driver" {
      activities: ["Monitor Road", "Control Vehicle", "Receive Warnings"]
    }
    
    actor "VehicleSystem" {
      activities: ["Sense Environment", "Process Data", "Actuate Controls"]
    }
    
    activity "Detect Obstacles" {
      inputs: ["Camera Feed", "Radar Data"]
      outputs: ["Obstacle List"]
    }
    
    activity "Calculate Safe Speed" {
      inputs: ["Obstacle List", "Current Speed"]
      outputs: ["Target Speed"]
      safety_level: "ASIL-D"
    }
    
    activity "Apply Brakes" {
      inputs: ["Target Speed"]
      outputs: ["Brake Command"]
      safety_level: "ASIL-D"
    }
  }
  
  // DIMENSION 2: Constraint Intelligence - Proper spacing and alignment
  layer functional {
    function "ObjectDetection" {
      inputs: ["CameraInterface", "RadarInterface"]
      outputs: ["ObjectData"]
      allocated_to: "SensorFusion"
      safety_level: "ASIL-D"
    }
    
    function "VelocityControl" {
      inputs: ["ObjectData", "VehicleSpeed"]
      outputs: ["BrakeRequest"]
      allocated_to: "BrakingController"
      safety_level: "ASIL-D"
    }
    
    function "SpeedAdjustment" {
      inputs: ["BrakeRequest"]
      outputs: ["ActuatorCommand"]
      allocated_to: "BrakeActuator"
      safety_level: "ASIL-C"
    }
    
    function "WarningGeneration" {
      inputs: ["ObjectData"]
      outputs: ["WarningSignal"]
      allocated_to: "DisplayController"
      safety_level: "ASIL-B"
    }
  }
  
  // DIMENSION 3: Optimization Intelligence - Complex connections
  layer logical {
    component "SensorFusion" {
      type: "SW"
      interfaces_in: ["CameraData", "RadarData", "LidarData"]
      interfaces_out: ["FusedObjects", "EnvironmentMap"]
      allocated_functions: ["ObjectDetection", "SensorCalibration"]
      safety_level: "ASIL-D"
    }
    
    component "BrakingController" {
      type: "SW"
      interfaces_in: ["FusedObjects", "VehicleState"]
      interfaces_out: ["BrakeCommand", "Status"]
      allocated_functions: ["VelocityControl", "EmergencyBraking"]
      safety_level: "ASIL-D"
    }
    
    component "DisplayController" {
      type: "SW"
      interfaces_in: ["WarningData"]
      interfaces_out: ["DisplaySignals"]
      allocated_functions: ["WarningGeneration", "UIRendering"]
      safety_level: "ASIL-B"
    }
    
    component "VehicleBus" {
      type: "SW"
      interfaces_in: ["AllComponentData"]
      interfaces_out: ["RoutedData"]
      allocated_functions: ["DataRouting", "MessageFiltering"]
      safety_level: "ASIL-D"
    }
    
    // DIMENSION 4: Routing Intelligence - Multiple connection types
    connection "SensorFusion" -> "BrakingController" {
      protocol: "CAN"
      data: "ObjectList"
    }
    
    connection "BrakingController" -> "VehicleBus" {
      protocol: "FlexRay"
      data: "BrakeCommands"
    }
    
    connection "SensorFusion" -> "DisplayController" {
      protocol: "Ethernet"
      data: "WarningEvents"
    }
  }
  
  // DIMENSION 5: Hierarchy Intelligence - Nested components
  layer physical {
    node "ADAS_ECU" {
      type: "Processor"
      deployed_components: ["SensorFusion", "BrakingController"]
      safety_level: "ASIL-D"
      
      subnode "ProcessorCore1" {
        deployed_components: ["SensorFusion"]
        frequency: "2.0 GHz"
      }
      
      subnode "ProcessorCore2" {
        deployed_components: ["BrakingController"]
        frequency: "2.0 GHz"
      }
    }
    
    node "Display_ECU" {
      type: "Display"
      deployed_components: ["DisplayController"]
      safety_level: "ASIL-B"
    }
    
    node "Gateway_ECU" {
      type: "Gateway"
      deployed_components: ["VehicleBus"]
      safety_level: "ASIL-D"
    }
    
    hardware_connection "ADAS_ECU" -> "Gateway_ECU" {
      bus: "CAN"
      bandwidth: "1 Mbps"
    }
    
    hardware_connection "Gateway_ECU" -> "Display_ECU" {
      bus: "LIN"
      bandwidth: "20 kbps"
    }
  }
  
  // DIMENSION 6: Safety & Regulatory Intelligence
  safety_requirements {
    requirement "REQ-001" {
      description: "System shall detect obstacles within 100m"
      safety_level: "ASIL-D"
      standard: "ISO 26262"
      verification: "Test"
    }
    
    requirement "REQ-002" {
      description: "Braking response time shall be < 100ms"
      safety_level: "ASIL-D"
      standard: "ISO 26262"
      verification: "Analysis"
    }
    
    requirement "REQ-003" {
      description: "Warning shall be displayed within 50ms"
      safety_level: "ASIL-B"
      standard: "ISO 26262"
      verification: "Test"
    }
  }
  
  // Traceability links
  trace "REQ-001" -> "ObjectDetection"
  trace "REQ-002" -> "VelocityControl"
  trace "REQ-003" -> "WarningGeneration"
  
  trace "ObjectDetection" -> "SensorFusion"
  trace "VelocityControl" -> "BrakingController"
  trace "WarningGeneration" -> "DisplayController"
  
  trace "SensorFusion" -> "ADAS_ECU.ProcessorCore1"
  trace "BrakingController" -> "ADAS_ECU.ProcessorCore2"
  trace "DisplayController" -> "Display_ECU"
  
  // DIMENSION 7: Aesthetic Intelligence - Rich visual elements
  style {
    color_scheme: "professional"
    grid_alignment: true
    shadow_depth: 4
    corner_radius: 6
    font_family: "Helvetica Neue"
    anti_aliasing: true
  }
  
  metadata {
    version: "1.0"
    author: "Test Suite"
    date: "2025-01-28"
    description: "Comprehensive test model for 7 dimensions validation"
  }
}
