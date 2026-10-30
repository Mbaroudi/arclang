# Modelica and OpenModelica: requirements verified by simulation

ArcLang exports the physical architecture of a model to a Modelica package,
and verifies requirements by simulating it in OpenModelica. The verdict of
each simulation goes where every other verdict goes: the production gate
(`requirements.verification`) and the `requirements_coverage` table.

The worked example is
[`examples/automotive/ev_range_simulation.arc`](../examples/automotive/ev_range_simulation.arc)
with its physics in
[`examples/automotive/physics/EVPhysics.mo`](../examples/automotive/physics/EVPhysics.mo):
a winter-range requirement that the long range pack meets and the standard
pack does not.

## Commands

| Command | What it does |
|---|---|
| `arclang export model.arc -f modelica -o out/` | Writes `out/<Package>.mo`, the structure of the physical architecture, drawn (see below). Notes what is not exported. |
| `arclang export model.arc -f modelica -o out/ --split` | The same package as one file per class in an `out/<Package>/` folder (a Modelica library): what Modex browses. |
| `arclang export model.arc -f modelica -o out/ --no-placement` | The structure alone, with no drawing: the text `arclang verify` simulates. |
| `arclang verify model.arc [--test TC-ID] [--json]` | Exports to a temporary directory, runs OpenModelica on every case that `simulate:`s (or one), reads the result, evaluates `expects:`, prints the values (each with its margin: `short by 0.07341`, `19.91 km to spare`), the requirements, configuration and model of each case and the time taken, and caches the verdicts. `-v` also prints what OpenModelica printed. Exits non-zero unless every case passes. |
| `arclang check model.arc` | Never simulates. Reads the cache: a fail, a stale or a missing verdict is a `verification:` warning. |
| `arclang gate model.arc` | Never simulates. A failing simulated case is a blocker; so is one whose verdict is stale or missing (inconclusive), with the command to run. |
| `arclang table model.arc --kind requirements_coverage` | The verdict of each case, and a **Key values** column with what the simulation evaluated to. |

### Finding OpenModelica

OpenModelica is an external tool, looked for in this order:

1. `ARCLANG_OMC`: the path of a local `omc`. When set and it does not run,
   nothing else is tried.
2. `ARCLANG_OMC_DOCKER`: a Docker image to run `omc` from.
3. An `omc` on the `PATH`.
4. Docker, with the default image `openmodelica/openmodelica:v1.27.1-minimal`
   (about 290 MB compressed, 1.1 GB on disk).

With Docker, ArcLang runs `docker run --rm -v <work dir>:/work -w /work
-v arclang-openmodelica-libraries:/root/.openmodelica/libraries <image> omc run.mos`.
The model, the sources and the script are files of the working directory:
no model text goes through a shell, and the script names only class names
ArcLang has checked. The minimal image has no Modelica Standard Library:
the first run installs MSL 4.1.0 into the `arclang-openmodelica-libraries`
volume, later runs reuse it. A run is limited to 900 s (`ARCLANG_OMC_TIMEOUT`,
in seconds); a container that outlives it is removed.

When no tool is found the verdict is **inconclusive**, with the reason
(`OpenModelica not available: ...`), and is not cached: a missing tool is
never a pass.

`arclang verify --check-tools [--json]` simulates nothing: it reports what
this machine has (the tool a simulation would use, whether Docker is
installed and running, whether the image is pulled, whether the Modelica
library is in the volume, whether a local `omc` runs). The VS Code
extension reads it, so discovery stays the compiler's.

## Simulation from VS Code

Open a folder with a model that simulates (a test case with `simulate:`, or
a `physics:` port) or a `.mo` file: a **Simulation** item appears in the
status bar. Its text says where things stand: `OpenModelica (Docker)` or
`(local)` when a tool is found, `needs setup` when none is, `Simulating…`
during a run, then the verdicts of the last run (`1 fail, 1 pass`). Click it
for the menu (also *ArcLang: Simulation* in the command palette):

