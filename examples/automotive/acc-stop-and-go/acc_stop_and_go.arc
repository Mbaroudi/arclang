// Adaptive Cruise Control with Stop & Go: the full engineering model.
//
// A full-speed-range ACC (ISO 15622:2018, FSRA) for a passenger car, from the
// stakeholders down to the ECUs and their bill of materials, following CESAM:
// the operational vision (why), the functional vision (what) and the
// constructional vision (how), each through its six aspects. Every
// requirement is derived from the one above it, allocated to the elements
// that satisfy it and verified: the control law by simulation in OpenModelica
// (physics/ACCPhysics.mo), the mode logic by executing the generated C,
// the safety case by a fault tree computed from the FMEA.
//
//   arclang check  examples/automotive/acc-stop-and-go/acc_stop_and_go.arc --lint
//   arclang table  examples/automotive/acc-stop-and-go/acc_stop_and_go.arc --kind cesam_grid -f md
//   arclang verify examples/automotive/acc-stop-and-go/acc_stop_and_go.arc
//   arclang cosim  examples/automotive/acc-stop-and-go/acc_stop_and_go.arc
//   arclang gate   examples/automotive/acc-stop-and-go/acc_stop_and_go.arc

model AccStopAndGo {
    name: "Adaptive Cruise Control with Stop & Go"
    description: "Holds the set speed, follows the vehicle ahead at the chosen time gap down to standstill and drives off again, and hands control back to the driver whenever the situation exceeds its authority"
    version: "3.0"
    method: "cesam"
}

// ===========================================================================
// OPERATIONAL VISION — why the system exists, for whom, in which world
// ===========================================================================

// ---- Stakeholders, needs and stakeholder requirements ----------------------

requirements stakeholder {
    stakeholder "Driver" { id: "STK-DRV" category: "user" influence: Critical interest: "Less fatigue on motorways and in congestion, without losing command of the car" }
    stakeholder "Passengers" { id: "STK-PAX" category: "user" influence: Medium interest: "A smooth ride: no jolts, no surprising braking" }
    stakeholder "Surrounding road users" { id: "STK-TRF" category: "third party" influence: High interest: "A car that behaves predictably and never closes in dangerously" }
    stakeholder "Vehicle manufacturer" { id: "STK-OEM" category: "customer" influence: Critical interest: "A competitive driver assistance feature, approved in every target market, at a controlled cost" }
    stakeholder "Type-approval authority" { id: "STK-AUTH" category: "authority" influence: Critical interest: "Evidence that the system meets ISO 15622 and the functional safety state of the art" }
    stakeholder "Service workshop" { id: "STK-SVC" category: "maintainer" influence: Medium interest: "Faults that are diagnosed quickly and sensors that can be recalibrated after a repair" }
    stakeholder "Functional safety manager" { id: "STK-FSM" category: "internal authority" influence: High interest: "A safety case: hazards analysed, safety goals met, residual risk argued" }

    concern "Less fatigue on long trips and in congestion" { id: "NEED-001" stakeholders: ["STK-DRV", "STK-PAX"] rationale: "Speed keeping and stop-and-go driving are the most tiring parts of driving" }
    concern "A safe distance at all times" { id: "NEED-002" stakeholders: ["STK-DRV", "STK-PAX", "STK-TRF", "STK-AUTH"] rationale: "Rear-end collisions are the most frequent motorway accidents" }
    concern "A smooth and predictable ride" { id: "NEED-003" stakeholders: ["STK-PAX", "STK-DRV", "STK-TRF"] rationale: "Harsh or surprising manoeuvres make passengers sick and following drivers react late" }
    concern "The driver stays in command" { id: "NEED-004" stakeholders: ["STK-DRV", "STK-AUTH", "STK-FSM"] rationale: "ACC is a level 1 assistance: the driver remains responsible and must be able to take over at any time" }
    concern "Approval in every target market" { id: "NEED-005" stakeholders: ["STK-OEM", "STK-AUTH"] }
    concern "Safe behaviour when something fails" { id: "NEED-006" stakeholders: ["STK-FSM", "STK-AUTH", "STK-DRV"] }
    concern "Quick diagnosis and repair" { id: "NEED-007" stakeholders: ["STK-SVC", "STK-OEM"] }
    concern "No control through forged messages" { id: "NEED-008" stakeholders: ["STK-OEM", "STK-AUTH"] rationale: "UNECE R155 makes cybersecurity part of the type approval" }

    req "STK-REQ-001" "Hold the chosen speed" {
        description: "The car shall hold a speed chosen by the driver between 30 km/h and 180 km/h without the driver touching the pedals"
        priority: High  frames: ["NEED-001"]  source: "Driver clinics, 2026 programme brief"  verification_method: "test"
    }
    req "STK-REQ-002" "Follow at a chosen time gap" {
        description: "The car shall follow the vehicle ahead at a time gap the driver chooses among four settings from 1.0 s to 2.5 s"
        priority: Critical  frames: ["NEED-001", "NEED-002"]  source: "ISO 15622:2018 §6.2"  verification_method: "test"
    }
    req "STK-REQ-003" "Stop and go in congestion" {
        description: "The car shall follow the vehicle ahead down to standstill, hold the car stopped and drive off again on the driver's confirmation"
        priority: High  frames: ["NEED-001"]  source: "ISO 15622:2018, full speed range ACC (FSRA)"  verification_method: "test"
    }
    req "STK-REQ-004" "Comfortable manoeuvres" {
        description: "The car shall not accelerate, brake or jerk harder than an attentive driver would in the same traffic"
        priority: High  frames: ["NEED-003"]  source: "ISO 15622:2018 §6.3"  verification_method: "test"
    }
    req "STK-REQ-005" "Hand control back in time" {
        description: "When the traffic needs more braking than the system may apply, the driver shall be warned at once, visually and audibly"
        priority: Critical  frames: ["NEED-004", "NEED-006"]  source: "ISO 15622:2018 §6.4"  verification_method: "test"
    }
    req "STK-REQ-006" "Driver overrides at any time" {
        description: "Any action of the driver on the brake pedal shall end the system's control at once; the accelerator shall prevail while pressed"
        priority: Critical  frames: ["NEED-004"]  source: "ISO 15622:2018 §6.1"  verification_method: "test"
    }
    req "STK-REQ-007" "Fail safe and say so" {
        description: "On any fault that can affect its control, the system shall switch off without a jolt and tell the driver"
        priority: Critical  frames: ["NEED-006"]  source: "Item definition, safety goal SG-1 to SG-3"  verification_method: "test"
    }
    req "STK-REQ-008" "Approval evidence" {
        description: "The system shall be developed and verified to ISO 15622:2018 and to ISO 26262 up to ASIL B"
        priority: Critical  frames: ["NEED-005"]  verification_method: "inspection"
    }
    req "STK-REQ-009" "Workshop diagnosis" {
        description: "A workshop shall read the system's faults with a standard UDS tester and recalibrate the sensors in less than 30 minutes"
        priority: Medium  frames: ["NEED-007"]  verification_method: "demonstration"
    }
    req "STK-REQ-010" "Tamper resistance" {
        description: "No message forged on a vehicle bus shall make the system accelerate or brake"
        priority: High  frames: ["NEED-008"]  source: "UNECE R155, ISO/SAE 21434"  verification_method: "test"
    }
    req "STK-REQ-011" "Cope with cut-ins" {
        description: "The car shall slow down smoothly for a vehicle that cuts in ahead and resume following it"
        priority: High  frames: ["NEED-002", "NEED-003"]  verification_method: "test"
    }
}

// ---- The operational world --------------------------------------------------

operational_analysis "Driving with traffic" {
    actor "Driver" {
        id: "OA-DRV"
        description: "Chooses speed and time gap, monitors the traffic, takes over when asked or at will"
    }
    actor "Vehicle ahead" {
        id: "OA-LEAD"
        description: "The vehicle in the ego lane that the car follows; it may brake, stop or drive off"
    }
    actor "Cutting-in vehicle" {
        id: "OA-CUT"
        description: "A vehicle changing into the ego lane in front of the car"
    }
    actor "Following vehicle" {
        id: "OA-FOLLOW"
        description: "The vehicle behind, whose driver relies on the car not braking without reason"
    }
    actor "Workshop technician" {
        id: "OA-TECH"
    }
    entity "Ego vehicle" {
        id: "OA-EGO"
        description: "The car equipped with the system"
        activity "Perceive the traffic ahead" { id: "OAC-PERCEIVE" }
        activity "Keep speed and distance" { id: "OAC-REGULATE" }
        activity "Inform and warn the driver" { id: "OAC-INFORM" }
    }
    entity "Road" {
        id: "OA-ROAD"
        description: "Motorways, main roads and urban congestion; lane markings, curves, slopes up to 12 %"
        activity "Carry the traffic" { id: "OAC-ROAD" }
    }
    entity "Weather and light" {
        id: "OA-WX"
        description: "Rain, fog, snow, low sun and night: the conditions the sensors must cope with or report"
        activity "Degrade visibility" { id: "OAC-WX" }
    }

    operational_activity "Choose speed and time gap" { id: "OAC-CHOOSE" description: "Set, resume or change the speed; pick one of the four time gaps" performed_by: "OA-DRV" }
    operational_activity "Supervise the drive" { id: "OAC-SUPERVISE" description: "Keep eyes on the road while the car regulates" performed_by: "OA-DRV" }
    operational_activity "Take over control" { id: "OAC-TAKEOVER" description: "Brake, accelerate or steer; end the assistance" performed_by: "OA-DRV" }
    operational_activity "Drive ahead" { id: "OAC-LEAD" performed_by: "OA-LEAD" }
    operational_activity "Change into the ego lane" { id: "OAC-CUT" performed_by: "OA-CUT" }
    operational_activity "Follow the ego vehicle" { id: "OAC-FOLLOW" performed_by: "OA-FOLLOW" }
    operational_activity "Diagnose and recalibrate" { id: "OAC-SERVICE" description: "Read the fault memory, replace a part, align the radar and the camera" performed_by: "OA-TECH" }

    operational_capability "Drive at the chosen speed and distance" {
        id: "OC-001"
        description: "Cover a motorway trip with the speed and the distance held for the driver"
        involves: ["OA-DRV", "OA-EGO", "OA-LEAD", "OA-ROAD"]
    }
    operational_capability "Cope with lane changes of others" {
        id: "OC-002"
        description: "Slow down for a vehicle cutting in, speed up when the vehicle ahead leaves the lane"
        involves: ["OA-EGO", "OA-CUT", "OA-LEAD", "OA-FOLLOW"]
    }
    operational_capability "Crawl through congestion" {
        id: "OC-003"
        description: "Stop and go behind the vehicle ahead in a traffic jam"
        involves: ["OA-DRV", "OA-EGO", "OA-LEAD"]
        extends: ["OC-001"]
    }
    operational_capability "Give control back to the driver" {
        id: "OC-004"
        description: "End the assistance by the driver's will, or ask the driver to take over when the traffic or a fault requires it"
        involves: ["OA-DRV", "OA-EGO"]
    }
    operational_capability "Keep the system serviceable" {
        id: "OC-005"
        involves: ["OA-TECH", "OA-EGO"]
    }

    interaction "Speed and time gap wishes" { id: "OE-001" from: "OAC-CHOOSE" to: "OAC-REGULATE" data_type: "Driver settings" }
    interaction "Motion of the vehicle ahead" { id: "OE-002" from: "OAC-LEAD" to: "OAC-PERCEIVE" data_type: "Relative distance and speed" }
    interaction "Lane change ahead" { id: "OE-003" from: "OAC-CUT" to: "OAC-PERCEIVE" data_type: "Object entering the lane" }
    interaction "Lane geometry" { id: "OE-004" from: "OAC-ROAD" to: "OAC-PERCEIVE" data_type: "Lanes and curvature" }
    interaction "Visibility" { id: "OE-005" from: "OAC-WX" to: "OAC-PERCEIVE" data_type: "Sensor conditions" }
    interaction "Traffic picture" { id: "OE-006" from: "OAC-PERCEIVE" to: "OAC-REGULATE" data_type: "Object to follow" }
    interaction "Ego motion seen from behind" { id: "OE-007" from: "OAC-REGULATE" to: "OAC-FOLLOW" data_type: "Brake lights and deceleration" }
    interaction "State and warnings" { id: "OE-008" from: "OAC-INFORM" to: "OAC-SUPERVISE" data_type: "ACC state, takeover request" }
    interaction "Takeover need" { id: "OE-009" from: "OAC-REGULATE" to: "OAC-INFORM" data_type: "Deceleration needed" }
    interaction "Driver action" { id: "OE-010" from: "OAC-TAKEOVER" to: "OAC-REGULATE" data_type: "Pedals and switches" }
    interaction "Fault memory and calibration" { id: "OE-011" from: "OAC-SERVICE" to: "OAC-PERCEIVE" data_type: "Diagnostic session" }

    operational_process "Respond to a cut-in" {
        id: "OP-CUTIN"
        involves: ["OAC-CUT", "OAC-PERCEIVE", "OAC-REGULATE", "OAC-FOLLOW"]
    }
    operational_process "Hand over to the driver" {
        id: "OP-HANDOVER"
        involves: ["OAC-REGULATE", "OAC-INFORM", "OAC-SUPERVISE", "OAC-TAKEOVER"]
    }
}

// ---- Operational interfaces: what flows in the operational world -----------

exchange_item "Relative distance and speed" { id: "EI-OPS-REL" mechanism: "FLOW" vision: "operational" description: "How far the vehicle ahead is and how fast it closes in, as a driver judges it" }
exchange_item "Object entering the lane" { id: "EI-OPS-CUT" mechanism: "EVENT" vision: "operational" description: "Indicator, lateral drift, a wheel on the line" }
exchange_item "Lanes and curvature" { id: "EI-OPS-LANES" mechanism: "FLOW" vision: "operational" }
exchange_item "Sensor conditions" { id: "EI-OPS-WX" mechanism: "FLOW" vision: "operational" description: "Rain rate, fog density, sun glare, dirt" }
exchange_item "Object to follow" { id: "EI-OPS-TARGET" mechanism: "FLOW" vision: "operational" }
exchange_item "Brake lights and deceleration" { id: "EI-OPS-BRAKE" mechanism: "FLOW" vision: "operational" description: "What the following driver sees of the car's braking" }
exchange_item "ACC state, takeover request" { id: "EI-OPS-HMI" mechanism: "EVENT" vision: "operational" }
exchange_item "Deceleration needed" { id: "EI-OPS-NEED" mechanism: "FLOW" vision: "operational" }
exchange_item "Pedals and switches" { id: "EI-OPS-DRIVER" mechanism: "EVENT" vision: "operational" }
exchange_item "Diagnostic session" { id: "EI-OPS-DIAG" mechanism: "OPERATION" vision: "operational" description: "Fault memory read-out, part replacement, alignment" }

// ---- Lifecycle: the phases the system goes through ------------------------

state_machine "Vehicle lifecycle" {
    kind: "lifecycle"
    initial: "Development"
    state Development { description: "Specification, design and verification" context: ["STK-OEM", "STK-FSM"] }
    state Homologation { description: "Type approval tests and the safety assessment" context: ["STK-AUTH", "STK-OEM", "STK-FSM"] }
    state EndOfLine { description: "Assembly, end-of-line radar and camera alignment, parameter coding" context: ["STK-OEM", "OA-EGO"] }
    state InService { description: "Driven by its owner on public roads" context: ["OA-DRV", "OA-LEAD", "OA-CUT", "OA-FOLLOW", "OA-ROAD", "OA-WX"] }
    state Workshop { description: "Diagnosis, repair and recalibration" context: ["OA-TECH", "STK-SVC"] }
    state Decommissioned { description: "Scrapped; the radar and the ECUs are recycled" context: ["STK-OEM", "STK-SVC"] }
    transition Development -> Homologation { guard: "design verified" }
    transition Homologation -> EndOfLine { guard: "type approved" }
    transition EndOfLine -> InService { guard: "end-of-line check passed" }
    transition InService -> Workshop { guard: "fault or scheduled service" }
    transition Workshop -> InService { guard: "repaired and recalibrated" }
    transition InService -> Decommissioned { guard: "end of vehicle life" }
}

// ---- Operational scenario ---------------------------------------------------

scenario "A truck cuts in on the motorway" {
    id: "SC-OPS-CUTIN"
    capability: "OC-002"
    participants: ["OA-DRV", "OA-EGO", "OA-LEAD", "OA-CUT", "OA-FOLLOW"]
    message "OA-DRV" -> "OA-EGO" "set 130 km/h, time gap 1.8 s"
    message "OA-LEAD" -> "OA-EGO" "drives at 100 km/h, 55 m ahead"
    message "OA-CUT" -> "OA-EGO" "changes lane 22 m ahead at 80 km/h"
    fragment alt "deceleration needed" {
        operand "within 3.5 m/s2" {
            message "OA-EGO" -> "OA-FOLLOW" "slows down smoothly, brake lights on"
            message "OA-EGO" -> "OA-DRV" "new vehicle followed"
        }
        operand "beyond 3.5 m/s2" {
            message "OA-EGO" -> "OA-DRV" "takeover request, chime"
            message "OA-DRV" -> "OA-EGO" "brakes"
        }
    }
}

