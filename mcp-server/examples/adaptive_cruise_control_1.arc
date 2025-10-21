// ================================================================
// ASIL-B Adaptive Cruise Control System Model
// Safety Standard: ISO 26262 ASIL-B
// Domain: Automotive
// ================================================================

model AdaptiveCruiseControl {
    metadata {
        version: "1.0"
        description: "ACC System"
        domain: "automotive"
        safety_standard: "iso26262"
        safety_level: "ASIL_B"
    }
}

// ================================================================
// STAKEHOLDER REQUIREMENTS
// ================================================================
requirements stakeholder {
    
    req STK-001 "Driver Speed Control" {
        description: "The driver shall be able to set and maintain a desired cruising speed"
        priority: High
        rationale: "Enables convenient highway driving and reduces driver fatigue"
    }
    
    req STK-002 "Automatic Distance Maintenance" {
        description: "The system shall automatically maintain safe following distance from vehicles ahead"
        priority: Critical
        rationale: "Core ACC functionality to prevent collisions while maintaining traffic flow"
    }
    
    req STK-003 "Driver Override Capability" {
        description: "The driver shall be able to override ACC at any time using brake or accelerator"
        priority: Critical
        rationale: "Driver must maintain ultimate control for safety"
    }
    
    req STK-004 "System Status Awareness" {
        description: "The driver shall be informed of ACC system status and operating mode"
        priority: High
        rationale: "Driver needs clear feedback about system state"
    }
    
    req STK-005 "Safe System Degradation" {
        description: "The system shall fail safely without causing dangerous situations"
        priority: Critical
        rationale: "Safety-critical system must handle failures gracefully"
        safety_level: ASIL_B
    }
}

// ================================================================
// SYSTEM REQUIREMENTS
// ================================================================
requirements system {
    
    req SYS-001 "Speed Range Operation" {
        description: "The ACC system shall operate in the speed range of 30-180 km/h"
        priority: High
        traces_to: ["STK-001"]
        verification: "Test in controlled environment across full speed range"
    }
    
    req SYS-002 "Target Speed Setting" {
        description: "The system shall allow speed setting in 5 km/h increments"
        priority: Medium
        traces_to: ["STK-001"]
    }
    
    req SYS-003 "Radar Detection Range" {
        description: "The forward radar shall detect vehicles up to 200 meters ahead"
        priority: Critical
        traces_to: ["STK-002"]
        safety_level: ASIL_B
    }
    
    req SYS-004 "Following Distance Calculation" {
        description: "The system shall calculate safe following distance based on current speed and time gap setting"
        priority: Critical
        traces_to: ["STK-002"]
        safety_level: ASIL_B
        verification: "Mathematical verification and dynamic testing"
    }
    
    req SYS-005 "Time Gap Settings" {
        description: "The system shall provide selectable time gaps of 1.0s, 1.5s, 2.0s, and 2.5s"
        priority: High
        traces_to: ["STK-002"]
    }
    
    req SYS-006 "Brake Override Detection" {
        description: "The system shall detect brake pedal activation within 50ms"
        priority: Critical
        traces_to: ["STK-003"]
        safety_level: ASIL_B
    }
    
    req SYS-007 "Accelerator Override Detection" {
        description: "The system shall detect accelerator pedal override and suspend ACC control"
        priority: Critical
        traces_to: ["STK-003"]
        safety_level: ASIL_B
    }
    
    req SYS-008 "Visual Feedback" {
        description: "The system shall display ACC status, set speed, and following distance on instrument cluster"
        priority: High
        traces_to: ["STK-004"]
    }
    
    req SYS-009 "Audible Warnings" {
        description: "The system shall provide audible warnings for critical situations requiring driver intervention"
        priority: Critical
        traces_to: ["STK-004"]
        safety_level: ASIL_B
    }
    
    req SYS-010 "Acceleration Control" {
        description: "The system shall control throttle to accelerate smoothly up to target speed with max acceleration 2 m/s²"
        priority: High
        traces_to: ["STK-001", "STK-002"]
    }
    
    req SYS-011 "Deceleration Control" {
        description: "The system shall control braking to decelerate smoothly with max deceleration 3 m/s²"
        priority: Critical
        traces_to: ["STK-002"]
        safety_level: ASIL_B
    }
}

