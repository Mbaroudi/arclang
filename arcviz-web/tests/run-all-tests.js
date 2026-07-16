#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

const API_URL = process.env.API_URL || 'http://localhost:3001';
const WEB_URL = process.env.WEB_URL || 'http://localhost:3002';

const TEST_SUITES = [
  {
    name: 'API Diagram Generation Tests',
    file: 'test-api-diagrams.js',
    description: 'Tests API endpoints for diagram generation with dynamic code parsing',
    category: 'API'
  },
  {
    name: 'Editor Interface Tests',
    file: 'test-editor-selenium.js',
    description: 'Tests Monaco editor, code editing, and localStorage integration',
    category: 'Frontend'
  },
  {
    name: 'Visualizer Interface Tests',
    file: 'test-visualizer-selenium.js',
    description: 'Tests diagram visualization, generation, and display',
    category: 'Frontend'
  },
  {
    name: 'End-to-End Workflow Tests',
    file: 'test-e2e-workflow.js',
    description: 'Tests complete workflow from editing to visualization',
    category: 'E2E'
  },
  {
    name: 'MCP Integration Tests',
    file: 'test-mcp-integration.js',
    description: 'Tests MCP server integration and AI capabilities',
    category: 'Integration'
  }
];

function runTest(testFile) {
  return new Promise((resolve, reject) => {
    const testPath = path.join(__dirname, testFile);
    const proc = spawn('node', [testPath], {
      stdio: 'inherit',
      env: {
        ...process.env,
        API_URL,
        WEB_URL
      }
    });
    
    proc.on('close', (code) => {
      if (code === 0) {
        resolve({ success: true, file: testFile });
      } else {
        resolve({ success: false, file: testFile, exitCode: code });
      }
    });
    
    proc.on('error', (error) => {
      reject({ success: false, file: testFile, error: error.message });
    });
  });
}

async function checkServers() {
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║         Server Availability Check             ║');
  console.log('╚════════════════════════════════════════════════╝\n');
  
  let apiAvailable = false;
  let webAvailable = false;
  
  try {
    console.log(`Checking API server at ${API_URL}...`);
    const apiResponse = await fetch(`${API_URL}/api/diagrams/types`).catch(() => null);
    apiAvailable = apiResponse && apiResponse.ok;
    
    if (apiAvailable) {
      console.log('✓ API server is accessible');
    } else {
      console.log('✗ API server is NOT accessible');
      console.log('  Please start the API server: cd apps/api && npm run dev');
    }
  } catch (error) {
    console.log('✗ API server check failed:', error.message);
  }
  
  try {
    console.log(`\nChecking Web server at ${WEB_URL}...`);
    const webResponse = await fetch(WEB_URL).catch(() => null);
    webAvailable = webResponse && webResponse.ok;
    
    if (webAvailable) {
      console.log('✓ Web server is accessible');
    } else {
      console.log('✗ Web server is NOT accessible');
      console.log('  Please start the Web server: cd apps/web && npm run dev');
    }
  } catch (error) {
    console.log('✗ Web server check failed:', error.message);
  }
  
  console.log('\n' + '─'.repeat(52) + '\n');
  
  return { apiAvailable, webAvailable };
}

