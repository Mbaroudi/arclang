# ArcLang MCP Server: architecture

## One idea

The compiler is the only authority on a model. The server adds no knowledge of
its own about ArcLang: it runs the compiler and hands back what it printed.

```
MCP client (Claude Desktop, Claude Code, ...)
        |  JSON-RPC over stdio
        v
server.py            lists tools and resources, routes each call to its group
        |
        v
tools/*.py           one handler per tool: resolve paths, call the wrapper,
        |            prefix the compiler's output with the outcome
        v
compiler/wrapper.py  the only place that writes an `arclang` command line
        |  subprocess, argument list, no shell
        v
arclang 4.0.1        check, build, gate, trace, impact, diff, export, diagram,
                     format, set, unset, rename, review, metamodel
```

## Modules

| Module | Role |
|---|---|
| `server.py` | Builds the tool table from the groups, serves the resources, runs over stdio. Works with the MCP Python SDK 1.x (decorators) and 2.x (handlers passed to the constructor) |
| `compiler/wrapper.py` | `ArcLangCompiler`: one method per subcommand, returning a `Run` (command, exit code, stdout, stderr). Parsers that read only lines the binary prints |
| `tools/common.py` | Path resolution, the shape of an answer (`report`) |
| `tools/core.py` | Check, build, gate, trace, impact, diff, export, metamodel, review |
| `tools/edit.py` | Format and the lossless edits, with a diff preview |
| `tools/generation.py` | Snippets compiled before they are returned; diagrams |
| `tools/safety.py` | Safety summary; hazards the model declares |
| `tools/expert.py` + `mbse_expert.py` | Keyword sorting of requirements, skeleton, SVG checks |
| `tools/batch.py` | Several operations in one call, each reduced |
| `tools/integration.py` | Tools the compiler does not back: they say so |
| `resources/` | The syntax rules and the reference model |
| `ai/generator.py` | Snippet templates; a Claude model when `ANTHROPIC_API_KEY` is set |
| `utils/config.py` | `.arclang-mcp.toml` and environment |

Each tool group declares its tools (`TOOLS`) next to their handlers, so a
tool's description and its behaviour are read and changed together.

## Rules the code holds to

1. **Only the wrapper writes command lines**, and only with options the CLI
   has. Positional arguments follow `--`, so a value such as `-5` or a file
   named `--write` is never read as an option.
2. **An answer is the compiler's output plus one outcome line.** The outcome
   comes from the exit code, or from a line the compiler prints
   (`PRODUCTION GATE: PASS`). Where the exit code is ambiguous the output
   decides: `arclang diff` exits 1 when two models differ, so the diff is read
   from its JSON; `arclang gate` exits 1 both on FAIL and on a model that does
   not compile, so a verdict is given only when the verdict line is there.
3. **No second description of the language.** The metamodel resource is the
   binary's output; the reference model is compiled, formatted and gated by
   the tests; the examples in the rules are compiled by the tests.
4. **Generated text is checked before it is returned.** A snippet is compiled
   inside the block it belongs to; a skeleton is compiled whole. A refusal is
   returned as a refusal, with the compiler's message.
5. **Writes and network calls are explicit.** Arguments are checked against
   the tool's schema in `server.py` before a handler runs (not every MCP SDK
   does it), so a flag is a real boolean. Edits need `write: true`.
   `arclang_review` is a dry run without `send: true`. The generation tools
   call a Claude model only when a key is configured, and say which wrote the
   text.
6. **Nothing is invented.** The requirement analysis assigns a safety level
   only when the text states one; the skeleton writes no trace, actor or
   hardware; a diagram type without a check gets no score.

## The program that runs

The compiler is `ARCLANG_BINARY`, else `arclang` on the `PATH`. The workspace
configuration file cannot set it. When that program is not version 4.0.1,
every answer carries a warning.

## Paths

A relative path is resolved from the workspace (`ARCLANG_WORKSPACE`, else the
current directory); `~` is expanded; an absolute path is used as given. The
server runs with the rights of the user who started it and reads or writes
where that user can: it is a local tool, not a sandbox.

## Tests

`tests/` runs the real binary: every tool, the resources against the compiler,
the failure paths (a model that does not compile, an unknown element, a missing
key), and a full session over stdio. See the README for how to run them.
