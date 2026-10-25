<!-- arclang-pr-report -->
## ArcLang gate: **FAIL**

1 of 1 model(s) fail the ISO26262 gate; 1 new blocker(s) in this change.

Base `main` (<commit>) -> head `WORKTREE` - ArcLang <version>

| Model | Gate before | Gate after | New blockers | Requirements | Changes |
|---|---|---|---|---|---|
| `model.arc` | PASS | FAIL (1 blocker) | 1 | 2/2 -> 2/2 satisfied, 2/2 -> 2/2 verified | +0 -0 ~1 |

### `model.arc`

Production gate: **PASS** -> **FAIL (1 blocker)**

New blockers:

- [timing.budget] functional_chain 'Reaction': sum of function latencies 230.0 ms exceeds the 100.0 ms budget

Regressions:

- budget Reaction is exceeded (230 ms for 100 ms)

Requirements: 2/2 -> 2/2 satisfied, 2/2 -> 2/2 verified

| Latency budget | Before | After | Limit |
|---|---|---|---|
| `Reaction` | 50 ms | 230 ms (exceeded) | 100 ms |

Impact: 1 element(s) reached by the changes, 1 of them blocker(s) of the gate.

<details><summary>Semantic diff: 0 added, 0 removed, 1 modified (0 renamed, 0 moved); 0 relationship(s) added, 0 removed, 0 changed</summary>

**System Analysis (SA)**: +0 -0 ~1

- modified SystemFunction `SF-SENSE` Sense
  - latency: "20 ms" -> "200 ms"

</details>

| Before | After |
|---|---|
| ![before](diagrams/model-arc-before.svg) | ![after](diagrams/model-arc-after.svg) |

Pictures: see the artifact.

<sub>The verdict is the production gate's (ISO26262); "new" means absent from the base. Exit status with `--fail-on blocker`: non-zero.</sub>
