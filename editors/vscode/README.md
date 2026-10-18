# ArcLang for VS Code

[ArcLang](https://arclang.fr) is Arcadia MBSE as code: a textual language and
a strict compiler for the Arcadia method (operational analysis, system
analysis, logical and physical architectures, EPBS), with Capella round-trip,
SysML v2 import and export, viewpoint diagrams and a production gate.

This extension brings the compiler into the editor. Everything it shows or
writes comes from the `arclang` binary: it never says more than the compiler
does.

## In the editor

- **Problems** with their rule (`trace.component`, `metamodel.type`, ...)
  pointing at the element concerned. The catalogue is **ArcLang: Show the
  catalogue of rules**.
- **Outline**, **completion** of keywords, attributes and enumeration values
  from the metamodel, **hover** documentation, **go to definition**,
  **rename** through the lossless source editor (comments and layout kept,
  refused when the element would not keep its identity).
- **Formatting** (`arclang format`): whitespace only, order and comments kept.
- **Diagrams** beside the source (**ArcLang: Open diagrams**), the compiler's
  own viewer of the Arcadia views, redrawn on every save.

## From the command line, without leaving VS Code

Every command of the palette is one CLI subcommand:

| Command | Runs |
|---|---|
| ArcLang: Check the model | `arclang check --json` |
| ArcLang: Run the production gate | `arclang gate --standard <setting>` |
| ArcLang: Transition plan, Apply a transition step | `arclang transition --plan`, `arclang transition --step` (applied in the editor, undoable) |
| ArcLang: Change impact of an element | `arclang impact` |
| ArcLang: Semantic diff against the committed version | `arclang diff` between `git HEAD` and the file |
| ArcLang: Export | `arclang export -f sys-ml`, `sysml-json`, `capella`, `req-if`, `c-headers`, `proto`, ... |
| ArcLang: Import | `arclang import -f sysml`, `capella`, `req-if`, ... (any SysML v2 text or API records) |
| ArcLang: Show the language metamodel | `arclang metamodel` |

## With AI

**`@arclang` in the chat view** answers with the compiler at hand. It runs on
the language models VS Code gives it (GitHub Copilot or another provider),
so there is no key to configure.

- `/model` writes or extends a model from a description. The answer is
  compiled before it is shown; when the compiler refuses it, the errors go
  back to the model (setting `arclang.ai.repairAttempts`), and the result
  comes with buttons to replace the open model or open it as a new file.
- `/fix` repairs the errors and findings of the open model, minimally.
- `/explain` explains the model, an element or a finding.
- `/review` reviews the model against `arclang check` and the production gate.
- `/transition` plans the next Arcadia step and offers to apply it.

**Agent mode and other assistants** get the compiler as tools:
`#arclangCheck`, `#arclangGate`, `#arclangMetamodel`, `#arclangRules`,
`#arclangImpact`, `#arclangTransitionPlan`, `#arclangSet`, `#arclangRename`.
Reading tools run without asking; the two editing tools ask for confirmation
and apply their change in the editor as an undoable edit, after the compiler
accepted it. Tools only touch files of the workspace or open in the editor.

**The ArcLang MCP server** (`mcp-server/` of the repository, Python) is
offered to agent mode when `arclang-mcp` is on the PATH or the setting
`arclang.mcp.command` names how to start it, for example
`uv run --directory /path/to/arclang/mcp-server arclang-mcp`.

## Requirements

The `arclang` compiler, 4.0.1 or later: a
[release](https://github.com/Mbaroudi/arclang/releases) on the PATH, or its
path in the setting `arclang.binary`. **ArcLang: Install or locate the
arclang compiler** helps. Chat features need a language model provider in
VS Code (GitHub Copilot, for instance); the rest works without.

## Settings

| Setting | Default | Meaning |
|---|---|---|
| `arclang.binary` | `arclang` | The executable |
| `arclang.diagrams.refreshOnSave` | `true` | Redraw open diagrams on save |
| `arclang.gate.standard` | `iso26262` | `iso26262`, `do178c` or `iec61508` |
| `arclang.ai.repairAttempts` | `2` | Corrections asked when a written model does not compile |
| `arclang.mcp.command`, `arclang.mcp.args` | | How to start the MCP server |

## Development

```bash
cd editors/vscode
npm install
npm run build              # dist/extension.js
npm run test:unit          # pure helpers, node:test
npm run test:integration   # in a VS Code instance; ARCLANG_BINARY=/path/to/arclang to point at a compiler
npx @vscode/vsce package --no-dependencies   # the .vsix
```

Open the folder in VS Code and press F5 to run the extension on the
repository's examples. The sample model the assistant is shown,
`media/reference.arc`, must compile and pass the gate: CI checks it.

## License

MIT, like the compiler.
