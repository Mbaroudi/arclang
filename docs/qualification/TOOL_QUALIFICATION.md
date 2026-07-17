# ArcLang Tool Qualification Support Kit

**Scope:** `arclang` compiler and production gate (`arclang gate`)
**Standards addressed:** ISO 26262-8:2018 §11 (automotive), DO-330 (aerospace, as tool supplement to DO-178C)
**Status:** qualification *support kit* — the classification below is an
engineering analysis provided by the tool developer. The final Tool
Confidence Level / Tool Qualification Level determination and the
qualification itself are the responsibility of the user organization's
safety process, using the evidence inventoried here.

---

## 1. Tool identification

| Item | Value |
|---|---|
| Tool name | `arclang` |
| Functions in scope | model compilation (parse + semantic analysis), traceability resolution, production gate (`arclang gate`) |
| Out of scope | diagram rendering, HTML explorer, Terraform/K8s generators, Simulink/FMI/ReqIF/SysML exporters (generation aids, outputs independently reviewed in the receiving tool) |
| Version identification | Git tag + commit SHA; release binaries ship with `SHA256SUMS` (see `docs/VERSIONING.md`) |
| Execution environment | offline CLI, no network access, deterministic outputs (same input → byte-identical output) |

## 2. Use cases

- **UC-1 — Model verification:** `arclang build` / `arclang check` detect
  malformed models, dangling traces, duplicate identities, methodology
  violations.
- **UC-2 — Requirements status:** the gate checks every requirement is
  satisfied by a trace and verified by a `test_case`.
- **UC-3 — HARA consistency:** the gate recomputes ASIL from declared
  S/E/C (ISO 26262-3 table 4) and flags contradictions with the declared
  ASIL; DO-178C failure conditions map to DAL.
- **UC-4 — Timing budgets:** the gate sums declared function latencies
  along a functional chain and compares against the declared budget.
- **UC-5 — Change control:** `arclang diff` reports semantic changes
  between two model versions by stable identity.

## 3. Tool classification — ISO 26262-8 §11

**Tool Impact (TI):** the tool does not generate embedded code in scope.
Its outputs are *verification verdicts*. A malfunction cannot *introduce*
an error into the item, but it **can fail to detect** one (e.g. wrongly
report a requirement as verified). → **TI2**.

**Tool error Detection (TD):** confidence that a tool malfunction is
detected by the process around it. Factors favouring detection:

- the gate's inputs and outputs are human-readable text reviewed in the
  same pull request (the verdict lists *which* trace/test satisfied each
  requirement — a reviewer can spot a wrong claim);
- deterministic outputs: any anomaly reproduces identically;
- the compiler fails loudly (unknown constructs are localized errors,
  never silent skips — see regression test
  `test_compile_rejects_unknown_tokens_with_location`).

If the user process includes an independent review of gate verdicts
against the model, **TD2** is defensible → **TCL2**. Without such review,
assume **TD3** → **TCL3**.

**Recommended qualification method (for TCL2/TCL3, ASIL C–D):**
*validation of the software tool* (ISO 26262-8, 11.4.9), using the test
inventory of §6 as the validation suite, executed on the pinned release
in the user environment.

## 4. Tool classification — DO-330 / DO-178C

The tool is a **verification tool**: it cannot insert an error into
airborne software, but its output could fail to detect an error, and it
is used to justify verification activities (requirements verification
status). → **Criteria 3** → **TQL-5** (typical); **Criteria 2 → TQL-4**
if used to justify *elimination* of other verification activities —
avoid that usage or qualify accordingly.

## 5. Tool functions and potential malfunctions

