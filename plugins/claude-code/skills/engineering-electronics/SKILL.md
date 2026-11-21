---
name: engineering-electronics
description: Use when a circuit or a printed circuit board is part of an ArcLang model: a KiCad schematic or board realizing a node, current or voltage limits, worst-case tolerances, KiCad rules (ERC, DRC), a value proposed from a standard series, or a twin on a real board.
---

# Electronics as code

A circuit is a KiCad schematic named by the node: KiCad reads it, ngspice simulates it, and
a measurement (`current(R1)`, a node voltage) is compared with the limit.

```arc
node "LED board" {
  id: "PN-LED"
  realized_by: ["circuit:led.kicad_sch", "pcb:led.kicad_pcb"]
  limits: ["current(R1) < @CAT-LED-R5.max_forward_current"]
}
```

- Bounds of a component (maximum current, voltage) come from the registry
  (`@CAT-...`), not from a typed number.
- A worst case is written on the node: `tolerances: ["R1 +- 5 %", "V1 +- 5 %"]`. The engine
  simulates every corner of the box (each named component at its value times 0.95 and 1.05:
  4 runs for two components, 256 at most for eight) and judges the limit on the worst one.
  The nominal circuit can pass where the worst case fails (9.4 mA nominal, 10.67 mA worst,
  against 10 mA): report the range the verdict keeps, not the nominal value. The corners
  are exact only where the value moves monotonically with each component; an extremum
  inside the box is missed. A tolerance on something with no plain value to vary (a diode)
  is `inconclusive`, never a pass. Changing a tolerance makes the record stale.
- `erc(errors)`, `erc(warnings)`, `erc(violations)` and the `drc(...)` equivalents are
  counts of KiCad's own rule checks, compared with `==`, `<=` or `<` and no unit. A count
  of findings is a rule, not a datum: it has no registry entry.
- A series value can be proposed (E24, E96, over a range, each simulated and judged: the
  `arclang_engineering action=propose-value`: `part`, `component`, `series` (default E24),
  `from`, `to`, at most three decades); the engine proposes, the limits decide, and the
  model is not changed: write the value yourself, then run `action=model`. A twin's
  physics parameter can be fitted to its measurements with `action=calibrate` (`twin`,
  `parameter`, `from`, `to`, optional `source`): a fit is a candidate, not a verdict.

Workflow: `arclang_engineering action=connectors` (is the circuit engine built?), then
`action=model`, then read the first line before anything else. See `engineering-verdicts` and
`engineering-credibility`.
