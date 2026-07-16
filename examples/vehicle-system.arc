// Vehicle Control System - Example ArcLang Model

// Data Types
DataType VehicleStatus {
    enumeration [IDLE, ACTIVE, FAULT, MAINTENANCE]
}

DataType CommandType {
    enumeration [START, STOP, PAUSE, RESUME, EMERGENCY_STOP]
}

// Exchange Items (Data Structures)
ExchangeItem Position {
    stereotype: "data"
    attributes {
        latitude: Double = 0.0
        longitude: Double = 0.0
        altitude: Double
    }
}

ExchangeItem Velocity {
    stereotype: "data"
    attributes {
        speed: Double = 0.0
        heading: Double = 0.0
    }
}

ExchangeItem VehicleState {
    stereotype: "data"
    attributes {
        position: Position
        velocity: Velocity
        status: VehicleStatus = IDLE
    }
}

ExchangeItem Command {
    stereotype: "event"
    attributes {
        commandType: CommandType
        timestamp: Long
        priority: Integer = 5
    }
}

// Operational Capabilities
Mission "Autonomous Vehicle Operation" {
    Capability "Navigate to Destination" {
        stereotype: "primary"
        
        SubCapability "Plan Route"
        SubCapability "Follow Route"
        SubCapability "Adapt to Traffic"
    }
    
    Capability "Detect and Avoid Obstacles" {
        stereotype: "safety"
        
        SubCapability "Sense Environment"
        SubCapability "Classify Objects"
        SubCapability "Calculate Avoidance Maneuver"
    }
    
    Capability "Operate Within Safety Constraints" {
        stereotype: "safety"
        
        SubCapability "Monitor System Health"
        SubCapability "Execute Emergency Stop"
        SubCapability "Report Faults"
    }
}

// System Functions
SystemFunction "Vehicle Control" {
    category: System
    icon: "🚗"
    
    SubFunction "Navigation" {
        category: Control
        icon: "🧭"
        
        SubFunction "Route Planning" {
            icon: "📍"
        }
        
        SubFunction "Path Tracking" {
            icon: "📏"
        }
        
        SubFunction "Obstacle Avoidance" {
            icon: "🚧"
        }
    }
    
    SubFunction "Motion Control" {
        category: System
        icon: "⚡"
        
        SubFunction "Throttle Control"
        SubFunction "Steering Control"
        SubFunction "Brake Control"
    }
    
    SubFunction "Sensor Processing" {
        category: Environmental
        icon: "📡"
        
        SubFunction "Camera Processing" {
            icon: "📷"
        }
        SubFunction "LIDAR Processing"
        SubFunction "Radar Processing"
    }
    
    SubFunction "Safety Monitor" {
        category: Management
        icon: "🛡️"
        
        SubFunction "Collision Detection"
        SubFunction "Emergency Stop"
    }
}

// Functional Chain
FunctionalChain "Emergency Stop Scenario" {
    Function "Detect Hazard" {
        category: Environmental
        icon: "⚠️"
        ports {
            in sensor_data: SensorData
            out hazard_detected: HazardEvent
        }
    }
    
    Function "Assess Risk" {
        category: Management
        icon: "🔍"
        ports {
            in hazard: HazardEvent
            out risk_level: RiskLevel
        }
    }
    
    Function "Decide Action" {
        category: Control
        icon: "🎯"
        ports {
            in risk: RiskLevel
            in vehicle_state: VehicleState
            out action_command: ActionCommand
        }
    }
    
    Function "Execute Emergency Stop" {
        category: System
        icon: "🛑"
        ports {
            in action: ActionCommand
            out brake_command: BrakeCommand
        }
    }
    
    Function "Apply Brakes" {
        category: System
        icon: "🔧"
        ports {
            in brake: BrakeCommand
            out brake_status: BrakeStatus
        }
    }
    
    Function "Monitor Stop" {
        category: Management
        icon: "📊"
        ports {
            in brake_status: BrakeStatus
            out stop_confirmed: StopConfirmation
        }
    }
    
    // Flow
    "Detect Hazard" -> "Assess Risk": HazardEvent
    "Assess Risk" -> "Decide Action": RiskLevel
    "Decide Action" -> "Execute Emergency Stop": ActionCommand
    "Execute Emergency Stop" -> "Apply Brakes": BrakeCommand
    "Apply Brakes" -> "Monitor Stop": BrakeStatus
}

// Logical Components
LogicalComponent "Vehicle Controller" {
    type: Logical
    
    SubComponent "Navigation System"
    SubComponent "Motion Control System"
    SubComponent "Sensor Fusion System"
    SubComponent "Safety Manager"
}
