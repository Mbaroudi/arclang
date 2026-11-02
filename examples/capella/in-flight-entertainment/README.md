# In-flight entertainment (Capella sample)

The Eclipse Capella reference sample, imported as is: 1 600 declarations and 118 diagrams.

**Level:** Reference · **Shows:** Capella import, round trip, five levels, large model

## What it models

The sample shipped with Eclipse Capella (EPL-2.0), imported with `arclang import -f capella`. Five levels, functions and ports at every level, components and actors, physical nodes and links, functional chains, capabilities, state machines, scenarios with fragments, the data model and the requirements, plus the diagrams of the `.aird` as `diagram` declarations. The ids are the Capella ids, so `arclang export -f capella` gives the model back. Its findings are those of the source model and are left as they are.

## What to look at

- What a Capella model looks like as text.
- The `diagram` declarations kept from the `.aird`.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang workbench examples/capella/in-flight-entertainment/in_flight_entertainment.arc
arclang export examples/capella/in-flight-entertainment/in_flight_entertainment.arc -f capella -o ife.capella
```

## To understand by changing it

- Open one of the 118 diagrams from *Diagrams…* in the workbench.

## Files

- `in_flight_entertainment.arc`
