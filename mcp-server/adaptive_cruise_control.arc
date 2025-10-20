model AdaptiveCruiseControl {
    metadata {
        name: "Adaptive Cruise Control System"
        version: "1.0.0"
        author: "System Architect"
        description: "ASIL-B compliant adaptive cruise control system for automotive applications"
        safety_standard: "ISO 26262"
    }

    // ========== STAKEHOLDER REQUIREMENTS ==========
    requirements stakeholder {
        req STK-001 "Adaptive Speed Control" {
            description: "The system shall maintain vehicle speed at driver-set target while adapting to traffic conditions"
            priority: Critical
            safety_level: ASIL_B
            rationale: "Core ACC functionality for safe highway driving"
        }

        req STK-002 "Safe Distance Maintenance" {
            description: "The system shall maintain safe following distance from preceding vehicles"
            priority: Critical
            safety_level: ASIL_B
            rationale: "Prevent rear-end collisions"
        }

        req STK-003 "Driver Override Capability" {
            description: "Driver shall be able to override system at any time via brake or accelerator"
            priority: Critical
            safety_level: ASIL_B
            rationale: "Driver must retain ultimate control"
        }

        req STK-004 "Speed Range Operation" {
            description: "System shall operate in speed range 30-180 km/h"
            priority: High
            rationale: "Typical highway and urban driving speeds"
        }

        req STK-005 "User Interface" {
            description: "System shall provide clear visual and haptic feedback to driver"
            priority: High
            rationale: "Driver situational awareness"
        }
    }

    // ========== SYSTEM REQUIREMENTS ==========
    requirements system {
        req SYS-001 "Target Speed Control" {
            description: "System shall control vehicle speed to match driver-set target speed ±2 km/h"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-001]
            verification: "Vehicle speed measurement and control accuracy test"
        }

        req SYS-002 "Distance Sensing" {
            description: "System shall detect vehicles in forward path up to 200m with ±0.5m accuracy"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-002]
            verification: "Radar sensor accuracy test in multiple weather conditions"
        }

        req SYS-003 "Time Gap Control" {
            description: "System shall maintain configurable time gap (1.0s, 1.5s, 2.0s, 2.5s) from preceding vehicle"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-002]
            verification: "Time gap measurement test"
        }

        req SYS-004 "Brake Intervention" {
            description: "System shall apply braking up to 0.3g deceleration when required to maintain safe distance"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-002]
            verification: "Emergency braking response time test"
        }

        req SYS-005 "Acceleration Control" {
            description: "System shall control throttle for smooth acceleration up to 0.2g"
            priority: High
            safety_level: ASIL_B
            traces: [STK-001]
            verification: "Acceleration smoothness and jerk test"
        }

        req SYS-006 "Driver Override Detection" {
            description: "System shall detect brake pedal application within 50ms"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-003]
            verification: "Pedal sensor response time test"
        }

        req SYS-007 "System Activation Conditions" {
            description: "System shall activate only when speed > 30 km/h and driver presses SET button"
            priority: High
            traces: [STK-004]
            verification: "Activation condition boundary test"
        }

        req SYS-008 "Failsafe Deactivation" {
            description: "System shall deactivate and alert driver if sensor failure detected"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-003]
            verification: "Fault injection testing"
        }

        req SYS-009 "Status Display" {
            description: "System shall display current state (Active/Standby/Off), set speed, and time gap on instrument cluster"
            priority: High
            traces: [STK-005]
            verification: "HMI display verification test"
        }

        req SYS-010 "Warning Alerts" {
            description: "System shall provide visual and audible warnings for critical situations (rapid approach, sensor failure)"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-005]
            verification: "Warning system response test"
        }
    }

    // ========== SAFETY REQUIREMENTS ==========
    requirements safety {
        req SAF-001 "Sensor Redundancy" {
            description: "Forward sensing shall use redundant sensors (radar + camera) with cross-validation"
            priority: Critical
            safety_level: ASIL_B
            traces: [SYS-002, SYS-008]
            verification: "Sensor failure mode analysis"
        }

        req SAF-002 "Diagnostic Coverage" {
            description: "System shall achieve >90% diagnostic coverage for ASIL-B rated functions"
            priority: Critical
            safety_level: ASIL_B
            traces: [SYS-008]
            verification: "Diagnostic coverage analysis"
        }

        req SAF-003 "Fail-Operational Time" {
            description: "System shall maintain safe operation for minimum 1000ms after single point failure"
            priority: Critical
            safety_level: ASIL_B
            traces: [SYS-008]
            verification: "Fault tolerance timing test"
        }

        req SAF-004 "Max Deceleration Limit" {
            description: "System shall never command deceleration exceeding 0.4g to prevent instability"
            priority: Critical
            safety_level: ASIL_B
            traces: [SYS-004]
            verification: "Deceleration limit enforcement test"
        }

        req SAF-005 "Watchdog Monitoring" {
            description: "Independent safety monitor shall supervise main controller with 100ms timeout"
            priority: Critical
            safety_level: ASIL_B
            traces: [SYS-008]
            verification: "Watchdog timing verification"
        }
    }

    // ========== LOGICAL ARCHITECTURE ==========
    architecture logical {
        component SensingSubsystem "Forward Sensing Subsystem" {
            description: "Detects and tracks objects in vehicle forward path"
            safety_level: ASIL_B
            
            provides interface IObjectDetection {
                description: "Provides detected object data"
                signals: [
                    "ObjectDistance: Real (m)",
                    "ObjectRelativeSpeed: Real (m/s)",
                    "ObjectLateralPosition: Real (m)",
                    "DetectionConfidence: Integer (0-100%)",
                    "SensorStatus: Enum {OK, DEGRADED, FAILED}"
                ]
            }
            
            requires interface IVehicleSpeed {
                description: "Current vehicle speed for relative calculations"
                signals: ["EgoSpeed: Real (m/s)"]
            }
        }

        component RadarSensor "77GHz FMCW Radar" {
            description: "Long-range forward radar sensor"
            safety_level: ASIL_B
            parent: SensingSubsystem
            
            provides interface IRadarData {
                signals: [
                    "TargetRange: Real (m)",
                    "TargetVelocity: Real (m/s)",
                    "TargetAzimuth: Real (degrees)"
                ]
            }
        }

        component CameraSensor "Forward Mono Camera" {
            description: "Vision-based object detection and validation"
            safety_level: ASIL_B
            parent: SensingSubsystem
            
            provides interface ICameraData {
                signals: [
                    "ObjectDetected: Boolean",
                    "ObjectWidth: Real (m)",
                    "LaneMarkings: Array"
                ]
            }
        }

        component ControllerSubsystem "ACC Control Subsystem" {
            description: "Main adaptive cruise control logic and decision making"
            safety_level: ASIL_B
            
            requires interface IObjectDetection
            requires interface IDriverInput {
                description: "Driver commands and overrides"
                signals: [
                    "ACCEnabled: Boolean",
                    "SetSpeed: Real (km/h)",
                    "TimeGapSetting: Enum {GAP_1_0, GAP_1_5, GAP_2_0, GAP_2_5}",
                    "BrakePedalPressed: Boolean",
                    "AccelPedalPressed: Boolean"
                ]
            }
            
            provides interface IVehicleCommands {
                description: "Actuator commands for speed control"
                signals: [
                    "TargetAcceleration: Real (m/s²)",
                    "TargetDeceleration: Real (m/s²)",
                    "ThrottleCommand: Real (0-100%)",
                    "BrakeCommand: Real (0-100%)"
                ]
            }
        }

        component SpeedController "Speed Regulation Controller" {
            description: "PID-based speed control when no lead vehicle"
            safety_level: ASIL_B
            parent: ControllerSubsystem
        }

        component DistanceController "Distance Regulation Controller" {
            description: "Maintains safe following distance behind lead vehicle"
            safety_level: ASIL_B
            parent: ControllerSubsystem
        }

        component ModeManager "System Mode Manager" {
            description: "Manages ACC operational states and transitions"
            safety_level: ASIL_B
            parent: ControllerSubsystem
        }

        component ActuationSubsystem "Vehicle Actuation Interface" {
            description: "Interfaces with vehicle powertrain and brake systems"
            safety_level: ASIL_B
            
            requires interface IVehicleCommands
            
            provides interface IActuationStatus {
                signals: [
                    "ActualThrottle: Real (0-100%)",
                    "ActualBrake: Real (0-100%)",
                    "ActuationHealthy: Boolean"
                ]
            }
        }

        component HMISubsystem "Human-Machine Interface" {
            description: "Driver display and interaction"
            safety_level: ASIL_B
            
            provides interface IDriverInput
            
            requires interface ISystemStatus {
                description: "ACC system status for display"
                signals: [
                    "ACCState: Enum {OFF, STANDBY, ACTIVE, WARNING}",
                    "CurrentSetSpeed: Real (km/h)",
                    "CurrentTimeGap: Real (s)",
                    "LeadVehicleDetected: Boolean",
                    "WarningType: Enum {NONE, RAPID_APPROACH, SENSOR_FAILURE}"
                ]
            }
        }

        component SafetyMonitor "Independent Safety Monitor" {
            description: "Watchdog and plausibility checking for ASIL-B compliance"
            safety_level: ASIL_B
            
            requires interface IVehicleCommands
            requires interface IObjectDetection
            
            provides interface ISafetyOverride {
                signals: [
                    "SafetyOverride: Boolean",
                    "FaultDetected: Enum {NONE, SENSOR, CONTROLLER, ACTUATOR}"
                ]
            }
        }

        // Connections
        connect SensingSubsystem.IObjectDetection -> ControllerSubsystem
        connect SensingSubsystem.IObjectDetection -> SafetyMonitor
        connect HMISubsystem.IDriverInput -> ControllerSubsystem
        connect ControllerSubsystem.IVehicleCommands -> ActuationSubsystem
        connect ControllerSubsystem.IVehicleCommands -> SafetyMonitor
    }

    // ========== PHYSICAL ARCHITECTURE ==========
    architecture physical {
        component RadarECU "Radar Electronic Control Unit" {
            description: "77GHz radar processing unit (Continental ARS540)"
            implements: [RadarSensor]
            properties: {
                "Processor": "Infineon AURIX TC397",
                "PowerConsumption": "8W",
                "OperatingTemp": "-40°C to 85°C",
                "CANBusSpeed": "500 kbps"
            }
        }

        component CameraECU "Front Camera ECU" {
            description: "Monocular camera processing unit (Mobileye EyeQ4)"
            implements: [CameraSensor]
            properties: {
                "Processor": "Mobileye EyeQ4",
                "PowerConsumption": "3W",
                "Resolution": "1280x960",
                "FrameRate": "30 fps"
            }
        }

        component ACCMainECU "ACC Main Controller ECU" {
            description: "Main ACC control unit with ASIL-B certified microcontroller"
            implements: [
                SpeedController,
                DistanceController,
                ModeManager,
                ControllerSubsystem
            ]
            properties: {
                "Processor": "Renesas RH850/F1KM (ASIL-D capable, running ASIL-B software)",
                "Memory": "4MB Flash, 512KB RAM",
                "PowerConsumption": "5W",
                "Redundancy": "Dual-core lockstep"
            }
        }

        component SafetyMonitorECU "Independent Safety Monitor" {
            description: "Separate microcontroller for safety monitoring"
            implements: [SafetyMonitor]
            properties: {
                "Processor": "STM32H7 (ASIL-B certified)",
                "PowerConsumption": "2W",
                "WatchdogCycle": "100ms"
            }
        }

        component InstrumentCluster "Digital Instrument Cluster" {
            description: "Driver display and input interface"
            implements: [HMISubsystem]
            properties: {
                "DisplayType": "12.3 inch TFT LCD",
                "Resolution": "1920x720"
            }
        }

        component PowertrainGateway "Powertrain CAN Gateway" {
            description: "Interface to engine and transmission control"
            implements: [ActuationSubsystem]
            properties: {
                "NetworkProtocol": "CAN-FD",
                "DataRate": "2 Mbps"
            }
        }

        // Physical connections via CAN bus
        connect RadarECU -> ACCMainECU via "CAN Bus (500 kbps)"
        connect CameraECU -> ACCMainECU via "CAN Bus (500 kbps)"
        connect ACCMainECU -> PowertrainGateway via "CAN-FD Bus (2 Mbps)"
        connect ACCMainECU -> InstrumentCluster via "CAN Bus (500 kbps)"
        connect ACCMainECU -> SafetyMonitorECU via "SPI (10 MHz)"
        connect SafetyMonitorECU -> PowertrainGateway via "Discrete Signal (Safety Override)"
    }

    // ========== OPERATIONAL SCENARIOS ==========
    scenarios {
        scenario NormalFollowing "Following Lead Vehicle" {
            description: "ACC maintains safe distance behind slower vehicle"
            precondition: "ACC active, lead vehicle detected ahead, ego speed > 30 km/h"
            steps: [
                "RadarSensor detects vehicle 80m ahead traveling at 80 km/h",
                "CameraSensor validates radar detection",
                "DistanceController calculates required time gap (2.0s selected)",
                "System determines current gap is 1.5s (insufficient)",
                "DistanceController commands 0.15g deceleration",
                "PowertrainGateway applies engine braking and light brake",
                "Vehicle decelerates smoothly to maintain 2.0s time gap",
                "InstrumentCluster displays active following mode"
            ]
            postcondition: "Safe 2.0s time gap maintained"
            traces: [SYS-001, SYS-003, SYS-004]
        }

        scenario SpeedControl "Cruising at Set Speed" {
            description: "ACC maintains driver-set speed on clear highway"
            precondition: "ACC active, no lead vehicle, set speed = 120 km/h"
            steps: [
                "SensingSubsystem confirms no vehicles in forward path (200m range)",
                "Current ego speed = 115 km/h (5 km/h below target)",
                "SpeedController calculates throttle command using PID",
                "PowertrainGateway applies 15% throttle",
                "Vehicle accelerates smoothly at 0.12g",
                "Speed reaches 120 km/h (±2 km/h tolerance)",
                "SpeedController maintains steady-state throttle"
            ]
            postcondition: "Vehicle cruising at 120 km/h ±2 km/h"
            traces: [SYS-001, SYS-005]
        }

        scenario DriverBrakeOverride "Driver Brake Override" {
            description: "Driver applies brakes to override ACC"
            precondition: "ACC active and controlling vehicle"
            steps: [
                "Driver presses brake pedal",
                "HMISubsystem detects brake pedal signal within 50ms",
                "ModeManager immediately transitions to STANDBY mode",
                "ControllerSubsystem ceases all throttle/brake commands",
                "Driver has full brake control",
                "InstrumentCluster displays STANDBY status",
                "ACC remains in standby until driver presses RESUME or SET"
            ]
            postcondition: "ACC in standby, driver in full control"
            traces: [SYS-006, STK-003]
        }

        scenario SensorFailure "Radar Sensor Failure Detected" {
            description: "System handles sensor failure safely"
            precondition: "ACC active, both sensors operational"
            steps: [
                "RadarECU detects internal fault (power supply issue)",
                "RadarSensor signals SensorStatus = FAILED",
                "SafetyMonitor detects sensor failure within 100ms",
                "CameraSensor continues operation (degraded mode)",
                "ModeManager evaluates degraded sensing capability",
                "System determines cannot maintain ASIL-B safety level",
                "ControllerSubsystem initiates controlled deactivation",
                "InstrumentCluster displays amber warning: SENSOR FAILURE - ACC UNAVAILABLE",
                "Audible warning chime (3 beeps)",
                "System transitions to OFF state",
                "Driver resumes manual speed control"
            ]
            postcondition: "ACC safely deactivated, driver alerted and in control"
            traces: [SYS-008, SAF-001, SAF-002]
        }
    }

    // ========== TRACEABILITY MATRIX ==========
    traceability {
        // Stakeholder -> System Requirements
        trace STK-001 -> [SYS-001, SYS-005]
        trace STK-002 -> [SYS-002, SYS-003, SYS-004]
        trace STK-003 -> [SYS-006, SYS-008]
        trace STK-004 -> [SYS-007]
        trace STK-005 -> [SYS-009, SYS-010]

        // System -> Safety Requirements
        trace SYS-002 -> [SAF-001]
        trace SYS-004 -> [SAF-004]
        trace SYS-008 -> [SAF-001, SAF-002, SAF-003, SAF-005]

        // Requirements -> Components (Logical)
        trace SYS-001 -> [SpeedController, ControllerSubsystem]
        trace SYS-002 -> [RadarSensor, CameraSensor, SensingSubsystem]
        trace SYS-003 -> [DistanceController]
        trace SYS-004 -> [DistanceController, ActuationSubsystem]
        trace SYS-006 -> [HMISubsystem]
        trace SYS-008 -> [SafetyMonitor, ModeManager]
        trace SYS-009 -> [HMISubsystem]
        trace SAF-001 -> [RadarSensor, CameraSensor, SafetyMonitor]
        trace SAF-005 -> [SafetyMonitor]

        // Logical -> Physical Components
        trace RadarSensor -> [RadarECU]
        trace CameraSensor -> [CameraECU]
        trace SpeedController -> [ACCMainECU]
        trace DistanceController -> [ACCMainECU]
        trace SafetyMonitor -> [SafetyMonitorECU]
    }
}
