// One-pedal drive of an electric vehicle: the accelerator pedal also commands
// the deceleration. Releasing it brings the car down by regeneration, the
// friction brake takes over where the motor cannot, and the car is held at
// standstill. Written the way a system architect hands it to a design review:
// requirements with their integrity level, the hazards behind them, the
// functions and their latency, the ECUs and the buses, the timing budgets
// that the gate checks. Integrity levels follow ISO 26262; the stop lamp
// rule is the one of UN R13-H (regenerative deceleration above 1.3 m/s2).
//
//   arclang check     examples/automotive/one_pedal_drive.arc
//   arclang gate      examples/automotive/one_pedal_drive.arc
//   arclang workbench examples/automotive/one_pedal_drive.arc
//
// Change request CR-217 (not applied here): the regenerative deceleration of
// the pedal rises from 0.15 g to 0.25 g, for the range in winter. It changes
// the controllability of the loss of grip under regeneration (HAZ-005), and
// it halves the time allowed to cut the regeneration when a wheel slips.

model OnePedalDrive {
    name: "One-pedal drive"
    description: "Accelerator pedal deceleration by regeneration, brake blending and standstill hold"
    version: "1.0"
    safety_standard: "ISO 26262"
}

// ---------------------------------------------------------------------------
// Requirements
// ---------------------------------------------------------------------------

requirements system {
    req "SYS-OPD-001" "Deceleration on pedal release" {
        description: "Above 7 km/h, releasing the accelerator pedal shall decelerate the vehicle by regeneration at up to 1.5 m/s2 (0.15 g), following a speed dependent map"
        category: "Functional"
        priority: "High"
        safety_level: "ASIL-B"
        verification_method: "test"
    }
    req "SYS-OPD-002" "Stop lamps" {
        description: "The stop lamps shall be lit whenever the requested deceleration exceeds 1.3 m/s2, whether it is regenerative or frictional, within 100 ms"
        category: "Regulatory"
        priority: "Critical"
        safety_level: "ASIL-B"
        rationale: "UN R13-H: the following traffic must see the deceleration the driver did not command with the brake pedal"
        verification_method: "test"
    }
    req "SYS-OPD-003" "Handover to the friction brake" {
        description: "When the regeneration cannot deliver the requested deceleration (battery full or cold, motor limit), the friction brake shall supply the difference within 150 ms"
        category: "Functional safety"
        priority: "Critical"
        safety_level: "ASIL-C"
        rationale: "The driver expects the same deceleration whatever the state of the battery"
        verification_method: "test"
    }
    req "SYS-OPD-004" "Standstill hold" {
        description: "After a stop by the pedal, the vehicle shall be held within 500 ms of standstill, on a slope up to 30 %"
        category: "Functional safety"
        priority: "High"
        safety_level: "ASIL-B"
        verification_method: "test"
    }
    req "SYS-OPD-005" "Pedal plausibility" {
        description: "The pedal position shall be measured on two independent tracks; a mismatch above 5 % for 20 ms shall be a detected fault and the safe state shall be reached within 100 ms"
        category: "Functional safety"
        priority: "Critical"
        safety_level: "ASIL-C"
        rationale: "100 ms is the fault tolerant time interval of the unintended acceleration and unintended deceleration hazards"
        verification_method: "test"
    }
    req "SYS-OPD-006" "Torque monitoring" {
        description: "The motor torque shall be compared with the request on an independent monitor; a deviation above the tolerance shall bring the system to the safe state within 100 ms"
        category: "Functional safety"
        priority: "Critical"
        safety_level: "ASIL-C"
        verification_method: "test"
    }
    req "SYS-OPD-007" "Driver authority" {
        description: "A brake pedal application shall always override the deceleration requested by the accelerator pedal"
        category: "Safety override"
        priority: "Critical"
        safety_level: "ASIL-C"
        verification_method: "test"
    }
    req "SYS-OPD-008" "Regeneration cut-off on wheel slip" {
        description: "When the brake system detects a wheel slip, the regenerative torque shall be removed within 100 ms"
        category: "Functional safety"
        priority: "Critical"
        safety_level: "ASIL-B"
        rationale: "Regeneration acts on one axle: on a low grip surface it can lock that axle and unsettle the vehicle"
        verification_method: "test"
    }
    req "SYS-OPD-009" "Driver information" {
        description: "The cluster shall show when the regeneration is limited and the deceleration is lower than usual"
        category: "Human machine interface"
        priority: "Medium"
        safety_level: "ASIL-A"
        verification_method: "demonstration"
    }
}

