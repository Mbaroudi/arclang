const fs = require('fs');
const path = require('path');

// Import the functional diagram renderer
const { renderFunctionalDataflow } = require('./dist/renderers/functional');

// Get input/output from command line or use defaults
const inputFile = process.argv[2] || 'sample-functional.json';
const outputFile = process.argv[3] || 'functional-diagram.svg';

console.log('🚀 Functional Dataflow Diagram Renderer Test (with System Boundary)\n');
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

// Check if we have system_analysis data
if (!model.system_analysis || model.system_analysis.length === 0) {
    console.error('✗ No system_analysis found in model');
    process.exit(1);
}

const systemAnalysis = model.system_analysis[0];
console.log(`🚀 Rendering: "${systemAnalysis.name}"`);
console.log(`  - Functions: ${systemAnalysis.functions.length}`);
console.log(`  - Exchanges: ${systemAnalysis.functional_exchanges.length}`);
console.log(`  - Actors: ${systemAnalysis.external_actors.length}`);
console.log('');

// Render the diagram
renderFunctionalDataflow(systemAnalysis)
    .then(result => {
        // Write the SVG to file
        fs.writeFileSync(outputFile, result.svg);
        
        console.log('✅ Functional diagram rendered successfully!');
        console.log(`  - Width: ${result.width}px`);
        console.log(`  - Height: ${result.height}px`);
        console.log(`  - Output: ${outputFile}` );
        console.log('');
        
        console.log('📊 Metadata:');
        console.log(`  - Type: ${result.metadata.diagramType}`);
        console.log(`  - Functions: ${result.metadata.functionCount}`);
        console.log(`  - Exchanges: ${result.metadata.exchangeCount}`);
        
        if (result.metadata.systemBoundary !== undefined) {
            console.log(`  - System Boundary: ${result.metadata.systemBoundary ? '✅ VALID' : '❌ VIOLATIONS'}`);
            if (result.metadata.boundaryViolations && result.metadata.boundaryViolations.length > 0) {
                console.log(`  - Violations: ${result.metadata.boundaryViolations.join(', ')}`);
            }
        }
        
        if (result.metadata.portPositioning !== undefined) {
            console.log(`  - Port Positioning: ${result.metadata.portPositioning ? '✅ COMPLIANT' : '❌ VIOLATIONS'}`);
            if (result.metadata.portViolations) {
                console.log(`    Violations: ${result.metadata.portViolations}`);
            }
            if (result.metadata.portWarnings) {
                console.log(`    Warnings: ${result.metadata.portWarnings}`);
            }
            if (result.metadata.portStats) {
                const stats = result.metadata.portStats;
                console.log(`    Total Ports: ${stats.totalPorts} (IN: ${stats.inputPorts}, OUT: ${stats.outputPorts}, INOUT: ${stats.bidirectionalPorts})`);
                console.log(`    By Side: LEFT=${stats.portsBySide.LEFT}, RIGHT=${stats.portsBySide.RIGHT}, TOP=${stats.portsBySide.TOP}, BOTTOM=${stats.portsBySide.BOTTOM}`);
            }
        }
        console.log('');
        
        console.log('🎉 Done! Open the SVG file to view the functional diagram with system boundary.');
    })
    .catch(error => {
        console.error('✗ Rendering failed:', error.message);
        console.error(error.stack);
        process.exit(1);
    });
