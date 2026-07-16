# ArcLang MCP Batch Execution - "Code Mode"

## Overview

The `arclang_execute_batch` tool enables **efficient context usage** by executing complex multi-step workflows on the MCP server side, filtering data **before** sending to the LLM context.

### Key Benefits

- **10-100x Token Reduction**: Load large models, filter to only errors/gaps/differences
- **Atomic Operations**: Batch compile → validate → generate diagrams in one call
- **Pre-filtering**: Extract only metrics, not entire JSON models
- **Single-Step Execution**: Complex logic runs server-side like "code mode"

## How It Works

```
Traditional Approach (wasteful):
1. Agent calls arclang_compile → 50KB output sent to context
2. Agent calls arclang_validate → 30KB output sent to context  
3. Agent calls arclang_generate_all → 200KB output sent to context
Total: 280KB (~70,000 tokens)

Batch Execution (efficient):
1. Agent calls arclang_execute_batch with operations array
2. Server executes all 3 operations
3. Server filters results to only errors + summary
4. Returns: { error_count: 2, errors: [...], diagram_count: 10 }
Total: 1KB (~250 tokens) = 280x reduction
```

## Available Operations

### 1. `compile_and_validate`
Compile and validate model, return **only errors** (not full output).

```json
{
  "action": "compile_and_validate",
  "params": {
    "model_path": "path/to/model.arc"
  },
  "filter": {
    "only_errors": true,
    "max_items": 5
  }
}
```

**Returns**: `{ status: "success", error_count: 0, warning_count: 2 }`

### 2. `generate_all_diagrams`
Generate all 10 diagram types, return **summary** (not 200KB SVG content).

```json
{
  "action": "generate_all_diagrams",
  "params": {
    "model_path": "path/to/model.arc",
    "output_dir": "./diagrams"
  },
  "filter": {
    "summary_only": true
  }
}
```

**Returns**: `{ diagram_count: 10, total_size_kb: 127, diagrams: [{name, size_kb}] }`

### 3. `analyze_traceability`
Find untraced requirements/components, return **only gaps** (not full matrix).

```json
{
  "action": "analyze_traceability",
  "params": {
    "model_path": "path/to/model.arc"
  },
  "filter": {
    "gaps_only": true,
    "max_items": 10
  }
}
```

**Returns**: `{ gap_count: 5, gaps: ["REQ-001 untraced", ...] }`

### 4. `safety_check`
Validate safety standards, return **only violations** (not full report).

```json
{
  "action": "safety_check",
  "params": {
    "model_path": "path/to/model.arc",
    "standard": "iso26262"
  },
  "filter": {
    "violations_only": true,
    "max_items": 10
  }
}
```

**Returns**: `{ status: "pass", violation_count: 0 }`

### 5. `extract_metrics`
Get model statistics **without** loading entire JSON.

```json
{
  "action": "extract_metrics",
  "params": {
    "model_path": "path/to/model.arc"
  }
}
```

**Returns**: `{ component_count: 42, requirement_count: 128, interface_count: 67 }`

### 6. `export_filtered`
Export model to JSON but filter to **specific sections** only.

```json
{
  "action": "export_filtered",
  "params": {
    "model_path": "path/to/model.arc",
    "filter_type": "components_only"
  },
  "filter": {
    "max_items": 20
  }
}
```

**Returns**: `{ components: [{id, name, type}, ...] }` (20 items max)

### 7. `compare_models`
Diff two models, return **only differences**.

```json
{
  "action": "compare_models",
  "params": {
    "model1_path": "v1/model.arc",
    "model2_path": "v2/model.arc"
  }
}
```

**Returns**: `{ differences: "3 components added, 2 modified", summary: "..." }`

## Complete Example

### Use Case: "Validate model and generate diagrams if valid"

