# ArcLang MCP Server

A [Model Context Protocol](https://modelcontextprotocol.io) server that gives an
AI assistant the ArcLang compiler, all of it: every capability of the
`arclang` command line is reachable through a tool. Check a model, run the
production gate, simulate its test cases and climb the realization ladder
(SIL, PIL, HDL, SoC, HIL), run its digital twins, prove or recover code,
diff, merge and restore versions, follow the impact of a change, compute
its tables and documents, read a model element by element and edit it with
precise requests through the compiler's workbench API (the one the VS Code
extension and its agent use) without losing its comments, import SysML v2,
Capella or ReqIF, and export (SysML v2, Capella, ReqIF, C, VHDL, FMU,
Modelica, STEP, GeoJSON...).

Version 4.13.0, for the `arclang` compiler built from this repository, of the
same version: the tools are written against the options that compiler's
`--help` prints, and a test fails when the two disagree. An answer warns
when the compiler is older than 4.3.0; between 4.3.0 and the current one, a
tool whose command the compiler lacks fails with the compiler's own message.
Python 3.10 or later.

## What it guarantees

The server is a thin layer over the `arclang` binary, and it holds to the
compiler's rule: it never says more than the compiler did.

- Every tool is one subcommand of the CLI, and every subcommand has its tool.
  `tools/cli.py` says which subcommand each tool runs, and lists what is left
  out on purpose with its reason: the commands that are interactive, stay
  running or open a browser (`serve`, `lsp`, `repl`, `explorer`), those the
  CLI answers "Not implemented" for (`clean`, `sync`, `plugin`,
  `info --dependencies`, `export -f pdf`), and a few options (`--open`,
  `debug --dap`, `merge --install`). `tests/test_cli_parity.py` reads
  `arclang --help` and fails when a subcommand, an option, a format, a view,
  a table kind or an edit operation exists that no tool reaches and no list
  leaves out. Every command line is an argument list, never a shell.
- A tool answer is the compiler's output, preceded by one line read from the
  exit code (or from the JSON the compiler printed). A failure is reported as
  a failure, with the compiler's message.
- The language reference a client reads is not a copy that can drift: the
  metamodel and the rule catalog are printed by the binary, the example
  model is compiled, formatted and gated by the test suite, and every
  construct the syntax rules teach is written in that model.
- Generated ArcLang (requirements, components, skeletons) is compiled before it
  is returned, and the answer says whether the compiler accepted it. An
  imported model is checked after it is written, and the answer says whether
  it compiles.
- Nothing is written to a model unless the call says `write: true`
  (`arclang_edit`, `arclang_set_attribute`, `arclang_unset_attribute`,
  `arclang_rename`, `arclang_format`, `arclang_transition`); without it the
  answer is a diff. `arclang_edit` applies its requests on a copy in a
  private temporary directory and refuses the whole list when the compiler
  refuses one.
  Tools that produce a file (`arclang_compile`, `arclang_export`,
  `arclang_import`, the diagram, table, document and report tools) write only
  where the call names an `output_path` or `output_dir`, with the exceptions
  their descriptions state: `arclang_compile` without `output_path` writes
  `<model>.json` next to the model, as `arclang build` does; the runs
  (`arclang_verify`, the stages, `arclang_twin`) keep their verdicts in the
  compiler's cache beside the model. `arclang_merge` and `arclang_restore`
  return a text unless told where to write it; `arclang_ci_init` prints the
  pipeline files unless `write: true`; `arclang_baseline` changes the
  repository only with `action: create`.
- What takes minutes or needs tools is asked for by name. `arclang_realize`
  and `arclang_partition` print their plan unless `run: true`;
  `arclang_schedule` runs on the emulator only with `prove: true`;
  `arclang_ci_run` simulates only with `verify: auto`; `arclang_twin`
  follows live data only for the `for_seconds` it is given. Those runs take
  a `timeout`.
- A call cannot name a program for the server to run. A serial port is a
  device name (`/dev/...`, `COM3`); a C compiler (`cc`) is the bare name of a
  program on the `PATH`; a command line (`arclang_recover` `with`,
  `arclang_hil` `command`) is refused unless the server was started with
  `ARCLANG_MCP_ALLOW_COMMANDS=1`. No tool opens a browser. Arguments are
  checked against each tool's schema, to the leaves, before anything runs, so
  `"false"` (a string) is refused and never read as a yes, and an empty path
  is refused.
