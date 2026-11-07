# Tactical ISR drone

Air vehicle, ground station and datalink, written to pass the production gate under three standards.

**Level:** Advanced · **Shows:** five Arcadia levels, user-defined types, units, chains with latency, HARA for three standards, cybersecurity, ICD

## What it models

A textbook, unclassified example: the numbers are credible, none comes from a real program. Operational analysis (commander, pilot, payload operator, ATC, maintainer), three functional chains with latency budgets (lost-link return to base within 10 s, sensor-to-screen video within 500 ms, geofence hold), 17 requirements each satisfied and verified, DAL B flight control and DAL C mission system, physical nodes typed through reusable `type`s, RF and avionics links with a complete ICD, a data model, two state machines, two scenarios with fragments, a HARA that passes the DO-178C, ISO 26262 and IEC 61508 gates, FMEA lines and a cybersecurity analysis (jamming, GNSS spoofing, datalink hijack, payload exfiltration).

## What to look at

- The reusable `type`s and the attributes they require of every instance.
- The three chains and their latency budgets.
- The cybersecurity analysis on declared scales.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate --standard do178c examples/defense/isr-drone/isr_drone.arc
arclang workbench examples/defense/isr-drone/isr_drone.arc
```

## To understand by changing it

- Run the gate with each of the three standards and compare what each checks.

## Files

- `isr_drone.arc`