- **Run simulations on this model** (also the play button in the editor
  title and *Simulate* in the workbench): runs `arclang verify`, with a
  progress notification that can be cancelled; the result offers *Show the
  verdicts* (the requirement coverage table) and *Show the log*. The log is
  the **ArcLang Simulation** output channel: each run begins with its time,
  file and tool, then each case with its name, the requirements it
  verifies, its configuration and model, each value with how far it is from
  its bound, and what OpenModelica printed; it ends on the count and the
  time taken.
- **Check my setup**: a checklist of Docker installed, Docker running, the
  OpenModelica image, the Modelica library, a local `omc`, and the Modex
  extension (to draw `.mo` files), each missing item with a button: open the
  Docker Desktop download page, start Docker Desktop, download the image
  (about 1 GB, with progress), install the library now, install Modex, or
  show the exact command in a terminal (typed, not run).
- **Set up OpenModelica (guided)**: the same fixes, one step at a time.
- **Export Modelica** writes `modelica/<Package>/`, one file per class
  (`--split`), and offers to open the drawing; an earlier single-file export
  written by ArcLang beside it is removed, since OpenModelica would see the
  package twice.
- **Open the Modelica drawing** asks which configuration and opens
  `<Package>/Systems/<Configuration>.mo` in Modex, the model that holds the
  whole architecture (the package itself has nothing to draw). The first
  time, it offers to add the export folder to `modex.libraryRoots` for the
  workspace, so that Modex draws each part from its class.
- **Open the documentation**.

When you run simulations and nothing is available, the checklist opens
instead of an error. The first run with Docker installs the Modelica library
into the volume (a few minutes, once); later runs reuse it, and unchanged
cases are answered from the verdict cache.

Settings (user settings only, ignored in an untrusted workspace for the two
that name a program): `arclang.simulation.image` (`ARCLANG_OMC_DOCKER`),
`arclang.simulation.omc` (`ARCLANG_OMC`), `arclang.simulation.timeout`
(`ARCLANG_OMC_TIMEOUT`, seconds). Nothing runs in an untrusted workspace; in
the browser build the item is hidden and the commands say that simulation
needs the desktop extension. Cancelling a run stops `arclang`; a Docker
container already started may finish on its own.

## Drawing the architecture in Modex and OMEdit

Graphical Modelica editors draw from annotations. Without them the canvas is
blank, so the export writes them by default:

- `Placement` on every component instance (a node in a system model, a
  behaviour component in a node), on a layered layout: inputs and
  upstream parts on the left, outputs and downstream parts on the right,
  boxes that do not overlap, the same text every run;
- a `Line` on every `connect`, in right angles and coloured by what it
  carries: electrical blue `{0,0,255}`, signal dark blue `{0,0,127}`,
  mechanical grey `{95,95,95}`, thermal red `{191,0,0}`;
- a `Diagram(coordinateSystem(extent=...))` on every model, sized to fit
  its boxes and wires;
- on a model that is itself a component (every node and behaviour
  component), an `Icon`: a rectangle with its name, over the same extent as
  its diagram, with its connectors placed on the left (inputs, positive
  pins, `_a` flanges) or the right (outputs, negative pins, `_b` flanges).
  A pin or flange with no direction goes where the connections written in
  the model leave it (the source of a link on the right, its target on the
  left), one side per class whatever diagram draws it. The icon is the
  diagram, so a box in a parent diagram shows each connector where it is
  in the model.

The `__ArcLang(id=...)` annotations are untouched and stay beside them.
`--no-placement` writes the previous text, byte for byte. `arclang verify`
always simulates that text, never the drawn one: the annotations are
comments to the compiler, but keeping them out means a drawing change can
neither alter a verdict nor invalidate the simulation cache (the cache key
hashes the package). A layout file or a different drawing therefore leaves
the cached verdicts fresh; a change of the model still makes them stale.

