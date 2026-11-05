# The realization ladder

A requirement is realized in stages, and each stage can say something about
it: the simulation of the architecture in OpenModelica (`arclang verify`),
the generated C run back to back with the model (`arclang sil`), the same C
on the target's processor (`arclang pil`), the machines allocated to
programmable logic in VHDL (`arclang hdl`), the processor and the logic
together as one SoC in simulation (`arclang soc`), the bitstream on the
board (`arclang soc --port`), and the digital twin on measurements from the
field (`arclang twin`). Each of these commands keeps its verdict in
`.arclang/sim-cache/` beside the model, keyed by what it depends on.

The **realization ladder** reads those verdicts and shows, per requirement,
what each stage says, whether that is still true of the model as it is,
and what to run next. Nothing runs when the ladder is computed: `check`,
the tables and the workbench only read what the commands left.

```text
Realization (what each stage says of each requirement; ✓ fresh pass · ✗ fail or error · ~ stale (changed since) · ○ never run · – not a stage of this requirement):
  SYS-PW-001 "Window closes on command"  simulation – · SIL ✓ · PIL ✓ · HDL ✓ · SoC ~ · board ○ · twin –
      next: arclang soc power_window.arc --test TC-PW-001
  SYS-PW-003 "Pinch stops the motion"  simulation – · SIL ✓ · PIL ○ · HDL ○ · SoC ○ · board ○ · twin –
      next: arclang pil power_window.arc --test TC-PW-002
```

## The stages

| Stage | Produced by | Applies to a requirement when |
|---|---|---|
| `simulation` | `arclang verify` | a case verifying it has `simulate:` |
| `sil` | `arclang sil` | a case verifying it has `execute:` |
| `pil` | `arclang pil` | a case verifying it has `execute:` |
| `hdl` | `arclang hdl` | an executed case, and a machine of the model is allocated to programmable logic (a node of kind `fpga`) |
| `soc` | `arclang soc` | as `hdl` |
| `board` | `arclang soc --port <serial port>` | as `hdl`: the SoC's bitstream on the board the node names (`board:`) or that its part implies |
| `twin` | `arclang twin` | a twin verifies it |

A stage that applies to no case of the requirement is not a stage of that
requirement (`–`, `n/a`).

## The state of a stage

Each case (or twin) of the requirement contributes the verdict of its run at
that stage, as the cache keeps it, in one of these states:

- **pass**: a fresh verdict that passes (the key of the cached run still
  matches the model, its sources, the case and the target);
- **fail** or **error**: a fresh verdict that fails, or a run in error;
- **stale**: a verdict whose key no longer matches: the model, its Modelica
  sources, the generated C or VHDL, the case, the target package, the
  measurements or the twin changed since the run;
- **missing**: never run.

The requirement's state at a stage is the worst over its cases, in the
order error, fail, stale, missing, pass: a requirement passes a stage only
when every case of it has a fresh passing verdict there.

`next` is the first stage, in the order above, that is not fresh evidence,
and the command that produces it, with the first case that needs it. A
failing stage names the case instead: the failure is the thing to look at,
not a command to run again.

## Where the ladder shows

- `arclang check model.arc`: the block above, after the warnings, when any
  requirement has a stage to climb. `arclang check --json` carries it as
  `realization`: one object per requirement with its `stages`, each with its
  `state`, its `cases` (id, state, reason, evidence) and its `target` (the
  tool, the target package, the board, the measurements).
- `arclang table`: the kind `realization`, a requirement per row, a column
  per stage (`ok`, `warn` for stale, `gap` for missing, `fail`), and `next`;
  `reached` names the highest passing stage. Declared in the model like any
  table (`table "Ladder" { kind: realization }`), rendered to Markdown or
  Excel like the others.
- The workbench snapshot (`arclang workbench`, VS Code, the MCP server):
  the `realization` field, the same objects as `check --json`.

## The rungs of a case

A case's status (`semantic_model.simulations`, by case id) keeps its
verdict as before, folded over the stages for `check`, `gate` and the
coverage table: a fresh PIL, HDL or SoC failure is the case's verdict, a
fresh pass is evidence. The stages are also kept apart, as `rungs`, each
with its `stage`, `verdict`, `state` (`fresh`, `stale`, `missing`),
`target`, `reason` and `evidence`: that is what the ladder reads.

## The twin's cache

`arclang twin` now keeps each twin's verdict in `.arclang/sim-cache/` as
`<twin>.twin.json`, keyed by the generated Modelica package, the sources,
the measurements file and the twin as written. A twin that is inconclusive
(no OpenModelica, a live twin without a recording) is not cached. The twin
is the last rung of the requirements it verifies: evidence from the field,
read like the others, never a warning when it is missing.

## A run on the board

`arclang soc --port` runs the cases through the board's serial port against
the bitstream it holds; its verdict is cached as the SoC's, with `on the
board (<port>)` in its tools. The ladder reads such a verdict as the
`board` stage, a simulated SoC as `soc`.
