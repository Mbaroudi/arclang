# A recorded session

The answers below are the ones the server gave, on a copy of the reference model
(`src/arclang_mcp/resources/reference_model.arc`) named `braking.arc`, with the
compiler built from this repository (`arclang 4.0.1`) and no API key set (the
workspace path is removed from the output). The questions are what a user might
ask; the tool and arguments are what the assistant calls.

## 1. "Does braking.arc compile?"

`arclang_validate` `{"model_path": "braking.arc"}`

```text
Check of braking.arc: compiles; 0 compilation warning(s), 0 traceability warning(s), 0 advisor(ies)

Checking braking.arc...
✓ No compilation errors

Constraints:
  ✓ Reaction time — "LF-001".latency + "LF-002".latency <= 50 ms  [30 ms vs 50 ms]

✓ Arcadia methodology: no advisories

Model metrics:
  Total elements: 10
  Requirements: 1
  Components: 6
  Traces: 4
  Traceability: 100.0%
```

## 2. "Is it ready for a design review?"

`arclang_gate` `{"model_path": "braking.arc"}`

```text
Production gate of braking.arc: PASS

Production-readiness gate (ISO26262) — braking.arc
  Requirements: 1 total, 1 satisfied, 1 verified
  Constraints: 1 declared, 1 hold

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
@@ -63,7 +63,7 @@
             is: "Periodic task"
             safety_level: "ASIL-D"
             port in threat { id: "LP-002" data_type: "ThreatLevel" }
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
    SystemFunction 'CommandBrakes' [SF-002] — via realizes (trace) from 'LF-002'

  Total affected elements: 6
```

## 7. "Are the traces plausible?"

`arclang_review` `{"model_path": "braking.arc"}`

```text
Trace review of braking.arc: dry run, nothing was sent

4 declared trace(s) and 0 undeclared pair(s) to judge: 4 question(s), one request each.
Sent to api.typesafe.ai for each: the kind, name, id and text attributes of the two elements, and the trace's rationale.
Nothing was sent (--dry-run).
First request:
[...]
```

Without `send: true` this is a dry run. The request shown after these lines is what would go to api.typesafe.ai; it is cut here.

## 8. "What is left to declare on the next Arcadia level?"

`arclang_transition` `{"model_path": "braking.arc"}`

```text
Transition plan of braking.arc: oa-to-sa: 1 candidate(s), 1 not realized; sa-to-la: 2 candidate(s), 0 not realized; la-to-pa: 2 candidate(s), 2 not realized

[
  {
    "step": "oa-to-sa",
    "label": "Operational activities to system functions",
    "candidates": [
      {
        "id": "OA-MonitorEnvironment",
        "name": "MonitorEnvironment",
        "kind": "operational_activity",
        "realized_by": []
      }
    ],
    "blocked": null
  },
  {
    "step": "sa-to-la",
    "label": "System functions to logical functions",
    "candidates": [
      {
        "id": "SF-001",
        "name": "AssessThreat",
        "kind": "function",
        "realized_by": [
          "LF-001"
        ]
      },
      {
        "id": "SF-002",
        "name": "CommandBrakes",
        "kind": "function",
        "realized_by": [
          "LF-002"
        ]
      }
    ],
    "blocked": null
  },
  {
    "step": "la-to-pa",
    "label": "Logical components to physical components",
    "candidates": [
      {
        "id": "LC-001",
        "name": "ThreatAssessor",
        "kind": "component",
        "realized_by": []
      },
      {
        "id": "LC-002",
        "name": "BrakeController",
        "kind": "component",
        "realized_by": []
      }
    ],
    "blocked": null
  }
]
```

The first line is read from the compiler's JSON; the plan changes nothing.

## 9. "Declare the system function for the operational activity."

`arclang_transition` `{"model_path": "braking.arc", "step": "oa-to-sa"}`

```text
Transition oa-to-sa of braking.arc: accepted, NOT written (pass write: true to apply)

--- braking.arc
+++ braking.arc (not written)
@@ -46,6 +46,9 @@
             from: "SF-001.threat_level"
             to: "SF-002.threat_level"
             exchange_item: "ThreatAssessed"
+        }
+        function "MonitorEnvironment" {
+            id: "SF-MonitorEnvironment"
         }
     }
 
@@ -188,3 +191,4 @@
 trace "LC-001" satisfies "REQ-BRK-001" { rationale: "Detects the imminent collision" }
 trace "LF-001" realizes "SF-001" { rationale: "Logical realisation of the threat assessment" }
 trace "LF-002" realizes "SF-002" { rationale: "Logical realisation of the braking command" }
+trace "SF-MonitorEnvironment" realizes "OA-MonitorEnvironment" {}


The compiler declared:
+ function 'MonitorEnvironment' (SF-MonitorEnvironment) realizes OA-MonitorEnvironment in system_analysis
```

As with an edit, the model is written only when the call says `write: true`.

## 10. "Export it to SysML v2."

`arclang_export` `{"model_path": "braking.arc", "format": "sys-ml", "output_path": "braking.sysml"}`

```text
Export of braking.arc to sys-ml: succeeded

Exporting braking.arc to SysML format...
✓ Export successful
  Input: braking.arc
  Output: braking.sysml
  Format: SysML

Written to braking.sysml:

// Generated by ArcLang: SysML v2 in the Arcadia profile (spec/SYSML_PROFILE.md)
package EmergencyBraking {
    private import ScalarValues::*;
    [...]
}
```

The whole file is returned when it is short; it is cut here.

## 11. "Read that SysML file back into ArcLang."

`arclang_import` `{"input_path": "braking.sysml", "format": "sysml", "output_path": "imported/braking.arc"}`

```text
Import of braking.sysml (sysml) to braking.arc: succeeded

Importing from SysML: braking.sysml...
  1 activities
  1 classes
  1 component exchanges
  1 constraints
  1 data types
  2 deployments
  1 enumerations
  2 exchange items
  1 functional exchanges
  4 functions
  1 hazards
  2 logical components
  1 messages
  7 names designating nothing known (left unresolved)
  1 operational actors
  1 operational entities
  1 operational interactions
  1 physical links
  2 physical nodes
  1 physical paths
  5 ports
  1 references by dependency
  1 requirements
  1 scenarios
  1 state machines
  3 states
  1 test cases
  4 traces
  3 transitions
  1 transitions to states not written (left out)
  3 types
  note: import 'ScalarValues' designates nothing known
  note: import 'MeasurementReferences' designates nothing known
  note: import 'ISQ' designates nothing known
  note: import 'SI' designates nothing known
  note: import 'MeasurementReferences' designates nothing known
  note: import 'ISQ' designates nothing known
  note: import 'SI' designates nothing known
✓ Import successful
  Input: braking.sysml
  Output: imported/braking.arc

The written model (imported/braking.arc) compiles; 0 compilation warning(s), 0 traceability warning(s), 0 advisor(ies).
```

The counts, what was left out and what was left unresolved are the compiler's
words; the last line is `arclang check --lint` run on the file it wrote.

## 12. "Any finding on the imported model?"

`arclang_validate` `{"model_path": "imported/braking.arc", "as_json": true}`

```text
Check of braking.arc: compiles; 0 finding(s) that block the gate

{
  "errors": [],
  "file": "imported/braking.arc",
  "findings": [],
  "metrics": {
    "components": 6,
    "functions": 3,
    "requirements": 1,
    "traces": 4
  },
  "success": true
}
```

A finding would carry its `rule` (the id `arclang_rules` explains), its
`severity` (`error` blocks the gate), the `element` and the `fix` the compiler
suggests.
