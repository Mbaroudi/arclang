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

## Climbing it: `arclang realize`

```text
arclang realize model.arc                       # every requirement, every stage but the board
arclang realize model.arc --requirement REQ-1   # one requirement
arclang realize model.arc --up-to sil           # stop at SIL
arclang realize model.arc --dry-run             # the plan and the gaps, nothing runs
arclang realize model.arc --keep-going          # past a failure, every step that can run
arclang realize model.arc --board               # the bitstream on the board too (needs --port's hardware)
arclang realize model.arc --json                # the plan, each run's output, the ladder after
```

The plan is the ladder's: every stage that is missing or stale, in the
order of the stages, each case once per stage (one step climbs every
requirement its case verifies). Each step runs the command that produces
the stage — `verify`, `sil`, `pil`, `hdl`, `soc`, `twin` — as the same
binary, streaming its output; then the model is compiled again and the
stage read back for the requirements concerned:

```text
→ SIL (missing): arclang sil power_window.arc --test TC-PW-001
  ...
  SYS-PW-001 SIL: ✓ pass
  SYS-PW-002 SIL: ✓ pass
→ PIL (missing): arclang pil power_window.arc --test TC-PW-001
```

A stage that fails stops the climb of its requirements (`the climb of
SYS-PW-002 stops here`): the case is the thing to look at, not a command to
run again; `--keep-going` runs the other steps anyway. A tool that is
absent (no Docker, no Zephyr image) fails its step the same way, with the
command's own message. The command succeeds when every stage that applies
to the chosen requirements is fresh evidence after the climb.

A **gap** is a requirement nothing climbs: no case simulates or executes
for it and no twin verifies it. `realize` names it and says what would
climb it (a `test_case` with `simulate:` or `execute:`, a `twin`); it never
writes one. The assistant's `arclang_realize` tool (VS Code chat, the MCP
server) returns the plan and the gaps, so an agent can propose the missing
cases through `arclang_edit` and run the climb.

## The stages through Docker

`arclang pil`, `hdl` and `soc` need toolchains a host rarely has (Zephyr
and Renode; GHDL and Yosys; LiteX and Verilator). The images under
`tools/embedded` hold them: `arclang-embedded-soc:1`, `arclang-embedded-fpga:1`,
`arclang-embedded-system:1`. When this host lacks a stage's tools and Docker
has the stage's image, the command runs itself in the image, the model's
directory mounted at `/work` (a model that imports files outside its
directory does not reach them there), and its verdict lands in the model's
cache as if it had run here: the ladder, `check` and `realize` read it the
same way. The command says so first:

```text
pil: no west, renode on this host: running in the Docker image arclang-embedded-soc:1 (/work/models/window mounted at /work)
```

`ARCLANG_EMBEDDED_DOCKER` names another image (for every stage), or `off`
keeps the commands on the host. A board on a serial port (`soc --port`)
always stays on the host. The arclang the image runs is the one its build
loop left in the `arclang-embedded-target` volume (`tools/embedded/<image>/run.sh
cargo build --release`), else one on the image's PATH.

## A case with a plant

A case closed on a plant (`execute: { plant: … }`, docs/embedded-c.md)
climbs like any other: SIL runs the C in the loop, PIL and HDL replay the
loop, the SoC runs in it. Its plant is built once as an FMU beside the
model (`.arclang/plant/`), on this host, before a stage relays to a Docker
image: the image steps the FMU, it does not build it.

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
