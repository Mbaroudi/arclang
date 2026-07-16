# ArcLang Rendering Engine Improvement Roadmap
## Response to Quality Audit - Strategic Implementation Plan

**Date**: November 6, 2025  
**Based On**: Rendering Engine Audit v1.0  
**Approach**: Pragmatic Incremental Enhancement

---

## Executive Response

**Audit Acceptance**: We acknowledge the audit findings - current quality is ~20% of Capella standards.

**Strategic Decision**: **Option B+** - Targeted improvement with selective deep fixes
- **Timeframe**: 6 months (Phase 1: 3 months, Phase 2: 3 months)
- **Goal**: Achieve 6-7/10 quality (suitable for 80% of use cases)
- **Focus**: High-impact improvements, not complete parity

**Rationale**:
1. ArcLang's value is "Capella-as-code", not "clone Capella exactly"
2. 80/20 rule: 20% effort → 80% perceived quality
3. User feedback loop: improve based on real usage, not hypothetical needs
4. Maintain agility: full 18-28 month rewrite would kill momentum

---

## Phase 1: Foundation (Months 1-3)
### Goal: 2.5/10 → 5/10 Quality

### 1.1 Semantic Analysis Layer (4 weeks)

**Implementation**:

```typescript
// File: src/compiler/semantic_analyzer.ts

export class SemanticAnalyzer {
  /**
   * Analyze model to extract MBSE-specific intelligence
   */
  analyze(model: ArcLangModel): SemanticContext {
    return {
      // Detect Arcadia phase
      phase: this.detectPhase(model),
      
      // Classify all elements
      elements: this.classifyElements(model),
      
      // Analyze relationships
      relationships: this.analyzeRelationships(model),
      
      // Detect complexity
      complexity: this.assessComplexity(model),
      
      // Recommend strategy
      recommendedStrategy: this.selectStrategy(model)
    };
  }
  
  private detectPhase(model: ArcLangModel): ArcadiaPhase {
    // Detect OA, SA, LA, PA based on element types
    if (model.operational_analysis) return 'OA';
    if (model.system_analysis) return 'SA';
    if (model.logical_architecture) return 'LA';
    if (model.physical_architecture) return 'PA';
    return 'SA'; // default
  }
  
  private classifyElements(model: ArcLangModel): ElementClassification {
    return {
      actors: this.findActors(model),
      components: this.findComponents(model),
      functions: this.findFunctions(model),
      interfaces: this.findInterfaces(model),
      nodes: this.findPhysicalNodes(model)
    };
  }
}
```

**Impact**: Enables context-aware layout decisions

---

### 1.2 Layout Strategy System (4 weeks)

**Implementation**: 3 core strategies

#### Strategy 1: Swimlane (Operational Diagrams)

```typescript
// File: src/visualization/strategies/swimlane_strategy.ts

export class SwimlaneStrategy implements LayoutStrategy {
  configure(semantic: SemanticContext): ELKConfig {
    return {
      'elk.algorithm': 'layered',
      'elk.direction': 'DOWN',
      
      // CRITICAL: Enable partitioning
      'elk.partitioning.activate': true,
      
      // Spacing for swimlanes
      'elk.spacing.componentComponent': 150,
      'elk.layered.spacing.nodeNodeBetweenLayers': 80,
      
      // Port constraints
      'elk.portConstraints': 'FIXED_SIDE',
      
      // Edge routing
      'elk.edgeRouting': 'ORTHOGONAL'
    };
  }
  
  preProcess(elements: Element[]): Element[] {
    // Assign partitions to actors
    const actors = elements.filter(e => e.type === 'actor');
    const system = elements.filter(e => e.type !== 'actor');
    
    actors.forEach((actor, i) => {
      actor.layoutOptions = {
        'elk.partitioning.partition': i * 2 // Even partitions
      };
    });
    
    system.forEach(elem => {
      elem.layoutOptions = {
        'elk.partitioning.partition': 1 // Middle partition
      };
    });
    
    return elements;
  }
  
  postProcess(svg: SVGElement): SVGElement {
    // Add visual swimlane boundaries
    return this.addSwimlaneLines(svg);
  }
}
```

