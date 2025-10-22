// ============================================================================
// REMOTE START FUNCTION - Complete Arcadia Architecture
// ============================================================================
// System: Vehicle Remote Start Function (ICE/Hybrid/Electric)
// Standard: Arcadia MBSE Methodology
// Safety: ISO 26262 ASIL B
// Security: ISO/SAE 21434
// Version: 1.0.0
// ============================================================================

model RemoteStartSystem {
    metadata {
        version: "1.0.0"
        description: "Complete Arcadia architecture for vehicle remote start function"
        domain: "automotive"
        safety_standard: "ISO_26262"
        security_standard: "ISO_SAE_21434"
        project_phase: "system_design"
    }
}

// ============================================================================
// OPERATIONAL ANALYSIS LAYER
// Stakeholder Requirements - Actors, Use Cases, Constraints
// ============================================================================

requirements stakeholder {
    req "STK-RS-001" "Remote Start from Smartphone" {
        description: "User must be able to remotely start the vehicle from their smartphone within 100m range"
        priority: Critical
    }
    
    req "STK-RS-002" "Safety Condition Enforcement" {
        description: "System must prevent remote start if vehicle safety conditions are not met"
        priority: Critical
        safety_level: ASIL_B
    }
    
    req "STK-RS-003" "Multi-Powertrain Support" {
        description: "Remote start must work for ICE, Hybrid, and Electric vehicles with appropriate powertrain control"
        priority: Critical
    }
    
    req "STK-RS-004" "Secure Authentication" {
        description: "System must authenticate user and secure all communications to prevent unauthorized access"
        priority: Critical
    }
    
    req "STK-RS-005" "Regulatory Compliance" {
        description: "Vehicle must comply with regional emission and safety regulations during remote start"
        priority: High
    }
    
    req "STK-RS-006" "User Status Confirmation" {
        description: "User shall receive confirmation of remote start status within 3 seconds"
        priority: High
    }
    
    req "STK-RS-007" "Automatic Shutdown" {
        description: "System must automatically shut down after 10 minutes if not manually overridden"
        priority: Medium
    }
    
    req "STK-RS-008" "Climate Pre-Conditioning" {
        description: "Remote start must maintain cabin comfort through climate control pre-conditioning"
        priority: Medium
    }
}

// ============================================================================
// SYSTEM ANALYSIS LAYER  
// System Requirements - Functional and Non-Functional
// ============================================================================

requirements system {
    req "SYS-RS-001" "Cryptographic Authentication" {
        description: "System shall authenticate user identity using cryptographic tokens before allowing remote start"
        priority: Critical
        safety_level: ASIL_B
    }
    
    req "SYS-RS-002" "Safety Interlock Verification" {
        description: "System shall verify all safety interlocks (parking brake, neutral, doors closed) before starting"
        priority: Critical
        safety_level: ASIL_B
    }
    
    req "SYS-RS-003" "Secure Communication Channel" {
        description: "System shall establish secure encrypted communication channel between smartphone and vehicle"
        priority: Critical
    }
    
    req "SYS-RS-004" "Battery State Monitoring" {
        description: "System shall monitor battery state of charge and prevent start if below 20 percent for EV or voltage below 11V for ICE"
        priority: High
    }
    
    req "SYS-RS-005" "Powertrain Type Detection" {
        description: "System shall activate appropriate powertrain control strategy based on vehicle type ICE Hybrid or EV"
        priority: Critical
    }
    
    req "SYS-RS-006" "Status Notification Service" {
        description: "System shall send status notifications success failure warnings to user smartphone"
        priority: High
    }
    
    req "SYS-RS-007" "Automatic Shutdown Timer" {
        description: "System shall implement automatic shutdown timer with maximum 10 minute duration"
        priority: High
    }
    
    req "SYS-RS-008" "Climate Control Activation" {
        description: "System shall activate climate control to target temperature based on user preferences"
        priority: Medium
    }
    
    req "SYS-RS-009" "Audit Trail Logging" {
        description: "System shall log all remote start attempts with timestamp and result for audit trail"
        priority: Medium
    }
    
    req "SYS-RS-010" "Security Attack Detection" {
        description: "System shall detect tampering or replay attacks and raise security alerts"
        priority: Critical
    }
}

