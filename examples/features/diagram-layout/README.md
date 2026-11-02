# Diagram layout demo

A small cruise control model built to exercise the diagram layout: many ports, nesting, fan-in and fan-out.

**Level:** Beginner · **Shows:** diagrams, layout, ports

## What it models

Components with many ports on both sides, a nested component and exchanges that fan in and out. A complete model all the same: every component performs functions, every requirement is satisfied and verified.

## What to look at

- How the workbench places ports and routes exchanges.
- The layout file the workbench keeps beside the model once you move boxes.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang workbench examples/features/diagram-layout/elk_advanced_demo.arc
arclang diagram examples/features/diagram-layout/elk_advanced_demo.arc -f elk -o graphs.json
```

## To understand by changing it

- Move boxes in the workbench, close it and open it again: your layout is kept in `elk_advanced_demo.layout.json`.

## Files

- `elk_advanced_demo.arc`
