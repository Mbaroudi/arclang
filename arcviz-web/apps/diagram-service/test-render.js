/**
 * Test script for operational activity diagram renderer
 * 
 * Usage: node test-render.js <json-file> <output-svg>
 */

const fs = require('fs');
const path = require('path');
const { renderOperationalActivity } = require('./dist/renderers/operational');

// Get command line arguments
const args = process.argv.slice(2);
const jsonFile = args[0] || '../../../acc_minimal.json';
const outputFile = args[1] || './test-output.svg';

console.log('🎨 ArcLang Diagram Service - Operational Activity Renderer Test\n');
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
console.log(`  - Operational Analyses: ${model.operational_analysis.length}`);
console.log(`  - System Analyses: ${model.system_analysis.length}`);
console.log(`  - Logical Architectures: ${model.logical_architecture.length}`);
console.log(`  - Physical Architectures: ${model.physical_architecture.length}\n`);

// Check if we have operational analysis
if (model.operational_analysis.length === 0) {
  console.warn('⚠️  Warning: No operational analysis found in model');
  console.log('Creating a sample operational analysis...\n');
  
  // Create a simple test model
  model.operational_analysis = [
    {
      name: 'Sample Operations',
      actors: [
        { name: 'User', id: null, icon: 'person', attributes: {} }
      ],
      entities: [
        { 
          id: 'E1', 
          name: 'User', 
          entity_type: 'Actor', 
          icon: 'person',
          description: 'System user',
          attributes: {}
        }
      ],
      capabilities: [],
      activities: [
        {
          id: 'OA-001',
          name: 'Start Process',
          performed_by: 'User',
          category: 'general',
          icon: 'circle',
          color: '#FFD966',
          sub_activities: [],
          attributes: {}
        },
        {
          id: 'OA-002',
          name: 'Execute Task',
          performed_by: 'User',
          category: 'general',
          icon: 'circle',
          color: '#FFD966',
          sub_activities: [],
          attributes: {}
        },
        {
          id: 'OA-003',
          name: 'Complete Process',
          performed_by: 'User',
          category: 'general',
          icon: 'circle',
          color: '#FFD966',
          sub_activities: [],
          attributes: {}
        }
      ],
      exchanges: [
        {
          from: 'OA-001',
          to: 'OA-002',
          data_type: 'Data',
          label: 'Process Data',
          protocol: null,
          attributes: {}
        },
        {
          from: 'OA-002',
          to: 'OA-003',
          data_type: 'Result',
          label: 'Task Result',
          protocol: null,
          attributes: {}
        }
      ],
      capability_associations: []
    }
  ];
}

// Render first operational analysis
const oa = model.operational_analysis[0];

console.log(`🎨 Rendering: "${oa.name}"`);
console.log(`  - Activities: ${oa.activities.length}`);
console.log(`  - Exchanges: ${oa.exchanges.length}`);
console.log(`  - Entities: ${oa.entities.length}\n`);

// Render diagram
renderOperationalActivity(oa)
  .then(result => {
    // Write SVG to file
    fs.writeFileSync(outputFile, result.svg);
    
    console.log('✅ Diagram rendered successfully!');
    console.log(`  - Width: ${result.width}px`);
    console.log(`  - Height: ${result.height}px`);
    console.log(`  - Output: ${outputFile}\n`);
    console.log('📊 Metadata:');
    console.log(`  - Type: ${result.metadata.diagramType}`);
    console.log(`  - Activities: ${result.metadata.activityCount}`);
    console.log(`  - Exchanges: ${result.metadata.exchangeCount}\n`);
    console.log('🎉 Done! Open the SVG file in a browser to view the diagram.');
  })
  .catch(error => {
    console.error('❌ Error rendering diagram:', error);
    console.error(error.stack);
    process.exit(1);
  });
