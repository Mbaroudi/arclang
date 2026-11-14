# Hybrid powertrain, P1 and P4

The case study of Banagar, Kramsu, Könnö and Andwari, *Connecting systems
engineering and domain expertise through a toolchain: A simulation-integrated
MBSE approach*, Journal of Systems and Software 232 (2026) 112671,
[doi:10.1016/j.jss.2025.112671](https://doi.org/10.1016/j.jss.2025.112671),
open access under CC BY 4.0. Their SysML v2 models (appendix B, and
[`sec_4/`](https://github.com/mvd-oulu/Standard-System-Interface/tree/SSI_v1.0.0/sec_4)
of their MIT-licensed repository) written in ArcLang: five subsystems, their
ports and the interfaces between them, with the units their attributes leave
implicit.

The article itself is here:
[`paper/banagar-2026-jss-112671.pdf`](paper/banagar-2026-jss-112671.pdf)
(27 pages, © 2025 the authors, published by Elsevier under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), copied unchanged
from the University of Oulu repository).

## Context

The paper is about the gap between the people who draw the system and the
people who simulate it. A system engineer models the architecture in SysML
v2; domain experts (engine, battery, vehicle dynamics) build the
physics-based simulations of each subsystem as FMUs. The authors propose a
workflow, the *Standard System Interface* (SSI), that joins the two:

- **Three standard packages**, written once with the domain experts: a
  *general definition* package (the parts and their attributes: vehicle,
  engine, motor, battery, control unit), a *port definition* package (one
  port per item, outgoing; an input is the same port conjugated, `~`) and an
  *interface definition* package (which port connects to which, carrying
  what). Five parts, eight ports, six interface definitions in all
  (section 4.1, Figs. 4–6).
- **A system definition package** per architecture, built only by reusing
  those definitions, so a system engineer can propose a new variant without
  knowing the inside of each subsystem (section 3.1, Figs. B.4–B.5).
- **The SSI transformer**, an open-source tool that reads the SysML v2 text,
  checks the port types against the interfaces, and writes the co-simulation
  architecture as an SSP System Structure Description (SSD) wiring the FMUs
  together (section 3.2). The SSDs were opened in AVL Model Connect with
  FMUs built in AVL Cruise M (Figs. 9–10).
- **Two communication points** where the system engineer and the domain
  experts meet: once to agree on the interfaces, once to review the
  architecture before it goes to simulation (Fig. 3). The authors insist the
  tool does not replace these meetings (section 4.4).

The case study applies the workflow to two parallel hybrid powertrains,
named after where the electric motor sits (section 4):

- **P1**: the motor is on the engine's crankshaft. The engine carries a
  second pair of mechanical ports towards the motor, and the vehicle has a
  single torque/speed interface with the propulsion. Mild hybrid: 48 V,
  5 kWh (Fig. 7).
- **P4**: the motor drives the second axle directly. The vehicle gets a
  second torque/speed interface, one for the engine and one for the motor,
  and the engine no longer connects to the motor. The motor is larger and
  the electrical system runs at 400 V, 10 kWh (Fig. 8).

The control unit is the same in both. The paper's point is that the second
architecture came from the same standard packages with no change beyond
choosing the parts.

This example adds two things the paper leaves open. **Units**: the authors
note (section 4.4) that units and quantities were handled by hand and are
not part of the transformation; here every attribute carries its unit
(`1500 kg`, `48 V`, `5 kWh`), and requirement `SYS-HP-002` asks for it.
**Directions**: the paper relies on SysML v2 to catch mismatches between
ports and interfaces, but that check compares item types, not which end
sends. The published P4 model has the motor's mechanical ports reversed, and
nothing in their chain flags it, as the next section shows.

| File | What |
|---|---|
| `hybrid_powertrain_p1.arc` | P1: the motor on the crankshaft, a 48 V battery of 5 kWh |
| `hybrid_powertrain_p4.arc` | P4: the motor on the second axle, a 400 V battery of 10 kWh, as published |
| `hybrid_powertrain_p4_fixed.arc` | P4 with the motor's mechanical ports corrected |

## What the compiler finds

In the published P4 model, the electric motor's mechanical ports are the
reverse of the P1 motor's: `port mtr_trq_out:~mech_trq;` and `port
mtr_speed_in:mech_spe;`, where P1 has `mtr_trq_out:mech_trq` and
`mtr_speed_in:~mech_spe`. Written with those directions, two interfaces join
ends that both receive or both send:

```text
$ arclang check examples/automotive/hybrid-powertrain/hybrid_powertrain_p4.arc
⚠ Compilation warnings:
  exchange direction: component_exchange 'int5: trq_int' leaves 'em.mtr_trq_out', an in port: an exchange leaves an out (or inout) port
  exchange direction: component_exchange 'int6: spe_int' arrives on 'em.mtr_speed_in', an out port: an exchange arrives on an in (or inout) port
```

The rule is `interface.direction` (`arclang rules`). The paper's own
transformer compares port types with the conjugation removed
(`port_types_compatible` strips `~`), and its SSD takes each connector's
kind from its role in the connection, so the co-simulation it generates is
right while the system model says otherwise. The P1 model and the corrected
P4 pass.

## Diagrams

Each sheet is the interconnection view `arclang diagram -f html` draws,
with the interfaces `arclang check` reports in red, then every part with its
attributes and ports, and every interface with its ends. PNG at twice the
resolution, PDF in vector; [`hybrid-powertrain.pdf`](diagrams/hybrid-powertrain.pdf)
holds the three sheets.

| Model | PNG | PDF |
|---|---|---|
| P1 | [p1.png](diagrams/p1.png) | [p1.pdf](diagrams/p1.pdf) |
| P4 as published | [p4-as-published.png](diagrams/p4-as-published.png) | [p4-as-published.pdf](diagrams/p4-as-published.pdf) |
| P4 corrected | [p4-corrected.png](diagrams/p4-corrected.png) | [p4-corrected.pdf](diagrams/p4-corrected.pdf) |

![P4 as published: int5 and int6 in red](diagrams/p4-as-published.png)

They are made by `tools/hybrid_powertrain_figures.mjs` (Playwright with
Chrome):

```text
ARCLANG_BIN=target/release/arclang node tools/hybrid_powertrain_figures.mjs
```

## Not carried over

Torque is written as a plain number (`max_torque_nm`): ArcLang has no torque
unit yet. The simulations (FMUs built in AVL Cruise M) are not part of the
paper's repository and are not reproduced here.
