---
name: engineering-chip
description: Use when a block of digital logic written in Verilog is part of an ArcLang model and must be judged as a chip block: synthesis on SKY130, timing, area, equivalence of the netlist, and placement, routing, DRC and power with layout.
---

# A chip block as code

```arc
node "Event counter" {
  id: "PN-COUNTER"
  realized_by: ["asic:counter.v"]
  top: "counter"
  clock: "clk"
  period: "10 ns"
  platform: "sky130hd"
  limits: ["area < 400 um2", "cells <= 40", "setup_slack >= 0 ns", "hold_slack >= 0 ns", "fmax >= 200 MHz"]
}
```

- Lot 1: Yosys synthesises on the SKY130 HD standard cells, OpenSTA times the netlist, and
  the netlist is proved equivalent to the RTL before any figure becomes a verdict. A block
  whose equivalence is not proved has no verdict.
- Lot 2: add `layout: "true"`. OpenROAD (the flow scripts) places, builds the clock tree and
  routes to GDS; KLayout runs DRC and LVS. The limits then include `layout_area`, `routed_*`
  slack, `power` and `drc`. A layout is minutes of a big machine: expect a long run.
- Only one top and one clock are supported, and only the SKY130 platform; several files or
  clocks are not done. Say so when asked for them.

Use `arclang_engineering action=model`, then read the first line. For the FPGA path of the
same logic (VHDL, GHDL, nextpnr), use the agent `fpga-hdl-engineer`, not this skill.
