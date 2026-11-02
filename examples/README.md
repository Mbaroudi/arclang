# ArcLang examples

Models written in ArcLang, the Arcadia method as code. Every example under
this directory (except `legacy/`) compiles with the strict parser, reports
zero findings with `arclang check`, and passes the production gate of its
standard; CI compiles them all (`tests/examples_compile.rs`, workflow
`Compile all active examples`). The generated outputs (explorer pages, JSON
exports) are not kept here: `arclang explorer <file>` and
`arclang export <file> -f json -o <out>` regenerate them.

To open a model in the workbench:

```bash
arclang workbench examples/complete_emergency_braking_simple.arc
arclang check --json examples/automotive/adaptive_cruise_control.arc
arclang gate --standard iso26262 examples/automotive/adaptive_cruise_control.arc
```

## Automotive

`complete_emergency_braking_simple.arc` (657 lines) is the flagship: an
autonomous emergency braking system through the four Arcadia levels, with the
mission and capability, a functional chain with its latency budget, logical
components with ports and exchanges, ECUs, buses and deployments, ISO 26262
HARA, modes, a scenario, a data model, test cases on every requirement and
declared diagrams. The tests, the golden files and the playground are built
on it. `arclang workbench examples/complete_emergency_braking_simple.arc`

`complete_emergency_braking_mbse.arc` (379 lines) is the same system written
down to the fifth level, the end product breakdown structure, with a product
breakdown diagram. `arclang workbench examples/complete_emergency_braking_mbse.arc`

`emergency_braking_logical.arc` (203 lines) looks at the logical architecture
of the braking function in depth: a three-sensor perception chain (radar,
camera, lidar) and the degraded modes that keep the function alive when one
sensor fails. `arclang workbench examples/emergency_braking_logical.arc`

`automotive/adaptive_cruise_control.arc` (528 lines) is the complete adaptive
cruise control: perception, fusion, target selection, longitudinal control,
driver override and safety monitoring, from the operational need to the ECUs
and the product breakdown, with two functional chains checked by constraints,
HARA, FMEA, modes, scenarios and a data model.
`arclang workbench examples/automotive/adaptive_cruise_control.arc`

`automotive/acc_minimal.arc` (106 lines) is the starter: one capability
followed through the four levels, three requirements, one hazard, three test
cases. Read this one first. `arclang workbench examples/automotive/acc_minimal.arc`

`automotive/acc_complete_architecture.arc` (279 lines) details the logical
architecture of the ACC: nine components, twenty-five logical functions and
the dataflow between them as functional exchanges.
`arclang workbench examples/automotive/acc_complete_architecture.arc`

`automotive/acc_from_capella.arc` (91 lines) shows what a Capella import looks
like after the first edit pass: the imported requirements, components and
traces, completed with the operational need, the functions and the test cases.
`arclang trace examples/automotive/acc_from_capella.arc --validate --matrix`

`automotive/timing_constraints.arc` (155 lines) demonstrates typed quantities,
user-defined types with required attributes and specialization, and
constraints over chains and links (sums, maxima, bus load), evaluated by the
gate. `arclang gate examples/automotive/timing_constraints.arc`

`automotive/remote_start/remote_start_architecture.arc` (769 lines) is a
vehicle remote start function for ICE, hybrid and electric powertrains:
stakeholder, system, safety, security and regulatory requirements refined and
traced, sixteen logical components on nine nodes, ISO 26262 HARA and an
ISO/SAE 21434 security analysis (assets, threats, scales), a scenario with an
`alt` fragment. `WORKFLOW_DEMO.md` and `README_SHARING.md` in the same
directory describe the workflow it came from and how to share its explorer.
`arclang workbench examples/automotive/remote_start/remote_start_architecture.arc`

## Aerospace

`aerospace/flight_control_system.arc` (329 lines) is a fly-by-wire primary
flight control system with two dissimilar flight control computers: DAL on
every element, a functional hazard assessment with the failure conditions, an
ICD on every ARINC 664 link, the channel switchover modes and the EPBS.
`arclang gate --standard do178c examples/aerospace/flight_control_system.arc`

## Rail

`rail/level_crossing_protection.arc` (270 lines) is a level crossing
protection system: train detection, road warning, barrier closure, clear
proving and the fail-safe state, with SIL levels, a risk graph read in the
gate's classes, modes with function triggers, a scenario and a constraint on
the protection cycle.
`arclang gate --standard iec61508 examples/rail/level_crossing_protection.arc`

## Defense