// ================================================================
// SAFETY REQUIREMENTS
// ================================================================
requirements safety {
    
    req SAF-001 "Sensor Redundancy" {
        description: "Critical sensing functions shall have redundant sensors with failure detection"
        priority: Critical
        safety_level: ASIL_B
        traces_to: ["STK-005", "SYS-003"]
        verification: "Failure mode analysis and hardware testing"
    }
    
    req SAF-002 "Watchdog Monitoring" {
        description: "ACC controller shall be monitored by independent watchdog with 100ms timeout"
        priority: Critical
        safety_level: ASIL_B
        traces_to: ["STK-005"]
    }
    
    req SAF-003 "Fail-Safe State" {
        description: "Upon critical failure detection, system shall transition to safe state within 200ms"
        priority: Critical
        safety_level: ASIL_B
        traces_to: ["STK-005"]
        verification: "Fault injection testing"
    }
    
    req SAF-004 "Actuator Limit Monitoring" {
        description: "System shall monitor actuator responses and detect out-of-range conditions"
        priority: Critical
        safety_level: ASIL_B
        traces_to: ["STK-005"]
    }
    
    req SAF-005 "Driver Alert Escalation" {
        description: "If driver does not respond to warnings, system shall escalate alerts and transition to safe state"
        priority: Critical
        safety_level: ASIL_B
        traces_to: ["STK-003", "STK-004", "STK-005"]
    }
    
    req SAF-006 "Communication Integrity" {
        description: "All safety-critical CAN messages shall use CRC and sequence counters"
        priority: Critical
        safety_level: ASIL_B
        traces_to: ["STK-005"]
    }
}

// ================================================================
// LOGICAL ARCHITECTURE
// ================================================================
architecture logical {
    
    component ACCController "ACC Control Unit" {
        description: "Main adaptive cruise control logic and coordination"
        safety_level: ASIL_B
        
        provides interface IACCControl {
            description: "ACC control commands and status"
            operations: [
                "setTargetSpeed(speed: float)",
                "setTimeGap(gap: float)",
                "enableACC()",
                "disableACC()",
                "getACCStatus(): ACCStatus"
            ]
        }
        
        requires interface ISensorData {
            description: "Sensor data input"
        }
        
        requires interface IVehicleControl {
            description: "Vehicle actuation commands"
        }
        
        traces_to: ["SYS-004", "SYS-010", "SYS-011", "SAF-002"]
    }
    
    component RadarSensor "Forward Radar" {
        description: "77 GHz radar for detecting vehicles and obstacles ahead"
        safety_level: ASIL_B
        
        provides interface IRadarData {
            description: "Radar detection data"
            operations: [
                "getTargetDistance(): float",
                "getTargetVelocity(): float",
                "getTargetAcceleration(): float",
                "getSensorStatus(): SensorStatus"
            ]
        }
        
        traces_to: ["SYS-003", "SAF-001"]
    }
    
    component CameraSensor "Forward Camera" {
        description: "Vision sensor for lane detection and target verification"
        safety_level: ASIL_B
        
        provides interface ICameraData {
            description: "Visual detection data"
            operations: [
                "getTargetClassification(): VehicleType",
                "getLanePosition(): float",
                "getSensorStatus(): SensorStatus"
            ]
        }
        
        traces_to: ["SAF-001"]
    }
    
    component SpeedSensor "Vehicle Speed Sensor" {
        description: "Wheel speed sensors for accurate vehicle speed"
        safety_level: ASIL_B
        
        provides interface ISpeedData {
            description: "Vehicle speed information"
            operations: [
                "getCurrentSpeed(): float",
                "getSensorStatus(): SensorStatus"
            ]
        }
        
        traces_to: ["SYS-001"]
    }
    
    component BrakeActuator "Brake Control Module" {
        description: "Electronic brake system interface"
        safety_level: ASIL_B
        
        provides interface IBrakeControl {
            description: "Brake actuation interface"
            operations: [
                "requestBrakePressure(pressure: float)",
                "getActuatorStatus(): ActuatorStatus"
            ]
        }
        
        requires interface IBrakeCommand {
            description: "Brake command input"
        }
        
        traces_to: ["SYS-011", "SAF-004"]
    }
    
    component ThrottleActuator "Throttle Control Module" {
        description: "Electronic throttle control interface"
        safety_level: ASIL_B
        
        provides interface IThrottleControl {
            description: "Throttle actuation interface"
            operations: [
                "requestThrottlePosition(position: float)",
                "getActuatorStatus(): ActuatorStatus"
            ]
        }
        
        requires interface IThrottleCommand {
            description: "Throttle command input"
        }
        
        traces_to: ["SYS-010", "SAF-004"]
    }
    
    component DriverInterface "Driver HMI" {
        description: "Driver human-machine interface for ACC control and feedback"
        
        provides interface IDriverCommands {
            description: "Driver input commands"
            operations: [
                "onSetButton()",
                "onResumeButton()",
                "onCancelButton()",
                "onSpeedUpButton()",
                "onSpeedDownButton()",
                "onGapButton()"
            ]
        }
        
        requires interface IDisplayUpdate {
            description: "Display information update"
        }
        
        traces_to: ["SYS-002", "SYS-005", "SYS-008"]
    }
    
    component PedalMonitor "Pedal Position Monitor" {
        description: "Monitors brake and accelerator pedal positions"
        safety_level: ASIL_B
        
        provides interface IPedalStatus {
            description: "Pedal position and status"
            operations: [
                "getBrakePosition(): float",
                "getAcceleratorPosition(): float",
                "isBrakePressed(): boolean",
                "isAcceleratorPressed(): boolean"
            ]
        }
        
        traces_to: ["SYS-006", "SYS-007"]
    }
    
    component SafetyMonitor "Safety Monitoring Unit" {
        description: "Independent safety monitor with watchdog functionality"
        safety_level: ASIL_B
        
        provides interface ISafetyCheck {
            description: "Safety verification interface"
            operations: [
                "checkSystemHealth(): HealthStatus",
                "triggerSafeState()",
                "resetWatchdog()"
            ]
        }
        
        traces_to: ["SAF-002", "SAF-003", "SAF-004"]
    }
    
    component WarningSystem "Driver Warning System" {
        description: "Manages visual and audible driver warnings"
        safety_level: ASIL_B
        
        provides interface IWarningOutput {
            description: "Warning activation interface"
            operations: [
                "displayWarning(message: string, level: WarningLevel)",
                "playAudioAlert(sound: AlertSound)",
                "clearWarnings()"
            ]
        }
        
        traces_to: ["SYS-009", "SAF-005"]
    }
    
    // Logical Connections
    connection SensorToController {
        from: RadarSensor.IRadarData
        to: ACCController.ISensorData
        description: "Radar data to controller"
    }
    
    connection CameraToController {
        from: CameraSensor.ICameraData
        to: ACCController.ISensorData
        description: "Camera data to controller"
    }
    
    connection SpeedToController {
        from: SpeedSensor.ISpeedData
        to: ACCController.ISensorData
        description: "Speed data to controller"
    }
    
    connection ControllerToBrake {
        from: ACCController.IVehicleControl
        to: BrakeActuator.IBrakeCommand
        description: "Brake commands from controller"
    }
    
    connection ControllerToThrottle {
        from: ACCController.IVehicleControl
        to: ThrottleActuator.IThrottleCommand
        description: "Throttle commands from controller"
    }
    
    connection DriverToController {
        from: DriverInterface.IDriverCommands
        to: ACCController.IACCControl
        description: "Driver commands to controller"
    }
    
    connection PedalToController {
        from: PedalMonitor.IPedalStatus
        to: ACCController.ISensorData
        description: "Pedal status to controller"
    }
    
    connection SafetyMonitoring {
        from: SafetyMonitor.ISafetyCheck
        to: ACCController.IACCControl
        description: "Safety monitoring of controller"
    }
    
    connection WarningActivation {
        from: ACCController.IACCControl
        to: WarningSystem.IWarningOutput
        description: "Warning activation from controller"
    }
}

