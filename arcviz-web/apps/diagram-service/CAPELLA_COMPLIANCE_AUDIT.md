# Capella/Arcadia Compliance Audit Report

**Date**: November 4, 2025  
**Reference**: Complete MBSE Capella/Arcadia Diagram Specifications

## Executive Summary

This audit compares our current ArcLang rendering implementation against official Capella/Arcadia specifications to identify gaps and required improvements.

---

## 1. Color Code Compliance (Section 3)

### Current Status: ⚠️ PARTIAL

**Required Colors (Table 6)**:
| Element | Required | Current | Status |
|---------|----------|---------|--------|
| Operational Entity | #FFFF99 | ✅ #FFFF99 | ✅ OK |
| Operational Actor | #FFFF99 | ✅ #FFFF99 | ✅ OK |
| Operational Activity | #FFB266 | ❌ #FFD966 | ❌ WRONG |
| System Actor | #E6D7B8 | ❌ Missing | ❌ MISSING |
| System Function | #ADD8E6 | ❌ #70AD47 | ❌ WRONG |
| Logical Component | #6495ED | ❌ #70AD47 | ❌ WRONG |
| Logical Function | #4682B4 | ❌ Missing | ❌ MISSING |
| Physical Behavioral | #4169E1 | ❌ Missing | ❌ MISSING |
| Physical Node | #FFD700 | ❌ Missing | ❌ MISSING |
| Interactions/Exchanges | #808080 | ❌ #000000 | ❌ WRONG |
| Functional Exchanges | #4169E1 | ❌ Missing | ❌ MISSING |

**Safety Overlays**: ❌ NOT IMPLEMENTED
- ASIL A/B/C/D borders
- DAL A/B/C/D borders  
- SIL 1/2/3/4 borders

---

## 2. Layout Rules Compliance (Section 4)

### 2.1 Operational Architecture Blank (OAB) - Section 4.1

**Critical Requirements**:
- ✅ Actors on diagram periphery
- ✅ Entities as containers
- ✅ Activities inside entities
- ⚠️ Symmetrical layout (partially)
- ✅ No overlapping (now fixed)

**Status**: ✅ MOSTLY COMPLIANT (90%)

### 2.2 System Architecture Blank (SAB) - Section 4.2

**CRITICAL Requirements** (Red Box):
- ❌ System boundary NOT implemented
- ❌ System NOT centered
- ❌ Actors NOT on periphery (outside system boundary)
- ❌ Functions NOT allocated inside system boundary
- ❌ No visual distinction between inside/outside system

**Status**: ❌ NOT COMPLIANT (0%)  
**Priority**: CRITICAL - This is marked "MOST IMPORTANT" in specs

### 2.3 Dataflow Diagrams (SDFB/LDFB/PDFB) - Section 4.3

**Requirements**:
- ⚠️ Left-to-right flow (hierarchical exists, not enforced)
- ❌ Sugiyama algorithm NOT implemented
- ❌ Input functions on left NOT enforced
- ❌ Output functions on right NOT enforced
- ⚠️ Minimize edge crossings (done by ELK, not measured)
- ❌ Exchange items on arrows (labels exist, not structured)

**Status**: ⚠️ PARTIALLY COMPLIANT (40%)

### 2.4 Logical/Physical Architecture (LAB/PAB) - Section 4.4

**Requirements**:
- ✅ Hierarchical containment (implemented)
- ✅ Nested box visualization (working)
- ❌ Ports on component boundaries NOT implemented
- ✅ Component exchanges between components
- ⚠️ Clear hierarchical levels (partially)
- ⚠️ Minimum 20px padding (not enforced)

**Status**: ⚠️ PARTIALLY COMPLIANT (60%)

### 2.5 Physical Architecture (PAB) - Section 4.5

