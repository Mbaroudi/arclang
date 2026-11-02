// ============================================================================
// REMOTE START FUNCTION - Complete Arcadia Architecture
// ============================================================================
// System: Vehicle Remote Start Function (ICE / Hybrid / Electric)
// Method: Arcadia, operational need down to the ECUs
// Safety: ISO 26262, ASIL B on the start inhibition path
// Security: ISO/SAE 21434, threats on the remote command
//
//   arclang check examples/automotive/remote_start/remote_start_architecture.arc
//   arclang gate  examples/automotive/remote_start/remote_start_architecture.arc
// ============================================================================

model RemoteStartSystem {
    metadata {
        version: "2.0.0"
        description: "Complete Arcadia architecture of a vehicle remote start function"
        domain: "automotive"
        safety_standard: "ISO 26262"
        security_standard: "ISO/SAE 21434"
        project_phase: "system_design"
    }
}

// ============================================================================
// STAKEHOLDER REQUIREMENTS
// ============================================================================

requirements stakeholder {
    req "STK-RS-001" "Remote Start from Smartphone" {
        description: "The user shall be able to start the vehicle from a smartphone, from anywhere with network coverage"
        priority: "Critical"
    }
    req "STK-RS-002" "Safety Condition Enforcement" {
        description: "The vehicle shall not start remotely unless it is safe to do so"
        priority: "Critical"
        safety_level: "ASIL-B"
    }
    req "STK-RS-003" "Multi-Powertrain Support" {
        description: "Remote start shall work for ICE, hybrid and electric vehicles with the appropriate powertrain control"
        priority: "Critical"
    }
    req "STK-RS-004" "Secure Authentication" {
        description: "Only the authenticated owner shall be able to start the vehicle; all communications shall be protected"
        priority: "Critical"
    }
    req "STK-RS-005" "Regulatory Compliance" {
        description: "A remotely started vehicle shall comply with the regional emission and safety regulations"
        priority: "High"
    }
    req "STK-RS-006" "User Status Confirmation" {
        description: "The user shall receive a confirmation of the remote start result within 3 s"
        priority: "High"
    }
    req "STK-RS-007" "Automatic Shutdown" {
        description: "A remotely started vehicle shall shut down by itself after 10 minutes unless the driver takes over"
        priority: "Medium"
    }
    req "STK-RS-008" "Climate Pre-Conditioning" {
        description: "Remote start shall bring the cabin to the comfort temperature the user chose"
        priority: "Medium"
    }
}

// ============================================================================
// SYSTEM REQUIREMENTS
// ============================================================================

requirements system {
    req "SYS-RS-001" "Cryptographic Authentication" {
        description: "The system shall authenticate the user with cryptographic tokens, within 500 ms, before any remote start"
        priority: "Critical"
        safety_level: "ASIL-B"
    }
    req "SYS-RS-002" "Safety Interlock Verification" {
        description: "The system shall verify every safety interlock (parking brake, park or neutral, doors and hood closed) within 200 ms before starting"
        priority: "Critical"
        safety_level: "ASIL-B"
    }
    req "SYS-RS-003" "Secure Communication Channel" {
        description: "The system shall carry the remote command over an encrypted, authenticated channel between the smartphone and the vehicle"
        priority: "Critical"
    }
    req "SYS-RS-004" "Battery State Monitoring" {
        description: "The system shall refuse the start when the traction battery is below 20 percent (EV) or the 12 V battery below 11 V"
        priority: "High"
    }
    req "SYS-RS-005" "Powertrain Type Detection" {
        description: "The system shall apply the start strategy of the installed powertrain: ICE, hybrid or EV"
        priority: "Critical"
    }
    req "SYS-RS-006" "Status Notification Service" {
        description: "The system shall notify the smartphone of success, failure and warnings, with 99.9 percent delivery"
        priority: "High"
    }
    req "SYS-RS-007" "Automatic Shutdown Timer" {
        description: "The system shall stop the powertrain 10 minutes after a remote start unless the driver has taken over"
        priority: "High"
    }
    req "SYS-RS-008" "Climate Control Activation" {
        description: "The system shall drive the HVAC to the target temperature stored in the user preferences"
        priority: "Medium"
    }
    req "SYS-RS-009" "Audit Trail Logging" {
        description: "The system shall log every remote start attempt with its timestamp and result"
        priority: "Medium"
    }
    req "SYS-RS-010" "Security Attack Detection" {
        description: "The system shall detect tampered and replayed commands and raise a security alert"
        priority: "Critical"
    }
}

requirements safety {
    req "SAFE-RS-001" "Watchdog Monitoring" {
        description: "The remote start controller shall be supervised by a watchdog with a 100 ms timeout"
        priority: "Critical"
        safety_level: "ASIL-B"
    }
    req "SAFE-RS-002" "Interlock Failure Handling" {
        description: "A failed or unavailable interlock reading shall inhibit the remote start and log a fault code"
        priority: "Critical"
        safety_level: "ASIL-B"
    }
    req "SAFE-RS-003" "Fault State Inhibition" {
        description: "Remote start shall be inhibited while any ASIL B or higher fault is active in the vehicle"
        priority: "Critical"
        safety_level: "ASIL-B"
    }
}

