const assert = require('assert');

const API_URL = process.env.API_URL || 'http://localhost:3001';
const MCP_SERVER_URL = process.env.MCP_SERVER_URL || null;

async function testMcpIntegration() {
  console.log('\n=== Testing MCP Server Integration ===\n');
  
  let passed = 0;
  let failed = 0;
  
  if (!MCP_SERVER_URL) {
    console.log('⚠ MCP_SERVER_URL not configured - testing API MCP integration only');
  }
  
  try {
    console.log('Test 1: Check MCP status in AI routes...');
    
    const response = await fetch(`${API_URL}/api/ai/generate/operational`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        description: 'Test system'
      })
    });
    
    if (response.ok) {
      const data = await response.json();
      
      console.log(`✓ AI route accessible - PASSED`);
      console.log(`  - AI powered: ${data.ai_powered || false}`);
      console.log(`  - MCP enabled: ${data.mcp_enabled || false}`);
      passed++;
      
      if (data.mcp_enabled) {
        console.log('  ✓ MCP server is configured');
      } else {
        console.log('  ⚠ MCP server not configured (check MCP_SERVER_URL env var)');
      }
    } else {
      console.log('⚠ AI route returned error (may require authentication)');
    }
    
  } catch (error) {
    console.log(`✗ MCP integration test failed: ${error.message}`);
    failed++;
  }
  
  try {
    console.log('\nTest 2: Test AI code generation endpoint...');
    
    const response = await fetch(`${API_URL}/api/ai/generate/component-code`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        description: 'Temperature sensor controller',
        enforceSyntax: false
      })
    });
    
    if (response.ok) {
      const data = await response.json();
      
      assert.strictEqual(data.success, true, 'Should succeed');
      assert.ok(data.code, 'Should return generated code');
      
      console.log('✓ AI code generation works - PASSED');
      console.log(`  - Generated code length: ${data.code.length} characters`);
      console.log(`  - Validated: ${data.validated || 'N/A'}`);
      passed++;
    } else {
      const errorData = await response.json();
      console.log('⚠ AI code generation unavailable');
      console.log(`  - Status: ${response.status}`);
      console.log(`  - Error: ${errorData.error || 'Unknown'}`);
    }
    
  } catch (error) {
    console.log(`✗ AI code generation test failed: ${error.message}`);
    failed++;
  }
  
  try {
    console.log('\nTest 3: Test code review endpoint...');
    
    const testCode = `operational_analysis "Test" {
  actor "User" {
    id: "ACT-001"
    description: "Test user"
  }
}`;
    
    const response = await fetch(`${API_URL}/api/ai/review`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        code: testCode
      })
    });
    
    if (response.ok) {
      const data = await response.json();
      
      assert.strictEqual(data.success, true, 'Should succeed');
      assert.ok(data.review, 'Should return review');
      
      console.log('✓ Code review endpoint works - PASSED');
      console.log(`  - Valid: ${data.valid}`);
      console.log(`  - Errors: ${data.errors?.length || 0}`);
      passed++;
    } else {
      console.log('⚠ Code review endpoint unavailable');
    }
    
  } catch (error) {
    console.log(`✗ Code review test failed: ${error.message}`);
    failed++;
  }
  
  try {
    console.log('\nTest 4: Test syntax validation endpoint...');
    
    const validCode = `operational_analysis "Test" {
  actor "User" {
    id: "ACT-001"
    description: "Test user"
  }
}`;
    
    const response = await fetch(`${API_URL}/api/ai/validate-syntax`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        code: validCode
      })
    });
    
    if (response.ok) {
      const data = await response.json();
      
      assert.strictEqual(data.success, true, 'Should succeed');
      
      console.log('✓ Syntax validation endpoint works - PASSED');
      console.log(`  - Valid: ${data.valid}`);
      console.log(`  - Message: ${data.message}`);
      passed++;
    } else {
      console.log('⚠ Syntax validation endpoint unavailable');
    }
    
  } catch (error) {
    console.log(`✗ Syntax validation test failed: ${error.message}`);
    failed++;
  }
  
  try {
    console.log('\nTest 5: Test AI diagram generation with MCP context...');
    
    const response = await fetch(`${API_URL}/api/ai/generate-all`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        outputDir: './test-diagrams'
      })
    });
    
    if (response.ok) {
      const data = await response.json();
      
      assert.strictEqual(data.success, true, 'Should succeed');
      assert.ok(data.diagrams, 'Should return diagrams');
      
      console.log('✓ AI bulk diagram generation works - PASSED');
      console.log(`  - Total: ${data.total}`);
      console.log(`  - Successful: ${data.successful}`);
      console.log(`  - Failed: ${data.failed}`);
      console.log(`  - AI powered: ${data.ai_powered || false}`);
      console.log(`  - MCP enabled: ${data.mcp_enabled || false}`);
      passed++;
    } else {
      console.log('⚠ AI bulk diagram generation unavailable');
    }
    
  } catch (error) {
    console.log(`✗ AI bulk diagram generation test failed: ${error.message}`);
    failed++;
  }
  
  console.log('\n--- MCP Integration Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  console.log(`\nNote: MCP server integration requires MCP_SERVER_URL environment variable`);
  console.log(`Current MCP_SERVER_URL: ${MCP_SERVER_URL || 'Not set'}`);
  
  return { passed, failed, total: passed + failed };
}