// ---------------------------------------------------------------------------
// Operational analysis: who needs what
// ---------------------------------------------------------------------------

operational_analysis "One-pedal Operations" {
    actor "Driver" {
        id: "ACT-001"
        description: "Drives with the accelerator pedal, uses the brake pedal when more is needed, keeps the authority"
    }
    actor "Following Traffic" {
        id: "ACT-002"
        description: "The vehicles behind, warned by the stop lamps"
    }
    actor "Road Surface" {
        id: "ACT-003"
        description: "Grip from dry asphalt to ice, slope up to 30 %"
    }
    entity "Ego Vehicle" {
        id: "ACT-004"
        description: "The electric vehicle under one-pedal control"
        activity "Adjust Speed with the Pedal" {
            id: "OA-001"
            description: "Accelerate and decelerate with the accelerator pedal alone"
        }
        activity "Brake on Demand" {
            id: "OA-002"
            description: "Decelerate harder with the brake pedal, which keeps the priority"
        }
        activity "Hold at Standstill" {
            id: "OA-003"
            description: "Stay where it stopped, on a slope as on the flat"
        }
        activity "Warn Following Traffic" {
            id: "OA-004"
            description: "Light the stop lamps when the deceleration is significant"
        }
    }
    operational_capability "Drive with one pedal" {
        id: "OC-001"
        description: "Control the speed, down to a stop, with the accelerator pedal only"
        involves: ["ACT-001", "ACT-002", "ACT-003", "ACT-004"]
    }
    interaction "Pedal movement" { from: "ACT-001" to: "OA-001" }
    interaction "Brake pedal" { from: "ACT-001" to: "OA-002" }
    interaction "Grip and slope" { from: "ACT-003" to: "OA-003" }
    interaction "Stop lamps" { from: "OA-004" to: "ACT-002" }
}

// ---------------------------------------------------------------------------
// System analysis: what the system does
// ---------------------------------------------------------------------------