- An answer warns when the `arclang` it runs is older than 4.3.0.
- Nothing leaves the machine unless the call says so (`arclang_review` with
  `send: true`), or an `ANTHROPIC_API_KEY` is set for the generation tools.
- What the compiler did not do, the server does not say it did: a binary
  read by `arclang_recover` is "proposed, NOT proven"; an observation is
  "measured, not proven"; a plan is "nothing was run".

## Install

```bash
# 1. The compiler
cargo build --release              # from the repository root
./install-arclang.sh               # copies the binary to ~/.local/bin/arclang
arclang --version

# 2. The server
cd mcp-server
pip install -e .
python test_install.py             # imports, and finds a compiler it can work with
```

### Claude Desktop

`~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "arclang": {
      "command": "arclang-mcp",
      "env": {
        "ARCLANG_WORKSPACE": "/path/to/your/models"
      }
    }
  }
}
```

### Claude Code

```bash
claude mcp add --env ARCLANG_WORKSPACE=/path/to/your/models arclang -- arclang-mcp
```

### Settings

| Variable | Meaning | Default |
|---|---|---|
| `ARCLANG_WORKSPACE` | Directory the server works in; relative paths are resolved from it | current directory |
| `ARCLANG_MCP_ROOTS` | Other directories the server may read and write in, separated like `PATH`; `*` is any path | unset: the workspace only |
| `ARCLANG_BINARY` | Path of the compiler | `arclang` on the `PATH` |
| `ARCLANG_SERVER` | Path of `arclang-server` (Enterprise edition), which `arclang_engineering` runs | `arclang-server` on the `PATH` |
| `ANTHROPIC_API_KEY` | Lets the generation tools ask a Claude model | unset: templates only |
| `TYPESAFE_API_KEY` | Needed by `arclang_review` with `send: true` | unset |
| `ARCLANG_PROPOSER` | The command `arclang_recover` with `propose: true` asks for a model (read by the compiler) | unset |
| `ARCLANG_MCP_ALLOW_COMMANDS` | `1` lets a call name a command to run (`with`, `command`) | unset: refused |

A tool given a path outside the workspace and outside `ARCLANG_MCP_ROOTS` refuses
it and says so: an agent can be asked for any path by what it read, and the
server does not write where you did not send it. To export next to your
models and to another folder, name that folder:
`ARCLANG_MCP_ROOTS=/path/to/exports`. In VS Code the extension starts the
server on the folders that are open.

Timeouts and the model can be written in `<workspace>/.arclang-mcp.toml`. The
compiler's path cannot: a workspace may be a cloned repository, and it must not
choose the program the server runs. Use `ARCLANG_BINARY`.

```toml
[compiler]
timeout = 30            # seconds, per command
review_timeout = 600    # seconds, for arclang_review

[ai]
model = "claude-opus-5-5"
```

## Resources

| URI | Content |
|---|---|
| `arclang://syntax-rules` | The rules most often broken (ids, quantities, types, constraints, state machines, scenarios, data model, physical paths), the constructs of verification that runs (`execute:`, `simulate:`, `twin`, `io_binding`), of code written by hand (`correspondence`), the levels of the V, organizations, documents and risks, each in a model the tests compile, and a model that compiles and passes the gate |
| `arclang://reference-model` | That model alone |
| `arclang://metamodel` | Element kinds, typed attributes, enumerations, units, trace rules, printed by the compiler |
| `arclang://grammar` | `spec/GRAMMAR.ebnf`, when the server runs from the repository |

## Tools

### Check and analyze

