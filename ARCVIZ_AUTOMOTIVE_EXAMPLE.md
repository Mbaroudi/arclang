# ArcViz Automotive Example - Adaptive Cruise Control System

## Overview

This document showcases ArcViz's Capella-style visualization for automotive systems engineering, using the Adaptive Cruise Control (ACC) system as an example.

## System Architecture Visualization

### Generated Output

The ArcViz visualization includes:

**1. Requirements View** (Capella-Style)
- Organized by safety categories
- Color-coded by priority
- ASIL badges for safety levels
- Full traceability visualization

**2. Component Architecture**
- Logical components with ports
- Data flow connections
- Function allocations
- Hierarchical organization

## Visual Elements

### Requirements Section

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Functional Safety                                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ [SYS-ACC-001] The ACC system shall maintain minimum 2-second following      │
│               distance at all speeds                                 [ASIL-B]│
│ Priority: ● Critical                                                         │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ Performance                                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│ [SYS-ACC-002] The system shall detect cut-in vehicles within 500ms  [ASIL-B]│
│ Priority: ● High                                                             │
│                                                                              │
│ [SYS-ACC-003] Maximum deceleration shall not exceed 3.5 m/s²        [ASIL-B]│
│ Priority: ● High                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ Safety Override                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ [SYS-ACC-004] Driver brake input shall immediately override ACC      [ASIL-C]│
│               control                                                        │
│ Priority: ● Critical                                                         │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ Operational Range                                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ [SYS-ACC-005] System shall operate in speed range 30-180 km/h       [ASIL-A]│
│ Priority: ● Medium                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Color Scheme (Capella-Inspired)

