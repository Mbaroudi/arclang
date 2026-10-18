# ArcLang MCP Server

A [Model Context Protocol](https://modelcontextprotocol.io) server that gives an
AI assistant the ArcLang 4.0 compiler: check a model, run the production gate,
diff two versions, follow the impact of a change, edit a model without losing
its comments, and export it (SysML v2, Capella, ReqIF, C headers, Protobuf).

Version 4.0.1, for the `arclang` 4.0.1 compiler (language 4.0.0). Python 3.10 or later.

## What it guarantees

The server is a thin layer over the `arclang` binary, and it holds to the
compiler's rule: it never says more than the compiler did.

- Every tool is one subcommand of the CLI. `compiler/wrapper.py` is the only
  place where a command line is written, and it passes only options the CLI has.
- A tool answer is the compiler's output, preceded by one line read from the
  exit code. A failure is reported as a failure, with the compiler's message.
- The language reference a client reads is not a copy that can drift: the
  metamodel is printed by the binary, and the example model is compiled and
  gated by the test suite.
- Generated ArcLang (requirements, components, skeletons) is compiled before it
  is returned, and the answer says whether the compiler accepted it.
- Nothing is written to a model unless the call says `write: true`. Tools
  that produce a file (`arclang_compile`, `arclang_export`, the diagram and
  report tools) write only where the call names an `output_path` or
  `output_dir`, with one exception that its description states:
  `arclang_compile` without `output_path` writes `<model>.json` next to the
  model, as `arclang build` does. Arguments are checked against each tool's
  schema, to the leaves, before anything runs, so `"false"` (a string) is
  refused and never read as a yes, and an empty path is refused.
- An answer warns when the `arclang` it runs is not version 4.0.1.
- Nothing leaves the machine unless the call says so (`arclang_review` with
  `send: true`), or an `ANTHROPIC_API_KEY` is set for the generation tools.
- What the compiler cannot do, the server does not pretend to do
  (`arclang_git_merge` answers that there is no merge).

## Install

```bash
# 1. The compiler
cargo build --release              # from the repository root
./install-arclang.sh               # copies the binary to ~/.local/bin/arclang
arclang --version                  # arclang 4.0.1

# 2. The server
cd mcp-server
pip install -e .
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
| `arclang://syntax-rules` | The rules most often broken, and a model that compiles and passes the gate |
| `arclang://reference-model` | That model alone |
| `arclang://metamodel` | Element kinds, typed attributes, enumerations, units, trace rules, printed by the compiler |
| `arclang://grammar` | `spec/GRAMMAR.ebnf`, when the server runs from the repository |

## Tools

### Check and analyse

| Tool | Command | Returns |
|---|---|---|
| `arclang_validate` | `check --lint` | Errors with line and column; counts of compilation warnings (typed attributes and constraints block the gate), traceability warnings, advisories |
| `arclang_compile` | `build` | Counts of the compiled model |
| `arclang_gate` | `gate` | PASS or FAIL and each blocker with its rule |
| `arclang_trace_analysis` | `trace --validate` | Coverage, issues, the matrix with rationales |
| `arclang_impact` | `impact` | What a change to one element reaches, by distance |
| `arclang_diff` | `diff --json` | Semantic diff by stable identity |
| `arclang_info` | `info --metrics` | Element counts |
| `arclang_metamodel` | `metamodel` | The metamodel, or one element kind |
| `arclang_safety_check` | `safety` | Counts of declared safety content (not a verdict) |
| `arclang_hazard_analysis` | `export -f json` | The hazards and FMEA entries the model declares |
| `arclang_review` | `review` | Plausibility of each trace, from TypeSafe's Jev. Dry run unless `send: true` |

### Edit

All four keep comments, order and layout, and are refused unless the edited
model compiles. Without `write: true` they return a diff and change nothing.

| Tool | Command |
|---|---|
| `arclang_format` | `format` |
| `arclang_set_attribute` | `set` |
| `arclang_unset_attribute` | `unset` |
| `arclang_rename` | `rename` |

### Export and diagrams

| Tool | Command | Returns |
|---|---|---|
| `arclang_export` | `export -f <format>` | `sys-ml`, `sysml-json`, `capella`, `req-if`, `c-headers`, `proto`, `json`, ... |
| `arclang_viewpoint_diagrams` | `diagram -f viewpoints\|elk\|html` | The Arcadia viewpoint diagrams (OAB, SAB, LAB, PAB, modes and states, scenarios) |
| `arclang_generate_diagram` | `diagram -f <type>` | One SVG |
| `arclang_generate_all_diagrams` | `diagram -f <type>`, each type | The files written, and why a type was not |
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
cover every tool, the resources against the compiler, and a session over stdio.
They pass with the MCP Python SDK 1.x and 2.x.

## Files

```
src/arclang_mcp/
  server.py              lists the tools and resources, routes the calls
  compiler/wrapper.py    the only place that writes arclang command lines
  tools/core.py          check, build, gate, trace, impact, diff, export, metamodel, review
  tools/edit.py          format, set, unset, rename
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
