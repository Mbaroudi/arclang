# Autonomous emergency braking (AEB)

The flagship: an AEB system through every Arcadia level, with ISO 26262 safety, modes, a scenario and declared diagrams.

**Level:** Intermediate · **Shows:** all Arcadia levels, ISO 26262 HARA, functional chain latency, state machine, scenario, data model, EPBS

## What it models

Three models of the same emergency braking function. `complete_emergency_braking_simple.arc` is the flagship: the mission and capability, a functional chain with its latency budget, logical components with ports and exchanges, ECUs, buses and deployments, the HARA, the modes, a scenario, a data model, test cases on every requirement and declared diagrams; the tests and the golden files are built on it. `complete_emergency_braking_mbse.arc` writes the same system down to the fifth level, the end product breakdown (EPBS). `emergency_braking_logical.arc` looks at the logical architecture in depth: a three-sensor perception chain (radar, camera, lidar) and the degraded modes that keep the function alive when a sensor fails.

## What to look at

- The functional chain from detection to braking and its latency budget, which the gate sums.
- The HARA: each hazard's ASIL drives the integrity level of the requirements and components behind it.
- The modes (a state machine) and the scenario: run them step by step in the debugger.
- In the `_mbse` model, the EPBS: the configuration items that are bought and built.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate examples/automotive/emergency-braking/complete_emergency_braking_simple.arc
arclang workbench examples/automotive/emergency-braking/complete_emergency_braking_simple.arc
arclang check examples/automotive/emergency-braking/emergency_braking_logical.arc
```

## To understand by changing it

- Raise the latency of one function of the chain from 25 ms to 60 ms: the sum, 125 ms, passes the 100 ms budget and the gate fails on the chain.
- In the debugger, fire the events of the braking modes one by one and watch the active state light up in the workbench.

## Files

- `complete_emergency_braking_simple.arc`
- `complete_emergency_braking_mbse.arc`
- `emergency_braking_logical.arc`
