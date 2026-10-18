# Changelog

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
