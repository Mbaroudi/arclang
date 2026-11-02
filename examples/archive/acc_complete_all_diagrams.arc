// ============================================================================
// ADAPTIVE CRUISE CONTROL - Complete MBSE Model
// Demonstrates ALL 10 Capella Diagram Types
// ============================================================================

// ============================================================================
// 1. OPERATIONAL ANALYSIS - For Operational Activity Diagrams
// ============================================================================
operational_analysis "Adaptive Cruise Control Operations" {
    actor "Driver" {
        id: "ACT-001"
        description: "Vehicle operator who sets cruise speed and monitors system"
    }
    
    actor "LeadVehicle" {
        id: "ACT-002"
        description: "Vehicle ahead being tracked"
    }
    
    entity "ACC System" {
        id: "ENT-001"
        type: "System"
        description: "Adaptive Cruise Control system"
    }
    
    operational_capability "Maintain Safe Following" {
        id: "OC-001"
        level: "Mission"
        description: "Automatically maintain safe distance from lead vehicle"
    }
    
    operational_activity "Set Cruise Speed" {
        id: "OA-001"
        performed_by: "ACT-001"
        description: "Driver activates ACC and sets desired speed"
    }
    
    operational_activity "Detect Lead Vehicle" {
        id: "OA-002"
        performed_by: "ENT-001"
        description: "System detects and tracks vehicle ahead"
    }
    
    operational_activity "Adjust Speed" {
        id: "OA-003"
        performed_by: "ENT-001"
        description: "System adjusts speed to maintain safe distance"
    }
    
    operational_exchange {
        from: "OA-001"
        to: "OA-002"
        data_type: "SpeedCommand"
        label: "Cruise Speed Set"
    }
    
    operational_exchange {
        from: "OA-002"
        to: "OA-003"
        data_type: "DistanceData"
        label: "Target Distance"
    }
}

// ============================================================================
// 2. DATA MODEL - For Class/Interface Diagrams
// ============================================================================

// Enumerations
data_type "ACCState" {
    base_type: null
    enumeration: [
        "OFF" = 0,
        "STANDBY" = 1,
        "ACTIVE" = 2,
        "OVERRIDDEN" = 3
    ]
}

data_type "VehicleStatus" {
    base_type: null
    enumeration: [
        "MOVING",
        "STOPPED",
        "FAULT"
    ]
}

// Data Structures
exchange_item "VehicleSpeed" {
    stereotype: "data"
    attributes {
        velocity: Double = 0.0
        acceleration: Double = 0.0
        timestamp: Long
    }
}

exchange_item "RadarTarget" {
    stereotype: "data"
    attributes {
        distance: Double
        relative_speed: Double
        angle: Double
        confidence: Integer = 100
    }
}

exchange_item "ACCCommand" {
    stereotype: "event"
    attributes {
        target_speed: Double
        state: ACCState
        timestamp: Long
    }
}

exchange_item "ThrottleCommand" {
    stereotype: "control"
    attributes {
        position_percent: Double
        rate_limit: Double = 10.0
    }
}

exchange_item "BrakeCommand" {
    stereotype: "control"
    attributes {
        pressure_bar: Double
        emergency: Boolean = false
    }
}

// ============================================================================
// 3. CAPABILITY MODEL - For Capability Diagrams
// ============================================================================

mission "Autonomous Speed Control" {
    id: "M-001"
    
    capability "Maintain Safe Following Distance" {
        id: "CAP-001"
        level: "Capability"
        stereotype: "primary"
        
        sub_capability "Detect Lead Vehicle" {
            id: "SC-001"
        }
        
        sub_capability "Calculate Safe Distance" {
            id: "SC-002"
        }
        
        sub_capability "Adjust Vehicle Speed" {
            id: "SC-003"
        }
    }
    
    capability "Handle Driver Override" {
        id: "CAP-002"
        level: "Capability"
        stereotype: "safety"
        
        sub_capability "Detect Brake Pedal" {
            id: "SC-004"
        }
        
        sub_capability "Disengage ACC" {
            id: "SC-005"
        }
    }
    
    capability "Manage System Modes" {
        id: "CAP-003"
        level: "Capability"
        
        sub_capability "Activate ACC" {
            id: "SC-006"
        }
        
        sub_capability "Deactivate ACC" {
            id: "SC-007"
        }
    }
}