system_analysis "One-pedal System" {
    actor "Driver" { id: "SA-DRV" }
    actor "Following Traffic" { id: "SA-TRF" }
    mission "Comfortable and efficient driving" { id: "MIS-001" }
    capability "One-pedal driving" {
        id: "CAP-001"
        mission: "MIS-001"
        realizes: "OC-001"
        involves: ["SF-001", "SF-002", "SF-003", "SF-004", "SF-005", "SF-006", "SF-007", "SF-008"]
    }
    function "Read Accelerator Pedal" {
        id: "SF-001"
        description: "Measure the pedal position on two independent tracks"
        safety_level: "ASIL-C"
        latency: 5 ms
        port out "pedal"
    }
    function "Interpret Driver Demand" {
        id: "SF-002"
        description: "Turn the pedal position and the speed into a torque or a deceleration demand"
        safety_level: "ASIL-C"
        latency: 10 ms
        port in "pedal"
        port out "demand"
    }
    function "Compute Regeneration Limit" {
        id: "SF-003"
        description: "The deceleration the motor can deliver now: charge power of the battery, motor and inverter limits, grip"
        safety_level: "ASIL-C"
        latency: 10 ms
        port in "slip"
        port in "charge_limit"
        port out "limit"
    }
    function "Blend Regeneration and Friction" {
        id: "SF-004"
        description: "Split the demand between the motor and the friction brake, never below the request"
        safety_level: "ASIL-C"
        latency: 10 ms
        port in "demand"
        port in "limit"
        port out "motor_request"
        port out "friction_request"
        port out "lamp_request"
    }
    function "Command Motor Torque" {
        id: "SF-005"
        description: "Apply the requested torque, positive or negative, with a ramp that keeps the comfort"
        safety_level: "ASIL-C"
        latency: 30 ms
        port in "motor_request"
        port in "cut_off"
    }
    function "Apply Friction Brake" {
        id: "SF-006"
        description: "Build the brake pressure that completes the regeneration"
        safety_level: "ASIL-D"
        latency: 120 ms
        port in "friction_request"
    }
    function "Switch Stop Lamps" {
        id: "SF-007"
        description: "Light the stop lamps above 1.3 m/s2 of requested deceleration"
        safety_level: "ASIL-B"
        latency: 20 ms
        port in "lamp_request"
    }
    function "Hold Vehicle at Standstill" {
        id: "SF-008"
        description: "Keep the vehicle stopped, on a slope as on the flat"
        safety_level: "ASIL-B"
        latency: 300 ms
    }
    function "Monitor Pedal and Torque" {
        id: "SF-009"
        description: "Check the two pedal tracks and compare the motor torque with the request"
        safety_level: "ASIL-C"
        latency: 20 ms
        port out "fault"
    }
    function "Reach Safe State" {
        id: "SF-010"
        description: "Remove the regeneration, let the friction brake and the driver take over, warn the driver"
        safety_level: "ASIL-C"
        latency: 50 ms
        port in "fault"
    }
    function "Detect Wheel Slip" {
        id: "SF-011"
        description: "Detect the wheel slip of the driven axle"
        safety_level: "ASIL-B"
        latency: 10 ms
        port out "slip"
    }
    function "Inform the Driver" {
        id: "SF-012"
        description: "Show the one-pedal state and the limited regeneration"
        safety_level: "ASIL-A"
    }

    functional_exchange "pedal position" { id: "FE-001" from: "SF-001.pedal" to: "SF-002.pedal" }
    functional_exchange "driver demand" { id: "FE-002" from: "SF-002.demand" to: "SF-004.demand" }
    functional_exchange "regeneration limit" { id: "FE-003" from: "SF-003.limit" to: "SF-004.limit" }
    functional_exchange "motor request" { id: "FE-004" from: "SF-004.motor_request" to: "SF-005.motor_request" }
    functional_exchange "friction request" { id: "FE-005" from: "SF-004.friction_request" to: "SF-006.friction_request" }
    functional_exchange "stop lamp request" { id: "FE-006" from: "SF-004.lamp_request" to: "SF-007.lamp_request" }
    functional_exchange "wheel slip" { id: "FE-007" from: "SF-011.slip" to: "SF-003.slip" }

    functional_chain "Pedal release to deceleration" {
        id: "FC-001"
        description: "From the pedal position to the negative motor torque"
        involves: ["SF-001", "SF-002", "SF-004", "SF-005"]
        latency_budget: 100 ms
    }
    functional_chain "Regeneration loss to friction handover" {
        id: "FC-002"
        description: "From the loss of regeneration capability to the brake pressure that replaces it"
        involves: ["SF-003", "SF-004", "SF-006"]
        latency_budget: 150 ms
    }
    functional_chain "Wheel slip to regeneration cut-off" {
        id: "FC-003"
        description: "From the slip detected by the brake system to the torque removed"
        involves: ["SF-011", "SF-003", "SF-005"]
        latency_budget: 100 ms
    }
    functional_chain "Fault to safe state" {
        id: "FC-004"
        description: "From the detected pedal or torque fault to the safe state"
        involves: ["SF-009", "SF-010"]
        latency_budget: 100 ms
    }
    functional_chain "Deceleration to stop lamps" {
        id: "FC-005"
        description: "From the blended demand to the lit lamps"
        involves: ["SF-004", "SF-007"]
        latency_budget: 100 ms
    }
}

