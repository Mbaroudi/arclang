# Migrating from ArcLang 3.0 to 4.0

ArcLang 4.0.0 is a major version: some models that compiled with 3.0.0 no
longer compile, and some compile to a different result. This note lists
every such change we know of, how to tell whether a model is affected, and
what to do. Changes that only add things are at the end.

The language version (`LANGUAGE_VERSION`, printed at the top of
`spec/METAMODEL.md`) and the tool version are both 4.0.0.

How to migrate a model: run `arclang check model.arc` with 4.0.0. Every
break below is reported as a compile error that names the element to fix;
none of them is silent. Then run `arclang gate model.arc`, whose verdict is
stricter than in 3.0 (section 3).

## 1. Models that no longer compile

### 1.1 Default identifiers are no longer truncated

An element declared without an `id` used to get an identifier made of a
prefix and the first three letters of its name. Two different elements
could silently share one identity: `BrakingModule` and `BrakeActuation`
were both `BC-Bra`. The identifier is now the prefix and the whole name,
with spaces as underscores: `OA-MonitorEnvironment`, `BC-BrakingModule`.

Affected: a model that refers to an element by its old truncated
identifier, typically in a trace.

```
trace "AcquireSensorData" realizes "OA-Mon" {}
```

```
unknown element 'OA-Mon' (to) — default ids are no longer truncated:
reference "MonitorEnvironment" by name, or give it an explicit id
```

Fix, one of:

- refer to the element by its name: `realizes "MonitorEnvironment"`;
- give the element an explicit identifier and refer to that:
  `activity "MonitorEnvironment" { id: "OA-MON" }`.

**Identity changes once.** Element UUIDs are derived from the identifier
(`docs/VERSIONING.md`). An element without an explicit `id` therefore gets
a new UUID in 4.0, and so do the artifacts keyed on it: Capella
synchronisation, ReqIF identifiers, FMI GUIDs. To keep the UUID an element
had in 3.0, give it its old truncated identifier explicitly
(`id: "OA-Mon"`): an explicit identifier is used as written. This is only
possible when that truncated identifier was not shared by two elements.

Elements that already had an explicit `id` are not affected.

### 1.2 Class fields

- A field declared twice in one `class` is an error. The later declaration
  used to win silently.
- A field whose type is not text is an error. It used to be dropped
  silently.

Fix: remove the duplicate; write the type as a name or a string.

### 1.3 Capabilities

`extends`, `includes` and `specializes` in a capability block are now
relations between capabilities, not free attributes.

- Their target must be a declared capability of the same level
  (operational or system). An unknown target is an error.
- A cycle through `specializes` or `includes` is an error.
- `involves` must name a declared actor, entity, component or function of
  that level.
- The same attribute key written twice in a capability block is an error.
- Two capabilities that resolve to the same identifier, explicit or
  default, are an error.

Affected: a model that used these keys as free text. Fix: name real
capabilities, or move the text to `description`.

## 2. Models that compile to a different result

### 2.1 SysML v2 export

The exporter was rewritten and its output is validated by the OMG pilot
implementation. A 3.0 export and a 4.0 export of the same model differ:
typed attributes carry their ISQ value type and unit
(`attribute latency : DurationValue = 25 [ms];`), exchanges are bound to
ports, scenarios, state machines, constraints and user-defined types are
exported, units the SI library lacks are declared in a local package.
Anything that post-processed the 3.0 text must be revisited. The export is
deterministic, as before.

### 2.2 Order of class fields

Class fields are emitted in declaration order in every output (JSON, SysML,
diagrams). In 3.0 their order was arbitrary.

### 2.3 Semantic model (JSON output, `/api/compile`)

- `capabilities` also holds operational capabilities, with a kind
  (`Operational` or `System`), a `parent` and `relations`. Nested
  capability blocks are flattened; the default identifier of a nested
  capability is `<parent id>/<Name>`.
- System actors and operational processes are registered elements: a trace
  or `involves` can target them, and two elements can newly collide on an
  identifier that used to be free. A collision is reported.
- New fields: `constraints`, `types`.

### 2.4 Explorer and diagrams

- `arclang explorer` keeps its sections; the diagram section is replaced by
  the viewpoint viewer (one tab per kind of view, model tree, navigation
  across layers). The page no longer loads D3 or dagre and embeds the
  layout engine, so it needs no network. Anything that post-processed the
  old diagram markup or script tags will not find them.
- `arclang diagram`: no format was removed and `mermaid` is still the
  default. `--layout` is rejected for the formats that are not viewpoints.
- `arclang diagram -f html` and `arclang explorer` read
  `<model>.layout.json` next to the model when it exists. A malformed
  layout file is an error.

## 3. A stricter production gate

Models still compile, but `arclang gate` can fail where it passed.

- Attributes the metamodel now types (`latency`, `wcet`, `period`,
  `bandwidth`, `safety_level`, `asil`, `multiplicity`, ...) are checked. A
  violation is a `metamodel:` warning at compile time and a blocker in the
  gate. See `spec/METAMODEL.md` for each kind's typed attributes.
- Timing is read as typed quantities. `latency: 25 ms` is the form to
  write. A latency without a unit is assumed to be milliseconds and
  reported as a warning; a latency that is not a time is a blocker.
- A violated `constraint` is a blocker.

Fix: write quantities with their unit (`25 ms`, `100 Mbps`, `512 MB`) and
enumerated values with a spelling of the metamodel.

## 4. Rust library API

For code that links the `arclang` crate:

- `compiler::sysmlv2_generator::generate_sysmlv2` takes the syntax tree as
  well: `generate_sysmlv2(&semantic_model, &ast)`.
- `ast::AttributeValue` has a new variant, `Quantity`; an exhaustive
  `match` on it needs an arm.
- `compiler::arcviz_elk_static` is removed.
- `web_server::systems_modeling::router` takes a shared workspace and a
  vocabulary.

## 5. What is new and breaks nothing

Described in the README: typed quantities and derived dimensions,
constraints, user-defined types, multiplicities, the percent sign as a
unit, `arclang fmt`, `arclang set` / `unset` / `rename`, the Systems
Modeling API with git history, authentication, writes and the SysML v2
vocabulary, `arclang review`. The `format` command, which answered `Not
implemented` in 3.0, is implemented.
