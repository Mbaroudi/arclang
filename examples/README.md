# ArcLang examples

Models written in ArcLang, the Arcadia method as code. Each example is a folder with the model, its Modelica physics when it is simulated, and a README: what it models, what to look at, how to open it, and a few changes to make to see how the compiler and the production gate react.

Every model here compiles with the strict parser, and CI compiles them all (`tests/examples_compile.rs`). The complete models pass the production gate of their standard: DO-178C for aerospace and defense (`arclang gate --standard do178c`), IEC 61508 for rail and energy, ISO 26262 otherwise. The simulated models pass it once `arclang verify` has run their cases; until then the gate reports them inconclusive. The `multifile` fragments show how files are assembled and carry no test cases; the Capella sample keeps the findings of its source model. `archive/` keeps the models of the old syntax, which are not maintained.

## Where to start

1. [Adaptive cruise control, the starter](01-getting-started/acc-minimal/): one capability through the four Arcadia levels. Read it first.
2. [Typed quantities and constraints](features/timing-constraints/): units, types and constraints the compiler checks.
3. [Autonomous emergency braking](automotive/emergency-braking/): the flagship, every level with ISO 26262 safety, modes and a scenario to run in the debugger.
4. [Electric vehicle range](automotive/ev-range/): a requirement verified by simulation in OpenModelica.
5. [Liquid cooling of an AI training hall](datacenter/ai-liquid-cooling/): ten scenarios, failures and a Monte Carlo.
6. [Adaptive cruise control with Stop & Go](automotive/acc-stop-and-go/): a full engineering model in CESAM, from the stakeholders to the bill of materials, verified by simulation, by executing its modes and by its fault trees.
7. Then the domain closest to yours, below.

## How to open an example