requirements functional {
    req "FUNC-RS-001" "Fast Credential Validation" {
        description: "Remote start request processing function shall validate user credentials within 500ms"
        priority: Critical
    }
    
    req "FUNC-RS-002" "Rapid Safety Checks" {
        description: "Safety interlock validation function shall check all preconditions within 200ms"
        priority: Critical
        safety_level: ASIL_B
    }
    
    req "FUNC-RS-003" "Powertrain Start Sequence" {
        description: "Powertrain control function shall initiate start sequence based on vehicle type"
        priority: Critical
    }
    
    req "FUNC-RS-004" "Reliable Notifications" {
        description: "Notification service shall deliver status updates with 99.9 percent reliability"
        priority: High
    }
}

// ============================================================================
// LOGICAL ARCHITECTURE LAYER
// Logical Components, Functions, and Logical Interfaces
// ============================================================================

architecture logical {
    component "Smartphone Application" {
        id: "LA-USER-001"
        layer: "User"
        stereotype: "Application"
        description: "Mobile application for remote vehicle control"
        
        interface_out: "RemoteStartRequest" {
            protocol: "HTTPS"
            format: "JSON"
        }
        
        interface_in: "StartConfirmation" {
            protocol: "HTTPS"
            format: "JSON"
        }
        
        function "displayUI" {
            description: "Display user interface for remote start"
        }
        
        function "sendRemoteStartRequest" {
            description: "Send encrypted remote start request"
        }
    }
    
    component "User Authentication Service" {
        id: "LA-USER-002"
        layer: "User"
        stereotype: "Service"
        description: "Cryptographic authentication and session management"
        
        interface_out: "TokenValidation" {
            protocol: "OAuth"
            format: "JWT"
        }
        
        function "generateAuthToken" {
            description: "Generate cryptographic authentication tokens"
        }
        
        function "validateBiometric" {
            description: "Validate user biometric data"
        }
    }
    
    component "Telematics Control Unit" {
        id: "LA-CONN-001"
        layer: "Connectivity"
        stereotype: "Gateway"
        safety_level: "ASIL_B"
        description: "Cellular gateway routing commands to vehicle"
        
        interface_in: "RemoteCommandIn" {
            protocol: "LTE"
            format: "Encrypted"
        }
        
        interface_out: "CANCommandOut" {
            protocol: "CAN"
            format: "Binary"
        }
        
        function "receiveRemoteCommands" {
            description: "Receive commands via cellular network"
        }
        
        function "routeToVehicleBus" {
            description: "Route commands to CAN bus"
        }
    }
    
    component "Secure Communication Manager" {
        id: "LA-CONN-002"
        layer: "Connectivity"
        stereotype: "Security"
        description: "Security validation and attack detection"
        
        interface_in: "SecurityValidation" {
            protocol: "Internal"
            format: "Encrypted"
        }
        
        function "verifyCryptographicSignature" {
            description: "Verify cryptographic signatures"
        }
        
        function "detectReplayAttacks" {
            description: "Detect replay attacks using nonce"
        }
    }
    
    component "Cloud Backend Service" {
        id: "LA-CONN-003"
        layer: "Connectivity"
        stereotype: "Service"
        description: "Cloud service for request queuing and audit"
        
        interface_in: "CloudRequestIn" {
            protocol: "HTTPS"
            format: "JSON"
        }
        
        interface_out: "CloudCommandOut" {
            protocol: "MQTT"
            format: "JSON"
        }
        
        function "queueRequests" {
            description: "Queue remote start requests"
        }
        
        function "storeAuditLogs" {
            description: "Store audit logs for compliance"
        }
    }
    
    component "Remote Start Controller" {
        id: "LA-CTRL-001"
        layer: "Control"
        stereotype: "Controller"
        safety_level: "ASIL_B"
        description: "Orchestrates remote start sequence"
        
        interface_in: "StartVehicleCommand" {
            protocol: "CAN"
            format: "Binary"
        }
        
        interface_out: "PowertrainStartCommand" {
            protocol: "CAN"
            format: "Binary"
        }
        
        function "orchestrateStartSequence" {
            description: "Orchestrate remote start sequence"
        }
        
        function "validateSafetyPreconditions" {
            description: "Validate all safety preconditions"
        }
    }
    
    component "Safety Interlock Validator" {
        id: "LA-CTRL-002"
        layer: "Control"
        stereotype: "SafetyFunction"
        safety_level: "ASIL_B"
        description: "Validates all safety preconditions"
        
        interface_in: "SensorInputs" {
            protocol: "CAN"
            format: "Binary"
        }
        
        interface_out: "InterlockStatus" {
            protocol: "Internal"
            format: "Boolean"
        }
        
        function "checkTransmissionPosition" {
            description: "Check transmission in Park or Neutral"
        }
        
        function "verifyParkingBrake" {
            description: "Verify parking brake engaged"
        }
    }
    
    component "Powertrain Start Manager" {
        id: "LA-CTRL-003"
        layer: "Control"
        stereotype: "Controller"
        safety_level: "ASIL_B"
        description: "Manages ICE, Hybrid, and EV start sequences"
        
        interface_in: "PowertrainControl" {
            protocol: "CAN"
            format: "Binary"
        }
        
        interface_out: "EngineStart" {
            protocol: "CAN"
            format: "Binary"
        }
        
        function "detectPowertrainType" {
            description: "Detect vehicle powertrain type"
        }
        
        function "executeStartSequence" {
            description: "Execute appropriate start sequence"
        }
    }
    
    component "Climate Control Pre-Conditioner" {
        id: "LA-CTRL-004"
        layer: "Control"
        stereotype: "Controller"
        description: "Activates HVAC system for cabin comfort"
        
        interface_in: "ClimateActivate" {
            protocol: "LIN"
            format: "Binary"
        }
        
        interface_out: "HVACControl" {
            protocol: "LIN"
            format: "Binary"
        }
        
        function "readClimatePreferences" {
            description: "Read user climate preferences"
        }
        
        function "activateHVAC" {
            description: "Activate HVAC system"
        }
    }
    
    component "Timer and Shutdown Manager" {
        id: "LA-CTRL-005"
        layer: "Control"
        stereotype: "Controller"
        safety_level: "ASIL_B"
        description: "Manages 10-minute auto shutdown timer"
        
        interface_out: "ShutdownCommand" {
            protocol: "Internal"
            format: "Binary"
        }
        
        function "startCountdownTimer" {
            description: "Start 10 minute countdown timer"
        }
        
        function "executeAutoShutdown" {
            description: "Execute automatic shutdown"
        }
    }
    
    component "Engine Control Unit" {
        id: "LA-VHC-001"
        layer: "Vehicle"
        stereotype: "ECU"
        safety_level: "ASIL_B"
        description: "Controls fuel injection and ignition"
        
        interface_in: "ICEStartCommand" {
            protocol: "CAN"
            format: "Binary"
        }
        
        interface_out: "EngineStatus" {
            protocol: "CAN"
            format: "Binary"
        }
        
        function "controlFuelInjection" {
            description: "Control fuel injection timing"
        }
        
        function "crankEngine" {
            description: "Crank engine via starter motor"
        }
    }
    
    component "Battery Management System" {
        id: "LA-VHC-002"
        layer: "Vehicle"
        stereotype: "ECU"
        safety_level: "ASIL_B"
        description: "Manages high voltage battery state"
        
        interface_in: "EVStartCommand" {
            protocol: "CAN_FD"
            format: "Binary"
        }
        
        interface_out: "BatteryStatus" {
            protocol: "CAN_FD"
            format: "Binary"
        }
        
        function "monitorBatteryState" {
            description: "Monitor HV battery state"
        }
        
        function "enableContactors" {
            description: "Enable battery contactors"
        }
    }
    
    component "Hybrid Control Unit" {
        id: "LA-VHC-003"
        layer: "Vehicle"
        stereotype: "ECU"
        safety_level: "ASIL_B"
        description: "Coordinates engine and electric motor"
        
        interface_in: "HybridStartCommand" {
            protocol: "CAN_FD"
            format: "Binary"
        }
        
        interface_out: "HybridModeStatus" {
            protocol: "CAN_FD"
            format: "Binary"
        }
        
        function "coordinateHybridPowertrain" {
            description: "Coordinate engine and electric motor"
        }
        
        function "selectOperatingMode" {
            description: "Select optimal hybrid mode"
        }
    }
    
    component "HVAC Control Module" {
        id: "LA-VHC-004"
        layer: "Vehicle"
        stereotype: "Module"
        description: "Controls heating and cooling systems"
        
        interface_in: "HVACActivate" {
            protocol: "LIN"
            format: "Binary"
        }
        
        interface_out: "CabinTemperature" {
            protocol: "LIN"
            format: "Analog"
        }
        
        function "controlCompressor" {
            description: "Control compressor and blower"
        }
        
        function "adjustTemperature" {
            description: "Adjust air temperature"
        }
    }
    
    component "Body Control Module" {
        id: "LA-VHC-005"
        layer: "Vehicle"
        stereotype: "Module"
        description: "Monitors doors, brake, and sensors"
        
        interface_out: "SensorData" {
            protocol: "CAN"
            format: "Binary"
        }
        
        function "monitorDoorStatus" {
            description: "Monitor door lock status"
        }
        
        function "readParkingBrake" {
            description: "Read parking brake sensor"
        }
    }
    
    component "Instrument Cluster" {
        id: "LA-VHC-006"
        layer: "Vehicle"
        stereotype: "Display"
        description: "Displays remote start status to driver"
        
        interface_in: "DisplayCommand" {
            protocol: "CAN"
            format: "Binary"
        }
        
        function "displayRemoteStartIndicator" {
            description: "Display remote start active"
        }
        
        function "showCountdownTimer" {
            description: "Show countdown timer"
        }
    }
    
    // ========================================================================
    // INTERFACE CONNECTIONS - Logical Component Interactions
    // ========================================================================
    
    interface "User to Backend" {
        from: "LA-USER-001"
        to: "LA-USER-002"
        description: "Remote start request from smartphone to backend"
    }
    
    interface "Backend to TCU" {
        from: "LA-USER-002"
        to: "LA-CONN-001"
        description: "Cloud command to vehicle TCU"
    }
    
    interface "TCU to Security" {
        from: "LA-CONN-001"
        to: "LA-CONN-002"
        description: "Command validation request"
    }
    
    interface "TCU to Auth Service" {
        from: "LA-CONN-001"
        to: "LA-CONN-003"
        description: "Token validation"
    }
    
    interface "TCU to Remote Start Controller" {
        from: "LA-CONN-001"
        to: "LA-CTRL-001"
        description: "Validated start command"
    }
    
    interface "Remote Start to Safety Monitor" {
        from: "LA-CTRL-001"
        to: "LA-CTRL-002"
        description: "Safety interlock check request"
    }
    
    interface "Remote Start to Vehicle State Manager" {
        from: "LA-CTRL-001"
        to: "LA-CTRL-003"
        description: "Vehicle state query"
    }
    
    interface "Remote Start to Power Management" {
        from: "LA-CTRL-001"
        to: "LA-CTRL-004"
        description: "Power activation command"
    }
    
    interface "Remote Start to Climate Control" {
        from: "LA-CTRL-001"
        to: "LA-CTRL-005"
        description: "Climate pre-conditioning command"
    }
    
    interface "Power Management to ECM" {
        from: "LA-CTRL-004"
        to: "LA-VHC-001"
        description: "Engine start command"
    }
    
    interface "Power Management to BMS" {
        from: "LA-CTRL-004"
        to: "LA-VHC-002"
        description: "EV/Hybrid start command"
    }
    
    interface "Climate Control to HVAC" {
        from: "LA-CTRL-005"
        to: "LA-VHC-004"
        description: "HVAC activation command"
    }
    
    interface "Safety Monitor to Sensors" {
        from: "LA-CTRL-002"
        to: "LA-VHC-005"
        description: "Sensor data request"
    }
    
    interface "Vehicle State to Display" {
        from: "LA-CTRL-003"
        to: "LA-VHC-006"
        description: "Status display update"
    }
    
    interface "ECM to Display" {
        from: "LA-VHC-001"
        to: "LA-VHC-006"
        description: "Engine status to display"
    }
    
    interface "BMS to Display" {
        from: "LA-VHC-002"
        to: "LA-VHC-006"
        description: "Battery status to display"
    }
}

