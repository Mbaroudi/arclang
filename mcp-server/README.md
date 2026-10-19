# ArcLang MCP Server

A [Model Context Protocol](https://modelcontextprotocol.io) server that gives an
AI assistant the ArcLang compiler: check a model, run the production gate,
diff two versions, follow the impact of a change, read a model element by
element and edit it with precise requests through the compiler's workbench
API (the one the VS Code extension and its agent use), without losing its
comments, declare the next Arcadia level from the current one, import SysML
v2, Capella or ReqIF, and export (SysML v2, Capella, ReqIF, C headers,
Protobuf).

Version 4.3.0, for the `arclang` compiler built from this repository (4.3.0 or
later: it needs `workbench` and `edit --request`, on top of `check --json`,
`rules`, `transition` and `import -f sysml`). Python 3.10 or later.

## What it guarantees

The server is a thin layer over the `arclang` binary, and it holds to the
compiler's rule: it never says more than the compiler did.

- Every tool is one subcommand of the CLI. `compiler/wrapper.py` is the only
  place where a command line is written, and it passes only options the CLI has
  and implements (`safety --report`, `--fmea`, `--fta` and
  `info --dependencies` answer "Not implemented", so they are not offered).
- A tool answer is the compiler's output, preceded by one line read from the
  exit code (or from the JSON the compiler printed). A failure is reported as
  a failure, with the compiler's message.
- The language reference a client reads is not a copy that can drift: the
  metamodel and the rule catalogue are printed by the binary, the example
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
  `arclang_import`, the diagram and report tools) write only where the call
  names an `output_path` or `output_dir`, with one exception that its
  description states: `arclang_compile` without `output_path` writes
  `<model>.json` next to the model, as `arclang build` does. Arguments are
  checked against each tool's schema, to the leaves, before anything runs, so
  `"false"` (a string) is refused and never read as a yes, and an empty path
  is refused.
- An answer warns when the `arclang` it runs is older than 4.3.0.
- Nothing leaves the machine unless the call says so (`arclang_review` with
  `send: true`), or an `ANTHROPIC_API_KEY` is set for the generation tools.
- What the compiler cannot do, the server does not pretend to do
  (`arclang_git_merge` answers that there is no merge).

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
| `ARCLANG_WORKSPACE` | Directory relative paths are resolved from | current directory |
| `ARCLANG_BINARY` | Path of the compiler | `arclang` on the `PATH` |
| `ANTHROPIC_API_KEY` | Lets the generation tools ask a Claude model | unset: templates only |
| `TYPESAFE_API_KEY` | Needed by `arclang_review` with `send: true` | unset |

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
| `arclang://syntax-rules` | The rules most often broken (ids, quantities, types, constraints, state machines, scenarios, data model, physical paths), and a model that compiles and passes the gate |
| `arclang://reference-model` | That model alone |
| `arclang://metamodel` | Element kinds, typed attributes, enumerations, units, trace rules, printed by the compiler |
| `arclang://grammar` | `spec/GRAMMAR.ebnf`, when the server runs from the repository |

## Tools

### Check and analyse

| Tool | Command | Returns |
|---|---|---|
| `arclang_validate` | `check --lint [--json]` | Errors with line and column; counts of compilation warnings (typed attributes and constraints block the gate), traceability warnings, advisories. With `as_json: true`, each finding with its `rule`, `severity`, `element` and `fix` |
| `arclang_rules` | `rules --format markdown\|json` | The catalogue of validation rules: meaning and remedy of each `rule` a finding names, or one `rule` alone |
| `arclang_compile` | `build` | Counts of the compiled model |
| `arclang_gate` | `gate --standard iso26262\|do178c\|iec61508` | PASS or FAIL and each blocker with its rule |
| `arclang_trace_analysis` | `trace --validate` | Coverage, issues, the matrix with rationales |
| `arclang_impact` | `impact` | What a change to one element reaches, by distance |
| `arclang_diff` | `diff --json` | Semantic diff by stable identity |
| `arclang_info` | `info --metrics` | Element counts |
| `arclang_metamodel` | `metamodel` | The metamodel, or one element kind |
| `arclang_safety_check` | `safety --standard` | Counts of declared safety content (not a verdict) |
| `arclang_hazard_analysis` | `export -f json` | The hazards and FMEA entries the model declares |
| `arclang_review` | `review` | Plausibility of each trace, from TypeSafe's Jev. Dry run unless `send: true` |