#### Strategy 2: Hierarchy (Component Diagrams)

```typescript
// File: src/visualization/strategies/hierarchy_strategy.ts

export class HierarchyStrategy implements LayoutStrategy {
  configure(semantic: SemanticContext): ELKConfig {
    return {
      'elk.algorithm': 'layered', // or 'mr.tree' for deep hierarchies
      'elk.direction': 'RIGHT',
      
      // CRITICAL: Handle nested components
      'elk.hierarchyHandling': 'INCLUDE_CHILDREN',
      
      // Padding for parent containers
      'elk.padding': '[top=40,left=30,bottom=30,right=30]',
      
      // Spacing
      'elk.spacing.nodeNode': 60,
      'elk.spacing.componentComponent': 100,
      
      // Port positioning
      'elk.portConstraints': 'FIXED_SIDE'
    };
  }
  
  preProcess(elements: Element[]): Element[] {
    // Set port sides: IN=left, OUT=right
    elements.forEach(elem => {
      if (elem.ports) {
        elem.ports.forEach(port => {
          if (port.direction === 'IN') {
            port.properties = { 'port.side': 'WEST' };
          } else if (port.direction === 'OUT') {
            port.properties = { 'port.side': 'EAST' };
          }
        });
      }
    });
    
    return elements;
  }
}
```

#### Strategy 3: Port-Centric (Functional Diagrams)

```typescript
// File: src/visualization/strategies/port_centric_strategy.ts

export class PortCentricStrategy implements LayoutStrategy {
  configure(semantic: SemanticContext): ELKConfig {
    return {
      'elk.algorithm': 'layered',
      'elk.direction': 'RIGHT',
      
      // CRITICAL: Port-to-port routing
      'elk.port.side': 'NORTH_SOUTH_EAST_WEST',
      'elk.portConstraints': 'FIXED_SIDE',
      
      // Edge optimization
      'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
      'elk.edgeRouting': 'ORTHOGONAL',
      
      // Spacing optimized for data flow
      'elk.layered.spacing.nodeNodeBetweenLayers': 120,
      'elk.layered.spacing.edgeNodeBetweenLayers': 40
    };
  }
  
  preProcess(elements: Element[]): Element[] {
    // Assign edge priorities based on data criticality
    elements.forEach(elem => {
      if (elem.edges) {
        elem.edges.forEach(edge => {
          if (edge.critical) {
            edge.layoutOptions = { 'elk.priority': 100 };
          }
        });
      }
    });
    
    return elements;
  }
}
```

**Strategy Selector**:

```typescript
// File: src/visualization/strategy_selector.ts

export class StrategySelector {
  select(semantic: SemanticContext): LayoutStrategy {
    const { phase, diagramType, elementTypes } = semantic;
    
    // Operational → Swimlane
    if (phase === 'OA' || diagramType === 'operational') {
      return new SwimlaneStrategy();
    }
    
    // Components with hierarchy → Hierarchy
    if (diagramType === 'component' || elementTypes.hasNesting) {
      return new HierarchyStrategy();
    }
    
    // Functional with data flows → Port-Centric
    if (diagramType === 'functional' || phase === 'SA') {
      return new PortCentricStrategy();
    }
    
    // Default: Hierarchy
    return new HierarchyStrategy();
  }
}
```

**Impact**: Context-appropriate layouts, eliminates "one size fits all"

---

### 1.3 Post-Processing Pipeline (3 weeks)

**Implementation**:

