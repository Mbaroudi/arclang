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
  name; ports are declared when the ends have none). **Drag a box anywhere
  on the diagram, resize it by the grip on its corner**: the arrangement is
  kept in `<model>.layout.json` beside the model. Let a box go on another
  to move it inside. `F2` renames, `Delete` removes,
  double-click goes to the source (see below), `Esc` cancels; right-click gives the native menu
  (rename, delete, show in source, show in tree, ask @arclang, copy id).
  **Full screen** gives the workbench the whole screen with all its tools
  (Esc gives the window back); **Save SVG** and **Save PNG** export the
  diagram as arranged, through the save dialog.
- **Diagram contents**: `⌘H` hides an element from the diagram (out of a
  named diagram's `shows:`, or a new named diagram without it on a layer
  view), *Show related* adds what it is connected to, *New diagram* opens a
  diagram around it; a **Trace** tool draws realizes, satisfies, allocates,
  verifies traces; `F2` renames on the canvas; `⌘F` finds in the diagram;
  exchanges and links are selectable and deletable, cascades are confirmed.
  A **Requirement** tool creates a requirement satisfied by the selection,
  *Satisfies…* traces an element to an existing one; a **Fragment** tool
  wraps the selected message of a scenario in a loop, opt, alt, par or
  critical fragment; Shift+click selects several elements, to delete, hide
  or set one attribute on all of them.
  See [COMPARISON.md](COMPARISON.md) for where this stands against Capella
  and SysML v2 tools.
- **Properties**, Capella style: General, Description, Resources and All
  tabs, the set attributes first and the others added on demand, the name
  edited in place (the identity never changes).
- **Model tree** and **Relations** views in the ArcLang side bar: the
  diagrams, then the elements by layer and ownership; owner, incoming,
  outgoing, traces and owned of the selection, each a step away. To put a
  tree node on a diagram, right-click it › **Show in the diagram**, or drag
  it onto the diagram **with Shift held**: VS Code drops into an editor or a
  webview only while Shift is pressed.
- **Navigation at scale**: *Open a diagram…* (`⌘⌥D`) and *Go to an element…*
  (`⌘⌥E`) quick picks; the status bar shows the gate verdict and the number
  of findings; findings are marked on the drawn elements, with their text as
  tooltip.

### Diagram and text side by side

A **single click** on a box, a line, a port or a table row only selects: the
properties, the tree and the other tabs follow, and nothing else moves. It
never opens, replaces or focuses an editor and never touches the text's cursor
or scroll position. When the model's text is already on screen in its own
editor group, the element's declaration is **highlighted** there (scrolled into
view only if it is out of the viewport); `arclang.workbench.followSelection`
set to `off` leaves the text alone.

A **double-click** on a box or a line goes to the source: the declaration is
selected in the text editor, which takes the focus. When the text is not on
screen it opens **beside** the workbench, in an editor group of its own (one is
created when the workbench is alone), never over the workbench's tab. The
*Source* button, *Go to source* of the properties, the tree and the tables use
the same rule. In full screen the selection does not follow into the text.

draw.io renames in place on a double-click; ArcLang goes to the source because
the text is the model. Set `arclang.workbench.doubleClick` to `rename` for
draw.io's gesture. `F2` and the menu always rename, and **Go to source** is
always in the context menu.

### New and Open

- **New model…** (command palette, the model tree's *New* menu, the empty
  tree's welcome view, the workbench's *New* menu): a blank model (a header),
  the Arcadia skeleton (the four levels, empty) or an example; it asks for the
  name and place, writes the `.arc` and opens the text and the workbench side
  by side.
- **New layer…**: Operational, System, Logical, Physical, EPBS, Requirements
  or Safety, among those the model does not hold yet. Written through
  `arclang edit` (checked by the compiler, one undo); a new level appears as an
  empty tab with its palette.
- **New diagram…**: a view kind, the starting elements (the selection, or ones
  you pick), a name; the `diagram` declaration is written and opens as a tab.
- **Open a diagram…** (also *Diagrams…* in the layer bar): the model's diagrams
  by layer, the declared and the automatic ones; the button of a row opens it in
  a tab of its own. **Open model…** opens a `.arc` or `.sysml` file.

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

## Semantic git: compare, history, baselines, merge

A model is versioned like code but reviewed and merged like a model (`docs/SEMANTIC_GIT.md`).

