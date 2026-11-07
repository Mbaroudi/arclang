# Aircraft electrical network, generator loss

A 28 V DC network loses its generator: load shedding and the battery must carry the essential bus for 30 minutes.

**Level:** Advanced · **Shows:** simulation (Modelica), failure case, envelope sweep, requirements per expectation

## What it models

The generator feeds the bus and charges the battery, then fails after ten minutes; the bus power control unit sheds the non-essential loads and the battery alone carries the essential ones, its voltage sagging as it empties and as it gets colder. The 40 Ah battery keeps the bus above 24 V for 30 minutes; the 25 Ah battery does not. The envelope case finds where the 40 Ah battery is not enough: at -20 °C past 400 W, and at 700 W whatever the temperature.

## What to look at

- Each expectation names the requirement it checks, so a sagging bus does not hide that the load shedding works.
- The envelope (sweep) case and its grid of verdicts.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

Simulation needs OpenModelica: on the desktop, *ArcLang: Set up OpenModelica*; on arclang.fr, sign in and the simulations run on the server. The production gate waits for the simulations: until `arclang verify` has run the cases, it reports them inconclusive.

From the command line, at the root of the repository:

```bash
arclang verify examples/aerospace/electrical-network/aircraft_electrical_network.arc
```

## To understand by changing it

- Read the sweep's verdicts in the report and find the boundary between pass and fail.

## Files

- `aircraft_electrical_network.arc`
- `physics/ElectricalNetworkPhysics.mo`