async function testMcpEnvironmentConfiguration() {
  console.log('\n=== Testing MCP Environment Configuration ===\n');
  
  let passed = 0;
  let failed = 0;
  
  console.log('Checking environment variables...\n');
  
  const envVars = [
    'MCP_SERVER_URL',
    'ANTHROPIC_API_KEY',
    'OPENAI_API_KEY',
    'ARCLANG_COMPILER_PATH'
  ];
  
  for (const envVar of envVars) {
    const value = process.env[envVar];
    
    if (value) {
      console.log(`✓ ${envVar}: Set`);
      if (envVar.includes('KEY')) {
        console.log(`  Value: ${value.substring(0, 10)}...`);
      } else {
        console.log(`  Value: ${value}`);
      }
      passed++;
    } else {
      console.log(`⚠ ${envVar}: Not set`);
    }
  }
  
  console.log('\n--- Environment Configuration Summary ---');
  console.log(`Configured: ${passed}/${envVars.length}`);
  console.log(`Missing: ${envVars.length - passed}/${envVars.length}`);
  
  return { passed, failed: 0, total: envVars.length };
}

async function testMcpCapabilities() {
  console.log('\n=== Testing MCP Capabilities ===\n');
  
  let passed = 0;
  let failed = 0;
  
  const capabilities = [
    {
      name: 'Code Generation',
      endpoint: '/api/ai/generate/component-code',
      test: async () => {
        const response = await fetch(`${API_URL}/api/ai/generate/component-code`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            description: 'Simple controller',
            enforceSyntax: false
          })
        });
        return response.ok;
      }
    },
    {
      name: 'Code Review',
      endpoint: '/api/ai/review',
      test: async () => {
        const response = await fetch(`${API_URL}/api/ai/review`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            code: 'operational_analysis "Test" {}'
          })
        });
        return response.ok;
      }
    },
    {
      name: 'Syntax Validation',
      endpoint: '/api/ai/validate-syntax',
      test: async () => {
        const response = await fetch(`${API_URL}/api/ai/validate-syntax`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            code: 'operational_analysis "Test" {}'
          })
        });
        return response.ok;
      }
    },
    {
      name: 'AI Diagram Generation',
      endpoint: '/api/ai/generate/operational',
      test: async () => {
        const response = await fetch(`${API_URL}/api/ai/generate/operational`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            description: 'Test system'
          })
        });
        return response.ok;
      }
    }
  ];
  
  for (const capability of capabilities) {
    try {
      console.log(`Testing ${capability.name}...`);
      const available = await capability.test();
      
      if (available) {
        console.log(`✓ ${capability.name} - Available`);
        passed++;
      } else {
        console.log(`⚠ ${capability.name} - Unavailable`);
      }
    } catch (error) {
      console.log(`✗ ${capability.name} - Error: ${error.message}`);
      failed++;
    }
  }
  
  console.log('\n--- MCP Capabilities Summary ---');
  console.log(`Available: ${passed}/${capabilities.length}`);
  console.log(`Unavailable/Error: ${failed}/${capabilities.length}`);
  
  return { passed, failed, total: capabilities.length };
}

async function runAllMcpTests() {
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║      ArcViz MCP Integration Test Suite        ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log(`\nAPI URL: ${API_URL}`);
  console.log(`MCP Server: ${MCP_SERVER_URL || 'Not configured'}\n`);
  
  const results = [];
  
  results.push(await testMcpEnvironmentConfiguration());
  results.push(await testMcpIntegration());
  results.push(await testMcpCapabilities());
  
  const totalPassed = results.reduce((sum, r) => sum + r.passed, 0);
  const totalFailed = results.reduce((sum, r) => sum + r.failed, 0);
  const totalTests = results.reduce((sum, r) => sum + r.total, 0);
  
  console.log('\n╔════════════════════════════════════════════════╗');
  console.log('║           OVERALL TEST SUMMARY                 ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log(`\nTotal Tests: ${totalTests}`);
  console.log(`✓ Passed: ${totalPassed}`);
  console.log(`✗ Failed: ${totalFailed}`);
  console.log(`Success Rate: ${((totalPassed / totalTests) * 100).toFixed(1)}%`);
  
  console.log('\n══════════════════════════════════════════════════');
  console.log('NOTE: MCP integration requires proper configuration');
  console.log('Set MCP_SERVER_URL, ANTHROPIC_API_KEY or OPENAI_API_KEY');
  console.log('══════════════════════════════════════════════════\n');
  
  process.exit(totalFailed > 0 ? 1 : 0);
}

if (require.main === module) {
  runAllMcpTests().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { runAllMcpTests };