// ===========================================================================
// FUNCTIONAL VISION — what the system does, in which modes, on which data
// ===========================================================================

// ---- The system and what surrounds it ---------------------------------------

system_analysis "ACC system" {
    actor "Driver" { id: "SA-DRV" description: "Uses the steering wheel switches and the pedals; sees the cluster" }
    actor "Vehicle ahead" { id: "SA-LEAD" description: "Any object in front: vehicles, motorcycles, trucks; stationary ones are not followed from speed" }
    actor "Road scene" { id: "SA-ROAD" description: "Lane markings, road edges and curvature as the camera sees them" }
    actor "Brake system (ESC)" { id: "SA-ESC" asil: "ASIL-D" description: "Applies the deceleration requested, measures wheel speeds and yaw rate, holds the car at standstill" }
    actor "Powertrain controller" { id: "SA-PT" asil: "ASIL-B" description: "Delivers the drive torque requested; reports the accelerator pedal" }
    actor "Electric park brake" { id: "SA-EPB" asil: "ASIL-B" description: "Secures the car after a long standstill" }
    actor "Instrument cluster" { id: "SA-CLU" asil: "ASIL-A" description: "Shows the ACC state, the set speed, the time gap and the takeover request; sounds the chime" }
    actor "Body controller" { id: "SA-BCM" asil: "QM" description: "Door, bonnet and seat-belt states" }
    actor "Diagnostic tester" { id: "SA-TESTER" description: "The workshop's UDS tester, on the diagnostic connector" }

    system_component "ACC" {
        id: "SC-ACC"
        description: "The system under design: every system function is allocated to it"
        allocated_functions: ["SF-RADAR", "SF-CAMERA", "SF-EGO", "SF-FUSE", "SF-PATH", "SF-SELECT", "SF-CUTDET", "SF-SET", "SF-OVR", "SF-MODE", "SF-GAP", "SF-SPEED", "SF-DIST", "SF-ARB", "SF-LIMIT", "SF-BRAKE", "SF-TORQUE", "SF-HOLD", "SF-HMI", "SF-TOR", "SF-PLAUS", "SF-DIAG"]
    }

    mission "Relieve the driver of longitudinal control" { id: "MIS-ASSIST" }
    mission "Keep the car safe and serviceable" { id: "MIS-SAFE" }

    capability "Speed control" {
        id: "CAP-SPEED"  mission: "MIS-ASSIST"  realizes: "OC-001"
        description: "Hold the set speed when no vehicle is followed"
        involves: ["SA-DRV", "SF-SET", "SF-EGO", "SF-SPEED", "SF-ARB", "SF-LIMIT", "SF-TORQUE", "SF-BRAKE"]
    }
    capability "Distance control" {
        id: "CAP-FOLLOW"  mission: "MIS-ASSIST"  realizes: "OC-001"
        description: "Follow the vehicle ahead at the chosen time gap"
        involves: ["SA-LEAD", "SF-RADAR", "SF-CAMERA", "SF-FUSE", "SF-PATH", "SF-SELECT", "SF-GAP", "SF-DIST", "SF-ARB", "SF-LIMIT", "SF-BRAKE", "SF-TORQUE"]
        includes: ["CAP-SPEED"]
    }
    capability "Cut-in and cut-out handling" {
        id: "CAP-CUTIN"  mission: "MIS-ASSIST"  realizes: "OC-002"
        involves: ["SA-LEAD", "SF-CUTDET", "SF-SELECT", "SF-DIST"]
        extends: ["CAP-FOLLOW"]
    }
    capability "Stop and go" {
        id: "CAP-STOPGO"  mission: "MIS-ASSIST"  realizes: "OC-003"
        description: "Follow into standstill, hold, drive off on confirmation"
        involves: ["SA-EPB", "SA-ESC", "SF-HOLD", "SF-MODE", "SF-DIST"]
        extends: ["CAP-FOLLOW"]
    }
    capability "Override and takeover" {
        id: "CAP-HANDOVER"  mission: "MIS-SAFE"  realizes: "OC-004"
        involves: ["SA-DRV", "SA-CLU", "SF-OVR", "SF-MODE", "SF-TOR", "SF-HMI"]
    }
    capability "Safe degradation" {
        id: "CAP-DEGRADE"  mission: "MIS-SAFE"  realizes: "OC-004"
        involves: ["SF-PLAUS", "SF-MODE", "SF-TOR", "SF-LIMIT"]
    }
    capability "Diagnosis and calibration" {
        id: "CAP-DIAG"  mission: "MIS-SAFE"  realizes: "OC-005"
        involves: ["SA-TESTER", "SF-DIAG", "SF-PLAUS"]
    }

    // Perception
    function "Measure objects by radar" {
        id: "SF-RADAR"  asil: "ASIL-B"  latency: 50 ms  period: 50 ms
        description: "Range, range rate and angle of the objects ahead, up to 200 m"
        port in "echo" { data_type: "Radar echo" }
        port out "radar_objects" { data_type: "Object list" }
    }
    function "Detect objects and lanes by camera" {
        id: "SF-CAMERA"  asil: "ASIL-B"  latency: 33 ms  period: 33 ms
        description: "Classify the objects ahead and find the lane markings"
        port in "image" { data_type: "Camera image" }
        port in "lane_markings" { data_type: "Camera image" }
        port out "camera_objects" { data_type: "Object list" }
        port out "lanes" { data_type: "Lane path" }
    }
    function "Estimate ego motion" {
        id: "SF-EGO"  asil: "ASIL-B"  latency: 5 ms  period: 10 ms
        description: "Speed, acceleration and yaw rate of the car from the wheel speeds and the inertial sensor"
        port in "wheel_speeds" { data_type: "Vehicle dynamics" }
        port out "ego_motion" { data_type: "Ego motion" }
    }
    function "Fuse objects into tracks" {
        id: "SF-FUSE"  asil: "ASIL-B"  latency: 15 ms  period: 50 ms
        description: "Associate radar and camera detections, track them, estimate their acceleration"
        port in "radar_objects" { data_type: "Object list" }
        port in "camera_objects" { data_type: "Object list" }
        port in "ego_motion" { data_type: "Ego motion" }
        port out "tracks" { data_type: "Tracks" }
    }
    function "Predict the ego path" {
        id: "SF-PATH"  asil: "ASIL-B"  latency: 5 ms
        description: "Course of the car over the next 4 s from the lanes and the yaw rate"
        port in "lanes" { data_type: "Lane path" }
        port in "ego_motion" { data_type: "Ego motion" }
        port out "path" { data_type: "Lane path" }
    }
    function "Select the target to follow" {
        id: "SF-SELECT"  asil: "ASIL-B"  latency: 5 ms
        description: "The nearest moving track inside the predicted path"
        port in "tracks" { data_type: "Tracks" }
        port in "path" { data_type: "Lane path" }
        port out "target" { data_type: "Target" }
    }
    function "Detect cut-ins and cut-outs" {
        id: "SF-CUTDET"  asil: "ASIL-B"  latency: 100 ms
        description: "A track crossing into or out of the path, confirmed over three cycles"
        port in "tracks" { data_type: "Tracks" }
        port in "path" { data_type: "Lane path" }
        port out "lane_change" { data_type: "Lane change" }
    }
    // Control
    function "Manage driver settings" {
        id: "SF-SET"  asil: "QM"  latency: 10 ms
        description: "Main switch, set, resume, cancel, speed steps, time gap"
        port in "switches" { data_type: "Driver settings" }
        port out "settings" { data_type: "Driver settings" }
    }
    function "Detect driver override" {
        id: "SF-OVR"  asil: "ASIL-B"  latency: 10 ms
        description: "Brake pedal, accelerator beyond the ACC demand, cancel"
        port in "pedals" { data_type: "Pedal state" }
        port out "override" { data_type: "Override" }
    }
    function "Manage the ACC mode" {
        id: "SF-MODE"  asil: "ASIL-B"  latency: 5 ms  period: 10 ms
        description: "Off, standby, cruising, following, standstill, overridden, degraded"
        port in "settings" { data_type: "Driver settings" }
        port in "override" { data_type: "Override" }
        port in "health" { data_type: "Health" }
        port in "target" { data_type: "Target" }
        port in "vehicle_state" { data_type: "Body state" }
        port out "mode" { data_type: "ACC mode" }
    }
    function "Compute the distance to keep" {
        id: "SF-GAP"  asil: "ASIL-B"  latency: 2 ms
        description: "d = d0 + time gap x speed, d0 = 4 m"
        port in "settings" { data_type: "Driver settings" }
        port in "ego_motion" { data_type: "Ego motion" }
        port out "gap_reference" { data_type: "Distance to keep" }
    }
    function "Control speed" {
        id: "SF-SPEED"  asil: "ASIL-B"  latency: 2 ms  period: 20 ms
        port in "settings" { data_type: "Driver settings" }
        port in "ego_motion" { data_type: "Ego motion" }
        port out "speed_acceleration" { data_type: "Acceleration demand" }
    }
    function "Control distance" {
        id: "SF-DIST"  asil: "ASIL-B"  latency: 5 ms  period: 20 ms
        description: "Acceleration from the distance error and the relative speed; the deceleration needed to stop behind the target"
        port in "target" { data_type: "Target" }
        port in "gap_reference" { data_type: "Distance to keep" }
        port in "ego_motion" { data_type: "Ego motion" }
        port in "lane_change" { data_type: "Lane change" }
        port out "distance_acceleration" { data_type: "Acceleration demand" }
        port out "deceleration_needed" { data_type: "Acceleration demand" }
    }
    function "Arbitrate the acceleration" {
        id: "SF-ARB"  asil: "ASIL-B"  latency: 2 ms
        description: "The lower of speed and distance control, according to the mode"
        port in "speed_acceleration" { data_type: "Acceleration demand" }
        port in "distance_acceleration" { data_type: "Acceleration demand" }
        port in "mode" { data_type: "ACC mode" }
        port out "acceleration_request" { data_type: "Acceleration demand" }
    }
    function "Bound acceleration and jerk" {
        id: "SF-LIMIT"  asil: "ASIL-B"  latency: 2 ms
        description: "Clamp to -3.5 .. +2.0 m/s2 and 2.5 m/s3, the safety envelope"
        port in "acceleration_request" { data_type: "Acceleration demand" }
        port out "acceleration_command" { data_type: "Acceleration demand" }
    }
    // Actuation
    function "Request deceleration" {
        id: "SF-BRAKE"  asil: "ASIL-B"  latency: 10 ms
        port in "acceleration_command" { data_type: "Acceleration demand" }
        port out "brake_request" { data_type: "Brake request" }
    }
    function "Request drive torque" {
        id: "SF-TORQUE"  asil: "ASIL-B"  latency: 10 ms
        port in "acceleration_command" { data_type: "Acceleration demand" }
        port out "torque_request" { data_type: "Torque request" }
    }
    function "Hold at standstill" {
        id: "SF-HOLD"  asil: "ASIL-B"  latency: 10 ms
        description: "Service brake hold, handed over to the park brake after 180 s"
        port in "mode" { data_type: "ACC mode" }
        port out "hold_request" { data_type: "Brake request" }
        port out "park_request" { data_type: "Park request" }
    }
    // Driver information and safety
    function "Show the ACC state" {
        id: "SF-HMI"  asil: "QM"  latency: 20 ms
        port in "mode" { data_type: "ACC mode" }
        port in "settings" { data_type: "Driver settings" }
        port out "display" { data_type: "ACC display" }
    }
    function "Request takeover" {
        id: "SF-TOR"  asil: "ASIL-B"  latency: 20 ms
        description: "Red symbol and chime when the deceleration needed exceeds 3.5 m/s2 or on a fault"
        port in "deceleration_needed" { data_type: "Acceleration demand" }
        port in "health" { data_type: "Health" }
        port out "takeover" { data_type: "Takeover request" }
    }
    function "Monitor sensors and plausibility" {
        id: "SF-PLAUS"  asil: "ASIL-B"  latency: 50 ms  period: 50 ms
        description: "Blindness, misalignment, radar/camera disagreement, controller self-tests"
        port in "tracks" { data_type: "Tracks" }
        port out "health" { data_type: "Health" }
        port out "fault_report" { data_type: "Fault report" }
    }
    function "Record and report faults" {
        id: "SF-DIAG"  asil: "QM"
        description: "Diagnostic trouble codes with freeze frames, UDS services, calibration routines"
        port in "fault_report" { data_type: "Fault report" }
        port in "tester_request" { data_type: "UDS request" }
        port out "tester_response" { data_type: "UDS response" }
    }

    // Exchanges with the surroundings
    functional_exchange "radar echo" { id: "FE-ECHO" from: "SA-LEAD" to: "SF-RADAR.echo" data_type: "Radar echo" }
    functional_exchange "image of the scene" { id: "FE-IMAGE" from: "SA-LEAD" to: "SF-CAMERA.image" data_type: "Camera image" }
    functional_exchange "lane markings" { id: "FE-MARKINGS" from: "SA-ROAD" to: "SF-CAMERA.lane_markings" data_type: "Camera image" }
    functional_exchange "wheel speeds and yaw rate" { id: "FE-WHEELS" from: "SA-ESC" to: "SF-EGO.wheel_speeds" data_type: "Vehicle dynamics" }
    functional_exchange "switches" { id: "FE-SWITCHES" from: "SA-DRV" to: "SF-SET.switches" data_type: "Driver settings" }
    functional_exchange "pedals" { id: "FE-PEDALS" from: "SA-PT" to: "SF-OVR.pedals" data_type: "Pedal state" }
    functional_exchange "doors and belts" { id: "FE-BODY" from: "SA-BCM" to: "SF-MODE.vehicle_state" data_type: "Body state" }
    functional_exchange "brake request" { id: "FE-BRAKEREQ" from: "SF-BRAKE.brake_request" to: "SA-ESC" data_type: "Brake request" }
    functional_exchange "torque request" { id: "FE-TORQUEREQ" from: "SF-TORQUE.torque_request" to: "SA-PT" data_type: "Torque request" }
    functional_exchange "standstill hold" { id: "FE-HOLD" from: "SF-HOLD.hold_request" to: "SA-ESC" data_type: "Brake request" }
    functional_exchange "park request" { id: "FE-PARK" from: "SF-HOLD.park_request" to: "SA-EPB" data_type: "Park request" }
    functional_exchange "ACC display" { id: "FE-DISPLAY" from: "SF-HMI.display" to: "SA-CLU" data_type: "ACC display" }
    functional_exchange "takeover request" { id: "FE-TAKEOVER" from: "SF-TOR.takeover" to: "SA-CLU" data_type: "Takeover request" }
    functional_exchange "tester request" { id: "FE-TESTREQ" from: "SA-TESTER" to: "SF-DIAG.tester_request" data_type: "UDS request" }
    functional_exchange "tester response" { id: "FE-TESTRSP" from: "SF-DIAG.tester_response" to: "SA-TESTER" data_type: "UDS response" }

    // Internal exchanges
    functional_exchange "radar objects" { id: "FE-RADOBJ" from: "SF-RADAR.radar_objects" to: "SF-FUSE.radar_objects" data_type: "Object list" }
    functional_exchange "camera objects" { id: "FE-CAMOBJ" from: "SF-CAMERA.camera_objects" to: "SF-FUSE.camera_objects" data_type: "Object list" }
    functional_exchange "lanes" { id: "FE-LANES" from: "SF-CAMERA.lanes" to: "SF-PATH.lanes" data_type: "Lane path" }
    functional_exchange "ego motion to fusion" { id: "FE-EGO-FUSE" from: "SF-EGO.ego_motion" to: "SF-FUSE.ego_motion" data_type: "Ego motion" }
    functional_exchange "ego motion to path" { id: "FE-EGO-PATH" from: "SF-EGO.ego_motion" to: "SF-PATH.ego_motion" data_type: "Ego motion" }
    functional_exchange "ego motion to gap" { id: "FE-EGO-GAP" from: "SF-EGO.ego_motion" to: "SF-GAP.ego_motion" data_type: "Ego motion" }
    functional_exchange "ego motion to speed control" { id: "FE-EGO-SPD" from: "SF-EGO.ego_motion" to: "SF-SPEED.ego_motion" data_type: "Ego motion" }
    functional_exchange "ego motion to distance control" { id: "FE-EGO-DST" from: "SF-EGO.ego_motion" to: "SF-DIST.ego_motion" data_type: "Ego motion" }
    functional_exchange "tracks to selection" { id: "FE-TRK-SEL" from: "SF-FUSE.tracks" to: "SF-SELECT.tracks" data_type: "Tracks" }
    functional_exchange "tracks to cut-in detection" { id: "FE-TRK-CUT" from: "SF-FUSE.tracks" to: "SF-CUTDET.tracks" data_type: "Tracks" }
    functional_exchange "tracks to monitoring" { id: "FE-TRK-MON" from: "SF-FUSE.tracks" to: "SF-PLAUS.tracks" data_type: "Tracks" }
    functional_exchange "path to selection" { id: "FE-PATH-SEL" from: "SF-PATH.path" to: "SF-SELECT.path" data_type: "Lane path" }
    functional_exchange "path to cut-in detection" { id: "FE-PATH-CUT" from: "SF-PATH.path" to: "SF-CUTDET.path" data_type: "Lane path" }
    functional_exchange "target to distance control" { id: "FE-TGT-DST" from: "SF-SELECT.target" to: "SF-DIST.target" data_type: "Target" }
    functional_exchange "target to mode" { id: "FE-TGT-MODE" from: "SF-SELECT.target" to: "SF-MODE.target" data_type: "Target" }
    functional_exchange "lane change" { id: "FE-LANECHG" from: "SF-CUTDET.lane_change" to: "SF-DIST.lane_change" data_type: "Lane change" }
    functional_exchange "settings to mode" { id: "FE-SET-MODE" from: "SF-SET.settings" to: "SF-MODE.settings" data_type: "Driver settings" }
    functional_exchange "settings to gap" { id: "FE-SET-GAP" from: "SF-SET.settings" to: "SF-GAP.settings" data_type: "Driver settings" }
    functional_exchange "settings to speed control" { id: "FE-SET-SPD" from: "SF-SET.settings" to: "SF-SPEED.settings" data_type: "Driver settings" }
    functional_exchange "settings to display" { id: "FE-SET-HMI" from: "SF-SET.settings" to: "SF-HMI.settings" data_type: "Driver settings" }
    functional_exchange "override" { id: "FE-OVR" from: "SF-OVR.override" to: "SF-MODE.override" data_type: "Override" }
    functional_exchange "health to mode" { id: "FE-HLT-MODE" from: "SF-PLAUS.health" to: "SF-MODE.health" data_type: "Health" }
    functional_exchange "health to takeover" { id: "FE-HLT-TOR" from: "SF-PLAUS.health" to: "SF-TOR.health" data_type: "Health" }
    functional_exchange "fault report" { id: "FE-FAULTS" from: "SF-PLAUS.fault_report" to: "SF-DIAG.fault_report" data_type: "Fault report" }
    functional_exchange "mode to arbitration" { id: "FE-MODE-ARB" from: "SF-MODE.mode" to: "SF-ARB.mode" data_type: "ACC mode" }
    functional_exchange "mode to hold" { id: "FE-MODE-HOLD" from: "SF-MODE.mode" to: "SF-HOLD.mode" data_type: "ACC mode" }
    functional_exchange "mode to display" { id: "FE-MODE-HMI" from: "SF-MODE.mode" to: "SF-HMI.mode" data_type: "ACC mode" }
    functional_exchange "distance to keep" { id: "FE-GAPREF" from: "SF-GAP.gap_reference" to: "SF-DIST.gap_reference" data_type: "Distance to keep" }
    functional_exchange "speed control acceleration" { id: "FE-ASPD" from: "SF-SPEED.speed_acceleration" to: "SF-ARB.speed_acceleration" data_type: "Acceleration demand" }
    functional_exchange "distance control acceleration" { id: "FE-ADST" from: "SF-DIST.distance_acceleration" to: "SF-ARB.distance_acceleration" data_type: "Acceleration demand" }
    functional_exchange "deceleration needed" { id: "FE-NEED" from: "SF-DIST.deceleration_needed" to: "SF-TOR.deceleration_needed" data_type: "Acceleration demand" }
    functional_exchange "acceleration request" { id: "FE-AREQ" from: "SF-ARB.acceleration_request" to: "SF-LIMIT.acceleration_request" data_type: "Acceleration demand" }
    functional_exchange "acceleration command to brakes" { id: "FE-ACMD-BRK" from: "SF-LIMIT.acceleration_command" to: "SF-BRAKE.acceleration_command" data_type: "Acceleration demand" }
    functional_exchange "acceleration command to powertrain" { id: "FE-ACMD-PT" from: "SF-LIMIT.acceleration_command" to: "SF-TORQUE.acceleration_command" data_type: "Acceleration demand" }

    // End-to-end paths, with their latency budgets (checked by the gate)
    functional_chain "Following control loop" {
        id: "FC-FOLLOW"  capability: "CAP-FOLLOW"
        description: "From the radar measurement to the brake request: one control cycle behind the vehicle ahead"
        involves: ["SF-RADAR", "SF-FUSE", "SF-SELECT", "SF-DIST", "SF-ARB", "SF-LIMIT", "SF-BRAKE"]
        latency_budget: 150 ms
    }
    functional_chain "Cut-in reaction" {
        id: "FC-CUTIN"  capability: "CAP-CUTIN"
        description: "From the first camera sight of the vehicle crossing the line to the first deceleration request"
        involves: ["SF-CAMERA", "SF-FUSE", "SF-CUTDET", "SF-DIST", "SF-ARB", "SF-LIMIT", "SF-BRAKE"]
        latency_budget: 300 ms
    }
    functional_chain "Brake pedal override" {
        id: "FC-OVERRIDE"  capability: "CAP-HANDOVER"
        description: "From the brake pedal switch to the released requests"
        involves: ["SF-OVR", "SF-MODE", "SF-ARB", "SF-LIMIT", "SF-TORQUE"]
        latency_budget: 100 ms
    }
    functional_chain "Takeover request" {
        id: "FC-TAKEOVER"  capability: "CAP-HANDOVER"
        description: "From the deceleration needed exceeding 3.5 m/s2 to the red symbol and the chime"
        involves: ["SF-DIST", "SF-TOR"]
        latency_budget: 100 ms
    }
    functional_chain "Fault to safe state" {
        id: "FC-FAULT"  capability: "CAP-DEGRADE"
        description: "From the detection of a sensor fault to the takeover request, the requests ramped out"
        involves: ["SF-PLAUS", "SF-MODE", "SF-ARB", "SF-LIMIT", "SF-TOR"]
        latency_budget: 200 ms
    }
}

