model EmergencyBrakingSensorFusion {
    metadata {
        name: "Emergency Braking Controller with Sensor Fusion"
        version: "2.0.0"
        author: "MBSE Safety Architect"
        description: "ASIL-D compliant emergency braking with multi-sensor fusion (Radar, Camera, LiDAR)"
        safety_standard: "ISO_26262"
    }

    // ========== STAKEHOLDER REQUIREMENTS ==========
    requirements stakeholder {
        req STK-001 "Vehicle Safety" {
            description: "The system shall prevent collisions by automatically applying brakes when obstacles are detected within safe distance"
            priority: Critical
            safety_level: ASIL_D
            rationale: "Primary safety goal - prevent collisions"
        }
        
        req STK-002 "Sensor Redundancy" {
            description: "The system shall use multiple independent sensor types (Radar, Camera, LiDAR) to ensure reliable obstacle detection"
            priority: Critical
            safety_level: ASIL_D
            rationale: "Redundancy for fail-operational capability"
        }
        
        req STK-003 "Driver Override" {
            description: "The driver shall be able to override automatic braking at any time via accelerator pedal"
            priority: Critical
            safety_level: ASIL_B
            rationale: "Driver must retain ultimate control"
        }
        
        req STK-004 "Response Time" {
            description: "The system shall initiate braking within 200ms of obstacle detection"
            priority: Critical
            safety_level: ASIL_D
            rationale: "Time-critical safety function"
        }
    }

    // ========== SYSTEM REQUIREMENTS ==========
    requirements system {
        req SYS-001 "Radar Detection Range" {
            description: "Radar shall detect obstacles up to 200m range with 0_5m accuracy"
            priority: Critical
            safety_level: ASIL_D
            traces: [STK-001, STK-002]
            verification: "Radar accuracy test in multiple weather conditions"
        }

        req SYS-002 "Camera Object Classification" {
            description: "Camera shall classify detected objects (vehicle, pedestrian, cyclist, static) with 95_percent accuracy"
            priority: Critical
            safety_level: ASIL_C
            traces: [STK-001, STK-002]
            verification: "Object classification validation test"
        }

        req SYS-003 "LiDAR 3D Mapping" {
            description: "LiDAR shall provide 3D point cloud with cm-level accuracy for precise object dimensions"
            priority: Critical
            safety_level: ASIL_C
            traces: [STK-001, STK-002]
            verification: "3D mapping accuracy test"
        }

        req SYS-004 "Sensor Fusion Algorithm" {
            description: "System shall fuse Radar, Camera, and LiDAR data using Kalman filtering with 99_9_percent reliability"
            priority: Critical
            safety_level: ASIL_D
            traces: [STK-002]
            verification: "Sensor fusion accuracy test with fault injection"
        }

        req SYS-005 "Threat Assessment" {
            description: "System shall calculate Time-To-Collision (TTC) with 50ms update rate"
            priority: Critical
            safety_level: ASIL_D
            traces: [STK-001, STK-004]
            verification: "TTC calculation accuracy test"
        }

        req SYS-006 "Emergency Braking Force" {
            description: "System shall command up to 1_0g deceleration for emergency braking"
            priority: Critical
            safety_level: ASIL_D
            traces: [STK-001]
            verification: "Maximum deceleration test"
        }

        req SYS-007 "Driver Override Detection" {
            description: "System shall detect accelerator pedal application within 30ms"
            priority: Critical
            safety_level: ASIL_B
            traces: [STK-003]
            verification: "Pedal sensor response time test"
        }

        req SYS-008 "Fail-Operational Mode" {
            description: "System shall maintain operation with single sensor failure (degraded mode)"
            priority: Critical
            safety_level: ASIL_D
            traces: [STK-002]
            verification: "Single fault tolerance test"
        }
    }

    // ========== SAFETY REQUIREMENTS ==========
    requirements safety {
        req SAF-001 "Triple Sensor Redundancy" {
            description: "Forward sensing shall use 3 independent sensors with cross-validation"
            priority: Critical
            safety_level: ASIL_D
            traces: [SYS-001, SYS-002, SYS-003, SYS-004]
            verification: "Sensor failure mode analysis"
        }

        req SAF-002 "Diagnostic Coverage" {
            description: "System shall achieve greater than 99_percent diagnostic coverage for ASIL-D rated functions"
            priority: Critical
            safety_level: ASIL_D
            traces: [SYS-008]
            verification: "Diagnostic coverage analysis"
        }

        req SAF-003 "Fail-Operational Time" {
            description: "System shall maintain safe operation for minimum 2000ms after single sensor failure"
            priority: Critical
            safety_level: ASIL_D
            traces: [SYS-008]
            verification: "Fault tolerance timing test"
        }

        req SAF-004 "Watchdog Monitoring" {
            description: "Independent safety monitor shall supervise main controller with 50ms timeout"
            priority: Critical
            safety_level: ASIL_D
            verification: "Watchdog timing verification"
        }
    }

    // ========== OPERATIONAL ANALYSIS ==========
    operational_analysis "Emergency Braking Operational Context" {
        actor "Driver" {
            id: "OA-ACT-001"
            description: "Human operator controlling the vehicle"
            category: "Human"
        }
        
        actor "Vehicle System" {
            id: "OA-ACT-002"
            description: "Automated emergency braking with sensor fusion"
            category: "System"
            safety_level: ASIL_D
        }
        
        actor "Road Environment" {
            id: "OA-ACT-003"
            description: "Physical environment with obstacles and hazards"
            category: "External"
        }
        
        actor "Other Vehicles" {
            id: "OA-ACT-004"
            description: "Surrounding traffic participants"
            category: "External"
        }
        
        actor "Pedestrians" {
            id: "OA-ACT-005"
            description: "Vulnerable road users"
            category: "External"
        }
        
        operational_activity "Monitor Environment" {
            id: "OA-01"
            description: "Driver continuously monitors road and traffic conditions"
            performed_by: "OA-ACT-001"
        }
        
        operational_activity "Control Vehicle" {
            id: "OA-02"
            description: "Driver controls speed, steering, and braking manually"
            performed_by: "OA-ACT-001"
        }
        
        operational_activity "Scan Environment" {
            id: "OA-03"
            description: "System continuously scans for collision risks using sensors"
            performed_by: "OA-ACT-002"
            safety_level: ASIL_C
        }
        
        operational_activity "Fuse Sensor Data" {
            id: "OA-04"
            description: "System fuses Radar, Camera, and LiDAR data for reliable detection"
            performed_by: "OA-ACT-002"
            safety_level: ASIL_D
        }
        
        operational_activity "Detect Collision Risk" {
            id: "OA-05"
            description: "System detects potential collisions and calculates TTC"
            performed_by: "OA-ACT-002"
            safety_level: ASIL_D
        }
        
        operational_activity "Alert Driver" {
            id: "OA-06"
            description: "System provides visual, audible, and haptic collision warnings"
            performed_by: "OA-ACT-002"
            safety_level: ASIL_B
        }
        
        operational_activity "Apply Emergency Brake" {
            id: "OA-07"
            description: "System automatically applies maximum braking force"
            performed_by: "OA-ACT-002"
            safety_level: ASIL_D
        }
        
        operational_activity "Override System" {
            id: "OA-08"
            description: "Driver overrides automatic braking by pressing accelerator"
            performed_by: "OA-ACT-001"
        }
        
        operational_activity "Present Hazards" {
            id: "OA-09"
            description: "Environment creates static obstacles (walls, barriers)"
            performed_by: "OA-ACT-003"
        }
        
        operational_activity "Navigate Traffic" {
            id: "OA-10"
            description: "Other vehicles move dynamically in traffic"
            performed_by: "OA-ACT-004"
        }
        
        operational_activity "Cross Road" {
            id: "OA-11"
            description: "Pedestrians enter vehicle path at crosswalks"
            performed_by: "OA-ACT-005"
        }
        
        operational_interaction "Hazard Detection" {
            id: "OI-01"
            from: "OA-ACT-003"
            to: "OA-ACT-002"
            exchange_item_kind: FLOW
            description: "Continuous obstacle position and velocity data"
        }
        
        operational_interaction "Driver Alert" {
            id: "OI-02"
            from: "OA-ACT-002"
            to: "OA-ACT-001"
            exchange_item_kind: EVENT
            description: "Collision warning alert (visual, audible, haptic)"
        }
        
        operational_interaction "Driver Override Command" {
            id: "OI-03"
            from: "OA-ACT-001"
            to: "OA-ACT-002"
            exchange_item_kind: EVENT
            description: "Driver cancels automatic braking via accelerator"
        }
        
        operational_interaction "Braking Execution" {
            id: "OI-04"
            from: "OA-ACT-002"
            to: "OA-ACT-001"
            exchange_item_kind: FLOW
            description: "Automatic brake application (deceleration force)"
        }
    }

    // ========== LOGICAL ARCHITECTURE ==========
    architecture logical {
        component SensorFusionSubsystem "Sensor Fusion Subsystem" {
            description: "Multi-sensor perception with Kalman filtering"
            safety_level: ASIL_D
            
            provides interface IFusedEnvironmentModel {
                description: "Fused obstacle data with confidence levels"
                signals: [
                    "FusedObjectList: Array",
                    "ObjectDistance: Real in meters",
                    "ObjectVelocity: Real in meters_per_second",
                    "ObjectClassification: Enum {VEHICLE, PEDESTRIAN, CYCLIST, STATIC}",
                    "DetectionConfidence: Integer (0 to 100 percent)",
                    "SensorHealthStatus: Enum {ALL_OK, ONE_FAILED, TWO_FAILED, CRITICAL_FAILURE}"
                ]
            }
            
            requires interface IRadarData {
                description: "Raw radar sensor data"
                signals: [
                    "TargetRange: Real in meters",
                    "TargetVelocity: Real in meters_per_second",
                    "TargetAzimuth: Real in degrees",
                    "RadarHealthy: Boolean"
                ]
            }
            
            requires interface ICameraData {
                description: "Vision-based object detection"
                signals: [
                    "ObjectDetected: Boolean",
                    "ObjectType: Enum",
                    "ObjectWidth: Real in meters",
                    "CameraHealthy: Boolean"
                ]
            }
            
            requires interface ILidarData {
                description: "3D point cloud data"
                signals: [
                    "PointCloud: Array",
                    "Object3DDimensions: Array (length, width, height)",
                    "LidarHealthy: Boolean"
                ]
            }
        }

        component RadarSensor "77GHz FMCW Radar" {
            description: "Long-range forward radar sensor (200m)"
            safety_level: ASIL_D
            parent: SensorFusionSubsystem
            
            provides interface IRadarData
        }

        component CameraSensor "Forward Monocular Camera" {
            description: "Vision-based object detection and classification"
            safety_level: ASIL_C
            parent: SensorFusionSubsystem
            
            provides interface ICameraData
        }

        component LidarSensor "Automotive LiDAR Sensor" {
            description: "3D laser scanner for precise object dimensions"
            safety_level: ASIL_C
            parent: SensorFusionSubsystem
            
            provides interface ILidarData
        }

        component ThreatAssessmentUnit "Threat Assessment Unit" {
            description: "Calculates Time-To-Collision and collision probability"
            safety_level: ASIL_D
            
            requires interface IFusedEnvironmentModel
            
            provides interface IThreatData {
                description: "Collision threat assessment"
                signals: [
                    "TimeToCollision: Real in seconds",
                    "CollisionProbability: Real (0_0 to 1_0)",
                    "ThreatLevel: Enum {NONE, LOW, MEDIUM, HIGH, CRITICAL}",
                    "RecommendedAction: Enum {MONITOR, WARN, BRAKE}"
                ]
            }
        }

        component BrakingDecisionUnit "Braking Decision Logic" {
            description: "Decides when and how to apply emergency braking"
            safety_level: ASIL_D
            
            requires interface IThreatData
            requires interface IDriverInput {
                description: "Driver override signals"
                signals: [
                    "AcceleratorPressed: Boolean",
                    "BrakePedalPressed: Boolean",
                    "DriverOverride: Boolean"
                ]
            }
            
            provides interface IBrakingCommand {
                description: "Brake actuation commands"
                signals: [
                    "EmergencyBrakeActive: Boolean",
                    "TargetDeceleration: Real in meters_per_second_squared",
                    "BrakePressure: Real (0 to 100 percent)"
                ]
            }
        }

        component BrakeActuatorInterface "Brake Actuator Interface" {
            description: "Interfaces with vehicle hydraulic brake system"
            safety_level: ASIL_D
            
            requires interface IBrakingCommand
            
            provides interface IActuationStatus {
                signals: [
                    "ActualBrakePressure: Real (0 to 100 percent)",
                    "ActualDeceleration: Real in meters_per_second_squared",
                    "ActuatorHealthy: Boolean"
                ]
            }
        }

        component DriverAlertHMI "Driver Alert Interface" {
            description: "Visual, audible, and haptic warnings"
            safety_level: ASIL_B
            
            requires interface IThreatData
            
            provides interface IWarningFeedback {
                signals: [
                    "VisualWarningActive: Boolean",
                    "AudibleAlertActive: Boolean",
                    "HapticFeedbackActive: Boolean",
                    "WarningLevel: Enum {OFF, LOW, MEDIUM, HIGH}"
                ]
            }
        }

        component SafetyMonitor "Independent Safety Monitor" {
            description: "Watchdog and plausibility checking for ASIL-D compliance"
            safety_level: ASIL_D
            
            requires interface IFusedEnvironmentModel
            requires interface IBrakingCommand
            requires interface IActuationStatus
            
            provides interface ISafetyOverride {
                signals: [
                    "SafetyOverride: Boolean",
                    "FaultDetected: Enum {NONE, SENSOR, FUSION, DECISION, ACTUATOR}",
                    "DegradedModeActive: Boolean"
                ]
            }
        }

        // Connections
        connect RadarSensor.IRadarData -> SensorFusionSubsystem
        connect CameraSensor.ICameraData -> SensorFusionSubsystem
        connect LidarSensor.ILidarData -> SensorFusionSubsystem
        connect SensorFusionSubsystem.IFusedEnvironmentModel -> ThreatAssessmentUnit
        connect SensorFusionSubsystem.IFusedEnvironmentModel -> SafetyMonitor
        connect ThreatAssessmentUnit.IThreatData -> BrakingDecisionUnit
        connect ThreatAssessmentUnit.IThreatData -> DriverAlertHMI
        connect BrakingDecisionUnit.IBrakingCommand -> BrakeActuatorInterface
        connect BrakingDecisionUnit.IBrakingCommand -> SafetyMonitor
        connect BrakeActuatorInterface.IActuationStatus -> SafetyMonitor
    }

    // ========== PHYSICAL ARCHITECTURE ==========
    architecture physical {
        component RadarECU "Radar Electronic Control Unit" {
            description: "77GHz radar processing unit Continental ARS540"
            implements: [RadarSensor]
            safety_level: ASIL_D
            properties: {
                "Processor": "Infineon AURIX TC397",
                "PowerConsumption": "8W",
                "OperatingTemp": "minus40C_to_85C",
                "CANBusSpeed": "500_kbps",
                "MaxRange": "200_meters",
                "RangeAccuracy": "0_5_meters"
            }
        }

        component CameraECU "Front Camera ECU" {
            description: "Monocular camera processing unit Mobileye EyeQ5"
            implements: [CameraSensor]
            safety_level: ASIL_C
            properties: {
                "Processor": "Mobileye_EyeQ5",
                "PowerConsumption": "4W",
                "Resolution": "1920x1200",
                "FrameRate": "36_fps",
                "FieldOfView": "50_degrees"
            }
        }

        component LidarECU "LiDAR Control Unit" {
            description: "Automotive LiDAR unit Luminar Iris Plus"
            implements: [LidarSensor]
            safety_level: ASIL_C
            properties: {
                "Processor": "Custom_FPGA",
                "PowerConsumption": "12W",
                "LaserWavelength": "1550_nm",
                "PointsPerSecond": "300k",
                "MaxRange": "250_meters",
                "RangeAccuracy": "2_cm"
            }
        }

        component EmergencyBrakeECU "Emergency Brake Main Controller" {
            description: "Main ASIL-D controller for sensor fusion and braking decision"
            implements: [
                SensorFusionSubsystem,
                ThreatAssessmentUnit,
                BrakingDecisionUnit
            ]
            safety_level: ASIL_D
            properties: {
                "Processor": "Renesas_RH850_F1KM_R7F7016643",
                "ProcessorNote": "ASIL-D certified dual-core lockstep",
                "Memory": "8MB_Flash_1MB_RAM",
                "PowerConsumption": "7W",
                "Redundancy": "Dual-core_lockstep_with_ECC",
                "ProcessingCycle": "10ms"
            }
        }

        component SafetyMonitorECU "Independent Safety Monitor ECU" {
            description: "Separate ASIL-D microcontroller for safety monitoring"
            implements: [SafetyMonitor]
            safety_level: ASIL_D
            properties: {
                "Processor": "STM32H7_ASIL_D_certified",
                "PowerConsumption": "2W",
                "WatchdogCycle": "50ms",
                "Architecture": "Independent_from_main_ECU"
            }
        }

        component InstrumentCluster "Digital Instrument Cluster" {
            description: "Driver display and warning interface"
            implements: [DriverAlertHMI]
            safety_level: ASIL_B
            properties: {
                "DisplayType": "12_3_inch_TFT_LCD",
                "Resolution": "1920x720",
                "RefreshRate": "60Hz"
            }
        }

        component BrakeHydraulicUnit "Electronic Brake Booster" {
            description: "Electro-hydraulic brake actuation system"
            implements: [BrakeActuatorInterface]
            safety_level: ASIL_D
            properties: {
                "Type": "Electronic_Brake_Booster_iBooster_2",
                "MaxPressure": "180_bar",
                "ResponseTime": "150ms",
                "Redundancy": "Dual_motor_dual_pressure_sensor"
            }
        }

        // Physical connections via CAN/Ethernet
        connect RadarECU -> EmergencyBrakeECU via "CAN_FD_Bus_2Mbps"
        connect CameraECU -> EmergencyBrakeECU via "CAN_FD_Bus_2Mbps"
        connect LidarECU -> EmergencyBrakeECU via "Automotive_Ethernet_100Mbps"
        connect EmergencyBrakeECU -> BrakeHydraulicUnit via "CAN_FD_Bus_5Mbps_Safety_Critical"
        connect EmergencyBrakeECU -> InstrumentCluster via "CAN_Bus_500kbps"
        connect EmergencyBrakeECU -> SafetyMonitorECU via "SPI_20MHz_Dedicated"
        connect SafetyMonitorECU -> BrakeHydraulicUnit via "Discrete_Signal_Safety_Override"
    }

    // ========== TRACEABILITY MATRIX ==========
    traceability {
        // Stakeholder to System Requirements
        trace STK-001 -> [SYS-001, SYS-005, SYS-006]
        trace STK-002 -> [SYS-001, SYS-002, SYS-003, SYS-004, SYS-008]
        trace STK-003 -> [SYS-007]
        trace STK-004 -> [SYS-005]

        // System to Safety Requirements
        trace SYS-001 -> [SAF-001]
        trace SYS-002 -> [SAF-001]
        trace SYS-003 -> [SAF-001]
        trace SYS-004 -> [SAF-001]
        trace SYS-008 -> [SAF-001, SAF-002, SAF-003]

        // Requirements to Logical Components
        trace SYS-001 -> [RadarSensor]
        trace SYS-002 -> [CameraSensor]
        trace SYS-003 -> [LidarSensor]
        trace SYS-004 -> [SensorFusionSubsystem]
        trace SYS-005 -> [ThreatAssessmentUnit]
        trace SYS-006 -> [BrakingDecisionUnit, BrakeActuatorInterface]
        trace SYS-007 -> [BrakingDecisionUnit]
        trace SYS-008 -> [SafetyMonitor, SensorFusionSubsystem]
        trace SAF-001 -> [RadarSensor, CameraSensor, LidarSensor, SensorFusionSubsystem]
        trace SAF-002 -> [SafetyMonitor]
        trace SAF-003 -> [SafetyMonitor, SensorFusionSubsystem]
        trace SAF-004 -> [SafetyMonitor]

        // Logical to Physical Components
        trace RadarSensor -> [RadarECU]
        trace CameraSensor -> [CameraECU]
        trace LidarSensor -> [LidarECU]
        trace SensorFusionSubsystem -> [EmergencyBrakeECU]
        trace ThreatAssessmentUnit -> [EmergencyBrakeECU]
        trace BrakingDecisionUnit -> [EmergencyBrakeECU]
        trace SafetyMonitor -> [SafetyMonitorECU]
        trace DriverAlertHMI -> [InstrumentCluster]
        trace BrakeActuatorInterface -> [BrakeHydraulicUnit]
    }
}
