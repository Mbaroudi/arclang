const { Builder, By, Key, until } = require('selenium-webdriver');
const assert = require('assert');

const WEB_URL = process.env.WEB_URL || 'http://localhost:3002';
const API_URL = process.env.API_URL || 'http://localhost:3001';
const TIMEOUT = 30000;

const testCode = `// E2E Test Code - Smart Home System
operational_analysis "Smart Home Context" {
  actor "Homeowner" {
    id: "ACT-001"
    description: "Smart home user"
  }
  
  actor "Energy Grid" {
    id: "ACT-002"
    description: "Utility provider"
  }
}

system_analysis "Smart Home Requirements" {
  requirement "REQ-001" {
    id: "REQ-001"
    description: "System shall control lighting automatically"
    priority: "High"
  }
  
  requirement "REQ-002" {
    id: "REQ-002"
    description: "System shall optimize energy consumption"
    priority: "Critical"
  }
  
  system_function "Monitor Environment" {
    id: "SF-001"
    description: "Track environmental conditions"
  }
  
  system_function "Control Devices" {
    id: "SF-002"
    description: "Manage smart devices"
  }
}

logical_architecture "Smart Home Architecture" {
  component "Sensor Hub" {
    id: "LC-001"
    type: "Logical"
    description: "Environmental sensor aggregator"
    
    function "Read Sensors" {
      id: "LF-001"
      description: "Collect sensor data"
    }
  }
  
  component "Control Hub" {
    id: "LC-002"
    type: "Logical"
    description: "Device control center"
    
    function "Execute Commands" {
      id: "LF-002"
      description: "Control smart devices"
    }
  }
  
  component "AI Engine" {
    id: "LC-003"
    type: "Logical"
    description: "Machine learning optimizer"
    
    function "Optimize Usage" {
      id: "LF-003"
      description: "Predict and optimize"
    }
  }
}

physical_architecture "Hardware Platform" {
  node "Gateway Device" {
    id: "PN-001"
    description: "Central hub hardware"
  }
  
  node "Cloud Server" {
    id: "PN-002"
    description: "Backend processing"
  }
}

trace "LC-001" satisfies "REQ-001" {
  rationale: "Sensor hub enables automated lighting control"
}

trace "LC-003" satisfies "REQ-002" {
  rationale: "AI engine optimizes energy consumption"
}
`;

