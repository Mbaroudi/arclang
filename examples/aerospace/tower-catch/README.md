# A tower catches a returning vehicle

The last 1,800 m of a heavy vehicle coming home: belly flop, flip, landing burn, and two arms that catch it.

**Level:** Intermediate · **Shows:** simulation (Modelica), requirements on a physical event, a state machine of phases, a what-if that fails the gate

## What it models

The vehicle falls flat until the drag balances its weight (about 67 m/s), turns upright in 6 s, relights its engines when the braking they must give reaches their lowest thrust (they cannot throttle below it), follows the thrust that brings its speed to zero at the height of the arms, and is cut below 1 m/s. Two fittings then hang on two arms, a spring and a damper, and share the weight. The physics is in `physics/CatchPhysics.mo`; the phases are a state machine.

The numbers are round and chosen for the example: they are not those of any real vehicle or tower.

## What to look at

| Requirement | Verdict |
|---|---|
| Speed at the arms at most 3 m/s | 1.26 m/s |
| Load per fitting at most 1,200 kN | 814 kN at the contact |
| Weight shared, 450 to 550 kN each | 497 kN |
| Landing propellant at most 7 t of the 8 t kept | 6.6 t |
| Engines never above 2,600 kN | 1,863 kN |
| A vehicle 15 % heavier, same limits | 2.2 m/s, 1,042 kN |

The weight on the arms is half of `m g` each, 497 kN, but the contact is harder: the peak is 814 kN. The requirement is on the peak.

## A what-if that fails

In `CatchPhysics.mo`, `FlipTooLow` starts the flip at 400 m instead of 1,100 m. Point the case `TC-DESIGN` at `CatchPhysics.FlipTooLow` and `arclang verify` says why the gate fails: the engines never relight in time, the vehicle reaches the arms at 72 m/s and the fittings would carry 13.9 MN.

## Try it

Simulation needs OpenModelica (on the desktop, *ArcLang: Set up OpenModelica*; or Docker with `ARCLANG_OMC_DOCKER`). From the root of the repository:

```bash
arclang verify examples/aerospace/tower-catch/tower_catch.arc
arclang gate --standard do178c examples/aerospace/tower-catch/tower_catch.arc
```