| Tool | Command | Returns |
|---|---|---|
| `arclang_validate` | `check --lint [--json] [--method] [--safety] [--format sarif]` | Errors with line and column; counts of compilation warnings (typed attributes and constraints block the gate), traceability warnings, advisories. With `as_json: true`, each finding with its `rule`, `severity`, `element` and `fix`; `method: cesam` for the CESAM rules; `format: sarif` for a SARIF log |
| `arclang_rules` | `rules --format markdown\|json` | The catalog of validation rules: meaning and remedy of each `rule` a finding names, or one `rule` alone |
| `arclang_compile` | `build [--target json\|xml\|capella]` | Counts of the compiled model |
| `arclang_gate` | `gate --standard iso26262\|do178c\|iec61508 [--format sarif]` | PASS or FAIL and each blocker with its rule |
| `arclang_realize` | `realize --json`, with `--dry-run` unless `run: true` | The realization ladder: each requirement's verdict per stage (simulation, sil, pil, hdl, soc, board, twin), the steps that would fill the missing or stale ones, the gaps. With `run: true` the climb is real (`keep_going`, `board`) |
| `arclang_partition` | `partition`, with `--dry-run` unless `run: true`; `table --kind partition` | The two runs that measure each machine as C (step time, footprint) and as VHDL (LUT4, Fmax), and what was measured already. With `run: true` they are run |
| `arclang_schedule` | `schedule --json [--wcet M=US] [--prove --cycles N]` | The bindings as tasks, the response time of each (fixed-priority analysis) from measured or given step times, whether each meets its period. FAILED when a step time is unknown, with how to measure it. `prove: true` runs the tasks on the target's emulator |
| `arclang_cesam` | `table --kind cesam_grid -f json`, `check --json --method cesam` | The model through the CESAM framework: each vision x aspect view with its elements and maturity (a gap names the Capella diagram that draws it), and the CESAM methodology advisories. `scope`: a level or the system of interest |
| `arclang_prove` | `prove [--correspondence N] --timeout S` | Whether code written by hand (an Arduino sketch) is the state machine its `correspondence` says: proven, refuted with the lines that do not hold and a case that shows it, or neither. The agent that recovers a model writes the machine and the correspondence, calls this, corrects what is refuted, and says "recovered" only on Proven. Needs CBMC and Bitwuzla (the formal image); says so without them |
| `arclang_recover` | `recover [--check] [--hdl DIR] [--view HTML]`, `recover board.dts`, `recover firmware.hex [--model M]`, `recover --propose [--with C] [--turns N]` | From the generated C: the state machines as ArcLang, proven when `check` (the default) generates that C again line for line. From a `.dts`: the board as a physical architecture (read, not proven). From a `.hex`: a correspondence proposed from the machine code read with Ghidra (NOT proven; `--check` is never passed). With `propose`: a proposer proposes a model of a sketch or a binary, written only once ArcLang has proven it |
| `arclang_drift` | `drift [--hdl DIR] [--prove --timeout S] [--json]` | The code as built against the model: each difference in the terms of the model; with `prove`, proven equivalent, or not equivalent with what shows it, or neither |
| `arclang_engineering` | `arclang-server engineering <action>` (Enterprise edition) | `connectors`, `model`, `status`, `credibility`, `artifacts`, `history`, `impact`, `size`, `trade`: the parts of a model judged by the engineering engines, and the verdicts read back. The first line says `every limit holds` only when it does; a failing, stale, not-run or inconclusive verdict has its own word. Without `arclang-server` (`ARCLANG_SERVER`, else the `PATH`) the answer says `UNAVAILABLE` and nothing is run. The public server names no engine: `connectors` lists them. Passes no credential |
| `arclang_trace_analysis` | `trace --validate [--matrix]`, `trace --suspects`, `trace --review [--from] [--to]` | Coverage, issues, the matrix with rationales; the suspect traces; traces marked as reviewed (a diff unless `write: true`) |
| `arclang_impact` | `impact [-e E]... [--change JSON] [--delete E] [--since REV] [--depth] [--kinds] [--direction] [-f]` | What a change reaches, by distance: one or several elements, a proposed edit never written, a deletion, the changes since a revision |
| `arclang_diff` | `diff --json`, `diff -f text\|json\|md\|html`, `diff --from REV [--to REV]` | Semantic diff by stable identity, between two files or two git revisions of one; the rich report with its consequences |
| `arclang_info` | `info --metrics` | Element counts |
| `arclang_metamodel` | `metamodel` | The metamodel, or one element kind |
| `arclang_safety_check` | `safety --standard [--fmea] [--fta] [--report] [-o]` | Counts of declared safety content, the FMEA entries, the fault tree of every hazard, the safety report in Markdown (not a verdict) |
| `arclang_hazard_analysis` | `export -f json` | The hazards and FMEA entries the model declares |
| `arclang_review` | `review` | Plausibility of each trace, from TypeSafe's Jev. Dry run unless `send: true` |

### Simulate, run the stages, observe

One tool per command. They build, emulate and simulate with tools the
compiler finds or runs in Docker: a call may take minutes and takes a
`timeout` (900 s by default). Without its tools a stage fails and says which
one is missing.

