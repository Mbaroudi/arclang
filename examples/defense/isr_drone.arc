// Tactical ISR unmanned aircraft system: the air vehicle, its ground control
// station and the datalink, modelled through the five Arcadia levels with
// airworthiness-grade attributes. A textbook, unclassified example: the
// numbers are credible, none is taken from a real programme.
//
// What it shows of ArcLang: user-defined types with required attributes,
// typed quantities with units, requirements satisfied by components and
// verified by test cases, functional chains with latency budgets checked by
// constraints, a HARA whose levels pass the DO-178C, ISO 26262 and IEC 61508
// gates, a cybersecurity analysis on scales, modes with guarded transitions,
// scenarios with fragments, deployments, physical exchanges with a complete
// ICD, configuration items and named diagrams.

model ISR_UAS {
  name: "Tactical ISR Unmanned Aircraft System"
  description: "Medium-altitude, 24 h endurance ISR unmanned aircraft system: air vehicle, ground control station and line-of-sight datalink"
  version: "1.0"
}

// ---------------------------------------------------------------------------
// Reusable types: what every instance must state, what it inherits.
// ---------------------------------------------------------------------------

type "Airborne computer" {
  description: "Line-replaceable airborne computing unit on the 28 V bus"
  required: ["dal", "power", "mass"]
  voltage: 28 V
}

type "Ground equipment" {
  description: "Ruggedised ground equipment on shelter power"
  required: ["power"]
  voltage: 230 V
}

type "Flight-critical function" {
  description: "Function whose failure contributes to a hazardous condition: DAL B, bounded latency"
  required: ["latency"]
  dal: "DAL-B"
}

type "Mission function" {
  description: "Function of the mission system: DAL C, bounded latency"
  required: ["latency"]
  dal: "DAL-C"
}

// ---------------------------------------------------------------------------
// Requirements
// ---------------------------------------------------------------------------

requirements performance {
  requirement "REQ-PERF-001" {
    description: "The air vehicle shall sustain 24 h of flight with the EO/IR payload operating"
    priority: "High" category: "performance"
    verification_method: "demonstration"
  }
  requirement "REQ-PERF-002" {
    description: "The datalink shall sustain 10 Mbit/s of payload data at 150 km line of sight"
    priority: "High" category: "performance"
    verification_method: "test"
  }
  requirement "REQ-PERF-003" {
    description: "The navigation solution shall hold a 2 m circular error probable with GNSS and INS blended"
    priority: "High" category: "performance"
    verification_method: "test"
  }
  requirement "REQ-PERF-004" {
    description: "Video shall reach the payload operator within 500 ms of acquisition, sensor to screen"
    priority: "Medium" category: "performance"
    verification_method: "test"
  }
  requirement "REQ-PERF-005" {
    description: "The gimbal shall hold the line of sight within 0.5 mrad while the aircraft manoeuvres at up to 30 degrees of bank"
    priority: "Medium" category: "performance"
    verification_method: "test"
  }
}

requirements safety {
  requirement "REQ-SAF-001" {
    description: "On loss of the C2 link the air vehicle shall enter the lost-link mode and command the return-to-base route within 10 s"
    priority: "Critical" category: "safety"
    dal: "DAL-B" safety_level: "DAL-B"
    rationale: "A link loss over the area of operations must never leave the aircraft without a flight plan"
    verification_method: "test"
  }
  requirement "REQ-SAF-002" {
    description: "The air vehicle shall not cross the geofence; a route that would cross it shall be replaced by a hold within 500 ms"
    priority: "Critical" category: "safety"
    dal: "DAL-B" safety_level: "DAL-B"
    verification_method: "test"
  }
  requirement "REQ-SAF-003" {
    description: "No single failure of the flight control computer shall result in loss of control; the two channels shall be monitored and switched within 100 ms"
    priority: "Critical" category: "safety"
    dal: "DAL-B" safety_level: "DAL-B"
    verification_method: "analysis"
  }
  requirement "REQ-SAF-004" {
    description: "The air vehicle shall hold a minimum safe altitude above the terrain database along the route and the return-to-base route"
    priority: "Critical" category: "safety"
    dal: "DAL-B" safety_level: "DAL-B"
    verification_method: "test"
  }
  requirement "REQ-SAF-005" {
    description: "The pilot shall be able to command an emergency recovery to a pre-planned site from any flight mode"
    priority: "High" category: "safety"
    dal: "DAL-B" safety_level: "DAL-B"
    verification_method: "demonstration"
  }
}

requirements security {
  requirement "REQ-SEC-001" {
    description: "C2 messages shall be mutually authenticated and encrypted end to end; a message failing authentication shall be discarded and counted"
    priority: "Critical" category: "security"
    verification_method: "test"
  }
  requirement "REQ-SEC-002" {
    description: "The navigation function shall detect a GNSS solution inconsistent with the inertial solution and continue on inertial navigation alone"
    priority: "Critical" category: "security"
    dal: "DAL-B" safety_level: "DAL-B"
    verification_method: "test"
  }
  requirement "REQ-SEC-003" {
    description: "Under jamming the datalink shall hop frequencies and, when the C2 channel stays unusable, hand over to the lost-link behaviour"
    priority: "High" category: "security"
    verification_method: "test"
  }
  requirement "REQ-SEC-004" {
    description: "Payload data shall be encrypted in transit and at rest; the keys shall be zeroised on tamper detection or on a command from the pilot station"
    priority: "High" category: "security"
    verification_method: "test"
  }
}

requirements airworthiness {
  requirement "REQ-AW-001" {
    description: "The flight control software shall be developed to DO-178C design assurance level B"
    priority: "Critical" category: "airworthiness"
    dal: "DAL-B" safety_level: "DAL-B"
    verification_method: "inspection"
  }
  requirement "REQ-AW-002" {
    description: "The mission system software shall be developed to DO-178C design assurance level C and partitioned from the flight control software in time and space"
    priority: "Critical" category: "airworthiness"
    dal: "DAL-C" safety_level: "DAL-C"
    verification_method: "inspection"
  }
  requirement "REQ-AW-003" {
    description: "The air vehicle shall report its health, fuel state and link margin to the pilot station at 1 Hz"
    priority: "Medium" category: "airworthiness"
    verification_method: "test"
  }
}

// ---------------------------------------------------------------------------
// Operational analysis: who needs what from the ISR unit
// ---------------------------------------------------------------------------

operational_analysis "Operational Analysis" {
  actor "Mission commander" {
    id: "OA-ACT-MC"
    description: "Owns the sortie: accepts the tasking, orders the recovery"
  }
  actor "UAS pilot" {
    id: "OA-ACT-PILOT"
    description: "Flies the air vehicle from the ground control station"
  }
  actor "Payload operator" {
    id: "OA-ACT-PAYOP"
    description: "Steers the EO/IR sensor and marks what is seen"
  }
  actor "Air traffic control" {
    id: "OA-ACT-ATC"
    description: "Clears the air vehicle through the controlled airspace"
  }
  actor "Maintainer" {
    id: "OA-ACT-MAINT"
    description: "Declares the air vehicle serviceable and performs the turnaround"
  }
  entity "Tasking authority" {
    id: "OA-ENT-TASK"
    description: "The headquarters that tasks the ISR unit and consumes its products"
    activity "Issue tasking order" {
      id: "OA-TASK"
      description: "Names the area of interest, the time window and the product expected"
    }
  }
  entity "ISR unit" {
    id: "OA-ENT-UNIT"
    description: "The unit operating the ground control station and the air vehicles"
    activity "Plan mission" {
      id: "OA-PLAN"
      description: "Route, geofence, lost-link plan and sensor tasks"
    }
    activity "Fly route" {
      id: "OA-FLY"
      description: "Launch, transit, loiter and transit back under pilot supervision"
    }
    activity "Observe area of interest" {
      id: "OA-OBSERVE"
      description: "Point the sensor, hold the target, record"
    }
    activity "Disseminate intelligence product" {
      id: "OA-DISSEM"
      description: "Annotate frames and tracks and release them to the tasking authority"
    }
    activity "Recover aircraft" {
      id: "OA-RECOVER"
      description: "Return, land and hand over to the maintainer"
    }
  }
  operational_capability "Provide persistent ISR over an area of interest" {
    id: "OC-ISR"
    involves: ["OA-ENT-TASK", "OA-ENT-UNIT", "OA-ACT-MC", "OA-ACT-PAYOP", "OA-OBSERVE", "OA-DISSEM"]
    includes: ["OC-SAFE"]
  }
  operational_capability "Operate safely in shared airspace" {
    id: "OC-SAFE"
    involves: ["OA-ACT-PILOT", "OA-ACT-ATC", "OA-FLY", "OA-RECOVER"]
  }
  interaction "Tasking order" { from: "OA-TASK" to: "OA-PLAN" }
  interaction "Mission plan" { from: "OA-PLAN" to: "OA-FLY" }
  interaction "Airspace clearance" { from: "OA-ACT-ATC" to: "OA-FLY" }
  interaction "Flight command" { from: "OA-ACT-PILOT" to: "OA-FLY" }
  interaction "Sensor steering" { from: "OA-ACT-PAYOP" to: "OA-OBSERVE" }
  interaction "Sensor product" { from: "OA-OBSERVE" to: "OA-DISSEM" }
  interaction "Intelligence product" { from: "OA-DISSEM" to: "OA-ENT-TASK" }
  interaction "Recovery order" { from: "OA-ACT-MC" to: "OA-RECOVER" }
  interaction "Serviceability report" { from: "OA-ACT-MAINT" to: "OA-PLAN" }
  communication_means "Tactical network" { from: "OA-ENT-TASK" to: "OA-ENT-UNIT" }
  operational_process "ISR sortie" {
    id: "OP-SORTIE"
    description: "From the tasking order to the released product and the recovered aircraft"
    involves: ["OA-TASK", "Tasking order", "OA-PLAN", "Mission plan", "OA-FLY", "OA-OBSERVE", "Sensor product", "OA-DISSEM", "OA-RECOVER"]
  }
}

