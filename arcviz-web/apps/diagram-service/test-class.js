/**
 * Test script for class/interface diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderClassDiagram } = require('./dist/renderers/class');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-class.json';
const outputFile = args[1] || 'vehicle-class.svg';

console.log('🎨 Class/Interface Diagram Renderer Test\n');
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

const classModel = {
  exchange_items: model.exchange_items || [],
  data_types: model.data_types || [],
};

console.log(`🎨 Rendering: "Data Model"`);
console.log(`  - Exchange Items: ${classModel.exchange_items.length}`);
console.log(`  - Data Types: ${classModel.data_types.length}\n`);

// Render diagram
renderClassDiagram(classModel)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Classes: ${result.metadata.classCount}`);
    console.log(`  - Data Types: ${result.metadata.dataTypeCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
