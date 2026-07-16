model "AdaptiveCruiseControl" {

    requirements stakeholder {
        req STK-001 "The driver shall be able to set desired speed betw..." {
            description: "The driver shall be able to set desired speed between 30 and 180 km/h."
            priority: "High"
        }
    }

    requirements system {
        req SYS-001 "The vehicle shall provide adaptive cruise control ..." {
            description: "The vehicle shall provide adaptive cruise control functionality."
            traces: ["STK-001"]
        }
        req SYS-002 "The system shall maintain safe following distance ..." {
            description: "The system shall maintain safe following distance from preceding vehicle."
            traces: ["STK-001"]
        }
        req SYS-003 "The system shall apply automatic braking with maxi..." {
            description: "The system shall apply automatic braking with maximum 0.3g deceleration."
            traces: ["STK-001"]
        }
        req SYS-004 "The system shall meet ASIL-D safety requirements f..." {
            description: "The system shall meet ASIL-D safety requirements for emergency braking."
            traces: ["STK-001"]
        }
        req SYS-005 "The system shall communicate with engine control u..." {
            description: "The system shall communicate with engine control unit via CAN bus at 500kbps."
            traces: ["STK-001"]
        }
    }

    requirements safety {
        req SAF-001 "The system shall meet ASIL-D safety requirements f..." {
            description: "The system shall meet ASIL-D safety requirements for emergency braking."
            safety_level: "ASIL_D"
        }
    }

    architecture operational {
        actor "User" {
            type: "human"
        }
        actor "System" {
            type: "system"
        }
    }

    architecture logical {
        component ControllerComponent1 "ControllerComponent1" {
            color: "#6495ED"
            stereotype: "<<controller>>"
            requires "SensorInput" {
                protocol: "CAN"
            }
            provides "ControlOutput" {
                protocol: "CAN"
            }
            safety_level: "ASIL_D"
        }
        component SensorComponent2 "SensorComponent2" {
            color: "#70AD47"
            stereotype: "<<sensor>>"
            provides "SensorData" {
                protocol: "CAN"
            }
            safety_level: "ASIL_D"
        }
        component SensorComponent3 "SensorComponent3" {
            color: "#70AD47"
            stereotype: "<<sensor>>"
            provides "SensorData" {
                protocol: "CAN"
            }
            safety_level: "ASIL_D"
        }
        component ActuatorComponent4 "ActuatorComponent4" {
            color: "#ED7D31"
            stereotype: "<<actuator>>"
            requires "CommandInput" {
                protocol: "CAN"
            }
            safety_level: "ASIL_D"
        }
        component ControllerComponent5 "ControllerComponent5" {
            color: "#6495ED"
            stereotype: "<<controller>>"
            requires "SensorInput" {
                protocol: "CAN"
            }
            provides "ControlOutput" {
                protocol: "CAN"
            }
            safety_level: "ASIL_D"
        }
    }

    architecture physical {
        node "MainECU" {
            type: "ECU"
            processor: "ARM Cortex-A53"
            memory: "4GB RAM"
        }
    }

}