- **On [arclang.fr/playground](https://arclang.fr/playground/)**: the examples are already in `examples/` of your workspace. Open a `.arc` file, then *ArcLang: Open the workbench beside the source*.
- **In VS Code** with the ArcLang extension: open this folder, then the same command.
- **From the command line**, at the root of the repository: `arclang check`, `arclang gate`, `arclang workbench`, `arclang verify` (simulations) followed by the path of the model; each README gives the commands.

Simulation needs OpenModelica: on the desktop, *ArcLang: Set up OpenModelica*; on arclang.fr, sign in and the simulations run on the server. *ArcLang: Run…* (or F5) steps through state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

## Getting started

| Example | Level | What it is |
|---|---|---|
| [Adaptive cruise control, the starter](01-getting-started/acc-minimal/) | Beginner | One capability followed through the four Arcadia levels: the smallest complete model, and the one to read first. |
| [A model in several files](01-getting-started/multifile/) | Beginner | Each team owns its file; the root file imports them and the traces resolve across the files. |

## Language features

| Example | Level | What it is |
|---|---|---|
| [Typed quantities and constraints](features/timing-constraints/) | Beginner | Units, user-defined types and constraints: the compiler checks dimensions, the gate checks the assertions. |
| [Diagram layout demo](features/diagram-layout/) | Beginner | A small cruise control model built to exercise the diagram layout: many ports, nesting, fan-in and fan-out. |

## Automotive

| Example | Level | What it is |
|---|---|---|
| [Autonomous emergency braking (AEB)](automotive/emergency-braking/) | Intermediate | The flagship: an AEB system through every Arcadia level, with ISO 26262 safety, modes, a scenario and declared diagrams. |
| [Adaptive cruise control with Stop & Go, the engineering model](automotive/acc-stop-and-go/) | Advanced | A full-speed-range ACC in CESAM: stakeholders, three visions, ECUs and bill of materials; the control law simulated, the modes executed, HARA, FMEA, fault trees and TARA. |
| [Adaptive cruise control, complete](automotive/adaptive-cruise/) | Intermediate | The full ACC: perception, fusion, target selection, longitudinal control, driver override and safety monitoring. |
| [One-pedal drive](automotive/one-pedal-drive/) | Intermediate | The accelerator pedal also brakes: regeneration first, then the friction brake, then the hold at standstill. |
| [An electric car product line](automotive/ev-product-line/) | Intermediate | What Arcadia gains from SysML v2: definitions, variants and configurations, typed allocations, tagged views, analyses. |
| [Remote start of a vehicle](automotive/remote-start/) | Advanced | Starting the car from a phone, for ICE, hybrid and electric powertrains: safety and cybersecurity together. |
| [Electric vehicle range, verified by simulation](automotive/ev-range/) | Advanced | A range requirement checked by physics: each configuration of the car is driven in OpenModelica. |
| [A fast charge kept cool](automotive/battery-fast-charge/) | Advanced | Charging from 10 % to 80 % in OpenModelica: cell heating, the cooling loop and the power the battery management allows. |

## Aerospace

| Example | Level | What it is |
|---|---|---|
| [Fly-by-wire flight control](aerospace/flight-control/) | Intermediate | Pilot inputs to control surfaces through two dissimilar computers, with what a DO-178C / ARP4754A review asks for. |
| [Inspection drone hover endurance](aerospace/drone-endurance/) | Advanced | Each drone configuration flown in OpenModelica: climb to 50 m, hold, and see how long the battery lasts. |
| [Aircraft electrical network, generator loss](aerospace/electrical-network/) | Advanced | A 28 V DC network loses its generator: load shedding and the battery must carry the essential bus for 30 minutes. |

## Defense

| Example | Level | What it is |
|---|---|---|
| [Mission computer of a combat aircraft](defense/mission-computer/) | Intermediate | Radar and electronic warfare fused into tracks, threats ranked, and a weapon released only with two operators. |
| [Tactical ISR drone](defense/isr-drone/) | Advanced | Air vehicle, ground station and datalink, written to pass the production gate under three standards. |

## Rail

| Example | Level | What it is |
|---|---|---|
| [Railway level-crossing protection](rail/level-crossing/) | Intermediate | Detect the train, close the barriers, prove the crossing protected, fall back to a safe state on any failure. |

## Energy

| Example | Level | What it is |
|---|---|---|
| [Battery charger controller](energy/battery-charger/) | Beginner | Charging modes and a fan as state machines, bound to I/O channels, generated as portable C and proven against the model in the loop (`arclang export -f c`, `arclang sil`, see [docs/embedded-c.md](../docs/embedded-c.md)). Behavior only: no architecture, not written for the gate. |
| [DC fast-charging station](energy/ev-charging-station/) | Advanced | A 350 kW public charger and its back end: CCS2, ISO 15118 plug-and-charge, OCPP 2.0.1, load management, payment. |

## Data center

| Example | Level | What it is |
|---|---|---|
| [Liquid cooling of an AI training hall](datacenter/ai-liquid-cooling/) | Advanced | Sixteen GPU racks on direct-to-chip cooling, ten scenarios simulated in OpenModelica, from a pump failure to a grid outage. |

## Semiconductor

| Example | Level | What it is |
|---|---|---|
| [AI accelerator platform](semiconductor/ai-accelerator/) | Advanced | A tray of eight AI accelerators, from the ML engineer's need down to the tensor cores, HBM and the cooling loop. |

## Building

| Example | Level | What it is |
|---|---|---|
| [Heating a house with a heat pump](building/heat-pump/) | Advanced | The coldest day of the year in OpenModelica: does the heat pump hold 21 °C through the night? |

## Spatial

| Example | Level | What it is |
|---|---|---|
| [A charging corridor in space](spatial/charging-corridor/) | Intermediate | Three charging hubs on a motorway, the fibre between them and the supervision center, placed on the map. |

## Business and IT

| Example | Level | What it is |
|---|---|---|
| [Data platform migration](business/data-platform-migration/) | Intermediate | Arcadia applied to an IT program: moving a media company's data from Oracle and Snowflake to a Databricks lakehouse. |

## Capella

| Example | Level | What it is |
|---|---|---|
| [Adaptive cruise control, after a Capella import](capella/acc-from-capella/) | Beginner | What a Capella import gives, and what the engineer adds in the first edit pass. |
| [In-flight entertainment (Capella sample)](capella/in-flight-entertainment/) | Reference | The Eclipse Capella reference sample, imported as is: 1 600 declarations and 118 diagrams. |

## Levels

- **Beginner**: short models, one idea each.
- **Intermediate**: a complete system through the Arcadia levels, with its safety analysis.
- **Advanced**: large models, simulation in OpenModelica, security analyses, several standards.
- **Reference**: imported as is, kept to test the exchanges.
