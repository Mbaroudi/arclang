/**
 * Test script for state machine diagram renderer
 */

const fs = require('fs');
const path = require('path');
const { renderStateMachine } = require('./dist/renderers/statemachine');

async function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 2) {
    console.error('Usage: node test-statemachine.js <input.json> <output.svg>');
    process.exit(1);
  }

  const inputFile = args[0];
  const outputFile = args[1];

  const data = JSON.parse(fs.readFileSync(inputFile, 'utf-8'));
  
  if (!data.state_machines || data.state_machines.length === 0) {
    console.error('Error: No state_machines found in input JSON');
    process.exit(1);
  }

  const stateMachine = data.state_machines[0];

  console.log(`Rendering state machine: ${stateMachine.name}`);
  console.log(`  - ${stateMachine.states.length} states`);
  console.log(`  - ${stateMachine.transitions.length} transitions`);

  const result = await renderStateMachine(stateMachine);

  fs.writeFileSync(outputFile, result.svg);

  console.log(`\nOutput written to: ${outputFile}`);
  console.log(`  - Dimensions: ${result.width}x${result.height}`);
  console.log(`  - States: ${result.metadata.stateCount}`);
  console.log(`  - Transitions: ${result.metadata.transitionCount}`);
}

main().catch(err => {
  console.error('Error:', err.message);
  process.exit(1);
});
