# Change-impact analysis

`arclang impact` answers "what does this change reach, and why?" from the
compiled model, never from names or guesses. It is deterministic: the same
model and seeds give the same report, byte for byte.

```text
arclang impact model.arc --element LC-BATT [--element ...] [--depth 4] [--kinds satisfies,verifies]
                         [--direction both|downstream|upstream] [--gate-standard ISO26262]
arclang impact model.arc --change '{"op":"set","element":"SF-A","key":"latency","value":"80 ms"}'
arclang impact model.arc --delete LC-LOG
arclang impact model.arc --since HEAD            # or main, a tag, a commit
        -f text|json|md|html|tsv|xlsx   [-o report.xlsx]
```

`arclang impact model.arc REQ-1` keeps printing what it always did.

## What is followed

Fifteen kinds of link, each typed and oriented (`--kinds`): realization
(`realizes`, `implements`, `refines`, `allocated_component`), allocation,
deployment, exchange (functional, component, operational, physical, links),
interface (ports), chain (functional chains, capabilities), satisfies,
verifies (test cases, `asserts`), safety (hazards, mitigations, threats),
containment, typing (`is:` users of a type), constraint (a constraint that
reads an attribute), view (declared diagrams and tables), behavior
(scenario messages, transitions), reference.

Three rules keep the set to what a change carries: a link is never walked
straight back (a component reaches its requirement, not every other
component satisfying it); what the changed element *depends on* is to
re-check, not changed, so only its evidence follows (tests, hazards, chains,
constraints, views); the far end of an exchange is reached and the walk
stops there.

Each impacted element carries its **distance**, the **shortest path** that
explains it (`LC-BATT --satisfies--> REQ-R --verified by--> TC-R`), a
**category** (requirement, test, safety, interface, budget, view, type user,
architecture) and a **severity**:

| Severity | When |
|---|---|
| blocker | a gate blocker names it; a requirement, hazard or budget carrying an ASIL/DAL/SIL; in a what-if, a requirement newly unsatisfied, a constraint or latency budget newly broken |
| major | an interface (port, exchange, link); a test to redo; a requirement, hazard or budget to re-check; a type user; an integrity-level element |
| minor | a view, any other element to review |

## What-if

`--change` (one `arclang edit` request or an array), `--delete` (the element
with the traces and list entries naming it) apply the edits to a copy of the
text, compile it, and compare the two models: the gate before and after (new
and cleared blockers), constraints that flip, latency budgets (sum of the
chain against its bound), requirement coverage, verdicts left stale (a
verdict stated by a run elsewhere, `verdict:`, that the change reaches),
elements added, removed or changed. An edit that does not compile is
refused with the compiler's reason (`success: false`, exit code 1); nothing
is written. `--since REV` does the same between the committed model and the
file (`--current FILE` for an unsaved buffer).

## Reports

`-f md|html|xlsx` write a change-request report: summary, impact (blockers
first), gate, constraints, budgets, coverage, verdicts, changes. The Excel
workbook uses the tables' own writer (frozen, filterable header, status
colors).

## In VS Code

*Impact analysis* opens in its own editor tab, live (debounced, shown stale
while recomputing): from the diagram (context menu, floating toolbar), the
model tree, the text (code action "Analyze impact of …"), the Properties
pane, or the palette (`Impact analysis: analyze an element…`, `what if…`,
`my changes`). The tab shows the graph (rings by distance, severity colors),
the table with the clickable *why* path, filters (severity, layer, category,
depth, direction, kinds of link), the what-if panel with **Apply the change**
(one undoable edit) or **Discard**, the exports, *Copy as table*, and *Ask AI*
when a provider is configured. The impacted elements are outlined on the
workbench diagrams by severity, with a count on each layer tab.
