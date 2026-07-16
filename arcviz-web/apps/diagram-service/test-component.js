/**
 * Test script for component architecture diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderComponentArchitecture } = require('./dist/renderers/component');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-component.json';
const outputFile = args[1] || 'vehicle-component.svg';

console.log('🎨 Component Architecture Diagram Renderer Test\n');
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

if (model.logical_architecture.length === 0) {
  console.error('❌ Error: No logical architecture found in model');
  process.exit(1);
}

const la = model.logical_architecture[0];

console.log(`🎨 Rendering: "${la.name}"`);
console.log(`  - Components: ${la.components.length}`);
console.log(`  - Component Exchanges: ${la.component_exchanges.length}`);
console.log(`  - Interfaces: ${la.interfaces.length}\n`);

// Render diagram
renderComponentArchitecture(la)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Components: ${result.metadata.componentCount}`);
    console.log(`  - Exchanges: ${result.metadata.exchangeCount}`);
    console.log(`  - Interfaces: ${result.metadata.interfaceCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