**UNIQUE Requirements**:
- ❌ Node Components (yellow) vs Behavioral (blue) NOT distinguished
- ❌ Behavioral deployed INSIDE nodes NOT implemented
- ❌ Physical Links (hardware connections) NOT distinguished
- ❌ HW/SW separation NOT clear

**Status**: ❌ NOT COMPLIANT (20%)

---

## 3. Port and Interface Specifications (Section 5)

### 5.1 Port Positioning Rules

**MANDATORY Positioning**:
- ❌ INPUT Ports: LEFT side NOT enforced
- ❌ OUTPUT Ports: RIGHT side NOT enforced
- ❌ BIDIRECTIONAL Ports: TOP/BOTTOM NOT enforced
- ❌ CONTROL Ports: TOP side NOT enforced
- ❌ Power/Ground Ports: BOTTOM side NOT enforced

**Distribution Rules**:
- ❌ Minimum 30px spacing NOT enforced
- ❌ Minimum 45° angle NOT enforced

**Status**: ❌ NOT IMPLEMENTED (0%)

### 5.2 Interface Notation

**Requirements**:
- ❌ Provided Interface (semi-circle) NOT implemented
- ❌ Required Interface (semi-circle inward) NOT implemented
- ⚠️ Port (small rectangle) EXISTS but not positioned

**Status**: ❌ NOT IMPLEMENTED (10%)

---

## 4. Traceability Visualization (Section 6)

**Requirements**:
- ❌ Vertical traceability across layers NOT implemented
- ❌ Trace link styles (dashed arrows) NOT implemented
- ❌ Horizontal traceability NOT implemented

**Status**: ❌ NOT IMPLEMENTED (0%)

---

## 5. Quality Metrics (Section 7)

### 7.1 Required Metrics (Table 10)

| Metric | Implemented | Status |
|--------|-------------|--------|
| Actor Placement (OA/SA) | ✅ Yes | ✅ OK |
| System Boundary (SA) | ❌ No | ❌ MISSING |
| Containment Validity | ✅ Yes | ✅ OK |
| Edge Crossings | ❌ Not measured | ⚠️ PARTIAL |
| Port Side Correctness | ❌ No | ❌ MISSING |
| Color Compliance | ❌ No | ❌ WRONG COLORS |
| Grid Alignment | ❌ No | ❌ MISSING |
| Label Overlap | ✅ Fixed | ✅ OK |
| Flow Direction | ⚠️ Partial | ⚠️ PARTIAL |
| Component Nesting (PA) | ❌ No | ❌ MISSING |
| Interface Notation | ❌ No | ❌ MISSING |
| Safety Annotations | ❌ No | ❌ MISSING |

**Quality Scoring Formula**: ❌ NOT IMPLEMENTED

**Acceptance Criteria**: ❌ NOT MEASURED
- Target for ASIL-D: 85/100
- Target for DAL-A: 90/100

**Status**: ❌ NOT IMPLEMENTED (20%)

---

## 6. Algorithm Selection (Section 8.1)

### Current vs Required (Table 12)

| Diagram Type | Required | Current | Status |
|--------------|----------|---------|--------|
| OAB | Force-directed + Periphery | ✅ Hybrid (includes periphery) | ✅ OK |
| SAB | Boundary-centered + Periphery | ❌ Hierarchical only | ❌ WRONG |
| SDFB/LDFB/PDFB | Sugiyama (Layered) | ⚠️ ELK (hierarchical) | ⚠️ PARTIAL |
| SFBD/LFBD/PFBD | Reingold-Tilford Tree | ❌ Not implemented | ❌ MISSING |
| LAB/PAB | Nested box packing | ✅ Hierarchical | ✅ OK |
| Sequence | Vertical timeline | ❌ Not implemented | ❌ MISSING |
| State Machines | Custom state layout | ✅ Exists | ✅ OK |

**Status**: ⚠️ PARTIALLY COMPLIANT (50%)

---

## 7. Multi-Pass Optimization Pipeline (Section 8.2)

