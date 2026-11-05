# ArcLang documentation

The documentation is published at **<https://arclang.fr/docs/>**. Every sample
on those pages is compiled, and every command shown is run, by the released
compiler when the site is built.

| Page | Content |
|---|---|
| [Getting started](https://arclang.fr/docs/getting-started/) | Install, first model, first verdict of the production gate |
| [Language](https://arclang.fr/docs/language/) | Layers, elements, traces, test cases, safety analysis |
| [Types and constraints](https://arclang.fr/docs/types-and-constraints/) | Typed quantities, type definitions, constraints |
| [Diagrams](https://arclang.fr/docs/diagrams/) | Viewpoint diagrams and the viewer |
| [Production gate](https://arclang.fr/docs/production-gate/) | What blocks, per standard |
| [Review and change](https://arclang.fr/docs/review-and-change/) | Semantic diff, impact, formatter, scripted edits |
| [Capella](https://arclang.fr/docs/capella/) and [SysML v2](https://arclang.fr/docs/sysml-v2/) | Exchange and comparison |
| [Command line](https://arclang.fr/docs/cli/) | Every command |

## In this directory

- [modelica.md](modelica.md): the Modelica export and requirements verified
  by simulation in OpenModelica (`arclang export -f modelica`, `arclang verify`).
- [simulink-connector.md](simulink-connector.md): what the Capella-Simulink
  connector exports, from a Capella or an ArcLang model
  (`arclang export -f simulink-connector`).
- [embedded-c.md](embedded-c.md): the state machines as portable C99, the
  I/O interface and its bindings, software in the loop against the model
  (`arclang export -f c`, `arclang sil`).
- [realization.md](realization.md): the realization ladder, what each stage
  (simulation, SIL, PIL, HDL, SoC, board, twin) says of each requirement from
  the verdicts the runs cached, in `check`, the tables and the workbench.
- [MIGRATION_4.0.md](MIGRATION_4.0.md): moving a model from 3.x to 4.0.
- [VERSIONING.md](VERSIONING.md): what a major, minor and patch release may change.
- [qualification/](qualification/): tool-qualification support kit.
- [spec/](spec/): design studies. The normative grammar and metamodel are in [`../spec/`](../spec/).
- [history/](history/): documents of versions 1 to 3, kept for the record.
  They describe commands and features that no longer exist; do not rely on them.
