# Layer-Specific Visualization - COMPLETE! 🎨

## Problem Identified

You were absolutely right! Having the **same layout for all layers wasn't appropriate**. In Capella and proper MBSE methodologies, each architectural layer should have its own **specific visual representation**.

## Solution: Layer-Specific Diagram Types

Now each layer has its **own layout algorithm and visual style**:

### 🎭 **Operational Analysis Layer**

**Purpose**: Show actor interactions and operational context

**Layout Algorithm**: **Force-directed** (organic, relationship-focused)
```javascript
'elk.algorithm': 'force'
'elk.force.repulsion': '80'
'elk.spacing.nodeNode': '80'
```

**Visual Style**:
- **Node Shape**: Rounded rectangles with thick borders (3px)
- **Color**: Yellow/gold theme
- **Layout**: Organic, shows relationships naturally
- **Best For**: 3 actors (Driver, Lead Vehicle, Environment)

**Capella Equivalent**: Operational Architecture Blank (OAB)

---

### 📋 **System Analysis Layer**

**Purpose**: Show requirement hierarchies and dependencies

**Layout Algorithm**: **Layered (Top-Down)** (hierarchical flow)
```javascript
'elk.algorithm': 'layered'
'elk.direction': 'DOWN'
'elk.layered.spacing.nodeNodeBetweenLayers': '100'
```

**Visual Style**:
- **Node Shape**: Rectangles with dashed borders (document style)
- **Color**: Pink theme (or safety-level colors)
- **Layout**: Top-down hierarchical layers
- **Best For**: 7 requirements with priorities

**Capella Equivalent**: System Architecture Blank (SAB) / Capabilities Diagram

---

### ⚙️ **Logical Architecture Layer**

**Purpose**: Show component structure and data flow

**Layout Algorithm**: **Layered (Left-Right)** (data flow emphasis)
```javascript
'elk.algorithm': 'layered'
'elk.direction': 'RIGHT'
'elk.hierarchyHandling': 'INCLUDE_CHILDREN'
```

**Visual Style**:
- **Node Shape**: Standard rounded rectangles
- **Color**: Blue theme (or safety-level colors)
- **Layout**: Left-to-right flow showing data progression
- **Best For**: 9 components in processing pipeline

**Capella Equivalent**: Logical Architecture Blank (LAB) / Logical Dataflow Blank (LDFB)

---

## Visual Comparison Table

| Layer | Algorithm | Direction | Node Style | Border | Use Case |
|-------|-----------|-----------|------------|--------|----------|
| **Operational** | Force-directed | Organic | Thick rounded | Solid 3px | Actor interactions |
| **System** | Layered | Top-Down | Document | Dashed 2px | Requirement flow |
| **Logical** | Layered | Left-Right | Standard | Solid 2px | Component pipeline |

## How It Works

### 1. **Automatic Layer Detection**

When you filter to a specific layer, the system:
1. Counts nodes per layer
2. Determines dominant layer
3. Selects appropriate layout algorithm
4. Applies layer-specific visual styling

```typescript
function detectDominantLayer(graph: ArchitectureGraph): string {
  // Counts nodes by layer
  // Returns 'operational', 'system', or 'logical'
}
```

### 2. **Layer-Specific Layouts**

```typescript
getLayerLayoutOptions(layer: string) {
  switch (layer) {
    case 'operational': return forceDirectedLayout()
    case 'system': return topDownLayout()
    case 'logical': return leftRightLayout()
  }
}
```

### 3. **Visual Differentiation**

**Actors (Operational)**:
- Thick borders (3px)
- More rounded corners (rx: 12)
- Larger spacing (80px)
- Emphasizes identity

**Requirements (System)**:
- Dashed borders (5,3 pattern)
- Document-like appearance
- Vertical hierarchy
- Emphasizes flow

**Components (Logical)**:
- Solid borders (2px)
- Standard rounded (rx: 8)
- Horizontal flow
- Emphasizes data processing

## Your ACC Architecture - Layer Views

### 🎭 **Operational View** (Select "Operational")

**What You See**:
```
         [Driver]
            |
            ↓
    ← [Lead Vehicle] →
            ↓
      [Environment]
```

**Characteristics**:
- 3 actors in organic arrangement
- Force-directed layout shows natural relationships
- Thick gold borders emphasize stakeholders
- Spacing shows independence

**Use When**: Presenting to stakeholders, discussing operational context

---

### 📋 **System View** (Select "System")

**What You See**:
```
            [SYS-ACC-001] Following Distance
                    |
            ┌───────┴───────┐
    [SYS-ACC-002]     [SYS-ACC-003]
    Cut-in            Deceleration
         |                  |
    [SYS-ACC-004]     [SYS-ACC-005]
    Override          Speed Range
         |                  |
    [SYS-ACC-006]     [SYS-ACC-007]
    Warnings          Diagnostics
```