### Priority Indicators
- 🔴 **Critical**: Red border (#d32f2f), light red background
- 🟠 **High**: Orange border (#f57c00), light orange background
- 🔵 **Medium**: Blue border (#1976d2), light blue background
- ⚪ **Low**: Gray border (#9e9e9e), light gray background

### ASIL Safety Levels
- 🔴 **ASIL-D**: Dark red badge (#b71c1c)
- 🔴 **ASIL-C**: Red badge (#d32f2f)
- 🟠 **ASIL-B**: Orange badge (#f57c00)
- 🟡 **ASIL-A**: Yellow badge (#fbc02d)

### Component Types
- 🔵 **Logical Components**: Blue gradient (#e3f2fd → #bbdefb)
- 🟢 **Functions**: Green gradient (#e8f5e9 → #c8e6c9)
- 🟡 **Requirements**: Orange gradient (#fff3e0 → #ffe0b2)

## Interactive Features

### User Controls

1. **🔍 Zoom In/Out**
   - Click buttons or use mouse wheel
   - Smooth scaling transformation

2. **🖱️ Pan & Drag**
   - Click and drag to navigate
   - Large diagrams fully explorable

3. **↻ Reset View**
   - Return to initial view
   - Reset zoom and position

4. **💾 Export SVG**
   - Save standalone vector graphic
   - Print-ready quality

## Comparison with Capella

| Feature | ArcViz | Capella Studio |
|---------|--------|----------------|
| **Setup** | ✅ Browser only | ❌ Eclipse + Java + Plugins |
| **Generation Time** | ✅ < 1 second | ❌ Manual drawing |
| **File Size** | ✅ ~50KB HTML | ❌ Large workspace |
| **Sharing** | ✅ Email HTML file | ❌ Screenshots/PDF export |
| **Updates** | ✅ Regenerate instantly | ❌ Manual sync |
| **Traceability** | ✅ Auto-generated | ❌ Manual connections |
| **ASIL Support** | ✅ Built-in badges | ✅ Manual labels |
| **Color Coding** | ✅ Automatic | ❌ Manual styling |
| **Interactive** | ✅ Full zoom/pan | ❌ Static views |
| **CI/CD** | ✅ CLI command | ❌ GUI-based |

## Architecture Components View

The ACC system includes these logical components:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    ACC Logical Architecture                               │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────┐      ┌──────────────┐      ┌─────────────────┐        │
│  │  Radar      │──────▶│  Fusion      │──────▶│  Speed          │        │
│  │  Sensor     │      │  Engine      │      │  Controller     │        │
│  └─────────────┘      └──────────────┘      └─────────────────┘        │
│       │                                              │                   │
│       │                                              ▼                   │
│  ┌─────────────┐                            ┌─────────────────┐        │
│  │  Camera     │                            │  Brake          │        │
│  │  Sensor     │                            │  Actuator       │        │
│  └─────────────┘                            └─────────────────┘        │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Usage

### Generate Visualization

```bash
# From your ArcLang model
arclang export examples/automotive/adaptive_cruise_control.arc \
    -o acc_architecture.html \
    -f arc-viz

# Open in browser
open acc_architecture.html
```

### Output Preview

The generated HTML file contains:

1. **Title Bar** 
   - System name: "Adaptive Cruise Control"
   - Subtitle: "ArcViz | Capella Style"

2. **Requirements Section**
   - 5 requirements grouped by category
   - Each with ASIL badge, priority indicator
   - Full descriptions visible

3. **Component Architecture** (if available)
   - Logical components with interfaces
   - Data flow arrows
   - Port annotations

4. **Interactive Controls**
   - Zoom/Pan/Reset buttons
   - Export SVG button
   - Keyboard shortcuts

5. **Footer Information**
   - Tool branding
   - Usage instructions

## Professional Use Cases

### 1. Safety Reviews
- ASIL levels clearly visible
- Critical requirements highlighted
- Traceability to components shown

### 2. Stakeholder Presentations
- Clean, professional appearance
- No installation required
- Interactive exploration

### 3. Documentation
- Embed in wikis/Confluence
- Export to PDF for reports
- Version control friendly (text-based)

### 4. CI/CD Integration
- Auto-generate on commit
- Deploy to web server
- Regression testing visuals

### 5. Requirements Management
- Visual verification of coverage
- Gap analysis through color coding
- Dependency analysis

## Real-World Example Output

The actual ArcViz output for ACC includes:

**File**: `acc_architecture.html` (self-contained)
**Size**: ~45KB
**Content**:
- SVG graphics with embedded styles
- JavaScript for interactivity
- No external dependencies
- Works offline

**Visual Quality**:
- ✅ Professional shadows and gradients
- ✅ Crisp typography (Segoe UI, Consolas)
- ✅ Color-coded elements
- ✅ Clean spacing and alignment
- ✅ Print-ready quality

## Advanced Features

### Traceability Arrows

Requirements connected to components:
```
[SYS-ACC-001] ────────▶ [Fusion Engine]
                        (satisfies)

[SYS-ACC-004] ────────▶ [Brake Actuator]
                        (implements)
```

### Multi-Layer Views

- Requirements layer
- Logical architecture layer
- Physical architecture layer (future)
- Behavioral view layer (future)

### Export Options

```bash
# Interactive HTML (default)
arclang export model.arc -o diagram.html -f arc-viz

# Also supports:
# - Mermaid (for GitHub)
# - PlantUML (for UML tools)
# - Capella XML (for Capella Studio)
# - JSON (for custom tools)
```

## Best Practices

### 1. Categorize Requirements
```arc
requirement "SYS-ACC-001" {
    category: "Functional Safety"  // Groups visually
    priority: "Critical"            // Color coding
    safety_level: "ASIL_B"         // Badge display
}
```

### 2. Use Descriptive IDs
```arc
// Good
SYS-ACC-001  // System-ACC-sequential

// Avoid
REQ-1        // Too generic
```

### 3. Add Traceability
```arc
trace "SYS-ACC-001" satisfies "REQ-SAFETY-100" {
    rationale: "Following distance ensures safety"
}
```

### 4. Organize Components
```arc
component "Fusion Engine" {
    type: "Logical"
    category: "Data Processing"
}
```

## Conclusion

**ArcViz provides Capella-quality system architecture visualizations with:**

✅ **Zero setup** - just a browser
✅ **Instant generation** - one command
✅ **Professional styling** - Capella-inspired
✅ **Full interactivity** - zoom, pan, explore
✅ **Standards compliant** - ISO 26262, ASIL support
✅ **Automation ready** - CI/CD friendly

**Perfect for**: Automotive, Aerospace, Defense, Medical Devices, and any safety-critical systems engineering project.

---

**Next Steps**: 
1. Open `acc_architecture.html` in your browser
2. Try zooming and panning
3. Export the SVG
4. Share with your team!