// ---- Functional requirements (the system as a black box) -------------------

requirements functional {
    req "FUN-ACC-001" "Speed range" {
        description: "Without a target, the system shall regulate the speed to the set speed, settable from 30 km/h to 180 km/h in steps of 1 km/h and 10 km/h"
        priority: High  asil: "QM"  verification_method: "test"
    }
    req "FUN-ACC-002" "Speed accuracy" {
        description: "On a level road the speed shall stay within 2 km/h of the set speed; on slopes up to 8 %, within 5 km/h"
        priority: Medium  asil: "QM"  verification_method: "test"
    }
    req "FUN-ACC-003" "Time gap settings" {
        description: "The driver shall choose the time gap among 1.0 s, 1.4 s, 1.8 s and 2.5 s; 1.8 s is selected at each engagement"
        priority: High  asil: "QM"  verification_method: "inspection"
    }
    req "FUN-ACC-004" "Time gap held" {
        description: "While following, the time gap shall never fall below 95 % of the setting when the vehicle ahead brakes at up to 3 m/s2"
        priority: Critical  asil: "ASIL-B"  verification_method: "analysis"
        rationale: "ISO 15622 §6.2.3: the steady-state time gap must hold; the transient margin keeps the following driver's expectations"
    }
    req "FUN-ACC-005" "Standstill distance" {
        description: "Following into standstill, the car shall stop between 3.5 m and 6 m behind the target and shall never move backwards"
        priority: High  asil: "ASIL-B"  verification_method: "analysis"
    }
    req "FUN-ACC-006" "Standstill hold" {
        description: "At standstill the car shall be held by the service brake, then by the electric park brake after 180 s"
        priority: High  asil: "ASIL-B"  verification_method: "test"
    }
    req "FUN-ACC-007" "Drive-off" {
        description: "After a standstill the car shall drive off again only on the driver's confirmation (resume or accelerator), and reach the target's speed"
        priority: High  asil: "ASIL-B"  verification_method: "analysis"
    }
    req "FUN-ACC-008" "Acceleration envelope" {
        description: "The commanded acceleration shall stay within -3.5 m/s2 and +2.0 m/s2 in every situation"
        priority: Critical  asil: "ASIL-B"  verification_method: "analysis"
        rationale: "ISO 15622 §6.3.2 limits for speeds above 20 m/s; stronger braking belongs to the driver and to the emergency braking function"
    }
    req "FUN-ACC-009" "Jerk limit" {
        description: "The commanded jerk shall not exceed 2.5 m/s3 in magnitude"
        priority: High  asil: "QM"  verification_method: "analysis"
    }
    req "FUN-ACC-010" "Cut-in detection" {
        description: "A vehicle entering the ego lane shall be confirmed as the new target within 300 ms of crossing the lane marking"
        priority: High  asil: "ASIL-B"  verification_method: "test"
    }
    req "FUN-ACC-011" "Cut-in reaction" {
        description: "After a cut-in at 20 m or more with a closing speed up to 20 km/h, the distance shall stay above 5 m and the time gap shall be restored to 95 % of the setting within 20 s"
        priority: High  asil: "ASIL-B"  verification_method: "analysis"
    }
    req "FUN-ACC-012" "Target in curves" {
        description: "The target shall be selected inside the predicted path on curves of radius 250 m and more"
        priority: Medium  asil: "ASIL-B"  verification_method: "test"
    }
    req "FUN-ACC-013" "Takeover request" {
        description: "When the deceleration needed to stop behind the target exceeds 3.5 m/s2, the takeover request shall be shown and sounded within 100 ms"
        priority: Critical  asil: "ASIL-B"  verification_method: "analysis"
    }
    req "FUN-ACC-014" "Brake pedal override" {
        description: "A brake pedal input shall end the control within 100 ms: requests released, mode overridden until the driver resumes"
        priority: Critical  asil: "ASIL-B"  verification_method: "test"
    }
    req "FUN-ACC-015" "Accelerator override" {
        description: "While the accelerator asks for more than the system, the system shall request no braking"
        priority: High  asil: "ASIL-B"  verification_method: "test"
    }
    req "FUN-ACC-016" "Fault reaction" {
        description: "On a fault of a sensor or of the controller, the requests shall be ramped out within the jerk limit and the takeover request raised within 200 ms"
        priority: Critical  asil: "ASIL-B"  verification_method: "test"
    }
    req "FUN-ACC-017" "Engagement conditions" {
        description: "The system shall engage only with the doors closed, the driver's belt fastened, the ESC active and no fault present"
        priority: High  asil: "ASIL-A"  verification_method: "test"
    }
    req "FUN-ACC-018" "State display" {
        description: "The cluster shall show the mode, the set speed, the time gap and whether a target is followed"
        priority: Medium  asil: "QM"  verification_method: "demonstration"
    }
    req "FUN-ACC-019" "Fault memory" {
        description: "Each fault shall be stored as a DTC with a freeze frame and read with UDS (ISO 14229) services 0x19 and 0x14"
        priority: Medium  asil: "QM"  verification_method: "test"
    }
    req "FUN-ACC-020" "Sensor calibration" {
        description: "Radar and camera alignment shall be performed by a UDS routine against a static target in less than 20 minutes"
        priority: Medium  asil: "QM"  verification_method: "demonstration"
    }
    req "FUN-ACC-021" "Authenticated requests" {
        description: "Every brake and torque request shall carry a message authentication code and a freshness value; a request failing the check shall be ignored and reported"
        priority: High  asil: "QM"  verification_method: "test"
    }
}

// ---- Functional modes: the behaviour the driver sees (executable, see io_binding) --

state_machine "ACC operating modes" {
    kind: "functional_modes"
    initial: "Off"
    mode Off { description: "Main switch off" during: ["Vehicle lifecycle.InService", "Vehicle lifecycle.Workshop"] }
    mode Standby { description: "Main switch on, not regulating" during: ["Vehicle lifecycle.InService"] }
    mode Cruising { description: "Holding the set speed, no target" during: ["Vehicle lifecycle.InService"] }
    mode Following { description: "Holding the time gap to the target" during: ["Vehicle lifecycle.InService"] }
    mode Standstill { description: "Stopped behind the target, held by the brakes" during: ["Vehicle lifecycle.InService"] }
    mode Overridden { description: "The driver braked: no request until resumed" during: ["Vehicle lifecycle.InService"] }
    mode Degraded { description: "Fault: requests ramped out, takeover requested" during: ["Vehicle lifecycle.InService"] }
    transition Off -> Standby { trigger: "LF-SWITCHES" guard: "main switch on" }
    transition Standby -> Off { trigger: "LF-SWITCHES" guard: "main switch off" }
    transition Standby -> Cruising { trigger: "LF-SWITCHES" guard: "set pressed above 30 km/h" }
    transition Cruising -> Following { trigger: "LF-SELECT" guard: "target in path" }
    transition Following -> Cruising { trigger: "LF-SELECT" guard: "target lost" }
    transition Following -> Standstill { trigger: "LF-STANDSTILL" guard: "vehicle stopped" }
    transition Standstill -> Following { trigger: "LF-SWITCHES" guard: "resume pressed" }
    transition Cruising -> Overridden { trigger: "LF-BRAKEPEDAL" }
    transition Following -> Overridden { trigger: "LF-BRAKEPEDAL" }
    transition Standstill -> Overridden { trigger: "LF-BRAKEPEDAL" }
    transition Overridden -> Standby { trigger: "LF-SWITCHES" guard: "pedal released" }
    transition Cruising -> Degraded { trigger: "LF-FAULTS" }
    transition Following -> Degraded { trigger: "LF-FAULTS" }
    transition Standstill -> Degraded { trigger: "LF-FAULTS" }
    transition Degraded -> Off { trigger: "LF-SWITCHES" guard: "main switch off" }
}

// ---- Functional scenarios -----------------------------------------------------

scenario "One following cycle" {
    id: "SC-FUN-FOLLOW"
    capability: "CAP-FOLLOW"
    participants: ["SF-RADAR", "SF-CAMERA", "SF-FUSE", "SF-SELECT", "SF-GAP", "SF-DIST", "SF-ARB", "SF-LIMIT", "SF-BRAKE"]
    message "SF-RADAR" -> "SF-FUSE" "radar objects" { timing: 50 ms }
    message "SF-CAMERA" -> "SF-FUSE" "camera objects" { timing: 33 ms }
    message "SF-FUSE" -> "SF-SELECT" "tracks"
    message "SF-SELECT" -> "SF-DIST" "target: 48 m, -3 m/s, -3 m/s2"
    message "SF-GAP" -> "SF-DIST" "distance to keep: 54 m"
    message "SF-DIST" -> "SF-ARB" "-2.1 m/s2"
    message "SF-ARB" -> "SF-LIMIT" "-2.1 m/s2"
    message "SF-LIMIT" -> "SF-BRAKE" "-2.1 m/s2 within the envelope"
}

scenario "Brake pedal during following" {
    id: "SC-FUN-OVERRIDE"
    capability: "CAP-HANDOVER"
    participants: ["SF-OVR", "SF-MODE", "SF-ARB", "SF-LIMIT", "SF-TORQUE", "SF-HMI"]
    state "SF-MODE" "Following"
    message "SF-OVR" -> "SF-MODE" "brake pedal pressed" { timing: 10 ms }
    state "SF-MODE" "Overridden"
    message "SF-MODE" -> "SF-ARB" "mode overridden: no request"
    message "SF-ARB" -> "SF-LIMIT" "0 m/s2"
    message "SF-LIMIT" -> "SF-TORQUE" "requests released" { timing: 100 ms }
    message "SF-MODE" -> "SF-HMI" "overridden"
}

// ---- Functional interfaces: the data the functions exchange --------------------

