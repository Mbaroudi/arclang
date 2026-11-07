model "Emergency Braking System - Semantic Enhanced" {
  metadata {
    version = "2.0"
    author = "ArcLang Compiler"
    description = "Emergency braking with full Capella metamodel semantics"
    timestamp = "2025-01-28"
  }
  
  operational_analysis "Vehicle Operation" {
    entity "Driver" {
      id = "OA-001"
      type = "OperationalActor"
      description = "Human driver operating the vehicle"
      
      activity "Monitor Road" {
        id = "OA-ACT-001"
        description = "Continuous monitoring of road conditions"
      }
      
      activity "Control Vehicle" {
        id = "OA-ACT-002"
        description = "Manual vehicle control"
      }
    }
    
    entity "Vehicle System" {
      id = "OA-002"
      type = "OperationalEntity"
      description = "Autonomous vehicle system"
      
      activity "Detect Obstacles" {
        id = "OA-ACT-003"
        description = "Sensor-based obstacle detection"
      }
      
      activity "Assess Risk" {
        id = "OA-ACT-004"
        description = "Evaluate collision risk"
      }
      
      activity "Execute Emergency Brake" {
        id = "OA-ACT-005"
        description = "Automatic emergency braking"
      }
    }
    
    capability "Collision Avoidance" {
      id = "CAP-001"
      description = "Prevent collisions through automatic braking"
      level = "critical"
    }
  }
  
  system_analysis "System Requirements" {
    requirement "SR-001" {
      type = "system"
      description = "System shall detect obstacles within 100m"
      priority = "critical"
      safety_level = "ASIL-D"
      category = "safety"
    }
    
    requirement "SR-002" {
      type = "system"
      description = "System shall assess collision risk within 50ms"
      priority = "critical"
      safety_level = "ASIL-D"
      category = "performance"
    }
    
    requirement "SR-003" {
      type = "system"
      description = "System shall initiate braking within 100ms of risk detection"
      priority = "critical"
      safety_level = "ASIL-D"
      category = "safety"
    }
    
    requirement "SR-004" {
      type = "system"
      description = "System shall apply maximum brake force when imminent collision detected"
      priority = "critical"
      safety_level = "ASIL-D"
      category = "safety"
    }
    
    requirement "SR-005" {
      type = "system"
      description = "System shall monitor sensor health continuously"
      priority = "high"
      safety_level = "ASIL-B"
      category = "reliability"
    }
  }
  
  logical_architecture "Logical Components" {
    component "Main Controller" {
      id = "LC-001"
      type = "LogicalComponent"
      level = "Logical"
      asil = "ASIL-D"
      description = "Central control unit for emergency braking"
      
      interface_in {
        name = "Sensor Data Input"
        protocol = "CAN"
        format = "J1939"
      }
      
      interface_in {
        name = "System Status"
        protocol = "CAN"
        format = "Proprietary"
      }
      
      interface_out {
        name = "Brake Commands"
        protocol = "FlexRay"
        format = "AUTOSAR"
      }
      
      interface_out {
        name = "Driver Alert"
        protocol = "LIN"
        format = "Standard"
      }
      
      function "Coordinate System" {
        id = "LF-001"
        type = "LogicalFunction"
        description = "Orchestrate all subsystems"
        inputs = ["raw_sensor_data", "system_health"]
        outputs = ["brake_decision", "alert_signal"]
      }
      
      function "Health Monitor" {
        id = "LF-002"
        type = "LogicalFunction"
        description = "Monitor system health"
        inputs = ["component_status"]
        outputs = ["health_report"]
      }
    }
    
    component "Sensor Fusion Unit" {
      id = "LC-002"
      type = "LogicalComponent"
      level = "Logical"
      asil = "ASIL-D"
      description = "Fuses data from multiple sensors"
      
      interface_in {
        name = "Radar Data"
        protocol = "Ethernet"
        format = "Custom"
      }
      
      interface_in {
        name = "Camera Data"
        protocol = "MIPI CSI-2"
        format = "RAW12"
      }
      
      interface_in {
        name = "Lidar Data"
        protocol = "Ethernet"
        format = "Point Cloud"
      }
      
      interface_out {
        name = "Fused Objects"
        protocol = "CAN"
        format = "J1939"
      }
      
      function "Fuse Sensor Data" {
        id = "LF-003"
        type = "LogicalFunction"
        description = "Combine multi-sensor inputs"
        inputs = ["radar_points", "camera_objects", "lidar_points"]
        outputs = ["fused_environment"]
      }
      
      function "Validate Detections" {
        id = "LF-004"
        type = "LogicalFunction"
        description = "Cross-validate sensor readings"
        inputs = ["fused_environment"]
        outputs = ["validated_objects"]
      }
    }
    
    component "Radar Interface" {
      id = "LC-003"
      type = "LogicalComponent"
      level = "Logical"
      asil = "ASIL-B"
      description = "Interface to radar sensor"
      
      interface_out {
        name = "Radar Output"
        protocol = "Ethernet"
        format = "Custom"
      }
      
      function "Process Radar" {
        id = "LF-005"
        type = "LogicalFunction"
        description = "Process raw radar data"
        inputs = ["radar_raw"]
        outputs = ["radar_objects"]
      }
    }
    
    component "Camera Interface" {
      id = "LC-004"
      type = "LogicalComponent"
      level = "Logical"
      asil = "ASIL-B"
      description = "Interface to camera sensor"
      
      interface_out {
        name = "Camera Output"
        protocol = "MIPI CSI-2"
        format = "RAW12"
      }
      
      function "Process Camera" {
        id = "LF-006"
        type = "LogicalFunction"
        description = "Process camera frames"
        inputs = ["camera_raw"]
        outputs = ["camera_objects"]
      }
    }
    
    component "Lidar Interface" {
      id = "LC-005"
      type = "LogicalComponent"
      level = "Logical"
      asil = "ASIL-B"
      description = "Interface to lidar sensor"
      
      interface_out {
        name = "Lidar Output"
        protocol = "Ethernet"
        format = "Point Cloud"
      }
      
      function "Process Lidar" {
        id = "LF-007"
        type = "LogicalFunction"
        description = "Process lidar point cloud"
        inputs = ["lidar_raw"]
        outputs = ["lidar_objects"]
      }
    }
    
    component "Object Tracker" {
      id = "LC-006"
      type = "LogicalComponent"
      level = "Logical"
      asil = "ASIL-D"
      description = "Track objects over time"
      
      interface_in {
        name = "Object Detections"
        protocol = "CAN"
        format = "J1939"
      }
      
      interface_out {
        name = "Tracked Objects"
        protocol = "CAN"
        format = "J1939"
      }
      
      function "Track Objects" {
        id = "LF-008"
        type = "LogicalFunction"
        description = "Maintain object tracks using Kalman filter"
        inputs = ["detected_objects"]
        outputs = ["tracked_objects"]
      }
      
      function "Predict Trajectories" {
        id = "LF-009"
        type = "LogicalFunction"
        description = "Predict future object positions"
        inputs = ["tracked_objects"]
        outputs = ["predicted_trajectories"]
      }
    }
    
    component "Risk Assessor" {
      id = "LC-007"
      type = "LogicalComponent"
      level = "Logical"
      asil = "ASIL-D"
      description = "Assess collision risk"
      
      interface_in {
        name = "Trajectory Data"
        protocol = "CAN"
        format = "J1939"
      }
      
      interface_out {
        name = "Risk Level"
        protocol = "CAN"
        format = "J1939"
      }
      
      function "Calculate TTC" {
        id = "LF-010"
        type = "LogicalFunction"
        description = "Calculate time to collision"
        inputs = ["ego_trajectory", "object_trajectories"]
        outputs = ["ttc_values"]
      }
      
      function "Assess Risk Level" {
        id = "LF-011"
        type = "LogicalFunction"
        description = "Determine overall risk level"
        inputs = ["ttc_values"]
        outputs = ["risk_assessment"]
      }
    }
    
    component "Brake Controller" {
      id = "LC-008"
      type = "LogicalComponent"
      level = "Logical"
      asil = "ASIL-D"
      description = "Control brake actuators"
      
      interface_in {
        name = "Brake Request"
        protocol = "FlexRay"
        format = "AUTOSAR"
      }
      
      interface_in {
        name = "Supervisor Override"
        protocol = "FlexRay"
        format = "AUTOSAR"
      }
      
      interface_out {
        name = "Brake Actuator"
        protocol = "PWM"
        format = "Direct"
      }
      
      function "Calculate Brake Force" {
        id = "LF-012"
        type = "LogicalFunction"
        description = "Determine required brake force"
        inputs = ["risk_level"]
        outputs = ["brake_force"]
      }
      
      function "Apply Brakes" {
        id = "LF-013"
        type = "LogicalFunction"
        description = "Actuate brake system"
        inputs = ["brake_force"]
        outputs = ["actuator_commands"]
      }
    }
    
    interface "Radar to Fusion" {
      from = "LC-003"
      to = "LC-002"
      exchange_items = ["radar_objects"]
    }
    
    interface "Camera to Fusion" {
      from = "LC-004"
      to = "LC-002"
      exchange_items = ["camera_objects"]
    }
    
    interface "Lidar to Fusion" {
      from = "LC-005"
      to = "LC-002"
      exchange_items = ["lidar_objects"]
    }
    
    interface "Fusion to Tracker" {
      from = "LC-002"
      to = "LC-006"
      exchange_items = ["validated_objects"]
    }
    
    interface "Tracker to Risk" {
      from = "LC-006"
      to = "LC-007"
      exchange_items = ["predicted_trajectories"]
    }
    
    interface "Risk to Brake" {
      from = "LC-007"
      to = "LC-008"
      exchange_items = ["risk_assessment"]
    }
    
    interface "Controller to Brake" {
      from = "LC-001"
      to = "LC-008"
      exchange_items = ["brake_commands", "supervisor_override"]
    }
  }
  
  physical_architecture "Hardware Deployment" {
    node "Central ECU" {
      id = "PA-001"
      type = "NodeComponent"
      processor = "ARM Cortex-A72"
      memory = "8GB"
      description = "Main processing unit"
    }
    
    node "Sensor Hub ECU" {
      id = "PA-002"
      type = "NodeComponent"
      processor = "ARM Cortex-R5"
      memory = "2GB"
      description = "Sensor data aggregation"
    }
    
    node "Brake ECU" {
      id = "PA-003"
      type = "NodeComponent"
      processor = "ARM Cortex-M7"
      memory = "512MB"
      description = "Brake actuation controller"
    }
  }
  
  trace "OA-ACT-003" -> "SR-001" {
    type = "realizes"
    rationale = "Obstacle detection activity realizes detection requirement"
  }
  
  trace "OA-ACT-004" -> "SR-002" {
    type = "realizes"
    rationale = "Risk assessment activity realizes timing requirement"
  }
  
  trace "OA-ACT-005" -> "SR-003" {
    type = "realizes"
    rationale = "Emergency brake activity realizes braking requirement"
  }
  
  trace "SR-001" -> "LC-002" {
    type = "satisfies"
    rationale = "Sensor fusion unit satisfies detection requirement"
  }
  
  trace "SR-002" -> "LC-007" {
    type = "satisfies"
    rationale = "Risk assessor satisfies timing requirement"
  }
  
  trace "SR-003" -> "LC-008" {
    type = "satisfies"
    rationale = "Brake controller satisfies braking timing requirement"
  }
  
  trace "SR-004" -> "LF-012" {
    type = "satisfies"
    rationale = "Brake force calculation satisfies maximum force requirement"
  }
  
  trace "SR-005" -> "LF-002" {
    type = "satisfies"
    rationale = "Health monitor satisfies monitoring requirement"
  }
  
  trace "LC-001" -> "PA-001" {
    type = "deploys_to"
    rationale = "Main controller deployed to central ECU"
  }
  
  trace "LC-002" -> "PA-002" {
    type = "deploys_to"
    rationale = "Sensor fusion deployed to sensor hub"
  }
  
  trace "LC-003" -> "PA-002" {
    type = "deploys_to"
    rationale = "Radar interface deployed to sensor hub"
  }
  
  trace "LC-004" -> "PA-002" {
    type = "deploys_to"
    rationale = "Camera interface deployed to sensor hub"
  }
  
  trace "LC-005" -> "PA-002" {
    type = "deploys_to"
    rationale = "Lidar interface deployed to sensor hub"
  }
  
  trace "LC-008" -> "PA-003" {
    type = "deploys_to"
    rationale = "Brake controller deployed to brake ECU"
  }
}
