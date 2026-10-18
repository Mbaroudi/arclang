# Changelog

## 4.3.0

- **The workbench** (ArcLang: Open the workbench): the compiler's diagram viewer beside the source with a palette (create actors, functions, components, nodes, ports; connect them by interactions, functional and component exchanges, physical links), a properties view typed by the metamodel, a semantic browser, the findings and the gate. Every gesture is an `arclang edit` request: refused unless the edited model compiles, applied as one undoable change in the editor. Text and views edit the same file; the selection follows from one to the other.
- **The model tree** in the ArcLang side bar: the elements of the active model by layer and ownership, with Go to source, Rename, Delete and Ask @arclang.
- **The agent**: `@arclang /agent` changes the open model step by step through the compiler; tools `arclang_model` (elements, ids, relationships, findings) and `arclang_edit` (a list of edits applied as one undoable change, refused as a whole if the model would not compile) for agent mode, the chat participant and other assistants.
- **SysML v2 files** open in the same workbench and tree: imported by the compiler for the views, written back through the SysML v2 export after an edit (ids kept, layout of the original not kept). Syntax colouring for `.sysml`.
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