capability_association {
    from: "CAP-001"
    to: "CAP-002"
    type: "includes"
    label: "safety override"
}

// ============================================================================
// 4. SYSTEM FUNCTIONS - For Functional Dataflow & Tree Diagrams
// ============================================================================

system_analysis "ACC Functional Architecture" {
    
    system_function "ACC Control" {
        id: "SF-ROOT"
        category: "System"
        icon: "🚗"
        
        sub_function "Sense Environment" {
            id: "SF-001"
            category: "Environmental"
            icon: "📡"
            
            ports {
                in radar_signal: "RawRadarData"
                out targets: "RadarTarget"
            }
            
            sub_function "Process Radar Returns" {
                id: "SF-001-1"
                category: "Environmental"
                icon: "📊"
            }
            
            sub_function "Track Targets" {
                id: "SF-001-2"
                category: "Environmental"
                icon: "🎯"
            }
            
            sub_function "Filter Noise" {
                id: "SF-001-3"
                category: "Environmental"
            }
        }
        
        sub_function "Determine Target Speed" {
            id: "SF-002"
            category: "Control"
            icon: "🧮"
            
            ports {
                in targets: "RadarTarget"
                in current_speed: "VehicleSpeed"
                in desired_speed: "Double"
                out target_speed: "Double"
            }
            
            sub_function "Calculate Time Gap" {
                id: "SF-002-1"
                category: "Control"
            }
            
            sub_function "Determine Speed Adjustment" {
                id: "SF-002-2"
                category: "Control"
            }
        }
        
        sub_function "Control Vehicle Speed" {
            id: "SF-003"
            category: "System"
            icon: "⚙️"
            
            ports {
                in target_speed: "Double"
                in current_speed: "VehicleSpeed"
                out throttle: "ThrottleCommand"
                out brake: "BrakeCommand"
            }
            
            sub_function "PID Controller" {
                id: "SF-003-1"
                category: "Control"
                icon: "📈"
            }
            
            sub_function "Throttle Actuator Control" {
                id: "SF-003-2"
                category: "System"
            }
            
            sub_function "Brake Actuator Control" {
                id: "SF-003-3"
                category: "System"
            }
        }
        
        sub_function "Monitor Driver Input" {
            id: "SF-004"
            category: "Management"
            icon: "👤"
            
            ports {
                in brake_pedal: "Boolean"
                in acc_button: "ACCCommand"
                out override_signal: "Boolean"
            }
            
            sub_function "Detect Brake Press" {
                id: "SF-004-1"
                category: "Management"
            }
            
            sub_function "Process ACC Buttons" {
                id: "SF-004-2"
                category: "Management"
            }
        }
        
        sub_function "Safety Monitor" {
            id: "SF-005"
            category: "Management"
            icon: "🛡️"
            
            sub_function "Check Sensor Health" {
                id: "SF-005-1"
                category: "Management"
            }
            
            sub_function "Validate Commands" {
                id: "SF-005-2"
                category: "Management"
            }
        }
    }
    
    functional_exchange {
        from_port: "SF-001.targets"
        to_port: "SF-002.targets"
        data_type: "RadarTarget"
        label: "Detected Targets"
    }
    
    functional_exchange {
        from_port: "SF-002.target_speed"
        to_port: "SF-003.target_speed"
        data_type: "Double"
        label: "Target Speed"
    }
    
    functional_exchange {
        from_port: "SF-004.override_signal"
        to_port: "SF-003.override"
        data_type: "Boolean"
        label: "Override"
    }
}