// ---------------------------------------------------------------------------
// System analysis: what the UAS does, with its timing
// ---------------------------------------------------------------------------

system_analysis "System Analysis" {
  actor "UAS pilot" { id: "SA-ACT-PILOT" }
  actor "Payload operator" { id: "SA-ACT-PAYOP" }
  actor "Air traffic control" { id: "SA-ACT-ATC" }
  actor "Tasking authority" { id: "SA-ACT-TASK" }
  actor "GNSS constellation" {
    id: "SA-ACT-GNSS"
    description: "External, untrusted: its signals can be jammed or spoofed"
  }

  mission "Tactical ISR support to the land force" {
    id: "MSN-ISR"
    description: "Give the land force a persistent, timely picture of an area of interest"
  }

  capability "Observe an area of interest" {
    id: "CAP-ISR"
    mission: "MSN-ISR"
    realizes: "OC-ISR"
    involves: ["SF-PTCMD", "SF-POINT", "SF-VIDEO", "SF-COMP", "SF-ENC", "SF-TX", "SF-DISP", "SF-EXPL", "FC-OBSERVE"]
    includes: ["CAP-NAV"]
  }
  capability "Fly safely and return without a link" {
    id: "CAP-SAFE"
    mission: "MSN-ISR"
    realizes: "OC-SAFE"
    involves: ["SF-LMON", "SF-RTB", "SF-GEO", "SF-HEALTH", "FC-LOSTLINK", "FC-GEOFENCE"]
  }
  capability "Navigate autonomously" {
    id: "CAP-NAV"
    mission: "MSN-ISR"
    involves: ["SF-NAV", "SF-SPOOF", "SF-ROUTE", "SF-FLY"]
  }

  // Flight-critical functions (DAL B)
  function "Acquire GNSS/INS fix" {
    id: "SF-NAV"
    is: "Flight-critical function"
    description: "Blends the GNSS fix and the inertial solution into one navigation solution at 100 Hz"
    latency: 20 ms
    period: 10 ms
    port in "gnss" port out "nav"
  }
  function "Monitor GNSS integrity" {
    id: "SF-SPOOF"
    is: "Flight-critical function"
    description: "Compares the GNSS fix with the inertial propagation; flags a spoofed or degraded fix"
    latency: 50 ms
    port in "nav" port out "integrity"
  }
  function "Control flight" {
    id: "SF-FLY"
    is: "Flight-critical function"
    description: "Inner loops: attitude, airspeed and altitude hold on the commanded route"
    latency: 20 ms
    period: 20 ms
    port in "nav" port in "route" port out "surface_cmd" port out "fcs_status"
  }
  function "Execute route" {
    id: "SF-ROUTE"
    is: "Flight-critical function"
    description: "Follows the active flight plan waypoint by waypoint; a geofence hold or a return route replaces it"
    latency: 100 ms
    port in "plan" port in "nav" port in "integrity" port in "geofence_cmd"
    port in "rtb" port out "route"
  }
  function "Enforce geofence" {
    id: "SF-GEO"
    is: "Flight-critical function"
    description: "Predicts the position 30 s ahead against the geofence volume; commands a hold before the boundary"
    latency: 50 ms
    port in "nav" port out "geofence_cmd"
  }
  function "Command return to base" {
    id: "SF-RTB"
    is: "Flight-critical function"
    description: "On lost link, selects the pre-planned return route and the recovery altitude"
    latency: 200 ms
    port in "link_state" port out "rtb_route"
  }

  // Mission system functions (DAL C)
  function "Monitor datalink" {
    id: "SF-LMON"
    is: "Mission function"
    description: "Declares the C2 link lost after 5 s without an authenticated frame"
    latency: 5000 ms
    port in "c2_rx" port out "link_state"
  }
  function "Authenticate and decrypt C2" {
    id: "SF-AUTH"
    is: "Mission function"
    description: "Verifies the authentication tag and the sequence number of every C2 frame"
    latency: 10 ms
    port in "rf_c2" port out "c2"
  }
  function "Issue flight commands" {
    id: "SF-CMD"
    is: "Mission function"
    description: "Turns the pilot's inputs into C2 messages"
    latency: 50 ms
    port in "pilot_input" port out "c2_tx"
  }
  function "Manage mission plan" {
    id: "SF-PLAN"
    is: "Mission function"
    description: "Validates and uploads the flight plan, the geofence and the lost-link plan"
    latency: 500 ms
    port in "tasking" port out "plan"
  }
  function "Steer sensor" {
    id: "SF-PTCMD"
    is: "Mission function"
    description: "Turns the payload operator's inputs into pointing commands"
    latency: 20 ms
    port in "operator_input" port out "point_cmd"
  }
  function "Stabilise and point sensor" {
    id: "SF-POINT"
    is: "Mission function"
    description: "Holds the line of sight on the commanded point through the aircraft's motion"
    latency: 50 ms
    port in "point_cmd" port out "los"
  }
  function "Acquire EO/IR video" {
    id: "SF-VIDEO"
    is: "Mission function"
    description: "30 Hz video from the daylight and the infrared channels"
    latency: 33 ms
    frequency: 30 Hz
    port in "los" port out "video_raw"
  }
  function "Compress video" {
    id: "SF-COMP"
    is: "Mission function"
    description: "Compresses the selected channel to the downlink rate with the frame metadata"
    latency: 120 ms
    port in "video_raw" port out "video"
  }
  function "Encrypt datalink frames" {
    id: "SF-ENC"
    is: "Mission function"
    description: "Encrypts and authenticates every frame leaving the air vehicle"
    latency: 10 ms
    port in "video" port out "video_enc"
  }
  function "Transmit over payload datalink" {
    id: "SF-TX"
    is: "Mission function"
    description: "Queues and transmits the frames over the 10 Mbit/s payload channel"
    latency: 60 ms
    port in "video_enc" port out "rf_pl"
  }
  function "Monitor health and report" {
    id: "SF-HEALTH"
    is: "Mission function"
    description: "Collects the built-in test results, fuel and link margin into a 1 Hz health report"
    latency: 1000 ms
    frequency: 1 Hz
    port in "status" port out "health"
  }

  // Ground functions
  function "Display video to operator" {
    id: "SF-DISP"
    description: "Decrypts, decodes and shows the video with its footprint on the map"
    dal: "DAL-D"
    latency: 40 ms
    port in "rf_pl" port out "display"
  }
  function "Record payload data" {
    id: "SF-STORE"
    description: "Records the compressed video and the frame metadata, encrypted at rest"
    dal: "DAL-D"
    latency: 100 ms
    port in "video"
  }
  function "Disseminate intelligence product" {
    id: "SF-EXPL"
    description: "Annotates frames and tracks and releases the product to the tasking authority"
    dal: "DAL-E"
    latency: 2000 ms
    port in "frames" port out "product"
  }

  functional_exchange "navigation solution" { from: "SF-NAV.nav" to: "SF-FLY.nav" }
  functional_exchange "navigation for route" { from: "SF-NAV.nav" to: "SF-ROUTE.nav" }
  functional_exchange "navigation for geofence" { from: "SF-NAV.nav" to: "SF-GEO.nav" }
  functional_exchange "navigation for integrity" { from: "SF-NAV.nav" to: "SF-SPOOF.nav" }
  functional_exchange "integrity flag" { from: "SF-SPOOF.integrity" to: "SF-ROUTE.integrity" }
  functional_exchange "geofence hold" { from: "SF-GEO.geofence_cmd" to: "SF-ROUTE.geofence_cmd" }
  functional_exchange "route command" { from: "SF-ROUTE.route" to: "SF-FLY.route" }
  functional_exchange "flight control status" { from: "SF-FLY.fcs_status" to: "SF-HEALTH.status" }
  functional_exchange "link state" { from: "SF-LMON.link_state" to: "SF-RTB.link_state" }
  functional_exchange "return route" { from: "SF-RTB.rtb_route" to: "SF-ROUTE.rtb" }
  functional_exchange "authenticated C2" { from: "SF-AUTH.c2" to: "SF-LMON.c2_rx" }
  functional_exchange "C2 uplink" { from: "SF-CMD.c2_tx" to: "SF-AUTH.rf_c2" }
  functional_exchange "flight plan" { from: "SF-PLAN.plan" to: "SF-ROUTE.plan" }
  functional_exchange "pointing command" { from: "SF-PTCMD.point_cmd" to: "SF-POINT.point_cmd" }
  functional_exchange "line of sight" { from: "SF-POINT.los" to: "SF-VIDEO.los" }
  functional_exchange "raw video" { from: "SF-VIDEO.video_raw" to: "SF-COMP.video_raw" }
  functional_exchange "compressed video" { from: "SF-COMP.video" to: "SF-ENC.video" }
  functional_exchange "video to record" { from: "SF-COMP.video" to: "SF-STORE.video" }
  functional_exchange "encrypted video" { from: "SF-ENC.video_enc" to: "SF-TX.video_enc" }
  functional_exchange "payload downlink" { from: "SF-TX.rf_pl" to: "SF-DISP.rf_pl" }
  functional_exchange "displayed frames" { from: "SF-DISP.display" to: "SF-EXPL.frames" }

  functional_chain "Lost-link reaction" {
    id: "FC-LOSTLINK"
    description: "From the last authenticated C2 frame to the first surface command of the return route"
    capability: "CAP-SAFE"
    involves: ["SF-LMON", "SF-RTB", "SF-ROUTE", "SF-FLY"]
    latency_budget: 10 s
  }
  functional_chain "Target observation" {
    id: "FC-OBSERVE"
    description: "From the operator's steering input to the frame on the operator's screen"
    capability: "CAP-ISR"
    involves: ["SF-PTCMD", "SF-POINT", "SF-VIDEO", "SF-COMP", "SF-ENC", "SF-TX", "SF-DISP"]
    latency_budget: 500 ms
  }
  functional_chain "Geofence hold" {
    id: "FC-GEOFENCE"
    description: "From the navigation fix to the hold being flown when the boundary is approached"
    capability: "CAP-SAFE"
    involves: ["SF-NAV", "SF-GEO", "SF-ROUTE", "SF-FLY"]
    latency_budget: 500 ms
    control or "Boundary approached?"
    sequence "SF-GEO" -> "Boundary approached?"
    sequence "Boundary approached?" -> "SF-ROUTE" { condition: "predicted position within 30 s of the boundary" }
  }
}