| Tool | Command | Returns |
|---|---|---|
| `arclang_verify` | `verify [--test] [--json]`; `--list`, `--check-tools`, `--replay CASE --point N`, `--statistics r\|python`, `--drawings`, `--drawing CLASS` | The cases that `simulate:` run in OpenModelica and their `expects:` judged; or, without simulating, the cases, what is installed, the results kept of a run, the statistics of a Monte Carlo run |
| `arclang_cosim` | `cosim [--test] [--cc] [--keep] [--json]` | Each case that `execute:`s as an FMI co-simulation in OMSimulator |
| `arclang_sil` | `sil [--test] [--cc]... [--keep] [--json]` | The generated C compiled and run on this computer, compared with the model step by step |
| `arclang_pil` | `pil [--test] [--target] [--keep] [--json]` | The same on the target's processor, in its emulator |
| `arclang_hdl` | `hdl [--test] [--synth] [--as-if] [--machine] [--keep] [--json]` | The VHDL simulated by GHDL against the fixed-point C and the model; with `synth`, resources and Fmax |
| `arclang_soc` | `soc [--test] [--bitstream] [--board] [--no-sim] [--gate-level] [--port] [--keep] [--json]` | The processor and the logic together in Verilator; the bitstream; the cases on a board |
| `arclang_hil` | `hil --port P \| --command C [--baud] [--test] [--timeout] [--json]` | The generated C on a board, through its serial port |
| `arclang_twin` | `twin [--twin] [--json]`, `--list`, `--live --for S` | Each twin driven by its measured data and compared with it; live data only for the seconds given |
| `arclang_observe` | `observe [--correspondence] --instrument P [--drive] [--watch] [--changes] [--seed] [--keep]`, or `--recorded FILE` | A board measured against the model, change by change: a measure, never a proof |
| `arclang_debug` | `debug [--machine\|--scenario\|--chain] [--events] [--set] [--json]`, `--list` | A state machine, a scenario or a chain run step by step |

### Versions

| Tool | Command | Returns |
|---|---|---|
| `arclang_merge` | `merge <base> <ours> <theirs> --json [-o] [--path] [--no-compile]` | The three-way merge by element: clean, or the conflicts (markers around the whole element), or a result that does not compile. Nothing is written without `output_path`. `arclang_git_merge` is its first name, kept for existing clients |
| `arclang_history` | `history <model> <element> [--limit] [--json]` | The commits that changed one element |
| `arclang_restore` | `restore --rev R [--element E]... [-o]` | The model, or some elements, as at a revision: a diff unless `output_path` |
| `arclang_baseline` | `baseline create\|list\|diff\|verify` | The baselines of the repository; `create` writes `baselines/<name>/` and a tag |

`arclang_diff` compares two revisions and `arclang_workbench` reads the model
as committed at one (`rev`).

### Tables, documents, pipelines

| Tool | Command | Returns |
|---|---|---|
| `arclang_table` | `table [--name\|--kind] [--scope] [--row-type] [--columns] [--rows] [--kinds] [--filter] [--gaps] [-f] [-o] [--declaration]`, `--list` | Any table or matrix of the model, as json, csv, tsv, md, html or xlsx (`arclang_cesam` and `arclang_partition` stay the two shaped answers) |
| `arclang_document` | `document [-t template] -o out.md\|.html\|.docx` | A template filled from the model, or the design review |
| `arclang_new` | `new <name> --template cesam -o file` | A model from a template; never over a file that exists |
| `arclang_report` | `report --pr --from BASE [--to] [-f] [-o] [--out-dir] [--fail-on]` | The pull-request report: diff, impact, gate before and after |
| `arclang_sarif` | `sarif [models] [-o] [--fail-on]` | Diagnostics and blockers of several models as one SARIF log |
| `arclang_ci_init` | `ci init [--provider] ...`, with `--dry-run` unless `write: true` | The pipeline files of a repository |
| `arclang_ci_run` | `ci run --verify never\|auto [--sarif] [--summary]` | What a pipeline runs, here |
| `arclang_ci_models` | `ci models [--json]` | The entry-point models of a repository |

### Read and edit through the workbench API

The compiler's workbench API is what the VS Code extension and its agent use;
these two tools give the same picture and the same edits to any MCP client
(Claude Desktop, Claude Code, Cursor, Copilot).

