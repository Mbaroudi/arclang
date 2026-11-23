# Versioning and Stability Policy

A model written in ArcLang must still compile years from now. This policy
is what makes depending on the language safe for long-lived programs.

## Language versioning (semver)

The **language** (syntax in `spec/GRAMMAR.ebnf`, typed metamodel in
`spec/METAMODEL.md`) is versioned independently of implementation details.
The current language version is the `LANGUAGE_VERSION` constant of
`src/compiler/metamodel.rs`, printed at the top of the metamodel document:

- **MAJOR** — a construct that used to compile no longer compiles, or
  compiles with different semantics. Requires a published migration
  guide and, where mechanically possible, an automated rewrite.
  The migration guide of each major version is `docs/MIGRATION_<version>.md`
  (`docs/MIGRATION_4.0.md`).
- **MINOR** — new constructs; every previously valid model still
  compiles with identical semantics and identical element identities.
- **PATCH** — fixes only; no grammar change.

### Metamodel changes

- Adding an element kind, a typed attribute, an enumeration value or a unit
  is **MINOR**.
- Removing one, or changing the type of an attribute, is **MAJOR**.
- A newly typed attribute never turns a compiling model into a compile
  error: violations are `metamodel:` warnings. They are blockers only in
  the production gate, whose verdict is expected to tighten between
  versions (4.0 typed `latency`, `bandwidth`, `safety_level`, `asil`,
  `multiplicity`, ...).
- Four things are errors although older versions compiled them, because the
  model said something the compiler could not honour: an attribute written
  twice in one block (the last value used to win silently); two requirements
  written with the same `id:` (one test verified both); a case executed on a
  state machine (`execute:`) whose expectation names a requirement the case
  does not verify, or that verifies a requirement none of its expectations
  checks (as for a simulated case); and a `configuration:` on a test case
  that names no configuration. An attribute spelled almost like one the
  compiler reads (`verdit:`) is a `metamodel:` warning, not an error.

### Verdicts and comparisons

A verdict is expected to tighten between versions, never to loosen.

- **Cached verdicts.** The key of a cached verdict carries the format of
  its stage. When a version changes what a verdict depends on or how it is
  judged, the format changes and every verdict cached by an earlier version
  reads as stale once: the gate says inconclusive and names the command to
  run again. The version after 4.12 does so for every stage (simulation,
  twin, SIL, co-simulation, PIL, HDL, SoC): the keys of the embedded stages
  now follow the plant a case closes its loop on and the machines wired
  upstream, and expectations are compared as below.
- **Comparisons.** In `expects:`, `constraint` and `rules`, two values are
  equal within 1e-9 of their scale (the larger of the two, the terms of a
  difference, the level a signal reached), no longer within 1e-9 of
  `max(|a|, |b|, 1)`: quantities under a nanosecond or a nanofarad are no
  longer all equal (`1.5 ns <= 1 ns` used to hold). A value that is not a
  finite number holds no expectation; in a constraint or a rule it is an
  error. A model that relied on the old absolute tolerance to compare a
  value with zero writes its tolerance (`within`), or compares at the scale
  it means.
- **What the gate asks for (after 4.13).** A requirement is satisfied by an
  element of the architecture: a `satisfies` trace from another requirement
  or from a test case no longer counts. A hazard whose fault tree fails
  beyond the rate of its integrity level blocks the gate (`check` already
  warned), and one whose failure modes carry no `fit:` is a warning.
  `mitigated_by: []` is no mitigation. A negative latency is not a time and
  blocks, instead of being subtracted from its chain.
- **Rules.** A comparison on an attribute the element does not have is
  neither true nor false, and `not` leaves it so: `not (mass > 50 kg)` no
  longer holds for an element with no mass, as `mass <= 50 kg` never did.
  Write `not has(mass) or ...` to let such an element pass.
- **Simulations.** A result that stops before `stop_time` is an error, not
  a run judged on the steps it made, unless the model ended the run itself
  (`terminate(...)`: the pack is charged, the trip is driven). A variable whose Modelica unit is
  `degC` is read in kelvin (`max(T) <= 55 °C` used to compare 80 with
  328.15). `sources:` are files of the model's folder (no absolute path, no
  `..`), a swept variable is a path with whole-number subscripts, and `nan`
  or `inf` is no level, no bound and no parameter of a law.
