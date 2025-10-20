# Example MCP Session with Claude

This document shows example interactions with Claude using the ArcLang MCP Server.

## Session 1: Create a New ACC System

**User**: I need to create an adaptive cruise control system. Can you help me start?

**Claude**: I'll help you create an ACC system model. Let me start by generating the main requirement.

[Uses `arclang_generate_requirement`]

**Generated Requirement**:
```arc
requirement "REQ-ACC-001" {
    description: "System shall maintain safe 2-second following distance from lead vehicle"
    priority: "Critical"
    safety_level: "ASIL_B"
    type: "Functional"
    verification_method: "Test"
}
```

**User**: Great! Now can you suggest an architecture for this?

**Claude**: [Uses `arclang_suggest_architecture`]

Based on your ACC requirement, I suggest these components:

1. **Radar Sensor** (Logical)
   - Measures distance and relative speed
   - Safety: ASIL-B

2. **ACC Controller** (Logical)
   - Calculates target speed
   - Implements control algorithm
   - Safety: ASIL-B

3. **Throttle Actuator** (Logical)
   - Controls vehicle speed
   - Safety: ASIL-B

4. **Safety Monitor** (Logical)
   - Independent monitoring
   - Safety: ASIL-B

**User**: Can you generate the Radar Sensor component?

**Claude**: [Uses `arclang_generate_component`]

```arc
component "Radar Sensor" {
    id: "LC-SENS-RADAR"
    type: "Logical"
    description: "77GHz radar for distance and speed measurement"
    safety_level: "ASIL_B"
    
    function "Measure Distance" {
        id: "LF-MEASURE-DIST"
        outputs: ["distance_m", "relative_speed_ms"]
        execution_time: "10ms"
    }
    
    function "Self-Diagnostic" {
        id: "LF-DIAG"
        outputs: ["sensor_status"]
        execution_time: "50ms"
    }
}
```

---

## Session 2: Validate Existing Model

**User**: Can you check if my model is valid?

**Claude**: [Uses `arclang_validate`]

✅ Validation passed

**Model**: acc_system.arc  
**Syntax**: Valid  
**Semantics**: Valid  

---

## Session 3: Traceability Analysis

**User**: Check my traceability coverage

**Claude**: [Uses `arclang_trace_analysis`]

🔗 **Traceability Analysis**

**Model**: acc_system.arc  
**Coverage**: 85%  
**Total Traces**: 12

⚠️  **Gaps Found** (3):

**Untraced Requirements** (2):
  - REQ-ACC-005: System shall handle sensor failure
  - REQ-ACC-007: System shall deactivate on brake press

**Untraced Components** (1):
  - LC-SAFETY-MON: Independent Safety Monitor

⚠️  Moderate traceability coverage - consider adding more traces

**User**: Can you help me create traces for those gaps?

**Claude**: I'll create the missing traces:

```arc
trace "LC-SAFETY-MON" satisfies "REQ-ACC-005" {
    rationale: "Safety monitor detects sensor failures and triggers safe state"
}

trace "LC-CTRL-BRAKE" satisfies "REQ-ACC-007" {
    rationale: "Brake controller immediately deactivates ACC when brake pedal pressed"
}
```

---

## Session 4: Safety Compliance Check

**User**: Validate this model against ISO 26262

**Claude**: [Uses `arclang_safety_check`]

🛡️  **Safety Compliance Check**

**Standard**: ISO26262  
**Model**: acc_system.arc

✅ **Status**: Compliant

**Issues Found** (2):

⚠️  **Missing Rationale**
   Some ASIL-B requirements lack detailed rationale

⚠️  **Incomplete FMEA**
   Component LC-SENS-RADAR missing failure modes

**Recommendations**:
  - Add detailed rationale for all ASIL-B/C requirements
  - Complete FMEA analysis for all safety-critical components
  - Add independent verification for ASIL-C components

---

## Session 5: Export Diagram

**User**: Generate an architecture diagram

**Claude**: [Uses `arclang_export_diagram`]

✅ Diagram generated successfully

**Model**: acc_system.arc  
**Format**: arc-viz-ultimate  
**Output**: acc_system.html

**Components**: 9  
**Connections**: 15

---

## Session 6: Git Merge Assistance

**User**: I have a merge conflict in my model. Can you help?

**Claude**: [Uses `arclang_git_merge`]

🔀 **Semantic Merge Analysis**

⚠️  **Conflicts detected**: 2

**REQ-ACC-001**:
  - Type: Modified in both branches
  - Ours: Changed priority from High to Critical
  - Theirs: Added safety_level ASIL_B
  - Suggestion: Keep Critical priority and add ASIL_B (combine both changes)

**LC-CTRL-SPEED**:
  - Type: Modified in both branches  
  - Ours: Changed execution_time from 20ms to 15ms
  - Theirs: Added new function "Monitor Health"
  - Suggestion: Keep 15ms execution time and add new function (combine both)

**User**: Thanks! I'll apply those suggestions.

---

## Session 7: Model Metrics

**User**: Show me statistics for my model

**Claude**: [Uses `arclang_info`]

📊 **Model Information**

**Path**: models/acc_system.arc  
**Size**: 15.2 KB

**Metrics**:
  - Requirements: 12
  - Components: 9
  - Functions: 18
  - Traces: 15
  - Actors: 2
  - Nodes: 3

**Safety Metrics**:
  - ASIL-D: 0
  - ASIL-C: 3
  - ASIL-B: 8
  - ASIL-A: 1

**Traceability Coverage**: 95%

---

These examples demonstrate how Claude can help with:
- Requirements generation from natural language
- Architecture suggestions
- Component generation
- Model validation
- Traceability analysis
- Safety compliance checking
- Diagram generation
- Merge conflict resolution
- Model statistics
