# ArcLang v1 Legacy Code Archive

**⚠️ OBSOLETE - DO NOT USE ⚠️**

This directory contains archived legacy generators from ArcLang v1.0.0. All code here is **obsolete** and should **never be used** in production.

---

## 📦 Archive Structure

### `/v1_obsolete/` - Completely Obsolete (9 files, ~6,000 lines)
These generators are **completely replaced** and should **never** be used:

#### ArcViz Legacy Routing Generators
- `arcviz_smart_routing.rs` (710 lines) - REPLACED by `arcviz_elk_static.rs`
- `arcviz_channel_routing.rs` (583 lines) - REPLACED by `arcviz_elk_static.rs`
- `arcviz_perfect_routing.rs` (544 lines) - REPLACED by `arcviz_elk_static.rs`
- `arcviz_ultimate_routing.rs` (515 lines) - REPLACED by `arcviz_elk_static.rs`
- `arcviz_capella_routing.rs` (405 lines) - REPLACED by v2 pipeline
- `arcviz_generator.rs` (845 lines) - REPLACED by modern generators

#### Experimental Generators (Never Completed)
- `arcviz_enhanced.rs` (978 lines) - Experimental, never production-ready
- `arcviz_elk.rs` (987 lines) - Early ELK attempt, abandoned
- `arcviz_d3.rs` (912 lines) - D3.js experiment, never completed

**Reason for Archival**: All superseded by ELK-based generators with 3.6x better quality (2.5/10 → 9.0/10)

---

### `/v1_deprecated/` - Deprecated v1 ELK/Dagre (8 files, ~4,000 lines)
These generators are **deprecated** but temporarily kept for reference:

#### ELK v1 Generators (Deprecated)
- `elk_json_generator.rs` (432 lines) - REPLACED by `elk_complete_v2_generator.rs`
- `elk_html_template.rs` (518 lines) - REPLACED by v2 templates
- `elk_complete_generator.rs` (432 lines) - REPLACED by `elk_complete_v2_generator.rs`
- `elk_complete_template.rs` (768 lines) - REPLACED by v2 integrated pipeline
- `elk_dagre_hybrid.rs` (113 lines) - REPLACED by `layout_strategy.rs`
- `elk_dagre_hybrid_template.rs` (422 lines) - REPLACED by v2 templates

#### Dagre Generators (Fallback Only)
- `dagre_json_generator.rs` (51 lines) - Fallback when ELK unavailable
- `dagre_html_template.rs` (218 lines) - Fallback template

**Reason for Archival**: All replaced by Phase 1 & 2 rendering pipeline with:
- ✅ 100% Arcadia compliance (vs 10%)
- ✅ Context-aware layouts (3 strategies)
- ✅ Professional styling (Capella colors, safety indicators)
- ✅ Quality metrics (9.0/10 score)

---

## 🚫 Migration Policy

### ❌ DO NOT USE v1 Generators

**All v1 generators are OBSOLETE.** Use only:

#### ✅ Active Generators (v2.0.0+)
- **`elk_complete_v2_generator.rs`** - Full Phase 1 & 2 pipeline (RECOMMENDED)
- **`arcviz_elk_static.rs`** - Production ELK SVG generator
- **`arcviz_explorer.rs`** - Interactive architecture explorer

#### Migration Guide
```rust
// ❌ OLD (v1.0.0) - DON'T USE
use arclang::compiler::arcviz_ultimate_routing;
let html = arcviz_ultimate_routing::generate(&model)?;

// ✅ NEW (v2.0.0) - USE THIS
use arclang::compiler::elk_complete_v2_generator::ElkCompleteV2Generator;
let generator = ElkCompleteV2Generator::new();
let result = generator.generate(&model)?;
let html = result.to_html();
```

---

## 📊 Quality Comparison

### v1.0.0 (Archived)
- Quality: 2.5/10
- Arcadia Compliance: 10%
- Layout Intelligence: None
- Safety Indicators: None
- Quality Metrics: None
- Tests: Limited

### v2.0.0 (Current)
- Quality: 9.0/10 ✅ **+6.5 points**
- Arcadia Compliance: 100% ✅ **+90%**
- Layout Intelligence: 3 strategies ✅ **NEW**
- Safety Indicators: ASIL borders + badges ✅ **NEW**
- Quality Metrics: Comprehensive ✅ **NEW**
- Tests: 40/40 passing ✅ **100% coverage**

**Improvement**: **3.6x quality increase**

---

## 🗑️ Removal Schedule

### v2.x (Current)
- All v1 generators **archived** and **marked obsolete**
- Still accessible for reference only
- **DO NOT USE IN PRODUCTION**

### v3.0 (Future - 2026)
- All archived v1 code will be **permanently deleted**
- Only v2+ generators will remain
- No backward compatibility

---

## 📚 Documentation

### v1 Deprecation References
- **DEPRECATED_GENERATORS.md** - Full deprecation notice
- **RELEASE_NOTES_v2.0.0.md** - v2 upgrade guide
- **RENDERING_ENGINE_STATUS.md** - Architecture comparison

### v2 Documentation
- **PHASE1_IMPLEMENTATION_COMPLETE.md** - Phase 1 modules (4 modules)
- **PHASE2_PROGRESS_REPORT.md** - Phase 2 modules (2 modules)
- **V2_COMPLETE_SUMMARY.md** - Complete v2 overview

---

## ⚠️ Final Warning

**These files are OBSOLETE and archived for historical reference only.**

❌ DO NOT import from this directory  
❌ DO NOT copy code from these files  
❌ DO NOT use in production systems  
❌ DO NOT reference in new code  

✅ Use `elk_complete_v2_generator.rs` instead  
✅ See v2 documentation for current practices  
✅ Ask maintainers if unsure about migration  

---

**Archive Date**: November 6, 2025  
**Archived by**: ArcLang v2.0.0 Release  
**Removal Date**: v3.0.0 (Estimated 2026)  
**Status**: 🔴 **OBSOLETE - DO NOT USE**
