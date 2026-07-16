# ArcLang Diagram Service

Capella-quality diagram rendering service for ArcLang models.

## Overview

This service consumes JSON exports from the ArcLang compiler and renders professional MBSE diagrams matching Capella's visual quality. It supports all Arcadia methodology diagram types.

## Features

### Supported Diagram Types

1. **Operational Activity Diagrams (OAD)**
   - Swimlane layout
   - Actor stick figures
   - Yellow activity boxes with icons
   - Data flow arrows with labels
   - Hierarchical activity decomposition

2. **Capability Decomposition**
   - Hierarchical tree layout
   - Mission/Capability/Sub-capability levels
   - Capability associations

3. **Functional Dataflow Diagrams (SFBD)**
   - Green function boxes
   - Functional ports (small squares on borders)
   - Port-to-port connections
   - Function hierarchy
   - External actors

4. **Component Block Diagrams (LAB)**
   - Blue component boxes
   - Nested components
   - Allocated functions (green boxes inside)
   - Component ports
   - Dashed green boxes for unallocated functions

5. **Physical Deployment Diagrams (PAB)**
   - Yellow hardware node boxes
   - Blue behavior components (software)
   - Gray hardware components
   - Physical links with protocols
   - Data exchanges

6. **Sequence Diagrams**
   - Lifelines with participant boxes
   - Synchronous/asynchronous/return messages
   - Combined fragments (PAR, OPT, LOOP, ALT)
   - Activation boxes
   - Timing constraints

7. **State Machine Diagrams**
   - States with entry/exit actions
   - Transitions with triggers and guards
   - Composite states (sub-states)
   - Initial and final states
   - Colored states

## Architecture

```
diagram-service/
├── src/
│   ├── types/           # TypeScript type definitions
│   │   ├── model.ts     # ArcLang model types (from JSON)
│   │   └── diagram.ts   # Diagram rendering types
│   ├── renderers/       # Diagram renderers (one per type)
│   │   ├── operational.ts
│   │   ├── functional.ts
│   │   ├── component.ts
│   │   ├── physical.ts
│   │   ├── sequence.ts
│   │   └── state-machine.ts
│   ├── layouts/         # Layout algorithms
│   │   ├── swimlane.ts  # For operational diagrams
│   │   ├── hierarchical.ts  # For component/function trees
│   │   └── timeline.ts  # For sequence diagrams
│   ├── utils/           # Utilities
│   │   ├── svg.ts       # SVG generation helpers
│   │   └── geometry.ts  # Geometric calculations
│   └── index.ts         # Main entry point
├── package.json
├── tsconfig.json
└── README.md
```

## Installation

```bash
cd arcviz-web/apps/diagram-service
npm install
```

## Usage

### Basic Usage

```typescript
import { Model } from '@arcviz/diagram-service';
import { renderOperationalActivity } from '@arcviz/diagram-service/renderers/operational';
import fs from 'fs';

// Load ArcLang JSON export
const model: Model = JSON.parse(fs.readFileSync('model.json', 'utf-8'));

// Render operational activity diagram
for (const oa of model.operational_analysis) {
  const svg = await renderOperationalActivity(oa, {
    width: 1200,
    height: 800,
    colorScheme: CAPELLA_COLORS,
  });
  
  fs.writeFileSync(`${oa.name}.svg`, svg.svg);
}
```

### Rendering All Diagram Types

```typescript
import { Model, DiagramOutput } from '@arcviz/diagram-service';
import * as renderers from '@arcviz/diagram-service/renderers';

async function renderAllDiagrams(modelPath: string, outputDir: string): Promise<void> {
  const model: Model = JSON.parse(fs.readFileSync(modelPath, 'utf-8'));
  
  // Operational Activity Diagrams
  for (const oa of model.operational_analysis) {
    const svg = await renderers.renderOperationalActivity(oa);
    fs.writeFileSync(`${outputDir}/${oa.name}_OAD.svg`, svg.svg);
  }
  
  // Functional Dataflow Diagrams
  for (const sa of model.system_analysis) {
    const svg = await renderers.renderFunctionalDataflow(sa);
    fs.writeFileSync(`${outputDir}/${sa.name}_SFBD.svg`, svg.svg);
  }
  
  // Component Block Diagrams
  for (const la of model.logical_architecture) {
    const svg = await renderers.renderComponentBlock(la);
    fs.writeFileSync(`${outputDir}/${la.name}_LAB.svg`, svg.svg);
  }
  
  // Physical Deployment Diagrams
  for (const pa of model.physical_architecture) {
    const svg = await renderers.renderPhysicalDeployment(pa);
    fs.writeFileSync(`${outputDir}/${pa.name}_PAB.svg`, svg.svg);
  }
  
  // Sequence Diagrams
  for (const scenario of model.scenarios) {
    const svg = await renderers.renderSequence(scenario);
    fs.writeFileSync(`${outputDir}/${scenario.name}_SD.svg`, svg.svg);
  }
  
  // State Machine Diagrams
  for (const sm of model.state_machines) {
    const svg = await renderers.renderStateMachine(sm);
    fs.writeFileSync(`${outputDir}/${sm.name}_SM.svg`, svg.svg);
  }
}
```

