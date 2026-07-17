# ArcLang — Arcadia-as-Code

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

**A textual modeling language for the [Arcadia](https://mbse-capella.org/arcadia.html) method, with a strict compiler written in Rust.**

Write systems-engineering models (Operational Analysis → System Analysis → Logical
Architecture → Physical Architecture → EPBS) as plain text. Version them in Git,
review them in pull requests, compile them to JSON and diagrams.

> **Project status: v3 foundation — pre-release.**
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
- **HTML diagram generation** via the ELK-based v2 rendering pipeline
  (`arclang export -f html`, `arclang explorer`).
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
- **SysML v2 interop export**: `arclang export -f sys-ml` emits the OMG
  SysML v2 textual notation (packages, part defs/usages, action defs,
  requirement defs, connect, satisfy — subset documented in the generator).
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

## Explicitly not implemented yet

These commands exist but fail honestly with `Not implemented` instead of
pretending to work: `repl`, `clean`, `format`, `new`, `sync` (PLM),
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
  version: "3.0.0"

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

## Design principles (v3)

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

## Repository layout

```
spec/GRAMMAR.ebnf     Canonical syntax specification
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
