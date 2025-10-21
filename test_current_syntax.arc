model AdaptiveCruiseControl {
    metadata {
        version: "1.0.0"
        author: "System Architect"
    }

    requirements stakeholder {
        req STK-001 "Distance Regulation" {
            description: "System shall maintain safe following distance"
            priority: Critical
        }
    }

    architecture logical {
        component SensingSubsystem "Forward Sensing" {
            description: "Detects vehicles ahead"
            
            provides interface IObjectDetection {
                signals: ["distance", "velocity"]
            }
        }
    }

    traceability {
        trace STK-001 -> [SensingSubsystem]
    }
}
