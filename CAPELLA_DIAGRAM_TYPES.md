# Capella Diagram Types - Full Reference

## The Real Issue

You're right! In Capella, each architectural level doesn't just have **one view** - it has **multiple diagram types**. Currently, the visualizer only shows one generic view per layer.

## Capella's Complete Diagram Types

### 1. Operational Analysis (OA)

| Diagram Type | Abbreviation | Purpose | Shows |
|--------------|--------------|---------|-------|
| **Operational Architecture Blank** | OAB | Actor interactions | Actors, interactions, use cases |
| **Operational Activity Breakdown** | OAB | Activity hierarchy | Operational activities tree |
| **Operational Process** | OPD | Process flow | Operational scenarios |
| **Operational Capabilities Blank** | OCB | Capabilities | System capabilities |

### 2. System Analysis (SA)

| Diagram Type | Abbreviation | Purpose | Shows |
|--------------|--------------|---------|-------|
| **System Architecture Blank** | SAB | System context | System + external actors |
| **System Dataflow Blank** | SDFB | Data exchanges | Data flow between functions |
| **System Capability** | MC | Capabilities | Mission/capabilities realized |
| **Functional Chain** | - | Function sequences | Ordered function execution |

### 3. Logical Architecture (LA)

| Diagram Type | Abbreviation | Purpose | Shows |
|--------------|--------------|---------|-------|
| **Logical Architecture Blank** | LAB | Component structure | Logical components hierarchy |
| **Logical Dataflow Blank** | LDFB | Data flows | Data exchange between components |
| **Logical Component Breakdown** | LCBD | Component tree | Hierarchical component decomposition |
| **Interfaces Diagram** | - | Interfaces | Component interfaces detail |

### 4. Physical Architecture (PA)

| Diagram Type | Abbreviation | Purpose | Shows |
|--------------|--------------|---------|-------|
| **Physical Architecture Blank** | PAB | Hardware deployment | Physical nodes, deployment |
| **Physical Dataflow Blank** | PDFB | Physical data flow | Data on hardware |
| **Physical Component Breakdown** | PCBD | Hardware tree | Node hierarchy |
| **Deployment Diagram** | - | Software → Hardware | Which SW runs on which HW |

### 5. EPBS (End Product Breakdown Structure)

| Diagram Type | Abbreviation | Purpose | Shows |
|--------------|--------------|---------|-------|
| **EPBS Architecture** | - | Product structure | Configuration items |
| **Configuration Item Breakdown** | CIBD | Product tree | CI hierarchy |
| **Physical/EPBS Mapping** | - | Allocation | Components → Products |

## What Your Visualizer Currently Shows

### Current State ❌
- **Operational**: Generic force layout (not specific diagram type)
- **System**: Generic top-down layout (not specific diagram type)
- **Logical**: Generic left-right layout (not specific diagram type)
- **Physical**: Not implemented
- **EPBS**: Not implemented

### What It Should Show ✅

**For Each Layer**, user should choose diagram type:

```
Layer: Operational ▼
  └─ Diagram Type: [OAB] [OPD] [OCB] [Activity Breakdown]
  
Layer: System ▼
  └─ Diagram Type: [SAB] [SDFB] [Capabilities] [Functional Chain]
  
Layer: Logical ▼
  └─ Diagram Type: [LAB] [LDFB] [LCBD] [Interfaces]
  
Layer: Physical ▼
  └─ Diagram Type: [PAB] [PDFB] [PCBD] [Deployment]
  
Layer: EPBS ▼
  └─ Diagram Type: [Architecture] [CIBD] [Mapping]
```

## Recommended Implementation

### Option 1: Two-Level Dropdown (Better UX)

