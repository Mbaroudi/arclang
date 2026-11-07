# Adaptive cruise control, after a Capella import

What a Capella import gives, and what the engineer adds in the first edit pass.

**Level:** Beginner · **Shows:** Capella import, traces, first edit pass

## What it models

The importer brings the requirements, the logical components and the traces; the engineer adds the operational need, the functions the components perform and the test cases. The complete model is `automotive/adaptive-cruise/adaptive_cruise_control.arc`.

## What to look at

- The parts the importer wrote and the parts added by hand.
- The trace matrix.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang trace examples/capella/acc-from-capella/acc_from_capella.arc --validate --matrix
```

## To understand by changing it

- Run the gate before and after adding a test case to a requirement.

## Files

- `acc_from_capella.arc`