// ============================================================================
// PHYSICAL ARCHITECTURE LAYER
// Hardware/Software Allocation, Network Topology
// ============================================================================

architecture physical {
    component "Telematics Control Unit Hardware" {
        id: "PA-ECU-001"
        layer: "Physical"
        stereotype: "ECU_Hardware"
        description: "LTE/5G modem with secure element"
    }
    
    component "Gateway ECU" {
        id: "PA-ECU-002"
        layer: "Physical"
        stereotype: "ECU_Hardware"
        safety_level: "ASIL_C"
        description: "CAN gateway with firewall"
    }
    
    component "Engine Control Module Hardware" {
        id: "PA-ECU-003"
        layer: "Physical"
        stereotype: "ECU_Hardware"
        safety_level: "ASIL_D"
        description: "High-speed ECU for engine control"
    }
    
    component "Battery Management System Hardware" {
        id: "PA-ECU-004"
        layer: "Physical"
        stereotype: "ECU_Hardware"
        safety_level: "ASIL_D"
        description: "HV battery monitoring and protection"
    }
    
    component "Body Control Module Hardware" {
        id: "PA-ECU-005"
        layer: "Physical"
        stereotype: "ECU_Hardware"
        description: "LIN master for body electronics"
    }
    
    component "TCU Application Software" {
        id: "PA-SW-001"
        layer: "Physical"
        stereotype: "Software"
        description: "AUTOSAR Adaptive with LTE stack"
    }
    
    component "Remote Start Controller Software" {
        id: "PA-SW-002"
        layer: "Physical"
        stereotype: "Software"
        safety_level: "ASIL_B"
        description: "AUTOSAR Classic safety controller"
    }
    
    component "Engine Control Software" {
        id: "PA-SW-003"
        layer: "Physical"
        stereotype: "Software"
        safety_level: "ASIL_D"
        description: "Fuel and ignition control algorithms"
    }
    
    component "BMS Control Software" {
        id: "PA-SW-004"
        layer: "Physical"
        stereotype: "Software"
        safety_level: "ASIL_D"
        description: "Battery monitoring and balancing"
    }
}


