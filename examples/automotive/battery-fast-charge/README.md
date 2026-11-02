# A fast charge kept cool

Charging from 10 % to 80 % in OpenModelica: cell heating, the cooling loop and the power the battery management allows.

**Level:** Advanced · **Shows:** simulation (Modelica), thermal, configurations, test cases

## What it models

The cells heat by their internal resistance, the cooling loop takes the heat away, and the battery management unit lowers the charge power when the cells get too hot. With a liquid chiller the charge is short and cool; with air cooling the cells stay below their limit only because the power is cut, and the charge takes too long.

## What to look at

- The two cooling configurations and the requirement each one must meet.
- `physics/FastChargePhysics.mo`: the pack's thermal model and the derating rule.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

Simulation needs OpenModelica: on the desktop, *ArcLang: Set up OpenModelica*; on arclang.fr, sign in and the simulations run on the server. The production gate waits for the simulations: until `arclang verify` has run the cases, it reports them inconclusive.

From the command line, at the root of the repository:

```bash
arclang verify examples/automotive/battery-fast-charge/battery_fast_charge.arc
```

## To understand by changing it

- Compare the charge time of the two configurations in the report, then tighten the time requirement and see which one fails.

## Files

- `battery_fast_charge.arc`
- `physics/FastChargePhysics.mo`
