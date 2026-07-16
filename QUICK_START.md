# ELK Advanced Features - Quick Start

## 🚀 5-Minute Quick Start

### 1. Export a Diagram
```bash
cd /Users/malek/Arclang
arclang export examples/automotive/adaptive_cruise_control.arc \
  -o my_diagram.html \
  -f arc-viz-elk-advanced
```

### 2. Open in Browser
```bash
open my_diagram.html
```

### 3. Interact with Diagram
- **Zoom**: Mouse wheel
- **Pan**: Click and drag background
- **Move nodes**: Toggle "Interactive Mode" and drag nodes
- **Re-layout**: Click "Re-layout" button
- **Fit view**: Click "Fit to View" button

---

## ✨ Key Features

### North/South Ports
Just name your interfaces with directional hints:

```arc
interface_in "North Sensor Data" { }   // → Purple port on top
interface_out "South Debug Port" { }    // → Red port on bottom
```

### Automatic Port Colors
- 🟢 **WEST** (input) = Green
- 🟠 **EAST** (output) = Orange
- 🟣 **NORTH** = Purple
- 🔴 **SOUTH** = Red

### Different Layouts Per Level
```
OA Level → Flows DOWN (operational)
LA Level → Flows RIGHT (logical data flow)
PA Level → Flows DOWN (physical deployment)
```

---

## 📊 Spacing Configuration

Current optimal settings (auto-applied):

| What | Value | Why |
|------|-------|-----|
| Node spacing | 120 | Clear separation |
| Layer spacing | 150 | Distinct levels |
| Edge-node gap | 60 | No overlaps |
| Min component width | 250 | Room for labels |
| Min component height | 200 | Room for ports |

---

## 🎨 Interactive Controls

```
┌─────────────────────────────────────────┐
│ [Re-layout] [Fit to View]              │
│ ☑ Interactive Mode                      │
│ ☑ Show Ports                            │
│ ☑ Show Labels                           │
└─────────────────────────────────────────┘
```

- **Re-layout**: Recompute positions
- **Fit to View**: Auto-zoom to fit
- **Interactive Mode**: Enable drag-and-drop
- **Show Ports**: Toggle port visibility
- **Show Labels**: Toggle all labels

---

## 📝 Example .arc File

```arc
logical_architecture "My System" {
    component "Sensor" {
        id: "LA-001"
        type: "Hardware"
        level: "LA"
        
        interface_in "North Sensors" {
            protocol: "SPI"
        }
        
        interface_out "Data Output" {
            protocol: "CAN"
        }
    }
    
    component "Controller" {
        id: "LA-002"
        type: "Software"
        level: "LA"
        
        interface_in "Sensor Input" {
            protocol: "CAN"
        }
        
        interface_out "Commands" {
            protocol: "CAN"
        }
    }
}
```

---

## 🔧 Build & Export

```bash
# Build compiler
cargo build --release

# Export diagram
./target/release/arclang export model.arc -o output.html -f arc-viz-elk-advanced

# View
open output.html
```

---

## ✅ Feature Checklist

- ✅ Hierarchical layout (different algorithms per level)
- ✅ North/South ports (4-sided positioning)
- ✅ Port margins (clean spacing)
- ✅ Mixed directions (per-level flow)
- ✅ Interactive constraints (drag-and-drop)
- ✅ Auto-sizing (components scale to content)
- ✅ No overlaps (optimal spacing)
- ✅ Zoom/pan (smooth navigation)
- ✅ Re-layout (on-demand refresh)
- ✅ Standalone HTML (no server needed)

---

## 📚 Documentation

- **Complete Guide**: `ELK_COMPLETE_GUIDE.md`
- **Implementation**: `ELK_ADVANCED_IMPLEMENTATION_SUMMARY.md`
- **Spacing Fix**: `ELK_SPACING_FIX.md`
- **Examples**: `examples/automotive/adaptive_cruise_control.arc`

---

## 🎯 Use Cases

### Best For:
- ✅ Capella architectural diagrams
- ✅ Multi-level system architectures (OA/LA/PA)
- ✅ Data flow diagrams
- ✅ Component interfaces
- ✅ Hardware/software allocation

### Visual Quality:
- Professional Capella-style appearance
- Clean orthogonal edge routing
- Color-coded ports and algorithms
- Interactive exploration
- Export-ready diagrams

---

## 💡 Pro Tips

1. **Long Component Names**: Component width auto-adjusts
2. **Many Ports**: Component height auto-adjusts (30px per port)
3. **Directional Ports**: Use "north", "south", "top", "bottom" in names
4. **Clean Layout**: Keep 10-15 components per level
5. **Interactive Mode**: Drag nodes, then re-layout to lock positions

---

## 🚀 That's It!

You now have production-ready Capella-quality diagrams with:
- Full ELK feature support
- Interactive controls
- Optimal spacing
- Professional appearance

**Start creating diagrams**: `arclang export model.arc -o diagram.html -f arc-viz-elk-advanced`
