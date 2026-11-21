---
name: engineering-verdicts
description: Use when a part of an ArcLang model (a circuit, a printed part, a chip block, an optical transform) must be judged by an engine, or when the verdicts of such a model must be read, refreshed or explained. Covers realized_by, limits, the arclang_engineering tool and what a verdict may be said to prove.
---

# Engineering verdicts

A node of a physical architecture says what realizes it and what must hold of it:

```arc
node "LED board" {
  id: "PN-LED"
  realized_by: ["circuit:led.kicad_sch"]
  limits: ["current(R1) < @CAT-LED-R5.max_forward_current"]
}
```

`realized_by` names the file the part is made from, prefixed by what reads it
(`circuit:`, `pcb:`, `part:`, `asic:`, `photonics:`). `limits` are what must hold. A bound
written `@ENTRY.field` comes from the registry (a catalog the model imports), with its
source, version and unit; never type a number the registry holds.

The engines belong to the **Enterprise edition**. The `arclang_engineering` tool calls
`arclang-server engineering`; without it the answer says `UNAVAILABLE` and nothing was run.

## The order

1. `arclang_engineering action=connectors`: what can judge, with which tool, and whether
   its engine is built. Never assume an engine is there.
2. Write the node. Run `arclang_validate` first: a model that does not compile is not judged.
3. `action=model`: runs the engines and keeps the verdicts beside the model.
4. `action=status`: are the kept verdicts still fresh against the model as it is. Runs nothing.
5. `action=credibility`: what each verdict rests on (see the skill `engineering-credibility`).
6. `action=impact` with `targets` or `since`: what a change reaches (units, requirements, cases).
7. `action=size` (a parameter over a range) and `action=trade` (materials of the registry)
   propose a value; the engine then judges it. The model is not changed by either.

## How to read the first line of an answer

| First line | Meaning |
|---|---|
| `every limit holds` / `every verdict is fresh and passes` | the only words that allow "pass" |
| `A LIMIT DOES NOT HOLD` | at least one limit failed: report which, with the measured value |
| `STALE` | what the verdict was run on has changed: run `model` again; not a pass |
| `NOT RUN` | no kept verdict: not a pass, not a failure |
| `NOT JUDGED` | a limit was inconclusive (an engine could not decide): not a pass |
| `UNAVAILABLE` | no Enterprise edition: no verdict is known |

Repeat the measured numbers as printed. Do not round a failing value into a pass. Do not
loosen a limit to make a verdict pass: a limit changes only because the requirement changes.

## What a verdict is not

A verdict is a simulation or a calculation on a model. It is not a measurement of a
product. Say what the credibility answer says and no more.
