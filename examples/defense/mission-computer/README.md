# Mission computer of a combat aircraft

Radar and electronic warfare fused into tracks, threats ranked, and a weapon released only with two operators.

**Level:** Intermediate · **Shows:** DO-178C, DAL A path, failure conditions, state machine, scenario

## What it models

Radar and electronic-warfare data are fused into tracks, threats are ranked, and a weapon is released only after a dual-operator authorization. Airborne software levels follow DO-178C; the weapon release path is DAL A.

## What to look at

- The release state machine: both operators must authorize.
- The hazards classified by failure condition.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate --standard do178c examples/defense/mission-computer/mission_computer.arc
```

## To understand by changing it

- Run *Weapon release states* in the debugger: fire `LF-012`, then `LF-013` with `pilot consent` true, then `LF-014` with `WSO consent within 5 s` false: nothing is released until the second operator consents.

## Files

- `mission_computer.arc`