- **What a result says between two steps.** A variable the model declares
  as changing only at events (a Boolean, an integer, a discrete real) keeps
  its value between two steps: `at(5 s, pump.on)` is 0 or 1, never 0.5, and
  its mean counts each value for the time it was kept. A variable whose unit
  the model declares and the compiler does not read is compared with a plain
  number only; against a quantity (`max(x) <= 400 m/s`) the case is an error
  that names the unit, where a note used to say it was compared as SI.
- **Measurements of a twin.** A cell that is written and is not a number
  (`ERR`, `N/A`) in a column the twin reads is an error naming the line; it
  used to be skipped like an empty cell. An empty cell and `NaN` stay "no
  value". `ARCLANG_OMC_TIMEOUT` that is not a whole number of seconds above
  zero is said on the standard error, and the default applies.
- **Nesting.** A text whose braces, brackets and parentheses nest deeper
  than 200 levels is refused (a `.arc` model, a SysML v2 text, a rule); a
  Capella file beyond 1000 levels of elements too. They used to end the
  process.
- **SysML v2 comments.** The text of a `doc` or `comment` is read as the
  language says: on each line after the first, the white space before a
  star, the star and one space after it are a margin, not text. The export
  writes that margin, so a text of several lines comes back as written
  (it used to gain indentation, and a line starting with a star was read
  differently by other tools). A file written by hand with such a margin
  loses the stars on import, where it used to keep them.
- **Capella export of the operational analysis.** An `activity` written
  inside an `entity` is exported, performed by that entity; it used to be
  left out, with the interactions and the capability involvements that
  name it. `precondition` and `postcondition` of a capability (operational,
  system, or a capability realization) are Capella's pre-condition and
  post-condition, a constraint each, and come back as such on import; they
  used to be written as property values, and read back among `constraints`.

## Identity stability guarantee

Element UUIDs are UUIDv5 in the fixed ArcLang namespace
(`febb6e9d-b5a0-51d7-bb17-0e4e67346213`), derived only from the element
id. **This derivation never changes across versions** — it is guarded by
golden-value tests (`uuid_is_stable_across_versions`) cross-checked
against an independent implementation. Exports keyed on identity
(Capella sync, ReqIF, FMI GUIDs) therefore survive tool upgrades.

### Default ids

An element declared without an explicit `id` gets `<PREFIX>-<Name>` (spaces
as underscores): `SF-AcquireSensorData`, `BC-BrakingModule`. The whole name
is used. Up to 3.0 only the first three characters of the name were kept,
so distinct elements could silently share one identity (`BrakingModule`
and `BrakeActuation` were both `BC-Bra`). This changed in 4.0.0, a major
version for that reason: elements without an explicit id changed id, and
therefore UUID, once, and a reference written against a truncated id is a
compile error that names the element to reference instead. See
`docs/MIGRATION_4.0.md`. From 4.0.0 on, this derivation is covered by the
guarantee above. Give elements an explicit `id` when their identity must
survive a rename.

## Deterministic output guarantee

For a given input model and tool version, every output (JSON, ReqIF,
SysML, FMI, gate report content) is byte-identical across runs and
machines. Timestamps in exchange formats are fixed by design. A diff in
a generated artifact always means a model or tool change, never noise.

### Class field order

The fields of a `class` are exported in the order they are declared, as UML
and Capella show them. Pre-release builds before this note sorted them by
name. That was changed before the first stable release: every export that
lists fields (SysML v2 attributes, C struct members, Protobuf field numbers,
diagrams) changed order once. Declaring a field twice is a compile error.
From the first stable release on, appending a field never renumbers the
fields before it.

## Releases

- Releases are cut from tags `vX.Y.Z`; binaries for Linux and macOS are
  published with a `SHA256SUMS` file and GitHub build provenance
  attestations.
- Qualified environments must pin a release and verify checksums (see
  `docs/qualification/TOOL_QUALIFICATION.md` §7).

## Deprecation

A construct is never removed in the release that deprecates it: it first
produces a compile warning naming the replacement for at least one MINOR
release, then becomes an error only in the next MAJOR.
