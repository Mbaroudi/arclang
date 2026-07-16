# Capella bridge

Converts a real Capella model (`.aird` + fragments) into a compiling ArcLang
`.arc` file, using [capellambse](https://github.com/DSD-DBS/py-capellambse)
(the DSD-DBS Python library that reads/writes native Capella files).

**Identity is preserved**: every emitted element carries its original Capella
UUID as the ArcLang `id:` attribute. This is the foundation for the M3
round-trip goal (Capella → arc → Capella with zero diff).

## Setup

Requires Python ≥ 3.10.

```bash
python3.12 -m venv .venv
.venv/bin/pip install capellambse
```

## Usage

```bash
.venv/bin/python capella2arc.py path/to/model.aird -o model.arc
arclang build model.arc
```

## Verified

Against the Capella 7.0 test model from the capellambse test suite
(`capellambse-study/tests/data/models/test7_0/`):

- 2 actors, 4 entities, 15 activities (OA); 6 functions (SA);
  15 components with nested hierarchy + 16 allocated functions and
  9 component exchanges (LA); 2 nodes (PA); 8 requirements and
  5 requirement↔element traces
- Output compiles with the strict v3 parser: **33 components,
  21 functions, 8 requirements, 5 traces, zero warnings** — every exchange
  endpoint and every trace endpoint resolves.

### Requirements & traces on the forward path

Capella requirement extensions are emitted as a top-level `requirements`
block (`requirement "<uuid>" { title: ... description: ... }` — the
description is the raw HTML `text`, byte-exact). Requirement↔element
relations are emitted as top-level traces, normalized to
`trace "<element-uuid>" satisfies "<requirement-uuid>"`:

| Capella relation type      | Meaning (seen from the requirement) | Emitted |
|----------------------------|-------------------------------------|---------|
| `CapellaOutgoingRelation`  | requirement → model element         | yes     |
| `CapellaIncomingRelation`  | model element → requirement         | yes     |
| `InternalRelation`         | requirement → requirement           | no      |

Dangling trace endpoints are compile **errors** in the strict parser, so a
trace is only emitted when both UUIDs were emitted in the same `.arc`.
Trace lines are sorted (Capella relation order is not stable across edits).

## Round-trip (arc2capella.py + roundtrip_test.sh)

The reverse direction: `arc2capella.py` applies an ArcLang model (AST JSON
from `arclang export -f json`) back onto a Capella model, matching elements
by UUID. It synchronizes:

- **names** of actors, entities, activities, functions, components, nodes
- **descriptions** (wherever the forward path emits them: actors, SA
  functions, logical components, PA nodes)
- **requirements**: `title:` → Capella requirement name, `description:` →
  Capella requirement `text`
- **`--create-missing`**: logical components in the .arc whose id is not a
  known Capella UUID are created — under their .arc parent when it matches
  a Capella element, otherwise in the LA root component package
  (`parent.components.create(name=...)`, which also creates the Part).
  The `arc-id → new Capella UUID` mapping is printed.
- **`--delete-missing`**: logical components in the Capella LA component
  package subtree that are absent from the .arc are deleted (representing
  Parts first, then the component). Explicit flag only, never by default.

```bash
# Full round-trip test — must print "ROUND-TRIP: ZERO DIFF ✓"
tools/capella_bridge/roundtrip_test.sh \
  "tests/fixtures/capella/test7_0" "Model Test 7.0.aird"

# Editing workflows — must print "WORKFLOW TEST: ALL 4 SCENARIOS PASSED ✓"
tools/capella_bridge/workflow_test.sh \
  "tests/fixtures/capella/test7_0" "Model Test 7.0.aird"
```

Verified on the Capella 7.0 test model:
- **Zero diff**: Capella → .arc → compile → apply back yields a
  byte-identical Capella model (68 elements matched, 0 unknown).
- **Workflow test** (4 scenarios on a temp copy): description edit
  propagates to the Capella XML; requirement text edit propagates; adding a
  component + `--create-missing` creates it in Capella (and the re-export
  contains it); removing it + `--delete-missing` deletes it (and the
  re-export is byte-identical to the edited .arc).

## Current scope / not yet covered

Reverse path (`arc2capella.py`):

- Creation/deletion covers **logical components only** — not functions,
  actors, entities, activities, nodes, exchanges or requirements.
- Deletion does **not cascade** references: exchanges, allocations or
  requirement relations pointing at a deleted component are left dangling.
  Safe for components without cross-references (e.g. freshly created ones).
- A description present in Capella but absent from the .arc is left
  untouched (an empty description cannot be distinguished from "not
  emitted"), so descriptions cannot be *cleared* through the bridge.
- Requirement relations (traces) are read-only: editing/adding/removing
  `trace` lines in the .arc does not change Capella relations.
- Attributes other than name/description/requirement text are not synced.

Forward path (`capella2arc.py`):

- Functional exchanges (SA), physical links/exchanges, EPBS, modes/states,
  scenarios, interfaces and ports are not emitted.
- Requirement metadata beyond name/text (identifier, long_name, type
  attributes) is not emitted.