requirements security {
    req "SEC-RS-001" "Command Encryption" {
        description: "Every remote command shall be encrypted with AES-256 and perfect forward secrecy"
        priority: "Critical"
    }
    req "SEC-RS-002" "Certificate Authentication" {
        description: "The vehicle shall authenticate the backend with certificates whose keys live in a hardware secure element"
        priority: "Critical"
    }
    req "SEC-RS-003" "Replay Attack Detection" {
        description: "A command whose nonce is older than 5 s or already seen shall be rejected"
        priority: "Critical"
    }
    req "SEC-RS-004" "Exponential Backoff" {
        description: "After 3 failed authentications the service shall back off exponentially"
        priority: "High"
    }
}

requirements regulatory {
    req "REG-RS-001" "FCC Compliance" {
        description: "The telematics unit shall comply with FCC Part 15 for radiated emissions"
        priority: "Critical"
    }
    req "REG-RS-002" "Idle Time Compliance" {
        description: "The remote start duration shall respect the local idle time regulations (10 minutes at most)"
        priority: "High"
    }
    req "REG-RS-003" "GDPR Privacy" {
        description: "User data and remote start logs shall be handled with GDPR consent and retention rules"
        priority: "High"
    }
    req "REG-RS-004" "UNECE R100 Compliance" {
        description: "The electric vehicle start sequence shall comply with UNECE R100"
        priority: "High"
    }
}

// Requirement refinement: system requirements refine the stakeholder needs
trace "SYS-RS-001" refines "STK-RS-004"
trace "SYS-RS-003" refines "STK-RS-004"
trace "SYS-RS-002" refines "STK-RS-002"
trace "SYS-RS-004" refines "STK-RS-002"
trace "SYS-RS-005" refines "STK-RS-003"
trace "SYS-RS-006" refines "STK-RS-006"
trace "SYS-RS-007" refines "STK-RS-007"
trace "SYS-RS-008" refines "STK-RS-008"
trace "SYS-RS-009" refines "STK-RS-005"
trace "SYS-RS-010" refines "STK-RS-004"
trace "REG-RS-002" refines "STK-RS-005"
trace "REG-RS-004" refines "STK-RS-005"

// ============================================================================
// OPERATIONAL ANALYSIS
// ============================================================================

operational_analysis "Remote Start Operations" {
    actor "Vehicle User" {
        id: "OA-USER"
        description: "Owner or authorized driver with the smartphone application"
    }
    actor "Mobility Service Operator" {
        id: "OA-OPERATOR"
        description: "Runs the connected-car backend and answers security alerts"
    }
    actor "Regulator" {
        id: "OA-REGULATOR"
        description: "Sets the idle time, emission and privacy rules"
    }
    entity "Connected Vehicle" {
        id: "OA-VEHICLE"
        activity "Receive the start request" { id: "OA-RECEIVE" }
        activity "Check the vehicle is safe to start" { id: "OA-CHECK" }
        activity "Start the powertrain" { id: "OA-START" }
        activity "Pre-condition the cabin" { id: "OA-CABIN" }
        activity "Confirm the status" { id: "OA-CONFIRM" }
        activity "Shut down after the idle time" { id: "OA-SHUTDOWN" }
    }
    operational_capability "Start the vehicle remotely" {
        id: "OC-RS-001"
        description: "The user starts the parked vehicle from a distance and finds it ready and comfortable"
        involves: ["OA-USER", "OA-OPERATOR", "OA-VEHICLE"]
    }
    interaction "Remote start request" { id: "OI-RS-001" from: "OA-USER" to: "OA-RECEIVE" }
    interaction "Status confirmation" { id: "OI-RS-002" from: "OA-CONFIRM" to: "OA-USER" }
    interaction "Security alerts" { id: "OI-RS-003" from: "OA-VEHICLE" to: "OA-OPERATOR" }
    interaction "Idle time rules" { id: "OI-RS-004" from: "OA-REGULATOR" to: "OA-SHUTDOWN" }
}

// ============================================================================
// SYSTEM ANALYSIS
// ============================================================================