class "ObjectTrack" {
    id: "CL-TRACK"
    track_id: "uint8"  range: "Distance"  range_rate: "Speed"  acceleration: "Acceleration"
    lateral_offset: "Distance"  existence_probability: "Probability"  object_class: "ObjectClass"  lane: "LaneAssignment"
}
class "TargetObject" {
    id: "CL-TARGET"  extends: "CL-TRACK"
    selected_since: "Duration"  in_path_confidence: "Probability"
}
class "EgoMotion" { id: "CL-EGO" speed: "Speed"  acceleration: "Acceleration"  yaw_rate: "YawRate"  standstill: "bool" }
class "LanePath" { id: "CL-PATH" curvature: "Curvature"  curvature_rate: "CurvatureRate"  lane_width: "Distance"  quality: "uint8" }
class "DriverSettings" { id: "CL-SETTINGS" main_switch: "bool"  set_speed: "Speed"  time_gap: "TimeGapSetting"  resume: "bool"  cancel: "bool" }
class "AccelerationRequest" { id: "CL-AREQ" acceleration: "Acceleration"  jerk_limit: "Jerk"  source: "string" }
class "BrakeRequest" { id: "CL-BREQ" deceleration: "Acceleration"  hold: "bool"  counter: "uint8"  mac: "uint64" }
class "TorqueRequest" { id: "CL-TREQ" wheel_torque: "Torque"  counter: "uint8"  mac: "uint64" }
class "AccDisplay" { id: "CL-DISPLAY" mode: "AccMode"  set_speed: "Speed"  time_gap: "TimeGapSetting"  target_detected: "bool"  target: "TargetObject" }
class "TakeoverRequest" { id: "CL-TOR" active: "bool"  reason: "FaultClass" }
class "HealthStatus" { id: "CL-HEALTH" radar_ok: "bool"  camera_ok: "bool"  controller_ok: "bool"  fault: "FaultClass"  codes: "DiagnosticTroubleCode" }
class "DiagnosticTroubleCode" { id: "CL-DTC" code: "uint32"  status: "uint8"  freeze_frame: "FreezeFrame" }

enumeration "AccMode" { id: "EN-MODE" values: ["Off", "Standby", "Cruising", "Following", "Standstill", "Overridden", "Degraded"] }
enumeration "LaneAssignment" { id: "EN-LANE" values: ["EgoLane", "LeftLane", "RightLane", "Unknown"] }
enumeration "ObjectClass" { id: "EN-CLASS" values: ["Car", "Truck", "Motorcycle", "Pedestrian", "Unknown"] }
enumeration "TimeGapSetting" { id: "EN-GAP" values: ["Gap1_0s", "Gap1_4s", "Gap1_8s", "Gap2_5s"] }
enumeration "FaultClass" { id: "EN-FAULT" values: ["None", "RadarBlind", "CameraUnavailable", "SensorDisagreement", "ControllerFault", "ActuatorUnavailable", "MessageAuthentication"] }
data_type "Speed" { id: "DT-SPEED" base: "float" unit: "m/s" }
data_type "Distance" { id: "DT-DIST" base: "float" unit: "m" }
data_type "TimeGap" { id: "DT-GAP" base: "float" unit: "s" }
data_type "Duration" { id: "DT-DURATION" base: "uint32" unit: "ms" }
data_type "Acceleration" { id: "DT-ACCEL" base: "float" description: "m/s², positive forward" }
data_type "Jerk" { id: "DT-JERK" base: "float" description: "m/s³" }
data_type "YawRate" { id: "DT-YAW" base: "float" description: "rad/s, positive to the left" }
data_type "Curvature" { id: "DT-CURV" base: "float" description: "1/m, positive to the left" }
data_type "CurvatureRate" { id: "DT-CURVRATE" base: "float" description: "1/m²" }
data_type "Torque" { id: "DT-TORQUE" base: "float" description: "N·m at the wheels" }
data_type "Pressure" { id: "DT-PRESSURE" base: "float" unit: "bar" }
data_type "Percent" { id: "DT-PERCENT" base: "float" unit: "%" }
data_type "Probability" { id: "DT-PROB" base: "float" description: "0 to 1" }
data_type "FreezeFrame" { id: "DT-FREEZE" base: "bytes" description: "The signals stored with a DTC (UDS 0x19 0x04)" }

association "display shows the target" { id: "AS-DISP" from: "CL-DISPLAY" to: "CL-TARGET" role: "target" kind: "association" multiplicity: "0..1" }
association "health reports faults" { id: "AS-HLT" from: "CL-HEALTH" to: "CL-DTC" role: "codes" kind: "composition" multiplicity: "0..*" }

class "VehicleDynamics" { id: "CL-VDYN" wheel_speed_front_left: "Speed"  wheel_speed_front_right: "Speed"  wheel_speed_rear_left: "Speed"  wheel_speed_rear_right: "Speed"  yaw_rate: "YawRate"  longitudinal_acceleration: "Acceleration" }
class "SwitchState" { id: "CL-SWITCH" main: "bool"  set_minus: "bool"  resume_plus: "bool"  cancel: "bool"  gap_button: "bool" }
class "PedalState" { id: "CL-PEDALS" brake_switch: "bool"  brake_pressure: "Pressure"  accelerator_position: "Percent" }
class "BodyState" { id: "CL-BODY" doors_closed: "bool"  bonnet_closed: "bool"  driver_belt: "bool" }
class "LaneChangeEvent" { id: "CL-LANECHG" track_id: "uint8"  direction: "string"  cut_in: "bool" }
class "OverrideEvent" { id: "CL-OVR" source: "string"  brake: "bool"  accelerator: "bool" }

exchange_item "Radar echo" { id: "EI-ECHO" mechanism: "FLOW" }
exchange_item "Camera image" { id: "EI-IMAGE" mechanism: "FLOW" }
exchange_item "Vehicle dynamics" { id: "EI-VDYN" mechanism: "FLOW" elements: ["CL-VDYN"] }
exchange_item "Driver settings" { id: "EI-SETTINGS" mechanism: "DATA" elements: ["CL-SETTINGS", "CL-SWITCH"] }
exchange_item "Pedal state" { id: "EI-PEDALS" mechanism: "FLOW" elements: ["CL-PEDALS"] }
exchange_item "Body state" { id: "EI-BODY" mechanism: "DATA" elements: ["CL-BODY"] }
exchange_item "Brake request" { id: "EI-BREQ" mechanism: "FLOW" elements: ["CL-BREQ"] }
exchange_item "Torque request" { id: "EI-TREQ" mechanism: "FLOW" elements: ["CL-TREQ"] }
exchange_item "Park request" { id: "EI-PARK" mechanism: "EVENT" }
exchange_item "ACC display" { id: "EI-DISPLAY" mechanism: "DATA" elements: ["CL-DISPLAY"] }
exchange_item "Takeover request" { id: "EI-TOR" mechanism: "EVENT" elements: ["CL-TOR"] }
exchange_item "UDS request" { id: "EI-UDSREQ" mechanism: "OPERATION" }
exchange_item "UDS response" { id: "EI-UDSRSP" mechanism: "OPERATION" elements: ["CL-DTC"] }
exchange_item "Object list" { id: "EI-OBJECTS" mechanism: "FLOW" elements: ["CL-TRACK"] }
exchange_item "Lane path" { id: "EI-PATH" mechanism: "FLOW" elements: ["CL-PATH"] }
exchange_item "Ego motion" { id: "EI-EGO" mechanism: "FLOW" elements: ["CL-EGO"] }
exchange_item "Tracks" { id: "EI-TRACKS" mechanism: "FLOW" elements: ["CL-TRACK"] }
exchange_item "Target" { id: "EI-TARGET" mechanism: "FLOW" elements: ["CL-TARGET"] }
exchange_item "Lane change" { id: "EI-LANECHG" mechanism: "EVENT" elements: ["CL-LANECHG"] }
exchange_item "Override" { id: "EI-OVR" mechanism: "EVENT" elements: ["CL-OVR"] }
exchange_item "Health" { id: "EI-HEALTH" mechanism: "DATA" elements: ["CL-HEALTH"] }
exchange_item "Fault report" { id: "EI-FAULT" mechanism: "EVENT" elements: ["CL-DTC"] }
exchange_item "ACC mode" { id: "EI-MODE" mechanism: "DATA" }
exchange_item "Distance to keep" { id: "EI-GAPREF" mechanism: "FLOW" }
exchange_item "Acceleration demand" { id: "EI-AREQ" mechanism: "FLOW" elements: ["CL-AREQ"] }

// ===========================================================================
// CONSTRUCTIONAL VISION — how the system is built
// ===========================================================================

// ---- Logical architecture: the software and sensor components --------------