| Tool | Command | What it does |
|---|---|---|
| `arclang_workbench` | `workbench --gate-standard <std> [--exports] [--rev R]` | A compact summary of the model: counts by kind, the elements (`id \| kind \| name \| owner \| attributes`) filtered by `kind`, `layer` or one `element` (then with its relationships and traces), the findings (rule, severity, element, message), the gate verdict, the diagrams (id, view, title). With `raw: true`, one `section` of the compiler's JSON as printed: `elements`, `diagrams`, `findings`, `gate`, `document`, or `sysmlv2`, `reqif`, `c_headers`, `proto` with `exports: true` |
| `arclang_edit` | `edit --request <json>`, once per request | A list of requests applied in order on one text: `set`, `unset`, `rename`, `add`, `add_raw`, `remove`, `move`, `add_trace`, `remove_trace`, `wrap_fragment`, `wrap_invariant`, `reconnect`, `duplicate`, `duplicate_many` (the schema of `arclang edit`). Elements are designated by the ids `arclang_workbench` lists, even when the text derives them from names. The whole list is refused, and nothing changes, when the compiler refuses one request; the answer names the request and the reason. Without `write: true` the answer is the diff |

```json
{"path": "braking.arc", "requests": [
  {"op": "add", "keyword": "function", "name": "Log braking", "container": "LC-002",
   "attributes": [["id", "\"LF-003\""], ["latency", "2 ms"]]},
  {"op": "set", "element": "LF-003", "key": "description", "value": "\"Records each braking command\""},
  {"op": "add_trace", "from": "LF-003", "kind": "realizes", "to": "SF-002"}
]}
```

### Edit and transform

All keep comments, order and layout, and are refused unless the edited model
compiles. Without `write: true` they return a diff and change nothing.
`arclang_set_attribute`, `arclang_unset_attribute` and `arclang_rename` are
kept for existing clients: `arclang_edit` is the general form.

| Tool | Command | What it does |
|---|---|---|
| `arclang_format` | `format` | Whitespace only |
| `arclang_set_attribute` | `set` | One attribute of one element; the value is ArcLang source (`25 ms`, `"text"`, `[A, B]`) |
| `arclang_unset_attribute` | `unset` | Removes one attribute |
| `arclang_rename` | `rename` | Renames one element; its identity (id) does not change |
| `arclang_transition` | `transition [--plan] [--step] [--into]` | Arcadia transition: without a `step` the compiler's JSON plan of what each step (oa-to-sa, sa-to-la, la-to-pa) would create; with a `step` the elements declared on the next level, `into` the component or node that receives them |

### Import, export and diagrams

| Tool | Command | Returns |
|---|---|---|
| `arclang_import` | `import -f <format> -o <output_path>` | Reads `sysml` (SysML v2 text, or Systems Modeling API records as JSON), `capella`, `req-if`, `doors`, `json`, `yaml`, `xml`, `mermaid`, `plant-uml` into a `.arc` file; relays what the compiler counted and left unresolved, and whether the written model compiles |
| `arclang_export` | `export -f <format> [--target] [--binding] [--split] [--no-placement] [--strategy] [--element] [--leaf-functions] [--port-naming]` | Every format of the CLI but `pdf` (which it refuses): `sys-ml`, `sysml-json`, `capella`, `req-if`, `c-headers`, `proto`, `json`, `yaml`, `xml`, `markdown`, `mermaid`, `plant-uml`, `html`, `terraform`, `simulink`, `simulink-connector`, `geojson`, `step`, and those that fill a directory: `c`, `fmu`, `zephyr`, `vhdl`, `modelica`, `fmi`. A directory is listed; without `output_path` nothing is kept |
| `arclang_viewpoint_diagrams` | `diagram -f viewpoints\|elk\|html [--view] [--layout] [--theme]` | The viewpoint diagrams (oab, sab, lab, pab, msm, es, cap, cdb, pbs, req, par, or the views of CESAM): a self-contained HTML viewer, the diagram model as JSON, or the ELK graphs |
| `arclang_generate_diagram` | `diagram -f <type> [--title]` | One SVG (needs the Node renderer), or a `mermaid` / `plant-uml` text diagram |
| `arclang_generate_all_diagrams` | `diagram -f <type>`, each SVG type | The files written, and why a type was not |
| `arclang_export_diagram` | `export` | Kept for existing clients |

### Write from a description