// ---------------------------------------------------------------------------
// Logical architecture: the air vehicle and the ground control station
// ---------------------------------------------------------------------------

logical_architecture "Logical Architecture" {
  component "Air vehicle" {
    id: "LC-AV"
    description: "The unmanned aircraft: flight-critical partition (DAL B) and mission partition (DAL C)"

    component "Flight control" {
      id: "LC-FCS"
      dal: "DAL-B" safety_level: "DAL-B"
      description: "Dual-channel flight control: inner loops, route following, geofence and lost-link return"
      port in "nav" port in "route" port in "link_state" port out "fcs_health"
      function "Stabilise and control flight" { id: "LF-FLY" realizes: "SF-FLY" latency: 20 ms }
      function "Follow route" { id: "LF-ROUTE" realizes: "SF-ROUTE" latency: 100 ms }
      function "Enforce geofence" { id: "LF-GEO" realizes: "SF-GEO" latency: 50 ms }
      function "Execute lost-link return" { id: "LF-RTB" realizes: "SF-RTB" latency: 200 ms }
    }
    component "Navigation" {
      id: "LC-NAV"
      dal: "DAL-B" safety_level: "DAL-B"
      description: "GNSS/INS blending with integrity monitoring"
      port out "nav"
      function "Blend GNSS and inertial solutions" { id: "LF-NAV" realizes: "SF-NAV" latency: 20 ms }
      function "Detect GNSS spoofing" { id: "LF-SPOOF" realizes: "SF-SPOOF" latency: 50 ms }
    }
    component "Mission management" {
      id: "LC-MMS"
      dal: "DAL-C" safety_level: "DAL-C"
      description: "Holds the active plan, the geofence and the lost-link plan; routes the C2 messages"
      port in "c2" port out "route"
      function "Manage flight plan" { id: "LF-PLAN" realizes: "SF-PLAN" latency: 500 ms }
    }
    component "EO/IR sensor" {
      id: "LC-EOIR"
      dal: "DAL-C" safety_level: "DAL-C"
      description: "Stabilised gimbal with daylight and infrared channels"
      port in "point_cmd" port out "video_raw"
      function "Stabilise and point" { id: "LF-POINT" realizes: "SF-POINT" latency: 50 ms }
      function "Acquire video" { id: "LF-VIDEO" realizes: "SF-VIDEO" latency: 33 ms }
    }
    component "Payload management" {
      id: "LC-PLM"
      dal: "DAL-C" safety_level: "DAL-C"
      description: "Compresses, records and forwards the sensor video; routes the operator's commands to the gimbal"
      port in "cmd" port in "video_raw" port out "point_cmd" port out "video"
      function "Compress video" { id: "LF-COMP" realizes: "SF-COMP" latency: 120 ms }
      function "Record payload data" { id: "LF-STORE" realizes: "SF-STORE" latency: 100 ms }
      function "Route payload commands" { id: "LF-PCTL" realizes: "SF-PTCMD" latency: 20 ms }
    }
    component "Health monitoring" {
      id: "LC-HMS"
      dal: "DAL-C" safety_level: "DAL-C"
      port in "fcs_health" port out "health"
      function "Build health report" { id: "LF-HEALTH" realizes: "SF-HEALTH" latency: 1000 ms }
    }
    component "Airborne data terminal" {
      id: "LC-ADT"
      dal: "DAL-C" safety_level: "DAL-C"
      description: "Line-of-sight datalink: C2 channel, payload channel and the crypto module between them"
      component "C2 channel" {
        id: "LC-ADT-C2"
        dal: "DAL-C" safety_level: "DAL-C"
        description: "Low-rate, frequency-hopping, authenticated command and control channel"
        port in "rf" port in "health" port out "c2" port out "payload_cmd"
        port out "link_state"
        function "Receive and authenticate C2" { id: "LF-C2RX" realizes: "SF-AUTH" latency: 10 ms }
        function "Monitor link" { id: "LF-LMON" realizes: "SF-LMON" latency: 5000 ms }
      }
      component "Payload channel" {
        id: "LC-ADT-PL"
        dal: "DAL-C" safety_level: "DAL-C"
        description: "10 Mbit/s payload downlink"
        port in "video" port out "rf"
        function "Transmit frames" { id: "LF-TX" realizes: "SF-TX" latency: 60 ms }
      }
      component "Crypto module" {
        id: "LC-CRYPTO"
        dal: "DAL-C" safety_level: "DAL-C"
        description: "Encrypts the payload frames, holds the keys, zeroises on command or tamper"
        port in "plain" port out "cipher"
        function "Encrypt frames" { id: "LF-ENC" realizes: "SF-ENC" latency: 10 ms }
      }
    }
  }

  component "Ground control station" {
    id: "LC-GCS"
    description: "Shelter with the pilot, payload, planning and exploitation stations and the ground data terminal"

    component "Pilot station" {
      id: "LC-PILOT-ST"
      dal: "DAL-C" safety_level: "DAL-C"
      port in "health" port out "cmd"
      function "Command flight" { id: "LF-PILOT" realizes: "SF-CMD" latency: 50 ms }
    }
    component "Payload station" {
      id: "LC-PAYLOAD-ST"
      dal: "DAL-D" safety_level: "DAL-D"
      port in "video" port out "cmd" port out "product"
      function "Steer sensor" { id: "LF-PTCMD" realizes: "SF-PTCMD" latency: 20 ms }
      function "Display video" { id: "LF-DISP" realizes: "SF-DISP" latency: 40 ms }
    }
    component "Mission planning" {
      id: "LC-MP"
      dal: "DAL-D" safety_level: "DAL-D"
      port out "plan"
      function "Plan mission" { id: "LF-MPLAN" realizes: "SF-PLAN" latency: 500 ms }
    }
    component "Exploitation" {
      id: "LC-EXPL"
      dal: "DAL-E" safety_level: "DAL-E"
      port in "product"
      function "Produce intelligence product" { id: "LF-EXPL" realizes: "SF-EXPL" latency: 2000 ms }
    }
    component "Ground data terminal" {
      id: "LC-GDT"
      dal: "DAL-C" safety_level: "DAL-C"
      description: "Tracking antenna, C2 transmitter and payload receiver"
      port in "cmd" port in "pcmd" port in "plan" port in "pl_rf"
      port out "c2_rf" port out "video" port out "health"
      function "Manage ground link" { id: "LF-GLINK" realizes: "SF-LMON" latency: 5000 ms }
    }
  }

  // Air vehicle internal exchanges
  component_exchange "NavSolution" { id: "CE-NAV" from_port: "LC-NAV.nav" to_port: "LC-FCS.nav" frequency: 100 Hz }
  component_exchange "RouteCommand" { id: "CE-ROUTE" from_port: "LC-MMS.route" to_port: "LC-FCS.route" frequency: 10 Hz }
  component_exchange "LinkState" { id: "CE-LINKSTATE" from_port: "LC-ADT-C2.link_state" to_port: "LC-FCS.link_state" frequency: 10 Hz }
  component_exchange "C2Command" { id: "CE-C2-CMD" from_port: "LC-ADT-C2.c2" to_port: "LC-MMS.c2" }
  component_exchange "PayloadCommand" { id: "CE-PAYLOAD-CMD" from_port: "LC-ADT-C2.payload_cmd" to_port: "LC-PLM.cmd" }
  component_exchange "PointingCommand" { id: "CE-POINT" from_port: "LC-PLM.point_cmd" to_port: "LC-EOIR.point_cmd" }
  component_exchange "RawVideo" { id: "CE-RAW-VIDEO" from_port: "LC-EOIR.video_raw" to_port: "LC-PLM.video_raw" bandwidth: 1.5 Gbps }
  component_exchange "CompressedVideo" { id: "CE-VIDEO" from_port: "LC-PLM.video" to_port: "LC-CRYPTO.plain" bandwidth: 8 Mbps }
  component_exchange "EncryptedVideo" { id: "CE-VIDEO-ENC" from_port: "LC-CRYPTO.cipher" to_port: "LC-ADT-PL.video" bandwidth: 8 Mbps }
  component_exchange "FlightControlHealth" { id: "CE-FCS-HEALTH" from_port: "LC-FCS.fcs_health" to_port: "LC-HMS.fcs_health" frequency: 1 Hz }
  component_exchange "HealthReport" { id: "CE-HEALTH" from_port: "LC-HMS.health" to_port: "LC-ADT-C2.health" frequency: 1 Hz }

  // Datalink
  component_exchange "C2Uplink" { id: "CE-RF-C2" from_port: "LC-GDT.c2_rf" to_port: "LC-ADT-C2.rf" protocol: "Encrypted frequency-hopping C2 waveform" bandwidth: 200 kbps }
  component_exchange "PayloadDownlink" { id: "CE-RF-PL" from_port: "LC-ADT-PL.rf" to_port: "LC-GDT.pl_rf" protocol: "Encrypted payload waveform" bandwidth: 10 Mbps }

  // Ground control station internal exchanges
  component_exchange "PilotCommand" { id: "CE-PILOT-CMD" from_port: "LC-PILOT-ST.cmd" to_port: "LC-GDT.cmd" }
  component_exchange "OperatorCommand" { id: "CE-PAYOP-CMD" from_port: "LC-PAYLOAD-ST.cmd" to_port: "LC-GDT.pcmd" }
  component_exchange "PlanUpload" { id: "CE-PLAN" from_port: "LC-MP.plan" to_port: "LC-GDT.plan" }
  component_exchange "VideoToOperator" { id: "CE-GCS-VIDEO" from_port: "LC-GDT.video" to_port: "LC-PAYLOAD-ST.video" bandwidth: 8 Mbps }
  component_exchange "HealthToPilot" { id: "CE-GCS-HEALTH" from_port: "LC-GDT.health" to_port: "LC-PILOT-ST.health" frequency: 1 Hz }
  component_exchange "AnnotatedProduct" { id: "CE-PRODUCT" from_port: "LC-PAYLOAD-ST.product" to_port: "LC-EXPL.product" }

  interface "C2 message interface" {
    id: "IF-C2"
    from: "LC-GDT"
    to: "LC-ADT-C2"
    description: "Authenticated, sequence-numbered command and control messages over the encrypted C2 waveform (generic STANAG-style message set)"
    bandwidth: 200 kbps
  }
  interface "Payload data interface" {
    id: "IF-PAYLOAD"
    from: "LC-ADT-PL"
    to: "LC-GDT"
    description: "Compressed, encrypted sensor frames with their metadata, motion imagery transport over the encrypted payload waveform"
    bandwidth: 10 Mbps
  }
  interface "Flight control bus interface" {
    id: "IF-FCS"
    from: "LC-MMS"
    to: "LC-FCS"
    description: "Route commands from the mission partition to the flight-critical partition over the ARINC 825 bus; validated at the boundary, 10 Hz"
  }

  capability_realization "Observe an area of interest" {
    id: "CR-ISR"
    realizes: "CAP-ISR"
    involves: ["LC-PAYLOAD-ST", "LC-GDT", "LC-ADT-PL", "LC-CRYPTO", "LC-PLM", "LC-EOIR", "LC-EXPL"]
  }
  capability_realization "Fly safely and return without a link" {
    id: "CR-SAFE"
    realizes: "CAP-SAFE"
    involves: ["LC-GDT", "LC-ADT-C2", "LC-FCS", "LC-NAV", "LC-HMS", "LC-PILOT-ST"]
  }
}

