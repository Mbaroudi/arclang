# Rendering Engine Integration Plan

**Status**: Phase 1 & 2 modules complete, but **not yet integrated** into generators  
**Issue**: New Rust modules exist independently, need to be wired into existing generators

---

## Current Situation

### ✅ What We Built (Phase 1 & 2)
6 new Rust modules with 4,500+ lines of code:
1. `semantic_analyzer.rs` - Phase detection, element classification
2. `layout_strategy.rs` - 3 context-aware strategies
3. `post_processor.rs` - Grid snap, alignment, spacing
4. `quality_metrics_v2.rs` - Quality scoring
5. `arcadia_rules_engine.rs` - 11 Arcadia rules
6. `professional_styler.rs` - Capella colors, safety indicators

### ❌ What's Missing
**Integration**: These modules are not called by the existing generators:
- `src/compiler/arcviz_elk.rs`
- `src/compiler/arcviz_dagre.rs`
- `src/compiler/elk_complete_generator.rs`
- etc.

### Current Flow (Without Integration)
```
Model → Parser → AST → Generator → HTML/SVG
                           ↓
                     (uses old logic)
```

### Target Flow (With Integration)
```
Model → Parser → AST → SemanticAnalyzer → StrategySelector → Generator
                           ↓                      ↓
                    SemanticContext        LayoutStrategy
                           ↓                      ↓
                    ArcadiaRules ← → ProfessionalStyler
                           ↓                      ↓
                    PostProcessor → QualityMetrics → HTML/SVG
```

---

## Integration Options

### Option A: Quick Integration (2-3 days)
**Integrate into one generator as proof-of-concept**

Target: `elk_complete_generator.rs` (newest, best maintained)

**Steps**:
1. Add `use` statements for new modules
2. Call `SemanticAnalyzer::analyze()` on model
3. Use `StrategySelector::select()` for ELK config
4. Apply `ArcadiaRulesEngine` to diagram data
5. Apply `ProfessionalStyler` to add colors/legends
6. Run `PostProcessor` for alignment
7. Calculate `QualityMetrics` and embed in HTML

**Code changes**: ~200 lines in 1 file

**Benefit**: See immediate quality improvement in one generator

---

### Option B: Full Integration (1-2 weeks)
**Integrate into all generators**

Generators to update:
1. `arcviz_elk.rs`
2. `arcviz_dagre.rs`
3. `elk_complete_generator.rs`
4. `elk_dagre_hybrid.rs`
5. `arcviz_enhanced.rs`

**Steps**: Repeat Option A for each generator

**Code changes**: ~1,000 lines across 5 files

**Benefit**: All generators benefit from improvements

---

### Option C: Refactor First (2-3 weeks)
**Create unified rendering pipeline**

1. Create new `src/compiler/unified_renderer.rs`
2. Extract common logic from all generators
3. Build pipeline: Semantic → Strategy → Rules → Style → Post → Metrics
4. Refactor all generators to use pipeline
5. Deprecate old generators over time

**Code changes**: ~2,000 lines, major refactor

**Benefit**: Clean architecture, maintainable, extensible

---

## Recommended Approach: Option A (Quick Integration)

**Why**: 
- Fast feedback (2-3 days vs weeks)
- Minimal risk (only touch 1 file)
- Proves value before investing more
- Can iterate to Option B or C based on results

### Implementation Plan

#### Step 1: Integrate SemanticAnalyzer
```rust
// In elk_complete_generator.rs

use crate::compiler::semantic_analyzer::SemanticAnalyzer;

pub fn generate(model: &Model) -> Result<String, Error> {
    // NEW: Analyze model semantics
    let analyzer = SemanticAnalyzer::new();
    let semantic = analyzer.analyze(model);
    
    println!("📊 Detected phase: {:?}", semantic.phase);
    println!("📊 Recommended strategy: {:?}", semantic.recommended_strategy);
    
    // ... rest of generation
}
```

#### Step 2: Use Layout Strategy
```rust
use crate::compiler::layout_strategy::{StrategySelector, LayoutStrategy};

pub fn generate(model: &Model) -> Result<String, Error> {
    let semantic = analyzer.analyze(model);
    
    // NEW: Select strategy
    let selector = StrategySelector::new();
    let strategy = selector.select(&semantic);
    
    println!("📐 Using strategy: {}", strategy.name());
    
    // Get ELK configuration from strategy
    let layout_config = strategy.configure(&semantic);
    
    // Use layout_config for ELK options
    let elk_options = layout_config.options;
    
    // ... rest of generation with elk_options
}
```