**Single Batch Call:**
```json
{
  "operations": [
    {
      "action": "compile_and_validate",
      "params": {"model_path": "system.arc"},
      "filter": {"only_errors": true}
    },
    {
      "action": "generate_all_diagrams",
      "params": {"model_path": "system.arc", "output_dir": "./output"},
      "filter": {"summary_only": true}
    },
    {
      "action": "analyze_traceability",
      "params": {"model_path": "system.arc"},
      "filter": {"gaps_only": true, "max_items": 5}
    }
  ],
  "stop_on_error": true,
  "return_summary_only": false
}
```

**Response (filtered):**
```json
{
  "results": [
    {
      "operation": 1,
      "action": "compile_and_validate",
      "status": "success",
      "result": {
        "status": "success",
        "error_count": 0,
        "warning_count": 3,
        "model_path": "system.arc"
      }
    },
    {
      "operation": 2,
      "action": "generate_all_diagrams",
      "status": "success",
      "result": {
        "status": "success",
        "diagram_count": 10,
        "total_size_kb": 127,
        "diagrams": [
          {"name": "operational.svg", "size_kb": 12},
          {"name": "component.svg", "size_kb": 15},
          ...
        ]
      }
    },
    {
      "operation": 3,
      "action": "analyze_traceability",
      "status": "success",
      "result": {
        "status": "success",
        "gap_count": 2,
        "gaps": [
          "REQ-005 untraced to components",
          "Component LC-012 has no requirement trace"
        ]
      }
    }
  ],
  "tokens_saved_estimate": 68500
}
```

**Token Usage Comparison:**
- Traditional: ~75,000 tokens (3 separate calls with full outputs)
- Batch: ~500 tokens (filtered results only)
- **Savings: 150x reduction**

## Parameters

### Global Parameters

- `operations` (array, required): List of operations to execute
- `stop_on_error` (boolean, default: false): Stop if any operation fails
- `return_summary_only` (boolean, default: false): Return only high-level summary

### Per-Operation Parameters

Each operation has:
- `action`: Operation type (see list above)
- `params`: Operation-specific parameters
- `filter`: Optional filtering configuration

### Common Filter Options

- `only_errors` (boolean): Return only errors, not full output
- `gaps_only` (boolean): Return only traceability gaps
- `violations_only` (boolean): Return only safety violations
- `summary_only` (boolean): Return summary stats, not full content
- `max_items` (integer): Limit array results to N items

## Integration with Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "arclang": {
      "command": "python",
      "args": ["-m", "arclang_mcp"],
      "env": {
        "ARCLANG_WORKSPACE": "/path/to/arclang/workspace"
      }
    }
  }
}
```

## Example Prompts

**Efficient Model Validation:**
```
"Use arclang_execute_batch to compile and validate emergency_braking.arc, 
returning only errors if any exist."
```

**Complete Workflow:**
```
"Batch execute: 1) validate model.arc, 2) if valid generate all diagrams, 
3) check traceability gaps. Return filtered summary only."
```

**Safety Audit:**
```
"Run safety check on automotive_system.arc per ISO 26262, 
return only violations and gap count."
```

## Token Efficiency Examples

| Task | Traditional | Batch Filtered | Savings |
|------|-------------|----------------|---------|
| Compile large model | 50,000 | 200 | 250x |
| Generate all diagrams | 200,000 | 1,000 | 200x |
| Traceability analysis | 80,000 | 500 | 160x |
| Safety validation | 100,000 | 300 | 333x |
| Full workflow (all 4) | 430,000 | 2,000 | **215x** |

## Benefits

1. **Context Efficiency**: Use 99% less context for large models
2. **Faster Execution**: Single batch call vs. multiple sequential calls
3. **Atomic Operations**: All-or-nothing execution with `stop_on_error`
4. **Pre-processing**: Complex filtering/aggregation runs server-side
5. **Scalability**: Handle 1000+ component models efficiently

## Implementation Notes

- Server-side filtering prevents large data from reaching LLM context
- Results are JSON-structured for easy parsing
- `tokens_saved_estimate` tracks efficiency gains
- Each operation is independent (parallel execution possible in future)
- Errors in one operation don't affect others (unless `stop_on_error: true`)

---

**Next Steps**: Enable this in Claude Desktop by configuring MCP server, then use batch execution for all multi-step ArcLang workflows.
