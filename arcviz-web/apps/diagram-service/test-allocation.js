#!/usr/bin/env node

const fs = require('fs');
const { renderAllocationDiagram } = require('./dist/renderers/allocation');

async function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 2) {
    console.error('Usage: node test-allocation.js <input.json> <output.svg>');
    process.exit(1);
  }

  const inputFile = args[0];
  const outputFile = args[1];

  const data = JSON.parse(fs.readFileSync(inputFile, 'utf-8'));
  
  if (!data.system_analysis || !data.logical_architecture) {
    console.error('Error: Need both system_analysis and logical_architecture in input JSON');
    process.exit(1);
  }

  console.log(`Rendering allocation diagram`);
  console.log(`  - ${data.system_analysis.functions?.length || 0} functions`);
  console.log(`  - ${data.logical_architecture.components?.length || 0} components`);
  console.log(`  - ${data.logical_architecture.unallocated_functions?.length || 0} unallocated`);

  const result = await renderAllocationDiagram(data);

  fs.writeFileSync(outputFile, result.svg);

  console.log(`\nOutput written to: ${outputFile}`);
  console.log(`  - Dimensions: ${result.width}x${result.height}`);
  console.log(`  - Allocations: ${result.metadata.allocationCount}`);
  console.log(`  - Unallocated: ${result.metadata.unallocatedCount}`);
}

main().catch(err => {
  console.error('Error:', err.message);
  console.error(err.stack);
  process.exit(1);
});