// ---------------------------------------------------------------------------
// Physical architecture: airborne LRUs, ground equipment and the links
// ---------------------------------------------------------------------------

physical_architecture "Physical Architecture" {
  // Air vehicle
  node "Flight control computer" {
    id: "PN-FCC"
    is: "Airborne computer"
    type: "Airborne LRU"
    description: "Dual-channel, command/monitor flight control computer"
    dal: "DAL-B" safety_level: "DAL-B"
    redundancy: "Dual channel, command/monitor, 100 ms switchover"
    processor: "Dual lockstep PowerPC class, 1 GHz"
    ram: 512 MB
    power: 35 W
    mass: 1.8 kg
    port "J1" { connector: "MIL-DTL-38999" }
    port "J2" { connector: "MIL-DTL-38999" }
    deploys "LC-FCS"
    behavior_component "Flight control software" {
      id: "BC-FCS-SW"
      dal: "DAL-B"
      description: "DO-178C DAL B flight control software, partitioned from the mission software"
      allocated_component: "LC-FCS"
      function "Run flight control loops" { id: "PF-FLY" realizes: "LF-FLY" latency: 20 ms period: 20 ms }
      function "Run lost-link return" { id: "PF-RTB" realizes: "LF-RTB" latency: 200 ms }
    }
  }
  node "GNSS/INS unit" {
    id: "PN-GNSS-INS"
    is: "Airborne computer"
    type: "Airborne LRU"
    description: "Multi-constellation GNSS receiver with a tactical-grade inertial measurement unit"
    dal: "DAL-B" safety_level: "DAL-B"
    power: 12 W
    mass: 0.9 kg
    port "J1" { connector: "MIL-DTL-38999" }
    deploys "LC-NAV"
  }
  node "Mission computer" {
    id: "PN-MC"
    is: "Airborne computer"
    type: "Airborne LRU"
    description: "Mission partition host: plan management, payload management, health monitoring"
    dal: "DAL-C" safety_level: "DAL-C"
    processor: "Multicore ARM class, 1.5 GHz, partitioned RTOS"
    ram: 8 GB
    storage: 1000 GB
    power: 60 W
    mass: 3.2 kg
    port "J1" { connector: "MIL-DTL-38999" }
    port "J2" { connector: "MIL-DTL-38999" }
    port "J3" { connector: "MIL-DTL-38999" }
    deploys "LC-MMS"
    deploys "LC-PLM"
    deploys "LC-HMS"
    behavior_component "Mission software" {
      id: "BC-MC-SW"
      dal: "DAL-C"
      description: "DO-178C DAL C mission software"
      allocated_component: "LC-MMS"
      function "Manage the active plan" { id: "PF-PLAN" realizes: "LF-PLAN" latency: 500 ms }
    }
  }
  node "Airborne radio" {
    id: "PN-RADIO"
    is: "Airborne computer"
    type: "Airborne LRU"
    description: "Line-of-sight data terminal: C2 transceiver, payload transmitter, embedded crypto"
    dal: "DAL-C" safety_level: "DAL-C"
    power: 45 W
    mass: 2.4 kg
    port "J1" { connector: "MIL-DTL-38999" }
    port "RF1" { connector: "TNC" }
    port "RF2" { connector: "TNC" }
    deploys "LC-ADT-C2"
    deploys "LC-ADT-PL"
    deploys "LC-CRYPTO"
  }
  node "EO/IR gimbal" {
    id: "PN-GIMBAL"
    is: "Airborne computer"
    type: "Airborne LRU"
    description: "Four-axis stabilised turret with daylight and mid-wave infrared channels"
    dal: "DAL-C" safety_level: "DAL-C"
    power: 90 W
    mass: 11.5 kg
    port "J1" { connector: "MIL-DTL-38999" }
    deploys "LC-EOIR"
  }
  node "Power distribution unit" {
    id: "PN-PDU"
    type: "Airborne LRU"
    description: "28 V distribution from the engine generator and the backup battery"
    voltage: 28 V
    power: 400 W
    mass: 2.0 kg
    hardware_component "Backup battery" { id: "HW-BATT" description: "20 min of avionics power without the generator" voltage: 28 V mass: 3.5 kg }
  }

  // Ground control station
  node "Ground data terminal" {
    id: "PN-GDT"
    is: "Ground equipment"
    type: "Ground equipment"
    description: "Tracking directional antenna on a mast, C2 transceiver and payload receiver"
    power: 350 W
    mass: 45 kg
    port "RF1" { connector: "N" }
    port "RF2" { connector: "N" }
    port "ETH1" { connector: "RJ45 ruggedised" }
    deploys "LC-GDT"
  }
  node "Pilot workstation" {
    id: "PN-PILOT-WS"
    is: "Ground equipment"
    type: "Ground equipment"
    description: "Ruggedised workstation with flight controls and two displays"
    power: 250 W
    deploys "LC-PILOT-ST"
  }
  node "Payload workstation" {
    id: "PN-PAYLOAD-WS"
    is: "Ground equipment"
    type: "Ground equipment"
    description: "Ruggedised workstation with hand controller and video displays"
    power: 300 W
    deploys "LC-PAYLOAD-ST"
  }
  node "Planning and exploitation workstation" {
    id: "PN-MP-WS"
    is: "Ground equipment"
    type: "Ground equipment"
    description: "Ruggedised workstation for mission planning and product exploitation"
    power: 250 W
    storage: 4000 GB
    deploys "LC-MP"
    deploys "LC-EXPL"
  }

  // Airborne links
  physical_link "Avionics bus" {
    id: "PL-AVBUS"
    from: "PN-MC.J1"
    to: "PN-FCC.J1"
    protocol: "ARINC 825 (CAN)"
    bandwidth: 1 Mbps
    latency: 2 ms
  }
  physical_link "Navigation bus" {
    id: "PL-NAV"
    from: "PN-GNSS-INS.J1"
    to: "PN-FCC.J2"
    protocol: "ARINC 429"
    bandwidth: 100 kbps
    latency: 1 ms
  }
  physical_link "Radio to flight control" {
    id: "PL-RADIO-FCC"
    from: "PN-RADIO.J1"
    to: "PN-FCC.J2"
    protocol: "ARINC 825 (CAN)"
    bandwidth: 1 Mbps
    latency: 2 ms
  }
  physical_link "Mission Ethernet" {
    id: "PL-RADIO-MC"
    from: "PN-RADIO.J1"
    to: "PN-MC.J2"
    protocol: "Ethernet 1000BASE-T"
    bandwidth: 1 Gbps
    latency: 1 ms
  }
  physical_link "Gimbal Ethernet" {
    id: "PL-GIMBAL"
    from: "PN-GIMBAL.J1"
    to: "PN-MC.J3"
    protocol: "Ethernet 1000BASE-T"
    bandwidth: 1 Gbps
    latency: 1 ms
  }
  physical_link "28 V feed to flight control computer" {
    id: "PL-PWR-FCC"
    from: "PN-PDU"
    to: "PN-FCC"
    protocol: "28 V DC, dual feed"
  }
  physical_link "28 V feed to mission computer" {
    id: "PL-PWR-MC"
    from: "PN-PDU"
    to: "PN-MC"
    protocol: "28 V DC"
  }

  // RF datalink
  physical_link "C2 datalink" {
    id: "PL-RF-C2"
    from: "PN-GDT.RF1"
    to: "PN-RADIO.RF1"
    description: "Line-of-sight C2 channel, frequency hopping, 150 km"
    protocol: "Encrypted frequency-hopping C2 waveform"
    bandwidth: 200 kbps
    latency: 50 ms
    range: 150 km
  }
  physical_link "Payload datalink" {
    id: "PL-RF-PL"
    from: "PN-RADIO.RF2"
    to: "PN-GDT.RF2"
    description: "Line-of-sight payload downlink, 150 km"
    protocol: "Encrypted payload waveform"
    bandwidth: 10 Mbps
    latency: 40 ms
    range: 150 km
  }

  // Ground control station LAN
  physical_link "Shelter LAN, pilot" { id: "PL-LAN-PILOT" from: "PN-PILOT-WS" to: "PN-GDT.ETH1" protocol: "Ethernet 1000BASE-T" bandwidth: 1 Gbps latency: 1 ms }
  physical_link "Shelter LAN, payload" { id: "PL-LAN-PAYLOAD" from: "PN-PAYLOAD-WS" to: "PN-GDT.ETH1" protocol: "Ethernet 1000BASE-T" bandwidth: 1 Gbps latency: 1 ms }
  physical_link "Shelter LAN, planning" { id: "PL-LAN-MP" from: "PN-MP-WS" to: "PN-GDT.ETH1" protocol: "Ethernet 1000BASE-T" bandwidth: 1 Gbps latency: 1 ms }
  physical_link "Shelter LAN, exploitation" { id: "PL-LAN-EXPL" from: "PN-PAYLOAD-WS" to: "PN-MP-WS" protocol: "Ethernet 1000BASE-T" bandwidth: 1 Gbps latency: 1 ms }

  // Interface control: every message crossing a link
  physical_exchange "C2 uplink messages" {
    id: "PX-C2-UPLINK"
    from: "PN-GDT"
    to: "PN-RADIO"
    via: "PL-RF-C2"
    message_type: "C2Message"
    frequency: "10 Hz"
    size: 256 B
    bandwidth: 20 kbps
  }
  physical_exchange "Health downlink" {
    id: "PX-HEALTH-DOWNLINK"
    from: "PN-RADIO"
    to: "PN-GDT"
    via: "PL-RF-C2"
    message_type: "HealthReport"
    frequency: "1 Hz"
    size: 512 B
    bandwidth: 4 kbps
  }
  physical_exchange "Video downlink" {
    id: "PX-VIDEO-DOWNLINK"
    from: "PN-RADIO"
    to: "PN-GDT"
    via: "PL-RF-PL"
    message_type: "SensorFrame"
    frequency: "30 Hz"
    bandwidth: 8 Mbps
  }
  physical_exchange "Navigation solution" {
    id: "PX-NAV"
    from: "PN-GNSS-INS"
    to: "PN-FCC"
    via: "PL-NAV"
    message_type: "NavigationSolution"
    frequency: "100 Hz"
    size: 64 B
  }
  physical_exchange "Route commands" {
    id: "PX-ROUTE"
    from: "PN-MC"
    to: "PN-FCC"
    via: "PL-AVBUS"
    message_type: "RouteCommand"
    frequency: "10 Hz"
    size: 128 B
  }
  physical_exchange "Link state" {
    id: "PX-LINKSTATE"
    from: "PN-RADIO"
    to: "PN-FCC"
    via: "PL-RADIO-FCC"
    message_type: "LinkState"
    frequency: "10 Hz"
    size: 16 B
  }

  physical_path "Video path, sensor to operator" {
    id: "PP-VIDEO"
    description: "The links a frame crosses from the gimbal to the payload operator's screen"
    involves: ["Gimbal Ethernet", "Mission Ethernet", "Payload datalink", "Shelter LAN, payload"]
  }
}

