# AI accelerator platform

A tray of eight AI accelerators, from the ML engineer's need down to the tensor cores, HBM and the cooling loop.

**Level:** Advanced · **Shows:** five Arcadia levels, units (Gbps, K, kWh), modes, HARA and FMEA, cybersecurity, three standards

## What it models

Tensor cores, 141 GB HBM3e per device at 4.8 TB/s, 700 W TDP, a 900 GB/s GPU-to-GPU fabric, a PCIe Gen5 host link and liquid cooling at a 308 K inlet. Chains with latency budgets (all-reduce step 2.5 ms, thermal throttling 100 ms, power brake 1 ms, attested boot 30 s), a power and thermal mode machine, three scenarios, a HARA with FMEA, a cybersecurity analysis (model weights, tenant data, firmware), 16 requirements each satisfied and verified.

## What to look at

- Units as the metamodel wants them: 4.8 TB/s is 38400 Gbps, 35 °C is 308 K.
- The power and thermal modes, triggered by declared functions.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang gate examples/semiconductor/ai-accelerator/ai_accelerator_platform.arc
arclang workbench examples/semiconductor/ai-accelerator/ai_accelerator_platform.arc
```

## To understand by changing it

- Change the TDP and follow the impact through the power and cooling elements.

## Files

- `ai_accelerator_platform.arc`