`defense/mission_computer.arc` (368 lines) is the tactical mission computer
of a two-seat combat aircraft: radar and electronic warfare data fused into
tracks, threat evaluation, and a weapon release that needs both operators,
with DAL levels, hazards classified by failure condition, the release state
machine and an engagement scenario.
`arclang gate --standard do178c examples/defense/mission_computer.arc`

### Tactical ISR drone

**File:** `defense/isr_drone.arc`

A tactical ISR unmanned aircraft system: air vehicle, ground control station and 150 km line-of-sight datalink, modeled through every Arcadia level and written to pass the production gate as it stands. Operational analysis (commander, pilot, payload operator, ATC, maintainer, an ISR sortie process), system functions with ports, exchanges and three functional chains with latency budgets (lost-link return to base within 10 s, sensor-to-screen video within 500 ms, geofence hold), 17 requirements each satisfied by a component and verified by a test case, a logical architecture with DAL B flight control and DAL C mission system, physical nodes typed through reusable `type`s with power, mass and memory in units, RF and avionics links with a complete ICD, configuration items, a data model (waypoints, flight plan, sensor frames, tracks, C2 messages, classification marking), two state machines, two scenarios with fragments, a HARA that passes the DO-178C, ISO 26262 and IEC 61508 gates, FMEA lines, and a cybersecurity analysis (jamming, GNSS spoofing, datalink hijack, payload exfiltration) on declared scales. Zero findings; three named diagrams.

```bash
arclang check examples/defense/isr_drone.arc
arclang gate --standard do178c examples/defense/isr_drone.arc
arclang workbench examples/defense/isr_drone.arc
```

## Energy

### DC fast-charging station

**File:** `energy/ev_charging_station.arc`

A public 350 kW DC fast-charging station and its charge point operator back end: CCS2 liquid-cooled connector at 1000 V / 500 A, ISO 15118-2 plug-and-charge, OCPP 2.0.1 to the CSMS over TLS 1.3, a 630 kVA grid connection shared by eight chargers under load management, an EMV payment terminal and RFID fallback. The model runs through every Arcadia level: operational actors and process, system functions with ports and three latency-budgeted functional chains (emergency stop in 100 ms, protection trip, session start in 10 s), logical components with interfaces and `realizes` traces, physical nodes with CAN FD, Ethernet, control-pilot PLC, DC bus and cellular links carrying ICD-complete exchanges, EPBS configuration items, plus a data model (charging session, signed meter values, tariff, fault record), the OCPP connector state machine with function-triggered transitions, two scenarios, a HARA with FMEA entries, and a cybersecurity analysis (payment data, firmware, session records against OCPP spoofing, skimming and firmware tampering). It showcases typed attributes with units (`power: 350 kW`, `efficiency: 97 %`, `rated_current: 500 A`), user-defined types, and five compiler-evaluated constraints including `voltage * current >= power`. Zero findings; passes `arclang gate --standard iec61508`; round-trips through the SysML v2 exporter and importer.

```bash
arclang check examples/energy/ev_charging_station.arc
arclang gate --standard iec61508 examples/energy/ev_charging_station.arc
arclang workbench examples/energy/ev_charging_station.arc
```

## Data center

### Liquid cooling of an AI training hall

**File:** `datacenter/ai_liquid_cooling.arc`, physics in `datacenter/physics/CoolingPhysics.mo`

Direct-to-chip liquid cooling of an AI hall: sixteen GPU racks give 85 % of their heat to cold plates; a coolant distribution unit (CDU) carries it through a plate exchanger to the facility water; dry coolers give it to the outside air and a chiller helps on hot afternoons. A product line of three variation points: the racks (120 kW today, 200 kW next generation), the CDU (two pumps, three pumps N+1, high flow N+1), the heat rejection (900 kW chiller, or larger coolers and 1.6 MW), in four configurations. The operational need, eleven requirements, the supply loop and the leak isolation chain with their latency budgets, the cooling modes (a state machine) and a pump failure during a training run (a scenario). Every component of the physics is placed and wired, so OMEdit draws and simulates each scenario. `arclang verify` runs them in OpenModelica:

