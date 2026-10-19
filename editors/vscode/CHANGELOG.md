# Changelog

## 4.4.0

- **Change what is already drawn**, draw.io style but on the model, every change checked by the compiler and one undo:
  - **Reconnect** an exchange by dragging an end of the selected line to another box or port (valid targets highlighted, port directions kept, a port made where it lands on a box), or with *Reconnect the source/target…*: component and functional exchanges, interactions, physical links, transitions, scenario messages; a trace is retargeted from the Relations view, its rationale kept.
  - **Reverse** an exchange (floating toolbar, menu, properties); its ports turn around with it when they carry nothing else.
  - **Rename in place** by double-click, on boxes and on the label of an exchange or message; F2 as before. A double-click no longer jumps to the source (*Go to source* is in the menu).
  - **Move to…** for any element, ports included: a port takes the ends of its exchanges with it.
  - **Copy, cut, paste, duplicate** (⌘C ⌘X ⌘V ⌘D): paste goes into the selected box, cut then paste is a move, ids made unique (`LC-X` becomes `LC-X-2`).
  - **Align** (left, centre, right, top, middle, bottom) and **distribute** a selection; **smart guides** while moving or resizing (Alt moves freely); **arrow keys** move the selection by 1, Shift by 10; **band selection** by dragging on the empty sheet (Space+drag pans); **bend** an exchange by its middle, a double-click on a bend removes it. All kept in `<model>.layout.json`.
  - **Quick connect**: an arrow on each side of the box under the pointer; dragged it connects, clicked it creates a connected element of the same kind.
  - **A context menu** on boxes, ports, exchanges, selections and the sheet, in place of VS Code's; **a floating toolbar** by the selection; **`?`** lists the shortcuts.
  - The properties of an exchange show its ends and its attributes (protocol, bandwidth, data type…), editable.
- Needs a compiler built from this repository after 4.3.0 (with an older one these gestures are refused with "unknown variant"): `arclang edit` learns `reconnect` (refused when a port goes the wrong way, an exchange would join an element to itself, or the compiler warns about the result) and `duplicate`; a rename falls back to a relationship of that name; a move of a port rewrites its exchanges; the layout file keeps bent exchanges (`edges` with `points`).
- `test/e2e/editing_test.js`: 33 checks of these gestures against the real compiler; `vscode_real_test.js` reconnects, opens the menu and aligns in VS Code itself (19 checks).

## 4.3.1

- **Free editing of the diagrams**, tested in VS Code itself: drag a box anywhere, resize it by the grip on its corner, let it go on another box to move it inside. The arrangement is written to `<model>.layout.json` beside the model (the file the CLI and the site read) and comes back when the model opens.
- Full screen keeps the palette, the properties and the toolbar: the workbench takes the whole screen (Zen mode, not centered); Esc gives the window back.
- The toolbar wraps instead of being cut: Save SVG, Save PNG and Full screen are always reachable in a narrow panel.
- Fixes: dropping a box on another never reached the page (no move happened); the find box showed before it was asked for; a moved block kept the indentation of its old place; a layout file the page cannot use no longer leaves the workbench waiting.
- Full screen and SVG/PNG/layout exports work inside VS Code (native save dialog; full screen is a focus mode that maximizes the editor).
- Diagram contents: hide from a diagram (⌘H), show related elements, new diagram around an element; Trace tool; rename in place (F2); find in the diagram (⌘F); exchanges selectable and deletable; cascades confirmed; a created element is selected and centered.
- A visible layer bar with diagram counts; drag-to-connect from a handle; the agent tested end to end.
- Requirement tool (a new requirement satisfied by the selection) and *Satisfies…* in the properties; Fragment tool around a scenario message; multi-selection with Shift+click to delete, hide or set one attribute on all; a minimap (Shift+M).
- Fixes: the selection survives a new snapshot; the system analysis palette writes `system_component`; a scenario message is selected as itself, not as an exchange carrying the same text (Delete removed that exchange).
- `test/e2e`: the page against the real compiler (37 palette tools, 21 gestures, every toolbar tool). `COMPARISON.md`: Capella and SysML v2 tools compared.

## 4.3.0

- **The workbench** (ArcLang: Open the workbench): the compiler's diagram viewer beside the source with a palette (create actors, functions, components, nodes, ports; connect them by interactions, functional and component exchanges, physical links), a properties view typed by the metamodel, a semantic browser, the findings and the gate. Every gesture is an `arclang edit` request: refused unless the edited model compiles, applied as one undoable change in the editor. Text and views edit the same file; the selection follows from one to the other.
- **The model tree** in the ArcLang side bar: the elements of the active model by layer and ownership, with Go to source, Rename, Delete and Ask @arclang.
- **The agent**: `@arclang /agent` changes the open model step by step through the compiler; tools `arclang_model` (elements, ids, relationships, findings) and `arclang_edit` (a list of edits applied as one undoable change, refused as a whole if the model would not compile) for agent mode, the chat participant and other assistants.
- **SysML v2 files** open in the same workbench and tree: imported by the compiler for the views, written back through the SysML v2 export after an edit (ids kept, layout of the original not kept). Syntax colouring for `.sysml`.
- Palette for modes and states (states, modes, transitions with a guard) and for scenarios (messages between lifelines); the cursor follows into the views for `.sysml` files too.
- Requires the compiler 4.3 (`arclang workbench`, `arclang edit`).

## 4.2.0

- Rewritten in TypeScript, bundled with esbuild, with unit and integration tests.
- `@arclang` chat participant (`/model`, `/fix`, `/explain`, `/review`, `/transition`): every model it writes is compiled by `arclang` before it is shown, corrected on refusal, applied from a button.
- Language model tools for agent mode: `arclang_check`, `arclang_gate`, `arclang_metamodel`, `arclang_rules`, `arclang_impact`, `arclang_transition_plan`, `arclang_set_attribute`, `arclang_rename`.
- The ArcLang MCP server is offered to agent mode (`arclang.mcp.command`).
- Commands: check, format (`arclang format` as the document formatter), apply a transition step, change impact, semantic diff against git HEAD, export, import (SysML v2, Capella, ReqIF, ...), metamodel and rules.
- The language server starts with the first `.arc` file, not at startup; a missing compiler is explained once, with ways to install it.

## 4.1.0

- Language server: diagnostics with rules, outline, completion, hover, definition, rename.
- Diagrams beside the source, production gate, transition plan.
