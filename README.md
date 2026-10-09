# ArcLang — Arcadia-as-Code

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

**A textual modeling language for the [Arcadia](https://mbse-capella.org/arcadia.html) method, with a strict compiler written in Rust.**

Write systems-engineering models (Operational Analysis → System Analysis → Logical
Architecture → Physical Architecture → EPBS) as plain text. Version them in Git,
review them in pull requests, compile them to JSON and diagrams.

**Website and in-browser playground: [arclang.fr](https://arclang.fr/)**. The
[playground](https://arclang.fr/playground/) runs this compiler in WebAssembly;
nothing is uploaded.

> **Version 4.0.0.** Upgrading from 3.0: see
> [docs/MIGRATION_4.0.md](docs/MIGRATION_4.0.md).
> This README only claims what the test suite verifies. Anything not listed under
> *Works today* should be assumed absent. See the [Roadmap](#roadmap).

---

## Why

Capella is a great workbench, but its models are binary-ish XML that doesn't diff,
merge, or review well. SysML v2 solved this for its own ecosystem with a textual
notation, formal semantics, and a standard API. **Arcadia has no official textual
notation — ArcLang aims to be that**, with round-trip Capella interoperability as
the long-term goal.

## Works today (verified by CI)

- **Strict compiler with real diagnostics.** Every token carries a source
  position; every error is reported as `message at line L, column C`. Unknown
  constructs are compile **errors** — nothing is ever silently dropped from your
  model. Constructs that parse but aren't yet stored in the model (scenarios,
  dataflows) produce loud warnings.
- **All five Arcadia layers parse to a typed AST**: actors, entities,
  capabilities, activities and interactions (OA); requirements, functions and
  functional exchanges (SA); components, interfaces (`provides`/`requires`,
  `interface_in`/`interface_out`) and component exchanges (LA); nodes, behavior/
  hardware components, links, physical exchanges and deployments (PA); systems,
  subsystems, assemblies and items (EPBS); plus `safety_analysis`
  (hazards, FMEA) and `trace` declarations.
- **A canonical grammar**: [`spec/GRAMMAR.ebnf`](spec/GRAMMAR.ebnf) is the single
  source of truth for the syntax and matches the parser.
- **Golden corpus in CI**: every example under `examples/` (except
  `examples/legacy/`) must compile — `cargo test` fails otherwise.
- **JSON export** of the parsed model (`arclang build`, `arclang export -f json`).
- **HTML architecture document** (`arclang export -f html`,
  `arclang explorer`): requirements, components, traceability and the
  viewpoint diagrams described below, in one self-contained file.
- **Traceability analysis computed from the model** (`arclang trace --validate
  --matrix`): real coverage numbers, real gap warnings.
- **MCP server** (`mcp-server/`) exposing compile/validate/trace/export to LLM
  agents, aligned 1:1 with the actual CLI.
- **Capella round-trip** (`tools/capella_bridge/`): a native Capella 7.0 model
  converts to compiling ArcLang (UUIDs preserved) and back **byte-identically**;
  description/requirement edits, component creation and deletion propagate into
  Capella. Both proven in CI on every push.
- **JSON API**: `POST /api/compile` on `arclang serve` returns the canonical
  semantic model (stable uuids included) or a localized structured error —
  covered by in-process integration tests.
- **Language server**: `arclang lsp --stdio` (tower-lsp) publishes compiler
  diagnostics at exact source positions on open/change/save.
- **Model validation**: duplicate identities, dangling deployment/allocation
  references (warnings on every build); Arcadia methodology advisories via
  `arclang check --lint` (layer consistency, function-less components).
- **A typed metamodel, declared in code**: [`spec/METAMODEL.md`](spec/METAMODEL.md)
  lists every element kind, its typed attributes, the enumerations, the unit
  table and the trace rules, with the Arcadia metaclass and SysML v2 construct
  each kind maps to. It is generated from `src/compiler/metamodel.rs`
  (`arclang metamodel`, `GET /api/metamodel`) and a golden test fails when the
  document and the compiler disagree.
- **Typed quantities**: `latency: 25 ms`, `bandwidth: 100 Mbps`. A number
  followed by a unit is a quantity with a dimension; an unknown unit is a
  compile error. Legacy strings (`"25 ms"`) keep working. Type violations
  (`latency: 135 MHz`, `asil: "High"`, an exchange naming a port its
  component does not declare) are `metamodel:` warnings at compile time and
  **blockers in the production gate**. The gate sums latencies across units.
- **SysML v2 export, validated by the OMG pilot**: `arclang export -f sys-ml`
  emits requirements, action defs with typed in/out items, functional chains
  as successions, part defs with nested parts and ports, interface defs,
  connections, flows, allocations, state defs, verification defs, `satisfy`
  and typed attributes using the ISQ/SI libraries. The export of **every
  active example** is accepted without error by the OMG SysML v2 pilot
  implementation (`tools/validate_sysml_corpus.sh`, run locally — not yet in
  CI). Scenarios are exported as occurrence definitions with ordered events
  and messages, constraints as asserted expressions. Known limit: hazards
  and FMEA have no SysML v2 construct and are exported as tagged requirements.
- **User-defined types and specialization**: `type "Safety ECU" extends "ECU"
  { safety_level: "ASIL-D" }`, then `is: "Safety ECU"` (or a list of types)
  on any element. The
  element inherits the type's attributes (its own values win) and, for
  components, its ports. A type lists the attributes its instances must
  provide (`required: [...]`). Compile errors: unknown or duplicate type,
  cyclic `extends`, a missing required attribute, a redefinition that changes
  a quantity's dimension, two types defining the same attribute differently
  unless the element redefines it. Inheritance is resolved before analysis, so the
  gate, the constraints and every export see effective values. Exported to
  SysML v2 as abstract definitions with `:>` and `:>>`.
- **Systems Modeling API**: `arclang serve model.arc ...` serves
  each model as a project under `/api/systems-modeling`, following the OMG
  Systems Modeling API resource model: projects, the `main` branch, commits,
  elements, relationships (`?direction=in|out|both`), roots, cursor pagination
  (`page[size]`, `page[after]`, `page[before]`, `Link` header) and ad-hoc
  queries (`POST /projects/{id}/query-results` with primitive and composite
  constraints). Every element and relationship has a deterministic UUID, an
  owner, a qualified name and typed attributes (quantities carry their unit
  and their SI value). The commit id is content-addressed: reformatting a
  model does not create a new snapshot. With `--history`, every git commit
  that touched the model file (imports resolved at that revision) is a commit
  of the API, the uncommitted working tree is the head when it differs, and
  `GET .../commits/{id}/changes` lists the elements added, modified or
  deleted since the previous commit, by identity. A revision that no longer
  compiles is served as an empty commit carrying the compiler's error.
  Relationships whose end does not resolve are listed in the commit, never
  dropped. There is a single branch.
  - **Two vocabularies, same commits.** Under `/api/systems-modeling`,
    `@type` is a kind of the ArcLang metamodel. Under `/api/sysml-v2`,
    `@type` is a KerML / SysML v2 metaclass (`PartDefinition`, `PartUsage`,
    `ActionDefinition`, `RequirementUsage`, `ConnectionUsage`,
    `SatisfyRequirementUsage`...) with the metamodel's property names:
    owning memberships (`OwningMembership`, `FeatureMembership`), typing,
    specialization and redefinition as relationships, connector `source` /
    `target`, documentation, qualified names, and expressions as KerML
    elements (`FeatureValue`, `ResultExpressionMembership`,
    `OperatorExpression`, `FeatureReferenceExpression`,
    `FeatureChainExpression`, `InvocationExpression`, typed literals): a
    latency of `10 ms` is the operator `[` applied to the integer 10 and a
    reference to the unit. This view is not a second mapping: it is the
    abstract syntax of the SysML v2 export, read back from the text the
    pilot validates, so the API and the export cannot disagree.
    Standard-library elements the model refers to (`ScalarValues::String`,
    `ISQBase::DurationValue`, `SI::kilogram`, `DataFunctions::max`) are
    served as stubs flagged `isLibraryElement`; what each written name
    designates comes from the pilot (`spec/sysml_library_index.json`,
    generated by `tools/sysml_library_index.py`), and a test fails when the
    exporter can write a library name the index lacks.
    `tools/check_sysml_abstract_syntax_corpus.sh` compares the view,
    element by element, with the abstract syntax the OMG pilot
    implementation derives from the same text: metaclass, names, owning
    membership, direction, abstractness, qualified typing / specialization /
    redefinition targets, connector ends and whole expression trees agree
    for every example (20 models, 3,837 declared elements at the time of
    writing) and for a model written to cover every expression form; four
    of those listings are frozen as fixtures and checked by `cargo test`
    without the pilot. A SysML element carries `arclang:element`, the
    ArcLang element it comes from.
    **What the SysML view does not hold**: the content of
    library elements (a stub has no members), and what KerML derives
    implicitly (connector end features, implied specializations, inherited
    members, the parameter features that carry an expression's arguments:
    an argument is owned by its expression and listed in `argument`). A
    connector end written as a feature chain (`p_A.out`) is given by
    `arclang:sourcePath` / `arclang:sourceFeature`, not by the anonymous
    chain feature KerML uses. It is read-only.
  - **Authentication and rights.** With `--token-file <file>` or the
    `ARCLANG_API_TOKEN` environment variable, every request under `/api`
    needs `Authorization: Bearer <token>`: one shared token with every
    right the server grants. With `--users-file <file>`, each line declares
    a user, `name role token`, the role being `read` or `write`; the token
    is given in clear or as `sha256:<hex digest>`, so the file need not
    hold a secret. A reader who writes gets 403. Tokens are 24 characters
    at least; only their SHA-256 is kept in memory, compared in full and
    against every user. The server listens on the loopback interface only.
  - **Writes, opt-in.** `arclang serve --allow-write --history model.arc`
    (a token is mandatory) accepts `POST /projects/{id}/commits` with
    `{"description", "previousCommit", "change": [...]}`. A change can:
    rename an element (a different `name` in the payload; accepted only
    when the element keeps its identity, that is when it writes an `id`
    and nothing refers to it by its name);
    set or remove attributes of an element (`{"identity": {"@id"},
    "payload": {"attributes": {"latency": {"value": 10, "unit": "ms"},
    "owner": null}}}`); create an element (`{"payload": {"@type":
    "LogicalComponent", "name": "Logger", "owner": {"@id"}, "attributes":
    {...}}}`, for `Actor`, `Requirement`, `SystemFunction`,
    `LogicalComponent`, `LogicalFunction`, `PhysicalNode`); delete an
    element with what it contains (`"payload": null`); create or delete a
    trace (`{"@type": "Trace", "traceKind", "source", "target"}`). The
    request is applied to the model file as lossless edits (comments and
    layout kept; a deleted block goes with the comments inside it), is all
    or nothing, and becomes **one git commit** of that file with the
    request's description, under the repository's own identity; a named
    user is served as `arclang:user` on the commit; the commit message is
    left exactly as requested, and the name is kept in a git note
    (`refs/notes/arclang-user`) so that it is still known after a restart.
    Nothing is
    written unless the edited model compiles and holds exactly what was
    asked: every value shows, only the named elements and traces appear or
    disappear, and no relationship is left without an end (deleting an
    element a trace still points to is refused: delete the trace in the
    same request). A file with uncommitted changes, or a request based on
    a commit that is no longer the head, is refused (409). Not supported:
    other kinds of elements and relationships, elements declared in
    imported files, creating a layer block the file does not have,
    projects and branches. Rights are per user but not per project, and
    there is no rate limiting: this is a local or trusted-network service,
    not an internet-facing one.
- **Multiplicity**: `multiplicity: 4` on a logical component or a physical
  node says how many of it its owner has; a range is written as text
  (`"0..1"`, `"1..*"`, `"*"`). It is typed by the metamodel (an ill-formed
  or empty range is a `metamodel:` warning and a gate blocker), served with
  its bounds by the API (`"multiplicity": {"lower": 1, "upper": null}`),
  and exported on the SysML v2 usage (`part p_LC_001 : Controller [1..*];`),
  where the SysML view serves it as a `MultiplicityRange` with its bounds,
  checked against the pilot like the rest. It is a statement about the
  architecture: constraints and budgets do not multiply by it.
- **Constraints**: `constraint "Budget" { assert: sum("FC-1", latency) <=
  "FC-1".latency_budget * 0.8 }`. Arithmetic over typed attributes with
  `sum`/`min`/`max`/`count` over chains, **dimension-checked**: comparing a
  time with a data rate, or naming an unknown element or attribute, is a
  compile error. Products and quotients carry derived dimensions
  (`voltage * current <= power`, `frame / bandwidth < 1 ms`). A violated constraint is a warning at compile time and a
  blocker in the production gate; `arclang check` prints each verdict with
  both evaluated sides. See `examples/automotive/timing_constraints.arc`.
- **Industrial governance**: MIT-licensed; dependency audit (RustSec) on
  every CI run; releases ship binaries + `SHA256SUMS` + GitHub build
  provenance attestations; language stability policy in
  `docs/VERSIONING.md`; tool-qualification support kit (ISO 26262-8 §11
  TCL analysis, DO-330 TQL, function/malfunction inventory, evidence
  matrix into the test suite) in `docs/qualification/`.
- **Semantic diff**: `arclang diff old.arc new.arc [--json]` compares two
  model versions by stable identity — reformatting or moving blocks is an
  empty diff, renaming an element (same id) is a modification with
  field-level old/new values, traceability changes are listed explicitly.
  Exit code 1 when models differ (diff(1) convention, CI-gateable).
- **Multi-file models**: `import "fragment.arc"` assembles a model from
  team-owned files (paths relative to the importing file, recursive, cycles
  and missing files are hard errors). Traces in one file resolve against
  elements of another — see `examples/multifile/`.
- **ReqIF exchange**: `arclang export -f req-if` emits OMG ReqIF 1.0 (the
  DOORS/Polarion/Jama exchange format) with deterministic identifiers and
  requirement-to-requirement relations; `arclang import -f req-if` reads
  foreign ReqIF (DOORS-style attribute names, XHTML text) into an ArcLang
  requirements block, preserving the foreign identity as `reqif_id`.
- **Simulation bridges**: `arclang export -f simulink` emits a MATLAB script
  that rebuilds the architecture in System Composer (components, oriented
  ports, connections) plus Stateflow skeletons for state machines;
  `arclang export -f fmi` emits one FMI 2.0 `modelDescription.xml` per
  component (causality from exchange direction, GUID = the component's
  deterministic ArcLang UUID). Interface contracts only — behaviour stays in
  the simulation tool.
- **Diagrams**, compiled from the model and drawn in Arcadia notation:
  - one per architecture viewpoint (OAB, SAB, LAB, PAB): functions nested
    in the component they are allocated to, exchanges bound to the ports
    they connect, functional chains resolved to node and edge ids;
  - missions and capabilities with what they involve and realize, nested
    capabilities and their extend / include / generalization relations, the
    data model (classes with fields in declared order, enumerations, data
    types with their unit, exchange items) and the product breakdown tree;
  - one per mode/state machine and one sequence diagram per scenario.

  Unresolved ports, double allocations and identity collisions are
  reported, never drawn as guesses.
  `arclang diagram model.arc -f html -o diagrams.html` writes a
  self-contained viewer (no network access: the ELK layout engine is
  embedded) with a tab per kind of view, a model tree showing the nesting
  of each view, links from an element to the views that also draw it and
  to the elements it is traced to (`trace A realizes B`), a way back,
  functional-chain highlighting, element details, SVG/PNG export and one
  print page per diagram;
  `arclang explorer` embeds the same viewer. `-f viewpoints` and `-f elk`
  emit the diagram model and the layout graphs as JSON; `--view lab`
  (or `sab`, `cap`, `cdb`, `pbs`, `msm`, `es`, ...) keeps one kind. The
  model is golden-tested; CI lays out and draws every example headless
  (`tools/diagram_render/check.sh`). Layout is automatic: there is no
  manual placement on top of it. A container can be folded into one box
  (double-click, Enter, or "Fold all"): its exchanges then end on the box.
  It can also be opened as a diagram of its own, with what it exchanges
  with drawn around it. With "Arrange" on, a box is dragged where the
  reader wants it and its exchanges are redrawn at right angles, without
  steering around other boxes. Folds, the open container and the
  arrangement are remembered by the browser while the reader works.
  "Save layout" downloads them as `<model>.layout.json`; kept next to the
  model, that file is read by `arclang diagram` and `arclang explorer`
  (or named with `--layout`), so an arranged diagram is shared and
  versioned with the model. The file only says how to show the model:
  what it names that the model no longer draws is reported and left out,
  and a file that is not a layout stops the command. Scenarios draw lifelines and ordered
  messages; combined fragments (alt/loop/par) are not in the language yet.

- **Formatter**: `arclang fmt model.arc` prints the model in canonical
  layout, `--write` rewrites in place (a directory is walked recursively),
  `--check` exits 1 when a file is not formatted. It rewrites whitespace
  only (indentation, spacing inside a line, blank lines): comments, the
  order of declarations, literal spelling and the author's line breaks are
  kept, because the formatter works on the token stream and never rebuilds
  text from the model. Every result is re-lexed and must carry exactly the
  same tokens and comments as the input, otherwise nothing is written. It
  does not wrap long lines or move a token to another line.

- **Scripted edits**: `arclang set model.arc LC-001 latency "10 ms"`
  changes one attribute of one element, `arclang unset` removes one,
  `arclang rename model.arc LC-001 "Brake controller"` renames an element
  that keeps its identity;
  `--write` rewrites the file, otherwise the result goes to stdout. The edit
  is a splice of the source text between two tokens: every comment, the
  layout and all other declarations stay byte for byte. An element is
  designated by its `id`, or by its name when it writes none. Nothing is
  written unless (1) the two syntax trees differ by that attribute of that
  element and nothing else, (2) the edited model compiles, and (3) the
  compiled element shows the requested value; a value that would be
  shadowed (a requirement title given on the declaration line) is refused,
  not silently ignored. Over the example corpus more than 90% of text attributes
  are editable this way (tested); the rest are refused with the reason. Limits: one
  attribute at a time, no creation, deletion or renaming of elements, no
  edit of relationships, and an element declared in an imported file is
  edited in that file.

- **Advisory review of traces (optional, external service)**:
  `arclang review model.arc` asks a judgment model (TypeSafe's Jev) one
  yes/no question per declared trace, "is this trace plausible, from what
  the two elements say about themselves?", and lists the traces under a
  plausibility threshold (`--threshold`, 0.3 by default); `--suggest` also
  judges undeclared component- or function-to-requirement pairs and lists
  those that may be a missing trace. This is the one thing the compiler
  cannot do: it checks that a trace links two elements that exist, not that
  the link makes sense. **It is advice, and it is kept apart**: the output
  is probabilities, never a verdict; nothing of it reaches the compiler,
  `arclang check` or the gate, which stay deterministic and offline; the
  answers of an external model are not reproducible. **It sends model
  content out**: the kind, name, id and text attributes of the elements
  concerned, and the rationale of a trace that states one, go to
  `api.typesafe.ai`, only when this command is run, with
  the key of `TYPESAFE_API_KEY` (environment or `./.env`); `--dry-run`
  prints what would be sent and sends nothing. What is tested is the
  plumbing (requests, retries, errors that never show the key, thresholds),
  against a local stand-in; the quality of the judgments is not something a
  test can guarantee. Measured on the examples (jev-1.13.0, 2026-10-09),
  declared traces against the same traces with their targets swapped:
  judged on the two elements alone, a declared trace is rated above a
  swapped one in about 80% of the pairs (197 declared against 148 swapped,
  measured independently over 16 models; 88% on a first sample of 23
  against 22); judged with the trace's stated rationale, in 91% of the
  pairs, against 78% without it on the same sample (61 declared against 47
  swapped). At the default threshold of 0.3, on the two elements alone, 9 of
  the 197 declared traces are listed, and 44% of the swapped ones. A swapped trace is not always a
  wrong one, and the examples are written in English by the authors of the
  tool: these figures direct a review, they do not replace it.

## Explicitly not implemented yet

These commands exist but fail honestly with `Not implemented` instead of
pretending to work: `repl`, `clean`, `new`, `sync` (PLM),
`plugin`, `lsp` TCP mode, safety FTA/report generation, dependency analysis.
The built-in Rust `import` command reads a simplified XML — real Capella
round-trip goes through `tools/capella_bridge/` (capellambse).

## Quick start

```bash
# Build
cargo build --release

# Compile a model (JSON output + real element counts)
./target/release/arclang build examples/complete_emergency_braking_simple.arc

# Check with traceability warnings
./target/release/arclang check examples/automotive/adaptive_cruise_control.arc --lint

# Traceability matrix
./target/release/arclang trace examples/automotive/acc_from_capella.arc --validate --matrix

# Model metrics
./target/release/arclang info examples/aerospace/flight_control_system.arc --metrics
```

## Language at a glance

```arc
model EmergencyBrakingSystem {
  version: "4.0.0"

  operational_analysis "Emergency Braking - Operational View" {
    actor Driver { description: "Vehicle operator" }

    entity Vehicle {
      activity MonitorEnvironment { description: "Observe surroundings" }
    }

    interaction DriverCommands {
      from: Driver
      to: Vehicle.MonitorEnvironment
    }
  }

  requirements safety {
    req "REQ-BRK-001" "Emergency braking activation" {
      description: "The system shall apply emergency braking when collision risk is critical"
      safety_level: "ASIL-D"
    }
  }

  system_analysis SA_Braking {
    function AssessThreat {
      inputs: ["tracked_objects"]
      outputs: ["threat_level"]
      safety_level: "ASIL-D"
    }
  }

  architecture logical {
    component "BrakeController" {
      id: "LC-001"
      provides interface IBrakeCommand { protocol: "CAN" }
      function "Compute braking force"
    }
  }
}

trace "LC-001" satisfies "REQ-BRK-001" { rationale: "Direct implementation" }
```

The full syntax is specified in [`spec/GRAMMAR.ebnf`](spec/GRAMMAR.ebnf).
Names may be identifiers (`Driver`, dotted `Vehicle.MonitorEnvironment`) or
strings (`"Brake Controller"`); IDs containing hyphens must be quoted.
This exact example compiles: 1 requirement, 3 components, 2 functions, 1 resolved trace.

## Design principles

1. **The compiler never lies.** No fake outputs, no hardcoded metrics, no
   "success" on an empty model. Unimplemented features fail explicitly.
2. **One grammar.** `spec/GRAMMAR.ebnf` is normative; parser divergence is a bug.
3. **Errors are localized.** Line and column, always.
4. **CI is the only source of claims.** If a feature isn't exercised by
   `cargo test`, this README doesn't advertise it.

## Roadmap

| Milestone | Content | Status |
|---|---|---|
| **M1 — Honest core** | Strict parser, spans, golden corpus, de-faked CLI | ✅ |
| **M2 — Stable identity** | Deterministic UUIDs on every element, dangling references as compile errors, single semantic model | ✅ |
| **M3 — Capella round-trip** | Native Capella import/export via [capellambse](https://github.com/DSD-DBS/py-capellambse) bridge, zero-diff round-trip + editing workflows in CI | ✅ (names/descriptions/requirements; see `tools/capella_bridge/README.md` for scope) |
| **M4 — Programmatic access** | JSON API over the semantic model (axum), LSP (tower-lsp) with diagnostics from spans | ✅ diagnostics & API (next: go-to-definition, completion, MCP as API client) |
| **M5 — Arcadia semantics** | Allocation rules (function→component), inter-layer consistency checks, SysML v2 interop export | ✅ (reference validation, methodology lints, SysML v2 subset export) |
| **M6 — Typed metamodel** | Metamodel independent of Capella, typed quantities and enumerations, SysML v2 export validated against the OMG pilot | ✅ (language 4.0, see `docs/MIGRATION_4.0.md`; typed quantities and derived dimensions, multiplicities, constraints, user-defined types with specialization and multiple typing, Systems Modeling API with git history, a comment-preserving formatter (`arclang fmt`); lossless attribute edits (`arclang set`/`unset`), API writes as git commits (attributes, renaming, creation and deletion of elements and traces) with named users and read / write roles, SysML v2 metaclass vocabulary in the API with expressions and library elements, checked against the pilot; next: library content in the SysML view, the remaining kinds through the API, rights per project) |

## Repository layout

```
spec/GRAMMAR.ebnf     Canonical syntax specification
spec/METAMODEL.md     Typed metamodel (generated from the compiler)
src/compiler/         Lexer, parser, AST, semantic analysis, codegen, renderers
src/cli/              Command-line interface
mcp-server/           MCP server (Python) for LLM agents
examples/             Compiling examples (CI-enforced) — legacy/ excluded
tests/                Test suite incl. golden corpus (examples_compile.rs)
docs/history/         Archived status reports from v1/v2 development
docs/spec/            Design documents (v2 unified syntax study, SysML v2 mapping)
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Rule #1: a feature exists when a test
proves it — PRs that add capabilities must add tests, and `cargo test` must
stay green.

## License

MIT — see [LICENSE](LICENSE).