// ---------------------------------------------------------------------------
// Logical architecture: how the system is structured
// ---------------------------------------------------------------------------

logical_architecture "One-pedal Logical Architecture" {
    component "Pedal Sensor Module" {
        id: "LC-001"
        safety_level: "ASIL-C"
        description: "Two independent tracks of the accelerator pedal"
        port out "pedal_tracks"
        function "Measure Track 1" { id: "LF-001" realizes: "SF-001" }
        function "Measure Track 2" { id: "LF-002" realizes: "SF-001" }
    }
    component "Driver Demand Interpreter" {
        id: "LC-002"
        safety_level: "ASIL-C"
        description: "The one-pedal map: pedal position and speed to demand"
        port in "pedal_tracks"
        port out "demand"
        function "Apply Pedal Map" { id: "LF-003" realizes: "SF-002" }
        function "Apply Creep and Fade" { id: "LF-004" realizes: "SF-002" description: "Smooth the demand around standstill" }
    }
    component "Regeneration Limiter" {
        id: "LC-003"
        safety_level: "ASIL-C"
        description: "How much the motor can decelerate the vehicle now"
        port in "charge_limit"
        port in "slip"
        port out "limit"
        function "Evaluate Charge Power Limit" { id: "LF-005" realizes: "SF-003" }
        function "Reduce on Slip" { id: "LF-006" realizes: "SF-003" description: "Reduce the limit as soon as the brake system reports a slip" }
    }
    component "Brake Blending Manager" {
        id: "LC-004"
        safety_level: "ASIL-C"
        description: "Splits the demand between the motor and the friction brake"
        port in "demand"
        port in "limit"
        port in "inhibit"
        port out "motor_request"
        port out "friction_request"
        port out "lamp_request"
        function "Split the Demand" { id: "LF-007" realizes: "SF-004" }
        function "Give Priority to the Brake Pedal" { id: "LF-008" realizes: "SF-004" }
    }
    component "Motor Torque Controller" {
        id: "LC-005"
        safety_level: "ASIL-C"
        description: "The inverter control: torque loop of the traction motor"
        port in "motor_request"
        port in "cut_off"
        function "Control Torque" { id: "LF-009" realizes: "SF-005" }
        function "Ramp Torque" { id: "LF-010" realizes: "SF-005" }
    }
    component "Friction Brake Controller" {
        id: "LC-006"
        safety_level: "ASIL-D"
        description: "The brake system: pressure build-up, wheel slip detection"
        port in "friction_request"
        port out "slip"
        function "Build Brake Pressure" { id: "LF-011" realizes: "SF-006" }
        function "Estimate Wheel Slip" { id: "LF-012" realizes: "SF-011" }
    }
    component "Stop Lamp Controller" {
        id: "LC-007"
        safety_level: "ASIL-B"
        description: "Body control of the stop lamps"
        port in "lamp_request"
        function "Drive Stop Lamps" { id: "LF-013" realizes: "SF-007" }
    }
    component "Standstill Hold Manager" {
        id: "LC-008"
        safety_level: "ASIL-B"
        description: "Detects the stop and holds the vehicle"
        function "Detect Standstill" { id: "LF-014" realizes: "SF-008" }
        function "Hold the Vehicle" { id: "LF-015" realizes: "SF-008" }
    }
    component "Safety Monitor" {
        id: "LC-009"
        safety_level: "ASIL-C"
        description: "Independent checks of the pedal tracks and of the torque, on the safety core"
        port in "pedal_tracks"
        port out "inhibit"
        port out "cut_off"
        function "Compare Pedal Tracks" { id: "LF-016" realizes: "SF-009" }
        function "Compare Torque with Request" { id: "LF-017" realizes: "SF-009" }
        function "Command Safe State" { id: "LF-018" realizes: "SF-010" }
    }
    component "Battery Management" {
        id: "LC-010"
        safety_level: "ASIL-B"
        description: "Charge power limit from the state of charge, the temperature and the cell voltages"
        port out "charge_limit"
        function "Compute Charge Power Limit" { id: "LF-019" realizes: "SF-003" }
    }
    component "Driver Display" {
        id: "LC-011"
        safety_level: "ASIL-A"
        description: "One-pedal state and limited regeneration in the cluster"
        function "Display One-pedal State" { id: "LF-020" realizes: "SF-012" }
    }

    component_exchange "Pedal tracks" { from_port: "LC-001.pedal_tracks" to_port: "LC-002.pedal_tracks" protocol: "SENT" period: 1 ms }
    component_exchange "Pedal tracks to the monitor" { from_port: "LC-001.pedal_tracks" to_port: "LC-009.pedal_tracks" protocol: "SENT" period: 1 ms }
    component_exchange "Driver demand" { from_port: "LC-002.demand" to_port: "LC-004.demand" }
    component_exchange "Regeneration limit" { from_port: "LC-003.limit" to_port: "LC-004.limit" }
    component_exchange "Charge power limit" { from_port: "LC-010.charge_limit" to_port: "LC-003.charge_limit" protocol: "CAN FD" period: 20 ms }
    component_exchange "Wheel slip" { from_port: "LC-006.slip" to_port: "LC-003.slip" protocol: "CAN FD" period: 10 ms }
    component_exchange "Motor request" { from_port: "LC-004.motor_request" to_port: "LC-005.motor_request" protocol: "CAN FD" period: 10 ms }
    component_exchange "Friction request" { from_port: "LC-004.friction_request" to_port: "LC-006.friction_request" protocol: "CAN FD" period: 10 ms }
    component_exchange "Stop lamp request" { from_port: "LC-004.lamp_request" to_port: "LC-007.lamp_request" protocol: "CAN" period: 20 ms }
    component_exchange "Blending inhibit" { from_port: "LC-009.inhibit" to_port: "LC-004.inhibit" }
    component_exchange "Torque cut-off" { from_port: "LC-009.cut_off" to_port: "LC-005.cut_off" protocol: "CAN FD" period: 10 ms }
}

