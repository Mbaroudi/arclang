# Change impact: the proposed change (SF-FuseData)

25 impacted: 4 blockers, 12 majors, 9 minors · gate ISO26262 PASS → FAIL  
Model: <path>

## Impact

| Element | Name | Kind | Layer | Category | Severity | Distance | Reason | Why |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| FC-001 | EmergencyBrakingChain | FunctionalChain | SA | budget | blocker | 1 | new gate blocker: functional_chain 'EmergencyBrakingChain': sum of function latencies 145.0 ms exceeds the 100.0 ms budget | SF-FuseData --involved in--> FC-001 |
| CST-001 | Braking chain within budget | Constraint | Transverse | budget | blocker | 2 | new gate blocker: constraint 'Braking chain within budget' is violated: `sum("FC-001", latency) <= "FC-001".latency_budget` — left side is 145 ms, right side is 100 ms | SF-FuseData --involved in--> FC-001 --constrained by--> CST-001 |
| REQ-AEB-002 | Sensor fusion latency | Requirement | SA | requirement | blocker | 3 | ASIL-D requirement: safety re-assessment | SF-FuseData --realized by--> Fuse sensor data --performed by--> LC-SensorFusion --satisfies--> REQ-AEB-002 |
| HAZ-002 | No braking on imminent collision | Hazard | Transverse | safety | blocker | 4 | ASIL-C safety item: safety re-assessment | SF-FuseData --realized by--> Fuse sensor data --performed by--> LC-SensorFusion --satisfies--> REQ-AEB-002 --mitigates--> HAZ-002 |
| FusionToTracking | FusionToTracking | FunctionalExchange | SA | interface | major | 1 | interface change: ICD to update | SF-FuseData --sends--> FusionToTracking |
| SF-FuseData.fused_data | fused_data | FunctionPort | SA | interface | major | 1 | interface change: ICD to update | SF-FuseData --has port--> SF-FuseData.fused_data |
| SF-FuseData.raw_sensor_data | raw_sensor_data | FunctionPort | SA | interface | major | 1 | interface change: ICD to update | SF-FuseData --has port--> SF-FuseData.raw_sensor_data |
| SensorToFusion | SensorToFusion | FunctionalExchange | SA | interface | major | 1 | interface change: ICD to update | SF-FuseData --receives--> SensorToFusion |
| LC-SensorFusion | SensorFusion | LogicalComponent | LA | architecture | major | 2 | ASIL-D element: safety re-assessment | SF-FuseData --realized by--> Fuse sensor data --performed by--> LC-SensorFusion |
| SF-AcquireSensorData | AcquireSensorData | SystemFunction | SA | architecture | major | 2 | ASIL-D element: safety re-assessment | SF-FuseData --receives--> SensorToFusion --sent by--> SF-AcquireSensorData |
| SF-AcquireSensorData.raw_sensor_data | raw_sensor_data | FunctionPort | SA | interface | major | 2 | interface change: ICD to update | SF-FuseData --receives--> SensorToFusion --sent by--> SF-AcquireSensorData.raw_sensor_data |
| SF-TrackObjects | TrackObjects | SystemFunction | SA | architecture | major | 2 | ASIL-D element: safety re-assessment | SF-FuseData --sends--> FusionToTracking --received by--> SF-TrackObjects |
| SF-TrackObjects.fused_data | fused_data | FunctionPort | SA | interface | major | 2 | interface change: ICD to update | SF-FuseData --sends--> FusionToTracking --received by--> SF-TrackObjects.fused_data |
| LC-RadarSensor | RadarSensor | LogicalComponent | LA | architecture | major | 3 | ASIL-D element: safety re-assessment | SF-FuseData --realized by--> Fuse sensor data --performed by--> LC-SensorFusion --receives 'raw radar targets' from--> LC-RadarSensor |
| PN-CentralADASECU | CentralADASECU | PhysicalNode | PA | architecture | major | 3 | ASIL-D element: safety re-assessment | SF-FuseData --realized by--> Fuse sensor data --performed by--> LC-SensorFusion --deployed on--> PN-CentralADASECU |
| TC-002 | Fusion latency measurement | TestCase | Transverse | test | major | 4 | verification to redo | SF-FuseData --realized by--> Fuse sensor data --performed by--> LC-SensorFusion --satisfies--> REQ-AEB-002 --verified by--> TC-002 |
| CAP-001 | EmergencyBraking | Capability | SA | architecture | minor | 1 | to review | SF-FuseData --involved in--> CAP-001 |
| Fuse sensor data | Fuse sensor data | LogicalFunction | LA | architecture | minor | 1 | to review | SF-FuseData --realized by--> Fuse sensor data |
| MIS-001 | SafeAutonomousBraking | Mission | SA | architecture | minor | 2 | to review | SF-FuseData --involved in--> CAP-001 --contributes to--> MIS-001 |
| OC-001 | Avoid Collisions | OperationalCapability | OA | architecture | minor | 2 | to review | SF-FuseData --involved in--> CAP-001 --realizes--> OC-001 |
| diagram:Braking chain | Braking chain | Diagram | LA | view | minor | 2 | view to refresh | SF-FuseData --realized by--> Fuse sensor data --shown in--> diagram:Braking chain |
| OA-ACT-001 | Driver | Actor | OA | architecture | minor | 3 | to review | SF-FuseData --involved in--> CAP-001 --realizes--> OC-001 --involves--> OA-ACT-001 |
| OA-ACT-002 | Pedestrian | Actor | OA | architecture | minor | 3 | to review | SF-FuseData --involved in--> CAP-001 --realizes--> OC-001 --involves--> OA-ACT-002 |
| OA-ACT-003 | TrafficEnvironment | Actor | OA | architecture | minor | 3 | to review | SF-FuseData --involved in--> CAP-001 --realizes--> OC-001 --involves--> OA-ACT-003 |
| OE-Vehicle | Vehicle | OperationalEntity | OA | architecture | minor | 3 | to review | SF-FuseData --involved in--> CAP-001 --realizes--> OC-001 --involves--> OE-Vehicle |

## Gate

| Change | Blocker |
| --- | --- |
| new blocker | functional_chain 'EmergencyBrakingChain': sum of function latencies 145.0 ms exceeds the 100.0 ms budget |
| new blocker | constraint 'Braking chain within budget' is violated: `sum("FC-001", latency) <= "FC-001".latency_budget` — left side is 145 ms, right side is 100 ms |
| gate | ISO26262 PASS → FAIL |

## Constraints

| Constraint | Name | Before | After | Value | Bound | Expression |
| --- | --- | --- | --- | --- | --- | --- |
| CST-001 | Braking chain within budget | holds | violated | 90 ms → 145 ms | 100 ms | sum("FC-001", latency) <= "FC-001".latency_budget |

## Budgets

| Chain | Name | Budget | Before | After | Without latency |
| --- | --- | --- | --- | --- | --- |
| FC-001 | EmergencyBrakingChain | 100 ms | 90 ms | 145 ms |  |

## Coverage

| Requirement | Name | Before | After | Satisfied by | Verified by |
| --- | --- | --- | --- | --- | --- |

## Changes

| Change | Element | Kind | Detail |
| --- | --- | --- | --- |
| changed | CST-001 | Constraint | left: 90 ms → 145 ms; satisfied: true → false |
| changed | SF-FuseData | SystemFunction | latency: 25 ms → 80 ms |