system_analysis "Remote Start System" {
    actor "Vehicle User" { id: "SA-USER" }
    actor "Mobility Service Operator" { id: "SA-OPERATOR" }
    system_component "Remote Start Function" {
        id: "SC-RS"
        description: "The system under design: from the smartphone request to the running, pre-conditioned vehicle"
    }
    capability "Remote start" {
        id: "CAP-RS-001"
        realizes: "OC-RS-001"
        involves: ["SF-AUTH", "SF-ILK", "SF-START", "SF-NOTIF"]
    }

    function "Authenticate the request" { id: "SF-AUTH" latency: 300 ms port in "request" port out "authenticated" }
    function "Secure the channel" { id: "SF-SEC" description: "Encrypt, authenticate and check the freshness of every command" }
    function "Verify the safety interlocks" { id: "SF-ILK" latency: 150 ms safety_level: "ASIL-B" port in "authenticated" port in "vehicle_state" port out "cleared" }
    function "Check the battery state" { id: "SF-BAT" description: "Refuse the start on a low traction or 12 V battery" port out "vehicle_state" }
    function "Select the powertrain strategy" { id: "SF-PWT" description: "ICE, hybrid or EV start sequence" }
    function "Start the powertrain" { id: "SF-START" latency: 1500 ms safety_level: "ASIL-B" port in "cleared" port out "running" }
    function "Pre-condition the cabin" { id: "SF-CLIM" port in "running" }
    function "Notify the user" { id: "SF-NOTIF" latency: 500 ms port in "running" }
    function "Run the shutdown timer" { id: "SF-TIMER" description: "Ten minutes, then stop unless the driver took over" }
    function "Log the audit trail" { id: "SF-LOG" }
    function "Detect attacks" { id: "SF-IDS" description: "Tampering and replay on the remote command" }

    functional_exchange "authenticated request" { id: "FE-RS-001" from: "SF-AUTH.authenticated" to: "SF-ILK.authenticated" }
    functional_exchange "battery and interlock state" { id: "FE-RS-002" from: "SF-BAT.vehicle_state" to: "SF-ILK.vehicle_state" }
    functional_exchange "start cleared" { id: "FE-RS-003" from: "SF-ILK.cleared" to: "SF-START.cleared" }
    functional_exchange "powertrain running" { id: "FE-RS-004" from: "SF-START.running" to: "SF-NOTIF.running" }
    functional_exchange "cabin pre-conditioning" { id: "FE-RS-005" from: "SF-START.running" to: "SF-CLIM.running" }

    functional_chain "Request to confirmation" {
        id: "FC-RS-001"
        description: "From the authenticated request to the confirmation on the smartphone"
        involves: ["SF-AUTH", "SF-ILK", "SF-START", "SF-NOTIF"]
        latency_budget: 3 s
    }
}

// ============================================================================
// LOGICAL ARCHITECTURE
// ============================================================================