// ---------------------------------------------------------------------------
// Configuration items
// ---------------------------------------------------------------------------

epbs "ISR UAS configuration" {
  system "Tactical ISR UAS" {
    id: "CI-UAS"
    subsystem "Air vehicle" {
      id: "CI-AV"
      item "Flight control computer" { id: "CI-FCC" part_number: "FCC-2100-B" supplier: "Avionics supplier (generic)" mass: 1.8 kg }
      item "Flight control software" { id: "CI-FCS-SW" part_number: "FCS-SW-4.3" supplier: "Prime contractor" }
      item "Mission computer" { id: "CI-MC" part_number: "MC-7000" supplier: "Avionics supplier (generic)" mass: 3.2 kg }
      item "Mission software" { id: "CI-MC-SW" part_number: "MSN-SW-2.1" supplier: "Prime contractor" }
      item "GNSS/INS unit" { id: "CI-GNSS-INS" part_number: "NAV-310" supplier: "Navigation supplier (generic)" mass: 0.9 kg }
      item "EO/IR gimbal" { id: "CI-GIMBAL" part_number: "EOIR-15" supplier: "Sensor supplier (generic)" mass: 11.5 kg }
      item "Power distribution unit" { id: "CI-PDU" part_number: "PDU-28-400" supplier: "Electrical supplier (generic)" mass: 2.0 kg }
    }
    subsystem "Datalink" {
      id: "CI-DL"
      item "Airborne radio" { id: "CI-RADIO" part_number: "ADT-150" supplier: "Datalink supplier (generic)" mass: 2.4 kg }
      item "Ground data terminal" { id: "CI-GDT" part_number: "GDT-150" supplier: "Datalink supplier (generic)" mass: 45 kg }
    }
    subsystem "Ground control station" {
      id: "CI-GCS"
      item "Pilot workstation" { id: "CI-PILOT-WS" part_number: "WS-R-2" supplier: "Rugged computing supplier (generic)" }
      item "Payload workstation" { id: "CI-PAYLOAD-WS" part_number: "WS-R-2" supplier: "Rugged computing supplier (generic)" }
      item "Planning and exploitation workstation" { id: "CI-MP-WS" part_number: "WS-R-3" supplier: "Rugged computing supplier (generic)" }
    }
  }
}

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

enumeration "FlightMode" {
  id: "EN-FLIGHTMODE"
  values: ["Preflight", "Takeoff", "Mission", "LostLink", "ReturnToBase", "Landing", "Emergency"]
}
enumeration "LinkStateValue" {
  id: "EN-LINKSTATE"
  values: ["Idle", "Acquiring", "Connected", "Degraded", "Lost"]
}
enumeration "ClassificationMarking" {
  id: "EN-MARKING"
  description: "Marking carried by every product; a plain enumeration, the handling rules are outside the model"
  values: ["Unclassified", "Restricted", "Confidential", "Secret"]
}
enumeration "SensorChannel" {
  id: "EN-CHANNEL"
  values: ["Daylight", "MidWaveInfrared"]
}

data_type "Latitude" { id: "DT-LAT" base: "Real" description: "Degrees, WGS 84" }
data_type "Longitude" { id: "DT-LON" base: "Real" description: "Degrees, WGS 84" }
data_type "Altitude" { id: "DT-ALT" base: "Real" unit: m description: "Metres above mean sea level" }
data_type "GroundSpeed" { id: "DT-SPEED" base: "Real" unit: "m/s" }
data_type "Duration" { id: "DT-DURATION" base: "Integer" unit: ms }
data_type "Timestamp" { id: "DT-TIME" base: "Integer" unit: ms description: "Milliseconds since the mission epoch" }
data_type "Identifier" { id: "DT-ID" base: "String" }
data_type "Bytes" { id: "DT-BYTES" base: "Binary" }