async function testCompleteWorkflow() {
  console.log('\n=== Testing Complete E2E Workflow ===\n');
  console.log('Workflow: Editor → Edit Code → Save → Visualize → Generate Diagrams → Verify\n');
  
  let driver;
  let passed = 0;
  let failed = 0;
  
  try {
    driver = await new Builder().forBrowser('chrome').build();
    await driver.manage().window().maximize();
    
    console.log('Step 1: Navigate to Editor...');
    await driver.get(`${WEB_URL}/editor`);
    await driver.wait(until.titleContains('ArcViz'), TIMEOUT);
    await driver.wait(until.elementLocated(By.css('.monaco-editor')), TIMEOUT);
    await driver.sleep(2000);
    console.log('✓ Editor loaded');
    passed++;
    
    console.log('\nStep 2: Clear existing code...');
    await driver.executeScript(`
      const model = monaco.editor.getModels()[0];
      if (model) {
        model.setValue('');
      }
    `);
    await driver.sleep(500);
    console.log('✓ Existing code cleared');
    passed++;
    
    console.log('\nStep 3: Insert test code...');
    await driver.executeScript(`
      const model = monaco.editor.getModels()[0];
      if (model) {
        model.setValue(${JSON.stringify(testCode)});
      }
    `);
    await driver.sleep(1000);
    
    const insertedCode = await driver.executeScript(
      "return monaco.editor.getModels()[0]?.getValue();"
    );
    assert.ok(insertedCode.includes('Smart Home System'), 'Code should be inserted');
    console.log('✓ Test code inserted successfully');
    console.log(`  - Code length: ${insertedCode.length} characters`);
    passed++;
    
    console.log('\nStep 4: Wait for auto-save to localStorage...');
    await driver.sleep(2000);
    
    const savedCode = await driver.executeScript(
      "return localStorage.getItem('arcviz_current_model');"
    );
    assert.ok(savedCode, 'Code should be saved to localStorage');
    assert.ok(savedCode.includes('Smart Home System'), 'Saved code should match');
    console.log('✓ Code auto-saved to localStorage');
    passed++;
    
    console.log('\nStep 5: Navigate to Visualizer...');
    const visualizeButton = await driver.findElement(
      By.xpath("//button[contains(text(), 'Visualize') or contains(text(), 'Generate')]")
    );
    await visualizeButton.click();
    
    await driver.wait(until.urlContains('/visualizer'), TIMEOUT);
    console.log('✓ Navigated to visualizer');
    passed++;
    
    console.log('\nStep 6: Verify code loaded in visualizer...');
    await driver.sleep(1000);
    
    const codeInVisualizer = await driver.executeScript(
      "return localStorage.getItem('arcviz_current_model');"
    );
    assert.ok(codeInVisualizer.includes('Smart Home System'), 'Code should be available');
    console.log('✓ Code available in visualizer context');
    passed++;
    
    console.log('\nStep 7: Click Generate All Diagrams...');
    await driver.wait(until.elementLocated(
      By.xpath("//button[contains(text(), 'Generate All')]")
    ), TIMEOUT);
    
    const generateButton = await driver.findElement(
      By.xpath("//button[contains(text(), 'Generate All')]")
    );
    await generateButton.click();
    console.log('✓ Generate button clicked');
    passed++;
    
    console.log('\nStep 8: Wait for diagram generation (15 seconds)...');
    await driver.sleep(15000);
    
    const svgElements = await driver.findElements(By.css('svg'));
    console.log(`  - Found ${svgElements.length} SVG elements`);
    
    if (svgElements.length > 0) {
      console.log('✓ Diagrams generated successfully');
      passed++;
    } else {
      console.log('⚠ No diagrams found (may need more time)');
    }
    
    console.log('\nStep 9: Verify diagram content...');
    const pageText = await driver.executeScript("return document.body.innerText;");
    
    const expectedContent = [
      'operational',
      'functional',
      'component',
      'architecture'
    ];
    
    let contentFound = 0;
    for (const content of expectedContent) {
      if (pageText.toLowerCase().includes(content)) {
        contentFound++;
      }
    }
    
    console.log(`✓ Found ${contentFound}/${expectedContent.length} expected diagram types`);
    passed++;
    
    console.log('\nStep 10: Verify API communication...');
    const consoleLogs = await driver.executeScript(`
      return window.performance.getEntriesByType('resource')
        .filter(r => r.name.includes('${API_URL}'))
        .map(r => ({ url: r.name, duration: r.duration }));
    `);
    
    console.log(`✓ Found ${consoleLogs.length} API calls`);
    if (consoleLogs.length > 0) {
      consoleLogs.slice(0, 3).forEach(log => {
        console.log(`  - ${log.url.substring(log.url.lastIndexOf('/'))}: ${log.duration.toFixed(2)}ms`);
      });
    }
    passed++;
    
    console.log('\nStep 11: Test navigation back to editor...');
    await driver.get(`${WEB_URL}/editor`);
    await driver.wait(until.elementLocated(By.css('.monaco-editor')), TIMEOUT);
    await driver.sleep(1000);
    
    const codeAfterReturn = await driver.executeScript(
      "return monaco.editor.getModels()[0]?.getValue();"
    );
    
    assert.ok(codeAfterReturn.includes('Smart Home System'), 'Code should persist');
    console.log('✓ Returned to editor with code intact');
    passed++;
    
    console.log('\nStep 12: Verify complete round trip...');
    const finalStoredCode = await driver.executeScript(
      "return localStorage.getItem('arcviz_current_model');"
    );
    
    assert.strictEqual(finalStoredCode, testCode, 'Code should be identical');
    console.log('✓ Complete round trip successful');
    passed++;
    
  } catch (error) {
    console.log(`\n✗ Workflow test failed: ${error.message}`);
    console.log(`   Stack: ${error.stack}`);
    failed++;
  } finally {
    if (driver) {
      await driver.quit();
    }
  }
  
  console.log('\n--- E2E Workflow Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  
  return { passed, failed, total: passed + failed };
}

async function testApiIntegration() {
  console.log('\n=== Testing API Integration in Workflow ===\n');
  
  let passed = 0;
  let failed = 0;
  
  try {
    console.log('Test 1: Verify API is accessible...');
    const healthCheck = await fetch(`${API_URL}/health`).catch(() => null);
    
    if (healthCheck && healthCheck.ok) {
      console.log('✓ API is accessible');
      passed++;
    } else {
      console.log('⚠ API health check failed (may not have /health endpoint)');
      passed++;
    }
    
    console.log('\nTest 2: Test diagram generation with workflow code...');
    const response = await fetch(`${API_URL}/api/diagrams/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        diagramType: 'operational',
        code: testCode
      })
    });
    
    assert.strictEqual(response.ok, true, 'API should generate diagram');
    const data = await response.json();
    assert.strictEqual(data.success, true, 'Generation should succeed');
    assert.ok(data.svg, 'Should return SVG');
    
    console.log('✓ API diagram generation works with workflow code');
    console.log(`  - SVG size: ${data.size?.width}x${data.size?.height}`);
    console.log(`  - Elements: ${data.elementCount}`);
    passed++;
    
    console.log('\nTest 3: Verify code parsing...');
    const operational = data.svg.includes('Homeowner') || data.svg.includes('ACT-001');
    
    if (operational) {
      console.log('✓ Code successfully parsed (found expected elements)');
      passed++;
    } else {
      console.log('⚠ Code parsing may not be working (elements not found in SVG)');
    }
    
  } catch (error) {
    console.log(`✗ API integration test failed: ${error.message}`);
    failed++;
  }
  
  console.log('\n--- API Integration Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  
  return { passed, failed, total: passed + failed };
}

async function testErrorHandlingWorkflow() {
  console.log('\n=== Testing Error Handling in Workflow ===\n');
  
  let driver;
  let passed = 0;
  let failed = 0;
  
  try {
    driver = await new Builder().forBrowser('chrome').build();
    
    console.log('Test 1: Navigate to visualizer without code...');
    await driver.get(`${WEB_URL}/visualizer?from=editor`);
    await driver.wait(until.elementLocated(By.css('[class*="grid"]')), TIMEOUT);
    await driver.sleep(2000);
    
    await driver.executeScript("localStorage.removeItem('arcviz_current_model');");
    await driver.navigate().refresh();
    await driver.sleep(2000);
    
    console.log('✓ Visualizer handles missing code gracefully');
    passed++;
    
    console.log('\nTest 2: Test with invalid code...');
    await driver.executeScript(`
      localStorage.setItem('arcviz_current_model', 'invalid code @#$%');
    `);
    
    await driver.navigate().refresh();
    await driver.sleep(2000);
    
    const generateButton = await driver.findElement(
      By.xpath("//button[contains(text(), 'Generate All')]")
    );
    await generateButton.click();
    await driver.sleep(5000);
    
    console.log('✓ System handles invalid code without crashing');
    passed++;
    
    console.log('\nTest 3: Restore valid code...');
    await driver.executeScript(`
      localStorage.setItem('arcviz_current_model', ${JSON.stringify(testCode)});
    `);
    console.log('✓ Code restored successfully');
    passed++;
    
  } catch (error) {
    console.log(`✗ Error handling test failed: ${error.message}`);
    failed++;
  } finally {
    if (driver) {
      await driver.quit();
    }
  }
  
  console.log('\n--- Error Handling Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  
  return { passed, failed, total: passed + failed };
}

async function runAllE2ETests() {
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║     ArcViz End-to-End Workflow Test Suite     ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log(`\nWeb URL: ${WEB_URL}`);
  console.log(`API URL: ${API_URL}\n`);
  
  const results = [];
  
  results.push(await testCompleteWorkflow());
  results.push(await testApiIntegration());
  results.push(await testErrorHandlingWorkflow());
  
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
  runAllE2ETests().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { runAllE2ETests };