// ================================================================
// PHYSICAL ARCHITECTURE
// ================================================================
architecture physical {
    
    component ACCElectronic "ACC Electronic Control Unit" {
        description: "Physical ECU containing ACC software and safety monitor"
        realizes: ["ACCController", "SafetyMonitor"]
        
        properties: {
            processor: "Dual-core ARM Cortex-R5F",
            memory: "4MB Flash, 512KB RAM",
            operating_system: "AUTOSAR Classic",
            power_supply: "12V automotive",
            can_interfaces: 2
        }
        
        traces_to: ["SAF-002"]
    }
    
    component RadarModule "77GHz Radar Module" {
        description: "Physical radar sensor module"
        realizes: ["RadarSensor"]
        
        properties: {
            frequency: "77 GHz",
            range: "0-200m",
            field_of_view: "±20 degrees",
            update_rate: "50 Hz"
        }
        
        traces_to: ["SYS-003"]
    }
    
    component FrontCamera "Front-Facing Camera" {
        description: "Physical camera module"
        realizes: ["CameraSensor"]
        
        properties: {
            resolution: "1920x1080",
            frame_rate: "30 fps",
            field_of_view: "60 degrees"
        }
    }
    
    component WheelSpeedSensors "ABS Wheel Speed Sensors" {
        description: "Physical wheel speed sensors (4x)"
        realizes: ["SpeedSensor"]
        
        properties: {
            type: "Hall effect",
            accuracy: "±0.5 km/h"
        }
    }
    
    component ESCModule "Electronic Stability Control Module" {
        description: "Existing ESC module providing brake actuation"
        realizes: ["BrakeActuator"]
        
        properties: {
            brake_channels: 4,
            max_pressure: "180 bar"
        }
        
        traces_to: ["SYS-011"]
    }
    
    component EngineECU "Engine Control Unit" {
        description: "Existing engine ECU providing throttle control"
        realizes: ["ThrottleActuator"]
        
        properties: {
            throttle_resolution: "0.1%"
        }
        
        traces_to: ["SYS-010"]
    }
    
    component InstrumentCluster "Digital Instrument Cluster" {
        description: "Physical instrument panel display"
        realizes: ["DriverInterface", "WarningSystem"]
        
        properties: {
            display_type: "12.3-inch TFT LCD",
            resolution: "1920x720"
        }
        
        traces_to: ["SYS-008", "SYS-009"]
    }
    
    component SteeringWheelControls "Steering Wheel Control Buttons" {
        description: "Physical control buttons on steering wheel"
        realizes: ["DriverInterface"]
        
        properties: {
            buttons: ["SET", "RESUME", "CANCEL", "+", "-", "GAP"]
        }
        
        traces_to: ["SYS-002", "SYS-005"]
    }
    
    component PedalSensors "Pedal Position Sensors" {
        description: "Physical pedal position sensors"
        realizes: ["PedalMonitor"]
        
        properties: {
            type: "Dual potentiometer",
            response_time: "< 50ms"
        }
        
        traces_to: ["SYS-006", "SYS-007"]
    }
    
    component CANBus "CAN Communication Network" {
        description: "Physical CAN bus network for ECU communication"
        
        properties: {
            speed: "500 kbps",
            protocol: "CAN 2.0B",
            topology: "Linear bus"
        }
        
        traces_to: ["SAF-006"]
    }
    
    // Physical Connections via CAN
    connection CANRadar {
        from: RadarModule
        to: CANBus
        protocol: "CAN"
        message_rate: "50 Hz"
    }
    
    connection CANCamera {
        from: FrontCamera
        to: CANBus
        protocol: "CAN"
        message_rate: "30 Hz"
    }
    
    connection CANACC {
        from: ACCElectronic
        to: CANBus
        protocol: "CAN"
        message_rate: "100 Hz"
    }
    
    connection CANESC {
        from: ESCModule
        to: CANBus
        protocol: "CAN"
        message_rate: "100 Hz"
    }
    
    connection CANEngine {
        from: EngineECU
        to: CANBus
        protocol: "CAN"
        message_rate: "100 Hz"
    }
    
    connection CANCluster {
        from: InstrumentCluster
        to: CANBus
        protocol: "CAN"
        message_rate: "50 Hz"
    }
}