architecture logical {
    // ---- User side ---------------------------------------------------------
    component "Smartphone Application" {
        id: "LA-USER-001"
        layer: "User"
        description: "Mobile application for remote vehicle control"
        port out "request" { protocol: "HTTPS" }
        port in "confirmation" { protocol: "HTTPS" }
        function "Display the remote start screen" { id: "LF-UI" }
        function "Send the remote start request" { id: "LF-SEND" realizes: "SF-SEC" }
    }
    component "User Authentication Service" {
        id: "LA-USER-002"
        layer: "User"
        description: "Cryptographic authentication and session management in the cloud"
        port in "credentials"
        port out "token"
        function "Issue authentication tokens" { id: "LF-TOKEN" realizes: "SF-AUTH" }
        function "Validate biometric and password factors" { id: "LF-BIO" realizes: "SF-AUTH" }
        function "Back off after failed attempts" { id: "LF-BACKOFF" }
    }
    // ---- Connectivity ------------------------------------------------------
    component "Cloud Backend Service" {
        id: "LA-CONN-003"
        layer: "Connectivity"
        description: "Queues the requests, forwards the commands, keeps the audit trail"
        port in "request" { protocol: "HTTPS" }
        port out "credentials"
        port in "token"
        port out "command" { protocol: "MQTT over TLS" }
        port in "status" { protocol: "MQTT over TLS" }
        port out "confirmation" { protocol: "HTTPS" }
        function "Queue the remote start requests" { id: "LF-QUEUE" }
        function "Store the audit log" { id: "LF-AUDIT" realizes: "SF-LOG" }
        function "Deliver the status notification" { id: "LF-PUSH" realizes: "SF-NOTIF" }
    }
    component "Telematics Control Unit" {
        id: "LA-CONN-001"
        layer: "Connectivity"
        safety_level: "ASIL-B"
        description: "Cellular gateway of the vehicle"
        port in "command" { protocol: "MQTT over TLS" }
        port out "status" { protocol: "MQTT over TLS" }
        port out "raw_command"
        port in "validated_status"
        function "Receive the remote commands" { id: "LF-RX" realizes: "SF-SEC" }
        function "Report the vehicle status" { id: "LF-TX" realizes: "SF-NOTIF" }
    }
    component "Secure Communication Manager" {
        id: "LA-CONN-002"
        layer: "Connectivity"
        description: "Signature, freshness and attack detection on every command"
        port in "raw_command"
        port out "validated_command" { protocol: "CAN FD" }
        port in "controller_status"
        port out "validated_status"
        function "Verify the cryptographic signature" { id: "LF-SIG" realizes: "SF-SEC" }
        function "Detect replay attacks" { id: "LF-REPLAY" realizes: "SF-IDS" }
    }
    // ---- Control -----------------------------------------------------------
    component "Remote Start Controller" {
        id: "LA-CTRL-001"
        layer: "Control"
        safety_level: "ASIL-B"
        description: "Orchestrates the remote start sequence"
        port in "validated_command" { protocol: "CAN FD" }
        port out "interlock_request"
        port in "interlock_status"
        port out "start_request"
        port in "powertrain_status"
        port out "climate_request" { protocol: "LIN" }
        port out "timer_start"
        port in "shutdown"
        port out "controller_status"
        port out "display" { protocol: "CAN" }
        function "Orchestrate the start sequence" { id: "LF-ORCH" realizes: "SF-START" }
        function "Gate the start on the preconditions" { id: "LF-GATE" realizes: "SF-ILK" }
        function "Service the watchdog" { id: "LF-WDG" }
    }
    component "Safety Interlock Validator" {
        id: "LA-CTRL-002"
        layer: "Control"
        safety_level: "ASIL-B"
        description: "Checks every safety precondition before a start"
        port in "interlock_request"
        port in "body_state" { protocol: "CAN" }
        port in "battery_state" { protocol: "CAN FD" }
        port out "interlock_status"
        function "Check the transmission position" { id: "LF-GEAR" realizes: "SF-ILK" }
        function "Verify the parking brake and the doors" { id: "LF-BRAKE" realizes: "SF-ILK" }
        function "Check the battery state" { id: "LF-BATT" realizes: "SF-BAT" }
    }
    component "Powertrain Start Manager" {
        id: "LA-CTRL-003"
        layer: "Control"
        safety_level: "ASIL-B"
        description: "Runs the ICE, hybrid or EV start sequence"
        port in "start_request"
        port out "ice_start" { protocol: "CAN" }
        port out "ev_start" { protocol: "CAN FD" }
        port out "hybrid_start" { protocol: "CAN FD" }
        port in "engine_status" { protocol: "CAN" }
        port in "battery_status" { protocol: "CAN FD" }
        port in "hybrid_status" { protocol: "CAN FD" }
        port out "powertrain_status"
        function "Detect the powertrain type" { id: "LF-PWT" realizes: "SF-PWT" }
        function "Execute the start sequence" { id: "LF-EXEC" realizes: "SF-START" }
    }
    component "Climate Pre-Conditioner" {
        id: "LA-CTRL-004"
        layer: "Control"
        description: "Brings the cabin to the stored comfort temperature"
        port in "climate_request" { protocol: "LIN" }
        port out "hvac_command" { protocol: "LIN" }
        function "Read the climate preferences" { id: "LF-PREF" realizes: "SF-CLIM" }
        function "Activate the HVAC" { id: "LF-HVAC" realizes: "SF-CLIM" }
    }
    component "Timer and Shutdown Manager" {
        id: "LA-CTRL-005"
        layer: "Control"
        safety_level: "ASIL-B"
        description: "Ten-minute shutdown timer"
        port in "timer_start"
        port out "shutdown"
        function "Run the countdown" { id: "LF-COUNT" realizes: "SF-TIMER" }
        function "Order the automatic shutdown" { id: "LF-STOP" realizes: "SF-TIMER" }
    }
    // ---- Vehicle -----------------------------------------------------------
    component "Engine Control Unit" {
        id: "LA-VHC-001"
        layer: "Vehicle"
        safety_level: "ASIL-B"
        description: "Fuel injection, ignition and cranking of the combustion engine"
        port in "ice_start" { protocol: "CAN" }
        port out "engine_status" { protocol: "CAN" }
        function "Crank the engine" { id: "LF-CRANK" realizes: "SF-START" }
        function "Control fuel injection and ignition" { id: "LF-FUEL" }
    }
    component "Battery Management System" {
        id: "LA-VHC-002"
        layer: "Vehicle"
        safety_level: "ASIL-B"
        description: "State and contactors of the traction battery"
        port in "ev_start" { protocol: "CAN FD" }
        port out "battery_status" { protocol: "CAN FD" }
        port out "battery_state" { protocol: "CAN FD" }
        function "Monitor the battery state" { id: "LF-SOC" realizes: "SF-BAT" }
        function "Close the contactors" { id: "LF-CONTACT" realizes: "SF-START" }
    }
    component "Hybrid Control Unit" {
        id: "LA-VHC-003"
        layer: "Vehicle"
        safety_level: "ASIL-B"
        description: "Coordinates the engine and the electric motor"
        port in "hybrid_start" { protocol: "CAN FD" }
        port out "hybrid_status" { protocol: "CAN FD" }
        function "Coordinate the hybrid powertrain" { id: "LF-HYB" realizes: "SF-START" }
        function "Select the operating mode" { id: "LF-MODE" }
    }
    component "HVAC Control Module" {
        id: "LA-VHC-004"
        layer: "Vehicle"
        description: "Heating, ventilation and air conditioning"
        port in "hvac_command" { protocol: "LIN" }
        function "Control the compressor and the blower" { id: "LF-COMP" realizes: "SF-CLIM" }
        function "Regulate the cabin temperature" { id: "LF-TEMP" }
    }
    component "Body Control Module" {
        id: "LA-VHC-005"
        layer: "Vehicle"
        safety_level: "ASIL-B"
        description: "Doors, hood, parking brake and gear position sensors"
        port out "body_state" { protocol: "CAN" }
        function "Monitor the doors and the hood" { id: "LF-DOORS" realizes: "SF-ILK" }
        function "Read the parking brake and the gear position" { id: "LF-PARK" realizes: "SF-ILK" }
    }
    component "Instrument Cluster" {
        id: "LA-VHC-006"
        layer: "Vehicle"
        description: "Shows the remote start state and the countdown to the driver"
        port in "display" { protocol: "CAN" }
        function "Display the remote start indicator" { id: "LF-IND" realizes: "SF-NOTIF" }
        function "Show the countdown" { id: "LF-CD" realizes: "SF-TIMER" }
    }

    // ---- Component exchanges -----------------------------------------------
    component_exchange "Remote start request" { from_port: "LA-USER-001.request" to_port: "LA-CONN-003.request" protocol: "HTTPS" }
    component_exchange "Credential check" { from_port: "LA-CONN-003.credentials" to_port: "LA-USER-002.credentials" }
    component_exchange "Authentication token" { from_port: "LA-USER-002.token" to_port: "LA-CONN-003.token" }
    component_exchange "Remote command" { from_port: "LA-CONN-003.command" to_port: "LA-CONN-001.command" protocol: "MQTT over TLS" }
    component_exchange "Vehicle status" { from_port: "LA-CONN-001.status" to_port: "LA-CONN-003.status" protocol: "MQTT over TLS" }
    component_exchange "Start confirmation" { from_port: "LA-CONN-003.confirmation" to_port: "LA-USER-001.confirmation" protocol: "HTTPS" }
    component_exchange "Command to validate" { from_port: "LA-CONN-001.raw_command" to_port: "LA-CONN-002.raw_command" }
    component_exchange "Validated status" { from_port: "LA-CONN-002.validated_status" to_port: "LA-CONN-001.validated_status" }
    component_exchange "Validated start command" { from_port: "LA-CONN-002.validated_command" to_port: "LA-CTRL-001.validated_command" protocol: "CAN FD" }
    component_exchange "Controller status" { from_port: "LA-CTRL-001.controller_status" to_port: "LA-CONN-002.controller_status" }
    component_exchange "Interlock check request" { from_port: "LA-CTRL-001.interlock_request" to_port: "LA-CTRL-002.interlock_request" }
    component_exchange "Interlock status" { from_port: "LA-CTRL-002.interlock_status" to_port: "LA-CTRL-001.interlock_status" }
    component_exchange "Body state" { from_port: "LA-VHC-005.body_state" to_port: "LA-CTRL-002.body_state" protocol: "CAN" }
    component_exchange "Battery state" { from_port: "LA-VHC-002.battery_state" to_port: "LA-CTRL-002.battery_state" protocol: "CAN FD" }
    component_exchange "Powertrain start request" { from_port: "LA-CTRL-001.start_request" to_port: "LA-CTRL-003.start_request" }
    component_exchange "Powertrain status" { from_port: "LA-CTRL-003.powertrain_status" to_port: "LA-CTRL-001.powertrain_status" }
    component_exchange "ICE start command" { from_port: "LA-CTRL-003.ice_start" to_port: "LA-VHC-001.ice_start" protocol: "CAN" }
    component_exchange "Engine status" { from_port: "LA-VHC-001.engine_status" to_port: "LA-CTRL-003.engine_status" protocol: "CAN" }
    component_exchange "EV start command" { from_port: "LA-CTRL-003.ev_start" to_port: "LA-VHC-002.ev_start" protocol: "CAN FD" }
    component_exchange "Battery status" { from_port: "LA-VHC-002.battery_status" to_port: "LA-CTRL-003.battery_status" protocol: "CAN FD" }
    component_exchange "Hybrid start command" { from_port: "LA-CTRL-003.hybrid_start" to_port: "LA-VHC-003.hybrid_start" protocol: "CAN FD" }
    component_exchange "Hybrid status" { from_port: "LA-VHC-003.hybrid_status" to_port: "LA-CTRL-003.hybrid_status" protocol: "CAN FD" }
    component_exchange "Climate pre-conditioning request" { from_port: "LA-CTRL-001.climate_request" to_port: "LA-CTRL-004.climate_request" protocol: "LIN" }
    component_exchange "HVAC command" { from_port: "LA-CTRL-004.hvac_command" to_port: "LA-VHC-004.hvac_command" protocol: "LIN" }
    component_exchange "Timer start" { from_port: "LA-CTRL-001.timer_start" to_port: "LA-CTRL-005.timer_start" }
    component_exchange "Shutdown order" { from_port: "LA-CTRL-005.shutdown" to_port: "LA-CTRL-001.shutdown" }
    component_exchange "Display command" { from_port: "LA-CTRL-001.display" to_port: "LA-VHC-006.display" protocol: "CAN" }
}

