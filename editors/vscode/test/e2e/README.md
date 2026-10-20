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
  layout (28 checks; keys sent as an AZERTY Mac sends them).

Run `./run.sh` from this folder after `cargo build --release` at the root.
The VS Code test runs on its own, with the compiler on the PATH (or
`ARCLANG_BINARY`):

```bash
VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/node_modules/playwright node vscode_real_test.js
```
