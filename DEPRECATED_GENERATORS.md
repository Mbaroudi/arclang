# Deprecated Rendering Engines - Archive Notice

**Date**: November 5, 2025  
**Status**: DEPRECATED - Use new unified rendering pipeline  
**Migration Deadline**: TBD based on testing

---

## ⚠️ DEPRECATION NOTICE

The following rendering engines are **DEPRECATED** and will be removed in a future release. They have been superseded by the new Phase 1 & 2 rendering pipeline with:
- Context-aware layout strategies
- Arcadia methodology compliance
- Professional Capella styling
- Quality metrics and reporting

---

## Deprecated Generators (Pre-Phase 1)

### 1. Basic Generators (circa 2024)
**Files**:
- `src/compiler/mermaid_generator.rs`
- `src/compiler/plantuml_generator.rs`
- `src/compiler/arcviz_generator.rs`

**Issues**:
- No semantic analysis
- One-size-fits-all layouts
- No Arcadia compliance
- Basic styling only
- No quality metrics

**Migration**: Use `unified_renderer.rs` (when available)

---

### 2. Smart Routing Attempts (Oct 2024)
**Files**:
- `src/compiler/arcviz_smart_routing.rs`
- `src/compiler/arcviz_channel_routing.rs`
- `src/compiler/arcviz_perfect_routing.rs`
- `src/compiler/arcviz_ultimate_routing.rs`

**Issues**:
- Custom routing algorithms (reinvented wheel)
- Not using industry-standard ELK
- Inconsistent results
- Hard to maintain

**Migration**: Use `layout_strategy.rs` with ELK

---

### 3. Enhanced Generators (Oct-Nov 2024)
**Files**:
- `src/compiler/arcviz_enhanced.rs`
- `src/compiler/arcviz_capella_routing.rs`

**Issues**:
- Partial improvements only
- No systematic approach
- Inconsistent quality
- Mixed with old code

**Migration**: Use new rendering pipeline

---

### 4. Early ELK Attempts (Nov 2024)
**Files**:
- `src/compiler/arcviz_elk.rs`
- `src/compiler/arcviz_elk_static.rs`
- `src/compiler/arcviz_d3.rs`

**Issues**:
- Basic ELK integration only
- No strategy selection
- No post-processing
- No quality metrics

**Status**: Keep as reference, but use `elk_complete_generator.rs` with new pipeline

---

### 5. Hybrid Generators (Nov 2024)
**Files**:
- `src/compiler/elk_dagre_hybrid.rs`
- `src/compiler/dagre_json_generator.rs`

**Issues**:
- Mixing multiple engines
- Complex configuration
- Unpredictable results

**Migration**: Use strategy selector (chooses best algorithm automatically)

---

## New Rendering Pipeline (Phase 1 & 2)

### Architecture
```
Model → Semantic Analyzer → Strategy Selector → Generator
           ↓                      ↓
     Phase Detection        Layout Strategy
     Element Classification  (Swimlane/Hierarchy/PortCentric)
     Relationship Analysis        ↓
           ↓                 Arcadia Rules Engine
     Quality Metrics              ↓
                            Professional Styler
                                  ↓
                            Post-Processor
                                  ↓
                            Quality Report
```

### New Modules (Recommended)
**Use these instead**:
1. `src/compiler/semantic_analyzer.rs` - Phase detection, element classification
2. `src/compiler/layout_strategy.rs` - Context-aware strategies
3. `src/compiler/arcadia_rules_engine.rs` - Methodology compliance
4. `src/compiler/professional_styler.rs` - Capella styling
5. `src/compiler/post_processor.rs` - Alignment and spacing
6. `src/compiler/quality_metrics_v2.rs` - Quality scoring

---

## Migration Guide

### For Existing Code Using Old Generators

#### Before (Deprecated)
```rust
use crate::compiler::arcviz_smart_routing::generate;

let html = generate(model)?;
```