// ============================================================================
// PHYSICAL ARCHITECTURE
// ============================================================================

architecture physical {
    node "Smartphone" {
        id: "PA-DEV-001"
        description: "User's phone, iOS or Android"
        deploys "LA-USER-001"
    }
    node "Connected Car Cloud Platform" {
        id: "PA-CLOUD-001"
        description: "Backend of the mobility service, two regions"
        deploys "LA-USER-002"
        deploys "LA-CONN-003"
    }
    node "Telematics Control Unit Hardware" {
        id: "PA-ECU-001"
        safety_level: "ASIL-B"
        description: "LTE/5G modem with a hardware secure element"
        processor: "NXP i.MX 8 with SE050 secure element"
        deploys "LA-CONN-001"
        deploys "LA-CONN-002"
    }
    node "Gateway ECU" {
        id: "PA-ECU-002"
        safety_level: "ASIL-B"
        description: "Central gateway with CAN firewall; runs the AUTOSAR Classic remote start controller"
        processor: "Infineon AURIX TC397"
        redundancy: "Lockstep cores, external watchdog"
        deploys "LA-CTRL-001"
        deploys "LA-CTRL-002"
        deploys "LA-CTRL-003"
        deploys "LA-CTRL-005"
    }
    node "Engine Control Module Hardware" {
        id: "PA-ECU-003"
        safety_level: "ASIL-B"
        description: "Engine ECU"
        deploys "LA-VHC-001"
    }
    node "Battery Management System Hardware" {
        id: "PA-ECU-004"
        safety_level: "ASIL-B"
        description: "Traction battery monitoring and protection"
        deploys "LA-VHC-002"
    }
    node "Hybrid Control Unit Hardware" {
        id: "PA-ECU-006"
        safety_level: "ASIL-B"
        deploys "LA-VHC-003"
    }
    node "Body Control Module Hardware" {
        id: "PA-ECU-005"
        safety_level: "ASIL-B"
        description: "LIN master of the body electronics and the HVAC"
        deploys "LA-VHC-005"
        deploys "LA-CTRL-004"
        deploys "LA-VHC-004"
    }
    node "Instrument Cluster Hardware" {
        id: "PA-ECU-007"
        deploys "LA-VHC-006"
    }

    physical_link "Internet" { id: "PL-RS-001" from: "PA-DEV-001" to: "PA-CLOUD-001" protocol: "HTTPS over TLS 1.3" }
    physical_link "Cellular network" { id: "PL-RS-002" from: "PA-CLOUD-001" to: "PA-ECU-001" protocol: "LTE / 5G, MQTT over TLS 1.3" bandwidth: 10 Mbps }
    physical_link "Telematics CAN FD" { id: "PL-RS-003" from: "PA-ECU-001" to: "PA-ECU-002" protocol: "CAN FD" bandwidth: 2 Mbps }
    physical_link "Powertrain CAN to engine" { id: "PL-RS-004" from: "PA-ECU-002" to: "PA-ECU-003" protocol: "CAN" bandwidth: 500 kbps }
    physical_link "Powertrain CAN FD to BMS" { id: "PL-RS-005" from: "PA-ECU-002" to: "PA-ECU-004" protocol: "CAN FD" bandwidth: 2 Mbps }
    physical_link "Powertrain CAN FD to HCU" { id: "PL-RS-006" from: "PA-ECU-002" to: "PA-ECU-006" protocol: "CAN FD" bandwidth: 2 Mbps }
    physical_link "Body CAN" { id: "PL-RS-007" from: "PA-ECU-002" to: "PA-ECU-005" protocol: "CAN" bandwidth: 500 kbps }
    physical_link "Cluster CAN" { id: "PL-RS-008" from: "PA-ECU-002" to: "PA-ECU-007" protocol: "CAN" bandwidth: 500 kbps }
}

