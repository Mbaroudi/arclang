# Capella bridge (M3 — proof of concept)

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
  9 component exchanges (LA); 2 nodes (PA)
- Output compiles with the strict v3 parser: **33 components,
  21 functions, zero warnings** — every exchange endpoint resolves.

## Current scope / not yet covered

- Requirements (Capella requirement extensions), functional exchanges (SA),
  physical links/exchanges, EPBS, modes/states, scenarios
- The reverse direction (arc → Capella) — planned via capellambse's write
  support; UUID preservation makes it possible
