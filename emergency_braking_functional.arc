system_analysis "Emergency Braking System Functions" {
    
    system_function "Sensor Data Acquisition" {
        id: "SF-001"
        description: "Collect data from all vehicle sensors"
        category: "Input"
        
        function "Acquire Radar Data" {
            id: "SF-001-01"
            description: "Read 77GHz radar object detections at 20Hz"
        }
        
        function "Acquire Camera Data" {
            id: "SF-001-02"
            description: "Process stereo camera images at 30Hz"
        }
        
        function "Acquire Lidar Data" {
            id: "SF-001-03"
            description: "Read 3D point cloud data at 10Hz"
        }
        
        function "Read Vehicle State" {
            id: "SF-001-04"
            description: "Get speed, steering, brake status at 100Hz"
        }
    }
    
    system_function "Multi-Sensor Fusion" {
        id: "SF-002"
        description: "Fuse and validate sensor data"
        category: "Processing"
        
        function "Validate Sensor Data" {
            id: "SF-002-01"
            description: "Check data integrity and plausibility"
        }
        
        function "Temporal Synchronization" {
            id: "SF-002-02"
            description: "Align data to common timestamp"
        }
        
        function "Spatial Registration" {
            id: "SF-002-03"
            description: "Transform to vehicle coordinates"
        }
        
        function "Kalman Filtering" {
            id: "SF-002-04"
            description: "Fuse multi-modal sensor data"
        }
    }
    
    system_function "Object Detection & Classification" {
        id: "SF-003"
        description: "Identify and classify environmental objects"
        category: "Perception"
        
        function "Segment Objects" {
            id: "SF-003-01"
            description: "Separate objects from background"
        }
        
        function "Classify Object Types" {
            id: "SF-003-02"
            description: "Identify vehicles, pedestrians, cyclists"
        }
        
        function "Estimate Dimensions" {
            id: "SF-003-03"
            description: "Calculate bounding boxes and sizes"
        }
    }
    
    system_function "Multi-Object Tracking" {
        id: "SF-004"
        description: "Track objects over time"
        category: "Perception"
        
        function "Associate Detections" {
            id: "SF-004-01"
            description: "Match new detections to tracks"
        }
        
        function "Update Tracks" {
            id: "SF-004-02"
            description: "Update position and velocity"
        }
        
        function "Predict Trajectories" {
            id: "SF-004-03"
            description: "Forecast 3-second motion"
        }
        
        function "Compute Uncertainty" {
            id: "SF-004-04"
            description: "Estimate prediction confidence"
        }
    }
    
    system_function "Collision Risk Assessment" {
        id: "SF-005"
        description: "Evaluate collision threat level"
        category: "Decision"
        
        function "Calculate TTC" {
            id: "SF-005-01"
            description: "Compute time-to-collision"
        }
        
        function "Evaluate Threat Level" {
            id: "SF-005-02"
            description: "Classify as safe, warning, critical"
        }
        
        function "Decide Intervention" {
            id: "SF-005-03"
            description: "Determine if braking needed"
        }
    }
    
    system_function "Braking Strategy Planning" {
        id: "SF-006"
        description: "Calculate optimal brake response"
        category: "Planning"
        
        function "Select Strategy" {
            id: "SF-006-01"
            description: "Choose partial or full braking"
        }
        
        function "Calculate Profile" {
            id: "SF-006-02"
            description: "Compute deceleration trajectory"
        }
        
        function "Optimize Comfort" {
            id: "SF-006-03"
            description: "Balance safety and smoothness"
        }
    }
    
    system_function "Brake Actuation" {
        id: "SF-007"
        description: "Execute braking commands"
        category: "Control"
        
        function "Send Brake Commands" {
            id: "SF-007-01"
            description: "Interface with brake ECU"
        }
        
        function "Monitor Response" {
            id: "SF-007-02"
            description: "Verify actual deceleration"
        }
        
        function "Handle Driver Override" {
            id: "SF-007-03"
            description: "Respond to manual control"
        }
    }
    
    system_function "Driver Warning" {
        id: "SF-008"
        description: "Alert driver of hazards"
        category: "HMI"
        
        function "Display Visual Alerts" {
            id: "SF-008-01"
            description: "Show warnings on cluster"
        }
        
        function "Emit Audio Alerts" {
            id: "SF-008-02"
            description: "Provide audible warnings"
        }
        
        function "Haptic Feedback" {
            id: "SF-008-03"
            description: "Vibrate steering wheel"
        }
    }
    
    system_function "System Health Monitoring" {
        id: "SF-009"
        description: "Monitor safety and availability"
        category: "Safety"
        
        function "Check Sensor Status" {
            id: "SF-009-01"
            description: "Verify sensors operational"
        }
        
        function "Diagnose Faults" {
            id: "SF-009-02"
            description: "Detect component failures"
        }
        
        function "Report Status" {
            id: "SF-009-03"
            description: "Communicate health to driver"
        }
    }
}
