#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { renderDataflow } = require('./dist/renderers/dataflow');

console.log('📊 Dataflow Diagram Renderer Test\n');

// Parse command line arguments
const args = process.argv.slice(2);
if (args.length < 2) {
  console.error('Usage: node test-dataflow.js <input-json> <output-html>');
  process.exit(1);
}

const inputPath = path.resolve(args[0]);
const outputPath = path.resolve(args[1]);

console.log(`📁 Input:  ${inputPath}`);
console.log(`📄 Output: ${outputPath}\n`);

// Load model
let model;
try {
  const jsonContent = fs.readFileSync(inputPath, 'utf-8');
  model = JSON.parse(jsonContent);
  console.log('✓ Loaded model');
} catch (error) {
  console.error(`✗ Failed to load model: ${error.message}`);
  process.exit(1);
}

// Check for system analysis
if (!model.system_analysis || model.system_analysis.length === 0) {
  console.error('✗ No system analysis found in model');
  process.exit(1);
}

const sa = model.system_analysis[0];

console.log(`📊 Rendering: "${sa.name}"`);
console.log(`  - Functions: ${sa.functions.length}`);
console.log(`  - Exchanges: ${sa.functional_exchanges?.length || 0}\n`);

// Render diagram
renderDataflow(sa)
  .then((output) => {
    // Write SVG to file
    fs.writeFileSync(outputPath, output.svg, 'utf-8');
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${output.width}px`);
    console.log(`  - Height: ${output.height}px`);
    console.log(`  - Output: ${outputPath}\n`);
    
    console.log('📊 Metadata:');
    console.log(`  - Type: ${output.metadata.diagramType}`);
    console.log(`  - Functions: ${output.metadata.functionCount}`);
    console.log(`  - Exchanges: ${output.metadata.exchangeCount}\n`);
    
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch((error) => {
    console.error(`✗ Rendering failed: ${error.message}`);
    console.error(error.stack);
    process.exit(1);
  });
