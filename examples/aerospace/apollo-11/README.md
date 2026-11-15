# Apollo 11, from SysML v2

The Apollo 11 model of Philipp Helle and Gerrit Schramm (Airbus), *Fly me to
the Moon: Modeling Apollo 11 using SysML v2* (INCOSE *Systems Engineering*,
2026), [airbus/apollo-11-sysml-v2](https://github.com/airbus/apollo-11-sysml-v2),
under the Mozilla Public License 2.0 ([`LICENSE-MPL-2.0.md`](LICENSE-MPL-2.0.md)).
It describes the mission in the five layers of the authors' CoSMA framework
(purpose, operation, function, logical, technical), with 297 requirements
traced down to the Saturn V, the command and lunar modules and the crew.

[`apollo11.arc`](apollo11.arc) is that model read by `arclang import`, at
commit `0e8a7d6`. It is a modification of the original and stays under the
Mozilla Public License 2.0; nothing was edited by hand but the name of the
model.

```text
arclang import -f sysml -o apollo11.arc apollo-11-sysml-v2/
arclang check apollo11.arc
arclang gate apollo11.arc
```

## What it shows

A model written by others, for another tool, is the test an importer and a
checker cannot pass by being written for each other. Reading it found four
defects in ArcLang, fixed since:

- a port's direction was guessed from letters in the name of its type, so a
  `StagingPort` and a `DockingPort` both became inputs; it now comes from the
  directed items of the definition, reversed for a conjugated port (`~Port`);
- `not x` was written `notx`;
- an astronaut written `part 'John Young' : 'John Young'` took the id of its type, so
  `arclang edit` of the type's attributes landed on the astronaut; an individual now
  never takes a type's name as its id;
- a requirement's `require` was one comparison. The model writes
  `a >= b and not c`, `structuralIntegrityMaintained`: 56 constraints, 33 of
  them with `and`, `not` or a bare boolean. A condition is now comparisons
  joined by `and`, `or`, `xor`, `implies`, negated by `not`, a boolean feature
  standing alone.

## What the compiler finds in the model

`arclang gate` reports 14 technical requirements (`CLR-`) that no element
satisfies: R055 (thrust of the first stage), R059, R061 to R066, R070, R072,
R074, R077, R104 and R105. The same list comes from reading the SysML source
for `satisfy`. Eleven are declared and never used: not instantiated in the
specification, not satisfied, refining nothing. Three (R077, R104, R105)
refine a functional requirement, so `arclang check` counts them as traced;
only the gate asks that something satisfy them.

The gate also lists the 22 stakeholder needs (`SHN-`) and the three top-level
specifications, which are not satisfied by an element either, and HLR-R060,
which the source satisfies through a container action ArcLang does not carry
over yet.

## Not carried over

- `arclang check` prints 154 warnings that `refines` links a requirement to a
  requirement: the model refines functions and actions with `#refinement`,
  which ArcLang keeps as a trace.
- 177 components have no trace, many of them individuals (astronauts,
  LM-1 to LM-15).