async function runAllTests(options = {}) {
  console.log('\n');
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║    ArcViz Comprehensive Test Suite Runner     ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log('\n');
  console.log(`API URL: ${API_URL}`);
  console.log(`Web URL: ${WEB_URL}`);
  console.log(`Mode: ${options.quick ? 'Quick (API only)' : 'Full'}\n`);
  
  const serverStatus = await checkServers();
  
  if (!serverStatus.apiAvailable) {
    console.log('\n⚠ WARNING: API server is not available!');
    console.log('Some tests will fail. Please start servers and try again.\n');
    
    if (!options.force) {
      console.log('Use --force to run tests anyway\n');
      process.exit(1);
    }
  }
  
  if (!serverStatus.webAvailable && !options.quick) {
    console.log('\n⚠ WARNING: Web server is not available!');
    console.log('Frontend and E2E tests will fail. Please start servers and try again.\n');
    
    if (!options.force) {
      console.log('Use --force to run tests anyway\n');
      process.exit(1);
    }
  }
  
  const suitesToRun = options.quick 
    ? TEST_SUITES.filter(s => s.category === 'API' || s.category === 'Integration')
    : TEST_SUITES;
  
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║            Running Test Suites                 ║');
  console.log('╚════════════════════════════════════════════════╝\n');
  
  const results = [];
  
  for (let i = 0; i < suitesToRun.length; i++) {
    const suite = suitesToRun[i];
    
    console.log(`\n[${i + 1}/${suitesToRun.length}] ${suite.name}`);
    console.log(`Category: ${suite.category}`);
    console.log(`Description: ${suite.description}`);
    console.log('─'.repeat(52));
    
    try {
      const result = await runTest(suite.file);
      results.push({ ...suite, ...result });
      
      if (result.success) {
        console.log(`\n✓ ${suite.name} completed successfully\n`);
      } else {
        console.log(`\n✗ ${suite.name} failed (exit code: ${result.exitCode})\n`);
      }
    } catch (error) {
      console.log(`\n✗ ${suite.name} error: ${error.error || error.message}\n`);
      results.push({ ...suite, success: false, error: error.error || error.message });
    }
    
    console.log('═'.repeat(52));
  }
  
  console.log('\n\n');
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║          FINAL TEST RESULTS SUMMARY            ║');
  console.log('╚════════════════════════════════════════════════╝\n');
  
  const successful = results.filter(r => r.success);
  const failed = results.filter(r => !r.success);
  
  console.log(`Total Test Suites: ${results.length}`);
  console.log(`✓ Passed: ${successful.length}`);
  console.log(`✗ Failed: ${failed.length}`);
  console.log(`Success Rate: ${((successful.length / results.length) * 100).toFixed(1)}%\n`);
  
  if (successful.length > 0) {
    console.log('✓ Successful Test Suites:');
    successful.forEach(s => {
      console.log(`  - ${s.name} (${s.category})`);
    });
    console.log('');
  }
  
  if (failed.length > 0) {
    console.log('✗ Failed Test Suites:');
    failed.forEach(s => {
      console.log(`  - ${s.name} (${s.category})`);
      if (s.exitCode) {
        console.log(`    Exit code: ${s.exitCode}`);
      }
      if (s.error) {
        console.log(`    Error: ${s.error}`);
      }
    });
    console.log('');
  }
  
  console.log('═'.repeat(52));
  console.log('');
  console.log('Test Categories:');
  console.log('  - API: API endpoint testing with dynamic code parsing');
  console.log('  - Frontend: Editor and Visualizer UI testing with Selenium');
  console.log('  - E2E: End-to-end workflow validation');
  console.log('  - Integration: MCP server and AI capabilities');
  console.log('');
  console.log('Environment Variables:');
  console.log(`  - API_URL=${API_URL}`);
  console.log(`  - WEB_URL=${WEB_URL}`);
  console.log(`  - MCP_SERVER_URL=${process.env.MCP_SERVER_URL || 'Not set'}`);
  console.log('');
  console.log('For individual test runs:');
  console.log('  node test-api-diagrams.js');
  console.log('  node test-editor-selenium.js');
  console.log('  node test-visualizer-selenium.js');
  console.log('  node test-e2e-workflow.js');
  console.log('  node test-mcp-integration.js');
  console.log('');
  
  process.exit(failed.length > 0 ? 1 : 0);
}

function printHelp() {
  console.log(`
ArcViz Comprehensive Test Suite Runner

Usage:
  node run-all-tests.js [options]

Options:
  --quick     Run only API and Integration tests (skip Selenium tests)
  --force     Run tests even if servers are not available
  --help      Show this help message

Environment Variables:
  API_URL     API server URL (default: http://localhost:3001)
  WEB_URL     Web server URL (default: http://localhost:3002)
  MCP_SERVER_URL   MCP server URL (optional)

Examples:
  node run-all-tests.js
  node run-all-tests.js --quick
  API_URL=http://localhost:3001 node run-all-tests.js
  node run-all-tests.js --force --quick

Test Suites:
  1. API Diagram Generation Tests
     - Tests dynamic code parsing
     - Tests diagram generation endpoints
     - Tests error handling

  2. Editor Interface Tests (Selenium)
     - Tests Monaco editor functionality
     - Tests code editing and saving
     - Tests localStorage integration

  3. Visualizer Interface Tests (Selenium)
     - Tests diagram visualization
     - Tests generation UI
     - Tests responsive layout

  4. End-to-End Workflow Tests (Selenium)
     - Tests complete user workflow
     - Tests editor → visualizer flow
     - Tests code persistence

  5. MCP Integration Tests
     - Tests MCP server integration
     - Tests AI capabilities
     - Tests code generation and review
`);
}

const args = process.argv.slice(2);

if (args.includes('--help') || args.includes('-h')) {
  printHelp();
  process.exit(0);
}

const options = {
  quick: args.includes('--quick'),
  force: args.includes('--force')
};

runAllTests(options).catch(error => {
  console.error('\n✗ Fatal error running tests:', error);
  process.exit(1);
});
