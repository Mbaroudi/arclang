# Archived Rendering Engines

This directory contains rendering engines and layouts that are no longer actively used in ArcLang.

## Why Archived?

These files have been moved to archive for the following reasons:
- Superseded by better implementations (e.g., hybrid engine)
- Duplicate functionality (e.g., multiple state machine renderers)
- Not part of Capella methodology
- Unused in production code

## Archived Renderers

### `breakdown-tree.ts`
- **Reason**: Superseded by `tree.ts`
- **Status**: Duplicate functionality
- **Alternative**: Use `renderers/tree.ts` instead

### `state-machine.ts`
- **Reason**: Superseded by `statemachine.ts`
- **Status**: Naming inconsistency (duplicate)
- **Alternative**: Use `renderers/statemachine.ts` instead

### `process-diagram.ts`
- **Reason**: Not part of Capella/Arcadia methodology
- **Status**: Out of scope
- **Alternative**: Not applicable

### `missions-capabilities.ts`
- **Reason**: Superseded by `capability.ts`
- **Status**: Duplicate functionality
- **Alternative**: Use `renderers/capability.ts` instead

### `class.ts`
- **Reason**: Superseded by `classdiagram.ts`
- **Status**: Naming inconsistency (duplicate)
- **Alternative**: Use `renderers/classdiagram.ts` instead

## Archived Layouts

### `nested-box-packing.ts`
- **Reason**: Superseded by hybrid ELK+Dagre+D3 engine
- **Status**: Inferior layout quality
- **Alternative**: Use `layouts/hybrid-elk-dagre-d3.ts` instead

### `periphery-constraint.ts`
- **Reason**: Superseded by hybrid engine
- **Status**: Not needed with multi-pass optimization
- **Alternative**: Use `layouts/hybrid-elk-dagre-d3.ts` instead

### `reingold-tilford.ts`
- **Reason**: Superseded by `tree.ts` layout
- **Status**: Duplicate functionality
- **Alternative**: Use `layouts/tree.ts` instead

### `multi-pass-optimizer.ts`
- **Reason**: Superseded by `hybrid-elk-dagre-d3.ts`
- **Status**: Old implementation of multi-pass concept
- **Alternative**: Use `layouts/hybrid-elk-dagre-d3.ts` instead

## Archived Utilities

### `quality-metrics.ts`
- **Reason**: Not actively used in production
- **Status**: May contain useful code for future metrics
- **Alternative**: None currently

### `traceability-styles.ts`
- **Reason**: Not actively used in production
- **Status**: May be useful for future traceability features
- **Alternative**: None currently

## Recovery

If you need to restore any archived file:

```bash
# From diagram-service directory
cp archive/renderers/file.ts src/renderers/
# or
cp archive/layouts/file.ts src/layouts/
```

Then update `src/index.ts` to export the restored module.

## Permanent Deletion

These files will remain archived until:
1. 6 months pass with no requests to restore them
2. A major version bump (v2.0.0) of ArcLang
3. Consensus that they are no longer useful for reference

---

**Archived**: November 3, 2025  
**Reason**: Code organization and cleanup  
**Status**: Safe to restore if needed
