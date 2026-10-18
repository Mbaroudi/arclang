# Arcadia in SysML v2: the profile ArcLang reads and writes

ArcLang exchanges models with SysML v2 through the element listing of the
Systems Modeling API (JSON records: `@type`, `@id`, names, relationships).
The textual notation is read into that listing first. This page fixes how
an Arcadia model is said in SysML v2 so that it comes back whole, and how a
SysML v2 model written without Arcadia in mind is read.

Status: the import (`arclang import -f sysml`) reads everything below. The
export (`arclang export -f sys-ml`) is being brought to it; the round-trip
bench (`tests/sysml_round_trip.rs`) measures the distance, kind by kind.

## Elements are usages

The structure of a SysML model is its usages: `part engine : Engine` is
the component, `Engine` its type. So an ArcLang element is a SysML
**usage** (`part`, `action`, `port`, `requirement`, `use case`, `state`...),
written where it sits (in its level package, in its parent part), carrying
its short name, name, documentation and attributes. An ArcLang `type` is a
SysML **definition**; an element `is:` its definition when it has one. A
plain ArcLang model, which declares no types, exports as usages alone: no
definition is invented for it.

Reading models written the other way round: a definition used once by an
anonymous usage (`part def <'X'> X { ... } part p_X : X;`) is one element,
identified by the definition; a definition used nowhere is an element of
its own; a definition used several times becomes a `type`, its usages the
elements.

## Identity and names

| ArcLang | SysML v2 |
|---|---|
| `id:` | short name: `part <'LC-001'> ...`. Every written element has one. |
| name | declared name, as written, quoted when it is not an identifier: `'Radar Sensor'`. |
| `description:` | `doc /* ... */` of the element. |
| other attributes | `attribute key : Type = value;` typed by the metamodel (quantities with their ISQ type and unit, else `String`, `Integer`, `Real`, `Boolean`). List attributes are lists (`(a, b)`), never a joined string. |
| references (`involves:`, `realizes:`, `allocated_functions:`...) | relationships (below), never text attributes. |

## Levels

An Arcadia level is a package carrying the level's metadata:

```sysml
package 'Operational Analysis' { #OA ... }
package 'System Analysis'      { #SA ... }
package 'Logical Architecture' { #LA ... }
package 'Physical Architecture'{ #PA ... }
package 'EPBS'                 { #EPBS ... }
```

The metadata definitions live in a library package the export writes once
(`ArcadiaProfile`): `metadata def OA; metadata def SA; metadata def LA;
metadata def PA; metadata def EPBS;` and the kinds below. An element may
carry a level metadata itself (`#LA part def ...`) when it is not in a level
package, or an `Arcadia` metadata with a `level` attribute
(`@Arcadia { level = "la"; }`).

Reading a model without level metadata: a package whose name contains
`operational`, `system`, `logical`, `physical`, `epbs` or `product` gives the
level; otherwise actions are system functions and parts logical components.

## Kinds

SysML says most Arcadia kinds by itself; metadata names the rest.

| Arcadia | SysML v2 | Metadata when SysML cannot tell |
|---|---|---|
| operational entity / actor | `part` in OA | `#Entity`, `#Actor` |
| system component / actor | `part` in SA | `#Actor` |
| logical component | `part` in LA, nested components as nested `part`s | `#Actor` for an actor |
| physical node / behaviour / hardware component | `part` in PA; behaviours and hardware nested in the node | `#Node`, `#Behaviour`, `#Hardware` |
| function / activity | `action` of the level, or `perform action` declared in the part that performs it; `in`/`out` item parameters are the ports | |
| functional chain / process | `action` whose steps are `action`s referencing (`:>`) the functions, with `then` successions (control nodes: `fork`/`join`/`decide`/`merge`) | `#Chain` |
| component port | `port` usage, direction by `in`/`out` on its port def or `~` conjugation | |
| physical port | `port` without direction | `#PhysicalPort` |
| functional exchange / interaction | `flow` between the ports (item parameters) of two actions, carrying its exchange items | |
| component exchange / communication mean / physical exchange | `connection` (or `interface`) between the ports of two parts | |
| physical link | `connection` between two nodes or their physical ports | `#Link` |
| exchange item | `item def` | `#ExchangeItem` when it has attributes |
| class | `item def` with attributes; `extends` by specialization | `#Class` |
| data type / enumeration | `attribute def` / `enum def` | |
| ArcLang `type` | `part def` / `action def`, its ports and attributes; `type X extends Y` by specialization | |
| requirement | `requirement` with `doc` and attributes | |
| hazard | `requirement` | `#Hazard` |
| mission / capability / operational capability / capability realization | `use case def` of the level | `#Mission`, `#Capability` |
| state machine | `state def` exhibited by the part or action it belongs to; `state`/`mode` usages, `transition` with `accept`/`if`/`do` | `#Mode` on a mode |
| scenario | `occurrence def` with one `part` per lifeline, `event occurrence`s and `flow` messages between them (fragments as `loop`/`alt`... occurrences) | |
| test case | `verification def` with `objective { verify requirement }` | |
| trace | `satisfy`, `allocate`, `dependency` named after the trace kind (`realizes`, `refines`, `implements`, `verifies`, `validates`) | |
| deployment | `allocate` of a logical component to a node | |
| diagram | `metadata` on the level package: `@Diagram { view; shows }` (not yet) | |

## What the import does with a SysML model

- Elements are the structural usages (parts, actions) and the definitions
  used nowhere; a usage nested in a part is its child (a behaviour on a
  node, a performed function); a step of a chain, a lifeline of a scenario
  or the subject of a requirement only refers to one.
- A usage's level is stated on it or its packages, else its parent's, else
  the default of its kind.
- Library elements (`isLibraryElement`) are not written.
- Connectors are read by their ends: `source`/`target`, `client`/`supplier`,
  `relatedFeature`, or ArcLang's own `arclang:sourceElement`. An end that is
  a port becomes `Owner.port`.
- A text attribute under a reference key (`involves`, `realizes`...) is left
  out and counted: a relationship is expected.
- The written model must compile; else the import fails and keeps the text
  as `<output>.rejected.arc`.
