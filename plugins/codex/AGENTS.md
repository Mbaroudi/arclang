# ArcLang

This project models systems in ArcLang (`.arc`) and judges them: the compiler checks the model,
simulation, proof and engineering engines give verdicts. The `arclang` MCP server exposes the
compiler (`arclang_*` tools).

- Say a requirement or a limit holds only when a tool's first line says so. A stale, missing,
  not-run or inconclusive verdict is not a pass.
- Engineering verdicts (`arclang_engineering`) are simulations of a model, not measurements of a
  product. Read `credibility` before saying what a verdict supports.
- `arclang_engineering` needs the Enterprise edition (`arclang-server`); without it the answer says
  `UNAVAILABLE` and no verdict is known.
- Never edit a model to make a failing verdict pass; propose the change and run it again.
- Never run `cargo fmt` in the arclang repository.

The skills of this plugin (`engineering-verdicts`, `engineering-credibility`,
`engineering-electronics`, `engineering-mechanics`, `engineering-chip`, `engineering-photonics`)
say how to use the tools for each domain.