// ============================================================================
// SAFETY (ISO 26262 HARA) AND SECURITY (ISO/SAE 21434)
// ============================================================================

safety_analysis {
    hazard "Vehicle moves at remote start" {
        id: "HAZ-RS-001"
        description: "The powertrain starts while the vehicle is in gear or the parking brake is off"
        severity: "S3"
        exposure: "E2"
        controllability: "C3"
        asil: "ASIL-B"
        mitigated_by: ["SYS-RS-002", "SAFE-RS-002", "SAFE-RS-003", "STK-RS-002"]
    }
    hazard "Engine runs in an enclosed space" {
        id: "HAZ-RS-002"
        description: "A combustion engine started remotely in a closed garage"
        severity: "S3"
        exposure: "E1"
        controllability: "C3"
        asil: "ASIL-A"
        mitigated_by: ["SYS-RS-007", "REG-RS-002", "STK-RS-007"]
    }
    hazard "Controller hang during the start sequence" {
        id: "HAZ-RS-003"
        description: "The remote start controller stops servicing the sequence with the starter engaged"
        severity: "S2"
        exposure: "E2"
        controllability: "C2"
        asil: "QM"
        mitigated_by: ["SAFE-RS-001"]
    }
}

security_scales {
    confidentiality: ["Public", "Restricted", "Confidential", "Secret"]
    integrity: ["Low", "Medium", "High"]
    availability: ["Low", "Medium", "High"]
    threat_kinds: ["Spoofing", "Tampering", "Replay", "Information disclosure", "Denial of service"]
}

security_analysis {
    level: "la"
    asset "Remote start command" {
        id: "ASSET-RS-001"
        kind: "information"
        members: ["SF-AUTH", "SF-SEC"]
        security_confidentiality: "Confidential"
        security_integrity: "High"
        security_availability: "Medium"
    }
    asset "Remote start service" {
        id: "ASSET-RS-002"
        kind: "functional"
        members: ["SF-START", "SF-NOTIF"]
        security_integrity: "High"
        security_availability: "Medium"
    }
    threat "Replayed start command" {
        id: "THR-RS-001"
        kind: "Replay"
        level: 3
        rationale: "A recorded command replayed later would start the vehicle without the owner"
        applies_to: ["ASSET-RS-001"]
        involves: ["LA-CONN-001", "LA-CONN-002"]
    }
    threat "Stolen credentials" {
        id: "THR-RS-002"
        kind: "Spoofing"
        level: 3
        rationale: "An attacker with the user's credentials starts the vehicle from any phone"
        applies_to: ["ASSET-RS-001", "ASSET-RS-002"]
        involves: ["LA-USER-001", "LA-USER-002"]
    }
    threat "Backend flooded with requests" {
        id: "THR-RS-003"
        kind: "Denial of service"
        level: 2
        applies_to: ["ASSET-RS-002"]
        involves: ["LA-CONN-003"]
    }
}