| Tool | What it does |
|---|---|
| `arclang_generate_requirement` | A `req` declaration, compiled before it is returned |
| `arclang_generate_component` | A logical component or physical node, compiled before it is returned |
| `arclang_suggest_architecture` | Components suggested by a Claude model. Needs `ANTHROPIC_API_KEY`, says so without it |
| `arclang_analyze_requirements` | Sorts sentences by Arcadia layer from their wording (keyword matching) |
| `arclang_generate_from_requirements` | A skeleton from that analysis, compiled. No trace, actor or hardware is invented |
| `arclang_assess_diagram_quality` | String checks on an SVG, for component, physical and operational diagrams |
| `arclang_suggest_diagram_enhancements` | Enhancement names from the failed checks |
| `arclang_generate_mbse_report` | Markdown summary of an analysis |

### Batch

`arclang_execute_batch` runs several operations in one call and returns a
reduced result for each: see [BATCH_EXECUTION_GUIDE.md](BATCH_EXECUTION_GUIDE.md).

### Not offered, on purpose

The one place that says so is `src/arclang_mcp/tools/cli.py`, with a reason
for each: `serve`, `lsp` and `repl` (they stay running or talk on the
standard input), `explorer` (a page for a person, which can open a browser),
`clean`, `sync` and `plugin` (the CLI answers "Not implemented"), and the
options `--open`, `debug --dap`, `merge --install`, `info --dependencies`,
`--verbose` and `--as`. Everything else of `arclang --help` has a tool.

## Tests

```bash
pip install -e ".[dev]"
pytest
```

The tests run the real binary (`ARCLANG_BINARY`, else `target/release/arclang`,
else `target/debug/arclang`, else the `PATH`) and are skipped without one. They
cover every tool, the resources against the compiler (the rule ids and
constructs the syntax rules name against the catalog, every fenced model of
the rules compiled), the workbench read and the scripted edits on the
flagship example, and a session over stdio. They pass with the MCP Python
SDK 1.x and 2.x.

Three files keep the server current with the CLI:

- `test_cli_parity.py` reads `arclang --help`: every subcommand is run by a
  tool or left out by name with its reason; every long option of a covered
  subcommand is an argument of its tool or left out by name; the formats,
  views, themes, table kinds, edit operations and option values the server
  offers are EQUAL to the CLI's, less the named exclusions.
- `test_command_lines.py` gives each such argument to its tool and checks
  the option is really on the command line built.
- `test_extended_tools.py` gives every path argument of every tool a path
  outside the workspace and checks it is refused.

The commands that simulate, build for a target, prove or drive a board are
not launched by the tests: their command line is recorded and read, and
their cheap paths (`--list`, `--check-tools`, a recorded observation, a
refusal) are run with the real binary.

## Files

```
src/arclang_mcp/
  server.py              lists the tools and resources, routes the calls
  compiler/wrapper.py    runs the binary (an argument list, no shell); the lists of formats, views, kinds
  tools/cli.py           which subcommand each tool runs; what is left out, with its reason
  tools/common.py        paths, the command-line builder, the shape of an answer
  tools/core.py          check, rules, build, gate, realize, partition, schedule, trace, impact, diff, export, metamodel, review
  tools/simulation.py    verify, cosim, sil, pil, hdl, soc, hil, twin, observe, debug
  tools/versioning.py    merge, history, restore, baseline
  tools/delivery.py      table, document, new, report, sarif, ci
  tools/recover.py       prove, recover, drift
  tools/edit.py          format, set, unset, rename
  tools/workbench.py     the workbench read and the scripted edits (arclang workbench, arclang edit)
  tools/transform.py     import, transition
  tools/generation.py    snippets (compiled) and diagrams
  tools/safety.py        safety summary, declared hazards
  tools/expert.py        requirement sorting, skeleton, SVG checks
  tools/batch.py         several operations in one call
  resources/             syntax rules, the reference model and the shorter examples
  ai/generator.py        templates, or a Claude model when a key is set
tests/                   pytest, against the real binary
```

See also [QUICKSTART.md](QUICKSTART.md), [ARCHITECTURE.md](ARCHITECTURE.md) and
[ARCLANG_SYNTAX_RULES.md](ARCLANG_SYNTAX_RULES.md).

## License

Business Source License 1.1, like the compiler: see the [LICENSE](../LICENSE) at the root of the repository.
Authors: Malek Baroudi and Bilel Laasami, co-founders of ArcLang.
