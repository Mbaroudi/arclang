/**
 * Test script for capability diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderCapabilityDiagram } = require('./dist/renderers/capability');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-capability.json';
const outputFile = args[1] || 'vehicle-capability.svg';

console.log('🎯 Capability Diagram Renderer Test\n');
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

const capabilityModel = {
  capabilities: model.capabilities || [],
  capability_associations: model.capability_associations || [],
};

console.log(`🎯 Rendering: "Capability Diagram"`);
console.log(`  - Capabilities: ${countCapabilities(capabilityModel.capabilities)}`);
console.log(`  - Associations: ${capabilityModel.capability_associations.length}\n`);

function countCapabilities(caps) {
  let count = 0;
  for (const cap of caps) {
    count++;
    if (cap.children && cap.children.length > 0) {
      count += countCapabilities(cap.children);
    }
  }
  return count;
}

renderCapabilityDiagram(capabilityModel)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Capabilities: ${result.metadata.capabilityCount}`);
    console.log(`  - Associations: ${result.metadata.associationCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
