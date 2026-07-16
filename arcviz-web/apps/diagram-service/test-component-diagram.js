const fs = require('fs');
const path = require('path');

// Import the component diagram renderer
const { renderComponentArchitecture } = require('./dist/renderers/component');

// Get input/output from command line or use defaults
const inputFile = process.argv[2] || 'sample-component.json';
const outputFile = process.argv[3] || 'component-diagram.svg';

console.log('🚀 Component Diagram Renderer Test\n');
console.log('📁 Input: ', inputFile);
console.log('📄 Output:', outputFile);
console.log('');

// Read the JSON model
let model;
try {
    const jsonData = fs.readFileSync(inputFile, 'utf8');
    model = JSON.parse(jsonData);
    console.log('✓ Loaded model');
} catch (error) {
    console.error('✗ Failed to load model:', error.message);
    process.exit(1);
}

// Check if we have logical_architecture data
if (!model.logical_architecture || model.logical_architecture.length === 0) {
    console.error('✗ No logical_architecture found in model');
    process.exit(1);
}

const logicalArch = model.logical_architecture[0];
console.log(`🚀 Rendering: "${logicalArch.name}"`);
console.log(`  - Components: ${logicalArch.components.length}`);
console.log(`  - Exchanges: ${logicalArch.component_exchanges.length}`);
console.log('');

// Render the diagram
renderComponentArchitecture(logicalArch)
    .then(result => {
        // Write the SVG to file
        fs.writeFileSync(outputFile, result.svg);
        
        console.log('✅ Component diagram rendered successfully!');
        console.log(`  - Width: ${result.width}px`);
        console.log(`  - Height: ${result.height}px`);
        console.log(`  - Output: ${outputFile}`);
        console.log('');
        
        console.log('📊 Metadata:');
        console.log(`  - Type: ${result.metadata.diagramType}`);
        console.log(`  - Components: ${result.metadata.componentCount}`);
        console.log(`  - Exchanges: ${result.metadata.exchangeCount}`);
        console.log('');
        
        console.log('🎉 Done! Open the SVG file to view the component diagram.');
    })
    .catch(error => {
        console.error('✗ Rendering failed:', error.message);
        console.error(error.stack);
        process.exit(1);
    });