## Configuration

### Layout Configuration

```typescript
import { SwimlaneLayoutConfig } from '@arcviz/diagram-service';

const layoutConfig: SwimlaneLayoutConfig = {
  direction: 'RIGHT',
  swimlaneWidth: 200,
  swimlaneSpacing: 20,
  activityHeight: 80,
  activityWidth: 150,
  nodeSpacing: 40,
  layerSpacing: 100,
  padding: {
    top: 40,
    right: 40,
    bottom: 40,
    left: 40,
  },
};
```

### Rendering Configuration

```typescript
import { RenderConfig, CAPELLA_COLORS } from '@arcviz/diagram-service';

const renderConfig: RenderConfig = {
  width: 1600,
  height: 1200,
  backgroundColor: '#FFFFFF',
  fontSize: 12,
  fontFamily: 'Arial, sans-serif',
  showGrid: false,
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
};
```

## Color Scheme

Matches Capella's default color scheme:

| Element Type | Color | Hex |
|--------------|-------|-----|
| Operational Activities | Yellow | #FFD966 |
| Actors | Blue | #2E75B6 |
| System Functions | Green | #70AD47 |
| Logical Components | Blue | #5B9BD5 |
| Physical Nodes | Yellow | #FFE699 |
| Behavior Components | Blue | #5B9BD5 |
| Hardware Components | Gray | #C0C0C0 |
| Capabilities | Orange | #FFC000 |
| States | Light Blue | #BDD7EE |

## Layout Algorithms

### 1. Swimlane Layout (Operational Diagrams)

- Horizontal lanes for each actor/entity
- Activities placed in corresponding lanes
- Automatic lane sizing
- Flow arrows between activities

### 2. Hierarchical Layout (Functions & Components)

- Top-down or left-right hierarchy
- Port positioning on node borders
- Port-to-port connection routing
- Nested component visualization
- Uses ELK (Eclipse Layout Kernel)

### 3. Timeline Layout (Sequence Diagrams)

- Vertical lifelines
- Chronological message ordering
- Fragment boxes (PAR, OPT, LOOP, ALT)
- Activation bar rendering

## Dependencies

- **elkjs**: Eclipse Layout Kernel for hierarchical layouts
- **d3**: SVG manipulation and rendering
- **dagre**: Graph layout for specific diagram types

## Development

### Build

```bash
npm run build
```

### Watch Mode

```bash
npm run dev
```

### Testing

```bash
npm test
```

### Linting

```bash
npm run lint
```

## Implementation Status

### ✅ Phase 1: Foundation (COMPLETE)
- [x] Project structure created
- [x] TypeScript types defined
- [x] Package configuration
- [x] Type definitions for all Capella structures

### ⏳ Phase 2: Core Renderers (Next)
- [ ] Operational Activity Diagram renderer
- [ ] Functional Dataflow Diagram renderer
- [ ] Component Block Diagram renderer
- [ ] Physical Deployment Diagram renderer

### ⏳ Phase 3: Advanced Diagrams
- [ ] Sequence Diagram renderer
- [ ] State Machine Diagram renderer

### ⏳ Phase 4: Layout Algorithms
- [ ] Swimlane layout implementation
- [ ] Hierarchical layout (ELK integration)
- [ ] Timeline layout (sequence diagrams)

### ⏳ Phase 5: SVG Utilities
- [ ] SVG element generators
- [ ] Path routing algorithms
- [ ] Port connection logic

## Integration with ArcViz Web

The diagram-service will be consumed by the ArcViz web application:

```typescript
// In arcviz-web/apps/web
import { renderOperationalActivity } from '@arcviz/diagram-service';

export async function generateDiagram(modelJson: string, diagramType: string) {
  const model = JSON.parse(modelJson);
  
  switch (diagramType) {
    case 'operational':
      return await renderOperationalActivity(model.operational_analysis[0]);
    // ... other diagram types
  }
}
```

## Future Enhancements

- Export to PNG/PDF
- Interactive diagrams (clickable elements)
- Zoom and pan controls
- Diagram filtering (show/hide elements)
- Custom color schemes
- Diagram comparison (diff visualization)
- Animation (state transitions, message flow)

## License

MIT

## Contributors

ArcLang Contributors

---

**Status**: Foundation Complete (Phase 1) ✅  
**Next**: Implement Operational Activity Diagram renderer (Task #5)
