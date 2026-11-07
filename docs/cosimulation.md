# Co-simulation in OMSimulator

`arclang cosim` runs a model's executed test cases as a standard FMI
co-simulation, in **OMSimulator**, OpenModelica's FMI and SSP master, and
judges the run as `arclang sil` judges the C: the case's `expects:` on it,
and its trace step by step against the model.

```text
arclang cosim model.arc                    every case that execute:s
arclang cosim model.arc --test TC-CHG-HIL  one case
arclang cosim model.arc --keep build/cosim the SSP, the result and the logs kept
arclang cosim model.arc --json             the outcomes as JSON
```

The point is that the behavior now runs **as an FMU, under a master that
is not ArcLang's**. The machine is exported as an FMI 2.0 co-simulation
FMU (`export -f fmu`), its plant as another FMU (OpenModelica's, or the
one the case gives, from Simulink, Dymola, Amesim…), and the two are
connected in an SSP package (System Structure and Parameterization 1.0)
that any SSP tool opens. What ArcLang proves on its own loop is then
proven again on the standard exchange.

## What a case becomes

For each case with `execute:` (docs/embedded-c.md, "Test cases executed
at fixed step"), ArcLang writes `cosim.ssp`:

| In the SSP | What it is |
|---|---|
| `resources/controller.fmu` | the behavior, `export -f fmu`, compiled with `--cc` (else `$CC`, else `cc`) |
| `resources/plant.fmu` | the case's `plant:`: a Modelica plant built by OpenModelica (FMI 2.0 CS, CVODE), or the `fmu:` it gives |
| `resources/stimuli.csv` | the case's `inputs:` as a table OMSimulator reads, one column per input of the FMU |
| `SystemStructure.ssd` | the three components, their connectors, the connections, the master's fixed step (the machine's period) and the stop time |

The table drives the controller's inputs that the plant does not drive.
The plant's outputs drive the machine's `in` variables named in
`outputs:`, and the machine's variables named in `inputs:` drive the plant.
OMSimulator runs the package at the machine's period, and the result is
read back at the end of each step.

A Modelica plant is wrapped (`ArcLangCosimPlant`) so that each wired
variable is an FMU input or output **in the machine's units**: OMSimulator
does not apply an SSP connection's linear transformation, so the wrapper
converts to and from SI itself. A given FMU is taken as it is. Its wired
variables must be inputs and outputs (an FMU's local variable cannot be
connected), and the machine variables wired to them must be declared in SI
units, because nothing converts along the connection.

## How the run is judged

The case's expectations are judged on the OMSimulator run and on the
model's own run. Both verdicts must agree, and the run must reproduce the
model's trace, every output at every step, within a tolerance:

- **without a plant**: what OMSimulator's result can tell apart. It prints
  12 significant digits, so the tolerance is 1e-9 of the largest value in
  the case, rounded up to a power of ten;
- **with a plant**: a hundred times the plant's tolerance (`1e-6` by
  default), relative to the largest value. The same plant FMU's CVODE runs
  under both masters: each run is right to its tolerance locally, but their
  internal steps differ, most after a jump of an input. OpenModelica's FMU
  integrates at the model's own tolerance, whatever the master asks at
  setup.

**The plant gets the controller's outputs one period late.** OMSimulator
steps every FMU from the same instant (a Jacobi master): over
`[t_k, t_k+1]` the plant is driven by what the controller gave at `t_k`,
which is the step before. ArcLang's own loop steps the machine first, then
the plant on the machine's new outputs. Neither is wrong, and the late one
is how a digital controller behaves when its command goes out at the next
tick. A case with a plant is therefore compared with the model's loop held
the same way (`plant::run_held`). Its expectations are still judged on the
model's own loop: a requirement that holds only without the delay shows as
a disagreement between the two verdicts.

The outcome says `OMSimulator 2.1.3; the behavior's FMU (…), plant …
built by OpenModelica …` in its tools.

## Measured

On the examples, in the Studio image (arm64):

| Model | Case | Steps × outputs | Largest difference |
|---|---|---|---|
| `automotive/power-window` | TC-PW-001, TC-PW-002 | 16 × 21, 13 × 21 | 0 |
| `automotive/adaptive-cruise` | TC-ACC-SIL-001 | 101 × 6 | 0 |
| `energy/battery-charger` | TC-CHG-001, TC-CHG-003 | 61 × 34 | 5e-11 |
| `automotive/speed-monitor` (data flow) | TC-SPD-001 to 003 | up to 101 × 9 | 5e-10 |
| `energy/battery-charger`, Modelica plant | TC-CHG-HIL | 151 × 34 | 1.5e-3 (tolerance 1e-2) |

On TC-CHG-HIL, every expectation holds on both runs. That includes
precharge at 2 A, bulk from 300 ms and absorption at 1 s, which prove
SYS-CHG-004 and SYS-CHG-005. The largest differences come right after the
setpoint jumps from 2 A to 9 A.

The cases that go through an `io_binding` are skipped (`– skip`): the FMU
exchanges the machine's variables, not its channels, and `arclang sil`
proves the channels.

## Where OMSimulator comes from

- On this computer: `OMSimulator` on the `PATH`, or the program named by
  `ARCLANG_OMSIMULATOR`. A Modelica plant also needs `omc`
  (docs/modelica.md).
- Otherwise the command runs in ArcLang Studio (docs/studio.md), which holds
  OMSimulator next to OpenModelica, as `pil`, `hdl` and `soc` do. Set
  `ARCLANG_EMBEDDED_DOCKER=off` to never relay.

OMSimulator's FMI loader looks for an FMI 2.0 binary in `linux64` on
x86_64 Linux, and in `linux32` on any other Linux processor (aarch64
included). On aarch64 Linux, each FMU put in the SSP therefore also gets
its binaries, and the libraries beside them (OpenModelica's SUNDIALS),
under `binaries/linux32`. The FMU `export -f fmu` writes is not changed.

A CSV table is interpolated linearly by OMSimulator, and its master's
instants carry rounding (0.1 s is read at 0.0999…). The stimuli table
therefore holds each step's value flat a quarter of a period on each side
of its instant, and puts the ramps between instants, where the master
never reads. An event is then read as exactly 1 or 0, never as `1e-15`.

## What is not done

- Cases through an `io_binding` (see above).
- Wires to the plant other than reals: a co-simulation wire carries a Real.
- A given plant FMU with its own units: the machine variables wired to it
  must be in SI.
- The verdict is not yet a rung of the realization ladder
  (docs/realization.md).
- An FMI 3.0 export of the behavior.