// ============================================================================
// 5. FUNCTIONAL CHAIN - For Functional Chain Diagrams
// ============================================================================

functional_chain "ACC Speed Adjustment Scenario" {
    name: "ACC Speed Adjustment"
    
    function "Detect Lead Vehicle" {
        id: "FC-001"
        category: "Environmental"
        icon: "📡"
        ports {
            in radar_returns: "RawRadarData"
            out lead_vehicle: "RadarTarget"
        }
    }
    
    function "Calculate Time Gap" {
        id: "FC-002"
        category: "Control"
        icon: "⏱️"
        ports {
            in lead_vehicle: "RadarTarget"
            in ego_speed: "VehicleSpeed"
            out time_gap: "Double"
        }
    }
    
    function "Determine Speed Command" {
        id: "FC-003"
        category: "Control"
        icon: "🎯"
        ports {
            in time_gap: "Double"
            in desired_gap: "Double"
            out speed_error: "Double"
        }
    }
    
    function "Execute Speed Control" {
        id: "FC-004"
        category: "System"
        icon: "⚙️"
        ports {
            in speed_error: "Double"
            out throttle_cmd: "ThrottleCommand"
            out brake_cmd: "BrakeCommand"
        }
    }
    
    function "Apply Actuators" {
        id: "FC-005"
        category: "System"
        icon: "🔧"
        ports {
            in throttle_cmd: "ThrottleCommand"
            in brake_cmd: "BrakeCommand"
            out vehicle_response: "VehicleSpeed"
        }
    }
    
    function "Monitor Response" {
        id: "FC-006"
        category: "Management"
        icon: "📊"
        ports {
            in vehicle_response: "VehicleSpeed"
            out feedback: "Double"
        }
    }
    
    exchange: "FC-001" -> "FC-002" [data: "RadarTarget", label: "Lead Vehicle Data"]
    exchange: "FC-002" -> "FC-003" [data: "Double", label: "Time Gap"]
    exchange: "FC-003" -> "FC-004" [data: "Double", label: "Speed Error"]
    exchange: "FC-004" -> "FC-005" [data: "ThrottleCommand", label: "Throttle"]
    exchange: "FC-004" -> "FC-005" [data: "BrakeCommand", label: "Brake"]
    exchange: "FC-005" -> "FC-006" [data: "VehicleSpeed", label: "Vehicle Response"]
}

// ============================================================================
// 6. LOGICAL COMPONENTS - For Component Block Diagrams
// ============================================================================

logical_architecture "ACC Logical Components" {
    
    component "ACC Controller" {
        id: "LC-001"
        component_type: "Logical"
        
        allocated_functions: ["SF-002", "SF-003"]
        
        ports {
            in radar_data: "RadarTarget"
            in vehicle_speed: "VehicleSpeed"
            in driver_command: "ACCCommand"
            out throttle: "ThrottleCommand"
            out brake: "BrakeCommand"
        }
        
        sub_component "Speed Planner" {
            id: "LC-001-1"
            component_type: "Logical"
            allocated_functions: ["SF-002"]
        }
        
        sub_component "Speed Controller" {
            id: "LC-001-2"
            component_type: "Logical"
            allocated_functions: ["SF-003"]
        }
    }
    
    component "Radar Sensor System" {
        id: "LC-002"
        component_type: "Logical"
        allocated_functions: ["SF-001"]
        
        ports {
            in rf_signal: "RawRadarData"
            out targets: "RadarTarget"
        }
        
        sub_component "Signal Processor" {
            id: "LC-002-1"
        }
        
        sub_component "Target Tracker" {
            id: "LC-002-2"
        }
    }
    
    component "Driver Interface" {
        id: "LC-003"
        component_type: "Logical"
        allocated_functions: ["SF-004"]
        
        ports {
            in button_press: "Boolean"
            out acc_command: "ACCCommand"
        }
    }
    
    component_exchange {
        from_port: "LC-002.targets"
        to_port: "LC-001.radar_data"
        exchange_item: "RadarTarget"
    }
    
    component_exchange {
        from_port: "LC-003.acc_command"
        to_port: "LC-001.driver_command"
        exchange_item: "ACCCommand"
    }
}

