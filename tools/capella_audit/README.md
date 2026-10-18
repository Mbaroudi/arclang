# Capella import audit

Measures how much of a Capella model its ArcLang import keeps, element by
element, so that the coverage is a number and not a feeling.

```sh
arclang import -f capella "Model.capella" -o model.arc
python3 tools/capella_audit/audit.py "Model.capella" model.arc "Model.aird"
```

Every typed element of the `.capella` lands in one bucket:

- **declared**: written with its Capella id;
- **carried**: alive inside another declaration (a class field, a port, an
  `involves` list, a trace, a nested state);
- **language gap**: the compiler has no place for it yet, named as such;
- **tooling**: Capella's own bookkeeping (transition links, REC/RPL
  catalogues, key values, packages), not model content.

The diagrams of the `.aird` are counted apart: declared, an element's own
view (scenarios, state machines, chains, paths), or left out.
