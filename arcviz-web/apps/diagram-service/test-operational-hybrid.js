/**
 * Test script for hybrid ELK+Dagre+D3 operational diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderOperationalActivityHybrid } = require('./dist/renderers/operational-hybrid');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-operational.json';
const outputFile = args[1] || 'operational-hybrid.svg';

console.log('🚀 Hybrid ELK+Dagre+D3 Operational Diagram Renderer Test\n');
console.log(`📁 Input:  ${jsonFile}`);
console.log(`📄 Output: ${outputFile}\n`);

const jsonPath = path.resolve(__dirname, jsonFile);
if (!fs.existsSync(jsonPath)) {
  console.error(`❌ Error: Input file not found: ${jsonPath}`);
  process.exit(1);
}

const modelJson = fs.readFileSync(jsonPath, 'utf-8');
const model = JSON.parse(modelJson);

console.log('✓ Loaded model');

const operationalModel = {
  name: model.operational_analysis?.[0]?.name || "Operational Analysis",
  actors: model.operational_analysis?.[0]?.actors || [],
  entities: model.operational_analysis?.[0]?.entities || [],
  activities: model.operational_analysis?.[0]?.activities || [],
  exchanges: model.operational_analysis?.[0]?.exchanges || [],
};

console.log(`🚀 Rendering with HYBRID engine: "${operationalModel.name}"`);
console.log(`  - Actors: ${operationalModel.actors.length}`);
console.log(`  - Entities: ${operationalModel.entities.length}`);
console.log(`  - Activities: ${operationalModel.activities.length}`);
console.log(`  - Exchanges: ${operationalModel.exchanges.length}\n`);

console.log('⚙️  Multi-Pass Optimization:');
console.log('  Layer 1: ELK (70%) - Hierarchical structure');
console.log('  Layer 2: Dagre (20%) - Edge crossing minimization');
console.log('  Layer 3: D3-Force (10%) - Collision detection');
console.log('  Layer 4: Capella - Style refinement\n');

renderOperationalActivityHybrid(operationalModel)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Hybrid diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Layout Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Engine: ${result.metadata.layoutEngine}`);
    console.log(`  - Activities: ${result.metadata.activityCount}`);
    console.log(`  - Exchanges: ${result.metadata.exchangeCount}\n`);
    console.log('🎯 Quality Scores:');
    console.log(`  - ELK: ${result.metadata.qualityScores.elk.toFixed(2)}`);
    console.log(`  - Dagre: ${result.metadata.qualityScores.dagre.toFixed(2)}`);
    console.log(`  - D3: ${result.metadata.qualityScores.d3.toFixed(2)}`);
    console.log(`  - Optimization Time: ${result.metadata.optimizationTime.toFixed(2)}ms\n`);
    console.log('🎉 Done! Open the SVG file to view the hybrid-optimized diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
