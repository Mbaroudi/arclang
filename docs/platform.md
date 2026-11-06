# ArcLang as an engineering platform

ArcLang began as MBSE as code: one typed, version-controlled system model
(`.arc`), compiled, checked and diagrammed. It has grown, stage by stage,
into a model-based engineering chain where the same model drives the
physics, the embedded software, the programmable logic, the co-simulation,
the verification and the digital twin. This page is the map: what the
identity is, how the compiler is layered, which domain each construct of the
language covers, and what is still to come. It is written for whoever adds
a stage: the rules below are what keeps the chain one thing.

## The identity

- **One model.** Requirements, architecture (Arcadia or SysML v2), behavior
  (`state_machine`), physics (`simulate:` on Modelica sources), targets
  (`node`, `io_binding`), cases (`test_case`) and twins (`twin`) live in the
  same typed text, under git. Nothing is kept in a tool's database.
- **The compiler never lies.** Every verdict comes from a deterministic
  tool run on generated artefacts, keyed by what it depends on, cached
  beside the model and read back as evidence (`check`, the tables, the
  realization ladder). A stale verdict is said to be stale.
- **Deterministic tools behind stable interfaces.** OpenModelica behind
  Modelica and FMI, GCC/Clang/Zephyr behind C, GHDL/Yosys/nextpnr/LiteX/
  Verilator behind VHDL, the board behind its serial port. Each runs on the
  host or in its Docker image; the model does not know which.
- **The assistant proposes, the compiler decides.** The AI agent (VS Code,
  MCP) writes and edits the model and runs the commands; it never issues a
  verdict of its own.

## The layers of the compiler

```text
.arc ──► parser ──► semantic model (typed: units, dimensions, ranges, references)
                         │
          ┌──────────────┼──────────────────┬───────────────────┐
          ▼              ▼                  ▼                   ▼
   Modelica backend   C backend         VHDL backend        FMI backend
   compiler/modelica  embedded/cgen     embedded/vhdl       embedded/fmu
   (OpenModelica)     (GCC, Clang,      (GHDL, Yosys,       (FMI 2.0 CS,
                       Zephyr, Renode)   nextpnr, LiteX)     fmpy-checked)
          │              │                  │                   │
          └──────────────┴────────┬─────────┴───────────────────┘
                                  ▼
                     verification (compiler/simulation, embedded/{sil,pil,hdl,system,plant})
                     verify · sil · pil · hdl · soc · twin — verdicts cached by key
                                  ▼
                     realization ladder (compiler/realization) and `arclang realize`
                                  ▼
                     deployment: bitstream on the board (soc --port), the twin on live data
```

The orchestration is in the compiler, in Rust, not in a script: `arclang
realize` climbs the ladder by running the same binary stage by stage, and
an agent drives it through `arclang_realize`. Python is the agent layer
(the MCP server), never the judge.

## The domains and their constructs

| Domain | In the language | Tools behind it |
|---|---|---|
| System architecture | `requirements`, `operational_analysis` … `physical_architecture`, `allocations`, `interfaces`, SysML v2 import/export | the compiler, Capella, SysML v2 |
| Physical model | `simulate: { model: … sources: […] }`, `twin`, `plant:` of an executed case | OpenModelica, Modelica Standard Library, FMI 2.0 and 3.0 CS (import), FMI 2.0 CS (export) |
| Embedded software | `state_machine` (regions, pseudo-states, `equations:`), `io_binding` (channels, period, watchdog, `target:` package), `node` with `processor:` | C generation, MISRA check, Zephyr, Renode |
| Digital hardware | `node` with `kind: "fpga"`, `device:`, `clock:`, `deploys`; fixed point from `range` | VHDL generation, GHDL, Yosys, nextpnr, LiteX, Verilator, the ULX3S |
| Communication | the channels of `io_binding`: gpio, adc, dac, pwm, can, uart | the target package's drivers, Renode's peripherals |
| Execution and verification | `test_case` with `simulate:`, `execute:` (and its `plant:`), `expects:`, sweeps, Monte Carlo, `gate` | `verify`, `sil`, `pil`, `hdl`, `soc`, the ladder |
| Digital twin | `twin` (`data:` file or live, `drives:`, `compares:`, `advance: "step"`) | OpenModelica as an FMU, MQTT/OPC UA/Kafka/HTTP |
| AI engineering agent | the VS Code chat tools and the MCP server (`arclang_edit`, `arclang_realize`, …) | Claude, any MCP client |

No new top-level block is needed for a domain: a processor or a logic
device is a `node` of the physical architecture, a firmware is what an
`io_binding` binds to a target package, a co-simulation is an executed case
with a plant. Adding a domain means adding a backend and a stage, not a
language.

## Where it runs

Everything above runs the same on a laptop, on a server or in Codespaces:
ArcLang Studio is one image with VS Code, the extension, the compiler and
every tool ([studio.md](studio.md)). The compiler on a computer without the
tools runs the embedded stages in that image.

## What is next, in order

1. ~~**HIL in simulation**~~ — done: the plant in the loop of `sil` and
   `soc` (`execute: { plant: … }`), Modelica or an FMU from another tool;
   `hdl` and `pil` replay the loop; the board through its port.
2. ~~**HW/SW partitioning**~~ — done: `arclang partition`, the same machine
   as C and as VHDL on the real tools, the table kind `partition`.
3. ~~**Tasks and scheduling**~~ — done: one task per `io_binding` in the
   exported application, `arclang schedule` on the step times measured,
   `--prove` running every task together in Renode, each period proved
   kept under load.
4. ~~**FMI 3.0 import**~~ — done: a plant's `fmu:` (and any FMU the
   stepper runs) is FMI 2.0 or FMI 3.0 co-simulation, from any tool,
   stepped through the C API by the same stepper. The export stays FMI 2.0
   until a consumer needs 3.0.
5. ~~**Dataflow outside machines**~~ — done: a machine with equations and
   no state is a data-flow block, and `in x: ... from "M".y` wires machines.
   The chain runs in the model, in the C (SIL, PIL), and in VHDL block by
   block, with each input's range taken from what it reads.

What is *not* planned: a general-purpose language, real-time HIL (the loops
here run in lock step, reproducible to the bit), a dependency on any
proprietary tool (Simulink enters as an FMU, nothing more).
