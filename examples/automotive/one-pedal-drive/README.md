# One-pedal drive

The accelerator pedal also brakes: regeneration first, then the friction brake, then the hold at standstill.

**Level:** Intermediate · **Shows:** ISO 26262, timing budgets, UN R13-H rule, state machine, scenario, change request

## What it models

Written the way a system architect hands it to a design review: requirements with their integrity level, the hazards behind them, the functions and their latency, the ECUs and the buses, the timing budgets the gate checks. The stop lamp rule is the one of UN R13-H (regenerative deceleration above 1.3 m/s²). A change request, CR-217, is described at the top and not applied: apply it yourself and see what it touches.

## What to look at

- The blending of regeneration and friction braking in the functions.
- The stop lamp requirement and the constraint that checks it.
- The comment describing change request CR-217.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate examples/automotive/one-pedal-drive/one_pedal_drive.arc
arclang workbench examples/automotive/one-pedal-drive/one_pedal_drive.arc
```

## To understand by changing it

- Before applying CR-217, run *ArcLang: Impact analysis: what if…* on the element it changes: computed on a copy, it lists the requirements and test cases to review.

## Files

- `one_pedal_drive.arc`
