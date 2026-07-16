/**
 * Test script for operational activity diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderOperationalActivity } = require('./dist/renderers/operational');

const args = process.argv.slice(2);
const jsonFile = args[0] || 'sample-operational.json';
const outputFile = args[1] || 'operational.svg';

console.log('🏊 Operational Activity Diagram Renderer Test\n');
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

// Extract operational analysis
const operationalModel = {
  name: model.operational_analysis?.[0]?.name || "Operational Analysis",
  actors: model.operational_analysis?.[0]?.actors || [],
  entities: model.operational_analysis?.[0]?.entities || [],
  activities: model.operational_analysis?.[0]?.activities || [],
  exchanges: model.operational_analysis?.[0]?.exchanges || [],
};

console.log(`🏊 Rendering: "${operationalModel.name}"`);
console.log(`  - Actors: ${operationalModel.actors.length}`);
console.log(`  - Entities: ${operationalModel.entities.length}`);
console.log(`  - Activities: ${operationalModel.activities.length}`);
console.log(`  - Exchanges: ${operationalModel.exchanges.length}\n`);

renderOperationalActivity(operationalModel)
  .then(result => {
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Actors: ${result.metadata.actorCount}`);
    console.log(`  - Activities: ${result.metadata.activityCount}\n`);
    console.log('🎉 Done! Open the SVG file to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