**Characteristics**:
- 7 requirements in hierarchical layers
- Top-down flow shows dependencies
- Dashed pink borders indicate requirements
- Safety badges (ASIL ratings) clearly visible
- 100px vertical spacing for clarity

**Use When**: Requirements review, safety analysis, compliance verification

---

### ⚙️ **Logical View** (Select "Logical")

**What You See**:
```
[Radar] → [Fusion] → [Target] → [Controller] → [Actuator]
    ↑                    ↑            ↑
[Camera] ──┘             |            |
                   [Safety Monitor] ──┘
                         ↑
                  [Driver Interface]
                         ↑
                   [Override Mgr]
```

**Characteristics**:
- 9 components in left-to-right pipeline
- Shows data flow: Sensors → Processing → Control → Actuation
- Safety monitors and HMI feed into main flow
- Component hierarchy preserved
- Blue borders (or safety colors) show criticality

**Use When**: Design review, architecture documentation, implementation planning

---

## Switching Between Views

### Step 1: Load Your Architecture
Upload `acc_complete_architecture.arc` and compile

### Step 2: Use the Layer Dropdown

```
[All Layers (19 nodes)] ▼
├─ Operational (3)   ← Force-directed, actor-focused
├─ System (7)        ← Top-down, requirement hierarchy
└─ Logical (9)       ← Left-right, component flow
```

### Step 3: Observe the Differences

**Switch to Operational**:
- Layout rearranges to organic force-directed
- Nodes show thick borders
- Spacing increases
- Focus on actor relationships

**Switch to System**:
- Layout switches to top-down
- Nodes show dashed borders
- Requirements arranged vertically
- Hierarchical dependency structure

**Switch to Logical**:
- Layout switches to left-right
- Nodes show solid borders
- Components arranged horizontally
- Data flow from left to right

## Technical Implementation

### Files Modified

#### 1. **ELK Layout** (`arcviz-web/apps/web/lib/elk/elk-layout.ts`)

**Added**:
- `detectDominantLayer()` function
- `getLayerLayoutOptions()` function
- Three distinct layout configurations

**Lines**: 32-91

#### 2. **Diagram Viewer** (`arcviz-web/apps/web/components/diagram/diagram-viewer.tsx`)

**Added**:
- Conditional node shape rendering
- Actor-specific styling (thick borders)
- Requirement-specific styling (dashed borders)
- Component-specific styling (standard)

**Lines**: 186-255

## Benefits of Layer-Specific Views

### 1. **Cognitive Load Reduction**
- Each view optimized for its purpose
- Visual cues match mental models
- Easier to understand context

### 2. **Professional MBSE Compliance**
- Matches Capella methodology
- Aligns with industry standards
- Proper separation of concerns

### 3. **Better Communication**
- Operational: Stakeholder discussions
- System: Requirements reviews
- Logical: Technical design sessions

### 4. **Improved Navigation**
- Force-directed: Find relationships
- Top-down: Trace dependencies
- Left-right: Follow data flow

## Capella Methodology Alignment

| Capella Diagram | ArcViz Layer | Layout | Purpose |
|-----------------|--------------|---------|---------|
| OAB (Operational Architecture Blank) | Operational | Force | Actor context |
| SAB (System Architecture Blank) | System | Top-Down | Requirements |
| LAB (Logical Architecture Blank) | Logical | Left-Right | Components |
| LDFB (Logical Dataflow Blank) | Logical | Left-Right | Data flows |

## Best Practices

### When to Use Each View

**Use Operational View When**:
- Presenting to non-technical stakeholders
- Discussing use cases and scenarios
- Identifying system boundaries
- Understanding actor interactions

**Use System View When**:
- Conducting requirements reviews
- Performing safety analysis
- Verifying ISO 26262 compliance
- Tracing requirements to design

**Use Logical View When**:
- Reviewing technical architecture
- Planning implementation
- Analyzing data flows
- Discussing component interactions

### View Combinations

**For Complete Understanding**: Start with "All Layers" to see the big picture, then drill into specific layers.

**For Presentations**: 
1. Operational → Show the problem space
2. System → Show what we need to achieve
3. Logical → Show how we'll achieve it

## Status: COMPLETE ✅

Your visualizer now provides:
- ✅ **3 distinct layout algorithms** (force, top-down, left-right)
- ✅ **3 visual styles** (thick/dashed/standard borders)
- ✅ **Layer-specific optimization** (spacing, direction, hierarchy)
- ✅ **Professional MBSE compliance** (Capella-aligned)
- ✅ **Automatic adaptation** (detects dominant layer)

## Test It Now!

```
http://localhost:3002/visualizer
```

1. Upload `acc_complete_architecture.arc`
2. Try each layer filter:
   - **Operational** → See organic actor network
   - **System** → See hierarchical requirements
   - **Logical** → See component pipeline

Each view will have its **own unique layout and style**! 🎨🚀

You were absolutely correct that having the same view for all layers wasn't appropriate. Now each layer tells its own story! 🎉
