/**
 * Test script for tree diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderTreeDiagram } = require('./dist/renderers/tree');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-tree.json';
const outputFile = args[1] || 'vehicle-tree.svg';

console.log('🌳 Tree Diagram Renderer Test\n');
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

const treeModel = {
  tree_type: model.tree_type,
  functions: model.functions || [],
  components: model.components || [],
};

console.log(`🌳 Rendering: "${treeModel.tree_type === 'function' ? 'Function' : 'Component'} Breakdown"`);
if (treeModel.tree_type === 'function') {
  console.log(`  - Functions: ${treeModel.functions.length}`);
} else {
  console.log(`  - Components: ${treeModel.components.length}`);
}
console.log('');

renderTreeDiagram(treeModel)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Tree Type: ${result.metadata.treeType}`);
    console.log(`  - Nodes: ${result.metadata.nodeCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
