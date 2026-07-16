# Capella Diagram Types - Implementation Plan

## Current Status

### ✅ What Works Now
- 3 layers parsed (Operational, System, Logical)
- Layer-specific layouts
- Basic node filtering
- Capella colors

### ❌ What's Missing - The Key Problem You Identified

**The Issue**: All layers show the same generic "Architecture Blank" view. No option to switch between:
- Architecture Blank (structure)
- Dataflow Blank (emphasize data)
- Component Breakdown (tree view)
- etc.

## What Needs To Be Implemented

### 1. Add Diagram Type Selector UI

**Location**: `arcviz-web/apps/web/app/visualizer/page.tsx`

**Add Second Dropdown**:
```tsx
// After layer selector
{selectedLayerFilter !== 'all' && (
  <select
    value={diagramType}
    onChange={(e) => setDiagramType(e.target.value as DiagramType)}
    className="text-sm border rounded px-2 py-1 ml-2"
  >
    {getDiagramTypesForLayer(selectedLayerFilter).map(dt => (
      <option key={dt.value} value={dt.value}>{dt.label}</option>
    ))}
  </select>
)}
```

**Helper Function**:
```tsx
const getDiagramTypesForLayer = (layer: string) => {
  switch (layer) {
    case 'logical':
      return [
        { value: 'lab', label: 'Architecture Blank (LAB)' },
        { value: 'ldfb', label: 'Dataflow Blank (LDFB)' },
        { value: 'lcbd', label: 'Component Breakdown' },
      ]
    case 'system':
      return [
        { value: 'sab', label: 'Architecture Blank (SAB)' },
        { value: 'sdfb', label: 'Dataflow Blank (SDFB)' },
      ]
    // etc.
  }
}
```

### 2. Pass Diagram Type to Layout

**Update DiagramViewer Component**:
```tsx
<DiagramViewer
  graph={filteredGraph}
  diagramType={diagramType}  // NEW
  width={1600}
  height={1200}
  onNodeClick={handleNodeClick}
  onEdgeClick={handleEdgeClick}
/>
```

### 3. Update Diagram Viewer to Accept Diagram Type

**File**: `arcviz-web/apps/web/components/diagram/diagram-viewer.tsx`

```tsx
interface DiagramViewerProps {
  graph: ArchitectureGraph
  diagramType?: DiagramType  // NEW
  width?: number
  height?: number
  onNodeClick?: (nodeId: string) => void
  onEdgeClick?: (edgeId: string) => void
}

export function DiagramViewer({ 
  graph, 
  diagramType,  // NEW
  width = 1200, 
  height = 800, 
  onNodeClick, 
  onEdgeClick 
}: DiagramViewerProps) {
  
  // Pass to layoutGraph
  const layouted = await layoutGraph(graph, diagramType)
  
  // Apply diagram-specific rendering
  if (diagramType === 'ldfb' || diagramType === 'sdfb') {
    // Emphasize arrows, show data labels
    renderDataflowDiagram(layouted)
  } else if (diagramType === 'lcbd' || diagramType === 'scbd') {
    // Tree structure
    renderBreakdownDiagram(layouted)
  } else {
    // Standard architecture blank
    renderArchitectureBlank(layouted)
  }
}
```

### 4. Implement Different Renderers

**Architecture Blank (AB)** - Current style:
- Standard boxes
- Normal connections
- Hierarchical or network layout

**Dataflow Blank (DFB)** - Emphasize flow:
- Thicker arrows (4px instead of 2px)
- Arrow labels showing data types
- Orthogonal routing
- Left-to-right flow emphasized

```tsx
function renderDataflowDiagram(svg, graph, layouted) {
  // Draw edges first and emphasize them
  edges.append('path')
    .attr('stroke-width', 4)  // Thicker
    .attr('marker-end', 'url(#arrow-data-large)')
    
  // Add data type labels on arrows
  edges.append('text')
    .text(edge => edge.dataType || 'data')
    .attr('font-weight', 'bold')
}
```

