model AdaptiveCruiseControl {
    metadata {
        name: "Adaptive Cruise Control System"
        version: "1_0_0"
        author: "System Architect"
        description: "ASIL-B compliant adaptive cruise control system for automotive applications"
        safety_standard: "ISO_26262"
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
            description: "System shall operate in speed range 30 to 180 km_h"
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
            description: "System shall control vehicle speed to match driver-set target speed within 2 km_h tolerance"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-001]
            verification: "Vehicle speed measurement and control accuracy test"
        }

        req SYS-002 "Distance Sensing" {
            description: "System shall detect vehicles in forward path up to 200m with 0_5m accuracy"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-002]
            verification: "Radar sensor accuracy test in multiple weather conditions"
        }

        req SYS-003 "Time Gap Control" {
            description: "System shall maintain configurable time gap (1_0s, 1_5s, 2_0s, 2_5s) from preceding vehicle"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-002]
            verification: "Time gap measurement test"
        }

        req SYS-004 "Brake Intervention" {
            description: "System shall apply braking up to 0_3g deceleration when required to maintain safe distance"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-002]
            verification: "Emergency braking response time test"
        }

        req SYS-005 "Acceleration Control" {
            description: "System shall control throttle for smooth acceleration up to 0_2g"
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
            description: "System shall activate only when speed greater than 30 km_h and driver presses SET button"
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
            description: "System shall display current state (Active, Standby, Off), set speed, and time gap on instrument cluster"
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
            description: "Forward sensing shall use redundant sensors (radar and camera) with cross-validation"
            priority: Critical
            safety_level: ASIL_B
            traces: [SYS-002, SYS-008]
            verification: "Sensor failure mode analysis"
        }

        req SAF-002 "Diagnostic Coverage" {
            description: "System shall achieve greater than 90_percent diagnostic coverage for ASIL-B rated functions"
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
            description: "System shall never command deceleration exceeding 0_4g to prevent instability"
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
                    "ObjectDistance: Real in meters",
                    "ObjectRelativeSpeed: Real in meters_per_second",
                    "ObjectLateralPosition: Real in meters",
                    "DetectionConfidence: Integer (0 to 100 percent)",
                    "SensorStatus: Enum {OK, DEGRADED, FAILED}"
                ]
            }
            
            requires interface IVehicleSpeed {
                description: "Current vehicle speed for relative calculations"
                signals: ["EgoSpeed: Real in meters_per_second"]
            }
        }

        component RadarSensor "77GHz FMCW Radar" {
            description: "Long-range forward radar sensor"
            safety_level: ASIL_B
            parent: SensingSubsystem
            
            provides interface IRadarData {
                signals: [
                    "TargetRange: Real in meters",
                    "TargetVelocity: Real in meters_per_second",
                    "TargetAzimuth: Real in degrees"
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
                    "ObjectWidth: Real in meters",
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
                    "SetSpeed: Real in km_h",
                    "TimeGapSetting: Enum {GAP_1_0, GAP_1_5, GAP_2_0, GAP_2_5}",
                    "BrakePedalPressed: Boolean",
                    "AccelPedalPressed: Boolean"
                ]
            }
            
            provides interface IVehicleCommands {
                description: "Actuator commands for speed control"
                signals: [
                    "TargetAcceleration: Real in meters_per_second_squared",
                    "TargetDeceleration: Real in meters_per_second_squared",
                    "ThrottleCommand: Real (0 to 100 percent)",
                    "BrakeCommand: Real (0 to 100 percent)"
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
                    "ActualThrottle: Real (0 to 100 percent)",
                    "ActualBrake: Real (0 to 100 percent)",
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
                    "CurrentSetSpeed: Real in km_h",
                    "CurrentTimeGap: Real in seconds",
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
            description: "77GHz radar processing unit Continental ARS540"
            implements: [RadarSensor]
            properties: {
                "Processor": "Infineon AURIX TC397",
                "PowerConsumption": "8W",
                "OperatingTemp": "minus_40C_to_85C",
                "CANBusSpeed": "500_kbps"
            }
        }

        component CameraECU "Front Camera ECU" {
            description: "Monocular camera processing unit Mobileye EyeQ4"
            implements: [CameraSensor]
            properties: {
                "Processor": "Mobileye_EyeQ4",
                "PowerConsumption": "3W",
                "Resolution": "1280x960",
                "FrameRate": "30_fps"
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
                "Processor": "Renesas_RH850_F1KM",
                "ProcessorNote": "ASIL-D capable running ASIL-B software",
                "Memory": "4MB_Flash_512KB_RAM",
                "PowerConsumption": "5W",
                "Redundancy": "Dual-core_lockstep"
            }
        }

        component SafetyMonitorECU "Independent Safety Monitor" {
            description: "Separate microcontroller for safety monitoring"
            implements: [SafetyMonitor]
            properties: {
                "Processor": "STM32H7_ASIL_B_certified",
                "PowerConsumption": "2W",
                "WatchdogCycle": "100ms"
            }
        }

        component InstrumentCluster "Digital Instrument Cluster" {
            description: "Driver display and input interface"
            implements: [HMISubsystem]
            properties: {
                "DisplayType": "12_3_inch_TFT_LCD",
                "Resolution": "1920x720"
            }
        }

        component PowertrainGateway "Powertrain CAN Gateway" {
            description: "Interface to engine and transmission control"
            implements: [ActuationSubsystem]
            properties: {
                "NetworkProtocol": "CAN_FD",
                "DataRate": "2_Mbps"
            }
        }

        // Physical connections via CAN bus
        connect RadarECU -> ACCMainECU via "CAN_Bus_500kbps"
        connect CameraECU -> ACCMainECU via "CAN_Bus_500kbps"
        connect ACCMainECU -> PowertrainGateway via "CAN_FD_Bus_2Mbps"
        connect ACCMainECU -> InstrumentCluster via "CAN_Bus_500kbps"
        connect ACCMainECU -> SafetyMonitorECU via "SPI_10MHz"
        connect SafetyMonitorECU -> PowertrainGateway via "Discrete_Signal_Safety_Override"
    }

    // ========== OPERATIONAL SCENARIOS ==========
    scenarios {
        scenario NormalFollowing "Following Lead Vehicle" {
            description: "ACC maintains safe distance behind slower vehicle"
            precondition: "ACC active, lead vehicle detected ahead, ego speed greater than 30 km_h"
            steps: [
                "RadarSensor detects vehicle 80m ahead traveling at 80 km_h",
                "CameraSensor validates radar detection",
                "DistanceController calculates required time gap (2_0s selected)",
                "System determines current gap is 1_5s (insufficient)",
                "DistanceController commands 0_15g deceleration",
                "PowertrainGateway applies engine braking and light brake",
                "Vehicle decelerates smoothly to maintain 2_0s time gap",
                "InstrumentCluster displays active following mode"
            ]
            postcondition: "Safe 2_0s time gap maintained"
            traces: [SYS-001, SYS-003, SYS-004]
        }

        scenario SpeedControl "Cruising at Set Speed" {
            description: "ACC maintains driver-set speed on clear highway"
            precondition: "ACC active, no lead vehicle, set speed equals 120 km_h"
            steps: [
                "SensingSubsystem confirms no vehicles in forward path (200m range)",
                "Current ego speed equals 115 km_h (5 km_h below target)",
                "SpeedController calculates throttle command using PID",
                "PowertrainGateway applies 15_percent throttle",
                "Vehicle accelerates smoothly at 0_12g",
                "Speed reaches 120 km_h (within 2 km_h tolerance)",
                "SpeedController maintains steady-state throttle"
            ]
            postcondition: "Vehicle cruising at 120 km_h within 2 km_h tolerance"
            traces: [SYS-001, SYS-005]
        }

        scenario DriverBrakeOverride "Driver Brake Override" {
            description: "Driver applies brakes to override ACC"
            precondition: "ACC active and controlling vehicle"
            steps: [
                "Driver presses brake pedal",
                "HMISubsystem detects brake pedal signal within 50ms",
                "ModeManager immediately transitions to STANDBY mode",
                "ControllerSubsystem ceases all throttle and brake commands",
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
                "RadarSensor signals SensorStatus equals FAILED",
                "SafetyMonitor detects sensor failure within 100ms",
                "CameraSensor continues operation (degraded mode)",
                "ModeManager evaluates degraded sensing capability",
                "System determines cannot maintain ASIL-B safety level",
                "ControllerSubsystem initiates controlled deactivation",
                "InstrumentCluster displays amber warning for sensor failure",
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
        // Stakeholder to System Requirements
        trace STK-001 -> [SYS-001, SYS-005]
        trace STK-002 -> [SYS-002, SYS-003, SYS-004]
        trace STK-003 -> [SYS-006, SYS-008]
        trace STK-004 -> [SYS-007]
        trace STK-005 -> [SYS-009, SYS-010]

        // System to Safety Requirements
        trace SYS-002 -> [SAF-001]
        trace SYS-004 -> [SAF-004]
        trace SYS-008 -> [SAF-001, SAF-002, SAF-003, SAF-005]

        // Requirements to Components (Logical)
        trace SYS-001 -> [SpeedController, ControllerSubsystem]
        trace SYS-002 -> [RadarSensor, CameraSensor, SensingSubsystem]
        trace SYS-003 -> [DistanceController]
        trace SYS-004 -> [DistanceController, ActuationSubsystem]
        trace SYS-006 -> [HMISubsystem]
        trace SYS-008 -> [SafetyMonitor, ModeManager]
        trace SYS-009 -> [HMISubsystem]
        trace SAF-001 -> [RadarSensor, CameraSensor, SafetyMonitor]
        trace SAF-005 -> [SafetyMonitor]

        // Logical to Physical Components
        trace RadarSensor -> [RadarECU]
        trace CameraSensor -> [CameraECU]
        trace SpeedController -> [ACCMainECU]
        trace DistanceController -> [ACCMainECU]
        trace SafetyMonitor -> [SafetyMonitorECU]
    }
}
