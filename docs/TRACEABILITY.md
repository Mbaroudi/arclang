# 🔗 Traceability Guide

**Complete guide for requirements traceability in ArcLang**

---

## Table of Contents

1. [Introduction](#introduction)
2. [Why Traceability Matters](#why-traceability-matters)
3. [Trace Types](#trace-types)
4. [Creating Traces](#creating-traces)
5. [Validation & Analysis](#validation--analysis)
6. [Traceability Matrix](#traceability-matrix)
7. [Best Practices](#best-practices)
8. [Certification Requirements](#certification-requirements)

---

## Introduction

**Traceability** is the ability to trace relationships between requirements, design elements, implementation, and verification.

### What is a Trace?

A trace is a **relationship** between two elements showing how one element relates to another.

```arc
trace "LC-001" satisfies "REQ-001" {
    rationale: "Controller implements the requirement"
}
```

### Key Concepts

- **Forward traceability**: Requirements → Design → Implementation → Test
- **Backward traceability**: Test → Implementation → Design → Requirements
- **Bidirectional traceability**: Both forward and backward
- **Coverage**: Percentage of requirements traced

---

## Why Traceability Matters

### 1. Regulatory Compliance

**Required by standards:**
- ISO 26262 (Automotive)
- DO-178C (Aerospace)
- IEC 61508 (Industrial)
- FDA 21 CFR Part 11 (Medical)

### 2. Change Impact Analysis

**Understanding effects of changes:**
```
REQ-001 changed
  ↓ satisfies
LC-001 affected
  ↓ implements  
LF-001 affected
  ↓ tested by
TC-001 must be updated
```

### 3. Verification & Validation

**Proving completeness:**
- All requirements have implementations
- All implementations are tested
- All tests trace to requirements

### 4. Project Management

**Tracking progress:**
- Requirements coverage: 85%
- Design coverage: 70%
- Test coverage: 90%

---

## Trace Types

### 1. Satisfies (Requirement ← Component)

**Meaning**: Component satisfies/implements a requirement

```arc
requirement "REQ-001" {
    description: "System shall measure distance"
}

component "Distance Sensor" {
    id: "LC-001"
}

trace "LC-001" satisfies "REQ-001" {
    rationale: "Sensor provides distance measurement capability"
    coverage: "Full"
}
```

### 2. Implements (Component ← Function)

**Meaning**: Function implements component behavior

```arc
component "Controller" {
    id: "LC-001"
    
    function "Control Speed" {
        id: "LF-001"
    }
}

trace "LF-001" implements "LC-001" {
    rationale: "Function realizes component behavior"
}
```

### 3. Deploys (Node ← Component)

**Meaning**: Physical node deploys logical component

```arc
component "Controller" {
    id: "LC-001"
}

node "Main ECU" {
    id: "PN-001"
}

trace "PN-001" deploys "LC-001" {
    rationale: "ECU hosts controller software"
    allocation: "Partition A"
}
```

### 4. Derives From (Requirement ← Requirement)

**Meaning**: Child requirement derives from parent

```arc
requirement "REQ-PARENT" {
    description: "High-level requirement"
}

requirement "REQ-CHILD" {
    description: "Detailed requirement"
}

trace "REQ-CHILD" derives_from "REQ-PARENT" {
    rationale: "Refinement of parent requirement"
}
```

### 5. Refines (Element ← Element)

**Meaning**: Element refines another at different abstraction level

```arc
trace "LC-001" refines "REQ-001" {
    rationale: "Component refines requirement to design level"
}
```

### 6. Mitigates (Requirement ← Hazard)

**Meaning**: Requirement mitigates a hazard

```arc
hazard "HAZ-001" {
    description: "Unintended acceleration"
}

requirement "REQ-SAFE-001" {
    description: "Detect brake override"
}

trace "REQ-SAFE-001" mitigates "HAZ-001" {
    rationale: "Brake override prevents unintended acceleration"
}
```

### 7. Verifies (Test ← Requirement)

**Meaning**: Test case verifies requirement

```arc
requirement "REQ-001" {
    description: "Response time < 100ms"
}

test_case "TC-001" {
    description: "Measure response time"
}

trace "TC-001" verifies "REQ-001" {
    rationale: "Test validates timing requirement"
    method: "Measurement"
    result: "Pass"
}
```

---

## Creating Traces

### Basic Trace

```arc
trace "SOURCE_ID" trace_type "TARGET_ID" {
    rationale: "Why this relationship exists"
}
```

### Full Trace Syntax

```arc
trace "LC-001" satisfies "REQ-001" {
    // Required
    rationale: "Detailed explanation of relationship"
    
    // Optional metadata
    coverage: "Full"              // Full, Partial, None
    verification: "Test"          // How verified
    status: "Approved"            // Draft, Review, Approved
    
    // Additional info
    notes: "Additional context"
    reviewer: "John Doe"
    review_date: "2025-10-18"
    
    // Attributes
    confidence: "High"            // High, Medium, Low
    completeness: "100%"
}
```

### Multiple Traces

```arc
// One requirement satisfied by multiple components
trace "LC-001" satisfies "REQ-001" { 
    rationale: "Primary implementation"
}

trace "LC-002" satisfies "REQ-001" {
    rationale: "Backup/redundant implementation"
}

// One component satisfies multiple requirements
trace "LC-001" satisfies "REQ-001" {
    rationale: "Distance measurement"
}

trace "LC-001" satisfies "REQ-002" {
    rationale: "Speed calculation"
}
```

### Trace Chains

**Complete chain from requirement to test:**

```arc
// Requirement
requirement "REQ-001" {
    description: "Maintain safe distance"
}

// Component satisfies requirement
component "ACC Controller" {
    id: "LC-001"
}

trace "LC-001" satisfies "REQ-001" {
    rationale: "Controller implements distance control"
}

// Function implements component
function "Control Distance" {
    id: "LF-001"
    parent: "LC-001"
}

trace "LF-001" implements "LC-001" {
    rationale: "Function realizes component behavior"
}

// Physical deployment
node "Main ECU" {
    id: "PN-001"
}

trace "PN-001" deploys "LC-001" {
    rationale: "Software runs on ECU"
}

// Test verification
test_case "TC-001" {
    description: "Verify distance control"
}

trace "TC-001" verifies "REQ-001" {
    rationale: "Test validates requirement"
}
```

---

## Validation & Analysis

### CLI Commands

```bash
# Validate traceability
arclang trace model.arc --validate

# Show traceability matrix
arclang trace model.arc --matrix

# Check coverage
arclang trace model.arc --coverage

# Find orphan requirements (not traced)
arclang trace model.arc --orphans

# Find missing traces
arclang trace model.arc --gaps

# Export trace report
arclang trace model.arc --report --output trace_report.html
```

### Validation Example

```bash
$ arclang trace model.arc --validate

Analyzing traceability in model.arc...

✓ Requirements: 10
✓ Components: 8
✓ Traces: 12

Validation Results:
──────────────────────────────────────
✓ All requirements have traces
✓ No orphan components
⚠ 2 requirements missing test verification
  - REQ-003 (no test case)
  - REQ-007 (no test case)

Traceability Coverage: 80%
Recommendation: Add test cases for REQ-003 and REQ-007
```

### Traceability Warnings

**Common issues detected:**

```bash
# Orphan requirement (not satisfied by any component)
⚠ REQ-001 is not satisfied by any component
  Recommendation: Add trace or mark as future work

# Orphan component (doesn't satisfy any requirement)
⚠ LC-999 doesn't satisfy any requirement
  Recommendation: Add trace or remove component

# Missing verification
⚠ REQ-005 has no test verification
  Recommendation: Add test case and trace

# Incomplete chain
⚠ LC-001 satisfies REQ-001 but has no functions
  Recommendation: Add functions to implement component

# Circular dependency
✗ REQ-001 derives from REQ-002, which derives from REQ-001
  Error: Circular dependency detected
```

---

## Traceability Matrix

### Forward Traceability Matrix

**Requirements → Components → Functions → Tests**

```
┌──────────┬──────────┬──────────┬──────────┬──────────┐
│ Req ID   │ LC-001   │ LC-002   │ LC-003   │ Tests    │
├──────────┼──────────┼──────────┼──────────┼──────────┤
│ REQ-001  │    ✓     │          │          │ TC-001   │
│ REQ-002  │    ✓     │    ✓     │          │ TC-002   │
│ REQ-003  │          │    ✓     │    ✓     │ TC-003   │
└──────────┴──────────┴──────────┴──────────┴──────────┘

Coverage: 100% (3/3 requirements traced)
```

### Backward Traceability Matrix

**Tests → Functions → Components → Requirements**

```
┌──────────┬──────────┬──────────┬──────────┬──────────┐
│ Test ID  │ LF-001   │ LF-002   │ LC-001   │ Req ID   │
├──────────┼──────────┼──────────┼──────────┼──────────┤
│ TC-001   │    ✓     │          │    ✓     │ REQ-001  │
│ TC-002   │          │    ✓     │    ✓     │ REQ-002  │
│ TC-003   │    ✓     │    ✓     │          │ REQ-003  │
└──────────┴──────────┴──────────┴──────────┴──────────┘

Coverage: 100% (3/3 tests traced)
```

### Generate Matrix

```bash
# HTML matrix
arclang trace model.arc --matrix --output matrix.html --format html

# CSV matrix
arclang trace model.arc --matrix --output matrix.csv --format csv

# Excel matrix
arclang trace model.arc --matrix --output matrix.xlsx --format excel
```

### Matrix Output Example

**HTML Output:**

```html
<!DOCTYPE html>
<html>
<head>
    <title>Traceability Matrix</title>
    <style>
        table { border-collapse: collapse; }
        th, td { border: 1px solid #ccc; padding: 8px; }
        .traced { background: #90EE90; }
        .missing { background: #FFB6C6; }
    </style>
</head>
<body>
    <h1>Traceability Matrix</h1>
    <table>
        <tr>
            <th>Requirement</th>
            <th>Component</th>
            <th>Test</th>
            <th>Status</th>
        </tr>
        <tr>
            <td>REQ-001</td>
            <td class="traced">LC-001</td>
            <td class="traced">TC-001</td>
            <td class="traced">✓ Complete</td>
        </tr>
        <tr>
            <td>REQ-002</td>
            <td class="traced">LC-002</td>
            <td class="missing">—</td>
            <td class="missing">⚠ Missing test</td>
        </tr>
    </table>
    <p>Coverage: 50% (1/2 fully traced)</p>
</body>
</html>
```

---

## Best Practices

### 1. Complete Rationale

```arc
// ✅ Good: Clear, specific rationale
trace "LC-001" satisfies "REQ-001" {
    rationale: "Distance sensor component provides radar-based measurement 
                capability required by REQ-001, with accuracy ±0.5m and 
                update rate 50Hz as specified"
}

// ❌ Bad: Vague rationale
trace "LC-001" satisfies "REQ-001" {
    rationale: "Implements requirement"
}
```

### 2. Trace Everything

```arc
// Complete traceability chain

// 1. Hazard to Safety Goal
trace "SG-001" mitigates "HAZ-001" { ... }

// 2. Safety Goal to Requirement
trace "REQ-001" derives_from "SG-001" { ... }

// 3. Requirement to Component
trace "LC-001" satisfies "REQ-001" { ... }

// 4. Component to Function
trace "LF-001" implements "LC-001" { ... }

// 5. Component to Hardware
trace "PN-001" deploys "LC-001" { ... }

// 6. Requirement to Test
trace "TC-001" verifies "REQ-001" { ... }
```

### 3. Validate Regularly

```bash
# Run validation after every change
git commit -m "Add new component"
arclang trace model.arc --validate

# Integrate into CI/CD
.github/workflows/ci.yml:
  - name: Validate Traceability
    run: arclang trace model.arc --validate --fail-on-warning
```

### 4. Coverage Metrics

```arc
traceability_metrics "Project Metrics" {
    // Forward traceability
    requirements_to_components: "95%"    // Target: 100%
    components_to_functions: "90%"       // Target: 100%
    requirements_to_tests: "85%"         // Target: 100%
    
    // Backward traceability
    tests_to_requirements: "100%"        // All tests must trace
    functions_to_components: "100%"      // All functions must trace
    
    // Overall
    overall_coverage: "93%"              // Good!
    target_coverage: "100%"
    
    // Orphans
    orphan_requirements: 2               // Should be 0
    orphan_components: 0                 // Good!
    missing_tests: 3                     // Should be 0
}
```

### 5. Review Traces

```arc
trace "LC-001" satisfies "REQ-001" {
    rationale: "Clear explanation"
    
    // Review information
    reviewed_by: "Safety Team"
    review_date: "2025-10-15"
    review_status: "Approved"
    review_comments: "Trace verified correct"
}
```

---

## Certification Requirements

### ISO 26262 Requirements

**Part 8 - Supporting Processes**

```arc
iso26262_traceability "ISO 26262 Compliance" {
    // Clause 8.6 - Configuration management
    requirement: "Bidirectional traceability throughout lifecycle"
    
    // Required traces
    traces_required: [
        "Safety goals ← → Functional safety requirements",
        "Functional safety requirements ← → Technical safety requirements",
        "Technical safety requirements ← → Software safety requirements",
        "Software safety requirements ← → Software architecture",
        "Software architecture ← → Software units",
        "Software units ← → Test cases"
    ]
    
    // Evidence
    evidence: "Traceability matrix demonstrating complete coverage"
    
    // Tool support
    tool: "ArcLang traceability validation"
}
```

### DO-178C Requirements

**Section 6.3 - Software Verification Process**

```arc
do178c_traceability "DO-178C Compliance" {
    // Objective
    objective: "Ensure complete and accurate traceability"
    
    // Required traces
    traces_required: [
        "High-level requirements ← → Low-level requirements",
        "Low-level requirements ← → Source code",
        "Requirements ← → Test cases",
        "Requirements ← → Test procedures",
        "Requirements ← → Test results"
    ]
    
    // Independence (DAL A/B)
    independence: "Traceability verified by independent team"
    
    // Evidence (Table A-7)
    evidence: [
        "Software Requirements Data (SRD) with traces",
        "Software Design Description (SDD) with traces",
        "Source code with requirement IDs",
        "Test cases with requirement references",
        "Traceability matrix"
    ]
}
```

### IEC 61508 Requirements

**Part 3 - Software requirements**

```arc
iec61508_traceability "IEC 61508 Compliance" {
    // Clause 7.4.2 - Traceability
    requirement: "Maintain traceability between all lifecycle phases"
    
    // Required traces
    traces_required: [
        "Safety requirements specification ← → Software safety requirements",
        "Software safety requirements ← → Software architecture",
        "Software architecture ← → Software modules",
        "Software modules ← → Source code",
        "Requirements ← → Verification"
    ]
    
    // SIL requirements
    sil_3_requirement: "Complete forward and backward traceability"
    sil_4_requirement: "Independent verification of all traces"
}
```

---

## Advanced Topics

### Impact Analysis

```arc
impact_analysis "Change Impact" {
    changed_element: "REQ-001"
    
    // Forward impact (what depends on this)
    forward_impact: [
        "LC-001 (satisfies REQ-001)",
        "LF-001 (implements LC-001)",
        "TC-001 (verifies REQ-001)"
    ]
    
    // Backward impact (what this depends on)
    backward_impact: [
        "REQ-PARENT (REQ-001 derives from)"
    ]
    
    // Recommendations
    actions: [
        "Review LC-001 design",
        "Update LF-001 implementation",
        "Re-run TC-001",
        "Update documentation"
    ]
}
```

### Trace Visualization

```bash
# Generate trace graph
arclang trace model.arc --visualize --output traces.svg

# Filter by type
arclang trace model.arc --visualize --type satisfies

# Show only high-level
arclang trace model.arc --visualize --level requirements
```

**Graph Output:**
```
REQ-001 ──satisfies──> LC-001 ──implements──> LF-001
   │                      │
   └────verifies────> TC-001
```

### Automated Trace Generation

```arc
// Auto-generate traces from attributes
component "Distance Sensor" {
    id: "LC-001"
    satisfies: ["REQ-001", "REQ-002"]  // Auto-creates traces
}

// Compiler generates:
// trace "LC-001" satisfies "REQ-001" { rationale: "Auto-generated" }
// trace "LC-001" satisfies "REQ-002" { rationale: "Auto-generated" }
```

---

## Summary

### Key Points

1. **Traceability is mandatory** for safety-critical systems
2. **Complete chains** from requirements through tests
3. **Validate regularly** to catch gaps early
4. **Document rationale** for all traces
5. **Use tools** to automate validation

### Coverage Goals

| Metric | Target | Critical |
|--------|--------|----------|
| Requirements → Components | 100% | Yes |
| Components → Functions | 100% | Yes |
| Requirements → Tests | 100% | Yes |
| Orphan Requirements | 0 | Yes |
| Orphan Components | 0 | No |

### Quick Commands

```bash
# Validate traceability
arclang trace model.arc --validate

# Generate matrix
arclang trace model.arc --matrix --output matrix.html

# Check coverage
arclang trace model.arc --coverage

# Find gaps
arclang trace model.arc --gaps

# Export report
arclang trace model.arc --report --format pdf
```

---

**Version**: 1.0.0  
**Complete**: Yes  
**Certification Ready**: Yes ✅  
**Standards**: ISO 26262, DO-178C, IEC 61508