class "Waypoint" {
  id: "CL-WP"
  latitude: "Latitude"
  longitude: "Longitude"
  altitude: "Altitude"
  speed: "GroundSpeed"
  loiter_time: "Duration"
}
class "Geofence" {
  id: "CL-GEOFENCE"
  vertices: "Waypoint[]"
  floor: "Altitude"
  ceiling: "Altitude"
}
class "FlightPlan" {
  id: "CL-PLAN"
  plan_id: "Identifier"
  waypoints: "Waypoint[]"
  return_route: "Waypoint[]"
  geofence: "Geofence"
  marking: "ClassificationMarking"
}
class "NavigationSolution" {
  id: "CL-NAV"
  timestamp: "Timestamp"
  position: "Waypoint"
  gnss_integrity: "Boolean"
  horizontal_error: "Real"
}
class "SensorFrame" {
  id: "CL-FRAME"
  frame_id: "Identifier"
  timestamp: "Timestamp"
  channel: "SensorChannel"
  footprint: "Waypoint[]"
  pixels: "Bytes"
  marking: "ClassificationMarking"
}
class "Track" {
  id: "CL-TRACK"
  track_id: "Identifier"
  position: "Waypoint"
  speed: "GroundSpeed"
  confidence: "Real"
  source_frame: "SensorFrame"
}
class "HealthReport" {
  id: "CL-HEALTH"
  timestamp: "Timestamp"
  mode: "FlightMode"
  fuel_remaining: "Real"
  bus_voltage: "Real"
  link_margin: "Real"
  fcs_channel_status: "String"
}
class "C2Message" {
  id: "CL-C2"
  sequence: "Integer"
  timestamp: "Timestamp"
  command: "String"
  mode_request: "FlightMode"
  authentication_tag: "Bytes"
}
class "KeyMaterial" {
  id: "CL-KEY"
  key_id: "Identifier"
  algorithm: "String"
  validity: "Duration"
  zeroised: "Boolean"
}

association "FlightPlan has waypoints" { id: "AS-PLAN-WP" from: "CL-PLAN" to: "CL-WP" role: "waypoints" kind: "composition" multiplicity: "2..*" }
association "FlightPlan bounded by geofence" { id: "AS-PLAN-GEO" from: "CL-PLAN" to: "CL-GEOFENCE" role: "geofence" kind: "composition" multiplicity: "1" }
association "Track derived from frame" { id: "AS-TRACK-FRAME" from: "CL-TRACK" to: "CL-FRAME" role: "source_frame" kind: "association" multiplicity: "1" }

exchange_item "C2 message" { id: "EI-C2" mechanism: "EVENT" elements: ["CL-C2"] }
exchange_item "Navigation solution" { id: "EI-NAV" mechanism: "FLOW" elements: ["CL-NAV"] }
exchange_item "Flight plan" { id: "EI-PLAN" mechanism: "DATA" elements: ["CL-PLAN"] }
exchange_item "Sensor video" { id: "EI-VIDEO" mechanism: "FLOW" elements: ["CL-FRAME"] }
exchange_item "Health report" { id: "EI-HEALTH" mechanism: "FLOW" elements: ["CL-HEALTH"] }
exchange_item "Intelligence product" { id: "EI-PRODUCT" mechanism: "DATA" elements: ["CL-FRAME", "CL-TRACK"] }

// ---------------------------------------------------------------------------
// Modes and states
// ---------------------------------------------------------------------------

state_machine "Air vehicle flight modes" {
  id: "SM-AV"
  owner: "LC-AV"
  initial: "Preflight"
  mode Preflight { description: "On the ground, systems powering up, built-in tests running" }
  mode Takeoff { description: "Automatic take-off to the safe altitude" }
  mode Mission {
    description: "Flying the plan under pilot supervision"
    initial: "Transit"
    mode Transit { description: "Flying between the launch site and the area of interest" }
    mode OnStation { description: "Loitering over the area of interest, sensor on target" }
    transition Transit -> OnStation { trigger: "SF-ROUTE" guard: "area of interest reached" }
    transition OnStation -> Transit { trigger: "SF-ROUTE" guard: "next area of interest or egress" }
  }
  mode LostLink { description: "No authenticated C2 frame for 5 s; holding the plan while waiting for the link" }
  mode ReturnToBase { description: "Flying the pre-planned return route at the recovery altitude" }
  mode Landing { description: "Automatic approach and landing at the recovery site" }
  mode Emergency { description: "Flying to the nearest pre-planned emergency site" }

  transition Preflight -> Takeoff { trigger: "SF-FLY" guard: "built-in tests passed and take-off cleared by the pilot" }
  transition Takeoff -> Mission { trigger: "SF-ROUTE" guard: "safe altitude reached" }
  transition Mission -> LostLink { trigger: "SF-LMON" guard: "no authenticated C2 frame for 5 s" timing: 5 s }
  transition LostLink -> Mission { trigger: "SF-LMON" guard: "link re-established and pilot acknowledges" }
  transition LostLink -> ReturnToBase { trigger: "SF-RTB" guard: "link still lost 10 s after the last frame" timing: 10 s }
  transition Mission -> ReturnToBase { trigger: "SF-ROUTE" guard: "mission complete or fuel at bingo" }
  transition ReturnToBase -> Landing { trigger: "SF-ROUTE" guard: "recovery point reached" }
  transition Mission -> Emergency { trigger: "SF-HEALTH" guard: "critical failure reported or emergency commanded by the pilot" }
  transition ReturnToBase -> Emergency { trigger: "SF-HEALTH" guard: "critical failure reported" }
  transition Emergency -> Landing { trigger: "SF-FLY" guard: "emergency site reached" }
  transition Landing -> Preflight { trigger: "SF-FLY" guard: "aircraft stopped" }
}