// ---------------------------------------------------------------------------
// Physical architecture: where it runs
// ---------------------------------------------------------------------------

physical_architecture "One-pedal Physical Architecture" {
    node "Pedal Sensor" {
        id: "PN-001"
        safety_level: "ASIL-C"
        description: "Dual track hall-effect pedal sensor with a SENT output per track"
        deploys "LC-001"
    }
    node "Vehicle Control Unit" {
        id: "PN-002"
        safety_level: "ASIL-C"
        processor: "Infineon AURIX TC397, lockstep cores"
        description: "Runs the pedal map, the regeneration limiter and the blending; the monitor runs on the safety core"
        deploys "LC-002"
        deploys "LC-003"
        deploys "LC-004"
        deploys "LC-009"
    }
    node "Inverter" {
        id: "PN-003"
        safety_level: "ASIL-C"
        processor: "Infineon AURIX TC375"
        deploys "LC-005"
    }
    node "Battery Management System" {
        id: "PN-004"
        safety_level: "ASIL-B"
        processor: "NXP S32K3"
        deploys "LC-010"
    }
    node "Brake System ECU" {
        id: "PN-005"
        safety_level: "ASIL-D"
        processor: "Infineon AURIX TC387, lockstep cores"
        description: "Electro-hydraulic brake: pressure build-up, ABS and stability control, electric parking brake"
        deploys "LC-006"
        deploys "LC-008"
    }
    node "Body Control Module" {
        id: "PN-006"
        safety_level: "ASIL-B"
        processor: "NXP S32K1"
        deploys "LC-007"
    }
    node "Instrument Cluster" {
        id: "PN-007"
        safety_level: "ASIL-A"
        processor: "NXP i.MX 8"
        deploys "LC-011"
    }
    physical_link "Pedal SENT lines" { id: "PL-001" from: "PN-001" to: "PN-002" protocol: "SENT" bandwidth: 100 kbps }
    physical_link "Powertrain CAN FD" { id: "PL-002" from: "PN-002" to: "PN-003" protocol: "CAN FD" bandwidth: 2 Mbps }
    physical_link "Battery CAN FD" { id: "PL-003" from: "PN-004" to: "PN-002" protocol: "CAN FD" bandwidth: 2 Mbps }
    physical_link "Chassis CAN FD" { id: "PL-004" from: "PN-002" to: "PN-005" protocol: "CAN FD" bandwidth: 2 Mbps }
    physical_link "Body CAN" { id: "PL-005" from: "PN-002" to: "PN-006" protocol: "CAN" bandwidth: 500 kbps }
    physical_link "Cluster CAN" { id: "PL-006" from: "PN-002" to: "PN-007" protocol: "CAN" bandwidth: 500 kbps }
    physical_exchange "Pedal tracks" { id: "PX-001" from: "PN-001" to: "PN-002" via: "Pedal SENT lines" message_type: "APP_Tracks" frequency: "1000 Hz" }
    physical_exchange "Motor torque request" { id: "PX-002" from: "PN-002" to: "PN-003" via: "Powertrain CAN FD" message_type: "MotorTorqueRequest" frequency: "100 Hz" }
    physical_exchange "Charge power limit" { id: "PX-003" from: "PN-004" to: "PN-002" via: "Battery CAN FD" message_type: "BMS_ChargeLimit" frequency: "50 Hz" }
    physical_exchange "Friction deceleration request" { id: "PX-004" from: "PN-002" to: "PN-005" via: "Chassis CAN FD" message_type: "FrictionDecelRequest" frequency: "100 Hz" }
    physical_exchange "Wheel slip status" { id: "PX-005" from: "PN-005" to: "PN-002" via: "Chassis CAN FD" message_type: "ESC_SlipStatus" frequency: "100 Hz" }
    physical_exchange "Stop lamp request" { id: "PX-006" from: "PN-002" to: "PN-006" via: "Body CAN" message_type: "StopLampRequest" frequency: "50 Hz" }
    physical_exchange "One-pedal status" { id: "PX-007" from: "PN-002" to: "PN-007" via: "Cluster CAN" message_type: "OPD_Status" frequency: "10 Hz" }
}

