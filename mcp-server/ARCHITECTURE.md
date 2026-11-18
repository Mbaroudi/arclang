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
tools/*.py           one handler per tool: resolve paths, write the command
        |            line, prefix the compiler's output with the outcome
        v
compiler/wrapper.py  ArcLangCompiler.run: the one place a command is run
        |  subprocess, argument list, no shell
        v
arclang              every subcommand but those tools/cli.py leaves out by
                     name (serve, lsp, repl, explorer, clean, sync, plugin)
```

## Modules

| Module | Role |
|---|---|
| `server.py` | Builds the tool table from the groups, serves the resources, runs over stdio. Works with the MCP Python SDK 1.x (decorators) and 2.x (handlers passed to the constructor) |
| `compiler/wrapper.py` | `ArcLangCompiler.run`, which every command goes through and which returns a `Run` (command, exit code, stdout, stderr); a method per subcommand for the first tools; the lists of formats, views, kinds and operations, held equal to the CLI's by the tests; parsers that read only lines the binary prints |
| `tools/cli.py` | The server against the command line: `COVERS` (tool to subcommands), `NOT_EXPOSED` (subcommands no tool runs, with reasons), `OPTIONS` (each option that is not an argument of the same name: reached under another name, passed by the server, or left out with its reason) |
| `tools/common.py` | Path resolution, `CommandLine` (options written `--option=value`, positionals after `--`, every path resolved), the guards for program names and serial ports, the shape of an answer (`report`) |
| `tools/core.py` | Check (text, JSON or SARIF), rules, build, gate, realize, partition, schedule, trace (validate, suspects, review), impact, diff, export, metamodel, review |
| `tools/simulation.py` | Verify, the stages (cosim, sil, pil, hdl, soc, hil), twin, observe, debug: one tool per command, each with a bounded time |
| `tools/versioning.py` | Merge by element, history of an element, restore at a revision, baselines |
| `tools/delivery.py` | Tables, documents, a new model, the pull-request report, SARIF, the pipeline files |
| `tools/recover.py` | Prove, recover (generated C, a device tree, a binary, a proposal), drift |
| `tools/edit.py` | Format and the lossless edits, with a diff preview |
| `tools/workbench.py` | The compiler's workbench API, as the VS Code extension uses it: `arclang workbench` reduced to the picture its agent reads (counts, elements by id, one element with its relationships, findings, gate, diagrams) or one raw section; `arclang edit` applied request by request on a copy in a private temporary directory, the whole list refused when one request is, the file written only with `write: true` |
| `tools/transform.py` | Import (writes only at `output_path`, then checks the result) and the Arcadia transition (plan as JSON, or the edited model as a diff until `write: true`) |
| `tools/generation.py` | Snippets compiled before they are returned; diagrams |
| `tools/safety.py` | Safety summary; hazards the model declares |
| `tools/expert.py` + `mbse_expert.py` | Keyword sorting of requirements, skeleton, SVG checks |
| `tools/batch.py` | Several operations in one call, each reduced |
| `resources/` | The syntax rules, the reference model and the shorter examples the rules show |
| `ai/generator.py` | Snippet templates; a Claude model when `ANTHROPIC_API_KEY` is set |
| `utils/config.py` | `.arclang-mcp.toml` and environment |

Each tool group declares its tools (`TOOLS`) next to their handlers, so a
tool's description and its behavior are read and changed together.

## Rules the code holds to

1. **Every command goes through `ArcLangCompiler.run`**, as an argument
   list, with only options the CLI has. The first tools call a method of
   the wrapper; the later ones write their command line next to their
   schema with `CommandLine`. Positional arguments follow `--`, and an
   option's value is written `--option=value`, so a value such as `-5`,
   `--help` or a file named `--write` is never read as an option.
   `tools/cli.py` says which subcommand each tool runs, and the tests hold
   it to `arclang --help`: a subcommand or an option no tool reaches fails
   a test until it is offered or left out there by name.
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
   does it), so a flag is a real boolean. Edits need `write: true`, and
   `arclang_edit` changes nothing at all when one of its requests is refused:
   the requests run on a temporary copy, and the file is written once, with
   the final text.
   `arclang_review` is a dry run without `send: true`. The generation tools
   call a Claude model only when a key is configured, and say which wrote the
   text. A run that takes minutes or needs hardware is asked for by name
   (`run: true`, `prove: true`, `verify: auto`, `live` with `for_seconds`)
   and has a bounded time.
6. **A call names no program to run.** What an agent read can ask it for any
   command, as for any path. A C compiler is a bare name on the `PATH`; a
   serial port is a device name; a command line (`recover --with`,
   `hil --command`) is passed on only when the person who started the
   server set `ARCLANG_MCP_ALLOW_COMMANDS=1`. `--open` is never passed.
7. **Nothing is invented.** The requirement analysis assigns a safety level
   only when the text states one; the skeleton writes no trace, actor or
   hardware; a diagram type without a check gets no score.

## The program that runs

The compiler is `ARCLANG_BINARY`, else `arclang` on the `PATH`. The workspace
configuration file cannot set it. When that program is older than the
compiler this server is written for (`MINIMUM_COMPILER_VERSION` in
`server.py`), every answer carries a warning.

## Paths

A relative path is resolved from the workspace (`ARCLANG_WORKSPACE`, else the
current directory); `~` is expanded. Every path a tool is given, to read or to
write, goes through `resolve_path` (`tools/common.py`), and is refused when it
leads outside the workspace and outside the directories named in
`ARCLANG_MCP_ROOTS` (separated like `PATH`; `*` there is any path). Where a
path leads is judged with `..` and links followed, so neither is a way out.

The paths come from an agent, and what the agent read (a model received from
someone, a document, a web page) can ask it for any path: a tool that writes
must not be led to a file elsewhere on the machine. This bounds the paths the
tools are given, and nothing more: the server and the compiler it runs keep
the rights of the user who started them (a model can still import a file by
its path, and a simulation runs what the model says). It is a local tool, not
a sandbox.

## Tests

`tests/` runs the real binary: every tool, the resources against the compiler,
the failure paths (a model that does not compile, an unknown element, a missing
key), and a full session over stdio. `test_cli_parity.py` reads the CLI's
`--help` and holds `tools/cli.py` and the lists of the wrapper to it;
`test_command_lines.py` checks that each argument said to reach an option
writes it. The one stand-in is the `recorded` fixture: for the commands that
simulate, build for a target, prove or drive a board, the command line is
recorded and the compiler is not run. See the README for how to run them.
