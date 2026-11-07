# Remote start of a vehicle

Starting the car from a phone, for ICE, hybrid and electric powertrains: safety and cybersecurity together.

**Level:** Advanced · **Shows:** ISO 26262, ISO/SAE 21434 security, requirements refinement, sixteen components on nine nodes, scenario with fragments

## What it models

Stakeholder, system, safety, security and regulatory requirements refined and traced; sixteen logical components on nine physical nodes; an ISO 26262 HARA and an ISO/SAE 21434 security analysis (assets, threats, scales); a scenario with an `alt` fragment. `WORKFLOW_DEMO.md` tells the workflow the model came from; `README_SHARING.md` how to share its explorer page.

## What to look at

- The requirements refined from stakeholder to system level.
- The security analysis: assets, threats and their scales, next to the HARA.
- The scenario and its `alt` fragment.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang gate examples/automotive/remote-start/remote_start_architecture.arc
arclang workbench examples/automotive/remote-start/remote_start_architecture.arc
```

## To understand by changing it

- Generate the design review document (*ArcLang: Generate a document*) and read how the security analysis is reported.

## Files

- `remote_start_architecture.arc`
- `WORKFLOW_DEMO.md`
- `README_SHARING.md`
- `remote_start_diagram.svg`
