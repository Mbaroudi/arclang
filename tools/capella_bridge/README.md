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
(`capellambse-study/tests/data/models/test7_0/`, mirrored at
`tests/fixtures/capella/test7_0/`):

- OA: 2 actors, 4 entities, 15 activities, 1 communication means,
  2 operational processes
- SA: 6 functions, 2 missions, 2 capabilities, 1 functional chain
- LA: 15 components with nested hierarchy + 16 allocated functions,
  9 component exchanges, 1 functional chain, 2 capability realizations
- PA: 33 nodes (every physical component, flat), 10 physical links,
  4 physical paths
- Data model: 19 classes (23 fields), 7 enumerations, 17 data types,
  17 exchange items
- Transverse: 8 state machines (2 modes, 26 states, 27 transitions),
  3 scenarios (14 messages; 4 scenarios skipped — no emitted participant)
- 8 requirements and 5 requirement↔element traces
- Output compiles with the strict v3 parser, **zero warnings** — every
  exchange endpoint, trace endpoint, involves/realizes/mission reference,
  scenario participant/message endpoint and transition endpoint resolves.

### Concept mapping (forward path)

| Capella (capellambse 0.8.1)                         | ArcLang                              |
|-----------------------------------------------------|--------------------------------------|
| `sa.all_missions`                                   | SA `mission` (id:)                   |
| `sa.all_capabilities` (+ `mission.exploits`)        | SA `capability` (mission:, realizes:, involves:) |
| `sa/la.all_functional_chains` (`.involved`)         | `functional_chain` (ordered involves:) |
| `la.all_capabilities` (CapabilityRealization)       | LA `capability_realization` (realizes:) |
| `oa.all_processes` (`.involved`)                    | OA `operational_process` (involves:) |
| `oa.all_entity_exchanges` (CommunicationMean)       | OA `communication_means` (from:/to:) |
| `pa.all_components`                                 | PA `node` (flat — no nesting in PA grammar) |
| `pa.all_physical_links` (port-owner endpoints)      | PA `link` (id:, from:/to: owner UUIDs) |
| `pa.all_physical_paths` (`.involved_links`)         | PA `physical_path` (involves: link NAMES) |
| `search("Class")` (`.owned_properties`)             | `class` (fields: name -> type name)  |
| `search("Enumeration")` (`.literals`)               | `enumeration` (values:)              |
| layer `data_package` walk (NumericType, StringType…)| `data_type`                          |
| `search("ExchangeItem")` (`.type`, `.elements`)     | `exchange_item` (mechanism:, elements:) |
| `search("StateMachine")` (regions/states/modes/transitions) | `state_machine` (mode/state/transition) |
| `search("Scenario")` (instance_roles -> Part.type, message ends) | `scenario` (participants:, message) |

Strictness rules honoured by the emitter: `involves:`/`realizes:`/`mission:`
lists reference **only UUIDs emitted in the same .arc** (dangling references
are compile errors); chains/processes/paths whose involvements are all
unemitted are **skipped** (involves is required non-empty); transition
endpoints are state NAMES declared in the same machine (unnamed pseudostates
and their transitions are skipped); transition triggers are emitted only when
they name an emitted element (unresolved triggers are compiler warnings);
scenario messages only connect emitted participants; a physical link is
emitted only when both port owners are emitted, and paths reference links by
name; exchange item `mechanism:` is emitted only for the five Arcadia values
(`UNSET` → omitted, ArcLang defaults to DATA).

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

- **names** of actors, entities, activities, functions, components, nodes —
  and of missions, capabilities, capability realizations, functional chains,
  operational processes, communication means, physical links, physical
  paths, classes, data types/enumerations and exchange items (sync-only:
  no create/delete for these kinds)
- **descriptions** (wherever the forward path emits them: actors, SA
  functions, logical components, PA nodes, and the new kinds above except
  data types/enumerations/exchange items, whose AST carries no attributes)
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
  byte-identical Capella model (184 elements matched, 0 unknown).
- **Workflow test** (4 scenarios on a temp copy): description edit
  propagates to the Capella XML; requirement text edit propagates; adding a
  component + `--create-missing` creates it in Capella (and the re-export
  contains it); removing it + `--delete-missing` deletes it (and the
  re-export is byte-identical to the edited .arc).

## Current scope / not yet covered

Reverse path (`arc2capella.py`):

- Creation/deletion covers **logical components only** — not functions,
  actors, entities, activities, nodes, exchanges, requirements, or any of
  the newer kinds (missions, capabilities, chains, paths, data model…).
  Those are **sync-only** (name + description by UUID).
- **State machines and scenarios cannot be synced back at all**: the
  ArcLang AST stores no `id:` for them (they are identified by name), so
  renaming a state machine, state, scenario or message in the .arc does
  nothing in Capella. Forward emission only.
- Structural edits to the new kinds are not synced: changing `involves:`,
  `realizes:`, `mission:`, `elements:`, `values:`, class fields,
  transitions or messages in the .arc does not modify the Capella
  involvement/realization/literal/property objects.
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

- Functional exchanges (SA), physical exchanges, EPBS, interfaces and
  ports are not emitted. PA functional chains are not emitted (the ArcLang
  PA grammar has no `functional_chain`).
- Data-model **Unions** are not emitted (`search("Class")` matches strict
  Classes only); class properties with non-identifier names or named
  `id`/`description` are dropped (ArcLang attribute keys are identifiers).
- State machines: unnamed (pseudo)states and transitions touching them are
  skipped; guards (Capella Constraint objects) are not emitted; `initial:`
  is not derived; nested regions inside composite states are not descended.
- Scenarios whose lifelines represent no emitted element are skipped
  (4/7 in the test model: lifelines representing Roles or unemitted parts);
  message kinds other than ASYNCHRONOUS_CALL map to the default
  synchronous type (REPLY/CREATE/DELETE/TIMER are not distinguished).
- Operational processes / functional chains / physical paths whose
  involvements are all unemitted are skipped (involves must be non-empty).
- OA operational capabilities are not emitted, so SA `capability`
  `realizes:` links to them are dropped (only emitted targets are
  referenced).
- Requirement metadata beyond name/text (identifier, long_name, type
  attributes) is not emitted.
