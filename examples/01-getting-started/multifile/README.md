# A model in several files

Each team owns its file; the root file imports them and the traces resolve across the files.

**Level:** Beginner · **Shows:** imports, team ownership, cross-file traces

## What it models

`main.arc` imports `requirements.arc` (the systems engineering team) and `logical.arc` (the architecture team). The traces in `main.arc` link elements of both fragments: they resolve only once the files are merged, which is what makes this a model assembly and not a text include.

## What to look at

- `import` in `main.arc`: paths are relative to the importing file.
- The traces at the end of `main.arc`, between a requirement of one file and a component of the other.
- Each fragment compiles alone, with findings about what lives in the other file: by design.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang check examples/01-getting-started/multifile/main.arc
arclang check examples/01-getting-started/multifile/logical.arc
```

## To understand by changing it

- Check `logical.arc` alone, then `main.arc`: the findings about the missing requirements disappear once the files are assembled.
- Rename a requirement id in `requirements.arc`: `main.arc` now reports the trace that points at nothing.

## Files

- `main.arc`
- `requirements.arc`
- `logical.arc`
