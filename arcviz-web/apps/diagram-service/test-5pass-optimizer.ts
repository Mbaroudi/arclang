/**
 * Test script for 5-Pass Optimization Pipeline
 * Validates Phase 3 enhancement: 95% -> 98% compliance
 */

import { optimizeDiagram, generateOptimizationReport } from './src/layouts/multi-pass-optimizer';
import { DiagramNode, DiagramEdge } from './src/types/diagram';

// Sample test data: Emergency Braking System (Logical Architecture)
const testNodes: DiagramNode[] = [
  {
    id: 'C1',
    label: 'Radar Sensor',
    type: 'component',
    metadata: { safety_level: 'ASIL-D' },
  },
  {
    id: 'C2',
    label: 'Camera Sensor',
    type: 'component',
    metadata: { safety_level: 'ASIL-D' },
  },
  {
    id: 'C3',
    label: 'Sensor Fusion',
    type: 'component',
    metadata: { safety_level: 'ASIL-D' },
  },
  {
    id: 'C4',
    label: 'Threat Assessment',
    type: 'component',
    metadata: { safety_level: 'ASIL-D' },
  },
  {
    id: 'C5',
    label: 'Emergency Brake Controller',
    type: 'component',
    metadata: { safety_level: 'ASIL-D' },
  },
  {
    id: 'C6',
    label: 'Brake Actuator',
    type: 'component',
    metadata: { safety_level: 'ASIL-B' },
  },
  {
    id: 'C7',
    label: 'Driver Interface',
    type: 'component',
    metadata: { safety_level: 'QM' },
  },
  {
    id: 'C8',
    label: 'Vehicle CAN Bus',
    type: 'component',
    metadata: { safety_level: 'ASIL-B' },
  },
  {
    id: 'C9',
    label: 'Object Tracker',
    type: 'component',
    metadata: { safety_level: 'ASIL-D' },
  },
];

const testEdges: DiagramEdge[] = [
  { id: 'E1', from: 'C1', to: 'C3', label: 'Radar Data', type: 'component-exchange' },
  { id: 'E2', from: 'C2', to: 'C3', label: 'Camera Data', type: 'component-exchange' },
  { id: 'E3', from: 'C3', to: 'C9', label: 'Fused Data', type: 'component-exchange' },
  { id: 'E4', from: 'C9', to: 'C4', label: 'Tracked Objects', type: 'component-exchange' },
  { id: 'E5', from: 'C4', to: 'C5', label: 'Threat Level', type: 'component-exchange' },
  { id: 'E6', from: 'C5', to: 'C6', label: 'Brake Command', type: 'component-exchange' },
  { id: 'E7', from: 'C5', to: 'C7', label: 'Warning Signal', type: 'component-exchange' },
  { id: 'E8', from: 'C8', to: 'C5', label: 'Vehicle Speed', type: 'component-exchange' },
  { id: 'E9', from: 'C8', to: 'C4', label: 'Vehicle Status', type: 'component-exchange' },
];

async function runTest() {
  console.log('🚀 Starting 5-Pass Optimization Pipeline Test\n');
  console.log(`Test Data: ${testNodes.length} nodes, ${testEdges.length} edges\n`);

  try {
    // Run optimization with all 5 passes enabled
    const result = await optimizeDiagram(testNodes, testEdges, {
      enablePass1: true,
      enablePass2: true,
      enablePass3: true,
      enablePass4: true,
      enablePass5: true,
      maxIterations: 5,
      targetCrossings: 10,
      gridSize: 20,
      diagramType: 'component-architecture',
    });

    // Generate and display report
    const report = generateOptimizationReport(result);
    console.log(report);

    // Validate results
    console.log('\n✅ VALIDATION RESULTS');
    console.log('='.repeat(70));
    
    if (result.passResults.length === 5) {
      console.log(`✓ All 5 passes completed successfully`);
    } else {
      console.log(`✗ Expected 5 passes, got ${result.passResults.length}`);
    }

    if (result.qualityMetrics.overallScore >= 95) {
      console.log(`✓ Quality score ${result.qualityMetrics.overallScore.toFixed(1)}/100 meets Phase 3 target (95+)`);
    } else {
      console.log(`⚠ Quality score ${result.qualityMetrics.overallScore.toFixed(1)}/100 below Phase 3 target (95+)`);
    }

    if (result.totalTimeMs < 20000) {
      console.log(`✓ Processing time ${result.totalTimeMs}ms within timeout (20s)`);
    } else {
      console.log(`✗ Processing time ${result.totalTimeMs}ms exceeds timeout`);
    }

    if (result.nodes.length === testNodes.length) {
      console.log(`✓ All nodes positioned (${result.nodes.length}/${testNodes.length})`);
    }

    if (result.edges.length === testEdges.length) {
      console.log(`✓ All edges routed (${result.edges.length}/${testEdges.length})`);
    }

    // Check for grid alignment (Pass 4 feature)
    const gridAligned = result.nodes.filter(n => 
      n.position.x % 20 === 0 && n.position.y % 20 === 0
    ).length;
    const gridAlignmentPercent = (gridAligned / result.nodes.length) * 100;
    
    if (gridAlignmentPercent >= 80) {
      console.log(`✓ Grid alignment ${gridAlignmentPercent.toFixed(1)}% meets target (80%+)`);
    } else {
      console.log(`⚠ Grid alignment ${gridAlignmentPercent.toFixed(1)}% below target (80%+)`);
    }

    console.log('\n🎉 5-Pass Optimization Pipeline Test Complete!');
    
    process.exit(0);
  } catch (error) {
    console.error('\n❌ Test Failed:', error);
    process.exit(1);
  }
}

runTest();
