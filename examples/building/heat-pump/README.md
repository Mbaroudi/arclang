# Heating a house with a heat pump

The coldest day of the year in OpenModelica: does the heat pump hold 21 °C through the night?

**Level:** Advanced · **Shows:** simulation (Modelica), thermal, configurations

## What it models

The outdoor air follows its daily cycle, the walls lose heat to it, the heat pump's capacity and efficiency fall as it gets colder, and the thermostat holds 21 °C. The 9 kW unit keeps the house comfortable through the night; the 6 kW unit does not.

## What to look at

- The two heat pump configurations.
- `physics/HeatingPhysics.mo`: the house and the heat pump.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

Simulation needs OpenModelica: on the desktop, *ArcLang: Set up OpenModelica*; on arclang.fr, sign in and the simulations run on the server. The production gate waits for the simulations: until `arclang verify` has run the cases, it reports them inconclusive.

From the command line, at the root of the repository:

```bash
arclang verify examples/building/heat-pump/heat_pump_heating.arc
```

## To understand by changing it

- Lower `envelope_conductance` (200 W/K, the walls' loss: better insulation) and see whether the 6 kW unit now passes.

## Files

- `heat_pump_heating.arc`
- `physics/HeatingPhysics.mo`
