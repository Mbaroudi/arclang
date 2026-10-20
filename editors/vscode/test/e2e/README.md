# Workbench end-to-end tests

The workbench page (the webview of the VS Code extension) in a real browser,
with a host that edits through `arclang edit` and snapshots through
`arclang workbench`, exactly as the extension does:

- `palette_test.js`: every palette tool of every view (37 tools), each edit
  written in the model text and the model still compiling;
- `gestures_test.js`: trace, hide from a diagram (viewpoint and named),
  show related, new diagram around an element, rename in place (F2), find
  (⌘F), properties (set, rename), delete an element and an exchange,
  connect by dragging from the handle, multi-selection and bulk delete, minimap, requirement, satisfies, fragment,
  bulk attribute edit (21 gestures);
- `editing_test.js`: changing what is drawn: reconnect component and
  functional exchanges, interactions, transitions and messages by their end
  handles, reverse, retarget a trace, rename exchanges and boxes in place,
  edit a link's attributes, move a port, copy, cut, paste, duplicate, a
  refused paste with its reason, band selection, align, distribute, nudge,
  smart guides, bends, quick connect, the context menu, the shortcuts list,
  and the cases a review raised (two bends kept, bends that follow a reversed, renamed
  or reconnected line, a refused cut kept, keys left to the fields, Esc in a
  field, a rename field closed on a model change) (43 checks; `SHOTS=dir`
  saves pictures);
- `panes_test.js`: the properties pane: a quantity of the wrong dimension
  refused as it is typed, the unit picker, an enumeration, a reference
  picker (kind checked, jump link, keyboard), the compiler's refusal shown
  before anything is written, Redefine and Reset to inherited, a refusal on
  apply kept on its field, relationships, traces with verdicts, findings
  with their fix, the Markdown preview, the common attributes of a
  multi-selection, a tree element dropped on a diagram, the section keys
  (33 checks);
- `toolbar_test.js`: every toolbar tool (exports reach the host as SVG,
  PNG and JSON, full screen, zoom, filters, layers, folding), no console error.

- `vscode_real_test.js`: the workbench inside VS Code itself, driven by
  Playwright through Electron: the toolbar is whole, a box dragged stays
  where it is let go, a box is resized by its grip, both are written to
  `model.layout.json` and back when the model reopens, PNG and SVG exports
  go through the save dialog, full screen keeps every tool and Esc leaves
  it, a box let go on a component moves into it, the end of an exchange
  dragged onto a box reconnects it, right-click opens the canvas's menu,
  Align bottom lines two boxes up, ⌘D twice in a row makes two copies
  (20 checks).
- `vscode_drawio_test.js`: draw.io's gestures inside VS Code: lines go
  around the boxes after a move and stay on their ports, the grid (Alt to
  place freely), eight resize handles keeping what a box holds in place,
  straight and curved lines kept in the layout file, a port slid along its
  box, a name dragged off its line, an end snapping onto a box near the
  pointer or over its body, a port let go on another box moved there, Reset
  layout, the full screen kept when a box is selected (29 checks; keys sent as an AZERTY Mac sends them).

- `vscode_panes_test.js`: the panes in VS Code itself: quantity, enumeration
  and reference edited in the pane, refusals as they are typed, ⌘Z (AZERTY),
  redefine and reset of an inherited value, the tree's kind icons, diagrams
  under their layer, the filter, "Show in the diagram", a diagram opened in
  a tab of its own ("<model> — <diagram>") beside the workbench, the
  selection shared by both (20 checks; whether VS Code hands a tree drag to
  the webview is printed as a note).

- `vscode_ai_test.js`: the AI features in VS Code itself, against the local
  fake provider `../ai/fake-llm-server.js` set as an OpenAI-compatible server
  (no network, no key): the key command, the status bar, ⌘I on two boxes, a
  proposal previewed (diff, marks), applied, compiled and undone with ⌘Z; an
  ask on a text selection applied from its notification; a suggestion after
  an edit that removes a trace, applied; the log without the key (30 checks).

- `tables_test.js` (run by `run.sh`): the table view in a browser, on tables
  the compiler computed from `tests/fixtures/tables/braking.arc`: rows,
  sorting, the filter bar and the column filters, only gaps, a cell edited
  (text and enumeration), links, Copy and exports reaching the host, the
  column picker, dark and high-contrast themes, matrix clicks (create,
  remove), the N² diagonal, 6000 rows drawn only where the view is.
- `vscode_tables_test.js`: the tables in VS Code itself on a copy of the
  flagship example: the coverage table in its own tab, the filter, a cell
  edited (the source changes, ⌘Z typed on AZERTY takes it back), CSV and
  Excel through the save dialog (the workbook read back with openpyxl when
  `OPENPYXL_PYTHON` names a Python that has it), Copy as TSV (`pbpaste`), a
  trace created from a matrix and removed after the confirmation.

Run `./run.sh` from this folder after `cargo build --release` at the root.
The VS Code test runs on its own, with the compiler on the PATH (or
`ARCLANG_BINARY`):

```bash
VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/node_modules/playwright node vscode_real_test.js
```