**Breakdown Diagram (BD)** - Tree view:
- Tree algorithm (mrtree)
- Parent-child connectors
- Indentation
- Collapsible nodes (future)

```tsx
function renderBreakdownDiagram(svg, graph, layouted) {
  // Use tree connectors (L-shaped)
  edges.append('path')
    .attr('d', d => `M${d.startX},${d.startY} L${d.startX},${d.endY} L${d.endX},${d.endY}`)
    .attr('stroke', '#999')
    .attr('stroke-width', 1)
    
  // Add expand/collapse icons
  nodes.append('circle')
    .attr('r', 5)
    .attr('fill', node => node.children ? '#666' : 'none')
}
```

## Implementation Steps

### Phase 1: Basic Infrastructure ✅ (Done)
- [x] Add DiagramType type definition
- [x] Update getDiagramLayoutOptions with all types
- [x] Update layoutGraph to accept diagramType
- [x] Physical & EPBS layer parsing

### Phase 2: UI (TODO - File was corrupted)
- [ ] Restore page.tsx properly
- [ ] Add diagramType state
- [ ] Add getDiagramTypesForLayer helper
- [ ] Add diagram type selector dropdown
- [ ] Pass diagramType to DiagramViewer

### Phase 3: Rendering (TODO)
- [ ] Update DiagramViewer props
- [ ] Implement renderDataflowDiagram
- [ ] Implement renderBreakdownDiagram
- [ ] Keep renderArchitectureBlank as default

### Phase 4: Styling (TODO)
- [ ] Capella-style node shapes with ports
- [ ] Proper arrow styles per type
- [ ] Color coding per Capella standards
- [ ] Add legends

## Visual Differences

### LAB (Logical Architecture Blank)
```
┌──────────────┐      ┌──────────────┐
│  Sensor      │─────>│  Controller  │
│  Fusion      │      │              │
└──────────────┘      └──────────────┘
```
- Standard boxes
- Simple arrows
- Component hierarchy

### LDFB (Logical Dataflow Blank)
```
┌──────────────┐   TargetData   ┌──────────────┐
│  Sensor      │═══════════════>│  Controller  │
│  Fusion      │                │              │
└──────────────┘                └──────────────┘
```
- EMPHASIZED arrows (thick, labeled)
- Data type labels
- Flow-oriented layout

### LCBD (Logical Component Breakdown)
```
Sensor Fusion
  ├─ Radar Processing
  ├─ Camera Processing
  └─ Data Correlation
      ├─ Time Alignment
      └─ Spatial Alignment
```
- Tree structure
- Parent-child lines
- Hierarchical indentation

## Files To Modify

1. ✅ `elk-layout.ts` - Layout algorithms (DONE)
2. ❌ `page.tsx` - Add UI selector (CORRUPTED - NEEDS FIX)
3. ❌ `diagram-viewer.tsx` - Different renderers (TODO)
4. ❌ Add new file: `capella-renderers.ts` - Rendering logic (TODO)

## Immediate Next Steps

**Priority 1: Fix Broken File**
1. Recreate `page.tsx` properly
2. Add diagram type state and selector
3. Test layer + diagram type selection

**Priority 2: Implement Renderers**
1. Keep current rendering as "Architecture Blank"
2. Add Dataflow renderer (thick arrows + labels)
3. Add Breakdown renderer (tree structure)

**Priority 3: Polish**
1. Capella-style node shapes
2. Port visualization
3. Proper legends
4. Color schemes per diagram type

## Expected Result

Users will see:
```
Layer: [Logical ▼]  Diagram: [Dataflow Blank (LDFB) ▼]
```

And get:
- Different layout algorithm
- Different visual style
- Different emphasis (structure vs flow vs hierarchy)

Just like real Capella! 🎯

## Current Blocker

The page.tsx file needs to be properly restored and updated. The infrastructure (layout algorithms) is ready, but UI integration was corrupted during implementation.

Would you like me to:
1. **Carefully restore and fix page.tsx** with diagram selector?
2. **Create a backup first** then implement?
3. **Provide the exact code** to manually add?
