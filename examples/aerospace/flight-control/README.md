# Fly-by-wire flight control

Pilot inputs to control surfaces through two dissimilar computers, with what a DO-178C / ARP4754A review asks for.

**Level:** Intermediate · **Shows:** DO-178C, DAL, functional hazard assessment, ICD on every link, modes, EPBS

## What it models

Two dissimilar flight control computers turn pilot inputs into control surface deflections. DAL on every element, a functional hazard assessment with the failure conditions, an ICD on every ARINC 664 link, the channel switchover modes and the EPBS.

## What to look at

- The failure conditions and the DAL they impose.
- The ICD attributes on the physical links: the gate checks they are complete.
- The switchover modes between the two channels.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate --standard do178c examples/aerospace/flight-control/flight_control_system.arc
arclang workbench examples/aerospace/flight-control/flight_control_system.arc
```

## To understand by changing it

- Delete the `protocol` of the link *ARINC 664 Network* and run the DO-178C gate: the ICD of that link is no longer complete.

## Files

- `flight_control_system.arc`
