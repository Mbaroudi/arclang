# A recorded session

The answers below are the ones the server gave, on a copy of the reference model
(`src/arclang_mcp/resources/reference_model.arc`) named `braking.arc`, with
arclang 4.0.1 and no API key set (the workspace path is removed from the output). The questions are what a user might ask; the
tool and arguments are what the assistant calls.

## 1. "Does braking.arc compile?"

`arclang_validate` `{"model_path": "braking.arc"}`

```text
Check of braking.arc: compiles; 0 compilation warning(s), 0 traceability warning(s), 0 advisor(ies)

Checking braking.arc...
✓ No compilation errors

✓ Arcadia methodology: no advisories

Model metrics:
  Total elements: 9
  Requirements: 1
  Components: 5
  Traces: 3
  Traceability: 100.0%
```

## 2. "Is it ready for a design review?"

`arclang_gate` `{"model_path": "braking.arc"}`

```text
Production gate of braking.arc: PASS

Production-readiness gate (ISO26262) — braking.arc
  Requirements: 1 total, 1 satisfied, 1 verified

PRODUCTION GATE: PASS ✓
```

## 3. "Write a requirement: the system shall release the brakes when the driver presses the accelerator. ASIL-B."

`arclang_generate_requirement` `{"description": "The system shall release the brakes when the driver presses the accelerator.", "safety_level": "ASIL-B"}`

````text
Written by the built-in template.

```arc
req "REQ-TSS-001" "The system shall release the brakes when the driver press..." {
  description: "The system shall release the brakes when the driver presses the accelerator."
  priority: "High"
  safety_level: "ASIL-B"
}
```

The compiler accepts it inside `requirements system { }` without a warning.
````

## 4. "Set the latency of LF-002 to 8 ms."

`arclang_set_attribute` `{"model_path": "braking.arc", "element": "LF-002", "key": "latency", "value": "8 ms"}`

```text
Set LF-002.latency: accepted, NOT written (pass write: true to apply)

--- braking.arc
+++ braking.arc (not written)
@@ -50,7 +50,7 @@
             id: "LC-002"
             safety_level: "ASIL-D"
             port in threat { data_type: "ThreatLevel" }
-            function "Compute braking force" { id: "LF-002" latency: 10 ms }
+            function "Compute braking force" { id: "LF-002" latency: 8 ms }
         }
 
         component_exchange ThreatToBrake {
```

Nothing was written: the edit was accepted by the compiler and shown as a diff.

## 5. "Go ahead."

`arclang_set_attribute` `{"model_path": "braking.arc", "element": "LF-002", "key": "latency", "value": "8 ms", "write": true}`

```text
Set LF-002.latency: written to braking.arc

LF-002.latency: 10 ms -> 8 ms
```

## 6. "What would a change to REQ-BRK-001 affect?"

`arclang_impact` `{"model_path": "braking.arc", "element": "REQ-BRK-001"}`

```text
Impact of 'REQ-BRK-001' in braking.arc: succeeded

Impact analysis for 'REQ-BRK-001':

  Distance 1 (directly affected):
    Component 'BrakeController' [LC-002] — via is satisfied by (trace) from 'REQ-BRK-001'
    Component 'ThreatAssessor' [LC-001] — via is satisfied by (trace) from 'REQ-BRK-001'

  Distance 2 (transitively affected):
    Function 'Assess threat' [LF-001] — via performs (allocation) from 'LC-001'
    Function 'Compute braking force' [LF-002] — via performs (allocation) from 'LC-002'

  Distance 3 (transitively affected):
    SystemFunction 'AssessThreat' [SF-001] — via realizes (trace) from 'LF-001'

  Total affected elements: 5
```

## 7. "Are the traces plausible?"

`arclang_review` `{"model_path": "braking.arc"}`

```text
Trace review of braking.arc: dry run, nothing was sent

3 declared trace(s) and 0 undeclared pair(s) to judge: 3 question(s), one request each.
Sent to api.typesafe.ai for each: the kind, name, id and text attributes of the two elements, and the trace's rationale.
Nothing was sent (--dry-run).
First request:
[...]
```

Without `send: true` this is a dry run. The request shown after these lines is what would go to api.typesafe.ai; it is cut here.
