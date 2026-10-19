# Workbench end-to-end tests

The workbench page (the webview of the VS Code extension) in a real browser,
with a host that edits through `arclang edit` and snapshots through
`arclang workbench`, exactly as the extension does:

- `palette_test.js`: every palette tool of every view (37 tools), each edit
  written in the model text and the model still compiling;
- `gestures_test.js`: trace, hide from a diagram (viewpoint and named),
  show related, new diagram around an element, rename in place (F2), find
  (⌘F), properties (set, rename), delete an element and an exchange,
  connect by dragging from the handle (14 gestures);
- `toolbar_test.js`: every toolbar tool (exports reach the host as SVG,
  PNG and JSON, full screen, zoom, filters, layers, folding), no console error.

Run `./run.sh` from this folder after `cargo build --release` at the root.
