# A drone from Capella to simulation

The architecture of an inspection drone is authored in Capella. ArcLang
reads it, adds what simulation needs in a file of its own, rolls the
executable model of each element up into one system model per
configuration, flies each mission in OpenModelica and judges the
requirements.

**Level:** Advanced · **Shows:** Capella import, `refine`, the roll-up of element models, missions in their own files, parameter impact

## The files

| File | Who writes it | What it holds |
|---|---|---|
| `capella/drone.capella`, `.aird` | the architect, in Capella | the functions, the logical and physical components, their ports and links, the requirements, the parameters (property values: `energy: 666 Wh`, `payload: 2 kg`) |
| `drone.arc` | `arclang import`, never edited | the same, in ArcLang: a new import replaces it |
| `simulation.arc` | the simulation engineer | `import "drone.arc"`, then by element id: the physics of each port, the executable model of each element (`model:`), the product line, the cases |
| `missions/*.mission` | the simulation engineer | the values of a mission, one parameter a line; a range sweeps |
| `physics/DronePhysics.mo` | the simulation engineer | one Modelica model per element, each extending the class ArcLang generates for it; no system model |

## Run it

```bash
cd examples/aerospace/drone-from-capella
arclang import capella/drone.capella -f capella -o drone.arc   # the architecture, from Capella
arclang verify simulation.arc                                   # each mission in OpenModelica
```

```text
✓ TC-SURVEY "Survey mission, standard pack": PASS
    ok   min(battery.soc) = 0.3797   (min("PN-BATT".soc) >= 0.2, 0.1797 to spare)
✓ TC-ENDURANCE "Hover endurance, extended pack": PASS
    ok   min(battery.soc) = 0.2551   (min("PN-BATT".soc) >= 0.2, 0.0551 to spare)
✓ TC-ENVELOPE "Operating envelope, extended pack": PASS
    sweep: 9 points, 9 pass, 0 do not
    ...
    ok   payload = 2 kg, ambient = 35 °C
        ok   min(battery.soc) = 0.2367   (min("PN-BATT".soc) >= 0.2, 0.03667 to spare)
```

## What ArcLang does with it

- **Capella's parameters as quantities.** A property value that reads as
  one (`666 Wh`, `20 °C`, `85 %`) comes back as a quantity, with its unit
  checked: the parameters the architect sets are the ones simulated.
- **`refine "<id>" { ... }`** adds to an element declared in another file:
  the physics of its ports, its executable model, what Capella has no
  notion of (a variation point and its variants). The import is never
  edited; after a new one, every refinement still applies, by the ids
  Capella gave the elements.
- **The roll-up.** Each element names its executable model (`model:
  "DronePhysics.Pack"`); `arclang export -f modelica` writes
  `DroneEndurance.Executable.<Configuration>`, the system model of each
  configuration with every element redeclared by its model (a node whose
  behavior components have models gets a class of its own). A case that
  names no model simulates the roll-up of its configuration.
- **Missions.** `simulate: { mission: "missions/envelope.mission" }` reads
  the mission's lines as the case's parameter values; a range or a list
  sweeps (`"PN-FRAME".payload = 0 kg .. 2 kg step 1 kg`). The mission is
  part of what the verdict depends on: change it and the verdict is stale.

## Change it in Capella

Lower the extended pack's energy from 666 Wh to 560 Wh in Capella (or in
`capella/drone.capella`), import again and verify:

```text
✗ TC-ENDURANCE "Hover endurance, extended pack": FAIL
    FAIL min(battery.soc) = 0.1141   (min("PN-BATT".soc) >= 0.2, short by 0.0859)
```

Nothing was copied by hand from the architecture to the simulation.
