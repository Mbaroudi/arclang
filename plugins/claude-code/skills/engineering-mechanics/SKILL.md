---
name: engineering-mechanics
description: Use when a mechanical part is part of an ArcLang model: a build123d script, a FreeCAD document or a STEP file realizing a node, loads and supports, stress or displacement limits, a thickness to size, materials to compare, rotating wheels or resonance.
---

# Mechanics as code

```arc
node "Bracket arm" {
  id: "PN-ARM"
  realized_by: ["part:bracket.py"]
  parameters: ["thickness=6 mm"]
  material: "MAT-PLA"
  fixed: ["x=min"]
  loads: ["50 N on x=max direction -z"]
  limits: ["displacement < 8 mm", "stress < @MAT-PLA.yield_strength"]
}
```

- `part:` takes a build123d script (`.py`), a FreeCAD document (`.FCStd`, its `Params`
  spreadsheet aliases take the model's `parameters`) or a STEP file (`.step`, `.stp`, in
  millimetres). A STEP file is a fixed shape: `parameters` is refused with it.
- The material is an entry of the registry (Young's modulus, Poisson ratio, yield strength,
  density), each with a source.
- The part is meshed and solved by two solvers (scikit-fem and CalculiX). Where they
  disagree beyond the tolerance the verdict is `inconclusive` and the gate refuses it: fix
  the support or the geometry (a smooth hub, a fillet), never loosen the tolerance.
- `action=size`: a parameter over a range (`part`, `parameter`, `from`, `to`, `step`; at most
  41 values), each solved and judged. `action=trade`: the same shape and loads under two or
  more materials of the registry; it proposes the lightest that passes and does not change
  the model.
- Stress goes with the square of the speed for a rotating wheel: sweep the speed, do not
  extrapolate from one point.

Run `arclang_engineering action=connectors` first; read `engineering-verdicts` for how to
read the answer.
