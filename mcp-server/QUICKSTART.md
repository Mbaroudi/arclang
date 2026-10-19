# ArcLang MCP Server: quick start

## 1. Install

```bash
cargo build --release && ./install-arclang.sh    # from the repository root
arclang --version                                # 4.3.0 or later
cd mcp-server && pip install -e .
python test_install.py                           # imports, and finds a compiler it can work with
```

## 2. Connect a client

Claude Desktop, in `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "arclang": {
      "command": "arclang-mcp",
      "env": { "ARCLANG_WORKSPACE": "/path/to/your/models" }
    }
  }
}
```

Claude Code:

```bash
claude mcp add --env ARCLANG_WORKSPACE=/path/to/your/models arclang -- arclang-mcp
```

Restart the client. It should list 35 `arclang_*` tools.

## 3. First steps

Put a model in the workspace. The reference model is a good start:

```bash
cp mcp-server/src/arclang_mcp/resources/reference_model.arc /path/to/your/models/braking.arc
```

Then ask, in plain words:

| You ask | The assistant calls | You get |
|---|---|---|
| "Does braking.arc compile?" | `arclang_validate` | Errors with line and column, warnings by kind |
| "What does this warning mean, and how do I fix it?" | `arclang_validate` (`as_json`), `arclang_rules` | Each finding with its rule and fix; the rule's meaning and remedy |
| "Is it ready for a design review?" | `arclang_gate` | PASS, or FAIL with each blocker |
| "What does REQ-BRK-001 affect?" | `arclang_impact` | Affected elements, by distance |
| "What is in this model? What is LC-002 connected to?" | `arclang_workbench` | Counts by kind, the elements with their ids and attributes, one element with its relationships and traces, the findings, the gate, the diagrams |
| "Add a logging function to the brake controller and trace it to SF-002" | `arclang_edit` | The requests applied in order through the compiler's editor, as a diff, then the write once you agree |
| "Set the latency of LF-002 to 8 ms" | `arclang_set_attribute` (or one `set` request of `arclang_edit`) | A diff, then the write once you agree |
| "What changed since yesterday's copy?" | `arclang_diff` | Elements and traces added, removed, modified |
| "What is left to declare on the system level?" | `arclang_transition` | The plan of each Arcadia step; then the declarations, as a diff until you say `write` |
| "Export it to SysML v2" | `arclang_export` (`sys-ml`) | The textual notation |
| "Read this .sysml file into ArcLang" | `arclang_import` (`sysml`) | The .arc file, what was counted and left unresolved, and whether it compiles |
| "Show me the logical architecture" | `arclang_viewpoint_diagrams` | An HTML viewer of the viewpoint diagrams |

A recorded session, with the real answers, is in
[examples/example_session.md](examples/example_session.md).

## 4. Writing ArcLang with an assistant

The compiler is strict, so the assistant must not guess the syntax. The server
gives it three resources to read first: `arclang://syntax-rules`,
`arclang://reference-model` and `arclang://metamodel`. Ask it to run
`arclang_validate` after every change: a model the compiler has not accepted
is not a model yet.

To change an existing model, the assistant should not rewrite the file: it
reads the ids with `arclang_workbench` and sends precise requests to
`arclang_edit` (add, set, move, trace...), which the compiler applies while
keeping comments and layout, and refuses as a whole when one request would
break the model.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| Every tool answers `cannot run 'arclang'` | The binary is not on the client's `PATH`. Set `ARCLANG_BINARY` to its full path in the client configuration |
| `Expected LeftBrace, got Integer(-1)` on a `req` line | The requirement id must be a string: `req "REQ-001" "Title" { }` |
| `metamodel.unit` finding on `latency: 25` | Quantities are unquoted numbers with a unit: `latency: 25 ms` |
| `arclang_transition` answers `no element with id or name 'physical_architecture'` | The target layer is an unnamed `architecture physical { }` block: pass `into` (the node that receives the components), or name the block (`physical_architecture "PA" { }`) |
| `arclang_suggest_architecture` answers "No suggestion" | It asks a Claude model: set `ANTHROPIC_API_KEY` |
| `arclang_review` with `send: true` fails | It needs `TYPESAFE_API_KEY` |
| A relative path is not found | Paths are resolved from `ARCLANG_WORKSPACE` |
