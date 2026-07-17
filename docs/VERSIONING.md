# Versioning and Stability Policy

A model written in ArcLang must still compile years from now. This policy
is what makes depending on the language safe for long-lived programs.

## Language versioning (semver)

The **language** (grammar + semantics, specified in `spec/GRAMMAR.ebnf`)
is versioned independently of implementation details:

- **MAJOR** — a construct that used to compile no longer compiles, or
  compiles with different semantics. Requires a published migration
  guide and, where mechanically possible, an automated rewrite.
- **MINOR** — new constructs; every previously valid model still
  compiles with identical semantics and identical element identities.
- **PATCH** — fixes only; no grammar change.

## Identity stability guarantee

Element UUIDs are UUIDv5 in the fixed ArcLang namespace
(`febb6e9d-b5a0-51d7-bb17-0e4e67346213`), derived only from the element
id. **This derivation never changes across versions** — it is guarded by
golden-value tests (`uuid_is_stable_across_versions`) cross-checked
against an independent implementation. Exports keyed on identity
(Capella sync, ReqIF, FMI GUIDs) therefore survive tool upgrades.

## Deterministic output guarantee

For a given input model and tool version, every output (JSON, ReqIF,
SysML, FMI, gate report content) is byte-identical across runs and
machines. Timestamps in exchange formats are fixed by design. A diff in
a generated artifact always means a model or tool change, never noise.

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