If `<model>.layout.json` (the workbench layout file) sits next to the model,
the offsets it holds for the physical view (`pab`) move the boxes from where
the automatic layout puts them, 10 sheet units for one Modelica unit, the
y axis flipped (sheet down, Modelica up). A system model reads the whole
view, a node the arrangement opened on it. Wires are re-routed around the
boxes; waypoints bent by hand are not carried over.

### Opening it

- **Modex** (VS Code): use `--split` and add the folder that holds the
  package folders to `modex.libraryRoots` in the settings. Modex browses a
  package by its folders and resolves the icons of a model's components
  only for packages it knows as libraries; a single `<Package>.mo` file
  opens, but its nested models cannot be reached, and a component whose
  class Modex does not know is a plain box with the class name. Open
  `<Package>/Systems/<Configuration>.mo` for the whole architecture, a file
  of `Physical/` for the inside of a node. The first start offers to
  install the Modelica Standard Library (network needed once): the
  connectors are `Modelica.*` classes, and accept it.
- **OMEdit** (OpenModelica): it reads the same annotations from either
  layout (open `<Package>.mo` or `<Package>/package.mo`, with the Modelica
  Standard Library 4.1.0 installed). OMEdit itself was not run for this
  page; the check is `omc`, which OMEdit uses to read them:

The package loads in `omc` with both layouts, and `getNthComponentAnnotation`,
`getNthConnectionAnnotation` and `getDiagramAnnotation` return the
placements, lines and diagram extent written here.

Known limits: the layout is layered and simple, so a model with many
unconnected parts is wrapped into columns of four and a dense model has
long wires routed round the outside; the colours cover the physics
ArcLang exports (fluid ports are not exported); a long name is shrunk to
the box by the editor; offsets from a layout file place boxes but their
size changes (`dw`, `dh`) and bent edges are ignored.

## The convention: generated structure, hand-written physics

The generated package holds the structure and only the structure. Every
model in it is `partial`. The physics lives in a package of its own, which
`extends` the generated models and redeclares their parts, so regenerating
never overwrites a line of it.

```text
EVRangeSimulation                      generated, never edited
├── Behaviour.<Node>.<Component>       partial model per behaviour component
├── Physical.<Node>                    partial model per physical node
│     connectors, parameters, `replaceable` behaviour components, connect(...)
└── Systems.<Configuration>            partial model per configuration
      `replaceable` nodes, variant parameters in `constrainedby (...)`, connect(...)

EVPhysics                              hand-written
├── model Battery extends EVRangeSimulation.Physical.BatteryPack; ... equations
├── model ControlUnit extends EVRangeSimulation.Physical.VehicleControlUnit(
│         redeclare EVPhysics.SpeedControl speedController);
└── model TouringRange extends EVRangeSimulation.Systems.Touring(
          redeclare EVPhysics.Battery batteryPack, ...);
```

- **Names.** The package is the model name in UpperCamelCase
  (`EVRangeSimulation`); classes are element names in UpperCamelCase,
  instances in lowerCamelCase (`Battery pack` gives `Physical.BatteryPack`
  and `batteryPack`); ports and parameters keep their ArcLang names.
- **Traceability.** Every class, component, connector and `connect` carries
  `annotation(__ArcLang(id="..."))`, the ArcLang id it comes from (Modelica
  requires vendor annotations to start with `__`); every class has a
  `Documentation(info=...)` from its description.
- **Parameters.** Typed quantity attributes become parameters of the SI
  type of their dimension, in SI units: `mass: 2100 kg` gives
  `parameter Modelica.Units.SI.Mass mass = 2100`, `energy: 82 kWh` gives
  `parameter Modelica.Units.SI.Energy energy = 2.952e8`. A ratio is
  `Real(unit="1")`, a data rate or size `Real(unit="bit/s")` / `Real(unit="bit")`,
  a plain number `Real`.
