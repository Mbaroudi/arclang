# Editor & Visualizer - User Guide

## How It Works Now

### Editor (http://localhost:3002/editor)
1. **Opens with default code** - Full Autonomous Emergency Braking System example
2. **Auto-saves** - Code automatically saves to browser localStorage after 1 second
3. **Manual save** - Click "Save" button or use Ctrl+S
4. **Edit freely** - All changes are persisted automatically

### Visualizer (http://localhost:3002/visualizer?from=editor)
1. **Reads your code** - Automatically loads from localStorage
2. **Auto-generates** - When opened from editor (`?from=editor`), automatically generates all diagrams
3. **Dynamic parsing** - Uses ArcLangParser to transform your code into diagrams
4. **10 diagram types** - Generates all Capella diagrams from your code

## Workflow

### Step 1: Edit Your Code
```
http://localhost:3002/editor
```
- Write or modify ArcLang code
- Code auto-saves every 1 second
- No need to click "Save" (but you can)

### Step 2: Visualize
Click the **"Visualize"** button (eye icon) or navigate to:
```
http://localhost:3002/visualizer?from=editor
```

### Step 3: View Diagrams
- Diagrams generate automatically
- Wait ~10 seconds for all 10 types
- View in grid or list layout
- Click any diagram to enlarge
- Export as SVG

## Current Code in Editor

The default code includes:
- **3 Actors**: Driver, Pedestrian, Vehicle
- **4 Requirements**: SYS-001 to SYS-004 (ASIL-D safety critical)
- **6 Components**: Radar, Camera, Sensor Fusion, Risk Analyzer, Brake Controller, Driver Interface
- **12 Functions**: LF-001 to LF-012
- **3 Physical Nodes**: Sensor ECU, CPU, Brake Control Unit
- **Complete traceability**: All trace relationships defined

## Diagram Types Generated

From your code, the following diagrams are generated:

1. **Operational Activity** - Shows actors and their activities
2. **Functional Dataflow** - Function connections and data flow
3. **Component Architecture** - Component structure and interfaces
4. **Sequence Diagram** - Interaction sequences
5. **State Machine** - State transitions (currently uses sample)
6. **Physical Architecture** - Hardware deployment
7. **Class Diagram** - Component relationships
8. **Tree Diagram** - Hierarchical structure  
9. **Capability Diagram** - Requirements as capabilities
10. **Functional Chain** - Function execution chains

## Working Diagram Types (4/10)

✅ **Fully Working with Dynamic Code**:
- Operational Activity
- Tree Diagram
- Capability Diagram
- Functional Chain

⚠️ **Partially Working** (need parser improvements):
- Functional Dataflow
- Component Architecture
- Sequence Diagram
- Physical Architecture
- Class Diagram

❌ **Using Sample Data**:
- State Machine (hardcoded)

## Test Your Own Code

### Example 1: Simple Smart Home
```arclang
operational_analysis "Smart Home" {
  actor "Homeowner" {
    id: "ACT-001"
    description: "Smart home user"
  }
  
  actor "Energy Provider" {
    id: "ACT-002"
    description: "Utility company"
  }
}

system_analysis "Requirements" {
  requirement "REQ-001" {
    id: "REQ-001"
    description: "System shall control lighting"
    priority: "High"
  }
}

logical_architecture "Components" {
  component "Light Controller" {
    id: "LC-001"
    type: "Logical"
    description: "Controls smart lights"
    
    function "Turn On" {
      id: "LF-001"
      description: "Activate lights"
    }
  }
}

trace "LC-001" satisfies "REQ-001" {
  rationale: "Light controller implements lighting requirement"
}
```

### Example 2: Automotive System
```arclang
operational_analysis "Vehicle Control" {
  actor "Driver" {
    id: "ACT-001"
    description: "Vehicle operator"
  }
}

system_analysis "Safety Requirements" {
  requirement "SYS-001" {
    id: "SYS-001"
    description: "System shall brake within 2 seconds"
    priority: "Critical"
    safety_level: "ASIL_D"
  }
}

logical_architecture "Brake System" {
  component "Brake ECU" {
    id: "LC-001"
    type: "Logical"
    safety_level: "ASIL_D"
    
    function "Apply Brake" {
      id: "LF-001"
      description: "Activate braking system"
    }
  }
}

physical_architecture "Hardware" {
  node "Brake Control Unit" {
    id: "PN-001"
    description: "Physical brake controller"
  }
}

trace "LC-001" satisfies "SYS-001" {
  rationale: "Brake ECU implements braking requirement"
}
```

## Troubleshooting

### "Diagrams are empty or showing sample data"
**Cause**: Your code might not have enough elements for certain diagram types.

**Solution**: 
1. Make sure you have all required sections:
   - `operational_analysis` with actors
   - `system_analysis` with requirements  
   - `logical_architecture` with components
   - `physical_architecture` with nodes
   - `trace` statements

2. Check API logs:
   ```bash
   tail -f /tmp/api-server.log
   ```

3. Look for parser messages like:
   ```
   Operational model: 3 actors, 3 entities, 3 activities
   ```

### "Editor not saving changes"
**Cause**: Auto-save waits 1 second after you stop typing.

**Solution**: 
- Wait 1-2 seconds after editing
- Or click "Save" button manually
- Or use Ctrl+S keyboard shortcut

### "Visualizer shows old code"
**Cause**: Browser cache or didn't wait for auto-save.

**Solution**:
1. Click "Save" in editor
2. Wait 2 seconds
3. Then navigate to visualizer
4. Or hard refresh visualizer: Ctrl+Shift+R

### "Some diagrams fail to generate"
**Cause**: Parser for that diagram type needs improvement.

**Solution**: This is expected - only 4/10 diagram types fully work with dynamic code parsing. The rest use sample data as fallback.

## API Endpoints

If you want to test the API directly:

### Generate Single Diagram
```bash
curl -X POST http://localhost:4001/api/diagrams/generate \
  -H "Content-Type: application/json" \
  -d '{
    "diagramType": "operational",
    "code": "operational_analysis \"Test\" { actor \"User\" { id: \"ACT-001\" } }"
  }'
```

### Generate All Diagrams
```bash
curl -X POST http://localhost:4001/api/diagrams/generate-all \
  -H "Content-Type: application/json" \
  -d '{
    "code": "operational_analysis \"Test\" { actor \"User\" { id: \"ACT-001\" } }"
  }'
```

### Get Available Diagram Types
```bash
curl http://localhost:4001/api/diagrams/types
```

## Next Steps

To improve diagram generation:
1. Enhance parsers for remaining 6 diagram types
2. Add more test examples in different domains
3. Improve traceability extraction
4. Add validation and error messages
5. Show parsing feedback in UI

## Quick Reference

| Action | Method |
|--------|--------|
| Edit code | Type in Monaco editor |
| Save manually | Click "Save" or Ctrl+S |
| Auto-save | Wait 1 second |
| Visualize | Click "Visualize" button |
| View diagrams | Navigate to visualizer |
| Export diagram | Click diagram → Export |
| Switch layout | Grid/List buttons |
| Enlarge diagram | Click diagram card |

## URLs

- **Editor**: http://localhost:3002/editor
- **Visualizer**: http://localhost:3002/visualizer?from=editor
- **API**: http://localhost:4001
- **API Docs**: http://localhost:4001/api/diagrams/types