| ID | Function | Potential malfunction | Detection / mitigation |
|---|---|---|---|
| TF-1 | Strict parsing | construct silently ignored → model incomplete | unknown tokens are hard errors with line/column; skipped-but-known blocks emit loud warnings; warnings must be reviewed (§7) |
| TF-2 | Identity registry | duplicate or unstable identity → wrong traceability | UUIDv5 deterministic (golden-value tests); duplicate ids produce warnings |
| TF-3 | Trace resolution | dangling trace accepted → false coverage | dangling references are compile ERRORS, not warnings |
| TF-4 | Requirement satisfaction check | requirement wrongly marked satisfied | verdict names the satisfying trace; reviewable in PR |
| TF-5 | Requirement verification check | requirement wrongly marked verified | verdict names the verifying `test_case` and its method |
| TF-6 | ASIL computation (ISO 26262-3 table 4) | wrong ASIL derived from S/E/C | full table encoded as data; exhaustive unit test against the published table |
| TF-7 | DAL mapping (DO-178C) | wrong DAL for a failure condition | unit-tested mapping |
| TF-8 | Timing budget summation | wrong latency sum or unit conversion | `parse_millis` unit-tested (ms/us/s); blown budget is a blocker |
| TF-9 | Gate verdict aggregation | PASS despite blockers | exit code asserted in tests; blockers listed individually in output |
| TF-10 | Semantic diff | change not reported | identity-based comparison unit-tested (rename, reorder, trace add) |

## 6. Verification evidence — traceability matrix

All tests run in CI on every commit (`.github/workflows/ci.yml`), on
Linux and macOS. Test names are stable identifiers into the repository.

| Function | Evidence (test) | Location |
|---|---|---|
| TF-1 | `test_compile_rejects_unknown_tokens_with_location` | `tests/integration_tests.rs` |
| TF-2 | `same_element_same_uuid`, `different_kind_different_uuid`, `different_id_different_uuid`, `uuid_is_stable_across_versions` (golden value cross-checked against an independent Python implementation) | `src/compiler/identity.rs` |
| TF-2 | `test_elements_have_stable_deterministic_uuids`, `test_duplicate_element_id_produces_warning` | `tests/integration_tests.rs` |
| TF-3 | `test_dangling_trace_is_a_compile_error`, `test_trace_by_name_is_normalized_to_id`, `test_dangling_capability_reference_is_an_error` | `tests/integration_tests.rs` |
| TF-4, TF-5 | `test_production_gate_fails_on_unverified_requirements` | `tests/integration_tests.rs` |
| TF-6 | `asil_matrix_matches_iso26262_table4` (all S×E×C combinations), `test_production_gate_catches_asil_mismatch` | `src/compiler/production_gate.rs`, `tests/integration_tests.rs` |
| TF-7 | `dal_mapping_matches_do178c` | `src/compiler/production_gate.rs` |
| TF-8 | `millis_parser_handles_common_units`, `test_production_gate_flags_blown_timing_budget` | `src/compiler/production_gate.rs`, `tests/integration_tests.rs` |
| TF-9 | `test_production_gate_passes_on_complete_flagship` (positive case; negative cases above assert FAIL + blocker listing) | `tests/integration_tests.rs` |
| TF-10 | `reordering_blocks_is_an_empty_diff`, `rename_is_a_modification_not_remove_add`, `added_requirement_and_trace_are_reported`, `description_change_is_field_level` | `src/compiler/semantic_diff.rs` |
| End-to-end | golden corpus: every non-legacy example must compile (`tests/examples_compile.rs`); flagship element-extraction guard | `tests/examples_compile.rs` |

## 7. Conditions and constraints of use

1. **Pin the release.** Use a tagged release binary and verify its
   `SHA256SUMS` entry before first use. Requalify on tool update.
2. **Review warnings.** Compiler warnings are part of the verdict: a
   model that compiles with warnings has known-unmodeled content. The
   gate treats methodology lints as findings; do not suppress them.
3. **Gate on exit codes** in CI (`arclang gate` returns non-zero on
   FAIL) — do not parse prose output.
4. **Independent review of verdicts** (recommended, supports TD2): a
   reviewer confirms, per released requirement, that the satisfying
   trace and verifying test case named by the gate are the intended ones.
5. **Do not extend tool credit** beyond the functions of §5. In
   particular, exporters (Simulink, FMI, ReqIF, SysML) are exchange
   aids; their outputs are verified in the receiving tool.

## 8. Known limitations

- The gate checks *declared* latencies against *declared* budgets; it
  does not measure execution.
- ASIL decomposition (ISO 26262-9) is not computed; only table-4
  derivation from S/E/C is.
- The gate's ICD completeness check verifies declared metadata
  (via/frequency/message_type/protocol), not electrical or protocol
  correctness.
- Multi-file merge relies on the duplicate-identity warnings for
  cross-file collisions; review them (§7.2).
