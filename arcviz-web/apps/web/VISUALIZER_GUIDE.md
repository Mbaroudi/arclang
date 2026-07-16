# ArcViz Visualizer - User Guide

## Overview

The ArcViz Visualizer provides interactive, hierarchical architecture diagrams using the Eclipse Layout Kernel (ELK) and D3.js for rendering.

## Features

### 🎨 **Automatic Layout with ELK**
- **Hierarchical Layout**: Components organized in layers
- **Smart Edge Routing**: Automatic path calculation with bend points
- **Port Management**: Fixed port ordering for clean diagrams
- **Layer-based**: Top-down flow from requirements to implementation

### 🎯 **Interactive Elements**

#### Nodes (Components/Requirements/Functions)
- **Click**: Select and view details in side panel
- **Hover**: Highlight with shadow effect
- **Color-coded by Type**:
  - 🔵 Components: Blue
  - 🟣 Functions: Purple
  - 🌸 Requirements: Pink
  - 🟢 Interfaces: Green

#### Safety Level Badges
- **ASIL-D / DAL-A**: 🔴 Red (Highest criticality)
- **ASIL-C / DAL-B**: 🟠 Orange
- **ASIL-B / DAL-C**: 🟡 Yellow
- **ASIL-A / DAL-D**: 🟢 Green
- **QM**: 🔵 Blue (Quality Management)

#### Edges (Relationships)
- **Click**: View relationship details
- **Hover**: Thicken line for emphasis
- **Color-coded by Type**:
  - 🟢 Satisfies: Green (Requirement fulfillment)
  - 🔵 Implements: Blue (Function implementation)
  - 🟣 Realizes: Purple (Architecture realization)
  - ⚫ Data: Gray (Data flow)

### 🔍 **Zoom & Pan**
- **Mouse Wheel**: Zoom in/out
- **Drag**: Pan around diagram
- **Zoom Buttons**: Precise zoom control
- **Fit to View**: Auto-scale to fit entire diagram
- **Zoom Range**: 10% to 400%

### 🎛️ **Toolbar Controls**

#### Layer Selection
Switch between Arcadia views:
- **Operational Analysis**: Actors, capabilities, activities
- **System Analysis**: Requirements, functions
- **Logical Architecture**: Components, interfaces
- **Physical Architecture**: Nodes, deployment
- **EPBS**: Product breakdown structure

#### Filters
Show/hide element types:
- ✅ Requirements
- ✅ Components
- ✅ Functions
- ✅ Interfaces
- ✅ Traces

#### Export Options
- **PNG**: Raster image (high quality)
- **SVG**: Vector graphics (scalable)
- **PDF**: Document format (printable)

### 📊 **Details Panel**

When you click on a node, the side panel shows:
- **Element ID**: Unique identifier
- **Label**: Display name
- **Type**: Component, Function, Requirement, etc.
- **Safety Level**: Criticality classification
- **Description**: Detailed explanation
- **Traceability**: Links to related elements
- **Properties**: Metadata (created, modified, author)

### 🎨 **Visual Design (Capella-style)**

#### Node Structure
```
┌─────────────────────────┐
│ ID-001          [ASIL-B]│
│                         │
│    Component Name       │
│                         │
└─────────────────────────┘
```

#### Color Palette
- **Safety Critical**: Red shades (ASIL-D, DAL-A)
- **High Safety**: Orange/Yellow (ASIL-C/B, DAL-B/C)
- **Low Safety**: Green (ASIL-A, DAL-D)
- **Quality Management**: Blue (QM)

## Keyboard Shortcuts

- **Mouse Wheel**: Zoom
- **Click + Drag**: Pan
- **Click Node**: Select
- **Escape**: Deselect / Close panel

## Use Cases

### 1. Requirements Traceability
- View which components satisfy which requirements
- Follow green "satisfies" edges from components to requirements
- Verify complete coverage

### 2. Safety Analysis
- Identify critical components by color (red = ASIL-D)
- Trace safety-critical paths through the architecture
- Verify safety level consistency

### 3. Architecture Review
- Present architecture to stakeholders
- Export diagrams for documentation
- Filter views for specific audiences

### 4. Impact Analysis
- Select a component to see its connections
- Identify affected elements for changes
- Understand dependency chains

## Tips & Best Practices

### For Better Layouts
1. **Keep hierarchies clear**: Requirements → Functions → Components
2. **Use meaningful IDs**: Helps identify elements quickly
3. **Limit connections**: Too many edges can clutter the view
4. **Group related elements**: Use consistent naming prefixes

### For Presentations
1. **Use Fit to View**: Start with full overview
2. **Zoom to focus areas**: Highlight specific subsystems
3. **Filter by type**: Show only relevant elements
4. **Export to PDF**: For documentation and reports

### For Analysis
1. **Enable all filters**: See complete picture
2. **Follow edge colors**: Understand relationship types
3. **Check safety badges**: Verify criticality levels
4. **Use details panel**: Deep dive into elements

## Performance

- **Recommended**: Up to 200 nodes
- **Maximum**: 1000 nodes (may be slow)
- **Optimization**: Use filters to reduce visible elements
- **Large diagrams**: Consider splitting into multiple views

## Integration

### From Editor
1. Write ArcLang code in Editor
2. Click **"Visualize"** button
3. Diagram opens in new tab
4. Changes sync automatically

### Export Formats
- **PNG**: For presentations and documents
- **SVG**: For further editing in tools like Inkscape
- **PDF**: For reports and printouts
- **Capella XML**: For import into Eclipse Capella
- **Mermaid**: For GitHub/GitLab documentation

## Troubleshooting

### Diagram Not Rendering
- Check console for errors
- Verify ArcLang syntax is valid
- Ensure all node IDs are unique
- Check that edges reference existing nodes

### Layout Issues
- Use **Fit to View** to reset
- Try different layer views
- Reduce number of visible elements with filters
- Reload page if layout is corrupted

### Performance Issues
- Reduce node count (< 200 recommended)
- Use filters to hide unnecessary elements
- Close other browser tabs
- Use Chrome/Edge for best performance

## Next Steps

1. **Edit Architecture**: Click "View in Editor" from details panel
2. **Share Diagram**: Use Share button to collaborate
3. **Export**: Download for documentation
4. **Explore Views**: Switch between Arcadia layers

---

For more help, visit [docs.arcviz.io](https://docs.arcviz.io)
