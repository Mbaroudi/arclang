# Typed quantities and constraints

Units, user-defined types and constraints: the compiler checks dimensions, the gate checks the assertions.

**Level:** Beginner · **Shows:** typed quantities, user-defined types, specialization, constraints

## What it models

A reusable definition requires every periodic task to state its worst case; a safety ECU is an ECU with an integrity level (specialization). Constraints over chains and links (sums, maxima, bus load) are evaluated by the gate.

## What to look at

- `type` definitions and their required attributes.
- Constraints that sum latencies and check a bus load.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang check examples/features/timing-constraints/timing_constraints.arc
arclang gate examples/features/timing-constraints/timing_constraints.arc
```

## To understand by changing it

- Write `10 kg` where a duration is expected: the compiler reports the wrong dimension.
- Remove the worst case of a periodic task: its type requires it.

## Files

- `timing_constraints.arc`