logical_architecture "ACC logical architecture" {
    // Perception
    component "Radar sensor" {
        id: "LC-RADAR"  asil: "ASIL-B"  type: "sensor"
        description: "77 GHz FMCW radar, 4 transmitters and 4 receivers: detections clustered into objects"
        port out "radar_objects" { interface: "Object list" }
        port out "radar_status" { interface: "Health" }
        function "Detect and cluster radar targets" { id: "LF-RADAR-DET" realizes: "SF-RADAR" latency: 45 ms period: 50 ms }
        function "Check radar blockage and alignment" { id: "LF-RADAR-BIST" realizes: "SF-PLAUS" period: 100 ms }
    }
    component "Front camera" {
        id: "LC-CAMERA"  asil: "ASIL-B"  type: "sensor"
        description: "Monocular camera behind the windscreen, 8 Mpixel, 120° and 30° fields"
        port out "camera_objects" { interface: "Object list" }
        port out "lanes" { interface: "Lane path" }
        port out "camera_status" { interface: "Health" }
        function "Detect and classify objects" { id: "LF-CAM-OBJ" realizes: "SF-CAMERA" latency: 30 ms period: 33 ms }
        function "Detect lane markings" { id: "LF-CAM-LANE" realizes: "SF-CAMERA" latency: 30 ms period: 33 ms }
        function "Detect camera blindness" { id: "LF-CAM-BLIND" realizes: "SF-PLAUS" period: 200 ms }
    }
    component "Ego motion estimator" {
        id: "LC-EGO"  asil: "ASIL-B"  type: "software"
        port in "vehicle_dynamics" { interface: "Vehicle dynamics" }
        port out "ego_motion" { interface: "Ego motion" }
        function "Estimate speed, acceleration and yaw rate" { id: "LF-EGO" realizes: "SF-EGO" latency: 2 ms period: 10 ms }
        function "Detect standstill" { id: "LF-EGO-STOP" realizes: "SF-EGO" period: 10 ms }
    }
    component "Sensor fusion" {
        id: "LC-FUSION"  asil: "ASIL-B"  type: "software"
        description: "Radar-camera association and multi-object tracking (interacting multiple model Kalman filters)"
        port in "radar_objects" { interface: "Object list" }
        port in "camera_objects" { interface: "Object list" }
        port in "lanes" { interface: "Lane path" }
        port in "ego_motion" { interface: "Ego motion" }
        port out "tracks" { interface: "Tracks" }
        port out "path" { interface: "Lane path" }
        function "Associate radar and camera objects" { id: "LF-ASSOC" realizes: "SF-FUSE" latency: 5 ms }
        function "Track objects" { id: "LF-TRACK" realizes: "SF-FUSE" latency: 8 ms }
        function "Predict the ego path" { id: "LF-PATH" realizes: "SF-PATH" latency: 3 ms }
    }
    component "Situation assessment" {
        id: "LC-SITUATION"  asil: "ASIL-B"  type: "software"
        port in "tracks" { interface: "Tracks" }
        port in "path" { interface: "Lane path" }
        port out "target" { interface: "Target" }
        port out "lane_change" { interface: "Lane change" }
        function "Select the target" { id: "LF-SELECT" realizes: "SF-SELECT" latency: 3 ms }
        function "Detect lane changes ahead" { id: "LF-CUTIN" realizes: "SF-CUTDET" latency: 100 ms description: "Lateral crossing confirmed over three fusion cycles" }
    }
    // Driver interaction
    component "Driver input handler" {
        id: "LC-HMI-IN"  asil: "ASIL-B"  type: "software"
        port in "switches" { interface: "Driver settings" }
        port in "pedals" { interface: "Pedal state" }
        port out "settings" { interface: "Driver settings" }
        port out "override" { interface: "Override" }
        function "Read the steering wheel switches" { id: "LF-SWITCHES" realizes: "SF-SET" latency: 10 ms }
        function "Detect the brake pedal" { id: "LF-BRAKEPEDAL" realizes: "SF-OVR" latency: 5 ms description: "Brake switch and master cylinder pressure, cross-checked" }
        function "Detect accelerator override" { id: "LF-ACCELPEDAL" realizes: "SF-OVR" latency: 10 ms }
    }
    component "Driver information" {
        id: "LC-HMI-OUT"  asil: "ASIL-B"  type: "software"
        port in "mode" { interface: "ACC mode" }
        port in "settings" { interface: "Driver settings" }
        port in "need" { interface: "Acceleration demand" }
        port in "health" { interface: "Health" }
        port out "hmi_frames" { interface: "ACC display" }
        function "Compose the ACC display" { id: "LF-DISPLAY" realizes: "SF-HMI" latency: 20 ms }
        function "Raise the takeover request" { id: "LF-TOR" realizes: "SF-TOR" latency: 15 ms }
    }
    // Decision and control
    component "ACC mode manager" {
        id: "LC-MODE"  asil: "ASIL-B"  type: "software"
        description: "The executable mode machine 'ACC operating modes', generated as C and proven against the model"
        port in "settings" { interface: "Driver settings" }
        port in "override" { interface: "Override" }
        port in "health" { interface: "Health" }
        port in "target" { interface: "Target" }
        port in "body_state" { interface: "Body state" }
        port out "mode" { interface: "ACC mode" }
        function "Run the mode machine" { id: "LF-MODE" realizes: "SF-MODE" latency: 2 ms period: 10 ms }
        function "Check the engagement conditions" { id: "LF-ENGAGE" realizes: "SF-MODE" latency: 1 ms }
    }
    component "Longitudinal controller" {
        id: "LC-CTRL"  asil: "ASIL-B"  type: "software"
        description: "Speed and constant-time-gap distance control, the law simulated in physics/ACCPhysics.mo"
        port in "settings" { interface: "Driver settings" }
        port in "ego_motion" { interface: "Ego motion" }
        port in "target" { interface: "Target" }
        port in "lane_change" { interface: "Lane change" }
        port in "mode" { interface: "ACC mode" }
        port out "acceleration_request" { interface: "Acceleration demand" }
        port out "need" { interface: "Acceleration demand" }
        function "Compute the distance to keep" { id: "LF-GAP" realizes: "SF-GAP" latency: 1 ms }
        function "Control the speed" { id: "LF-SPEEDCTL" realizes: "SF-SPEED" latency: 1 ms period: 20 ms }
        function "Control the distance" { id: "LF-DISTCTL" realizes: "SF-DIST" latency: 2 ms period: 20 ms }
        function "Estimate the deceleration needed" { id: "LF-NEED" realizes: "SF-DIST" latency: 1 ms }
        function "Arbitrate speed and distance" { id: "LF-ARB" realizes: "SF-ARB" latency: 1 ms }
    }
    // Safety
    component "Safety monitor" {
        id: "LC-SAFETY"  asil: "ASIL-B"  type: "software"
        description: "Independent of the controller (separate lockstep core, own copy of the inputs): bounds the command and decides the safe state"
        port in "acceleration_request" { interface: "Acceleration demand" }
        port in "tracks" { interface: "Tracks" }
        port in "radar_status" { interface: "Health" }
        port in "camera_status" { interface: "Health" }
        port out "acceleration_command" { interface: "Acceleration demand" }
        port out "health" { interface: "Health" }
        port out "fault_report" { interface: "Fault report" }
        function "Enforce the acceleration envelope" { id: "LF-ENVELOPE" realizes: "SF-LIMIT" latency: 1 ms }
        function "Cross-check radar and camera" { id: "LF-PLAUS" realizes: "SF-PLAUS" latency: 10 ms }
        function "Detect faults" { id: "LF-FAULTS" realizes: "SF-PLAUS" latency: 10 ms description: "Sensor status, plausibility, controller alive and sequence counters, RAM and stack checks" }
    }
    // Actuation and communication
    component "Actuator coordinator" {
        id: "LC-ACT"  asil: "ASIL-B"  type: "software"
        port in "acceleration_command" { interface: "Acceleration demand" }
        port in "mode" { interface: "ACC mode" }
        port out "requests" { interface: "Brake request" }
        function "Request brake deceleration" { id: "LF-BRAKEREQ" realizes: "SF-BRAKE" latency: 2 ms period: 10 ms }
        function "Request drive torque" { id: "LF-TORQUEREQ" realizes: "SF-TORQUE" latency: 2 ms period: 10 ms }
        function "Hold at standstill" { id: "LF-STANDSTILL" realizes: "SF-HOLD" latency: 2 ms description: "Hold request to the ESC; park brake request after 180 s" }
    }
    component "Vehicle bus interface" {
        id: "LC-COM"  asil: "ASIL-B"  type: "software"
        description: "AUTOSAR COM stack: E2E protection, SecOC authentication, signal gateway to the chassis CAN FD"
        port in "requests" { interface: "Brake request" }
        port in "hmi_frames" { interface: "ACC display" }
        port in "uds_response" { interface: "UDS response" }
        port out "vehicle_dynamics" { interface: "Vehicle dynamics" }
        port out "switches" { interface: "Driver settings" }
        port out "pedals" { interface: "Pedal state" }
        port out "body_state" { interface: "Body state" }
        port out "uds_request" { interface: "UDS request" }
        function "Authenticate outgoing requests" { id: "LF-MAC" realizes: "SF-BRAKE" latency: 1 ms description: "AES-128 CMAC truncated to 24 bits, 8-bit freshness" }
        function "Verify incoming frames" { id: "LF-VERIFY" realizes: "SF-EGO" latency: 1 ms description: "E2E profile 5 counters and CRC, SecOC check of the ESC frames" }
    }
    component "Diagnostic manager" {
        id: "LC-DIAG"  asil: "QM"  type: "software"
        port in "fault_report" { interface: "Fault report" }
        port in "uds_request" { interface: "UDS request" }
        port out "uds_response" { interface: "UDS response" }
        function "Store and report DTCs" { id: "LF-DTC" realizes: "SF-DIAG" }
        function "Run the alignment routines" { id: "LF-CALIB" realizes: "SF-DIAG" description: "Static target alignment of the radar (± 0.1°) and the camera" }
    }

    component_exchange "Radar objects" { id: "CE-RADOBJ" from_port: "LC-RADAR.radar_objects" to_port: "LC-FUSION.radar_objects" data_type: "Object list" period: 50 ms }
    component_exchange "Radar status" { id: "CE-RADSTAT" from_port: "LC-RADAR.radar_status" to_port: "LC-SAFETY.radar_status" data_type: "Health" period: 100 ms }
    component_exchange "Camera objects" { id: "CE-CAMOBJ" from_port: "LC-CAMERA.camera_objects" to_port: "LC-FUSION.camera_objects" data_type: "Object list" period: 33 ms }
    component_exchange "Lanes" { id: "CE-LANES" from_port: "LC-CAMERA.lanes" to_port: "LC-FUSION.lanes" data_type: "Lane path" period: 33 ms }
    component_exchange "Camera status" { id: "CE-CAMSTAT" from_port: "LC-CAMERA.camera_status" to_port: "LC-SAFETY.camera_status" data_type: "Health" period: 200 ms }
    component_exchange "Vehicle dynamics" { id: "CE-VDYN" from_port: "LC-COM.vehicle_dynamics" to_port: "LC-EGO.vehicle_dynamics" data_type: "Vehicle dynamics" period: 10 ms }
    component_exchange "Ego motion to fusion" { id: "CE-EGO-FUS" from_port: "LC-EGO.ego_motion" to_port: "LC-FUSION.ego_motion" data_type: "Ego motion" period: 10 ms }
    component_exchange "Ego motion to control" { id: "CE-EGO-CTL" from_port: "LC-EGO.ego_motion" to_port: "LC-CTRL.ego_motion" data_type: "Ego motion" period: 10 ms }
    component_exchange "Tracks" { id: "CE-TRACKS" from_port: "LC-FUSION.tracks" to_port: "LC-SITUATION.tracks" data_type: "Tracks" period: 50 ms }
    component_exchange "Tracks for monitoring" { id: "CE-TRACKS-MON" from_port: "LC-FUSION.tracks" to_port: "LC-SAFETY.tracks" data_type: "Tracks" period: 50 ms }
    component_exchange "Predicted path" { id: "CE-PATH" from_port: "LC-FUSION.path" to_port: "LC-SITUATION.path" data_type: "Lane path" period: 50 ms }
    component_exchange "Target to control" { id: "CE-TGT-CTL" from_port: "LC-SITUATION.target" to_port: "LC-CTRL.target" data_type: "Target" period: 50 ms }
    component_exchange "Target to mode manager" { id: "CE-TGT-MODE" from_port: "LC-SITUATION.target" to_port: "LC-MODE.target" data_type: "Target" period: 50 ms }
    component_exchange "Lane change" { id: "CE-LANECHG" from_port: "LC-SITUATION.lane_change" to_port: "LC-CTRL.lane_change" data_type: "Lane change" }
    component_exchange "Switches" { id: "CE-SWITCHES" from_port: "LC-COM.switches" to_port: "LC-HMI-IN.switches" data_type: "Driver settings" period: 20 ms }
    component_exchange "Pedals" { id: "CE-PEDALS" from_port: "LC-COM.pedals" to_port: "LC-HMI-IN.pedals" data_type: "Pedal state" period: 10 ms }
    component_exchange "Settings to mode manager" { id: "CE-SET-MODE" from_port: "LC-HMI-IN.settings" to_port: "LC-MODE.settings" data_type: "Driver settings" }
    component_exchange "Settings to control" { id: "CE-SET-CTL" from_port: "LC-HMI-IN.settings" to_port: "LC-CTRL.settings" data_type: "Driver settings" }
    component_exchange "Settings to display" { id: "CE-SET-HMI" from_port: "LC-HMI-IN.settings" to_port: "LC-HMI-OUT.settings" data_type: "Driver settings" }
    component_exchange "Override" { id: "CE-OVR" from_port: "LC-HMI-IN.override" to_port: "LC-MODE.override" data_type: "Override" }
    component_exchange "Body state" { id: "CE-BODY" from_port: "LC-COM.body_state" to_port: "LC-MODE.body_state" data_type: "Body state" period: 100 ms }
    component_exchange "Mode to control" { id: "CE-MODE-CTL" from_port: "LC-MODE.mode" to_port: "LC-CTRL.mode" data_type: "ACC mode" period: 10 ms }
    component_exchange "Mode to actuators" { id: "CE-MODE-ACT" from_port: "LC-MODE.mode" to_port: "LC-ACT.mode" data_type: "ACC mode" period: 10 ms }
    component_exchange "Mode to display" { id: "CE-MODE-HMI" from_port: "LC-MODE.mode" to_port: "LC-HMI-OUT.mode" data_type: "ACC mode" period: 10 ms }
    component_exchange "Health to mode manager" { id: "CE-HLT-MODE" from_port: "LC-SAFETY.health" to_port: "LC-MODE.health" data_type: "Health" period: 10 ms }
    component_exchange "Health to display" { id: "CE-HLT-HMI" from_port: "LC-SAFETY.health" to_port: "LC-HMI-OUT.health" data_type: "Health" period: 10 ms }
    component_exchange "Acceleration request" { id: "CE-AREQ" from_port: "LC-CTRL.acceleration_request" to_port: "LC-SAFETY.acceleration_request" data_type: "Acceleration demand" period: 20 ms }
    component_exchange "Deceleration needed" { id: "CE-NEED" from_port: "LC-CTRL.need" to_port: "LC-HMI-OUT.need" data_type: "Acceleration demand" period: 20 ms }
    component_exchange "Acceleration command" { id: "CE-ACMD" from_port: "LC-SAFETY.acceleration_command" to_port: "LC-ACT.acceleration_command" data_type: "Acceleration demand" period: 20 ms }
    component_exchange "Actuator requests" { id: "CE-REQ" from_port: "LC-ACT.requests" to_port: "LC-COM.requests" data_type: "Brake request" period: 10 ms }
    component_exchange "Display frames" { id: "CE-HMI" from_port: "LC-HMI-OUT.hmi_frames" to_port: "LC-COM.hmi_frames" data_type: "ACC display" period: 100 ms }
    component_exchange "Fault report" { id: "CE-FAULTS" from_port: "LC-SAFETY.fault_report" to_port: "LC-DIAG.fault_report" data_type: "Fault report" }
    component_exchange "UDS request" { id: "CE-UDSREQ" from_port: "LC-COM.uds_request" to_port: "LC-DIAG.uds_request" data_type: "UDS request" }
    component_exchange "UDS response" { id: "CE-UDSRSP" from_port: "LC-DIAG.uds_response" to_port: "LC-COM.uds_response" data_type: "UDS response" }

    interface "Object list interface" { id: "IF-OBJECTS" from: "LC-RADAR" to: "LC-FUSION" protocol: "CAN FD, 2 Mbit/s" period: 50 ms size: 512 B description: "Up to 32 objects per cycle: range, range rate, angle, RCS, existence probability" }
    interface "Camera object and lane interface" { id: "IF-CAMERA" from: "LC-CAMERA" to: "LC-FUSION" protocol: "SOME/IP over 100BASE-T1" period: 33 ms size: 4 kB }
    interface "Acceleration command interface" { id: "IF-ACMD" from: "LC-SAFETY" to: "LC-ACT" description: "Shared memory between cores, E2E profile 7 (CRC-64, counter), checked on every read" }
    interface "Chassis request interface" { id: "IF-CHASSIS" from: "LC-ACT" to: "LC-COM" protocol: "CAN FD" period: 10 ms description: "ACC_BrakeRequest 0x1A0 and ACC_TorqueRequest 0x1A1, E2E profile 5 and SecOC; the ESC times out after 30 ms" }

    capability_realization "Following realized" { id: "CR-FOLLOW" realizes: "CAP-FOLLOW" involves: ["LC-RADAR", "LC-CAMERA", "LC-EGO", "LC-FUSION", "LC-SITUATION", "LC-CTRL", "LC-SAFETY", "LC-ACT", "LC-COM"] }
    capability_realization "Stop and go realized" { id: "CR-STOPGO" realizes: "CAP-STOPGO" involves: ["LC-EGO", "LC-MODE", "LC-CTRL", "LC-ACT", "LC-COM"] }
    capability_realization "Handover realized" { id: "CR-HANDOVER" realizes: "CAP-HANDOVER" involves: ["LC-HMI-IN", "LC-MODE", "LC-HMI-OUT", "LC-ACT"] }
    capability_realization "Degradation realized" { id: "CR-DEGRADE" realizes: "CAP-DEGRADE" involves: ["LC-SAFETY", "LC-MODE", "LC-HMI-OUT", "LC-ACT"] }
    capability_realization "Diagnosis realized" { id: "CR-DIAG" realizes: "CAP-DIAG" involves: ["LC-DIAG", "LC-COM", "LC-RADAR", "LC-CAMERA"] }
}

// ---- Technical requirements: what the components and the hardware shall be -----

requirements technical {
    req "TEC-ACC-001" "Radar detection range" {
        description: "The radar shall detect a passenger car at 160 m and a motorcycle at 100 m with 99 % probability, range accuracy 0.25 m, every 50 ms"
        priority: High  asil: "ASIL-B"  subject: "LC-RADAR"  verification_method: "test"
    }
    req "TEC-ACC-002" "Radar field of view" {
        description: "The radar shall cover ±9° up to 200 m and ±60° up to 70 m"
        priority: Medium  asil: "ASIL-B"  subject: "LC-RADAR"  verification_method: "test"
    }
    req "TEC-ACC-003" "Camera performance" {
        description: "The camera shall classify vehicles up to 120 m and detect lane markings up to 60 m at 30 frames per second, from -40 °C to +85 °C behind the windscreen"
        priority: High  asil: "ASIL-B"  subject: "LC-CAMERA"  verification_method: "test"
    }
    req "TEC-ACC-004" "Fusion latency" {
        description: "Tracks shall be published at most 15 ms after the last sensor cycle, for up to 64 objects"
        priority: High  asil: "ASIL-B"  subject: "LC-FUSION"  verification_method: "analysis"
    }
    req "TEC-ACC-005" "Controller timing" {
        description: "The controller shall run every 20 ms with a worst-case execution time under 2 ms on its core"
        priority: High  asil: "ASIL-B"  subject: "LC-CTRL"  verification_method: "analysis"
    }
    req "TEC-ACC-006" "Independent safety monitor" {
        description: "The safety monitor shall run on a lockstep core separate from the controller, with its own copy of the inputs, so that no single fault defeats both (ISO 26262-9 §6 dependent failures)"
        priority: Critical  asil: "ASIL-B"  subject: "LC-SAFETY"  verification_method: "inspection"
    }
    req "TEC-ACC-007" "Protected actuator requests" {
        description: "Brake and torque requests shall be sent every 10 ms with E2E profile 5 (CRC, alive counter); the receiving ECU shall time out after 30 ms"
        priority: Critical  asil: "ASIL-B"  subject: "LC-ACT"  verification_method: "test"
    }
    req "TEC-ACC-008" "Authenticated frames" {
        description: "Brake and torque requests shall carry an AES-128 CMAC truncated to 24 bits and an 8-bit freshness value (AUTOSAR SecOC)"
        priority: High  asil: "QM"  subject: "LC-COM"  verification_method: "test"
    }
    req "TEC-ACC-009" "Brake pedal detection" {
        description: "The brake pedal shall be detected from the brake switch and the master cylinder pressure, cross-checked, within 5 ms"
        priority: Critical  asil: "ASIL-B"  subject: "LC-HMI-IN"  verification_method: "test"
    }
    req "TEC-ACC-010" "Fault memory" {
        description: "The diagnostic manager shall keep 50 DTCs with freeze frames in non-volatile memory across power cycles"
        priority: Medium  asil: "QM"  subject: "LC-DIAG"  verification_method: "test"
    }
    req "TEC-ACC-011" "Processing headroom" {
        description: "Each core of the ADAS controller shall stay under 70 % load in the worst case, leaving room for updates"
        priority: Medium  asil: "QM"  subject: "PN-ADAS"  verification_method: "analysis"
    }
    req "TEC-ACC-012" "Bus load" {
        description: "The chassis CAN FD shall stay under 50 % load with the ACC frames added"
        priority: Medium  asil: "QM"  subject: "PL-CHASSIS"  verification_method: "analysis"
    }
    req "TEC-ACC-013" "Power" {
        description: "The radar, the camera and the ADAS controller together shall draw less than 40 W at 13.5 V"
        priority: Medium  asil: "QM"  verification_method: "analysis"
    }
    req "TEC-ACC-014" "Camera blindness" {
        description: "A blinded or dirty camera shall be detected within 2 s"
        priority: High  asil: "ASIL-B"  subject: "LC-CAMERA"  verification_method: "test"
    }
    req "TEC-ACC-015" "Standstill hand-over" {
        description: "At standstill the actuator coordinator shall send a hold request to the ESC and, after 180 s, a park request to the electric park brake"
        priority: High  asil: "ASIL-B"  subject: "LC-ACT"  verification_method: "test"
    }
}

// ---- Technical configurations: what the hardware state allows ---------------

state_machine "Sensor configurations" {
    kind: "technical_configurations"
    initial: "FullSensorSet"
    state FullSensorSet { description: "Radar and camera available: every mode" realizes: ["ACC operating modes.Standby", "ACC operating modes.Cruising", "ACC operating modes.Following", "ACC operating modes.Standstill", "ACC operating modes.Overridden"] during: ["Vehicle lifecycle.InService"] }
    state RadarOnly { description: "Camera blinded or failed: following above 30 km/h only, no stop and go, no cut-in anticipation" realizes: ["ACC operating modes.Standby", "ACC operating modes.Cruising", "ACC operating modes.Following", "ACC operating modes.Overridden"] during: ["Vehicle lifecycle.InService"] }
    state SafeOff { description: "Radar or controller fault: the system stays off" realizes: ["ACC operating modes.Off", "ACC operating modes.Degraded"] during: ["Vehicle lifecycle.InService", "Vehicle lifecycle.Workshop"] }
    transition FullSensorSet -> RadarOnly { guard: "camera blinded" }
    transition RadarOnly -> FullSensorSet { guard: "camera available for 10 s" }
    transition FullSensorSet -> SafeOff { guard: "radar or controller fault" }
    transition RadarOnly -> SafeOff { guard: "radar or controller fault" }
    transition SafeOff -> FullSensorSet { guard: "self-test passed at ignition" }
}

// ---- Physical architecture: the ECUs, their cores and the buses ---------------

