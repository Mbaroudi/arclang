# The Capella-Simulink connector

ArcLang does what the Eclipse Capella-Simulink connector does (Thales and
Obeo, EPL 2.0): it turns chosen elements of a Capella model into the
connector's pivot (`.mlcore`) and the pivot into the MATLAB script that
builds the Simulink project: models for functions and functional chains,
data dictionaries for data, Stateflow charts for state machines, each
element tagged with the Capella id it comes from.

Both halves of the connector are ported, and checked against the
connector's own tests: the generator against its 41 expected scripts, the
model-to-model transformation against its 34 configurations (Capella
selection, strategy, options) and their expected scripts. Every one gives
the connector's script to the character.

## Command

```
arclang export <model> -f simulink-connector -o <out> [options]
```

`<model>` is an ArcLang model, or a `.capella` file (its `.aird` beside it
is read too). An ArcLang model is first written as Capella by the Capella
exporter (the one `-f capella` uses), then transformed: nothing in the
language is specific to this export.

It writes `<out>.mlcore`, the pivot (the file the connector's MATLAB toolbox
imports with `CapSLBridge.import`), and `<out>.m`, the script itself. A
`.m` or `.mlcore` extension on `<out>` is dropped.

| Option | What it does |
|---|---|
| `--strategy functions` (default) | Functions, their ports, and the states they are available in. Each chosen function and its sub-functions; a leaf becomes a model of its own (or a subsystem, see below), an exchange a line, an exchanged item a bus. |
| `--strategy functional-chains` | A model per chain, referencing the models of its functions, its exchanges as lines. |
| `--strategy state-machines` | A model per state machine, holding its Stateflow chart: states, junctions for pseudo states and final states, transitions with their triggers (messages fed by ports) and guards. |
| `--strategy data-pkg` | Data packages as data dictionaries: classes and exchange items as buses, numeric types as types (MATLAB's own for Capella's predefined ones), physical quantities as signals, enumerations. |
| `--element <id or name>` | The elements to export, by Capella id or by name; repeat it or separate them by commas. A name must name one element of the strategy's kind; when several have it, the error lists their ids. By default: every function directly under a layer's root function, every chain, every state machine, every layer's data package. |
| `--leaf-functions model\|subsystem` | The connector's `leaf.functions.export.option`. |
| `--port-naming ExchangeItem\|CapellaElement` | The connector's `port.naming.option`: a port is named after its first functional exchange (default) or after the Capella port. |

```
arclang export model.capella -f simulink-connector -o out/model --element "Top 3"
arclang export model.arc -f simulink-connector -o out/modes --strategy state-machines
```

## What is the connector's, and what is not

The transformation runs as the connector's does: the strategy widens the
selection (a function's ports and sub-functions, a port's exchanges, a
chain's functions, a machine's regions and transitions...), then the rules
run in the order Kitalpha's Transposer gives them (each rule after those its
premises name), and the dictionaries are linked to those their buses use.
The pivot's elements, their order and their names are the connector's.

A state machine's chart takes its layout from the machine's diagram in the
`.aird`, as in the connector. The connector reads the positions from the
diagram as Eclipse draws it, so its output depends on the platform's fonts:
its tests accept one script per platform. ArcLang lays the diagram out as
the platform of the connector's reference scripts draws it (Eclipse on Linux,
DejaVu Sans 8 pt): the positions and sizes of states, junctions and the clock
positions of transition ends are those of the reference scripts. The size of
the regions of a state with several regions, and of a machine with several
regions, are estimates (no test of the connector covers them).

The connector refuses a state machine whose region has no diagram. A model
written from ArcLang has none, so the command lays such a chart out itself
(states on a grid, transitions between their centers) and says so in a
note. This layout is ArcLang's, not the connector's.

The ids of the pivot's elements are UUIDs (version 5) of their place in the
document, where the connector's are random: the same model always gives the
same `.mlcore`.

## Sources and tests

- `src/compiler/simulink_connector/m2m/`: the transformation (`model.rs`
  reads the Capella XMI, `scope.rs` widens the selection, `transposer.rs`
  orders and applies the rules, `function.rs`, `data.rs` and `sm.rs` are
  the rules, `diagram.rs` the diagram geometry, `drawing.rs` ArcLang's
  layout of undrawn charts).
- `src/compiler/simulink_connector/generator.rs`: the script.
- `src/compiler/simulink_connector/mlcore.rs`: the `.mlcore` writer.
- `tests/simulink_connector.rs` with the connector's projects, configurations
  and expected scripts in `tests/fixtures/simulink_connector/` (EPL 2.0, see
  its `README.md`).