// ============================================================================
// MODES, SCENARIO
// ============================================================================

state_machine "Remote start sequence" {
    initial: "Idle"
    mode Idle { description: "Vehicle parked, no remote request" }
    mode Authenticating { description: "Token and signature under verification" }
    mode CheckingInterlocks { description: "Gear, brake, doors, battery under check" }
    mode Starting { description: "Powertrain start sequence running" }
    mode Running { description: "Powertrain running, cabin pre-conditioning, countdown" }
    mode Inhibited { description: "Start refused: interlock, battery or active fault" }
    transition Idle -> Authenticating { trigger: "LF-RX" }
    transition Authenticating -> CheckingInterlocks { trigger: "LF-SIG" guard: "signature and nonce valid" }
    transition Authenticating -> Idle { trigger: "LF-REPLAY" guard: "replay or tampering detected" }
    transition CheckingInterlocks -> Starting { trigger: "LF-GATE" guard: "all interlocks satisfied" }
    transition CheckingInterlocks -> Inhibited { trigger: "LF-GATE" guard: "an interlock failed" }
    transition Starting -> Running { trigger: "LF-EXEC" guard: "powertrain reports running" timing: 1500 ms }
    transition Running -> Idle { trigger: "LF-STOP" timing: 10 min }
    transition Running -> Idle { trigger: "LF-PARK" guard: "driver takes over" }
    transition Inhibited -> Idle { trigger: "LF-TX" }
}

scenario "Remote start from the smartphone" {
    participants: ["LA-USER-001", "LA-CONN-003", "LA-CONN-001", "LA-CONN-002", "LA-CTRL-001", "LA-CTRL-002", "LA-CTRL-003", "LA-VHC-001", "LA-VHC-002"]
    message "LA-USER-001" -> "LA-CONN-003" "remote start request"
    message "LA-CONN-003" -> "LA-CONN-001" "remote command"
    message "LA-CONN-001" -> "LA-CONN-002" "command to validate"
    message "LA-CONN-002" -> "LA-CTRL-001" "validated start command"
    message "LA-CTRL-001" -> "LA-CTRL-002" "interlock check request"
    message "LA-CTRL-002" -> "LA-CTRL-001" "interlock status" { timing: 200 ms }
    fragment alt "powertrain type" {
        operand "ICE" {
            message "LA-CTRL-003" -> "LA-VHC-001" "ICE start command"
            message "LA-VHC-001" -> "LA-CTRL-003" "engine status"
        }
        operand "EV" {
            message "LA-CTRL-003" -> "LA-VHC-002" "EV start command"
            message "LA-VHC-002" -> "LA-CTRL-003" "battery status"
        }
    }
    message "LA-CTRL-001" -> "LA-CONN-002" "controller status"
    message "LA-CONN-001" -> "LA-CONN-003" "vehicle status"
    message "LA-CONN-003" -> "LA-USER-001" "start confirmation" { timing: 3 s }
}

// ============================================================================
// VERIFICATION
// ============================================================================

test_case "TC-RS-001" { verifies: ["STK-RS-001", "STK-RS-006", "SYS-RS-006"] method: "demonstration" description: "Remote start from the app on the test track; confirmation time measured" }
test_case "TC-RS-002" { verifies: ["STK-RS-002", "SYS-RS-002", "SAFE-RS-002"] method: "test" description: "Every interlock violated in turn on the HIL bench: no start, fault code logged" }
test_case "TC-RS-003" { verifies: ["STK-RS-003", "SYS-RS-005"] method: "test" description: "Start sequence on an ICE, a hybrid and an EV configuration" }
test_case "TC-RS-004" { verifies: ["STK-RS-004", "SYS-RS-001", "SYS-RS-003", "SEC-RS-001", "SEC-RS-002"] method: "test" description: "Penetration test of the command path: encryption, certificates, authentication latency" }
test_case "TC-RS-005" { verifies: ["STK-RS-005", "REG-RS-001", "REG-RS-003", "REG-RS-004"] method: "inspection" description: "Compliance file reviewed: FCC Part 15 report, GDPR records, UNECE R100 certificate" }
test_case "TC-RS-006" { verifies: ["STK-RS-007", "SYS-RS-007", "REG-RS-002"] method: "test" description: "Shutdown timer measured at 10 minutes without a driver" }
test_case "TC-RS-007" { verifies: ["STK-RS-008", "SYS-RS-008"] method: "test" description: "Cabin temperature reaches the stored preference" }
test_case "TC-RS-008" { verifies: ["SYS-RS-004"] method: "test" description: "Start refused at 19 percent traction battery and at 10.9 V" }
test_case "TC-RS-009" { verifies: ["SYS-RS-009"] method: "inspection" description: "Audit log inspected after a campaign of successful and refused starts" }
test_case "TC-RS-010" { verifies: ["SYS-RS-010", "SEC-RS-003", "SEC-RS-004"] method: "test" description: "Replayed and tampered commands rejected, alert raised, backoff observed" }
test_case "TC-RS-011" { verifies: ["SAFE-RS-001", "SAFE-RS-003"] method: "test" description: "Controller stalled and ASIL B fault injected: start inhibited by the watchdog" }