#### Step 3: Apply Arcadia Rules
```rust
use crate::compiler::arcadia_rules_engine::ArcadiaRulesEngine;

pub fn generate(model: &Model) -> Result<String, Error> {
    // ... after ELK layout
    
    // NEW: Apply Arcadia rules
    let rules_engine = ArcadiaRulesEngine::new();
    let result = rules_engine.apply(&mut diagram_data, &semantic);
    
    println!("✓ Rules applied: {}/{}", 
        result.rules_passed, result.rules_applied);
    
    if !result.violations.is_empty() {
        println!("⚠ Violations: {}", result.violations.len());
    }
    
    // ... continue
}
```

#### Step 4: Apply Professional Styling
```rust
use crate::compiler::professional_styler::{ProfessionalStyler, StyleConfig};

pub fn generate(model: &Model) -> Result<String, Error> {
    // ... after rules
    
    // NEW: Apply professional styling
    let config = StyleConfig::default(); // Capella theme
    let styler = ProfessionalStyler::new(config);
    styler.apply_styles(&mut diagram_data, &semantic);
    
    println!("🎨 Applied Capella styling");
    
    // ... continue
}
```

#### Step 5: Post-Process
```rust
use crate::compiler::post_processor::{PostProcessor, PostProcessConfig};

pub fn generate(model: &Model) -> Result<String, Error> {
    // ... after styling
    
    // NEW: Post-process for alignment
    let config = PostProcessConfig::default(); // 10px grid
    let processor = PostProcessor::new(config);
    let processed = processor.process(diagram_data);
    
    println!("✨ Post-processed: grid-snapped & aligned");
    
    // ... continue with processed data
}
```

#### Step 6: Calculate Quality Metrics
```rust
use crate::compiler::quality_metrics_v2::QualityMetrics;

pub fn generate(model: &Model) -> Result<String, Error> {
    // ... after post-processing
    
    // NEW: Calculate quality
    let metrics = QualityMetrics::new();
    let report = metrics.calculate(&processed, &semantic);
    
    println!("📊 Quality Score: {:.1}/10", report.overall_score);
    println!("   Edge Crossings: {}", report.edge_crossings);
    println!("   Node Overlaps: {}", report.node_overlaps);
    println!("   Arcadia Compliance: {:.0}%", report.arcadia_compliance);
    
    // Embed report in HTML output
    let html = format!(r#"
        <div class="quality-report">
            <h3>Quality Report</h3>
            <p>Score: {:.1}/10</p>
            <p>Compliance: {:.0}%</p>
        </div>
        {}
    "#, report.overall_score, report.arcadia_compliance, diagram_html);
    
    Ok(html)
}
```

---

## Next Steps

### Immediate (Today)
1. ✅ Document current status
2. Choose integration approach
3. Start with Option A (quick integration)

### This Week
1. Integrate into `elk_complete_generator.rs`
2. Test with real models
3. Measure quality improvement
4. Take before/after screenshots

### Next Week
1. Fix any integration bugs
2. Tune parameters (spacing, colors, etc.)
3. Document integration patterns
4. Decide: Option B (full integration) or Option C (refactor)?

---

## Success Criteria

After Option A integration, we should see:
- ✅ Quality scores printed to console
- ✅ Different layouts for OA vs LA vs PA diagrams
- ✅ Capella colors (green/blue/orange)
- ✅ Safety borders on ASIL components
- ✅ Legends with phase information
- ✅ Grid-snapped coordinates (multiples of 10)
- ✅ Quality report embedded in HTML

---

## Conclusion

**Status**: Phase 1 & 2 implementation is **complete and tested** (40/40 tests passing)

**Gap**: Need to **integrate** modules into generators to see actual quality improvement

**Recommendation**: Start with Option A (quick integration into 1 generator) to prove value

**Estimated Time**: 2-3 days for Option A, then reassess

**Next Action**: Integrate semantic analyzer into `elk_complete_generator.rs` as first step
