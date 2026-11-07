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

VS Code 1.14x turns off Electron's `--inspect`, which Playwright's
`_electron.launch` needs. `launch.js` starts VS Code with Chromium's
debugging port instead and attaches to it (`launchVSCode(app, args)`); it
works with older builds too. Every `vscode_*` test and demo goes through
it. The window's size is the page's viewport (`win.setViewportSize`); what
once asked Electron's main process is read elsewhere: the clipboard with
`pbpaste`, the window's zoom from its `devicePixelRatio`, and the demo's
video from the frames the debugging protocol sends (ffmpeg makes the
video). A key sent while a webview has the focus may stay in that page:
where that happens, a test clicks the active tab before the palette.
Keep the user data directory short: VS Code's socket in it must fit in
macOS's 104 characters.

- `vscode_montecarlo_test.js`: Monte Carlo results in VS Code: OpenModelica
  runs every sample, the page shows the probability, the margins, what moves
  them, the failing samples; Python analyses them; a sample replays on the
  diagram (11 checks).
- `vscode_real_test.js`: the workbench inside VS Code itself, driven by
  Playwright: the toolbar is whole, a box dragged stays
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

- `vscode_tidy_test.js`: tidying a diagram up after a move by hand: a box
  let go over another (the line to it follows), the offer saying why, Tidy
  up removing the overlap and writing the layout file, `⌘Z` bringing the
  move back, Keep my layout, Always tidy up setting
  `arclang.workbench.reorganize` and tidying the next move by itself, the
  Layout menu's entries (14 checks).

- `vscode_panes_test.js`: the panes in VS Code itself: quantity, enumeration
  and reference edited in the pane, refusals as they are typed, ⌘Z (AZERTY),
  redefine and reset of an inherited value, the tree's kind icons, diagrams
  under their layer, the filter, "Show in the diagram", a diagram opened in
  a tab of its own ("<model> — <diagram>") beside the workbench, the
  selection shared by both, a tree drop in the form VS Code hands a webview
  (the item's resource URI, Shift held) (21 checks; whether Playwright's
  synthetic drag from the tree reaches the webview is printed as a note).

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
  Excel through the save dialog (the workbook read back with openpyxl, by
  `OPENPYXL_PYTHON` or else `python3` when it has it), Copy as TSV (`pbpaste`), a
  trace created from a matrix and removed after the confirmation.

Run `./run.sh` from this folder after `cargo build --release` at the root.
The VS Code test runs on its own, with the compiler on the PATH (or
`ARCLANG_BINARY`):

```bash
VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/node_modules/playwright node vscode_real_test.js
```