// ============================================================================
// 7. PHYSICAL ARCHITECTURE - For Physical Deployment Diagrams
// ============================================================================

physical_architecture "ACC Hardware Deployment" {
    
    physical_node "Main ECU" {
        id: "PN-001"
        node_type: "Hardware"
        processor: "Infineon AURIX TC397"
        memory: "8MB Flash, 1MB RAM"
        
        behavior_component "ACC Software" {
            id: "BC-001"
            allocated_functions: ["SF-002", "SF-003", "SF-004", "SF-005"]
        }
        
        hardware_component "CAN Controller" {
            id: "HW-001"
            hw_type: "Communication"
            specs: "CAN FD, 5 Mbps"
        }
        
        deployment {
            component: "LC-001"
            node: "PN-001"
        }
    }
    
    physical_node "Radar ECU" {
        id: "PN-002"
        node_type: "Hardware"
        processor: "NXP S32R274"
        memory: "4MB Flash"
        
        behavior_component "Radar Processing" {
            id: "BC-002"
            allocated_functions: ["SF-001"]
        }
        
        hardware_component "77GHz RF Frontend" {
            id: "HW-002"
            hw_type: "Sensor"
            specs: "4 TX, 8 RX channels"
        }
        
        deployment {
            component: "LC-002"
            node: "PN-002"
        }
    }
    
    physical_node "Throttle Actuator" {
        id: "PN-003"
        node_type: "Hardware"
        
        hardware_component "DC Motor" {
            id: "HW-003"
            hw_type: "Actuator"
        }
    }
    
    physical_node "Brake Actuator" {
        id: "PN-004"
        node_type: "Hardware"
        
        hardware_component "Hydraulic Pump" {
            id: "HW-004"
            hw_type: "Actuator"
        }
    }
    
    physical_link {
        from: "PN-001"
        to: "PN-002"
        protocol: "CAN FD"
        bandwidth: "5 Mbps"
    }
    
    physical_link {
        from: "PN-001"
        to: "PN-003"
        protocol: "PWM"
        bandwidth: "100 Hz"
    }
    
    physical_link {
        from: "PN-001"
        to: "PN-004"
        protocol: "CAN"
        bandwidth: "500 kbps"
    }
}

// ============================================================================
// 8. STATE MACHINES - For State Machine Diagrams
// ============================================================================

