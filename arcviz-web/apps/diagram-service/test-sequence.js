/**
 * Test script for sequence diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderSequenceDiagram } = require('./dist/renderers/sequence');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-sequence.json';
const outputFile = args[1] || 'auth-sequence.svg';

console.log('🎨 Sequence Diagram Renderer Test\n');
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

if (model.scenarios.length === 0) {
  console.error('❌ Error: No scenarios found in model');
  process.exit(1);
}

const scenario = model.scenarios[0];

console.log(`🎨 Rendering: "${scenario.name}"`);
console.log(`  - Participants: ${scenario.participants.length}`);
console.log(`  - Messages: ${scenario.messages.length}`);
console.log(`  - Fragments: ${scenario.fragments.length}\n`);

// Render diagram
renderSequenceDiagram(scenario)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Participants: ${result.metadata.participantCount}`);
    console.log(`  - Messages: ${result.metadata.messageCount}`);
    console.log(`  - Fragments: ${result.metadata.fragmentCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
