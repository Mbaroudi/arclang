const assert = require('assert');

const API_URL = process.env.API_URL || 'http://localhost:3001';

const testArcLangCode = `
operational_analysis "Test Context" {
  actor "User" {
    id: "ACT-001"
    description: "System user"
  }
  
  actor "System" {
    id: "ACT-002"
    description: "Main system"
  }
}

system_analysis "Requirements" {
  requirement "REQ-001" {
    id: "REQ-001"
    description: "System shall respond within 1 second"
    priority: "High"
  }
  
  system_function "Process Request" {
    id: "SF-001"
    description: "Handle incoming requests"
  }
}

logical_architecture "Architecture" {
  component "Frontend" {
    id: "LC-001"
    type: "Logical"
    description: "User interface component"
    
    function "Render UI" {
      id: "LF-001"
      description: "Display user interface"
    }
  }
  
  component "Backend" {
    id: "LC-002"
    type: "Logical"
    description: "Business logic component"
    
    function "Process Data" {
      id: "LF-002"
      description: "Handle data processing"
    }
  }
  
  component "Database" {
    id: "LC-003"
    type: "Logical"
    description: "Data persistence layer"
    
    function "Store Data" {
      id: "LF-003"
      description: "Persist information"
    }
  }
}

physical_architecture "Deployment" {
  node "Web Server" {
    id: "PN-001"
    description: "Frontend hosting"
  }
  
  node "App Server" {
    id: "PN-002"
    description: "Backend processing"
  }
}

trace "LC-001" satisfies "REQ-001" {
  rationale: "Frontend ensures responsive UI"
}
`;

async function testApiDiagramGeneration() {
  console.log('\n=== Testing API Diagram Generation ===\n');
  
  const tests = [
    { name: 'Generate single diagram with code', endpoint: '/api/diagrams/generate' },
    { name: 'Generate all diagrams with code', endpoint: '/api/diagrams/generate-all' },
    { name: 'Get diagram types', endpoint: '/api/diagrams/types' }
  ];
  
  let passed = 0;
  let failed = 0;
  
  for (const test of tests) {
    try {
      console.log(`Testing: ${test.name}...`);
      
      if (test.endpoint === '/api/diagrams/generate') {
        const response = await fetch(`${API_URL}${test.endpoint}`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            diagramType: 'operational',
            code: testArcLangCode
          })
        });
        
        assert.strictEqual(response.ok, true, `Expected 200, got ${response.status}`);
        
        const data = await response.json();
        assert.strictEqual(data.success, true, 'Expected success: true');
        assert.ok(data.svg, 'Expected SVG content');
        assert.strictEqual(data.diagramType, 'operational', 'Expected operational diagram type');
        
        console.log(`✓ ${test.name} - PASSED`);
        console.log(`  - SVG size: ${data.size?.width}x${data.size?.height}`);
        console.log(`  - Elements: ${data.elementCount}`);
        passed++;
        
      } else if (test.endpoint === '/api/diagrams/generate-all') {
        const response = await fetch(`${API_URL}${test.endpoint}`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            code: testArcLangCode
          })
        });
        
        assert.strictEqual(response.ok, true, `Expected 200, got ${response.status}`);
        
        const data = await response.json();
        assert.strictEqual(data.success, true, 'Expected success: true');
        assert.strictEqual(data.total, 10, 'Expected 10 diagram types');
        assert.ok(data.successful > 0, 'Expected at least some successful generations');
        assert.ok(Array.isArray(data.diagrams), 'Expected diagrams array');
        
        console.log(`✓ ${test.name} - PASSED`);
        console.log(`  - Total: ${data.total}, Successful: ${data.successful}, Failed: ${data.failed}`);
        
        data.diagrams.slice(0, 3).forEach(diagram => {
          console.log(`  - ${diagram.type}: ${diagram.elementCount} elements`);
        });
        
        passed++;
        
      } else if (test.endpoint === '/api/diagrams/types') {
        const response = await fetch(`${API_URL}${test.endpoint}`);
        
        assert.strictEqual(response.ok, true, `Expected 200, got ${response.status}`);
        
        const data = await response.json();
        assert.ok(Array.isArray(data.types), 'Expected types array');
        assert.strictEqual(data.types.length, 10, 'Expected 10 diagram types');
        
        const expectedTypes = [
          'operational', 'functional', 'component', 'sequence',
          'state-machine', 'physical', 'class', 'tree',
          'capability', 'functional-chain'
        ];
        
        const returnedTypes = data.types.map(t => t.id);
        expectedTypes.forEach(type => {
          assert.ok(returnedTypes.includes(type), `Expected type: ${type}`);
        });
        
        console.log(`✓ ${test.name} - PASSED`);
        console.log(`  - Available types: ${returnedTypes.join(', ')}`);
        passed++;
      }
      
    } catch (error) {
      console.log(`✗ ${test.name} - FAILED`);
      console.log(`  Error: ${error.message}`);
      failed++;
    }
  }
  
  console.log('\n--- API Test Summary ---');
  console.log(`Passed: ${passed}/${tests.length}`);
  console.log(`Failed: ${failed}/${tests.length}`);
  
  return { passed, failed, total: tests.length };
}