// ================================================================
// OPERATIONAL SCENARIOS
// ================================================================
architecture operational {
    
    scenario NormalCruising "Normal ACC Cruising" {
        description: "Vehicle maintains set speed with no vehicle ahead"
        
        actors: ["Driver", "ACCController", "ThrottleActuator", "SpeedSensor"]
        
        steps: [
            "Driver activates ACC and sets target speed",
            "ACCController monitors current speed via SpeedSensor",
            "ACCController commands ThrottleActuator to maintain target speed",
            "System provides visual feedback to driver"
        ]
        
        traces_to: ["STK-001", "SYS-001"]
    }
    
    scenario FollowingVehicle "Following Vehicle Ahead" {
        description: "ACC maintains safe distance behind slower vehicle"
        
        actors: ["ACCController", "RadarSensor", "BrakeActuator", "SpeedSensor"]
        
        steps: [
            "RadarSensor detects vehicle ahead",
            "ACCController calculates required following distance",
            "ACCController adjusts speed to maintain time gap",
            "System uses brake or throttle as needed",
            "Visual display shows following distance status"
        ]
        
        traces_to: ["STK-002", "SYS-004"]
    }
    
    scenario EmergencyBraking "Driver Brake Override" {
        description: "Driver overrides ACC by pressing brake pedal"
        
        actors: ["Driver", "PedalMonitor", "ACCController", "BrakeActuator"]
        
        steps: [
            "Driver presses brake pedal",
            "PedalMonitor detects brake activation within 50ms",
            "ACCController immediately suspends control",
            "Direct brake control transfers to driver",
            "System displays ACC suspended status"
        ]
        
        traces_to: ["STK-003", "SYS-006", "SAF-003"]
    }
    
    scenario SystemFailure "Safety Monitor Detects Failure" {
        description: "Critical failure detected and safe state activated"
        
        actors: ["SafetyMonitor", "ACCController", "WarningSystem", "Driver"]
        
        steps: [
            "SafetyMonitor detects controller malfunction",
            "WarningSystem issues critical alert to driver",
            "SafetyMonitor forces safe state transition",
            "ACC control is disabled",
            "Vehicle control returns to driver",
            "Failure is logged for diagnostics"
        ]
        
        traces_to: ["STK-005", "SAF-003", "SAF-005"]
    }
}

// ================================================================
// END OF MODEL
// ================================================================
