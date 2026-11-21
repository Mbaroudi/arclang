---
name: engineering-credibility
description: Use when asked how far to trust an engineering verdict of an ArcLang model, or before a verdict is quoted in a document, a review or a customer answer. Reads the credibility record of each kept verdict and says what it supports and what it does not.
---

# Credibility of a verdict

A pass printed by an engine is a number against a limit. How far to trust it is a separate
question, answered by `arclang_engineering action=credibility` (optionally with `id=PN-...`).
It answers six questions for each kept verdict, by what actually ran. Read all of them.

1. Is the record fresh? A stale or missing record (`! the record is stale or missing`)
   means the numbers describe an earlier model.
2. What did the verdict rest on? `action=artifacts` lists the files and the runs, with hashes.
3. Where do the bounds come from? A bound taken from the registry has a source and a unit; a
   number typed in the limit has neither.
4. Was a second solver or method used? A part judged by two solvers that agree within the
   tolerance is stronger than one judged by one. Where they disagree the answer is
   `inconclusive`, and the gate refuses it: never average them.
5. Did the engine check itself? Engines run credibility checks before a figure becomes a
   verdict (for the optical transform: ideal equality to 1e-9 and agreement with the
   reference coupler to 1e-12). A verdict whose check failed is not a verdict.
6. What was left out? Read the `data` lines and any `UNSOURCED` mark: a limit written in
   place, without a source, is said so.

## What to write in a document

- Quote the measured value with its unit, the limit, and the engine's name.
- Say "simulated" or "calculated", never "measured" or "tested", unless a measurement exists.
- Say what the model leaves out (for a circuit-level optical model: dispersion, crosstalk,
  detector noise; for a part: contact, fatigue, anything the loads do not state).
- If the record is stale, say so and run `action=model` before quoting.
