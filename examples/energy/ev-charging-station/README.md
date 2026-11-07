# DC fast-charging station

A 350 kW public charger and its back end: CCS2, ISO 15118 plug-and-charge, OCPP 2.0.1, load management, payment.

**Level:** Advanced · **Shows:** IEC 61508, units and constraints, OCPP state machine, HARA and FMEA, cybersecurity, SysML v2 round trip

## What it models

CCS2 liquid-cooled connector at 1000 V / 500 A, ISO 15118-2 plug-and-charge, OCPP 2.0.1 to the CSMS over TLS 1.3, a 630 kVA grid connection shared by eight chargers under load management, an EMV payment terminal and an RFID fallback, through every Arcadia level. Typed attributes with units (`power: 350 kW`, `efficiency: 97 %`) and five compiler-evaluated constraints, including `voltage * current >= power`.

## What to look at

- The constraints with units: the compiler checks the dimensions.
- The OCPP connector state machine with function-triggered transitions.
- The cybersecurity analysis: payment data, firmware, session records.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate --standard iec61508 examples/energy/ev-charging-station/ev_charging_station.arc
arclang workbench examples/energy/ev-charging-station/ev_charging_station.arc
```

## To understand by changing it

- Lower `rated_current` until `voltage * current >= power` fails, and read the message.

## Files

- `ev_charging_station.arc`
