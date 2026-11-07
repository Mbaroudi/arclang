# Adaptive cruise control, complete

The full ACC: perception, fusion, target selection, longitudinal control, driver override and safety monitoring.

**Level:** Intermediate · **Shows:** all Arcadia levels, functional chains, constraints, HARA, FMEA, modes, scenarios

## What it models

`adaptive_cruise_control.arc` follows the ACC from the operational need to the ECUs and the product breakdown, with two functional chains checked by constraints, a HARA, an FMEA, the operating modes, scenarios and a data model. `acc_complete_architecture.arc` details the logical architecture: nine components, twenty-five logical functions and the dataflow between them, written both as functional exchanges and as component exchanges between ports.

## What to look at

- The two functional chains and the constraints that check their timing.
- The FMEA lines next to the HARA.
- The state machine *ACC operating modes*: its guards are conditions you set in the debugger.
- In `acc_complete_architecture.arc`, the dataflow diagram: the twenty-five functions and their exchanges.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate examples/automotive/adaptive-cruise/adaptive_cruise_control.arc
arclang workbench examples/automotive/adaptive-cruise/adaptive_cruise_control.arc
```

## To understand by changing it

- Run the state machine (*ArcLang: Run…*), fire `LF-022`: nothing happens until you set the guard `main switch on` to true in the debug console.
- Run *ArcLang: Impact analysis: analyze an element…* on a sensor component: see which functions, requirements and test cases a change to it touches.

## Files

- `adaptive_cruise_control.arc`
- `acc_complete_architecture.arc`