- **Compare ▾** (tool row of the workbench, or *ArcLang: Compare the model with a branch, tag, baseline or HEAD…*): the diagrams are drawn with the changes marked: added green, modified amber, removed ghosted (dashed red) where they were, moved and renamed with a badge. The layer tabs count the changes; a panel lists them by layer with the gate before and after, the new blockers and the requirements left unsatisfied or unverified; a click selects the element, with *Source*, *Diff* and *History*. The comparison is read-only; *Exit* goes back to editing.
- **History…** on an element (context menu, Properties): the commits that changed it, and the model at one of them beside the current one.
- **Create baseline…** (the gate result first), **Compare with a baseline…**, **Verify a baseline…**: `baselines/<name>/` with the gate report, the coverage table, the exports and a manifest of SHA-256 hashes, and an annotated tag `baseline/<name>`.
- **Install the semantic merge driver**: `git merge` then merges `.arc` and `.layout.json` files by element: different elements, or different attributes, never conflict; a real conflict is marked around the element and listed in the **Resolve conflicts** view (*Accept ours*, *Accept theirs*, *Edit*).

## CI/CD: the gate in your pipeline

The compiler runs in your pipeline (`docs/CI_CD.md`): every model compiled, the production gate, the simulations that simulate, a SARIF file for code scanning, and on a pull request one comment (semantic diff, impact, the gate before and after, new blockers) that is updated on each push.

- **ArcLang: Configure CI**: pick the CI systems (GitHub Actions, GitLab CI, Azure DevOps, Jenkins; the one the git remote points at is ticked), read each file in a preview, then write them with `arclang ci init`. Nothing is written until you choose *Write*; a file that exists is replaced only after a second confirmation. The files are not run against the provider by ArcLang: the first pipeline run is their test.
- **A status item** in the status bar for a repository with the GitHub workflow: the latest run of the current branch (passed, failed, running, or `?` when it was cancelled or skipped: only a success is shown as passing). It is read with the GitHub CLI (`gh`), with its own login: ArcLang stores no token, and says in words when `gh` is missing or not logged in. Nothing is read in an untrusted workspace. Settings: `arclang.ci.statusBar`, `arclang.ci.pollSeconds`, `arclang.ci.gh`.

## Simulation with OpenModelica

A **Simulation** status item (when the workspace has a model that simulates or a `.mo` file) shows whether OpenModelica is ready (`OpenModelica (Docker)`), needs setup, or the verdicts of the last run. Its menu (*ArcLang: Simulation*): run the simulations (progress, cancel, then the verdict table), *Check my setup* (a checklist with a one-click fix for Docker, the image, the Modelica library and Modex), a guided setup, export Modelica, open the Modelica drawing, the documentation. Running with nothing installed opens the guided setup rather than an error. The first run installs the Modelica library (minutes, once). Settings: `arclang.simulation.image`, `arclang.simulation.omc`, `arclang.simulation.timeout`. Needs the compiler's `verify --check-tools`. See `docs/modelica.md`. Desktop only.

## Importing a model

Bring a model in from the model view's **Import** button (cloud icon), its welcome when the workspace has no `.arc` file, the Explorer's context menu, or *File ▾ › Import a model…* in the workbench. One quick pick asks what you import:

| Source | What is read |
|--------|--------------|
| Capella model | a `.capella` (or `.melodymodeller`), its `.aird` (read through the `.capella` beside it), or a project folder; every layer, the requirements, the diagrams |
| SysML v2 file(s) or folder | one `.sysml`/`.kerml`, several selected together, or a folder, as one model; a `.json` listing of API records |
| SysML v2 API server | base address (https, or http on this machine), optional token (kept in the secret storage), then the project and the commit; every element is read page by page |
| ReqIF / DOORS | a `.reqif` or `.reqifz`; into the open model (one undoable edit, checked with it) or a new `.arc` |

A `.capella`, `.sysml` or `.reqif` file dropped on the workbench sheet (Shift held) is offered for import too. The **import report** says what was imported and what was left out and why, whether the written model compiles, or the refusal and the `.rejected.arc` the compiler kept; it has *Open workbench*, *Open model* and *Run check*.

## Saving and saving as

