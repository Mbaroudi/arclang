/**
 * Test script for functional chain diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderFunctionalChainDiagram } = require('./dist/renderers/functional-chain');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-functional-chain.json';
const outputFile = args[1] || 'emergency-stop-chain.svg';

console.log('⛓️  Functional Chain Diagram Renderer Test\n');
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

const chainModel = {
  name: model.name,
  functions: model.functions || [],
  functional_exchanges: model.functional_exchanges || [],
  execution_order: model.execution_order || [],
};

console.log(`⛓️  Rendering: "${chainModel.name}"`);
console.log(`  - Functions: ${chainModel.functions.length}`);
console.log(`  - Exchanges: ${chainModel.functional_exchanges.length}\n`);

renderFunctionalChainDiagram(chainModel)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Chain: ${result.metadata.chainName}`);
    console.log(`  - Functions: ${result.metadata.functionCount}`);
    console.log(`  - Exchanges: ${result.metadata.exchangeCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