```typescript
// File: src/visualization/post_processor.ts

export class PostProcessor {
  process(svg: SVGElement, config: PostProcessConfig): SVGElement {
    let result = svg;
    
    // Step 1: Grid Snapping (align to 10px grid)
    result = this.snapToGrid(result, 10);
    
    // Step 2: Element Alignment (horizontal/vertical)
    result = this.alignElements(result);
    
    // Step 3: Spacing Distribution
    result = this.distributeSpacing(result);
    
    // Step 4: Label Optimization (avoid overlaps)
    result = this.optimizeLabels(result);
    
    return result;
  }
  
  private snapToGrid(svg: SVGElement, gridSize: number): SVGElement {
    // Find all elements with x, y positions
    const elements = svg.querySelectorAll('[x][y]');
    
    elements.forEach(elem => {
      const x = parseFloat(elem.getAttribute('x'));
      const y = parseFloat(elem.getAttribute('y'));
      
      // Snap to nearest grid point
      elem.setAttribute('x', String(Math.round(x / gridSize) * gridSize));
      elem.setAttribute('y', String(Math.round(y / gridSize) * gridSize));
    });
    
    return svg;
  }
  
  private alignElements(svg: SVGElement): SVGElement {
    // Group elements by approximate Y coordinate
    const groups = this.groupByY(svg, threshold: 20);
    
    // Align each group to average Y
    groups.forEach(group => {
      const avgY = group.reduce((sum, e) => sum + e.y, 0) / group.length;
      group.forEach(e => e.setAttribute('y', String(avgY)));
    });
    
    return svg;
  }
  
  private distributeSpacing(svg: SVGElement): SVGElement {
    // Find all element pairs with small gaps
    const elements = this.getElements(svg);
    const gaps = this.findGaps(elements);
    
    // Adjust positions to create even spacing
    const targetGap = this.calculateOptimalGap(gaps);
    this.applySpacing(elements, targetGap);
    
    return svg;
  }
}
```

**Impact**: Professional appearance, eliminates visual messiness

---

### 1.4 Quality Metrics System (2 weeks)

**Implementation**:

```typescript
// File: src/visualization/quality_metrics.ts

export class QualityMetrics {
  calculate(svg: SVGElement, semantic: SemanticContext): QualityReport {
    return {
      // Core metrics
      edgeCrossings: this.countEdgeCrossings(svg),
      nodeOverlaps: this.detectNodeOverlaps(svg),
      whitespaceBalance: this.assessWhitespace(svg),
      alignmentScore: this.scoreAlignment(svg),
      
      // Arcadia compliance
      arcadiaCompliance: this.checkArcadiaCompliance(svg, semantic),
      
      // Overall score
      overallScore: this.calculateOverallScore(),
      
      // Warnings
      warnings: this.generateWarnings()
    };
  }
  
  private countEdgeCrossings(svg: SVGElement): number {
    const edges = svg.querySelectorAll('line, path');
    let crossings = 0;
    
    // Check each pair of edges for intersection
    for (let i = 0; i < edges.length; i++) {
      for (let j = i + 1; j < edges.length; j++) {
        if (this.edgesIntersect(edges[i], edges[j])) {
          crossings++;
        }
      }
    }
    
    return crossings;
  }
  
  private checkArcadiaCompliance(svg: SVGElement, semantic: SemanticContext): number {
    let score = 100;
    const rules = this.getArcadiaRules(semantic.phase);
    
    rules.forEach(rule => {
      if (!this.checkRule(svg, rule)) {
        score -= rule.weight;
      }
    });
    
    return Math.max(0, score);
  }
  
  private generateWarnings(): string[] {
    const warnings: string[] = [];
    
    if (this.edgeCrossings > 5) {
      warnings.push('High edge crossing count - consider different layout algorithm');
    }
    
    if (this.nodeOverlaps > 0) {
      warnings.push('Node overlaps detected - increase spacing');
    }
    
    if (this.arcadiaCompliance < 70) {
      warnings.push('Low Arcadia compliance - check phase-specific rules');
    }
    
    return warnings;
  }
}
```

**CLI Integration**:

```bash
arclang diagram model.arc -o output.svg --quality-report

# Output:
# ✅ Diagram generated: output.svg
# 
# Quality Report:
#   Overall Score: 6.5/10
#   - Edge Crossings: 3 (target: <5) ✅
#   - Node Overlaps: 0 ✅
#   - Whitespace Balance: 0.52 (target: 0.4-0.6) ✅
#   - Alignment Score: 0.65 (target: >0.8) ⚠️
#   - Arcadia Compliance: 75% (target: >90%) ⚠️
# 
# Warnings:
#   ⚠️ Low alignment score - consider grid snap
#   ⚠️ Arcadia compliance below target - check phase rules
```