physical_architecture "ACC physical architecture" {
    node "Front radar" {
        id: "PN-RADAR"  asil: "ASIL-B"  type: "sensor ECU"
        description: "Behind the front bumper, heated radome"
        processor: "Radar MCU with hardware FFT accelerator"
        power: 5 W  mass: 0.25 kg  voltage: 12 V
        position: { x: 3.85 m y: 0 m z: 0.45 m }
        shape: { box: [110 mm, 80 mm, 35 mm] }
        hardware_component "77 GHz transceiver" { id: "HW-MMIC" description: "4 transmitters, 4 receivers, 76-81 GHz" power: 2 W }
        hardware_component "Radar microcontroller" { id: "HW-RADMCU" description: "Lockstep cores, 4 MB flash" }
        behavior_component "Radar firmware" {
            id: "BC-RADAR"  allocated_component: "LC-RADAR"  asil: "ASIL-B"
            function "Range-Doppler processing and CFAR detection" { id: "PF-RADAR-DSP" realizes: "LF-RADAR-DET" period: 50 ms }
            function "Blockage and misalignment monitoring" { id: "PF-RADAR-BIST" realizes: "LF-RADAR-BIST" period: 100 ms }
        }
    }
    node "Windscreen camera" {
        id: "PN-CAMERA"  asil: "ASIL-B"  type: "sensor ECU"
        processor: "Vision SoC with neural accelerator"
        power: 6 W  mass: 0.18 kg  voltage: 12 V
        position: { x: 1.9 m y: 0 m z: 1.35 m }
        hardware_component "8 Mpixel imager" { id: "HW-IMAGER" }
        behavior_component "Camera firmware" {
            id: "BC-CAMERA"  allocated_component: "LC-CAMERA"  asil: "ASIL-B"
            function "Object detection network" { id: "PF-CAM-OBJ" realizes: "LF-CAM-OBJ" period: 33 ms }
            function "Lane detection" { id: "PF-CAM-LANE" realizes: "LF-CAM-LANE" period: 33 ms }
            function "Blindness detection" { id: "PF-CAM-BLIND" realizes: "LF-CAM-BLIND" period: 200 ms }
        }
    }
    node "ADAS domain controller" {
        id: "PN-ADAS"  asil: "ASIL-B"  type: "domain ECU"
        description: "Two processors on one board: an application SoC for perception and control, a lockstep safety MCU for the monitor, the actuators and the communication"
        processor: "8-core application SoC + 3-core lockstep safety MCU"
        ram: 8 GB  flash: 64 MB  storage: 32 GB  power: 22 W  voltage: 12 V  mass: 0.9 kg
        hardware_component "Application SoC" { id: "HW-SOC" description: "8 Arm cores at 2 GHz, 8 GB LPDDR4" power: 15 W }
        hardware_component "Safety MCU" { id: "HW-SMCU" description: "3 lockstep cores, ASIL-D capable, hardware security module" power: 2 W }
        hardware_component "CAN FD and Ethernet transceivers" { id: "HW-XCVR" }
        behavior_component "Ego motion software" { id: "BC-EGO" allocated_component: "LC-EGO" function "Ego motion filter" { id: "PF-EGO" realizes: "LF-EGO" period: 10 ms } function "Standstill detection" { id: "PF-EGO-STOP" realizes: "LF-EGO-STOP" } }
        behavior_component "Fusion software" { id: "BC-FUSION" allocated_component: "LC-FUSION" function "Association" { id: "PF-ASSOC" realizes: "LF-ASSOC" } function "IMM tracking" { id: "PF-TRACK" realizes: "LF-TRACK" } function "Path prediction" { id: "PF-PATH" realizes: "LF-PATH" } }
        behavior_component "Situation software" { id: "BC-SITUATION" allocated_component: "LC-SITUATION" function "Target selection" { id: "PF-SELECT" realizes: "LF-SELECT" } function "Lane change detection" { id: "PF-CUTIN" realizes: "LF-CUTIN" } }
        behavior_component "Driver input software" { id: "BC-HMI-IN" allocated_component: "LC-HMI-IN" function "Switch decoding" { id: "PF-SWITCHES" realizes: "LF-SWITCHES" } function "Brake pedal detection" { id: "PF-BRAKEPEDAL" realizes: "LF-BRAKEPEDAL" } function "Accelerator override" { id: "PF-ACCELPEDAL" realizes: "LF-ACCELPEDAL" } }
        behavior_component "Driver information software" { id: "BC-HMI-OUT" allocated_component: "LC-HMI-OUT" function "Display composition" { id: "PF-DISPLAY" realizes: "LF-DISPLAY" } function "Takeover request" { id: "PF-TOR" realizes: "LF-TOR" } }
        behavior_component "Mode manager software" { id: "BC-MODE" allocated_component: "LC-MODE" function "Generated mode machine step" { id: "PF-MODE" realizes: "LF-MODE" period: 10 ms } function "Engagement check" { id: "PF-ENGAGE" realizes: "LF-ENGAGE" } }
        behavior_component "Controller software" { id: "BC-CTRL" allocated_component: "LC-CTRL" function "Distance reference" { id: "PF-GAP" realizes: "LF-GAP" } function "Speed loop" { id: "PF-SPEEDCTL" realizes: "LF-SPEEDCTL" period: 20 ms } function "Distance loop" { id: "PF-DISTCTL" realizes: "LF-DISTCTL" period: 20 ms } function "Deceleration needed" { id: "PF-NEED" realizes: "LF-NEED" } function "Arbitration" { id: "PF-ARB" realizes: "LF-ARB" } }
        behavior_component "Safety monitor software" { id: "BC-SAFETY" allocated_component: "LC-SAFETY" asil: "ASIL-B" function "Envelope clamp" { id: "PF-ENVELOPE" realizes: "LF-ENVELOPE" } function "Plausibility" { id: "PF-PLAUS" realizes: "LF-PLAUS" } function "Fault detection" { id: "PF-FAULTS" realizes: "LF-FAULTS" } }
        behavior_component "Actuator software" { id: "BC-ACT" allocated_component: "LC-ACT" function "Brake request" { id: "PF-BRAKEREQ" realizes: "LF-BRAKEREQ" period: 10 ms } function "Torque request" { id: "PF-TORQUEREQ" realizes: "LF-TORQUEREQ" period: 10 ms } function "Standstill hold" { id: "PF-STANDSTILL" realizes: "LF-STANDSTILL" } }
        behavior_component "Communication stack" { id: "BC-COM" allocated_component: "LC-COM" function "SecOC authentication" { id: "PF-MAC" realizes: "LF-MAC" } function "E2E and SecOC verification" { id: "PF-VERIFY" realizes: "LF-VERIFY" } }
        behavior_component "Diagnostic software" { id: "BC-DIAG" allocated_component: "LC-DIAG" asil: "QM" function "DTC manager" { id: "PF-DTC" realizes: "LF-DTC" } function "Alignment routines" { id: "PF-CALIB" realizes: "LF-CALIB" } }
    }
    node "Electronic stability control" {
        id: "PN-ESC"  asil: "ASIL-D"  type: "chassis ECU (supplied)"
        description: "Brake actuator: builds the deceleration requested, holds the car, measures the wheel speeds"
    }
    node "Engine control module" { id: "PN-ECM" asil: "ASIL-B" type: "powertrain ECU (supplied)" }
    node "Electric park brake" { id: "PN-EPB" asil: "ASIL-B" type: "chassis ECU (supplied)" }
    node "Central gateway" { id: "PN-GW" asil: "ASIL-B" type: "gateway (supplied)" description: "Routes between the chassis, powertrain and body networks; firewall; DoIP to the diagnostic connector" }
    node "Instrument cluster" { id: "PN-CLU" asil: "ASIL-A" type: "body ECU (supplied)" }
    node "Body controller" { id: "PN-BCM" asil: "QM" type: "body ECU (supplied)" }

    physical_link "Radar private CAN FD" { id: "PL-RADAR" from: "PN-RADAR" to: "PN-ADAS" protocol: "CAN FD" bandwidth: 2 Mbps length: 3 m max_length: 10 m }
    physical_link "Camera Ethernet" { id: "PL-CAMERA" from: "PN-CAMERA" to: "PN-ADAS" protocol: "100BASE-T1" bandwidth: 100 Mbps length: 2.5 m max_length: 15 m }
    physical_link "Chassis CAN FD" { id: "PL-CHASSIS" from: "PN-ADAS" to: "PN-ESC" protocol: "CAN FD" bandwidth: 2 Mbps length: 2 m }
    physical_link "Chassis CAN FD to park brake" { id: "PL-EPB" from: "PN-ADAS" to: "PN-EPB" protocol: "CAN FD" bandwidth: 2 Mbps length: 3.5 m }
    physical_link "Chassis CAN FD backbone" { id: "PL-BACKBONE" from: "PN-ADAS" to: "PN-GW" protocol: "CAN FD" bandwidth: 2 Mbps length: 1.5 m }
    physical_link "Powertrain CAN FD" { id: "PL-POWERTRAIN" from: "PN-GW" to: "PN-ECM" protocol: "CAN FD" bandwidth: 2 Mbps length: 2 m }
    physical_link "Body CAN to cluster" { id: "PL-CLUSTER" from: "PN-GW" to: "PN-CLU" protocol: "CAN" bandwidth: 500 kbps length: 1 m }
    physical_link "Body CAN to body controller" { id: "PL-BODY" from: "PN-GW" to: "PN-BCM" protocol: "CAN" bandwidth: 500 kbps length: 1.5 m }

    physical_exchange "Brake request frames" { id: "PX-BRAKE" from: "PN-ADAS" to: "PN-ESC" via: "PL-CHASSIS" message_type: "ACC_BrakeRequest 0x1A0" period: 10 ms frequency: 100 Hz size: 16 B }
    physical_exchange "Wheel speed frames" { id: "PX-WHEELS" from: "PN-ESC" to: "PN-ADAS" via: "PL-CHASSIS" message_type: "ESC_WheelSpeeds 0x0B0" period: 10 ms frequency: 100 Hz size: 16 B }
    physical_exchange "Park request frames" { id: "PX-PARK" from: "PN-ADAS" to: "PN-EPB" via: "PL-EPB" message_type: "ACC_ParkRequest 0x1A2" period: 100 ms frequency: 10 Hz size: 8 B }
    physical_exchange "Torque request frames" { id: "PX-TORQUE" from: "PN-ADAS" to: "PN-GW" via: "PL-BACKBONE" message_type: "ACC_TorqueRequest 0x1A1" period: 10 ms frequency: 100 Hz size: 16 B }
    physical_exchange "Display frames" { id: "PX-DISPLAY" from: "PN-ADAS" to: "PN-GW" via: "PL-BACKBONE" message_type: "ACC_Display 0x3C0" period: 100 ms frequency: 10 Hz size: 8 B }
    physical_exchange "Radar object frames" { id: "PX-RADAR" from: "PN-RADAR" to: "PN-ADAS" via: "PL-RADAR" message_type: "RDR_Objects 0x200-0x21F" period: 50 ms frequency: 20 Hz size: 512 B }
    physical_exchange "Camera object stream" { id: "PX-CAMERA" from: "PN-CAMERA" to: "PN-ADAS" via: "PL-CAMERA" message_type: "SOME/IP CameraObjects" period: 33 ms frequency: 30 Hz size: 4 kB }

    physical_path "Torque request route" { id: "PP-TORQUE" involves: ["PL-BACKBONE", "PL-POWERTRAIN"] }
    physical_path "Display route" { id: "PP-DISPLAY" involves: ["PL-BACKBONE", "PL-CLUSTER"] }
    physical_path "Body state route" { id: "PP-BODY" involves: ["PL-BODY", "PL-BACKBONE"] }
}

// ---- Product breakdown: what is bought, built and delivered --------------------

epbs "ACC product breakdown" {
    system "ACC Stop & Go kit" {
        id: "CI-ACC"
        subsystem "Sensing" {
            id: "CI-SENSING"
            item "Long-range radar module" { id: "CI-RADAR" part_number: "ACC-RDR-77-01" supplier: "Radar supplier (sourcing 2026)" }
            item "Radar bracket and heated radome" { id: "CI-RADOME" part_number: "ACC-RDR-BRK-02" }
            item "Windscreen camera module" { id: "CI-CAMERA" part_number: "ACC-CAM-8M-01" supplier: "Camera supplier (sourcing 2026)" }
            item "Camera bracket" { id: "CI-CAMBRK" part_number: "ACC-CAM-BRK-01" }
        }
        subsystem "Processing" {
            id: "CI-PROCESSING"
            item "ADAS domain controller" { id: "CI-ADAS" part_number: "ACC-ECU-ADAS-03" }
            item "ACC application software" { id: "CI-SW-APP" part_number: "SW-ACC-3.0.0" description: "Fusion, situation, controller, mode manager, driver interface" }
            item "Safety MCU software" { id: "CI-SW-SAFE" part_number: "SW-ACC-SAFE-3.0.0" description: "Safety monitor, actuator coordinator, communication stack" }
            item "Calibration data set" { id: "CI-CAL" part_number: "CAL-ACC-3.0.0-EU" }
        }
        subsystem "Driver interface" {
            id: "CI-HMI"
            item "Steering wheel switch pack" { id: "CI-SWITCHES" part_number: "ACC-SWP-01" }
            item "Cluster ACC graphics" { id: "CI-GRAPHICS" part_number: "HMI-ACC-3.0" }
        }
        subsystem "Wiring" {
            id: "CI-WIRING"
            item "Radar harness" { id: "CI-HRN-RDR" part_number: "HRN-ACC-RDR-01" }
            item "Camera Ethernet harness" { id: "CI-HRN-CAM" part_number: "HRN-ACC-CAM-01" }
        }
    }
}

// ---- Constructional scenarios ---------------------------------------------------

scenario "Cut-in at 22 m" {
    id: "SC-CON-CUTIN"
    capability: "CAP-CUTIN"
    participants: ["LC-CAMERA", "LC-RADAR", "LC-FUSION", "LC-SITUATION", "LC-CTRL", "LC-SAFETY", "LC-ACT", "LC-HMI-OUT"]
    message "LC-CAMERA" -> "LC-FUSION" "object crossing the lane marking" { exchange: "Camera objects" timing: 33 ms }
    message "LC-RADAR" -> "LC-FUSION" "new object 22 m, -5.6 m/s" { exchange: "Radar objects" timing: 50 ms }
    message "LC-FUSION" -> "LC-SITUATION" "tracks" { exchange: "Tracks" }
    message "LC-SITUATION" -> "LC-CTRL" "lane change: cut-in confirmed" { exchange: "Lane change" timing: 100 ms }
    message "LC-SITUATION" -> "LC-CTRL" "new target" { exchange: "Target to control" }
    message "LC-CTRL" -> "LC-SAFETY" "acceleration request -3.9 m/s2" { exchange: "Acceleration request" }
    message "LC-SAFETY" -> "LC-ACT" "acceleration command -3.5 m/s2 (clamped)" { exchange: "Acceleration command" }
    fragment opt "deceleration needed above 3.5 m/s2" {
        message "LC-CTRL" -> "LC-HMI-OUT" "deceleration needed 4.1 m/s2" { exchange: "Deceleration needed" }
    }
}

scenario "Stop behind the vehicle ahead and drive off" {
    id: "SC-CON-STOPGO"
    capability: "CAP-STOPGO"
    participants: ["LC-EGO", "LC-MODE", "LC-CTRL", "LC-ACT", "LC-HMI-IN"]
    state "LC-MODE" "Following"
    message "LC-EGO" -> "LC-MODE" "standstill" { exchange: "Ego motion to control" }
    state "LC-MODE" "Standstill"
    message "LC-MODE" -> "LC-ACT" "mode standstill: hold" { exchange: "Mode to actuators" }
    fragment opt "standstill longer than 180 s" {
        message "LC-ACT" -> "LC-ACT" "park request to the electric park brake"
    }
    message "LC-HMI-IN" -> "LC-MODE" "resume pressed" { exchange: "Settings to mode manager" }
    state "LC-MODE" "Following"
    message "LC-MODE" -> "LC-CTRL" "mode following" { exchange: "Mode to control" }
}

scenario "Speed control on a free motorway" {
    id: "SC-CON-SPEED"
    capability: "CAP-SPEED"
    participants: ["LC-HMI-IN", "LC-MODE", "LC-CTRL", "LC-SAFETY", "LC-ACT"]
    message "LC-HMI-IN" -> "LC-MODE" "set pressed at 112 km/h" { exchange: "Settings to mode manager" }
    state "LC-MODE" "Cruising"
    message "LC-HMI-IN" -> "LC-CTRL" "set speed 130 km/h" { exchange: "Settings to control" }
    message "LC-CTRL" -> "LC-SAFETY" "+1.2 m/s2" { exchange: "Acceleration request" }
    message "LC-SAFETY" -> "LC-ACT" "+1.2 m/s2" { exchange: "Acceleration command" }
}

scenario "Radar fault while following" {
    id: "SC-CON-FAULT"
    capability: "CAP-DEGRADE"
    participants: ["LC-RADAR", "LC-SAFETY", "LC-MODE", "LC-ACT", "LC-HMI-OUT", "LC-DIAG"]
    message "LC-RADAR" -> "LC-SAFETY" "status: blocked" { exchange: "Radar status" timing: 100 ms }
    message "LC-SAFETY" -> "LC-MODE" "health: radar blind" { exchange: "Health to mode manager" }
    state "LC-MODE" "Degraded"
    message "LC-MODE" -> "LC-ACT" "ramp the requests out" { exchange: "Mode to actuators" }
    message "LC-SAFETY" -> "LC-HMI-OUT" "takeover request" { exchange: "Health to display" timing: 200 ms }
    message "LC-SAFETY" -> "LC-DIAG" "DTC C1A01 radar blocked" { exchange: "Fault report" }
}

scenario "Workshop alignment" {
    id: "SC-CON-DIAG"
    capability: "CAP-DIAG"
    participants: ["LC-COM", "LC-DIAG", "LC-RADAR", "LC-CAMERA"]
    message "LC-COM" -> "LC-DIAG" "UDS 0x19 read DTCs" { exchange: "UDS request" }
    message "LC-DIAG" -> "LC-COM" "C1A01 radar blocked, freeze frame" { exchange: "UDS response" }
    message "LC-COM" -> "LC-DIAG" "UDS 0x31 start routine: radar alignment" { exchange: "UDS request" }
    message "LC-DIAG" -> "LC-COM" "alignment done, 0.08° residual" { exchange: "UDS response" }
}