state_machine "ACC Controller States" {
    name: "ACC_Controller_SM"
    initial_state: "OFF"
    
    state "OFF" {
        entry_actions: ["disable_actuators", "clear_targets"]
        exit_actions: ["log_activation"]
    }
    
    state "STANDBY" {
        entry_actions: ["enable_sensors", "initialize_controllers"]
        exit_actions: []
        
        sub_state "Ready" {
            entry_actions: ["display_ready_indicator"]
        }
        
        sub_state "Waiting for Speed" {
            entry_actions: []
        }
    }
    
    state "ACTIVE" {
        entry_actions: ["engage_control", "notify_driver"]
        exit_actions: ["disengage_control"]
        
        sub_state "Following" {
            entry_actions: ["track_lead_vehicle"]
            internal_transitions: ["update_speed_continuously"]
        }
        
        sub_state "Cruising" {
            entry_actions: ["maintain_set_speed"]
        }
    }
    
    state "OVERRIDDEN" {
        entry_actions: ["suspend_control", "log_override"]
        exit_actions: []
    }
    
    state "FAULT" {
        entry_actions: ["emergency_shutdown", "alert_driver"]
    }
    
    transition {
        from: "OFF"
        to: "STANDBY"
        trigger: "ACC_BUTTON_PRESSED"
        guard: "speed > 30_kph && speed < 180_kph"
        action: "initialize_system"
    }
    
    transition {
        from: "STANDBY"
        to: "ACTIVE"
        trigger: "SET_BUTTON_PRESSED"
        guard: "no_faults"
        action: "capture_current_speed"
    }
    
    transition {
        from: "ACTIVE"
        to: "OVERRIDDEN"
        trigger: "BRAKE_PEDAL_PRESSED"
        guard: null
        action: "immediate_disengage"
    }
    
    transition {
        from: "OVERRIDDEN"
        to: "ACTIVE"
        trigger: "RES_BUTTON_PRESSED"
        guard: "brake_released"
        action: "resume_control"
    }
    
    transition {
        from: "ACTIVE"
        to: "STANDBY"
        trigger: "CANCEL_BUTTON_PRESSED"
        guard: null
        action: null
    }
    
    transition {
        from: "STANDBY"
        to: "OFF"
        trigger: "ACC_BUTTON_PRESSED"
        guard: null
        action: "shutdown"
    }
    
    transition {
        from: "*"
        to: "FAULT"
        trigger: "SENSOR_FAULT_DETECTED"
        guard: null
        action: "emergency_procedure"
        priority: "1"
    }
    
    transition {
        from: "FAULT"
        to: "OFF"
        trigger: "FAULT_CLEARED"
        guard: "system_check_ok"
        action: "reset"
    }
}

// ============================================================================
// 9. SCENARIOS - For Sequence Diagrams
// ============================================================================

scenario "ACC Activation and Speed Adjustment" {
    name: "ACC_Activation_Scenario"
    
    participant "Driver" {
        id: "P-001"
        type: "Actor"
    }
    
    participant "ACC Controller" {
        id: "P-002"
        type: "Component"
    }
    
    participant "Radar Sensor" {
        id: "P-003"
        type: "Component"
    }
    
    participant "Speed Controller" {
        id: "P-004"
        type: "Component"
    }
    
    message {
        from: "P-001"
        to: "P-002"
        label: "Press ACC Button"
        type: "Synchronous"
        activation: true
    }
    
    message {
        from: "P-002"
        to: "P-002"
        label: "Initialize System"
        type: "Synchronous"
        activation: false
    }
    
    message {
        from: "P-002"
        to: "P-003"
        label: "Enable Sensor"
        type: "Asynchronous"
        activation: true
    }
    
    message {
        from: "P-003"
        to: "P-002"
        label: "Sensor Ready"
        type: "Return"
        activation: false
    }
    
    message {
        from: "P-001"
        to: "P-002"
        label: "Press SET Button"
        type: "Synchronous"
        activation: true
    }
    
    message {
        from: "P-002"
        to: "P-004"
        label: "Activate Speed Control"
        type: "Synchronous"
        activation: true
    }
    
    fragment "LOOP" {
        label: "Continuous Control"
        condition: "ACC Active"
        
        message {
            from: "P-003"
            to: "P-002"
            label: "Radar Data"
            type: "Asynchronous"
        }
        
        message {
            from: "P-002"
            to: "P-004"
            label: "Target Speed"
            type: "Asynchronous"
        }
        
        message {
            from: "P-004"
            to: "P-002"
            label: "Speed Adjusted"
            type: "Return"
        }
    }
    
    timing_constraint {
        from_message: "Radar Data"
        to_message: "Speed Adjusted"
        max_duration: "100ms"
        requirement: "SYS-ACC-002"
    }
}

// ============================================================================
// 10. TRACEABILITY
// ============================================================================

trace {
    from: "OC-001"
    to: "SF-001"
    type: "realizes"
}

trace {
    from: "SF-001"
    to: "LC-002"
    type: "allocated_to"
}

trace {
    from: "LC-002"
    to: "PN-002"
    type: "deployed_to"
}
