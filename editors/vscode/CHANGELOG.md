# Changelog

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