// ---------------------------------------------------------------------------
// Safety: HARA and FMEA
// ---------------------------------------------------------------------------

safety_analysis {
    hazard "Unintended regenerative deceleration" {
        id: "HAZ-001"
        description: "The motor decelerates the vehicle although the driver asked for none"
        severity: "S2"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-B"
        mitigated_by: ["SYS-OPD-005", "SYS-OPD-006", "SYS-OPD-002"]
    }
    hazard "Unintended acceleration" {
        id: "HAZ-002"
        description: "The motor delivers a positive torque the driver did not ask for"
        severity: "S3"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-C"
        mitigated_by: ["SYS-OPD-005", "SYS-OPD-006", "SYS-OPD-007"]
    }
    hazard "Loss of deceleration at pedal release" {
        id: "HAZ-003"
        description: "The regeneration stops and the friction brake does not replace it"
        severity: "S2"
        exposure: "E4"
        controllability: "C2"
        asil: "ASIL-B"
        mitigated_by: ["SYS-OPD-003", "SYS-OPD-007"]
    }
    hazard "Roll-away after the stop" {
        id: "HAZ-004"
        description: "The vehicle moves after a one-pedal stop on a slope"
        severity: "S2"
        exposure: "E3"
        controllability: "C2"
        asil: "ASIL-A"
        mitigated_by: ["SYS-OPD-004"]
    }
    hazard "Loss of grip under regeneration" {
        id: "HAZ-005"
        description: "The regeneration locks the driven axle on a low grip surface"
        severity: "S3"
        exposure: "E2"
        controllability: "C2"
        asil: "ASIL-A"
        mitigated_by: ["SYS-OPD-008"]
    }
    fmea "Pedal track drift" {
        id: "FMEA-001"
        failure_mode: "One pedal track drifts"
        effect: "Unintended torque if the drift is not detected"
        cause: "Magnet or Hall element ageing"
        severity: "S3"
        occurrence: "Remote"
        detection: "Comparison of the two tracks, 20 ms"
        rpn: 45
    }
    fmea "Wrong torque sign" {
        id: "FMEA-002"
        failure_mode: "Negative torque commanded as positive, or the reverse"
        effect: "Unintended acceleration or deceleration"
        cause: "Software defect or corrupted CAN FD frame"
        severity: "S3"
        occurrence: "Remote"
        detection: "Independent comparison of the torque with the request, E2E protected frames"
        rpn: 54
    }
}