**Impact**: Visibility into quality, actionable feedback

---

### Phase 1 Deliverables

✅ **Semantic Analysis Layer**: Understands model context  
✅ **3 Core Strategies**: Swimlane, Hierarchy, Port-Centric  
✅ **Post-Processing**: Grid snap, alignment, spacing  
✅ **Quality Metrics**: Scoring + warnings

**Expected Quality**: 5-6/10 (from 2.5/10)

---

## Phase 2: Arcadia Compliance (Months 4-6)
### Goal: 5/10 → 7/10 Quality

### 2.1 Arcadia Rules Engine (5 weeks)

**Implementation**:

```typescript
// File: src/arcadia/rules_engine.ts

export class ArcadiaRulesEngine {
  private rules: Map<ArcadiaPhase, Rule[]>;
  
  constructor() {
    this.rules = new Map([
      ['OA', this.getOperationalRules()],
      ['SA', this.getSystemRules()],
      ['LA', this.getLogicalRules()],
      ['PA', this.getPhysicalRules()]
    ]);
  }
  
  apply(elements: Element[], phase: ArcadiaPhase): Element[] {
    const phaseRules = this.rules.get(phase) || [];
    
    phaseRules.forEach(rule => {
      elements = rule.apply(elements);
    });
    
    return elements;
  }
  
  private getOperationalRules(): Rule[] {
    return [
      // Rule 1: Actors must be at boundaries
      new PositioningRule({
        elementType: 'actor',
        position: 'boundary',
        priority: 'high'
      }),
      
      // Rule 2: Activities must be inside system boundary
      new ContainmentRule({
        elementType: 'activity',
        container: 'system_boundary',
        priority: 'high'
      }),
      
      // Rule 3: Use swimlane layout
      new LayoutRule({
        strategy: 'swimlane',
        partitionBy: 'actor'
      })
    ];
  }
  
  private getLogicalRules(): Rule[] {
    return [
      // Rule 1: Interfaces must show direction
      new InterfaceRule({
        providedStyle: 'lollipop',
        requiredStyle: 'socket'
      }),
      
      // Rule 2: Components must use Capella colors
      new ColorRule({
        sensor: '#70AD47',
        controller: '#6495ED',
        actuator: '#ED7D31'
      }),
      
      // Rule 3: Safety borders for ASIL components
      new SafetyRule({
        ASIL_D: { borderWidth: 6, borderColor: '#8B0000' },
        ASIL_C: { borderWidth: 4, borderColor: '#CC0000' }
      })
    ];
  }
  
  private getPhysicalRules(): Rule[] {
    return [
      // Rule 1: ECUs must use 3D representation
      new ECURule({
        color: '#FFE699',
        style: '3d',
        showProcessor: true
      }),
      
      // Rule 2: Behavior components nest inside ECUs
      new DeploymentRule({
        nestBehaviorComponents: true,
        showAllocation: true
      })
    ];
  }
}
```

**Rules Configuration** (JSON):

```json
{
  "operational_analysis": {
    "actors": {
      "position": "boundary",
      "symbol": "⊕",
      "color": "#E8F4F8"
    },
    "activities": {
      "containerRequired": true,
      "flowDirection": "top-to-bottom"
    },
    "layout": {
      "strategy": "swimlane",
      "partitionBy": "actor"
    }
  },
  "logical_architecture": {
    "components": {
      "colors": {
        "sensor": "#70AD47",
        "controller": "#6495ED",
        "actuator": "#ED7D31",
        "default": "#BFBFBF"
      },
      "safetyBorders": {
        "ASIL_D": { "width": 6, "color": "#8B0000" },
        "ASIL_C": { "width": 4, "color": "#CC0000" }
      }
    },
    "interfaces": {
      "provided": "lollipop",
      "required": "socket",
      "showProtocol": true
    }
  },
  "physical_architecture": {
    "nodes": {
      "style": "3d",
      "color": "#FFE699",
      "showSpecs": true
    },
    "deployment": {
      "nestComponents": true,
      "showAllocation": true
    }
  }
}
```

