# An electric car product line

What Arcadia gains from SysML v2: definitions, variants and configurations, typed allocations, tagged views, analyses.

**Level:** Intermediate · **Shows:** variants and configurations, port and interface definitions, typed allocations, views by tags, SysML v2

## What it models

A small electric car line written with the constructs ArcLang shares with SysML v2: reusable port and interface definitions, variation points and configurations, allocations with a type, views filtered by tags, and analyses tied to requirements.

## What to look at

- The variation points and the configurations that pick a variant in each.
- The views: each one keeps the elements that carry its tags.
- Export it to SysML v2 and read the same structure in the standard's syntax.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang check examples/automotive/ev-product-line/ev_product_line.arc
arclang export examples/automotive/ev-product-line/ev_product_line.arc -f sys-ml -o ev_product_line.sysml
```

## To understand by changing it

- Add a configuration that picks another variant, and open it in the workbench.

## Files

- `ev_product_line.arc`