### Current: 4-Pass (Section 8.2 requires 5-pass)

**Current Implementation**:
1. ✅ Layer 1: ELK (70%) - Initial hierarchical
2. ✅ Layer 2: Dagre (20%) - Crossing reduction
3. ✅ Layer 3: D3 (10%) - Collision detection
4. ✅ Layer 4: Capella - Style refinement

**Required 5-Pass Pipeline**:
1. ✅ Pass 1: Initial Layout (1-2s)
2. ⚠️ Pass 2: Crossing Reduction (3-5s) - EXISTS but not timed
3. ⚠️ Pass 3: Edge Beautification (2-3s) - PARTIAL (no orthogonal routing)
4. ❌ Pass 4: Fine-Tuning (3-5s) - NOT IMPLEMENTED
5. ❌ Pass 5: Arcadia Compliance (1-2s) - NOT IMPLEMENTED

**Missing in Pass 3**:
- ❌ Orthogonal routing
- ❌ Bezier smoothing for allocation links
- ❌ Port optimization
- ⚠️ Label positioning (exists, not optimal)

**Missing Pass 4**:
- ❌ Micro-adjustments (sub-pixel)
- ❌ Grid alignment
- ❌ Aspect ratio correction
- ❌ Whitespace distribution (Gini coefficient)

**Missing Pass 5**:
- ❌ Layer-specific rule verification
- ❌ Color coding validation
- ❌ Traceability validation
- ❌ Quality report generation

**Status**: ⚠️ PARTIALLY COMPLIANT (60%)

---

## 8. Technology Stack (Section 8.3)

### Current vs Required

| Component | Required | Current | Status |
|-----------|----------|---------|--------|
| Base Layout | ELK or Dagre | ✅ ELK + Dagre + D3 | ✅ BETTER |
| Constraint Solving | OR-Tools/Choco | ❌ None | ❌ MISSING |
| Optimization | Simulated Annealing | ❌ None | ❌ MISSING |
| Graph Analysis | JGraphT/NetworkX | ❌ Custom | ⚠️ PARTIAL |
| Rendering | D3.js/SVG.js | ✅ SVG generation | ✅ OK |

**Status**: ⚠️ PARTIALLY COMPLIANT (50%)

---

## 9. Performance Targets (Section 8.4)

**Table 13 Requirements**:
| Model Size | Elements | Target | Our Current |
|------------|----------|--------|-------------|
| Small | < 50 | < 1s | ✅ ~90ms |
| Medium | 50-200 | 1-5s | ✅ ~100ms |
| Large | 200-500 | 5-15s | ❓ Not tested |
| Very Large | 500-2000 | 15-60s | ❓ Not tested |

**Status**: ✅ COMPLIANT for small/medium (tested)

---

## 10. Arcadia 7 Dimensions Coverage

### Current Coverage Assessment

**5 Engineering Perspectives (Layers)**:
1. ✅ Operational Analysis (OA) - WORKING (with gaps)
2. ⚠️ System Analysis (SA) - MISSING boundary
3. ⚠️ Logical Architecture (LA) - PARTIAL (wrong colors)
4. ⚠️ Physical Architecture (PA) - MISSING deployment
5. ❌ EPBS - NOT IMPLEMENTED

**Status**: ⚠️ 3/5 LAYERS FUNCTIONAL (60%)

---

## CRITICAL GAPS SUMMARY

### 🔴 CRITICAL (Must Fix for Production)

1. **System Boundary Not Implemented** (Section 4.2)
   - SAB diagrams require visible system boundary
   - Actors must be on periphery OUTSIDE system
   - This is marked "MOST IMPORTANT" in specs

2. **Wrong Colors Throughout** (Section 3.2)
   - Operational Activities: #FFB266 not #FFD966
   - System Functions: #ADD8E6 not #70AD47
   - Logical Components: #6495ED not #70AD47
   - All logical/physical colors wrong