// ===========================================================================
// SAFETY AND SECURITY
// ===========================================================================

// HARA (ISO 26262-3): each hazardous event classified, its safety requirements
// named; the FMEA lines below are the basic events of its fault tree
// (`arclang safety --standard iso26262 --fta`), whose rate is held to the
// target of its ASIL (100 FIT for ASIL B), or to the budget the OEM sets for
// ASIL A (1000 FIT).
safety_analysis {
    hazard "Unintended acceleration towards the vehicle ahead" {
        id: "HAZ-001"
        description: "The car accelerates while it should hold or slow down behind a vehicle"
        severity: "S3"  exposure: "E4"  controllability: "C1"  asil: "ASIL-B"
        mitigated_by: ["FUN-ACC-008", "FUN-ACC-014", "FUN-ACC-016", "TEC-ACC-006"]
        gate: "and"  mission_time: 1 h
    }
    hazard "Insufficient deceleration behind a braking vehicle" {
        id: "HAZ-002"
        description: "The car brakes too late or too little: target lost, wrong target, late reaction"
        severity: "S3"  exposure: "E4"  controllability: "C1"  asil: "ASIL-B"
        mitigated_by: ["FUN-ACC-004", "FUN-ACC-010", "FUN-ACC-012", "FUN-ACC-013", "FUN-ACC-016"]
        mission_time: 1 h
    }
    hazard "Unexpected harsh braking" {
        id: "HAZ-003"
        description: "Braking on a ghost target; the following vehicle runs into the car"
        severity: "S2"  exposure: "E4"  controllability: "C2"  asil: "ASIL-B"
        mitigated_by: ["FUN-ACC-008", "FUN-ACC-009", "TEC-ACC-006"]
        mission_time: 1 h
    }
    hazard "Roll-away from standstill" {
        id: "HAZ-004"
        description: "The car held by the system starts rolling on a slope"
        severity: "S2"  exposure: "E3"  controllability: "C2"  asil: "ASIL-A"
        mitigated_by: ["FUN-ACC-006", "TEC-ACC-015"]
        budget_fit: 1000  mission_time: 1 h
    }
    hazard "Drive-off without confirmation" {
        id: "HAZ-005"
        description: "The car drives off from standstill on its own into a pedestrian crossing in front"
        severity: "S3"  exposure: "E2"  controllability: "C2"  asil: "ASIL-A"
        mitigated_by: ["FUN-ACC-007"]
        budget_fit: 1000  mission_time: 1 h
    }
    hazard "Override not effective" {
        id: "HAZ-006"
        description: "The system keeps requesting torque while the driver brakes"
        severity: "S3"  exposure: "E3"  controllability: "C2"  asil: "ASIL-B"
        mitigated_by: ["FUN-ACC-014", "FUN-ACC-015", "TEC-ACC-009"]
        mission_time: 1 h
    }

    fmea "Wrong acceleration computed" {
        id: "FMEA-001"  element: "LC-CTRL"  leads_to: ["HAZ-001", "HAZ-003"]  fit: 10
        failure_mode: "The controller outputs a positive acceleration behind a closer target"
        cause: "Software fault, corrupted RAM"  effect: "Acceleration towards the vehicle ahead unless clamped"
        severity: "9"  occurrence: "2"  detection: "3"  rpn: 54
    }
    fmea "Safety monitor fails silent" {
        id: "FMEA-002"  element: "LC-SAFETY"  leads_to: ["HAZ-001"]  fit: 5
        failure_mode: "The envelope clamp passes any command"
        cause: "Latent fault of the monitor core, not caught by its startup test"  effect: "The controller's fault is no longer contained"
        severity: "9"  occurrence: "1"  detection: "4"  rpn: 36
    }
    fmea "Radar range rate drift" {
        id: "FMEA-003"  element: "LC-RADAR"  leads_to: ["HAZ-002"]  fit: 30
        failure_mode: "The closing speed is underestimated"
        cause: "Transceiver frequency drift"  effect: "Late braking behind a slower vehicle"
        severity: "8"  occurrence: "2"  detection: "3"  rpn: 48
    }
    fmea "Camera misses a cut-in" {
        id: "FMEA-004"  element: "LC-CAMERA"  leads_to: ["HAZ-002"]  fit: 20
        failure_mode: "A vehicle changing lane is seen late"
        cause: "Low sun, spray"  effect: "Cut-in detected by the radar only, 200 ms later"
        severity: "7"  occurrence: "3"  detection: "3"  rpn: 63
    }
    fmea "Wrong target in a curve" {
        id: "FMEA-005"  element: "LC-SITUATION"  leads_to: ["HAZ-002"]  fit: 40
        failure_mode: "The vehicle in the adjacent lane is followed"
        cause: "Path prediction error on a curve entry"  effect: "The real vehicle ahead is not followed"
        severity: "8"  occurrence: "3"  detection: "2"  rpn: 48
    }
    fmea "Ghost target" {
        id: "FMEA-006"  element: "LC-FUSION"  leads_to: ["HAZ-003"]  fit: 50
        failure_mode: "A radar ghost under a bridge is confirmed as a vehicle"
        cause: "Multipath reflection not rejected by the camera check"  effect: "Unneeded braking"
        severity: "6"  occurrence: "3"  detection: "3"  rpn: 54
    }
    fmea "Hold request ignored" {
        id: "FMEA-007"  element: "LC-ACT"  leads_to: ["HAZ-004"]  fit: 25
        failure_mode: "The ESC does not receive or apply the hold request"
        cause: "Frame lost, ESC fault"  effect: "The car rolls; the park brake is engaged on timeout"
        severity: "7"  occurrence: "2"  detection: "2"  rpn: 28
    }
    fmea "Spurious drive-off" {
        id: "FMEA-008"  element: "LC-MODE"  leads_to: ["HAZ-005"]  fit: 8
        failure_mode: "Standstill left for Following without a resume"
        cause: "Mode machine fault"  effect: "Drive-off without confirmation"
        severity: "9"  occurrence: "1"  detection: "3"  rpn: 27
    }
    fmea "Brake switch stuck open" {
        id: "FMEA-009"  element: "LC-HMI-IN"  leads_to: ["HAZ-006"]  fit: 15
        failure_mode: "The brake pedal is not seen by the switch"
        cause: "Switch wear"  effect: "Override detected from the brake pressure only, 20 ms later"
        severity: "8"  occurrence: "2"  detection: "2"  rpn: 32
    }
    fmea "Corrupted actuator request" {
        id: "FMEA-010"  element: "LC-COM"  leads_to: ["HAZ-003"]  fit: 2
        failure_mode: "A brake request is altered on the bus"
        cause: "EMC, transceiver fault"  effect: "Rejected by the E2E check at the ESC"
        severity: "6"  occurrence: "1"  detection: "1"  rpn: 6
    }
}

security_scales {
    id: "SEC-SCALES"
    confidentiality: ["0", "1", "2", "3"]
    integrity: ["0", "1", "2", "3"]
    availability: ["0", "1", "2", "3"]
    threat_kinds: ["Spoofing", "Tampering", "Denial of service", "Elevation of privilege"]
}

// TARA (ISO/SAE 21434): the assets whose integrity the car's safety depends on.
security_analysis {
    asset "Actuator requests" {
        id: "AS-REQUESTS"  kind: "information"
        members: ["EI-BREQ", "EI-TREQ", "LF-MAC"]
        security_integrity: "3"  security_availability: "2"
    }
    asset "Sensor picture" {
        id: "AS-SENSORS"  kind: "information"
        members: ["EI-OBJECTS", "EI-TRACKS"]
        security_integrity: "3"  security_availability: "2"
    }
    asset "Calibration and software" {
        id: "AS-CALIB"  kind: "functional"
        members: ["LF-CALIB", "LF-DTC"]
        security_integrity: "3"
    }
    threat "Forged brake or torque request" {
        id: "TH-FORGE"  kind: "Spoofing"  level: 3
        rationale: "A compromised ECU on the chassis network sends ACC requests to the ESC or the engine"
        applies_to: ["AS-REQUESTS"]  involves: ["LC-COM", "LC-ACT"]
        security_integrity: "3"
    }
    threat "Radar jamming or spoofing" {
        id: "TH-JAM"  kind: "Denial of service"  level: 2
        rationale: "A transmitter in the 77 GHz band blinds the radar or injects a false object"
        applies_to: ["AS-SENSORS"]  involves: ["LC-RADAR", "LC-FUSION"]
        security_availability: "2"  security_integrity: "3"
    }
    threat "Malicious recalibration" {
        id: "TH-CALIB"  kind: "Tampering"  level: 2
        rationale: "An aftermarket tool on the diagnostic connector misaligns the radar or changes the limits"
        applies_to: ["AS-CALIB"]  involves: ["LC-DIAG", "LC-COM"]
        security_integrity: "3"
    }
}

// ===========================================================================
// TRACEABILITY — each level derived from the one above, each element justified
// ===========================================================================

// Functional requirements derived from the stakeholder requirements
trace "FUN-ACC-001" derives "STK-REQ-001" {}
trace "FUN-ACC-002" derives "STK-REQ-001" {}
trace "FUN-ACC-003" derives "STK-REQ-002" {}
trace "FUN-ACC-004" derives "STK-REQ-002" {}
trace "FUN-ACC-005" derives "STK-REQ-003" {}
trace "FUN-ACC-006" derives "STK-REQ-003" {}
trace "FUN-ACC-007" derives "STK-REQ-003" {}
trace "FUN-ACC-008" derives "STK-REQ-004" {}
trace "FUN-ACC-009" derives "STK-REQ-004" {}
trace "FUN-ACC-010" derives "STK-REQ-011" {}
trace "FUN-ACC-011" derives "STK-REQ-011" {}
trace "FUN-ACC-012" derives "STK-REQ-002" {}
trace "FUN-ACC-013" derives "STK-REQ-005" {}
trace "FUN-ACC-014" derives "STK-REQ-006" {}
trace "FUN-ACC-015" derives "STK-REQ-006" {}
trace "FUN-ACC-016" derives "STK-REQ-007" {}
trace "FUN-ACC-017" derives "STK-REQ-007" {}
trace "FUN-ACC-018" derives "STK-REQ-005" {}
trace "FUN-ACC-019" derives "STK-REQ-009" {}
trace "FUN-ACC-020" derives "STK-REQ-009" {}
trace "FUN-ACC-021" derives "STK-REQ-010" {}
trace "FUN-ACC-008" derives "STK-REQ-008" { rationale: "The ISO 15622 envelope is part of the approval evidence" }

// Technical requirements derived from the functional ones
trace "TEC-ACC-001" derives "FUN-ACC-012" {}
trace "TEC-ACC-002" derives "FUN-ACC-010" {}
trace "TEC-ACC-003" derives "FUN-ACC-010" {}
trace "TEC-ACC-004" derives "FUN-ACC-010" {}
trace "TEC-ACC-005" derives "FUN-ACC-004" {}
trace "TEC-ACC-006" derives "FUN-ACC-008" {}
trace "TEC-ACC-007" derives "FUN-ACC-016" {}
trace "TEC-ACC-008" derives "FUN-ACC-021" {}
trace "TEC-ACC-009" derives "FUN-ACC-014" {}
trace "TEC-ACC-010" derives "FUN-ACC-019" {}
trace "TEC-ACC-011" derives "FUN-ACC-004" {}
trace "TEC-ACC-012" derives "FUN-ACC-016" {}
trace "TEC-ACC-013" derives "STK-REQ-008" {}
trace "TEC-ACC-014" derives "FUN-ACC-016" {}
trace "TEC-ACC-015" derives "FUN-ACC-006" {}

// The system functions realize the operational activities
trace "SF-RADAR" realizes "OAC-PERCEIVE" {}
trace "SF-CAMERA" realizes "OAC-PERCEIVE" {}
trace "SF-FUSE" realizes "OAC-PERCEIVE" {}
trace "SF-SPEED" realizes "OAC-REGULATE" {}
trace "SF-DIST" realizes "OAC-REGULATE" {}
trace "SF-HMI" realizes "OAC-INFORM" {}
trace "SF-TOR" realizes "OAC-INFORM" {}
trace "SF-DIAG" realizes "OAC-SERVICE" {}

// Stakeholder requirements satisfied by the functions that answer them
trace "SF-SPEED" satisfies "STK-REQ-001" {}
trace "SF-DIST" satisfies "STK-REQ-002" {}
trace "SF-GAP" satisfies "STK-REQ-002" {}
trace "SF-HOLD" satisfies "STK-REQ-003" {}
trace "SF-MODE" satisfies "STK-REQ-003" {}
trace "SF-LIMIT" satisfies "STK-REQ-004" {}
trace "SF-TOR" satisfies "STK-REQ-005" {}
trace "SF-OVR" satisfies "STK-REQ-006" {}
trace "SF-PLAUS" satisfies "STK-REQ-007" {}
trace "SF-LIMIT" satisfies "STK-REQ-008" { rationale: "The ISO 15622 envelope" }
trace "LC-SAFETY" satisfies "STK-REQ-008" { rationale: "The ASIL B safety mechanism of the safety case" }
trace "SC-ACC" satisfies "STK-REQ-008" { rationale: "The system as a whole is what is approved" }
trace "SF-DIAG" satisfies "STK-REQ-009" {}
trace "LC-COM" satisfies "STK-REQ-010" {}
trace "SF-CUTDET" satisfies "STK-REQ-011" {}

// Functional requirements satisfied by system functions
trace "SF-SPEED" satisfies "FUN-ACC-001" {}
trace "SF-SET" satisfies "FUN-ACC-001" {}
trace "SF-SPEED" satisfies "FUN-ACC-002" {}
trace "SF-SET" satisfies "FUN-ACC-003" {}
trace "SF-DIST" satisfies "FUN-ACC-004" {}
trace "SF-GAP" satisfies "FUN-ACC-004" {}
trace "SF-DIST" satisfies "FUN-ACC-005" {}
trace "SF-HOLD" satisfies "FUN-ACC-006" {}
trace "SF-MODE" satisfies "FUN-ACC-007" {}
trace "SF-LIMIT" satisfies "FUN-ACC-008" {}
trace "SF-LIMIT" satisfies "FUN-ACC-009" {}
trace "SF-CUTDET" satisfies "FUN-ACC-010" {}
trace "SF-DIST" satisfies "FUN-ACC-011" {}
trace "SF-SELECT" satisfies "FUN-ACC-012" {}
trace "SF-PATH" satisfies "FUN-ACC-012" {}
trace "SF-TOR" satisfies "FUN-ACC-013" {}
trace "SF-OVR" satisfies "FUN-ACC-014" {}
trace "SF-OVR" satisfies "FUN-ACC-015" {}
trace "SF-PLAUS" satisfies "FUN-ACC-016" {}
trace "SF-MODE" satisfies "FUN-ACC-017" {}
trace "SF-HMI" satisfies "FUN-ACC-018" {}
trace "SF-DIAG" satisfies "FUN-ACC-019" {}
trace "SF-DIAG" satisfies "FUN-ACC-020" {}
trace "SF-BRAKE" satisfies "FUN-ACC-021" {}
trace "SF-TORQUE" satisfies "FUN-ACC-021" {}

