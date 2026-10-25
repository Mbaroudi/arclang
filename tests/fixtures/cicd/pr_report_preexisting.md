<!-- arclang-pr-report -->
## ArcLang gate: **FAIL**

1 of 1 model(s) fail the ISO26262 gate; 0 new blocker(s) in this change.

Base `main` (<commit>) -> head `WORKTREE` - ArcLang <version>

| Model | Gate before | Gate after | New blockers | Requirements | Changes |
|---|---|---|---|---|---|
| `model.arc` | FAIL (1 blocker) | FAIL (1 blocker) | 0 | 2/2 -> 2/2 satisfied, 2/2 -> 2/2 verified | +0 -0 ~1 |

### `model.arc`

Production gate: **FAIL (1 blocker)** -> **FAIL (1 blocker)**

Blockers already there at the base (not introduced by this change):

- [timing.budget] functional_chain 'Reaction': sum of function latencies 230.0 ms exceeds the 100.0 ms budget

Requirements: 2/2 -> 2/2 satisfied, 2/2 -> 2/2 verified

Impact: 5 element(s) reached by the changes, 0 of them blocker(s) of the gate.

<details><summary>Semantic diff: 0 added, 0 removed, 1 modified (0 renamed, 0 moved); 0 relationship(s) added, 0 removed, 0 changed</summary>

**Logical Architecture (LA)**: +0 -0 ~1

- modified LogicalComponent `LC-001` Controller
  - description: "Computes the plan" -> "Computes the plan, better"

</details>

<sub>The verdict is the production gate's (ISO26262); "new" means absent from the base. Exit status with `--fail-on blocker`: zero.</sub>
