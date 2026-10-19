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
- `toolbar_test.js`: every toolbar tool (exports reach the host as SVG,
  PNG and JSON, full screen, zoom, filters, layers, folding), no console error.

- `vscode_real_test.js`: the workbench inside VS Code itself, driven by
  Playwright through Electron: the toolbar is whole, a box dragged stays
  where it is let go, a box is resized by its grip, both are written to
  `model.layout.json` and back when the model reopens, PNG and SVG exports
  go through the save dialog, full screen keeps every tool and Esc leaves
  it, a box let go on a component moves into it (16 checks).

Run `./run.sh` from this folder after `cargo build --release` at the root.
The VS Code test runs on its own, with the compiler on the PATH (or
`ARCLANG_BINARY`):

```bash
VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/node_modules/playwright node vscode_real_test.js
```
