#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { renderClassDiagram } = require('./dist/renderers/classdiagram');

async function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 2) {
    console.error('Usage: node test-classdiagram.js <input.json> <output.svg>');
    process.exit(1);
  }

  const inputFile = args[0];
  const outputFile = args[1];

  const data = JSON.parse(fs.readFileSync(inputFile, 'utf-8'));
  
  if (!data.class_models || data.class_models.length === 0) {
    console.error('Error: No class_models found in input JSON');
    process.exit(1);
  }

  const classModel = data.class_models[0];

  console.log(`Rendering class diagram: ${classModel.name}`);
  console.log(`  - ${classModel.classes?.length || 0} classes`);
  console.log(`  - ${classModel.interfaces?.length || 0} interfaces`);
  console.log(`  - ${classModel.data_structures?.length || 0} data structures`);

  const result = await renderClassDiagram(classModel);

  fs.writeFileSync(outputFile, result.svg);

  console.log(`\nOutput written to: ${outputFile}`);
  console.log(`  - Dimensions: ${result.width}x${result.height}`);
  console.log(`  - Classes: ${result.metadata.classCount}`);
  console.log(`  - Interfaces: ${result.metadata.interfaceCount}`);
  console.log(`  - Data Structures: ${result.metadata.dataStructureCount}`);
}

main().catch(err => {
  console.error('Error:', err.message);
  console.error(err.stack);
  process.exit(1);
});
