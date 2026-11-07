# A charging corridor in space

Three charging hubs on a motorway, the fibre between them and the supervision center, placed on the map.

**Level:** Intermediate · **Shows:** locations (WGS 84), shapes and positions, routes, latency from length, GeoJSON and STEP exports

## What it models

Three charging hubs on the A6 between Auxerre and Beaune, the fibre that links them along the motorway, and the supervision center in Lyon. Each hub stands where it is, its cabinet has a size and its equipment a place in it, and the links have a route, a length and a latency the compiler checks.

## What to look at

- `location`, `shape` and `route` on the physical elements.
- The `spatial.*` rules: a link shorter than the ground it covers, longer than its `max_length`, or with a latency faster than light.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang check examples/spatial/charging-corridor/charging_corridor.arc
arclang export examples/spatial/charging-corridor/charging_corridor.arc -f geojson -o corridor.geojson
```

## To understand by changing it

- Open the GeoJSON export on a map (geojson.io), then move a hub and export again.
- Set the `latency` of the fibre below the time light takes over its length: `spatial.latency` blocks the gate.

## Files

- `charging_corridor.arc`
