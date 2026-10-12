# Batch execution

`arclang_execute_batch` runs several operations in one call. Each operation
runs a compiler command and returns a reduced result (counts, a verdict, the
first items of a list) instead of the full output, which keeps a large model
out of the assistant's context.

Every figure is read from the compiler's output or from a file on disk. An
operation whose command fails is an error carrying the compiler's message.

## Call

```json
{
  "operations": [
    { "action": "compile_and_validate", "params": { "model_path": "braking.arc" } },
    { "action": "gate", "params": { "model_path": "braking.arc" } },
    { "action": "export_filtered",
      "params": { "model_path": "braking.arc", "filter_type": "components_only" },
      "filter": { "max_items": 5 } }
  ],
  "stop_on_error": true,
  "return_summary_only": false
}
```

| Parameter | Meaning | Default |
|---|---|---|
| `operations` | Operations, executed in order | required |
| `stop_on_error` | Stop at the first operation that fails | `false` |
| `return_summary_only` | Return only the status of each operation | `false` |
| `filter.max_items` (per operation) | Cap of each list returned | `10` |

## Operations

| Action | Parameters | Result |
|---|---|---|
| `compile_and_validate` | `model_path` | `compiles`, compilation warnings (those that block the gate apart), traceability warnings, methodology advisories, `metrics`, coverage |
| `gate` | `model_path`, `standard`? | `verdict` (PASS or FAIL), `blockers` |
| `analyze_traceability` | `model_path` | Coverage, `issues` |
| `extract_metrics` | `model_path` | Element counts, coverage |
| `safety_check` | `model_path`, `standard`? | Counts of declared safety content (not a verdict) |
| `export_filtered` | `model_path`, `filter_type`? | Part of the JSON export: `components_only` (logical components), `requirements_only`, `traces_only`, `all` |
| `compare_models` | `model1_path`, `model2_path` | Semantic diff: counts and changes |
| `generate_all_diagrams` | `model_path`, `output_dir`? | Files written with their size, and why a type was not generated |
| `analyze_diagrams` | `diagram_paths` | Size, dimensions and element counts of SVG files |
| `enhance_diagram` | `diagram_path`, `enhancements` | Writes `<name>_enhanced.svg` only when an enhancement changed something; `improve_contrast` (darkens light fills, text included), `add_grid`, `enlarge_text`. Says which ones changed nothing |

A FAIL of the gate is a result, not an error: the operation succeeded and the
model failed. A model that does not compile is an error.

## Result

```json
{
  "summary": { "total_operations": 3, "executed": 3, "successful": 3, "failed": 0 },
  "results": [
    { "operation": 1, "action": "compile_and_validate", "status": "success",
      "result": { "compiles": true, "compilation_warning_count": 0, "...": "..." } },
    { "operation": 2, "action": "gate", "status": "success",
      "result": { "verdict": "PASS", "blockers": [] } },
    { "operation": 3, "action": "export_filtered", "status": "success",
      "result": { "layer": "logical architecture", "logical_component_count": 2, "components": ["..."] } }
  ]
}
```

A failed operation has `"status": "error"` and an `"error"` field.

## Limits

- `enhance_diagram` rewrites SVG text (sizes, colours); it adds no model content.