```typescript
// First dropdown: Layer
<select>
  <option>Operational Analysis</option>
  <option>System Analysis</option>
  <option>Logical Architecture</option>
  <option>Physical Architecture</option>
  <option>EPBS</option>
</select>

// Second dropdown: Diagram Type (changes based on layer)
<select>
  <!-- If Operational selected -->
  <option>Architecture Blank (OAB)</option>
  <option>Activity Breakdown</option>
  <option>Process Diagram</option>
  <option>Capabilities Blank</option>
  
  <!-- If System selected -->
  <option>Architecture Blank (SAB)</option>
  <option>Dataflow Blank (SDFB)</option>
  <option>Capabilities</option>
  <option>Functional Chain</option>
  
  <!-- etc. -->
</select>
```

### Option 2: Single Combined Dropdown

```typescript
<select>
  <optgroup label="Operational Analysis">
    <option>OA: Architecture Blank (OAB)</option>
    <option>OA: Process Diagram (OPD)</option>
    <option>OA: Capabilities Blank</option>
  </optgroup>
  
  <optgroup label="System Analysis">
    <option>SA: Architecture Blank (SAB)</option>
    <option>SA: Dataflow Blank (SDFB)</option>
    <option>SA: Capabilities</option>
  </optgroup>
  
  <optgroup label="Logical Architecture">
    <option>LA: Architecture Blank (LAB)</option>
    <option>LA: Dataflow Blank (LDFB)</option>
    <option>LA: Component Breakdown</option>
  </optgroup>
  
  <!-- etc. -->
</select>
```

## Key Differences Between Diagram Types

### Example: Logical Architecture

**LAB (Architecture Blank)**:
- Shows: Component boxes and their connections
- Layout: Hierarchical or network
- Focus: Component relationships

**LDFB (Dataflow Blank)**:
- Shows: Data exchanges between components
- Layout: Flow-oriented
- Focus: Information flow
- Arrows labeled with data types

**LCBD (Component Breakdown)**:
- Shows: Tree of components
- Layout: Hierarchical tree
- Focus: Decomposition structure

**Interfaces Diagram**:
- Shows: One component's interfaces in detail
- Layout: Star pattern (component in center)
- Focus: Interface contracts

## Visual Style Differences

### Architecture Blank (AB)
- Boxes with connections
- Standard component shapes
- Hierarchical or network layout

### Dataflow Blank (DFB)
- Emphasized arrows (thicker)
- Arrow labels (data names)
- Flow-oriented left-right or top-down

### Breakdown Diagrams (BD)
- Tree structure
- Parent-child links
- Indentation or tree layout

### Process/Scenario Diagrams
- Swim lanes
- Sequential ordering
- Time/flow axis

## Why This Matters

### For Your ACC Example

**Currently**: You see one generic "logical view"

**Should Be**:
- **LAB**: Shows 9 components in architecture
- **LDFB**: Shows data flow from Radar → Fusion → Controller
- **LCBD**: Shows component hierarchy (Sensor Fusion contains sub-functions)
- **Interfaces**: Shows Long Range Radar's interfaces in detail

Each diagram type tells a **different story** about the same architectural layer!

## Implementation Priority

### Phase 1 (Essential) ✅
1. **LAB** (Logical Architecture Blank)
2. **SAB** (System Architecture Blank)
3. **OAB** (Operational Architecture Blank)

### Phase 2 (Important)
4. **LDFB** (Logical Dataflow Blank)
5. **SDFB** (System Dataflow Blank)
6. **PAB** (Physical Architecture Blank)

### Phase 3 (Nice to Have)
7. **LCBD** (Component Breakdown)
8. **EPBS Architecture**
9. **Functional Chains**
10. **Scenarios/Process Diagrams**

## Current Status

Your visualizer needs:
1. ✅ Parse all 5 layers (will add Physical + EPBS)
2. ❌ **Add diagram type selector** ← KEY MISSING FEATURE
3. ❌ **Implement different rendering per diagram type**
4. ❌ **Change layout algorithm per diagram type**

## Next Steps

Would you like me to:
1. **Add a diagram type selector** with 2-3 types per layer?
2. **Implement different visualizations** for each diagram type?
3. **Start with essential ones** (AB + DFB for each layer)?

The key insight: **Capella isn't just 3-5 layers, it's 15-20 different diagram types across those layers!** Each tells a different architectural story. 🎯
