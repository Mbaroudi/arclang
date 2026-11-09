# Adaptive cruise control with Stop & Go, the engineering model

A full-speed-range ACC (ISO 15622:2018) for a passenger car, modelled the way an engineering team would deliver it: from the stakeholders and their needs down to the ECUs, the buses and the bill of materials, with every requirement derived, allocated and verified.

**Level:** Advanced · **Shows:** CESAM (all three visions, all six aspects), every Arcadia level, simulation, executed modes, safety and security analyses

## What it models

`acc_stop_and_go.arc` follows CESAM (`method: "cesam"`), so the workbench opens it on the CESAM grid; it reads as Arcadia or SysML v2 all the same, through 23 diagrams.

| Vision | What it holds |
|---|---|
| Operational (why) | 7 stakeholders and 8 needs, 11 stakeholder requirements; the operational world (driver, vehicle ahead, cut-in vehicle, following vehicle, technician, road, weather) with 12 activities, 11 interactions, 5 operational capabilities and 2 processes; the vehicle lifecycle; a cut-in scenario |
| Functional (what) | 9 system actors, 2 missions, 7 capabilities; 22 system functions and 49 functional exchanges typed by 25 exchange items and 18 classes; 5 functional chains with latency budgets; 21 functional requirements with figures; the 7 ACC modes as an executable machine; 2 functional scenarios |
| Constructional (how) | 13 logical components (34 functions, 34 exchanges, 4 interfaces); 15 technical requirements, each with its subject; the sensor configurations; 9 ECUs with 6 hardware parts, 13 behavior components and 34 physical functions, 8 links, 7 frames and 3 routes; the product breakdown; 5 constructional scenarios |

Around them, a HARA (6 hazardous events, S/E/C and ASIL) with an FMEA of 10 failure modes in FIT, the basic events of the fault trees `arclang safety --standard iso26262 --fta` computes (each hazard within its budget); a TARA (3 assets, 3 threats); and 125 traces: every functional requirement derived from a stakeholder requirement, every technical one from a functional one, each satisfied by the functions and components that answer it.

## How it is verified

- **The control law, by simulation.** `physics/ACCPhysics.mo` is the constant-time-gap controller of the logical component *Longitudinal controller*, with the ego vehicle's longitudinal dynamics and four traffic scenarios: a braking lead vehicle, a cut-in 22 m ahead, an emergency stop at 7 m/s² (beyond the ACC's 3.5 m/s²: the takeover request, the driver's reaction) and stop-and-go in a jam. OpenModelica runs them; the verdicts read the time gap, the deceleration, the jerk, the distance, the speed.
- **The modes, by execution.** The machine *ACC operating modes* is generated as C and run against the model (`arclang sil`), and as an FMU in OMSimulator (`arclang cosim`): override by the brake pedal, standstill and drive-off on resume, and a fault through the I/O of the safety MCU.
- **The timing and the power, by analysis.** Six constraints sum the latencies along the chains and the power of the ECUs.
- **The rest** (track, HIL bench, supplier reports, inspections) are written as planned cases with their procedure: their verdicts are recorded when they are run.

## Try it

On [arclang.fr/playground](https://arclang.fr/playground/), open `acc_stop_and_go.arc`, then *ArcLang: Open the workbench beside the source*: the CESAM grid, then Arcadia · Capella or SysML v2 above it. Signed in, *Simulate* runs the four simulations and the co-simulations on the server.

From the command line, at the root of the repository:

```bash
arclang check  examples/automotive/acc-stop-and-go/acc_stop_and_go.arc --lint
arclang table  examples/automotive/acc-stop-and-go/acc_stop_and_go.arc --kind cesam_grid -f md
arclang verify examples/automotive/acc-stop-and-go/acc_stop_and_go.arc
arclang sil    examples/automotive/acc-stop-and-go/acc_stop_and_go.arc
arclang gate   examples/automotive/acc-stop-and-go/acc_stop_and_go.arc
```
