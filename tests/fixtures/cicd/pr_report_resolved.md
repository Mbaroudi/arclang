<!-- arclang-pr-report -->
## ArcLang gate: **PASS**

All 1 model(s) compile and pass the ISO26262 gate.

Base `main` (<commit>) -> head `WORKTREE` - ArcLang <version>

| Model | Gate before | Gate after | New blockers | Requirements | Changes |
|---|---|---|---|---|---|
| `model.arc` | FAIL (1 blocker) | PASS | 0 | 2/2 -> 2/2 satisfied, 2/2 -> 2/2 verified | +0 -0 ~1 |

### `model.arc`

Production gate: **FAIL (1 blocker)** -> **PASS**

Resolved by this change:

- [timing.budget] functional_chain 'Reaction': sum of function latencies 230.0 ms exceeds the 100.0 ms budget

Requirements: 2/2 -> 2/2 satisfied, 2/2 -> 2/2 verified

| Latency budget | Before | After | Limit |
|---|---|---|---|
| `Reaction` | 230 ms | 50 ms | 100 ms |

Impact: 1 element(s) reached by the changes, 0 of them blocker(s) of the gate.

<details><summary>Semantic diff: 0 added, 0 removed, 1 modified (0 renamed, 0 moved); 0 relationship(s) added, 0 removed, 0 changed</summary>

**System Analysis (SA)**: +0 -0 ~1

- modified SystemFunction `SF-SENSE` Sense
  - latency: "200 ms" -> "20 ms"

</details>

<sub>The verdict is the production gate's (ISO26262); "new" means absent from the base. Exit status with `--fail-on blocker`: zero.</sub>