- **Configurations.** A node that deploys only variants is in the system
  models of the configurations selecting one of them, and out of the others,
  with its links. A node that deploys a *variation* declares the parameters
  its variants carry; each system model sets them to the values of the
  variant its configuration selects, in the `constrainedby` clause, so they
  also apply to the redeclared model. A model without configurations has one
  system model, `Systems.System`.
- **Deterministic.** The same model gives the same text: ports and links in
  declaration order, parameters sorted by name, no date.

## Ports and connectors

A port says what it carries with `physics:`, on the port, on a physical port,
or on the type the port `is:` (a port definition):

```arc
type "HV plus" { physics: electrical }
type "HV minus" { physics: electrical pole: negative }
node "Battery pack" { id: "PN-BATT" port "hv_p" { is: "HV plus" } port "hv_n" { is: "HV minus" } }
node "Drive unit" { id: "PN-DRIVE" port "shaft" { physics: mechanical_rotational flange: b } port in "torque_cmd" { physics: signal } }
```

| `physics:` | Modelica Standard Library connector |
|---|---|
| `signal` | `Modelica.Blocks.Interfaces.RealInput` (`port in`) / `RealOutput` (`port out`) |
| `electrical` | `Modelica.Electrical.Analog.Interfaces.PositivePin`, `NegativePin` with `pole: negative` |
| `mechanical_translational` | `Modelica.Mechanics.Translational.Interfaces.Flange_a`, `Flange_b` |
| `mechanical_rotational` | `Modelica.Mechanics.Rotational.Interfaces.Flange_a`, `Flange_b` |
| `thermal` | `Modelica.Thermal.HeatTransfer.Interfaces.HeatPort_a`, `HeatPort_b` |
| `fluid` | not exported yet (noted) |

A flange or heat port is `_b` with `flange: b`, or for an `out` port saying
nothing; `_a` otherwise. A port without `physics:` is not exported, and the
export says so: ArcLang does not guess.

