#\!/usr/bin/env node

const fs = require('fs');
const { renderSystemContext } = require('./dist/renderers/system-context');

async function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 2) {
    console.error('Usage: node test-system-context.js <input.json> <output.svg>');
    process.exit(1);
  }

  const inputFile = args[0];
  const outputFile = args[1];

  const data = JSON.parse(fs.readFileSync(inputFile, 'utf-8'));
  
  if (\!data.system_analysis || data.system_analysis.length === 0) {
    console.error('Error: No system_analysis found in input JSON');
    process.exit(1);
  }

  const systemAnalysis = data.system_analysis[0];

  console.log(`Rendering system context: ${systemAnalysis.name}`);
  console.log(`  - ${systemAnalysis.external_actors?.length || 0} actors`);
  console.log(`  - ${systemAnalysis.functions?.length || 0} functions`);

  const result = await renderSystemContext(systemAnalysis);

  fs.writeFileSync(outputFile, result.svg);

  console.log(`\nOutput written to: ${outputFile}`);
  console.log(`  - Dimensions: ${result.width}x${result.height}`);
  console.log(`  - Actors: ${result.metadata.actorCount}`);
}

main().catch(err => {
  console.error('Error:', err.message);
  console.error(err.stack);
  process.exit(1);
});
