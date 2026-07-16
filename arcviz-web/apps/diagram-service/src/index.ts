/**
 * ArcLang Diagram Service
 * 
 * Main entry point for Capella-quality diagram rendering
 * Uses Hybrid ELK+Dagre+D3 engine for optimal layout quality
 */

export * from './types/model';
export * from './types/diagram';

// ============================================================================
// ACTIVE RENDERERS - Production Ready
// ============================================================================

// Operational Analysis (OA) - Use hybrid for best quality
export * from './renderers/operational-hybrid';

// System Analysis (SA)
export * from './renderers/functional';
export * from './renderers/dataflow';
export * from './renderers/capability';

// Logical Architecture (LA)
export * from './renderers/component';
export * from './renderers/functional-chain';

// Physical Architecture (PA)
export * from './renderers/physical';

// Cross-layer diagrams
export * from './renderers/sequence';
export * from './renderers/statemachine';
export * from './renderers/classdiagram';
export * from './renderers/tree';
export * from './renderers/system-context';

// ============================================================================
// ACTIVE LAYOUTS - Production Ready
// ============================================================================

// Primary: Hybrid multi-pass optimizer (ELK 70% + Dagre 20% + D3 10%)
export * from './layouts/hybrid-elk-dagre-d3';

// Supporting layouts
export * from './layouts/hierarchical';      // For component/physical diagrams
export * from './layouts/swimlane';          // For operational diagrams (backup)
export * from './layouts/elk-operational';   // ELK-only operational (backup)
export * from './layouts/timeline';          // For sequence diagrams
export * from './layouts/state-graph';       // For state machines
export * from './layouts/tree';              // For tree diagrams

// ============================================================================
// UTILITIES
// ============================================================================

export * from './utils/svg';
export * from './utils/safety-colors';
export * from './utils/capella-colors';
export * from './utils/system-boundary';
export * from './utils/port-validation';
export { measureText } from './utils/text-metrics';
export * from './utils/exchange-item-visualization';
export * from './utils/interface-notation';

console.log('ArcLang Diagram Service initialized - Hybrid ELK+Dagre+D3 engine ready');
