# Liquid cooling of an AI training hall

Sixteen GPU racks on direct-to-chip cooling, ten scenarios simulated in OpenModelica, from a pump failure to a grid outage.

**Level:** Advanced · **Shows:** simulation (Modelica), product line, failure scenarios, Monte Carlo, debugger, OMEdit drawing

## What it models

Sixteen GPU racks give 85 % of their heat to cold plates; a coolant distribution unit (CDU) carries it through a plate exchanger to the facility water; dry coolers give it to the outside air and a chiller helps on hot afternoons. Three variation points (racks, CDU, heat rejection) in four configurations. The scenarios: the design summer day, a CDU pump failing in a training run, a grid outage, a late generator, a leaking rack, next generation racks, the climate and load envelope, and a Monte Carlo over the summers. Every component of the physics is placed and wired, so OMEdit draws and simulates each scenario.

## What to look at

- The verdict of each scenario (the table below).
- The leak isolation chain and its latency budget.
- The Monte Carlo case: a probability of passing, and the parameters that matter most.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

Simulation needs OpenModelica: on the desktop, *ArcLang: Set up OpenModelica*; on arclang.fr, sign in and the simulations run on the server. The production gate waits for the simulations: until `arclang verify` has run the cases, it reports them inconclusive.

*ArcLang: Run…* (or F5) steps through its state machines, scenarios and functional chains in the debugger, on the desktop and in the browser.

From the command line, at the root of the repository:

```bash
arclang verify examples/datacenter/ai-liquid-cooling/ai_liquid_cooling.arc
arclang workbench examples/datacenter/ai-liquid-cooling/ai_liquid_cooling.arc
```

## To understand by changing it

- Run the pump failure with two pumps, then with three (N+1): the report shows the GPUs throttled, then not.
- Run the Monte Carlo case and read the sensitivity of the margin to each parameter.

## Files

- `ai_liquid_cooling.arc`
- `physics/CoolingPhysics.mo`

## The scenarios and their verdicts

| Scenario | Verdict |
|---|---|
| Design summer day (35 °C), N+1 pumps | pass: GPUs at 76 °C at most, supply at most 36.4 °C, partial PUE 1.077 |
| A CDU pump stops at 14 h, two pumps | fail: the controller caps the GPUs to 69 % of their power to hold 83 °C |
| The same failure, N+1 pumps | pass: the spare pump takes over |
| Grid outage at 14 h (pumps on the UPS, fans on the generator after 30 s, chiller back after 10 min) | pass: the dry coolers alone hold the GPUs at 79 °C |
| The generator 1, 3 or 5 min late, with the loop's 40 MJ/K or a buffer tank (160 MJ/K) | 5 of 6: 3 min ride through on the loop alone; 5 min throttle (94 %) unless the tank is there |
| A rack leaks at 14 h: valves closed in 8.5 s, rack powered off 10, 20, 30 or 60 s after the alarm | 2 of 4: off within 20 s its GPUs stay under 95 °C; at 30 s they reach 102 °C, at 60 s 140 °C; the other racks train on |
| Racks of 200 kW on the current plant | fail: GPUs capped to 73 % |
| Racks of 200 kW, high flow CDU, larger coolers, 1.6 MW chiller | pass: GPUs at 77 °C, partial PUE 1.075 |
| 30 to 45 °C outside × 100 to 132 kW per rack | 9 of 12: at 45 °C (and 40 °C with 132 kW) the GPUs throttle |
| Monte Carlo over the summers (35 °C ± 3 K, 110 to 132 kW per rack) | pass: 60 of 60 days without throttling |
