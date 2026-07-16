# Edge Filtering Fix - Layer Isolation

## Problem
When filtering to a specific layer (e.g., "Operational"), edges that referenced nodes from other layers (e.g., LC-001 from Logical layer) were still included, causing:
```
Error: Referenced shape does not exist: LC-001
```

## Root Cause
The edge filtering logic was using OR condition:
```typescript
// WRONG: Includes edges if EITHER endpoint is in the layer
sourceNode?.layer === selectedLayerFilter || targetNode?.layer === selectedLayerFilter
```

This meant:
- Viewing "Operational" layer
- Edge from "Driver" (operational) → "LC-001" (logical) was included
- But "LC-001" node was filtered out
- ELK layout failed because edge referenced non-existent node

## Solution
Changed to AND condition - both endpoints must exist:
```typescript
// CORRECT: Only includes edges where BOTH endpoints exist in filtered nodes
const filteredNodes = graph.nodes.filter(n => n.layer === selectedLayerFilter)
const filteredNodeIds = new Set(filteredNodes.map(n => n.id))

const filteredEdges = graph.edges.filter(e => 
  filteredNodeIds.has(e.source) && filteredNodeIds.has(e.target)
)
```

## What This Means

### Operational Layer (3 actors)
**Before**: Would try to show edges to components → Error  
**After**: Only shows edges between actors → Works perfectly

### System Layer (7 requirements)
**Before**: Would try to show edges to components → Error  
**After**: Only shows requirement-to-requirement edges → Works perfectly

### Logical Layer (9 components)
**Before**: Would try to show edges to requirements → Error  
**After**: Only shows component-to-component edges → Works perfectly

### All Layers
**Before**: Worked fine (all nodes present)  
**After**: Still works fine (no filtering applied)

## Benefits

1. **Clean Layer Isolation**: Each layer view is now truly isolated
2. **No More Errors**: ELK layout always receives valid graphs
3. **Better Understanding**: See relationships within each layer clearly
4. **Cross-Layer Views**: Use "All Layers" to see cross-layer traces

## Visual Impact

**Operational View (3 nodes)**:
- Shows only actor-to-actor interactions
- No edges to components or requirements
- Pure operational context

**System View (7 nodes)**:
- Shows only requirement-to-requirement traces
- Dependency chains within requirements
- Pure system analysis

**Logical View (9 nodes)**:
- Shows only component-to-component flows
- Data flow within logical architecture
- Pure design view

## File Changed
- `arcviz-web/apps/web/app/visualizer/page.tsx` (lines 162-179)

## Status: FIXED ✅

Layer filtering now works correctly. Try switching between layers - each view will render successfully with its appropriate nodes and edges!
