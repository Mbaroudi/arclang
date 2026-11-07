# Semantic git: a model versioned like code, reviewed and merged like a model

A model is text in git, but a line diff of a model says little: moving a block is a hundred changed lines, renaming an element is one, two people adding two functions to the same list conflict. ArcLang compares and merges by **element identity** (the stable UUID the compiler derives from each id), so that:

- a diff shows elements, not lines;
- an element has a history;
- two branches that touch different elements merge without a conflict;
- an auditor (ISO 26262, DO-178C) gets a baseline they can verify.

Everything is computed by the compiler (`arclang`), and works the same from the command line, from CI and from the VS Code workbench.

## `arclang diff`

```text
arclang diff model.arc other.arc              # two files (the original output, unchanged)
arclang diff model.arc --from HEAD~1          # the file as committed, against the working tree
arclang diff model.arc --from main --to v1.0  # two revisions; --to defaults to WORKTREE
arclang diff model.arc --from main -f md      # text (default), json, md, html;  -o file
```

The models are read with `git archive` of the revision, so **imports are resolved at that revision**. The report:

- **per element**: added, removed, renamed (same UUID, new name), moved to another owner, and each changed attribute with its old and new value, grouped by layer (OA, SA, LA, PA, EPBS, data, transverse) and kind;
- **relationships**: exchanges, traces, allocations and deployments added, removed or **retargeted** (one end kept, the other changed);
- **consequences**, from the impact engine's what-if: the production gate before and after (blockers added and removed), requirements newly unsatisfied or unverified, constraints that flip, latency budgets, verification verdicts, coverage counts, and how many elements the change reaches;
- **layout**: changes of `<model>.layout.json` are listed apart and never counted as model changes (`layout_only: true` when nothing else changed).

Exit status follows diff(1): 1 when the **model** differs, 0 otherwise (a layout-only change is 0).

```text
Semantic diff (by stable identity): HEAD~1 -> WORKTREE

System Analysis (SA): +0 -0 ~1
  ~ modified SystemFunction 'AcquireSensorData' [SF-AcquireSensorData]
      latency: "20 ms" -> "80 ms"

Consequences:
  Production gate: PASS (ISO26262) -> FAIL (0 blocker(s) -> 2)
    + blocker: functional_chain 'EmergencyBrakingChain': sum of function latencies 150.0 ms exceeds the 100.0 ms budget
  Budget EmergencyBrakingChain: 90 ms -> 150 ms (limit 100 ms), EXCEEDED
```

`arclang workbench model.arc --rev <revision>` prints the workbench snapshot of the committed model with its layout file (what the VS Code compare mode draws).

## `arclang history <file> <element> [--limit N] [--json]`

The commits (hash, author, date, subject) that changed one element, with what changed in each. They are found by the semantic diff between **successive revisions of the model and of what it imports**, not by line blame: a commit that only reformatted the file, or did not touch the element, is not listed. The element is an id or a unique name; a rename is followed (the identity is the id).

## `arclang merge <base> <ours> <theirs> [-o out]`: the merge driver

A three-way merge by element identity. The files are read as a tree of declarations (the same outline the editor uses); each declaration is matched by `keyword#id` (or its header when it has no id).

- changes to different elements, or to different attributes of one element, merge without a conflict, wherever they are in the file, even on neighboring lines, even two attributes on one line;
- two branches adding different elements to the same container merge in both orders (additions made on both sides are ordered by key, so the result does not depend on who merges);
- an element removed on one side and untouched on the other is removed;
- comments, blank lines, indentation, line endings and the order of the declarations nobody touched are kept as written (the comments above a declaration travel with it);
- a real conflict (the same attribute changed differently, an element changed on one side and removed on the other, the same id added with different content) is written as markers around the **element**, with a header naming it and both values. Each side is a whole element, so the file stays well formed:

```text
<<<<<<< ours function 'FuseData': description: "Fuse radar and camera, ours" (ours) vs description: "Fuse radar and camera, theirs" (theirs)
    function FuseData { ... "ours" ... }
=======
    function FuseData { ... "theirs" ... }
>>>>>>> theirs function 'FuseData'
```

