# Electric vehicle range, verified by simulation

A range requirement checked by physics: each configuration of the car is driven in OpenModelica.

**Level:** Advanced · **Shows:** simulation (Modelica), test cases with verdicts, envelope sweep, configurations, debugger

## What it models

The physical architecture is exported to Modelica and the physics is hand-written in `physics/EVPhysics.mo`. `arclang verify` simulates each configuration: the long range pack keeps its reserve over 300 km at -10 °C, the standard pack does not. Each run ends when the 300 km are driven or the pack is empty. The envelope case sweeps the long range pack over the temperatures and speeds the requirement names.

## What to look at

- The `test_case`s with `simulation` and their expectations (`final(…) >= 300 km`): each names the requirement it checks.
- `physics/EVPhysics.mo`: the battery, the motor and the vehicle, written in Modelica.
- The drive modes, a winter highway trip and the speed control loop: run them in the debugger.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

Simulation needs OpenModelica: on the desktop, *ArcLang: Set up OpenModelica*; on arclang.fr, sign in and the simulations run on the server. The production gate waits for the simulations: until `arclang verify` has run the cases, it reports them inconclusive.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang verify examples/automotive/ev-range/ev_range_simulation.arc
arclang export examples/automotive/ev-range/ev_range_simulation.arc -f modelica -o out/
```

## To understand by changing it

- Run the simulations and open the report (PDF): one page per test case, with the curves and the first instant an expectation fails.
- Change the capacity of the standard pack until it passes the 300 km case.

## Files

- `ev_range_simulation.arc`
- `physics/EVPhysics.mo`