`physical_link`s and `physical_exchange`s between ports become `connect(...)`:
between nodes in the system models, between a node's ports and its behaviour
components (or between those) in the node's model. Their ends must have the
same physics, and a signal must go from an output to an input (a node port
relaying a behaviour component's keeps its direction): otherwise the model
does not compile, export or not. An exchange between behaviour components of
two different nodes is not exported (it is routed through ports of the nodes).

## Verification by simulation

```arc
test_case "Winter range, long range pack" {
    id: "TC-RANGE-LR"
    verifies: ["REQ-RANGE"]
    method: "analysis"
    simulate: { model: "EVPhysics.TouringRange" configuration: "CFG-TOURING" stop_time: 3.2 h sources: ["physics/EVPhysics.mo"] }
    expects: ["final(vehicleBody.distance) >= 300 km", "min(\"PN-BATT\".soc) >= 0.1"]
}
```

`simulate:` takes `backend` (see below), `model` (required: the hand-written
Modelica class, which must extend the system model of `configuration`;
OpenModelica checks it with `extendsFrom`), `stop_time` (required) and
`start_time` as time quantities, `sources` (Modelica files, relative to the
model file), `tolerance` (default `1e-6`) and `intervals` (output intervals,
default 500). A case that simulates neither asserts constraints nor gives a
verdict.

### Sweeps: one case over many conditions

`sweep:` runs the case over a grid of parameter values, and its `expects:`
must hold at every point:

```arc
simulate: { model: "EVPhysics.TouringRange" configuration: "CFG-TOURING" stop_time: 3.2 h
            sources: ["physics/EVPhysics.mo"]
            sweep: ["vehicleBody.ambient = 253.15 K, 263.15 K, 293.15 K",
                    "\"PN-VCU\".speedController.target_speed = 90 km/h .. 130 km/h step 20 km/h"] }
```

Each axis is `parameter = values`: a Modelica path of the simulated model
(its first part may be an ArcLang id in quotes), then a list of values or a
range `a .. b step s` in one unit. The grid is every combination (here 3 × 3),
at most 200 points. All of them run in one OpenModelica session, each with
`-override`; the model is compiled once per point, so a point costs about a
second.

A value only counts once it is proven: each point also asks OpenModelica to
print the parameters it ran with (`-output`), and a parameter missing from
that line (a name the model does not have, which OpenModelica only warns
about) or holding another value (a constant, a `final` or an evaluated
parameter) makes the case an **error**, never a silent pass. The verdict is
`pass` when every point holds, `fail` when one does not (the reason names how
many and the first), `error` when one could not run. `arclang verify` prints
every point with its values and margins; `--json` gives them as `points`.

Each of `expects:` compares two expressions over the result. A variable is a
Modelica name of the result (`vehicleBody.distance`, `vehicleBody.car.v`), or
an ArcLang element in quotes followed by the rest of the path
(`"PN-BATT".soc` is `batteryPack.soc`). A variable is a time series, read
through `min(v)`, `max(v)`, `final(v)`, `initial(v)`, `mean(v)` (over time)
or `at(t, v)` (interpolated). Numbers may carry units (`300 km`, `100 km/h`,
`1 h`); `+ - * /` and parentheses work as in constraints. Values are
compared in SI units. When OpenModelica gives the unit of a variable (from
`<model>_init.xml`), the comparison is dimension-checked: `final(distance) >= 300 s`
or `>= 300` with no unit is an **error** verdict, not a fail.

Verdicts:

| Verdict | When | Cached | Gate |
|---|---|---|---|
| `pass` | every expectation holds | yes | verified |
| `fail` | one does not; the values are reported | yes | blocker |
| `error` | the model does not load, does not extend its system, does not simulate, or an expectation is ill-typed | yes | blocker |
| `inconclusive` | no OpenModelica, a timeout; or, when compiling, a verdict never computed or stale | no | blocker |

The cache is `.arclang/sim-cache/<test case id>.json` beside the model: the
verdict, the evaluated values, the OpenModelica log and version, and the key
(SHA-256 of the generated package, every source file and the case: backend,
model, configuration, times, tolerance, intervals, expectations). Any change
to one makes the cached verdict stale. The cache may be committed as
evidence; `arclang gate` in CI then needs no OpenModelica, and fails when the
evidence is stale.

### Backends

`backend:` names what runs the simulation; `openmodelica` is the default and
the only one built. Every backend ends in the same table of time series and
the same `expects:` evaluation, so the others change no model:

- `fmu`: any FMU (Dymola, Simulink, PSCAD, ...), run through FMPy;
- `command`: an external script that writes a results CSV.

They are accepted by the language today and answer **inconclusive**
(`backend 'fmu' is declared but not implemented yet`).

## SysML v2

`physics:`, `pole:` and `flange:` are written as `String` attributes of the
ports and types. `expects:` is a `String [*]` attribute. `simulate:` is
ArcLang-only: SysML v2 has no value for a block of settings, so it is written
as its ArcLang text in a `String` attribute, which the importer reads back.

## Limits

- **Causal and acausal.** Signals are causal (`RealInput`/`RealOutput`);
  electrical, mechanical and thermal ports are acausal connectors, and
  ArcLang does not check that the physics balances: OpenModelica does, when
  it checks the hand-written models.
- **Fluid** ports are not exported: `Modelica.Fluid` connectors need a
  medium, which ArcLang does not model yet.
- **The logical architecture** is not exported as models: it gives the
  variants and their parameters only.
- Nodes do not nest in ArcLang; behaviour components are one level inside
  their node. A port holding ports (`Owner.port.sub`) is not exported.
- A behaviour component a configuration leaves out stays in its node's model
  (the node model is shared by all configurations).
- Simulating compiles C code: about 20 s a case with Docker on a laptop.
- With Docker on Linux, files the container writes in the working directory
  belong to root; ArcLang reads the result and removes the directory as far
  as it can.
- A model compiled from text (the language server, the API) has no file, so
  no cache: its simulated cases are inconclusive.
