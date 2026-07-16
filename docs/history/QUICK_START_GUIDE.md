# ArcViz Multi-Layer Visualizer - Quick Start 🚀

## What's Fixed

✅ **Build errors** - Cargo.toml cleaned up  
✅ **File upload** - Upload `.arc` files directly  
✅ **Layout errors** - All nodes parse correctly  
✅ **Multi-layer support** - Operational, System, AND Logical layers  
✅ **Capella styling** - Full color scheme with safety levels  

## Access the Visualizer

**URL**: http://localhost:3002/visualizer

## Upload Your Architecture

1. Click **"Upload .arc File"** button
2. Select: `/Users/malek/Arclang/examples/automotive/acc_complete_architecture.arc`
3. Click **"Compile & Visualize"**

## Switch Between Layers

Look for the dropdown in the header:

```
[All Layers (26 nodes)] ▼
```

### View Options:
- **All Layers** → See complete architecture (26 nodes)
- **Operational** → See 3 actors (Driver, Lead Vehicle, Environment)
- **System** → See 7 requirements (SYS-ACC-001 to SYS-ACC-007)
- **Logical** → See 16 components + functions (LC-001 to LC-009)

## Node Colors (Capella Style)

| Color | Meaning |
|-------|---------|
| 🟡 Yellow | Actors (operational) |
| 🩷 Pink | Requirements (system) |
| 🔵 Blue | Components (logical) |
| 💜 Indigo | Functions (logical) |
| 🔴 Red | ASIL_D / Critical safety |
| 🟠 Orange | ASIL_C / High safety |
| 🟡 Yellow | ASIL_B / Medium safety |
| 🟢 Green | ASIL_A / Low safety |

## Your ACC Architecture Breakdown

### 📍 Operational Layer (3 nodes)
- ACT-001: Driver
- ACT-002: Lead Vehicle  
- ACT-003: Environment

### 📋 System Layer (7 nodes)
- SYS-ACC-001: Following distance (ASIL_B)
- SYS-ACC-002: Cut-in detection (ASIL_B)
- SYS-ACC-003: Deceleration limit (ASIL_B)
- SYS-ACC-004: Brake override (ASIL_C)
- SYS-ACC-005: Speed range (ASIL_A)
- SYS-ACC-006: Warnings (ASIL_A)
- SYS-ACC-007: Diagnostics (ASIL_B)

### 🔧 Logical Layer (16 nodes)
**Components**:
- LC-001: Long Range Radar
- LC-002: Forward Camera
- LC-003: Sensor Fusion
- LC-004: Target Selection
- LC-005: Longitudinal Controller
- LC-006: Actuator Command
- LC-007: Safety Monitor
- LC-008: Driver Interface
- LC-009: Override Manager

**Functions**: LF-001 to LF-025 (nested in components)

## Interaction Guide

- **Click** nodes → See details panel
- **Hover** → View tooltips
- **Mouse wheel** → Zoom in/out
- **Drag** → Pan diagram
- **Dropdown** → Switch layers

## Files Changed

1. `Cargo.toml` - Removed missing test binaries
2. `arcviz-web/apps/api/src/services/compiler.ts` - Multi-layer parsing
3. `arcviz-web/apps/web/lib/elk/elk-layout.ts` - Actor colors
4. `arcviz-web/apps/web/app/visualizer/page.tsx` - Layer filtering UI

## Documentation

- **Complete details**: `MULTI_LAYER_SUPPORT_COMPLETE.md`
- **Layout fix**: `LAYOUT_ERROR_FIXED.md`
- **Initial setup**: `ARCVIZ_FIXES_COMPLETE.md`

## Ready to Explore!

Everything is configured and running. Just open the visualizer and upload your ACC architecture to see all three layers with full Capella styling! 🎉