3. **Safety Overlays Missing** (Section 3.3)
   - ASIL A/B/C/D border colors
   - DAL A/B/C/D border colors
   - SIL 1/2/3/4 border colors
   - Required for automotive/aerospace

4. **Port Positioning Not Enforced** (Section 5.1)
   - INPUT must be LEFT
   - OUTPUT must be RIGHT
   - CONTROL must be TOP
   - This is "MANDATORY" in specs

5. **Physical Architecture Deployment Missing** (Section 4.5)
   - No distinction between Node (HW) and Behavioral (SW)
   - No visual deployment (SW nested in HW)
   - No physical link visualization

### 🟡 HIGH PRIORITY (Required for Full Compliance)

6. **Quality Metrics Not Measured** (Section 7)
   - No quality scoring formula implementation
   - No validation against acceptance criteria
   - No compliance reports generated

7. **Interface Notation Missing** (Section 5.2)
   - No provided interface symbols (semi-circles)
   - No required interface symbols
   - No proper interface rendering

8. **Traceability Not Implemented** (Section 6)
   - No vertical traceability across layers
   - No trace link visualization
   - No horizontal traceability

9. **Pass 4 & 5 Missing** (Section 8.2)
   - No fine-tuning pass (grid alignment, aspect ratio)
   - No Arcadia compliance pass (validation)

### 🟢 MEDIUM PRIORITY (Enhancements)

10. **Constraint Solver Missing** (Section 8.3)
11. **Sugiyama Algorithm for Dataflow** (Section 4.3)
12. **Orthogonal Edge Routing** (Section 8.2, Pass 3)
13. **EPBS Layer** (Section 2.5)

---

## COMPLIANCE SCORE CARD

| Category | Score | Weight | Weighted |
|----------|-------|--------|----------|
| Color Compliance | 20% | 15% | 3.0% |
| Layout Rules | 55% | 20% | 11.0% |
| Port/Interface | 10% | 15% | 1.5% |
| Traceability | 0% | 10% | 0.0% |
| Quality Metrics | 20% | 15% | 3.0% |
| Algorithm Selection | 50% | 10% | 5.0% |
| Multi-Pass Pipeline | 60% | 10% | 6.0% |
| Performance | 100% | 5% | 5.0% |

### **TOTAL COMPLIANCE: 34.5%** ❌

**Required for Production**: 75% (Acceptable)  
**Required for ASIL-D**: 85%  
**Required for DAL-A**: 90%

---

## RECOMMENDED ACTION PLAN

### Phase 1: Critical Fixes (Week 1)
1. Implement correct color scheme (Table 6)
2. Add system boundary for SAB diagrams
3. Implement safety-critical border overlays
4. Fix port positioning rules

### Phase 2: High Priority (Week 2)
5. Implement quality metrics validation
6. Add interface notation (semi-circles)
7. Implement Pass 5 (Arcadia compliance validation)
8. Add physical architecture deployment visualization

### Phase 3: Full Compliance (Week 3-4)
9. Implement traceability visualization
10. Add Pass 4 (fine-tuning: grid, aspect ratio)
11. Integrate constraint solver (OR-Tools)
12. Implement Sugiyama for dataflow diagrams
13. Add EPBS layer support

### Target: 85% compliance within 4 weeks

---

## CONCLUSION

Our current implementation provides a **strong foundation** with the hybrid ELK+Dagre+D3 engine achieving excellent performance and zero overlaps. However, we are **only 34.5% compliant** with official Capella/Arcadia specifications.

**Critical gaps preventing production use**:
- Wrong colors throughout (automotive/aerospace won't accept)
- Missing system boundary (core SAB requirement)
- No safety-critical annotations (regulatory requirement)
- Port positioning not enforced (causes confusion)
- No quality measurement/validation

**Recommended**: Implement Phase 1 critical fixes immediately before any production deployment.
