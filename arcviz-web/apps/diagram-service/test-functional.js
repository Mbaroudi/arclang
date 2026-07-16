/**
 * Test script for functional dataflow diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderFunctionalDataflow } = require('./dist/renderers/functional');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-functional.json';
const outputFile = args[1] || 'camera-functional.svg';

console.log('🎨 Functional Dataflow Renderer Test\n');
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

if (model.system_analysis.length === 0) {
  console.error('❌ Error: No system analysis found in model');
  process.exit(1);
}

const sa = model.system_analysis[0];

console.log(`🎨 Rendering: "${sa.name}"`);
console.log(`  - Functions: ${sa.functions.length}`);
console.log(`  - Functional Exchanges: ${sa.functional_exchanges.length}`);
console.log(`  - External Actors: ${sa.external_actors.length}\n`);

// Count total ports
let totalPorts = 0;
for (const func of sa.functions) {
  totalPorts += func.ports.length;
}
console.log(`  - Total Ports: ${totalPorts}\n`);

// Render diagram
renderFunctionalDataflow(sa)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Functions: ${result.metadata.functionCount}`);
    console.log(`  - Exchanges: ${result.metadata.exchangeCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