state_machine "Datalink states" {
  id: "SM-DL"
  owner: "LC-ADT"
  initial: "Idle"
  state Idle { description: "Transceiver powered, no acquisition in progress" }
  state Acquiring { description: "Synchronising the hopping pattern and authenticating the ground data terminal" }
  state Connected { description: "Authenticated frames flowing both ways" }
  state Degraded { description: "Frame error rate above threshold: payload rate reduced, C2 kept" }
  state Lost { description: "No authenticated frame for 5 s" }
  transition Idle -> Acquiring { trigger: "SF-LMON" guard: "ground data terminal in view" }
  transition Acquiring -> Connected { trigger: "SF-AUTH" guard: "mutual authentication succeeded" }
  transition Connected -> Degraded { trigger: "SF-LMON" guard: "frame error rate above threshold for 1 s" }
  transition Degraded -> Connected { trigger: "SF-LMON" guard: "frame error rate back below threshold" }
  transition Degraded -> Lost { trigger: "SF-LMON" guard: "no authenticated frame for 5 s" timing: 5 s }
  transition Connected -> Lost { trigger: "SF-LMON" guard: "no authenticated frame for 5 s" timing: 5 s }
  transition Lost -> Acquiring { trigger: "SF-LMON" guard: "carrier detected again" }
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

scenario "Mission execution with target observation" {
  id: "SC-OBSERVE"
  owner: "CAP-ISR"
  participants: ["LC-PAYLOAD-ST", "LC-GDT", "LC-ADT-C2", "LC-PLM", "LC-EOIR", "LC-CRYPTO", "LC-ADT-PL", "LC-EXPL"]
  message "LC-PAYLOAD-ST" -> "LC-GDT" "steer to target" { exchange: "CE-PAYOP-CMD" type: "async" }
  message "LC-GDT" -> "LC-ADT-C2" "payload command" { exchange: "CE-RF-C2" type: "async" }
  message "LC-ADT-C2" -> "LC-PLM" "payload command" { exchange: "CE-PAYLOAD-CMD" type: "async" }
  message "LC-PLM" -> "LC-EOIR" "pointing command" { exchange: "CE-POINT" type: "async" timing: 20 ms }
  fragment loop "every frame, 30 Hz" {
    message "LC-EOIR" -> "LC-PLM" "raw frame" { exchange: "CE-RAW-VIDEO" type: "async" }
    message "LC-PLM" -> "LC-CRYPTO" "compressed frame" { exchange: "CE-VIDEO" type: "async" }
    message "LC-CRYPTO" -> "LC-ADT-PL" "encrypted frame" { exchange: "CE-VIDEO-ENC" type: "async" }
    message "LC-ADT-PL" -> "LC-GDT" "payload downlink" { exchange: "CE-RF-PL" type: "async" timing: 40 ms }
    message "LC-GDT" -> "LC-PAYLOAD-ST" "frame to display" { exchange: "CE-GCS-VIDEO" type: "async" }
  }
  message "LC-PAYLOAD-ST" -> "LC-EXPL" "annotated frames and tracks" { exchange: "CE-PRODUCT" type: "async" }
}

scenario "Lost-link handling" {
  id: "SC-LOSTLINK"
  owner: "CAP-SAFE"
  participants: ["LC-PILOT-ST", "LC-GDT", "LC-ADT-C2", "LC-FCS", "LC-HMS"]
  message "LC-PILOT-ST" -> "LC-GDT" "flight command" { exchange: "CE-PILOT-CMD" type: "async" }
  message "LC-GDT" -> "LC-ADT-C2" "last authenticated C2 frame" { exchange: "CE-RF-C2" type: "async" }
  message "LC-ADT-C2" -> "LC-FCS" "link lost" { exchange: "CE-LINKSTATE" type: "async" timing: 5 s }
  fragment alt {
    operand "link recovered before 10 s" {
      message "LC-GDT" -> "LC-ADT-C2" "C2 frame" { exchange: "CE-RF-C2" type: "async" }
      message "LC-ADT-C2" -> "LC-FCS" "link restored" { exchange: "CE-LINKSTATE" type: "async" }
    }
    operand "link still lost at 10 s" {
      message "LC-FCS" -> "LC-HMS" "return-to-base engaged" { exchange: "CE-FCS-HEALTH" type: "async" timing: 10 s }
      message "LC-HMS" -> "LC-ADT-C2" "health report" { exchange: "CE-HEALTH" type: "async" }
    }
  }
}

// ---------------------------------------------------------------------------
// Safety analysis
// ---------------------------------------------------------------------------

safety_analysis {
  // Each hazard carries its DO-178C failure condition with the DAL it
  // implies, and the ISO 26262 cross-reading (S/E/C with the ASIL the table
  // gives) so the same HARA passes the three production gates.
  hazard "Loss of control of the air vehicle" {
    id: "HZ-001"
    description: "Flight control fails to hold the aircraft on its route; uncontrolled descent over the area of operations"
    condition: "Hazardous"
    dal: "DAL-B"
    severity: "S3"
    exposure: "E4"
    controllability: "C2"
    asil: "ASIL-C"
    mitigated_by: ["REQ-SAF-003", "REQ-AW-001", "REQ-SAF-005"]
  }
  hazard "Mid-air collision" {
    id: "HZ-002"
    description: "The air vehicle leaves its cleared volume and conflicts with manned traffic"
    condition: "Hazardous"
    dal: "DAL-B"
    severity: "S3"
    exposure: "E2"
    controllability: "C3"
    asil: "ASIL-B"
    mitigated_by: ["REQ-SAF-002", "REQ-SAF-001"]
  }
  hazard "Loss of the C2 link" {
    id: "HZ-003"
    description: "The pilot can no longer command the air vehicle; the aircraft continues without supervision"
    condition: "Major"
    dal: "DAL-C"
    severity: "S2"
    exposure: "E3"
    controllability: "C2"
    asil: "ASIL-A"
    mitigated_by: ["REQ-SAF-001", "REQ-SEC-003", "REQ-AW-003"]
  }
  hazard "Controlled flight into terrain" {
    id: "HZ-004"
    description: "The route or the return route is flown below the terrain, in particular on a spoofed navigation solution"
    condition: "Hazardous"
    dal: "DAL-B"
    severity: "S3"
    exposure: "E3"
    controllability: "C2"
    asil: "ASIL-B"
    mitigated_by: ["REQ-SAF-004", "REQ-SEC-002", "REQ-PERF-003"]
  }

  fmea "Flight control computer channel halt" {
    id: "FM-001"
    failure_mode: "Processor halt of the command channel"
    effect: "Transient in the surface commands until the monitor channel takes over"
    cause: "Single-event upset, power transient"
    severity: "Hazardous"
    occurrence: "1e-6 per flight hour"
    detection: "Cross-channel monitor, switchover within 100 ms"
    rpn: 36
  }
  fmea "C2 transceiver failure" {
    id: "FM-002"
    failure_mode: "Receiver no longer demodulates the C2 waveform"
    effect: "Link declared lost after 5 s; return to base after 10 s"
    cause: "Component failure, antenna damage"
    severity: "Major"
    occurrence: "1e-5 per flight hour"
    detection: "Frame counter, built-in test"
    rpn: 48
  }
  fmea "GNSS fix spoofed or degraded" {
    id: "FM-003"
    failure_mode: "GNSS fix drifts from the true position"
    effect: "Navigation falls back to inertial propagation, error grows 1 m per minute"
    cause: "Spoofing, multipath, constellation outage"
    severity: "Hazardous"
    occurrence: "1e-4 per flight hour in contested areas"
    detection: "GNSS/INS consistency monitor, 50 ms"
    rpn: 60
  }
  fmea "Gimbal stabilisation loss" {
    id: "FM-004"
    failure_mode: "Gyro failure in the stabilisation loop"
    effect: "Video unusable; mission aborted, flight unaffected"
    cause: "Gyro wear, connector vibration"
    severity: "Minor"
    occurrence: "1e-4 per flight hour"
    detection: "Gyro built-in test, operator observation"
    rpn: 24
  }
}

// ---------------------------------------------------------------------------
// Cybersecurity analysis
// ---------------------------------------------------------------------------

security_scales {
  id: "SEC-SCALES"
  confidentiality: ["None", "Low", "Medium", "High"]
  integrity: ["None", "Low", "Medium", "High"]
  availability: ["None", "Low", "Medium", "High"]
  traceability: ["None", "Low", "Medium", "High"]
  threat_kinds: ["Jamming", "Spoofing", "Hijack", "Exfiltration", "Tampering"]
}

security_analysis {
  id: "SEC-SA"
  level: "sa"
  asset "Command and control of the air vehicle" {
    id: "AS-C2"
    kind: "functional"
    description: "The pilot's ability to command the aircraft and the aircraft's trust in what it receives"
    members: ["SF-CMD", "SF-AUTH", "SF-LMON", "EI-C2", "CL-C2"]
    security_integrity: "High"
    security_availability: "High"
  }
  asset "Navigation solution" {
    id: "AS-NAV"
    kind: "functional"
    members: ["SF-NAV", "SF-SPOOF", "EI-NAV", "CL-NAV"]
    security_integrity: "High"
    security_availability: "Medium"
  }
  asset "Payload data" {
    id: "AS-PAYLOAD"
    kind: "information"
    members: ["SF-VIDEO", "SF-STORE", "EI-VIDEO", "CL-FRAME", "CL-TRACK"]
    security_confidentiality: "High"
    security_integrity: "Medium"
  }
  asset "Mission plan" {
    id: "AS-PLAN"
    kind: "information"
    members: ["SF-PLAN", "EI-PLAN", "CL-PLAN"]
    security_confidentiality: "High"
    security_integrity: "High"
  }
  asset "Key material" {
    id: "AS-KEYS"
    kind: "information"
    members: ["SF-ENC", "CL-KEY"]
    security_confidentiality: "High"
    security_integrity: "High"
    security_traceability: "High"
  }
  threat "Jamming of the C2 channel" {
    id: "TH-JAM"
    kind: "Jamming"
    level: 3
    rationale: "A ground jammer within the footprint denies the C2 channel; the aircraft must behave safely without it"
    applies_to: ["AS-C2"]
    involves: ["SA-ACT-PILOT", "SA-ACT-GNSS"]
    security_availability: "High"
  }
  threat "GNSS spoofing" {
    id: "TH-SPOOF"
    kind: "Spoofing"
    level: 3
    rationale: "A spoofed constellation walks the aircraft off its route or below the terrain"
    applies_to: ["AS-NAV"]
    involves: ["SA-ACT-GNSS"]
    security_integrity: "High"
  }
  threat "Datalink hijack" {
    id: "TH-HIJACK"
    kind: "Hijack"
    level: 3
    rationale: "A rogue ground terminal injects or replays C2 frames to take the aircraft"
    applies_to: ["AS-C2", "AS-PLAN"]
    involves: ["SA-ACT-PILOT"]
    security_integrity: "High"
    security_traceability: "High"
  }
  threat "Payload data exfiltration" {
    id: "TH-EXFIL"
    kind: "Exfiltration"
    level: 2
    rationale: "Intercepted downlink or a recovered recorder exposes the imagery and the area of interest"
    applies_to: ["AS-PAYLOAD", "AS-KEYS"]
    involves: ["SA-ACT-PAYOP"]
    security_confidentiality: "High"
  }
}

security_analysis {
  id: "SEC-LA"
  level: "la"
  asset "Datalink service" {
    id: "AS-LINK"
    kind: "functional"
    members: ["LF-C2RX", "LF-LMON", "LF-TX", "LF-ENC", "LF-GLINK"]
    security_integrity: "High"
    security_availability: "High"
  }
  threat "Rogue ground terminal" {
    id: "TH-ROGUE-GDT"
    kind: "Hijack"
    level: 3
    rationale: "The threat of the system level, seen at the components that carry the link"
    applies_to: ["AS-LINK"]
    involves: ["LC-GDT", "LC-ADT-C2", "LC-CRYPTO"]
    security_integrity: "High"
  }
  threat "Wideband jammer" {
    id: "TH-WB-JAM"
    kind: "Jamming"
    level: 3
    applies_to: ["AS-LINK"]
    involves: ["LC-ADT-C2", "LC-ADT-PL", "LC-FCS"]
    security_availability: "High"
  }
}

trace "TH-ROGUE-GDT" realizes "TH-HIJACK"
trace "TH-WB-JAM" realizes "TH-JAM"

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

test_case "Endurance flight" {
  id: "TC-PERF-001"
  description: "Flight test: 24 h sortie with the payload operating, fuel reserve measured at landing"
  method: "demonstration" verifies: ["REQ-PERF-001"]
  expected: "Landing with at least 30 min of fuel after 24 h airborne"
}
test_case "Datalink range and throughput" {
  id: "TC-PERF-002"
  description: "Flight test at 150 km line of sight: payload throughput and frame error rate measured over 30 min"
  method: "test" verifies: ["REQ-PERF-002"]
  expected: "10 Mbit/s sustained, frame error rate below 1e-4"
}
test_case "Navigation accuracy" {
  id: "TC-PERF-003"
  description: "Flight test against a surveyed reference trajectory"
  method: "test" verifies: ["REQ-PERF-003"]
  expected: "Horizontal error at or below 2 m CEP"
}
test_case "Video latency, sensor to screen" {
  id: "TC-PERF-004"
  description: "Timestamped light source in the sensor field of view, time to the operator's display measured on the bench with the real datalink"
  method: "test" verifies: ["REQ-PERF-004"]
  expected: "Latency at or below 500 ms over 1000 frames"
}
test_case "Gimbal stabilisation" {
  id: "TC-PERF-005"
  description: "Motion table test, 30 degrees of bank, line-of-sight jitter measured"
  method: "test" verifies: ["REQ-PERF-005"]
  expected: "Jitter at or below 0.5 mrad RMS"
}
test_case "Lost-link return to base" {
  id: "TC-SAF-001"
  description: "Flight test: C2 transmitter switched off on station; time to the return route measured from the last frame"
  method: "test" verifies: ["REQ-SAF-001", "REQ-SEC-003"]
  expected: "LostLink mode at 5 s, return route commanded before 10 s"
}
test_case "Geofence hold" {
  id: "TC-SAF-002"
  description: "Hardware-in-the-loop: route crossing the geofence uploaded; hold commanded before the boundary"
  method: "test" verifies: ["REQ-SAF-002"]
  expected: "Hold commanded within 500 ms, boundary never crossed"
}
test_case "Flight control channel switchover" {
  id: "TC-SAF-003"
  description: "Fault injection on the command channel; surface command continuity analysed"
  method: "analysis" verifies: ["REQ-SAF-003"]
  expected: "Switchover within 100 ms, no surface transient above 2 degrees"
}
test_case "Terrain clearance" {
  id: "TC-SAF-004"
  description: "Hardware-in-the-loop: return route over high terrain, minimum altitude checked along the route"
  method: "test" verifies: ["REQ-SAF-004"]
  expected: "Altitude never below the minimum safe altitude"
}
test_case "Emergency recovery" {
  id: "TC-SAF-005"
  description: "Flight test: emergency recovery commanded from each flight mode"
  method: "demonstration" verifies: ["REQ-SAF-005"]
  expected: "Aircraft flies to the planned emergency site from every mode"
}
test_case "C2 authentication" {
  id: "TC-SEC-001"
  description: "Replayed and forged C2 frames injected on the bench"
  method: "test" verifies: ["REQ-SEC-001", "REQ-SEC-004"]
  expected: "Every forged or replayed frame discarded and counted; keys zeroised on command"
}
test_case "GNSS spoofing detection" {
  id: "TC-SEC-002"
  description: "GNSS simulator walks the fix 50 m from the inertial solution"
  method: "test" verifies: ["REQ-SEC-002"]
  expected: "Fix rejected within 1 s, inertial-only navigation continued"
}
test_case "Flight control software DAL B compliance" {
  id: "TC-AW-001"
  description: "Review of the DO-178C life-cycle data of the flight control software"
  method: "inspection" verifies: ["REQ-AW-001"]
  expected: "DAL B objectives met with independence where required"
}
test_case "Mission software partitioning" {
  id: "TC-AW-002"
  description: "Review of the partitioning analysis and of the DAL C life-cycle data of the mission software"
  method: "inspection" verifies: ["REQ-AW-002"]
  expected: "No path from the mission partition to the flight-critical partition outside the validated bus interface"
}
test_case "Health report rate" {
  id: "TC-AW-003"
  description: "Health report timestamps logged at the pilot station over one hour"
  method: "test" verifies: ["REQ-AW-003"]
  expected: "One report per second, none older than 2 s"
}

// ---------------------------------------------------------------------------
// Constraints: what the model claims, checked on its values
// ---------------------------------------------------------------------------

constraint "Lost-link reaction within budget" {
  id: "CST-LOSTLINK"
  assert: sum("FC-LOSTLINK", latency) <= "FC-LOSTLINK".latency_budget
}
constraint "Video chain within budget, processing and transport" {
  id: "CST-VIDEO"
  assert: sum("FC-OBSERVE", latency) + sum("PP-VIDEO", latency) <= "FC-OBSERVE".latency_budget
}
constraint "Geofence chain within budget" {
  id: "CST-GEOFENCE"
  assert: sum("FC-GEOFENCE", latency) <= "FC-GEOFENCE".latency_budget
}
constraint "Video fits the payload datalink" {
  id: "CST-DOWNLINK"
  assert: "Payload data interface".bandwidth <= "Payload datalink".bandwidth
}
constraint "C2 traffic fits the C2 datalink" {
  id: "CST-C2LINK"
  assert: "C2 message interface".bandwidth <= "C2 datalink".bandwidth
}
constraint "Avionics within the power distribution budget" {
  id: "CST-POWER"
  assert: "PN-FCC".power + "PN-GNSS-INS".power + "PN-MC".power + "PN-RADIO".power + "PN-GIMBAL".power <= "PN-PDU".power
}

// ---------------------------------------------------------------------------
// Traceability
// ---------------------------------------------------------------------------

// System functions answer the operational need
trace "SF-PLAN" realizes "OA-PLAN"
trace "SF-ROUTE" realizes "OA-FLY"
trace "SF-VIDEO" realizes "OA-OBSERVE"
trace "SF-EXPL" realizes "OA-DISSEM"
trace "SF-RTB" realizes "OA-RECOVER"

// Components satisfy the requirements
trace "LC-AV" satisfies "REQ-PERF-001" { rationale: "Fuel and power budgets sized for 24 h with the payload operating" }
trace "LC-ADT" satisfies "REQ-PERF-002" { rationale: "Payload channel sized at 10 Mbit/s with link margin at 150 km" }
trace "LC-GDT" satisfies "REQ-PERF-002" { rationale: "Tracking antenna gain closes the link budget at 150 km" }
trace "LC-NAV" satisfies "REQ-PERF-003" { rationale: "GNSS/INS blending at 100 Hz" }
trace "LC-PLM" satisfies "REQ-PERF-004" { rationale: "Compression latency 120 ms within the 500 ms chain" }
trace "LC-PAYLOAD-ST" satisfies "REQ-PERF-004" { rationale: "Decode and display within 40 ms" }
trace "LC-EOIR" satisfies "REQ-PERF-005" { rationale: "Four-axis stabilisation" }
trace "LC-FCS" satisfies "REQ-SAF-001" { rationale: "Lost-link return executed by the flight-critical partition" }
trace "LC-ADT-C2" satisfies "REQ-SAF-001" { rationale: "Declares the link lost after 5 s without an authenticated frame" }
trace "LC-FCS" satisfies "REQ-SAF-002" { rationale: "Geofence prediction 30 s ahead, hold commanded before the boundary" }
trace "LC-FCS" satisfies "REQ-SAF-003" { rationale: "Dual channel with cross monitoring" }
trace "LC-FCS" satisfies "REQ-SAF-004" { rationale: "Minimum safe altitude enforced on every route" }
trace "LC-MMS" satisfies "REQ-SAF-004" { rationale: "Plans validated against the terrain database before upload" }
trace "LC-PILOT-ST" satisfies "REQ-SAF-005" { rationale: "Emergency recovery command available in every mode" }
trace "LC-FCS" satisfies "REQ-SAF-005" { rationale: "Emergency mode reachable from every flight mode" }
trace "LC-ADT-C2" satisfies "REQ-SEC-001" { rationale: "Authentication tag and sequence number checked on every frame" }
trace "LC-CRYPTO" satisfies "REQ-SEC-001" { rationale: "Holds the session keys of the C2 channel" }
trace "LC-NAV" satisfies "REQ-SEC-002" { rationale: "GNSS/INS consistency monitor" }
trace "LC-ADT" satisfies "REQ-SEC-003" { rationale: "Frequency hopping on the C2 channel" }
trace "LC-CRYPTO" satisfies "REQ-SEC-004" { rationale: "Encryption in transit, zeroisation on command or tamper" }
trace "LC-PLM" satisfies "REQ-SEC-004" { rationale: "Recorder encrypted at rest" }
trace "LC-FCS" satisfies "REQ-AW-001" { rationale: "Flight control software developed to DAL B" }
trace "LC-MMS" satisfies "REQ-AW-002" { rationale: "Mission software developed to DAL C in its own partition" }
trace "LC-HMS" satisfies "REQ-AW-003" { rationale: "Health report built at 1 Hz" }
trace "LC-GCS" satisfies "REQ-AW-003" { rationale: "Health shown at the pilot station" }
trace "LC-ADT-PL" satisfies "REQ-PERF-002" { rationale: "Payload transmitter at 10 Mbit/s" }
trace "LC-MP" satisfies "REQ-SAF-002" { rationale: "Geofence drawn and validated at planning" }
trace "LC-EXPL" satisfies "REQ-SEC-004" { rationale: "Products stored encrypted at rest" }

// Configuration items realize the nodes
trace "CI-FCC" realizes "PN-FCC"
trace "CI-FCS-SW" realizes "BC-FCS-SW"
trace "CI-MC" realizes "PN-MC"
trace "CI-MC-SW" realizes "BC-MC-SW"
trace "CI-GNSS-INS" realizes "PN-GNSS-INS"
trace "CI-GIMBAL" realizes "PN-GIMBAL"
trace "CI-PDU" realizes "PN-PDU"
trace "CI-RADIO" realizes "PN-RADIO"
trace "CI-GDT" realizes "PN-GDT"
trace "CI-PILOT-WS" realizes "PN-PILOT-WS"
trace "CI-PAYLOAD-WS" realizes "PN-PAYLOAD-WS"
trace "CI-MP-WS" realizes "PN-MP-WS"

// ---------------------------------------------------------------------------
// Diagrams
// ---------------------------------------------------------------------------

diagram "Lost-link reaction chain" {
  view: sab
  description: "The system functions of the lost-link chain and the exchanges between them"
  shows: ["SF-LMON", "SF-RTB", "SF-ROUTE", "SF-FLY", "SF-AUTH", "SF-NAV"]
}

diagram "Datalink architecture" {
  view: lab
  description: "The ground data terminal, the airborne data terminal and what they serve"
  shows: ["LC-GDT", "LC-ADT-C2", "LC-ADT-PL", "LC-CRYPTO", "LC-FCS", "LC-MMS", "LC-PLM", "LC-HMS"]
}

diagram "Air vehicle deployment" {
  view: pab
  description: "The airborne LRUs, their software and the buses between them"
  shows: ["PN-FCC", "PN-GNSS-INS", "PN-MC", "PN-RADIO", "PN-GIMBAL", "PN-PDU"]
}