### Read and edit through the workbench API

The compiler's workbench API is what the VS Code extension and its agent use;
these two tools give the same picture and the same edits to any MCP client
(Claude Desktop, Claude Code, Cursor, Copilot).

| Tool | Command | What it does |
|---|---|---|
| `arclang_workbench` | `workbench --gate-standard <std> [--exports]` | A compact summary of the model: counts by kind, the elements (`id \| kind \| name \| owner \| attributes`) filtered by `kind`, `layer` or one `element` (then with its relationships and traces), the findings (rule, severity, element, message), the gate verdict, the diagrams (id, view, title). With `raw: true`, one `section` of the compiler's JSON as printed: `elements`, `diagrams`, `findings`, `gate`, `document`, or `sysmlv2`, `reqif`, `c_headers`, `proto` with `exports: true` |
| `arclang_edit` | `edit --request <json>`, once per request | A list of requests applied in order on one text: `set`, `unset`, `rename`, `add`, `add_raw`, `remove`, `move`, `add_trace`, `remove_trace`, `wrap_fragment` (the schema of `arclang edit`). Elements are designated by the ids `arclang_workbench` lists, even when the text derives them from names. The whole list is refused, and nothing changes, when the compiler refuses one request; the answer names the request and the reason. Without `write: true` the answer is the diff |

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
| `arclang_export` | `export -f <format>` | `sys-ml`, `sysml-json`, `capella`, `req-if`, `c-headers`, `proto`, `json`, `markdown`, `mermaid`, `plant-uml`, `html`, ... |
| `arclang_viewpoint_diagrams` | `diagram -f viewpoints\|elk\|html` | The Arcadia viewpoint diagrams (OAB, SAB, LAB, PAB, modes and states, scenarios): a self-contained HTML viewer, the diagram model as JSON, or the ELK graphs |
| `arclang_generate_diagram` | `diagram -f <type>` | One SVG (needs the Node renderer), or a `mermaid` / `plant-uml` text diagram |
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

### Not available

`arclang_git_merge` is listed so that a client asking for a merge gets a true
answer: the compiler has no merge. Use `arclang_diff`.

## Tests

```bash
pip install -e ".[dev]"
pytest
```

The tests run the real binary (`ARCLANG_BINARY`, else `target/release/arclang`,
else `target/debug/arclang`, else the `PATH`) and are skipped without one. They
cover every tool, the resources against the compiler (the format lists and
edit ops of the wrapper against the CLI's `--help`, the rule ids and constructs
the syntax rules name against the catalogue and the reference model), the
workbench read and the scripted edits on the flagship example, and a session
over stdio. They pass with the MCP Python SDK 1.x and 2.x.

## Files

```
src/arclang_mcp/
  server.py              lists the tools and resources, routes the calls
  compiler/wrapper.py    the only place that writes arclang command lines
  tools/core.py          check, rules, build, gate, trace, impact, diff, export, metamodel, review
  tools/edit.py          format, set, unset, rename
  tools/workbench.py     the workbench read and the scripted edits (arclang workbench, arclang edit)
  tools/transform.py     import, transition
  tools/generation.py    snippets (compiled) and diagrams
  tools/safety.py        safety summary, declared hazards
  tools/expert.py        requirement sorting, skeleton, SVG checks
  tools/batch.py         several operations in one call
  tools/integration.py   what the compiler does not do yet
  resources/             syntax rules and the reference model
  ai/generator.py        templates, or a Claude model when a key is set
tests/                   pytest, against the real binary
```

See also [QUICKSTART.md](QUICKSTART.md), [ARCHITECTURE.md](ARCHITECTURE.md) and
[ARCLANG_SYNTAX_RULES.md](ARCLANG_SYNTAX_RULES.md).

## License

MIT. Authors: Malek Baroudi and Bilel Laasami.
