// Layout demo: a small cruise control model written to exercise the diagram
// layout engine, with components that carry many ports on both sides, a
// nested component and exchanges that fan in and out. It is a complete model
// all the same: every component performs functions, every requirement is
// satisfied and verified.
//
//   arclang workbench examples/elk_advanced_demo.arc
//   arclang diagram examples/elk_advanced_demo.arc -f elk -o graphs.json

model LayoutDemo {
    name: "Cruise control layout demo"
    version: "2.0"
}

requirements system {
    req "SYS-001" "Speed hold" { description: "The system shall hold the set speed within 2 km/h" priority: "Critical" }
    req "SYS-002" "Obstacle detection" { description: "The system shall detect obstacles within a 150 m range" priority: "Critical" }
    req "SYS-003" "Safety alert" { description: "A detected fault shall be announced to the driver within 500 ms" priority: "High" }
}

operational_analysis "Vehicle System" {
    actor "Driver" { id: "ACT-001" description: "Person operating the vehicle" }
    entity "Vehicle" { id: "ACT-002" activity "Cruise at a set speed" { id: "OA-001" } }
    operational_capability "Cruise" { id: "OC-001" involves: ["ACT-001", "ACT-002"] }
}

system_analysis "Advanced Cruise Control" {
    capability "Cruise control" { id: "CAP-001" realizes: "OC-001" involves: ["SF-001", "SF-002", "SF-003"] }
    function "Read the driver commands" { id: "SF-001" port out "target" }
    function "Measure the distance ahead" { id: "SF-002" port out "distance" }
    function "Regulate the speed" { id: "SF-003" port in "target" port in "distance" port out "throttle" port out "brake" }
    function "Supervise the system" { id: "SF-004" }
    functional_exchange "target speed" { id: "FE-001" from: "SF-001.target" to: "SF-003.target" }
    functional_exchange "distance" { id: "FE-002" from: "SF-002.distance" to: "SF-003.distance" }
}

logical_architecture "ACC Control System" {
    component "User Interface" {
        id: "LA-001"
        type: "Software"
        port in "DriverCommands" { protocol: "CAN" }
        port in "NetworkStatus" { protocol: "CAN" }
        port out "TargetSpeed"
        port out "SystemStatus" { protocol: "CAN" }
        function "Read the driver commands" { id: "LF-001" realizes: "SF-001" }
        function "Show the system status" { id: "LF-002" }
    }
    component "Speed Controller" {
        id: "LA-002"
        type: "Software"
        port in "TargetSpeed"
        port in "CurrentSpeed" { protocol: "CAN" }
        port in "DistanceMeasurement" { protocol: "CAN" }
        port in "SafetyLimits"
        port out "ThrottleCommand" { protocol: "CAN" }
        port out "BrakeCommand" { protocol: "CAN" }
        port out "ControllerState"
        function "Regulate the speed" { id: "LF-003" realizes: "SF-003" }
        function "Hold the distance" { id: "LF-004" realizes: "SF-003" }
    }
    component "Radar System" {
        id: "LA-003"
        type: "Hardware"
        port in "SensorData" { protocol: "SPI" }
        port out "DistanceMeasurement" { protocol: "CAN" }
        port out "DebugPort" { protocol: "UART" }
        function "Measure the distance ahead" { id: "LF-005" realizes: "SF-002" }
        component "Radar Front End" {
            id: "LA-003-1"
            port out "SensorData" { protocol: "SPI" }
            function "Digitise the echoes" { id: "LF-006" }
        }
    }
    component "Safety Monitor" {
        id: "LA-004"
        type: "Software"
        port in "ControllerState"
        port in "DistanceMeasurement"
        port out "SafetyLimits"
        port out "SafetyAlert" { protocol: "CAN" }
        function "Supervise the controller" { id: "LF-007" realizes: "SF-004" }
        function "Raise the safety alert" { id: "LF-008" realizes: "SF-004" }
    }
    component "Vehicle Network" {
        id: "LA-005"
        type: "Hardware"
        port in "ThrottleCommand" { protocol: "CAN" }
        port in "BrakeCommand" { protocol: "CAN" }
        port in "SafetyAlert" { protocol: "CAN" }
        port in "SystemStatus" { protocol: "CAN" }
        port out "CurrentSpeed" { protocol: "CAN" }
        port out "NetworkStatus" { protocol: "CAN" }
        function "Route the CAN frames" { id: "LF-009" realizes: "SF-003" }
    }
    component_exchange "Radar front end" { from_port: "LA-003-1.SensorData" to_port: "LA-003.SensorData" }
    component_exchange "Target speed" { from_port: "LA-001.TargetSpeed" to_port: "LA-002.TargetSpeed" }
    component_exchange "Distance to controller" { from_port: "LA-003.DistanceMeasurement" to_port: "LA-002.DistanceMeasurement" }
    component_exchange "Distance to monitor" { from_port: "LA-003.DistanceMeasurement" to_port: "LA-004.DistanceMeasurement" }
    component_exchange "Controller state" { from_port: "LA-002.ControllerState" to_port: "LA-004.ControllerState" }
    component_exchange "Safety limits" { from_port: "LA-004.SafetyLimits" to_port: "LA-002.SafetyLimits" }
    component_exchange "Throttle" { from_port: "LA-002.ThrottleCommand" to_port: "LA-005.ThrottleCommand" }
    component_exchange "Brake" { from_port: "LA-002.BrakeCommand" to_port: "LA-005.BrakeCommand" }
    component_exchange "Safety alert" { from_port: "LA-004.SafetyAlert" to_port: "LA-005.SafetyAlert" }
    component_exchange "System status" { from_port: "LA-001.SystemStatus" to_port: "LA-005.SystemStatus" }
    component_exchange "Current speed" { from_port: "LA-005.CurrentSpeed" to_port: "LA-002.CurrentSpeed" }
    component_exchange "Network status" { from_port: "LA-005.NetworkStatus" to_port: "LA-001.NetworkStatus" }
}

physical_architecture "ECU Deployment" {
    node "Central ECU" {
        id: "PA-001"
        port "PowerSupply" { connector: "12 V" }
        deploys "LA-001"
        deploys "LA-002"
        deploys "LA-004"
        deploys "LA-005"
    }
    node "Sensor ECU" {
        id: "PA-002"
        deploys "LA-003"
    }
    physical_link "Sensor CAN" { id: "PL-001" from: "PA-002" to: "PA-001" protocol: "CAN" bandwidth: 500 kbps }
}

test_case "TC-001" { verifies: ["SYS-001"] method: "test" description: "Speed held on the chassis dynamometer" }
test_case "TC-002" { verifies: ["SYS-002"] method: "test" description: "Range measured on the proving ground" }
test_case "TC-003" { verifies: ["SYS-003"] method: "test" description: "Fault injected, alert latency measured" }

trace "LA-001" satisfies "SYS-001" { rationale: "Set speed read from the driver" }
trace "LA-002" satisfies "SYS-001" { rationale: "Speed regulation within 2 km/h" }
trace "LA-003" satisfies "SYS-002" { rationale: "Radar range" }
trace "LA-003-1" satisfies "SYS-002" { rationale: "Front end sensitivity" }
trace "LA-004" satisfies "SYS-003" { rationale: "Fault to alert within 500 ms" }
trace "LA-005" satisfies "SYS-003" { rationale: "Alert frame routed to the cluster" }
trace "SF-003" realizes "OA-001"

diagram "Fan-in and fan-out" {
    view: lab
    shows: ["LA-002", "LA-004", "LA-005"]
    context: true
}
