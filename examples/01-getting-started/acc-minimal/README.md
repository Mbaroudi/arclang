# Adaptive cruise control, the starter

One capability followed through the four Arcadia levels: the smallest complete model, and the one to read first.

**Level:** Beginner · **Shows:** Arcadia levels, requirements, traces, hazard, test cases, production gate

## What it models

An adaptive cruise control reduced to its core: the driver's need to keep a safe distance, the system functions that serve it, three logical components on two ECUs, three requirements, one hazard and the three test cases that verify the requirements. Every element is traced, so the model passes the production gate.

## What to look at

- `operational_analysis`: the driver, the ego vehicle and the capability *Keep a safe following distance*.
- `system_analysis` and the logical architecture: the functions, and the components that perform them.
- `requirements` with their ASIL, `trace … satisfies …` and the `test_case`s that verify them.
- The hazard: its ASIL must agree with its severity, exposure and controllability (the ISO 26262 table).

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang check examples/01-getting-started/acc-minimal/acc_minimal.arc
arclang gate examples/01-getting-started/acc-minimal/acc_minimal.arc
arclang workbench examples/01-getting-started/acc-minimal/acc_minimal.arc
```

## To understand by changing it

- Delete one `trace … satisfies …` line, then run the gate: it names the requirement left without an element that satisfies it.
- Set the hazard's `asil` to `ASIL-D`: the gate recomputes it from S3, E4 and C2 (ASIL-C) and blocks the contradiction.
- Open the workbench and add a component from the palette: the text is edited for you, and *Undo* takes it back.

## Files

- `acc_minimal.arc`