// ---------------------------------------------------------------------------
// Modes, scenarios
// ---------------------------------------------------------------------------

state_machine "One-pedal modes" {
    initial: "Standby"
    mode Standby { description: "Ready, vehicle stopped or below 7 km/h" }
    mode OnePedal { description: "Deceleration by the pedal, regeneration available" }
    mode RegenerationLimited { description: "The motor cannot deliver all of the demand, the friction brake completes" }
    mode Hold { description: "Stopped and held" }
    state Degraded { description: "Pedal or torque fault, regeneration removed, the driver and the friction brake decelerate" }
    transition Standby -> OnePedal { trigger: "LF-003" guard: "speed above 7 km/h" }
    transition OnePedal -> RegenerationLimited { trigger: "LF-005" guard: "limit below the demand" }
    transition RegenerationLimited -> OnePedal { trigger: "LF-005" guard: "limit above the demand" }
    transition OnePedal -> Hold { trigger: "LF-014" guard: "speed zero" }
    transition Hold -> Standby { trigger: "LF-003" guard: "pedal pressed" }
    transition OnePedal -> Degraded { trigger: "LF-018" }
    transition RegenerationLimited -> Degraded { trigger: "LF-018" }
    transition Degraded -> Standby { trigger: "LF-003" guard: "ignition cycle" }
}

scenario "Pedal release at 80 km/h" {
    participants: ["LC-001", "LC-002", "LC-004", "LC-005", "LC-007"]
    message "LC-001" -> "LC-002" "pedal tracks"
    message "LC-002" -> "LC-004" "deceleration demand"
    message "LC-004" -> "LC-005" "negative torque request" { timing: 100 ms }
    message "LC-004" -> "LC-007" "stop lamp request" { timing: 100 ms }
}

scenario "Wheel slip under regeneration" {
    participants: ["LC-006", "LC-003", "LC-004", "LC-005"]
    message "LC-006" -> "LC-003" "wheel slip"
    message "LC-003" -> "LC-004" "reduced regeneration limit"
    message "LC-004" -> "LC-005" "torque request reduced" { timing: 100 ms }
}

// ---------------------------------------------------------------------------
// Verification and constraints
// ---------------------------------------------------------------------------

test_case "TC-OPD-001" { verifies: ["SYS-OPD-001"] method: "test" description: "Pedal release at 30, 80 and 130 km/h on the dynamometer track, deceleration measured against the map" }
test_case "TC-OPD-002" { verifies: ["SYS-OPD-002"] method: "test" description: "Stop lamp activation threshold and delay measured on the vehicle" }
test_case "TC-OPD-003" { verifies: ["SYS-OPD-003"] method: "test" description: "Battery full and cold battery: friction brake takeover measured on the track" }
test_case "TC-OPD-004" { verifies: ["SYS-OPD-004"] method: "test" description: "Stops on 5, 15 and 30 % slopes, release of the vehicle after 10 minutes" }
test_case "TC-OPD-005" { verifies: ["SYS-OPD-005"] method: "test" description: "Fault injection on each pedal track on the HIL bench, time to the safe state" }
test_case "TC-OPD-006" { verifies: ["SYS-OPD-006"] method: "test" description: "Torque sign and amplitude faults injected on the HIL bench, time to the safe state" }
test_case "TC-OPD-007" { verifies: ["SYS-OPD-007"] method: "test" description: "Brake pedal applied during a pedal-commanded deceleration" }
test_case "TC-OPD-008" { verifies: ["SYS-OPD-008"] method: "test" description: "Pedal release on a split-grip and an ice surface, regenerative torque removal time on the HIL bench" }
test_case "TC-OPD-009" { verifies: ["SYS-OPD-009"] method: "demonstration" description: "Message shown with a full battery and with a cold battery" }

