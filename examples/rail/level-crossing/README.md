# Railway level-crossing protection

Detect the train, close the barriers, prove the crossing protected, fall back to a safe state on any failure.

**Level:** Intermediate · **Shows:** IEC 61508 / EN 50129, SIL, risk graph, fail-safe modes, scenario

## What it models

Train detection, road warning, barrier closure, clear proving and the fail-safe state, with SIL levels, a risk graph read in the gate's classes, modes with function triggers, a scenario and a constraint on the protection cycle.

## What to look at

- The fail-safe state and how every failure leads to it.
- The constraint on the protection cycle.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate --standard iec61508 examples/rail/level-crossing/level_crossing_protection.arc
```

## To understand by changing it

- Run the modes in the debugger: approach, close, prove, and a detector failure.

## Files

- `level_crossing_protection.arc`