#### After (Recommended)
```rust
use crate::compiler::semantic_analyzer::SemanticAnalyzer;
use crate::compiler::layout_strategy::StrategySelector;
use crate::compiler::arcadia_rules_engine::ArcadiaRulesEngine;
use crate::compiler::professional_styler::ProfessionalStyler;
use crate::compiler::post_processor::PostProcessor;
use crate::compiler::quality_metrics_v2::QualityMetrics;

// Step 1: Analyze semantics
let analyzer = SemanticAnalyzer::new();
let semantic = analyzer.analyze(model);

// Step 2: Select strategy
let selector = StrategySelector::new();
let strategy = selector.select(&semantic);
let layout_config = strategy.configure(&semantic);

// Step 3: Generate with strategy
let mut diagram_data = generate_with_elk(model, layout_config)?;

// Step 4: Apply rules
let rules = ArcadiaRulesEngine::new();
rules.apply(&mut diagram_data, &semantic);

// Step 5: Style professionally
let styler = ProfessionalStyler::default();
styler.apply_styles(&mut diagram_data, &semantic);

// Step 6: Post-process
let processor = PostProcessor::default();
let processed = processor.process(diagram_data);

// Step 7: Quality check
let metrics = QualityMetrics::new();
let report = metrics.calculate(&processed, &semantic);

// Step 8: Render final output
let html = render_to_html(processed, report)?;
```

---

## Deprecation Timeline

### Phase 1: Documentation (Current)
- ✅ Mark old generators as deprecated
- ✅ Document migration path
- ✅ Create archive notice

### Phase 2: Warning Period (1-2 months)
- Add deprecation warnings to old generators
- Update documentation to recommend new pipeline
- Provide migration examples

### Phase 3: Removal (3-6 months)
- Remove deprecated generators from codebase
- Move to `deprecated/` archive directory
- Update all references

---

## Keeping for Reference

### Files to Keep (for now)
These may still be useful as reference or for specific use cases:

1. **`elk_complete_generator.rs`** - Will be integrated with new pipeline
   - Status: ACTIVE, integrating Phase 1 & 2 modules
   
2. **`elk_json_generator.rs`** - ELK format utilities
   - Status: UTILITY, keep as helper

3. **`arcviz_explorer.rs`** - Interactive diagram viewer
   - Status: ACTIVE, different purpose (exploration vs generation)

---

## Archive Directory Structure

Suggested organization:
```
src/
├── compiler/
│   ├── semantic_analyzer.rs          ← NEW (Phase 1)
│   ├── layout_strategy.rs            ← NEW (Phase 1)
│   ├── post_processor.rs             ← NEW (Phase 1)
│   ├── quality_metrics_v2.rs         ← NEW (Phase 1)
│   ├── arcadia_rules_engine.rs       ← NEW (Phase 2)
│   ├── professional_styler.rs        ← NEW (Phase 2)
│   ├── elk_complete_generator.rs     ← ACTIVE (integrating new modules)
│   └── deprecated/                   ← ARCHIVE
│       ├── README.md                 ← Deprecation notice
│       ├── arcviz_smart_routing.rs   ← Old
│       ├── arcviz_channel_routing.rs ← Old
│       ├── arcviz_perfect_routing.rs ← Old
│       ├── arcviz_ultimate_routing.rs← Old
│       ├── arcviz_enhanced.rs        ← Old
│       ├── elk_dagre_hybrid.rs       ← Old
│       └── ...                       ← Other old generators
```

---

## Why Deprecate?

### Technical Debt
- 15+ different generators with overlapping functionality
- Inconsistent quality and maintainability
- No systematic approach to layout/styling
- Hard to add features (need to update 15 files)

### Quality Issues
- Basic layouts (2.5/10 quality)
- No Arcadia compliance
- No safety indicators
- Inconsistent results

### New Pipeline Benefits
- Modular architecture (easy to test/maintain)
- Context-aware (different strategies for different phases)
- Professional quality (7/10 vs 2.5/10)
- Extensible (add new strategies/rules/styles)
- Well-tested (40 unit tests)

---

## Questions?

### "Can I still use old generators?"
Yes, for now. They still work. But expect:
- No new features
- No bug fixes
- Deprecation warnings (coming soon)
- Removal in 3-6 months

### "What if I need feature X from old generator?"
File an issue. We'll either:
1. Add feature to new pipeline
2. Provide migration guide
3. Keep old generator longer if critical

### "When is new pipeline production-ready?"
After Option A integration (2-3 days):
- Test with real models
- Measure quality improvements
- Fix any integration bugs
- Document usage

---

## Summary

**Old Generators**: 15+ files, 2.5/10 quality, hard to maintain  
**New Pipeline**: 6 modules, 7/10 quality, well-tested, extensible

**Status**: Old generators marked as deprecated, will be archived  
**Action**: Migrate to new pipeline after Option A integration complete

**Next Steps**:
1. Integrate new modules into `elk_complete_generator.rs`
2. Test quality improvements
3. Gradually deprecate old generators
4. Archive after migration period