- a clean result is compiled (beside `--path`, the file's place in the repository, so that imports resolve); if it does not compile the driver says why and exits 1;
- `--json` describes the outcome (conflicts with their two sides, notes, compile error). Exit status: 0 clean, 1 conflicts or does not compile.

**Layout files** (`*.layout.json`) are merged by the element each entry places (a box's position, an edge's bends, a port's shift, a fold, a group...): entries of different elements merge; the same entry changed on both sides takes **theirs**, said on standard error.

### Installing it

```text
arclang merge --install            # shows what it changes, asks, then writes
arclang merge --install --yes      # no question
arclang merge --install --dry-run  # shows only
```

It sets `merge.arclang.name` and `merge.arclang.driver` (`<arclang> merge %O %A %B -o %A --path %P`) in the repository's `.git/config` and adds `*.arc merge=arclang` and `*.layout.json merge=arclang` to `.gitattributes`. Nothing is written without confirmation; running it twice changes nothing. In VS Code: *ArcLang: Install the semantic merge driver*.

## Baselines

```text
arclang baseline create <name> [--standard iso26262|do178c|iec61508] [--model file] [--require-pass]
arclang baseline list [--json]
arclang baseline diff <a> <b> [-f text|json|md|html]
arclang baseline verify <name> [--json]
```

`create` checks that the working tree is clean (tracked files; untracked model or layout files; `baselines/` itself is ignored), that the model compiles, runs the production gate, then writes `baselines/<name>/`:

| file | content |
|------|---------|
| `gate_report.json`, `gate_report.md` | the gate verdict, findings, counts |
| `requirements_coverage.csv` | the requirement coverage table |
| `model.sysml`, `model.json` | the SysML v2 text and the model |
| `manifest.json` | below |

and creates the annotated tag `baseline/<name>` on the commit. It never pushes. A baseline is never overwritten. The gate result is recorded whatever it is (`--require-pass` refuses a failing gate). Commit `baselines/<name>/` when you want it with the model; the tag stays valid.

**Manifest**: format version, name, standard, creation time (UTC), compiler version, git commit, branch and tag, the model path, the SHA-256 and size of every source file (the model, what it imports, its layout file), the gate summary, **the identities of the model** (id, UUID, kind, name of every element and relationship, with a SHA-256 of the list), and the SHA-256 and size of every file of the baseline.

`verify` checks: every file against the manifest; that the tag exists, is annotated and points at the manifest's commit; that the manifest is the one the tag was made with (its SHA-256 is in the tag message); that the sources **as committed** still match; and that the model compiled from that commit still has the same identities. Any difference is a failure (exit 1) saying which.

`diff` is the semantic diff between the two commits, with the two gate reports side by side.

## In VS Code

- **Compare ▾** in the workbench tool row (and *ArcLang: Compare the model with a branch, tag, baseline or HEAD…*): HEAD, a quick pick of branches, tags, baselines and remotes, or a baseline. The diagrams are drawn with the changes marked: added green, modified amber, **removed ghosted** (dashed red, struck through) in the box that held them and at their old place when the old layout had one, moved and renamed with a badge (every status has a glyph as well as a color); counters on the layer tabs; a side panel grouped by layer, with the gate, new blockers, unsatisfied and unverified requirements, each change clickable to select the element, with *Source*, *Diff* (VS Code's diff against the revision) and *History*. The comparison is read-only; *Exit* returns to editing.
- **History…** (context menu, Properties pane, Compare ▾): the commits of the element; selecting one draws the model at that revision beside the current one.
- **Create baseline…**: name, standard, the gate result shown first; **Compare with a baseline…**; *Verify a baseline…*.
- **Resolve conflicts** view (Activity bar, ArcLang): the element conflicts left by a merge, by file and element, with *Accept ours*, *Accept theirs* and *Edit*; a message announces them.

## Limits

- Elements are matched by id: an element declared without an `id:` is matched by its declaration line, so renaming it is a removal and an addition (give elements ids).
- Moving an element to another container is, for the merge, a removal and an addition: if the other branch changed it meanwhile, that is a conflict.
- A merge conflict is reported at the element level: attributes of the same element changed differently conflict as a whole element, with both complete versions in the markers.
- Two branches adding the **same id** with different content conflict; with the same content, they agree.
- The merge keeps the text; it does not reformat. `import` lines are ordinary lines (adding different imports in the same place conflicts).
- `arclang diff --from` needs the file to be in a git repository; the models at both revisions must compile (a revision that does not is named and refused).
- `history` compiles one revision per commit: on a long history use `--limit`.
- A baseline of a dirty tree is refused; `--allow-dirty` exists but the baseline then describes files that are not in the tagged commit.
- Layout merge takes theirs on a same-entry conflict by design: a layout never changes what the model means.
