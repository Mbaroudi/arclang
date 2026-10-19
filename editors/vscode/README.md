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

## The workbench: text or mouse, your choice

**ArcLang: Open the workbench beside the source** (the icon in the editor
title, or *Reopen Editor With… → ArcLang Workbench*) puts the compiler's
diagram viewer next to the text, with what a Capella user expects around it.
Text and views edit the same `.arc` file: type in one, click in the other,
the selection follows; Undo, Redo and Save are the editor's own.

- **Palette** per view, in a rail with one-letter shortcuts: create actors,
  entities, activities, functions, components, nodes, ports, classes, states
  and modes; connect them by interactions, functional and component
  exchanges, physical links, transitions and messages (two clicks, then a
  name; ports are declared when the ends have none). Drag a function or a
  component onto another to move it. `F2` renames, `Delete` removes,
  double-click opens, `Esc` cancels; right-click gives the native menu
  (rename, delete, show in source, show in tree, ask @arclang, copy id).
- **Properties**, Capella style: General, Description, Resources and All
  tabs, the set attributes first and the others added on demand, the name
  edited in place (the identity never changes).
- **Model tree** and **Relations** views in the ArcLang side bar: the
  diagrams, then the elements by layer and ownership; owner, incoming,
  outgoing, traces and owned of the selection, each a step away.
- **Navigation at scale**: *Open a diagram…* (`⌘⌥D`) and *Go to an element…*
  (`⌘⌥E`) quick picks; the status bar shows the gate verdict and the number
  of findings; findings are marked on the drawn elements, with their text as
  tooltip.

Every gesture becomes `arclang edit` requests: the compiler refuses the ones
that would leave a model that does not compile, and an accepted one lands in
the editor as a single change that Undo takes back. Comments, order and layout
of the text are kept.

**SysML v2 files** (`.sysml`) open in the same workbench: the compiler imports
the text for the tree and the views, and writes an edited model back through
its SysML v2 export (the Arcadia profile, ids kept). The text is then the
compiler's rendering of it: comments and layout of the original are not kept,
unlike `.arc` files. **ArcLang: Import** converts a `.sysml` into a `.arc`
file when you prefer to continue in ArcLang.

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
- `/agent` changes the open model step by step: it reads the elements and
  their ids through the compiler, applies precise edits (add, connect, move,
  trace, rename, set) that you confirm one gesture at a time, retries what the
  compiler refuses, and ends with a check and the gate.

**Agent mode and other assistants** get the compiler as tools:
`#arclangModel` (the elements, ids, relationships and findings of a model),
`#arclangEdit` (a list of edits applied as one undoable change),
`#arclangCheck`, `#arclangGate`, `#arclangMetamodel`, `#arclangRules`,
`#arclangImpact`, `#arclangTransitionPlan`, `#arclangSet`, `#arclangRename`.
Reading tools run without asking; the editing tools ask for confirmation and
apply their change in the editor as an undoable edit, after the compiler
accepted it. Tools only touch files of the workspace or open in the editor.

**The ArcLang MCP server** (`mcp-server/` of the repository, Python) is
offered to agent mode when `arclang-mcp` is on the PATH or the setting
`arclang.mcp.command` names how to start it, for example
`uv run --directory /path/to/arclang/mcp-server arclang-mcp`.

## Install

- VS Code: [ArcLang on the Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=arclang.arclang)
- VSCodium, Cursor, Gitpod, Theia and others: [ArcLang on Open VSX](https://open-vsx.org/extension/arclang/arclang)
- By hand: the `.vsix` attached to every [release](https://github.com/Mbaroudi/arclang/releases),
  then **Extensions: Install from VSIX...**.

## Requirements

The `arclang` compiler, 4.3.0 or later: a
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

## Security

- **Only the compiler runs, never a shell.** Every command is `arclang` with
  an argument list (no shell, no string interpolation). The executable and the
  MCP server command are user settings (`scope: machine`): a repository's
  `.vscode/settings.json` cannot choose what the extension runs.
- **Untrusted workspaces.** In a workspace VS Code does not trust, the
  compiler is not run on its files: no diagnostics, commands, diagrams or
  tools until the workspace is trusted.
- **Diagrams** are drawn in a webview under a content security policy: only
  the viewer's own scripts (marked with a fresh nonce) run, no network, no
  local resources, no command URIs, no message channel to the extension. The
  model's text is data in that page, never a script.
- **Language model tools** read only files of the workspace or open in the
  editor (symbolic links are resolved before the check). The two tools that
  edit a model ask for confirmation, go through the compiler's source editor,
  and apply undoable edits. The assistant is told that the content of models
  and compiler outputs is data, not instructions.
- **Temporary files** live in private directories (mode 0700, unguessable
  names) removed after use.
- **Nothing leaves the machine** through the extension itself. With
  `@arclang`, the open model, its diagnostics and the compiler outputs are
  sent to the language model provider you configured in VS Code, and only
  then. The extension has no telemetry and makes no network request; its
  only external link is the GitHub releases page, opened on request.
- **Supply chain.** One runtime dependency (`vscode-languageclient`), a
  committed lockfile, `npm audit` in CI, Dependabot updates.

Report a vulnerability as described in the repository's `SECURITY.md`.

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

### Publishing a version

The extension is published under the publisher `arclang` on both registries,
from the same `.vsix`. A version number can be published only once on each.

1. Bump `version` in `package.json` and describe the change in `CHANGELOG.md`.
2. Tag the repository `vX.Y.Z`. The release workflow builds the `.vsix`,
   attaches it to the GitHub release and, when the repository secrets
   `VSCE_PAT` (Azure DevOps token, scope *Marketplace: Manage*) and
   `OVSX_PAT` (Open VSX access token) are set, publishes it to the
   Marketplace and to Open VSX.
3. By hand, with the same tokens in the environment and never on the command
   line of a shell history:

   ```bash
   npx @vscode/vsce verify-pat arclang          # reads VSCE_PAT
   npx @vscode/vsce publish --packagePath arclang-vscode-X.Y.Z.vsix
   npx ovsx publish arclang-vscode-X.Y.Z.vsix   # reads OVSX_PAT
   ```

Publisher pages: [Marketplace](https://marketplace.visualstudio.com/manage/publishers/arclang),
[Open VSX](https://open-vsx.org/namespace/arclang).

## License

MIT, like the compiler.