// ============================================================================
// SAFETY AND SECURITY REQUIREMENTS
// ISO 26262 and ISO/SAE 21434
// ============================================================================

requirements safety {
    req "SAFE-RS-001" "Watchdog Monitoring" {
        description: "Remote start controller shall implement watchdog monitoring with 100ms timeout"
        priority: Critical
        safety_level: ASIL_B
    }
    
    req "SAFE-RS-002" "Interlock Failure Handling" {
        description: "Safety interlock failure shall inhibit remote start and log fault code"
        priority: Critical
        safety_level: ASIL_B
    }
    
    req "SAFE-RS-003" "Fault State Inhibition" {
        description: "Remote start shall be inhibited if any ASIL B or higher fault is active"
        priority: Critical
        safety_level: ASIL_B
    }
}

requirements security {
    req "SEC-RS-001" "AES Encryption" {
        description: "All remote commands shall use AES 256 encryption with perfect forward secrecy"
        priority: Critical
    }
    
    req "SEC-RS-002" "Certificate Authentication" {
        description: "System shall implement certificate based authentication with hardware backed key storage"
        priority: Critical
    }
    
    req "SEC-RS-003" "Replay Attack Detection" {
        description: "Replay attack detection shall use cryptographic nonce with maximum 5 second window"
        priority: Critical
    }
    
    req "SEC-RS-004" "Exponential Backoff" {
        description: "Failed authentication attempts shall trigger exponential backoff after 3 failures"
        priority: High
    }
}

