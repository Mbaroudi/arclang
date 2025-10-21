// Adaptive Cruise Control System - ASIL-B
// ISO 26262 Compliant Model

model AdaptiveCruiseControl {
  metadata {
    version: "1.0.0"
    domain: "Automotive"
    safety_standard: "ISO 26262"
    asil_level: "ASIL_B"
    description: "Adaptive Cruise Control System with distance regulation and speed control"
  }

  // ============================================
  // SYSTEM REQUIREMENTS
  // ============================================
  
  requirements SystemRequirements {
    
    // Functional Requirements
    requirement FR_001 {
      id: "FR_001"
      title: "Distance Regulation"
      description: "The ACC system shall maintain a safe following distance from the vehicle ahead"
      priority: "Critical"
      safety_level: "ASIL_B"
      rationale: "Core ACC functionality for collision avoidance"
    }

    requirement FR_002 {
      id: "FR_002"
      title: "Speed Control"
      description: "The ACC system shall automatically adjust vehicle speed between 30-150 km/h"
      priority: "Critical"
      safety_level: "ASIL_B"
      rationale: "Maintain set speed when no vehicle ahead"
    }

    requirement FR_003 {
      id: "FR_003"
      title: "Time Gap Setting"
      description: "The driver shall be able to set time gap between 1.0-2.5 seconds"
      priority: "High"
      safety_level: "ASIL_B"
      rationale: "Allow driver customization of following distance"
    }
  }

  // ============================================
  // LOGICAL ARCHITECTURE
  // ============================================

  logical_architecture ACCLogicalArchitecture {
    
    // Sensing Function
    component SensingFunction {
      id: "LC_SENSE"
      type: "Logical"
      description: "Detects and tracks target vehicles"
      safety_level: "ASIL_B"
      
      implements: ["FR_004", "SR_002"]
      
      inputs {
        radar_raw_data: "RawRadarData"
        vehicle_speed: "Speed"
      }
      
      outputs {
        target_distance: "Distance"
        target_relative_speed: "Speed"
        target_detected: "Boolean"
        sensor_status: "Status"
      }
      
      functions {
        - "Process radar signals"
        - "Track target vehicles"
        - "Validate sensor data"
        - "Provide sensor diagnostics"
      }
    }

    // Control Function
    component ControlFunction {
      id: "LC_CTRL"
      type: "Logical"
      description: "Calculates required acceleration/deceleration"
      safety_level: "ASIL_B"
      
      implements: ["FR_001", "FR_002", "FR_003", "SR_003", "PR_001"]
      
      inputs {
        target_distance: "Distance"
        target_relative_speed: "Speed"
        target_detected: "Boolean"
        desired_speed: "Speed"
        desired_time_gap: "Time"
        current_speed: "Speed"
        driver_override: "Boolean"
      }
      
      outputs {
        desired_acceleration: "Acceleration"
        acc_active: "Boolean"
        acc_status: "Status"
      }
      
      functions {
        - "Calculate following distance"
        - "Determine required acceleration"
        - "Manage ACC states"
        - "Handle driver override"
        - "Limit deceleration rate"
      }
    }
  }

  // ============================================
  // TRACEABILITY
  // ============================================

  traceability {
    // Functional Requirements to Components
    FR_001 -> [LC_CTRL, PC_ACC_ECU]
    FR_002 -> [LC_CTRL, LC_ACT, PC_ACC_ECU, PC_THROTTLE]
    FR_003 -> [LC_HMI, PC_CLUSTER]
  }
}