The workbench toolbar has a **Save** button: a ● shows while the model has unsaved changes (also in the tab's title), ⌘S / Ctrl+S on the canvas saves it without touching the canvas's undo history, and a `.sysml` is written back as SysML v2. With `files.autoSave` on it reads *Saved*. **File ▾ › Save as…** writes SysML v2, Capella, ReqIF or a copy as `.arc` through `arclang export`.

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
| ArcLang: Import | `arclang import -f sysml`, `capella`, `req-if`, ... (see *Importing a model*) |
| ArcLang: Show the language metamodel | `arclang metamodel` |

## With AI

MBSE as code with an AI at hand, the way Cursor works on source code: select
part of a diagram (or of the text), ask, look at the proposed change, apply
it or not. The AI proposes; **the compiler decides**: every proposed change
is a list of `arclang edit` requests run on a copy of the model first, and
nothing changes until you click Apply (one undoable edit).

### Ask AI on a selection (⌘I)

- In the workbench, select one or more boxes or lines (click, Shift+click, or
  a band), then **⌘I** (Ctrl+I), **✦** in the floating toolbar, or **Ask AI…**
  in the context menu. A prompt opens by the selection, with quick actions:
  *Explain*, *Fix the findings*, *Complete* (missing ports, exchanges,
  traces), *Refine* (into sub-components or functions), *Check against
  requirements*.
- The AI is shown the source of the selected elements, their relationships
  and neighbours (`arclang.ai.contextScope`), the findings on them, the
  metamodel of their kinds and the gate. It may read more elements and run
  `arclang impact` itself (read-only tools).
- Its answer comes back as a **proposal**: an explanation, the edits in
  words, the diff of the source, the changed boxes marked on the diagram and
  the findings the change would add. **Apply** (one ⌘Z undoes it), **Open
  diff** (VS Code's diff view), **Refine…** (a follow-up question on the same
  proposal), **Reject**. When the compiler refuses an edit, its error goes
  back to the AI (`arclang.ai.repairAttempts`) before anything is shown.
- In the `.arc` text editor: select some lines, then **⌘I** (when the
  selection is in an ArcLang file), the light bulb (**Ask ArcLang AI about
  the selection…**, and **Fix with ArcLang AI** on a diagnostic), the context
  menu, or **ArcLang: Ask AI about the selection**. The proposal opens in
  the diff view with Apply / Refine / Reject.

⌘I is VS Code's inline chat key; in an `.arc` file with a selection ArcLang
takes it (rebind `arclang.ai.askSelection` in Keyboard Shortcuts to give it
back). ⌘K is left alone: VS Code uses it for its chords.

### Impact-driven suggestions while editing

The workbench's **✦ AI** tab (beside Properties, Relations, Details) shows
what your last edit touched and broke, as the compiler sees it: traces that
went away, requirements no longer satisfied, new findings (latency budget,
interfaces, missing traces…), new gate blockers, and `arclang impact` on the
changed elements. Then, by `arclang.ai.suggestOnEdit`:

- `onRequest` (default): the consequences are shown; **Propose changes with
  AI** asks for proposals. **Analyse the impact** of a selection (the tab, the
  context menu, or **ArcLang: Analyse impact with AI**) does the same for any
  element.
- `afterEachEdit`: when an edit has consequences, the AI proposes one to
  three architecture changes, each with its rationale and the consequence it
  resolves, each compiled on a copy before it is shown.
- `off`: nothing.

Analyses wait for a pause in editing, a newer edit cancels an older one, and
nothing blocks the editor. Suggestions are labelled as AI proposals: Preview,
Apply or Dismiss.

### Choosing the AI

**ArcLang: Configure AI** (or the ✦ item of the status bar) chooses the
provider, the model, the suggestions and the context. Keys are never
settings: **ArcLang: Set AI API key** stores them in VS Code's secret storage
(the OS keychain), **ArcLang: Clear AI API key** removes one, **ArcLang:
Test AI connection** sends one short sentence and shows the answer.

| `arclang.ai.provider` | Setup |
|---|---|
| `vscode-lm` (default) | Nothing: the models VS Code offers (GitHub Copilot, or another extension's). **ArcLang: Choose the AI model** picks one (`arclang.ai.vscodeModel`). |
| `anthropic` | A key from console.anthropic.com. Model `claude-opus-5-5` by default; `claude-sonnet-5-5`, `claude-haiku-5-5`. |
| `openai` | A key from platform.openai.com. Model `gpt-5` by default. |
| `azure-openai` | `arclang.ai.endpoint` = `https://<resource>.openai.azure.com`, `arclang.ai.azureDeployment`, `arclang.ai.apiVersion` (`2024-10-21`), and the resource's key. |
| `mistral` | A key from console.mistral.ai. Model `mistral-large-latest` by default; `codestral-latest`. |
| `google` | A key from aistudio.google.com. Model `gemini-2.5-pro` by default. |
| `ollama` | Ollama running locally (`ollama pull llama3.1`); no key; nothing leaves the machine. `arclang.ai.endpoint` for another host. |
| `openai-compatible` | `arclang.ai.endpoint` = the server's base URL (`http://localhost:1234/v1` for LM Studio, vLLM, llama.cpp, LiteLLM…), `arclang.ai.model`; a key only if the server wants one. |
| `off` | No AI: nothing is ever sent, by any feature. |

Tool calling uses each provider's own format (Anthropic tool use, OpenAI and
Mistral function calling, Gemini function declarations, Ollama tools, VS
Code's language model tools); answers stream. No SDK: plain HTTPS requests,
with a timeout (`arclang.ai.timeoutSeconds`) and cancellation. `@arclang`
and `/agent` run on the configured provider too (with `vscode-lm`, on the
model chosen in the chat, as before).

**What is sent, and to whom.** The first request to a cloud provider (per
provider and endpoint) asks first, saying what is sent and where
(`arclang.ai.confirmBeforeSending`); local servers are not asked. The
**ArcLang AI** output (**ArcLang: Show the AI log**) lists every request:
provider, model, endpoint, purpose, size, tool calls, time; never a key, and
keys are removed from every error message. The provider, the endpoint and
the consent are user settings only: a repository's settings cannot redirect
your requests or your key.

### @arclang in the chat view

**`@arclang` in the chat view** answers with the compiler at hand. With the
default provider it runs on the language models VS Code gives it (GitHub
Copilot or another provider), so there is no key to configure.

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

The `arclang` compiler, 4.8.0 or later: a
[release](https://github.com/Mbaroudi/arclang/releases) on the PATH, or its
path in the setting `arclang.binary`. **ArcLang: Install or locate the
arclang compiler** helps. Chat features need a language model provider in
VS Code (GitHub Copilot, for instance); the rest works without.

## Settings

| Setting | Default | Meaning |
|---|---|---|
| `arclang.binary` | `arclang` | The executable |
| `arclang.diagrams.refreshOnSave` | `true` | Redraw open diagrams on save |
| `arclang.workbench.followSelection` | `highlight` | A click on the diagram highlights the declaration in the visible text (`off`: nothing) |
| `arclang.workbench.doubleClick` | `source` | A double-click goes to the source (`rename`: rename in place, as draw.io) |
| `arclang.gate.standard` | `iso26262` | `iso26262`, `do178c` or `iec61508` |
| `arclang.ai.provider` | `vscode-lm` | The AI: `vscode-lm`, `anthropic`, `openai`, `azure-openai`, `mistral`, `google`, `ollama`, `openai-compatible`, `off` (user setting) |
| `arclang.ai.model`, `arclang.ai.vscodeModel` | provider's default | The model |
| `arclang.ai.endpoint` | provider's API | Base URL; required for Azure and OpenAI-compatible servers (user setting) |
| `arclang.ai.azureDeployment`, `arclang.ai.apiVersion` | | Azure OpenAI |
| `arclang.ai.maxTokens`, `arclang.ai.temperature`, `arclang.ai.timeoutSeconds` | `4096`, provider's, `90` | Answers |
| `arclang.ai.suggestOnEdit` | `onRequest` | `off`, `onRequest`, `afterEachEdit` |
| `arclang.ai.contextScope` | `selectionAndNeighbours` | `selection`, `selectionAndNeighbours`, `wholeModel` |
| `arclang.ai.confirmBeforeSending` | `true` | Ask before the first request to a cloud provider (user setting) |
| `arclang.ai.repairAttempts` | `2` | Corrections asked when what an AI wrote does not compile |
| `arclang.mcp.command`, `arclang.mcp.args` | | How to start the MCP server |
| `arclang.simulation.image`, `arclang.simulation.omc` | empty | Docker image / local `omc` for simulations (user settings) |
| `arclang.simulation.timeout` | `900` | Seconds a simulation may take |

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
- **Nothing leaves the machine unless you ask an AI.** With Ask AI, the
  suggestions or `@arclang`, the context described above goes to the
  provider of `arclang.ai.provider`, and only then; with `off`, never. The
  first request to a cloud provider is confirmed, every request is listed in
  the ArcLang AI output, keys live in the secret storage and are removed from
  every error and log. The extension has no telemetry; otherwise its only
  external link is the GitHub releases page, opened on request.
- **An AI's answer is data.** Only edit requests of known shapes are taken
  from it, each run by the compiler on a copy; nothing it says is executed,
  and nothing is applied without your Apply. The model's text is sent to it
  as data, with the instruction to ignore anything in it that reads as an
  order.
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
