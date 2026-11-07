# Inspection drone hover endurance

Each drone configuration flown in OpenModelica: climb to 50 m, hold, and see how long the battery lasts.

**Level:** Advanced · **Shows:** simulation (Modelica), configurations, requirements per configuration

## What it models

The flight controller climbs to 50 m and holds; the rotors draw the power momentum theory gives for the weight they carry, the battery's own weight included. The long endurance drone must hover 25 minutes with 2 kg, the survey drone 20 minutes with 1 kg: each requirement applies to its configuration only, and the gate wants a passing case in each.

## What to look at

- Requirements that apply to one configuration only.
- `physics/DronePhysics.mo`: rotor power from momentum theory.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

Simulation needs OpenModelica: on the desktop, *ArcLang: Set up OpenModelica*; on arclang.fr, sign in and the simulations run on the server. The production gate waits for the simulations: until `arclang verify` has run the cases, it reports them inconclusive.

From the command line, at the root of the repository:

```bash
arclang verify examples/aerospace/drone-endurance/drone_endurance.arc
```

## To understand by changing it

- Add 0.5 kg of payload to the survey drone and run the cases again.

## Files

- `drone_endurance.arc`
- `physics/DronePhysics.mo`
