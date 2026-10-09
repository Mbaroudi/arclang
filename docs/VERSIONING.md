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