// Requirements satisfied by the components and the hardware
trace "LC-CTRL" satisfies "FUN-ACC-001" { rationale: "Speed loop" }
trace "LC-CTRL" satisfies "FUN-ACC-004" { rationale: "Constant time gap law, simulated" }
trace "LC-CTRL" satisfies "FUN-ACC-005" { rationale: "d0 = 4 m at standstill" }
trace "LC-ACT" satisfies "FUN-ACC-006" {}
trace "LC-MODE" satisfies "FUN-ACC-007" { rationale: "Standstill is left only on resume" }
trace "LC-SAFETY" satisfies "FUN-ACC-008" { rationale: "Independent envelope clamp" }
trace "LC-SAFETY" satisfies "FUN-ACC-009" {}
trace "LC-SITUATION" satisfies "FUN-ACC-010" {}
trace "LC-CAMERA" satisfies "FUN-ACC-010" {}
trace "LC-CTRL" satisfies "FUN-ACC-011" {}
trace "LC-FUSION" satisfies "FUN-ACC-012" {}
trace "LC-HMI-OUT" satisfies "FUN-ACC-013" {}
trace "LC-HMI-IN" satisfies "FUN-ACC-014" {}
trace "LC-MODE" satisfies "FUN-ACC-014" {}
trace "LC-HMI-IN" satisfies "FUN-ACC-015" {}
trace "LC-SAFETY" satisfies "FUN-ACC-016" {}
trace "LC-MODE" satisfies "FUN-ACC-017" {}
trace "LC-HMI-OUT" satisfies "FUN-ACC-018" {}
trace "LC-DIAG" satisfies "FUN-ACC-019" {}
trace "LC-DIAG" satisfies "FUN-ACC-020" {}
trace "LC-COM" satisfies "FUN-ACC-021" {}
trace "LC-EGO" satisfies "FUN-ACC-002" { rationale: "Speed estimate from four wheel speeds, 0.1 km/h resolution" }
trace "LC-EGO" satisfies "FUN-ACC-005" { rationale: "Standstill detected from the wheel pulses" }
trace "LC-RADAR" satisfies "TEC-ACC-001" {}
trace "LC-RADAR" satisfies "TEC-ACC-002" {}
trace "LC-CAMERA" satisfies "TEC-ACC-003" {}
trace "LC-FUSION" satisfies "TEC-ACC-004" {}
trace "LC-CTRL" satisfies "TEC-ACC-005" {}
trace "LC-SAFETY" satisfies "TEC-ACC-006" {}
trace "PN-ADAS" satisfies "TEC-ACC-006" { rationale: "Lockstep safety MCU separate from the application SoC" }
trace "LC-ACT" satisfies "TEC-ACC-007" {}
trace "LC-COM" satisfies "TEC-ACC-008" {}
trace "LC-HMI-IN" satisfies "TEC-ACC-009" {}
trace "LC-DIAG" satisfies "TEC-ACC-010" {}
trace "PN-ADAS" satisfies "TEC-ACC-011" {}
trace "LC-COM" satisfies "TEC-ACC-012" {}
trace "PN-ADAS" satisfies "TEC-ACC-013" {}
trace "PN-RADAR" satisfies "TEC-ACC-013" {}
trace "PN-CAMERA" satisfies "TEC-ACC-013" {}
trace "LC-CAMERA" satisfies "TEC-ACC-014" {}
trace "LC-ACT" satisfies "TEC-ACC-015" {}

// ===========================================================================
// VERIFICATION — each requirement has its case; the verdicts are computed
// ===========================================================================

// ---- By simulation of the control law (OpenModelica, physics/ACCPhysics.mo) -----

test_case "Following a braking vehicle" {
    id: "TC-SIM-FOLLOW"
    verifies: ["FUN-ACC-004", "FUN-ACC-008", "FUN-ACC-009", "FUN-ACC-001"]
    method: "analysis"
    description: "100 km/h at 1.8 s; the vehicle ahead brakes at 3 m/s2 to 50 km/h, then accelerates back to 90 km/h"
    simulate: { model: "ACCPhysics.FollowBraking" stop_time: 120 s sources: ["physics/ACCPhysics.mo"] }
    expects: [
        "FUN-ACC-004: min(acc.timeGap) >= 1.71 s",
        "FUN-ACC-008: max(acc.decel) <= 3.5",
        "FUN-ACC-008: min(acc.decel) >= -2.0",
        "FUN-ACC-009: max(acc.jerk) <= 2.5",
        "FUN-ACC-009: min(acc.jerk) >= -2.5",
        "FUN-ACC-001: max(ego.v) <= 130 km/h"
    ]
}

test_case "Cut-in 22 m ahead" {
    id: "TC-SIM-CUTIN"
    verifies: ["FUN-ACC-011", "FUN-ACC-008"]
    method: "analysis"
    description: "100 km/h at 1.8 s; a vehicle at 80 km/h cuts in 22 m ahead at 10 s"
    simulate: { model: "ACCPhysics.CutIn" stop_time: 40 s sources: ["physics/ACCPhysics.mo"] }
    expects: [
        "FUN-ACC-011: min(gap) >= 5 m",
        "FUN-ACC-011: at(30 s, acc.timeGap) >= 1.71 s",
        "FUN-ACC-008: max(acc.decel) <= 3.5"
    ]
}

test_case "Vehicle ahead brakes at 7 m/s2" {
    id: "TC-SIM-HARD"
    verifies: ["FUN-ACC-013", "FUN-ACC-008"]
    method: "analysis"
    description: "The braking needed exceeds the ACC's authority: the takeover request must be raised within 100 ms; the driver, reacting in 1 s, stops behind"
    simulate: { model: "ACCPhysics.HardBraking" stop_time: 15 s sources: ["physics/ACCPhysics.mo"] }
    expects: [
        "FUN-ACC-013: at(5.1 s, acc.takeoverRequest) >= 1",
        "FUN-ACC-008: max(acc.decel) <= 3.5",
        "FUN-ACC-013: min(gap) >= 2 m"
    ]
}

test_case "Stop and go in a jam" {
    id: "TC-SIM-STOPGO"
    verifies: ["FUN-ACC-005", "FUN-ACC-007"]
    method: "analysis"
    description: "50 km/h behind a vehicle that stops at 1.5 m/s2, stands 8 s and drives off at 1 m/s2"
    simulate: { model: "ACCPhysics.StopAndGo" stop_time: 50 s sources: ["physics/ACCPhysics.mo"] }
    expects: [
        "FUN-ACC-005: min(gap) >= 3.5 m",
        "FUN-ACC-005: at(21 s, gap) <= 6 m",
        "FUN-ACC-005: min(ego.v) >= 0 m/s",
        "FUN-ACC-005: at(21 s, ego.v) <= 0.01 m/s",
        "FUN-ACC-007: final(ego.v) >= 13 m/s"
    ]
}

// ---- By executing the generated mode machine (C, software in the loop) ----------

io_binding "ADAS safety MCU" {
    machine: "ACC operating modes"
    period: 10 ms
    watchdog: true
    inputs: [
        "LF-SWITCHES <- can 0x101",
        "LF-SELECT <- can 0x120",
        "LF-STANDSTILL <- can 0x130",
        "LF-BRAKEPEDAL <- gpio 4 rising",
        "LF-FAULTS <- gpio 7 rising",
        "main switch on <- gpio 3",
        "main switch off <- gpio 6",
        "set pressed above 30 km/h <- adc 0 > 2048",
        "target in path <- gpio 8",
        "target lost <- gpio 9",
        "vehicle stopped <- gpio 11",
        "resume pressed <- gpio 12",
        "pedal released <- gpio 5"
    ]
    outputs: ["Following -> gpio 10", "Standstill -> gpio 13", "Degraded -> can 0x300", "Cruising -> pwm 1 750", "Overridden -> uart 1 0x4F"]
}

test_case "Brake pedal override" {
    id: "TC-SIL-OVERRIDE"
    verifies: ["FUN-ACC-014"]
    method: "test"
    description: "Engage, follow, press the brake pedal, release and resume: the generated step() against the model"
    execute: {
        machine: "ACC operating modes"
        step: 10 ms
        stop_time: 1 s
        inputs: [
            "0 s: main switch on", "10 ms: LF-SWITCHES",
            "50 ms: set pressed above 30 km/h", "60 ms: LF-SWITCHES",
            "100 ms: target in path", "110 ms: LF-SELECT",
            "200 ms: LF-BRAKEPEDAL",
            "300 ms: pedal released", "310 ms: LF-SWITCHES"
        ]
    }
    expects: [
        "at(20 ms, \"ACC operating modes\".Standby) == 1",
        "at(150 ms, \"ACC operating modes\".Following) == 1",
        "at(250 ms, \"ACC operating modes\".Overridden) == 1",
        "final(\"ACC operating modes\".Standby) == 1",
        "max(\"ACC operating modes\".Degraded) == 0"
    ]
}

test_case "Standstill and drive-off on resume" {
    id: "TC-SIL-STOPGO"
    verifies: ["FUN-ACC-007", "FUN-ACC-006"]
    method: "test"
    description: "Follow into standstill: the mode stays Standstill until resume is pressed, never leaves it on its own"
    execute: {
        machine: "ACC operating modes"
        step: 10 ms
        stop_time: 1 s
        inputs: [
            "0 s: main switch on", "10 ms: LF-SWITCHES",
            "50 ms: set pressed above 30 km/h", "60 ms: LF-SWITCHES",
            "100 ms: target in path", "110 ms: LF-SELECT",
            "200 ms: vehicle stopped", "210 ms: LF-STANDSTILL",
            "300 ms: LF-SELECT", "400 ms: LF-SWITCHES",
            "600 ms: resume pressed", "610 ms: LF-SWITCHES"
        ]
    }
    expects: [
        "at(250 ms, \"ACC operating modes\".Standstill) == 1",
        "at(500 ms, \"ACC operating modes\".Standstill) == 1",
        "final(\"ACC operating modes\".Following) == 1"
    ]
}

test_case "Fault while following" {
    id: "TC-SIL-FAULT"
    verifies: ["FUN-ACC-016"]
    method: "test"
    description: "A fault while following reaches Degraded at once; the main switch off returns to Off: through the I/O glue of the safety MCU"
    execute: {
        machine: "ACC operating modes"
        binding: "ADAS safety MCU"
        stop_time: 500 ms
        inputs: [
            "0 s: main switch on", "10 ms: LF-SWITCHES",
            "30 ms: set pressed above 30 km/h", "40 ms: LF-SWITCHES",
            "60 ms: target in path", "70 ms: LF-SELECT",
            "150 ms: LF-FAULTS",
            "200 ms: main switch on = false", "200 ms: main switch off", "250 ms: LF-SWITCHES"
        ]
    }
    expects: [
        "at(100 ms, \"ACC operating modes\".Following) == 1",
        "at(160 ms, \"ACC operating modes\".Degraded) == 1",
        "final(\"ACC operating modes\".Off) == 1"
    ]
}

// ---- By analysis: the latency budgets of the chains -------------------------------

constraint "Following loop within budget" { id: "CST-FOLLOW" assert: sum("FC-FOLLOW", latency) <= "FC-FOLLOW".latency_budget }
constraint "Cut-in reaction within budget" { id: "CST-CUTIN" assert: sum("FC-CUTIN", latency) <= "FC-CUTIN".latency_budget }
constraint "Override within budget" { id: "CST-OVERRIDE" assert: sum("FC-OVERRIDE", latency) <= "FC-OVERRIDE".latency_budget }
constraint "Takeover request within budget" { id: "CST-TAKEOVER" assert: sum("FC-TAKEOVER", latency) <= "FC-TAKEOVER".latency_budget }
constraint "Fault reaction within budget" { id: "CST-FAULT" assert: sum("FC-FAULT", latency) <= "FC-FAULT".latency_budget }
constraint "Sensors and controller within the power budget" { id: "CST-POWER" assert: "PN-ADAS".power + "PN-RADAR".power + "PN-CAMERA".power <= 40 W }

test_case "End-to-end latencies" {
    id: "TC-ANA-LATENCY"
    verifies: ["FUN-ACC-010", "FUN-ACC-013", "FUN-ACC-014", "FUN-ACC-016"]
    method: "analysis"
    description: "The latencies of the functions summed along each chain against its budget"
    asserts: ["CST-CUTIN", "CST-TAKEOVER", "CST-OVERRIDE", "CST-FAULT", "CST-FOLLOW"]
}
test_case "Power budget" {
    id: "TC-ANA-POWER"
    verifies: ["TEC-ACC-013"]
    method: "analysis"
    asserts: ["CST-POWER"]
}

// ---- On the bench and in the vehicle: planned, verdicts recorded when run ----------

test_case "Speed accuracy on the test track" { id: "TC-VEH-SPEED" verifies: ["FUN-ACC-002"] method: "test" procedure: "Level road and 8 % slope, set speeds 50, 90, 130 km/h; log the speed for 5 min each" expected: "Within 2 km/h level, 5 km/h on the slope" }
test_case "Time gap settings" { id: "TC-INS-GAP" verifies: ["FUN-ACC-003"] method: "inspection" procedure: "Review the switch decoding and the controller parameters" }
test_case "Cut-in detection latency on the HIL bench" { id: "TC-HIL-CUTIN" verifies: ["FUN-ACC-010", "TEC-ACC-004"] method: "test" procedure: "Replay 200 recorded cut-ins; measure lane crossing to target switch" expected: "300 ms at most, every case" }
test_case "Target selection on curves" { id: "TC-VEH-CURVE" verifies: ["FUN-ACC-012", "TEC-ACC-001", "TEC-ACC-002"] method: "test" procedure: "Proving ground, radius 250 m to 1000 m, target in the ego and the adjacent lanes" }
test_case "Accelerator override" { id: "TC-HIL-ACCEL" verifies: ["FUN-ACC-015"] method: "test" procedure: "HIL: accelerator above the ACC demand while following a braking target" expected: "No brake request while the pedal prevails" }
test_case "Engagement conditions" { id: "TC-HIL-ENGAGE" verifies: ["FUN-ACC-017"] method: "test" procedure: "Each condition violated in turn: door, belt, ESC off, active fault" }
test_case "Cluster display" { id: "TC-DEM-HMI" verifies: ["FUN-ACC-018"] method: "demonstration" }
test_case "UDS fault memory" { id: "TC-HIL-UDS" verifies: ["FUN-ACC-019", "TEC-ACC-010"] method: "test" procedure: "Inject each fault, read with 0x19, clear with 0x14, power cycle" }
test_case "Workshop alignment" { id: "TC-DEM-CALIB" verifies: ["FUN-ACC-020"] method: "demonstration" procedure: "Static target, standard tester, stopwatch" expected: "Less than 20 minutes" }
test_case "SecOC rejection" { id: "TC-HIL-SECOC" verifies: ["FUN-ACC-021", "TEC-ACC-008"] method: "test" procedure: "Send requests with a wrong MAC, a replayed freshness value, a wrong counter" expected: "All rejected and reported" }
test_case "Standstill hold on a slope" { id: "TC-VEH-HOLD" verifies: ["FUN-ACC-006", "TEC-ACC-015"] method: "test" procedure: "12 % slope, standstill 200 s" expected: "No roll; park brake engaged at 180 s" }
test_case "Radar acceptance" { id: "TC-SUP-RADAR" verifies: ["TEC-ACC-001", "TEC-ACC-002"] method: "test" procedure: "Supplier test report against the radar specification" }
test_case "Camera acceptance" { id: "TC-SUP-CAMERA" verifies: ["TEC-ACC-003", "TEC-ACC-014"] method: "test" procedure: "Supplier test report; climatic chamber -40 °C to +85 °C; blindness cases" }
test_case "Timing analysis" { id: "TC-ANA-WCET" verifies: ["TEC-ACC-005", "TEC-ACC-011"] method: "analysis" procedure: "Static WCET analysis of the controller task; load measurement on the target under the worst-case scenario" }
test_case "Freedom from interference" { id: "TC-INS-FFI" verifies: ["TEC-ACC-006"] method: "inspection" procedure: "Dependent failure analysis of the controller and the monitor (ISO 26262-9)" }
test_case "E2E timeout at the ESC" { id: "TC-HIL-E2E" verifies: ["TEC-ACC-007"] method: "test" procedure: "Stop the requests, corrupt the CRC, freeze the counter" expected: "ESC releases within 30 ms" }
test_case "Brake pedal detection" { id: "TC-HIL-PEDAL" verifies: ["TEC-ACC-009"] method: "test" procedure: "Brake switch and pressure sensor stimulated separately and together" }
test_case "Bus load" { id: "TC-ANA-BUS" verifies: ["TEC-ACC-012"] method: "analysis" procedure: "Worst-case bus load of the chassis CAN FD with the ACC frames" }
test_case "Approval file" { id: "TC-INS-APPROVAL" verifies: ["STK-REQ-008"] method: "inspection" procedure: "ISO 15622 test report and the ISO 26262 safety case reviewed by the assessor" }

// ---- Validation of the stakeholder requirements by the system's verdicts -----------

test_case "Stakeholder acceptance drive" {
    id: "TC-VAL-DRIVE"
    verifies: ["STK-REQ-001", "STK-REQ-002", "STK-REQ-003", "STK-REQ-004", "STK-REQ-005", "STK-REQ-006", "STK-REQ-007", "STK-REQ-009", "STK-REQ-010", "STK-REQ-011"]
    method: "demonstration"
    procedure: "Panel of 12 drivers, 2000 km of motorway and urban congestion, questionnaire per stakeholder need"
}

// ---- Declared views ---------------------------------------------------------------

diagram "Perception to actuation" {
    view: lab
    description: "The following loop through the logical components"
    shows: ["LC-RADAR", "LC-CAMERA", "LC-FUSION", "LC-SITUATION", "LC-CTRL", "LC-SAFETY", "LC-ACT", "LC-COM"]
}
diagram "Driver in command" {
    view: lab
    description: "Switches, pedals, modes and the takeover request"
    shows: ["LC-HMI-IN", "LC-MODE", "LC-HMI-OUT", "LC-SAFETY", "LC-ACT"]
}
diagram "Around the ADAS controller" {
    view: pab
    shows: ["PN-ADAS"]
    context: true
}