async function testApiWithoutCode() {
  console.log('\n=== Testing API with Sample Data (no code) ===\n');
  
  try {
    console.log('Testing: Generate diagram without code (should use sample data)...');
    
    const response = await fetch(`${API_URL}/api/diagrams/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        diagramType: 'operational'
      })
    });
    
    assert.strictEqual(response.ok, true, `Expected 200, got ${response.status}`);
    
    const data = await response.json();
    assert.strictEqual(data.success, true, 'Expected success: true');
    assert.ok(data.svg, 'Expected SVG content');
    
    console.log('✓ Sample data fallback works - PASSED');
    console.log(`  - SVG size: ${data.size?.width}x${data.size?.height}`);
    
    return { passed: 1, failed: 0, total: 1 };
    
  } catch (error) {
    console.log('✗ Sample data fallback - FAILED');
    console.log(`  Error: ${error.message}`);
    return { passed: 0, failed: 1, total: 1 };
  }
}

async function testApiErrorHandling() {
  console.log('\n=== Testing API Error Handling ===\n');
  
  const tests = [
    { 
      name: 'Invalid diagram type',
      body: { diagramType: 'invalid-type', code: testArcLangCode },
      expectedStatus: 400
    },
    { 
      name: 'Empty request body',
      body: {},
      expectedStatus: 400
    }
  ];
  
  let passed = 0;
  let failed = 0;
  
  for (const test of tests) {
    try {
      console.log(`Testing: ${test.name}...`);
      
      const response = await fetch(`${API_URL}/api/diagrams/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(test.body)
      });
      
      assert.strictEqual(response.status, test.expectedStatus, 
        `Expected ${test.expectedStatus}, got ${response.status}`);
      
      const data = await response.json();
      assert.ok(data.error, 'Expected error message');
      
      console.log(`✓ ${test.name} - PASSED`);
      console.log(`  - Error message: "${data.error}"`);
      passed++;
      
    } catch (error) {
      console.log(`✗ ${test.name} - FAILED`);
      console.log(`  Error: ${error.message}`);
      failed++;
    }
  }
  
  console.log('\n--- Error Handling Test Summary ---');
  console.log(`Passed: ${passed}/${tests.length}`);
  console.log(`Failed: ${failed}/${tests.length}`);
  
  return { passed, failed, total: tests.length };
}

async function runAllApiTests() {
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║   ArcViz API Diagram Generation Test Suite    ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log(`\nAPI URL: ${API_URL}\n`);
  
  const results = [];
  
  results.push(await testApiDiagramGeneration());
  results.push(await testApiWithoutCode());
  results.push(await testApiErrorHandling());
  
  const totalPassed = results.reduce((sum, r) => sum + r.passed, 0);
  const totalFailed = results.reduce((sum, r) => sum + r.failed, 0);
  const totalTests = results.reduce((sum, r) => sum + r.total, 0);
  
  console.log('\n╔════════════════════════════════════════════════╗');
  console.log('║           OVERALL TEST SUMMARY                 ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log(`\nTotal Tests: ${totalTests}`);
  console.log(`✓ Passed: ${totalPassed}`);
  console.log(`✗ Failed: ${totalFailed}`);
  console.log(`Success Rate: ${((totalPassed / totalTests) * 100).toFixed(1)}%\n`);
  
  process.exit(totalFailed > 0 ? 1 : 0);
}

if (require.main === module) {
  runAllApiTests().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { runAllApiTests };