**Impact**: Diagrams follow Arcadia methodology automatically

---

### 2.2 Professional Styling System (3 weeks)

**Implementation**:

```typescript
// File: src/visualization/professional_styler.ts

export class ProfessionalStyler {
  applyStyles(svg: SVGElement, semantic: SemanticContext): SVGElement {
    // Apply Capella color scheme
    svg = this.applyColorCoding(svg, semantic);
    
    // Add safety indicators
    svg = this.applySafetyIndicators(svg, semantic);
    
    // Add shadows and depth
    svg = this.addDepthEffects(svg);
    
    // Add legend
    svg = this.addLegend(svg, semantic);
    
    return svg;
  }
  
  private applyColorCoding(svg: SVGElement, semantic: SemanticContext): SVGElement {
    const colorMap = {
      'actor': '#E8F4F8',
      'sensor': '#70AD47',
      'controller': '#6495ED',
      'actuator': '#ED7D31',
      'function': '#70AD47',
      'ecu': '#FFE699'
    };
    
    semantic.elements.forEach(elem => {
      const color = colorMap[elem.stereotype] || '#BFBFBF';
      const svgElem = svg.querySelector(`#${elem.id}`);
      if (svgElem) {
        svgElem.setAttribute('fill', color);
      }
    });
    
    return svg;
  }
  
  private applySafetyIndicators(svg: SVGElement, semantic: SemanticContext): SVGElement {
    semantic.elements.forEach(elem => {
      if (elem.safetyLevel) {
        const svgElem = svg.querySelector(`#${elem.id}`);
        if (svgElem) {
          const borderConfig = this.getSafetyBorder(elem.safetyLevel);
          svgElem.setAttribute('stroke', borderConfig.color);
          svgElem.setAttribute('stroke-width', String(borderConfig.width));
        }
      }
    });
    
    return svg;
  }
  
  private addLegend(svg: SVGElement, semantic: SemanticContext): SVGElement {
    const legend = this.createLegend({
      title: `${semantic.phase} - ${semantic.diagramType}`,
      items: this.getLegendItems(semantic)
    });
    
    svg.appendChild(legend);
    return svg;
  }
}
```

**Impact**: Professional appearance suitable for presentations

---

### 2.3 Complete Sequence & State Machine Diagrams (4 weeks)

#### Sequence Diagrams

```typescript
// File: src/visualization/sequence_layout.ts

export class SequenceLayout {
  layout(scenario: Scenario): SVGElement {
    // Custom time-based layout (not ELK)
    const participants = scenario.participants;
    const messages = scenario.messages;
    
    // Position participants horizontally
    const participantSpacing = 200;
    participants.forEach((p, i) => {
      p.x = 100 + (i * participantSpacing);
      p.y = 50;
    });
    
    // Position messages vertically by time
    const messageSpacing = 60;
    messages.forEach((m, i) => {
      m.y = 150 + (i * messageSpacing);
    });
    
    return this.renderSequenceDiagram(participants, messages);
  }
  
  private renderSequenceDiagram(participants, messages): SVGElement {
    const svg = createSVG();
    
    // Draw lifelines
    participants.forEach(p => {
      svg.appendChild(this.createLifeline(p));
    });
    
    // Draw messages
    messages.forEach(m => {
      svg.appendChild(this.createMessage(m));
    });
    
    return svg;
  }
}
```

#### State Machine Diagrams

```typescript
// File: src/visualization/statemachine_layout.ts