| Scenario | Verdict |
|---|---|
| Design summer day (35 °C), N+1 pumps | pass: GPUs at 76 °C at most, supply at most 36.4 °C, partial PUE 1.077 |
| A CDU pump stops at 14 h, two pumps | fail: the controller caps the GPUs to 69 % of their power to hold 83 °C |
| The same failure, N+1 pumps | pass: the spare pump takes over |
| Grid outage at 14 h (pumps on the UPS, fans on the generator after 30 s, chiller back after 10 min) | pass: the dry coolers alone hold the GPUs at 79 °C |
| The generator 1, 3 or 5 min late, with the loop's 40 MJ/K or a buffer tank (160 MJ/K) | 5 of 6: 3 min ride through on the loop alone; 5 min throttle (94 %) unless the tank is there |
| A rack leaks at 14 h: valves closed in 8.5 s, rack powered off 10, 20, 30 or 60 s after the alarm | 2 of 4: off within 20 s its GPUs stay under 95 °C; at 30 s they reach 102 °C, at 60 s 140 °C; the other racks train on |
| Racks of 200 kW on the current plant | fail: GPUs capped to 73 % |
| Racks of 200 kW, high flow CDU, larger coolers, 1.6 MW chiller | pass: GPUs at 77 °C, partial PUE 1.075 |
| 30 to 45 °C outside × 100 to 132 kW per rack | 9 of 12: at 45 °C (and 40 °C with 132 kW) the GPUs throttle |
| Monte Carlo over the summers (35 °C ± 3 K, 110 to 132 kW per rack) | pass: 60 of 60 days without throttling |

```bash
arclang check examples/datacenter/ai_liquid_cooling.arc
arclang verify examples/datacenter/ai_liquid_cooling.arc
arclang workbench examples/datacenter/ai_liquid_cooling.arc
```

## Semiconductor

### AI accelerator platform

**File:** `semiconductor/ai_accelerator_platform.arc`

A data-center compute tray of eight AI accelerators (tensor cores, 141 GB HBM3e per device at 4.8 TB/s, 700 W TDP, a 900 GB/s GPU-to-GPU fabric, a PCIe Gen5 x16 host link, liquid cooling at a 308 K inlet), modeled from operational analysis to physical architecture and EPBS: ML platform engineer, operator, security officer and tenant; system functions and chains with latency budgets (all-reduce step 2.5 ms, thermal throttling 100 ms, power brake 1 ms, attested boot 30 s); compute die, HBM, fabric, host interface, power delivery, thermal manager and security processor inside the accelerator, a fabric switch, a BMC with firmware attestation, the host runtime and the orchestration agent; OAM modules, baseboard, host server, power shelf and CDU with typed links and ICD-complete physical exchanges. It adds a data model (tensor descriptors, kernel launches, telemetry, attestation reports), a power and thermal mode machine whose transitions are triggered by declared functions, three scenarios, a HARA with FMEA, a cybersecurity analysis (model weights, tenant data, firmware; firmware tampering, side channel, fabric eavesdropping), 16 requirements each satisfied and verified, 5 dimension-checked constraints and three named diagrams. Zero findings; passes the production gate under ISO 26262, DO-178C and IEC 61508; round-trips through the SysML v2 export.

```bash
arclang check examples/semiconductor/ai_accelerator_platform.arc
arclang gate --standard iso26262 examples/semiconductor/ai_accelerator_platform.arc
arclang workbench examples/semiconductor/ai_accelerator_platform.arc
```

## Business

`business/data_platform_migration/data_platform_migration.arc` (783 lines)
applies Arcadia to an IT program: the migration of a media company's data
estate from Oracle and Snowflake to a Databricks lakehouse. Stakeholder and
system requirements refined and traced, the data teams' operational need,
twenty-four logical components grouped by layer, the cloud deployment with one
link per pair of nodes, a migration wave chain checked against its nightly
window, migration wave states and acceptance tests. No integrity levels: the
gate checks coverage, ICDs and methodology.
`arclang workbench examples/business/data_platform_migration/data_platform_migration.arc`

## Layout demo

`elk_advanced_demo.arc` (141 lines) is a small cruise control model written to
exercise the diagram layout: components with many ports on both sides, a
nested component, exchanges that fan in and out.
`arclang diagram examples/elk_advanced_demo.arc -f elk -o graphs.json`

## Capella import

`capella/in_flight_entertainment.arc` (4950 lines) is the in-flight
entertainment system imported from the Capella sample model, kept as the
importer wrote it. It is the round-trip corpus of the Capella and SysML v2
exports; its findings are those of the source model and are left as they are.
`arclang workbench examples/capella/in_flight_entertainment.arc`

## Multifile

`multifile/main.arc` imports `requirements.arc` and `logical.arc`: each team
owns its fragment, the root file assembles them, and the traces of the root
resolve across the fragments only after the merge. The fragments report
findings when checked alone, by design. `arclang check examples/multifile/main.arc`

## Legacy

`legacy/` holds files written for syntax versions that were never implemented.
They do not compile and are excluded from the test corpus.