requirements regulatory {
    req "REG-RS-001" "FCC Compliance" {
        description: "System shall comply with FCC Part 15 for radiated emissions"
        priority: Critical
    }
    
    req "REG-RS-002" "Idle Time Compliance" {
        description: "Remote start duration shall comply with local idle time regulations maximum 10 minutes"
        priority: High
    }
    
    req "REG-RS-003" "GDPR Privacy" {
        description: "System shall comply with GDPR for user data privacy and consent"
        priority: High
    }
    
    req "REG-RS-004" "UNECE R100 Compliance" {
        description: "Remote start shall comply with UNECE R100 for electric vehicle safety"
        priority: High
    }
}

// ============================================================================
// TRACEABILITY MATRIX
// Requirements to Components/Functions mapping
// ============================================================================

trace "STK-RS-001" satisfies "SYS-RS-001" {}
trace "STK-RS-001" satisfies "FUNC-RS-001" {}
trace "STK-RS-002" satisfies "SYS-RS-002" {}
trace "STK-RS-002" satisfies "FUNC-RS-002" {}
trace "STK-RS-003" satisfies "SYS-RS-005" {}
trace "STK-RS-004" satisfies "SYS-RS-001" {}
trace "STK-RS-004" satisfies "SYS-RS-003" {}
trace "STK-RS-005" satisfies "SYS-RS-009" {}
trace "STK-RS-006" satisfies "SYS-RS-006" {}
trace "STK-RS-007" satisfies "SYS-RS-007" {}
trace "STK-RS-008" satisfies "SYS-RS-008" {}



