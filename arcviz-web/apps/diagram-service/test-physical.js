/**
 * Test script for physical architecture diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderPhysicalArchitecture } = require('./dist/renderers/physical');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-physical.json';
const outputFile = args[1] || 'avionics-physical.svg';

console.log('🎨 Physical Architecture Diagram Renderer Test\n');
console.log(`📁 Input:  ${jsonFile}`);
console.log(`📄 Output: ${outputFile}\n`);

// Load JSON model
const jsonPath = path.resolve(__dirname, jsonFile);
if (!fs.existsSync(jsonPath)) {
  console.error(`❌ Error: Input file not found: ${jsonPath}`);
  process.exit(1);
}

const modelJson = fs.readFileSync(jsonPath, 'utf-8');
const model = JSON.parse(modelJson);

console.log('✓ Loaded model');

if (model.physical_architecture.length === 0) {
  console.error('❌ Error: No physical architecture found in model');
  process.exit(1);
}

const pa = model.physical_architecture[0];

console.log(`🎨 Rendering: "${pa.name}"`);
console.log(`  - Physical Nodes: ${pa.nodes.length}`);
console.log(`  - Physical Links: ${pa.links.length}`);
console.log(`  - Physical Exchanges: ${pa.physical_exchanges.length}\n`);

// Render diagram
renderPhysicalArchitecture(pa)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Nodes: ${result.metadata.nodeCount}`);
    console.log(`  - Links: ${result.metadata.linkCount}`);
    console.log(`  - Exchanges: ${result.metadata.exchangeCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