constraint "Request to confirmation within budget" {
    id: "CST-RS-001"
    assert: sum("FC-RS-001", latency) <= "FC-RS-001".latency_budget
}

// ============================================================================
// TRACEABILITY
// ============================================================================

// The system satisfies the stakeholder needs
trace "SC-RS" satisfies "STK-RS-001"
trace "SC-RS" satisfies "STK-RS-002"
trace "SC-RS" satisfies "STK-RS-003"
trace "SC-RS" satisfies "STK-RS-004"
trace "SC-RS" satisfies "STK-RS-005"
trace "SC-RS" satisfies "STK-RS-006"
trace "SC-RS" satisfies "STK-RS-007"
trace "SC-RS" satisfies "STK-RS-008"

// The logical components satisfy the system, safety, security and regulatory requirements
trace "LA-USER-001" satisfies "SYS-RS-006" { rationale: "Shows the confirmation and the warnings" }
trace "LA-USER-002" satisfies "SYS-RS-001" { rationale: "Token issuance and factor validation" }
trace "LA-USER-002" satisfies "SEC-RS-004" { rationale: "Exponential backoff after three failures" }
trace "LA-CONN-003" satisfies "SYS-RS-006" { rationale: "Push notification with delivery tracking" }
trace "LA-CONN-003" satisfies "SYS-RS-009" { rationale: "Audit log of every attempt" }
trace "LA-CONN-003" satisfies "REG-RS-003" { rationale: "Consent and retention rules on the user data" }
trace "LA-CONN-001" satisfies "SYS-RS-003" { rationale: "TLS 1.3 session with the backend" }
trace "LA-CONN-001" satisfies "REG-RS-001" { rationale: "Certified radio module" }
trace "LA-CONN-001" satisfies "SEC-RS-002" { rationale: "Certificates in the secure element" }
trace "LA-CONN-002" satisfies "SEC-RS-001" { rationale: "AES-256 payload encryption" }
trace "LA-CONN-002" satisfies "SEC-RS-003" { rationale: "Nonce window of 5 s" }
trace "LA-CONN-002" satisfies "SYS-RS-010" { rationale: "Replay and tampering detection, alert to the backend" }
trace "LA-CTRL-001" satisfies "SYS-RS-002" { rationale: "No start request before the interlock status is clear" }
trace "LA-CTRL-001" satisfies "SAFE-RS-001" { rationale: "Watchdog serviced by the sequence" }
trace "LA-CTRL-001" satisfies "SAFE-RS-003" { rationale: "Active fault list checked before the start" }
trace "LA-CTRL-002" satisfies "SYS-RS-002" { rationale: "Gear, brake, doors and hood within 200 ms" }
trace "LA-CTRL-002" satisfies "SYS-RS-004" { rationale: "Battery thresholds checked" }
trace "LA-CTRL-002" satisfies "SAFE-RS-002" { rationale: "Missing reading inhibits and logs" }
trace "LA-CTRL-003" satisfies "SYS-RS-005" { rationale: "Powertrain detection and strategy selection" }
trace "LA-CTRL-003" satisfies "REG-RS-004" { rationale: "EV start sequence per UNECE R100" }
trace "LA-CTRL-004" satisfies "SYS-RS-008" { rationale: "HVAC driven to the stored temperature" }
trace "LA-CTRL-005" satisfies "SYS-RS-007" { rationale: "Ten-minute countdown" }
trace "LA-CTRL-005" satisfies "REG-RS-002" { rationale: "Idle time limit enforced" }
trace "LA-VHC-001" satisfies "SYS-RS-005" { rationale: "ICE start sequence" }
trace "LA-VHC-002" satisfies "SYS-RS-004" { rationale: "Traction battery state of charge" }
trace "LA-VHC-003" satisfies "SYS-RS-005" { rationale: "Hybrid start sequence" }
trace "LA-VHC-004" satisfies "SYS-RS-008" { rationale: "Compressor and blower control" }
trace "LA-VHC-005" satisfies "SYS-RS-002" { rationale: "Door, hood, brake and gear sensors" }
trace "LA-VHC-006" satisfies "SYS-RS-007" { rationale: "Countdown shown to the driver" }

// System functions realize the operational activities
trace "SF-AUTH" realizes "OA-RECEIVE"
trace "SF-ILK" realizes "OA-CHECK"
trace "SF-START" realizes "OA-START"
trace "SF-CLIM" realizes "OA-CABIN"
trace "SF-NOTIF" realizes "OA-CONFIRM"
trace "SF-TIMER" realizes "OA-SHUTDOWN"

diagram "Command path" {
    view: lab
    description: "From the smartphone to the powertrain start manager"
    shows: ["LA-USER-001", "LA-CONN-003", "LA-CONN-001", "LA-CONN-002", "LA-CTRL-001", "LA-CTRL-002", "LA-CTRL-003"]
}
diagram "Vehicle network" {
    view: pab
    shows: ["PA-ECU-002"]
    context: true
}