constraint "Pedal release within budget" {
    id: "CST-OPD-001"
    assert: sum("FC-001", latency) <= "FC-001".latency_budget
}
constraint "Friction handover within budget" {
    id: "CST-OPD-002"
    assert: sum("FC-002", latency) <= "FC-002".latency_budget
}
constraint "Regeneration cut-off on slip within budget" {
    id: "CST-OPD-003"
    assert: sum("FC-003", latency) <= "FC-003".latency_budget
}
constraint "Safe state within the fault tolerant time" {
    id: "CST-OPD-004"
    assert: sum("FC-004", latency) <= "FC-004".latency_budget
}
constraint "Stop lamps within budget" {
    id: "CST-OPD-005"
    assert: sum("FC-005", latency) <= "FC-005".latency_budget
}

// ---------------------------------------------------------------------------
// Traceability
// ---------------------------------------------------------------------------

trace "LC-002" satisfies "SYS-OPD-001" { rationale: "The pedal map sets the deceleration of the release" }
trace "LC-005" satisfies "SYS-OPD-001" { rationale: "The torque loop delivers the negative torque" }
trace "LC-004" satisfies "SYS-OPD-002" { rationale: "The blending knows the deceleration and requests the lamps" }
trace "LC-007" satisfies "SYS-OPD-002" { rationale: "The body control lights the lamps" }
trace "LC-004" satisfies "SYS-OPD-003" { rationale: "The blending requests the friction share" }
trace "LC-006" satisfies "SYS-OPD-003" { rationale: "The brake system builds the pressure" }
trace "LC-003" satisfies "SYS-OPD-003" { rationale: "The limiter tells when the motor falls short" }
trace "LC-008" satisfies "SYS-OPD-004" { rationale: "Standstill detection and hold" }
trace "LC-001" satisfies "SYS-OPD-005" { rationale: "Two independent tracks" }
trace "LC-009" satisfies "SYS-OPD-005" { rationale: "The monitor compares the tracks" }
trace "LC-009" satisfies "SYS-OPD-006" { rationale: "The monitor compares the torque with the request on the safety core" }
trace "LC-004" satisfies "SYS-OPD-007" { rationale: "The brake pedal has the priority in the blending" }
trace "LC-003" satisfies "SYS-OPD-008" { rationale: "The limiter reduces the regeneration on slip" }
trace "LC-006" satisfies "SYS-OPD-008" { rationale: "The brake system estimates the slip" }
trace "LC-005" satisfies "SYS-OPD-008" { rationale: "The torque loop removes the torque" }
trace "LC-010" satisfies "SYS-OPD-003" { rationale: "The battery tells how much it can absorb" }
trace "LC-011" satisfies "SYS-OPD-009" { rationale: "The cluster shows the limited regeneration" }
trace "SF-001" realizes "OA-001"
trace "SF-006" realizes "OA-002"
trace "SF-008" realizes "OA-003"
trace "SF-007" realizes "OA-004"

diagram "From pedal to wheels" {
    view: lab
    description: "The deceleration chain through the logical components"
    shows: ["LC-001", "LC-002", "LC-003", "LC-004", "LC-005", "LC-006"]
}
diagram "Around the Vehicle Control Unit" {
    view: pab
    shows: ["PN-002"]
    context: true
}