export class StateMachineLayout {
  layout(stateMachine: StateMachine): SVGElement {
    // Use ELK Stress algorithm for state machines
    const elkConfig = {
      'elk.algorithm': 'stress',
      'elk.stress.epsilon': 0.1,
      'elk.spacing.nodeNode': 80
    };
    
    return this.layoutWithELK(stateMachine, elkConfig);
  }
}
```

**Impact**: All 10 diagram types now working

---

### Phase 2 Deliverables

✅ **Arcadia Rules Engine**: Phase-specific compliance  
✅ **Professional Styling**: Colors, legends, indicators  
✅ **Sequence Diagrams**: Custom time-based layout  
✅ **State Machine Diagrams**: ELK Stress algorithm

**Expected Quality**: 7-8/10 (from 5-6/10)

---

## Implementation Priority Matrix

| Component | Impact | Effort | Priority | Timeline |
|-----------|--------|--------|----------|----------|
| Semantic Analysis | High | Medium | 🔴 P0 | Week 1-4 |
| Swimlane Strategy | High | Medium | 🔴 P0 | Week 5-6 |
| Hierarchy Strategy | High | Medium | 🔴 P0 | Week 7-8 |
| Port-Centric Strategy | Medium | Medium | 🟡 P1 | Week 9-10 |
| Post-Processing | High | Low | 🔴 P0 | Week 11-12 |
| Quality Metrics | Medium | Low | 🟡 P1 | Week 13 |
| Arcadia Rules | High | High | 🟡 P1 | Week 14-18 |
| Professional Styling | Medium | Low | 🟢 P2 | Week 19-21 |
| Sequence Diagrams | Low | Medium | 🟢 P2 | Week 22-24 |
| State Machines | Low | Low | 🟢 P2 | Week 25-26 |

---

## Success Metrics

### Phase 1 Success Criteria (Month 3)

✅ **Quality Score**: ≥ 5/10 (from 2.5/10)  
✅ **Edge Crossings**: < 10 per diagram  
✅ **Node Overlaps**: < 2 per diagram  
✅ **Grid Aligned**: 100% of elements  
✅ **User Feedback**: "Acceptable for internal use"

### Phase 2 Success Criteria (Month 6)

✅ **Quality Score**: ≥ 7/10  
✅ **Arcadia Compliance**: ≥ 80%  
✅ **Edge Crossings**: < 5 per diagram  
✅ **Node Overlaps**: 0  
✅ **All Diagram Types**: 10/10 working  
✅ **User Feedback**: "Suitable for customer presentations"

---

## Risk Mitigation

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| ELK integration complexity | Medium | High | Start with documented examples, incremental adoption |
| Performance degradation | Low | Medium | Profile early, optimize critical paths |
| Arcadia rule conflicts | Medium | Medium | Validation suite, prioritize rules |
| Breaking changes to API | Low | High | Versioned exports, backward compatibility |

### Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Underestimated effort | Medium | High | 20% buffer, descope Phase 2 if needed |
| Dependency delays | Low | Medium | Use stable ELK version, minimize external deps |
| Scope creep | High | High | Strict prioritization, "Phase 3" backlog |

---

## Testing Strategy

### Unit Tests
- Each strategy with sample models
- Post-processing algorithms
- Quality metric calculations
- Arcadia rule validation

### Integration Tests
- Full pipeline for all 10 diagram types
- Quality regression tests
- Visual snapshot tests

### Acceptance Tests
- Real aerospace models (if available)
- Real automotive models (if available)
- Comparison to Capella reference diagrams

---

## Documentation Plan

### Developer Documentation
- Architecture overview
- Strategy implementation guide
- ELK configuration reference
- Arcadia rules reference

### User Documentation
- Quality report interpretation
- Best practices for model structure
- Troubleshooting guide
- Migration guide (from current version)

---

## Conclusion

This 6-month roadmap delivers **80% of perceived quality with 20% of full-parity effort**. By focusing on high-impact improvements (semantic analysis, layout strategies, quality metrics, Arcadia compliance), we achieve:

- **Phase 1** (3 months): Acceptable internal quality (5-6/10)
- **Phase 2** (3 months): Customer-presentable quality (7-8/10)

This pragmatic approach maintains ArcLang's agility while significantly narrowing the Capella quality gap. Future phases can build on this foundation based on real user needs.

**Next Step**: Implement Phase 1, Week 1 - Semantic Analysis Layer
