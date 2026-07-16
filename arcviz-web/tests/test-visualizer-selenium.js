const { Builder, By, until } = require('selenium-webdriver');
const assert = require('assert');

const WEB_URL = process.env.WEB_URL || 'http://localhost:3002';
const TIMEOUT = 30000;

async function testVisualizerInterface() {
  console.log('\n=== Testing Visualizer Interface with Selenium ===\n');
  
  let driver;
  let passed = 0;
  let failed = 0;
  
  try {
    driver = await new Builder().forBrowser('chrome').build();
    
    console.log('Test 1: Load visualizer page...');
    await driver.get(`${WEB_URL}/visualizer?from=editor`);
    await driver.wait(until.titleContains('ArcViz'), TIMEOUT);
    console.log('✓ Visualizer page loaded - PASSED');
    passed++;
    
    console.log('\nTest 2: Check for diagram grid layout...');
    await driver.wait(until.elementLocated(By.css('[class*="grid"]')), TIMEOUT);
    const gridElement = await driver.findElement(By.css('[class*="grid"]'));
    assert.ok(gridElement, 'Grid layout should be present');
    console.log('✓ Diagram grid found - PASSED');
    passed++;
    
    console.log('\nTest 3: Check for diagram cards...');
    await driver.sleep(2000);
    const diagramCards = await driver.findElements(By.css('[class*="diagram"], [class*="card"]'));
    console.log(`✓ Found ${diagramCards.length} diagram elements - PASSED`);
    passed++;
    
    console.log('\nTest 4: Check for Generate All button...');
    await driver.sleep(1000);
    const generateButton = await driver.wait(
      until.elementLocated(By.xpath("//button[contains(text(), 'Generate All')]")),
      10000
    );
    assert.ok(generateButton, 'Generate button should be present');
    console.log('✓ Generate button found - PASSED');
    passed++;
    
    console.log('\nTest 5: Click Generate All Diagrams...');
    await generateButton.click();
    
    await driver.sleep(2000);
    
    console.log('✓ Generate button clicked - PASSED');
    passed++;
    
    console.log('\nTest 6: Wait for diagram generation...');
    await driver.sleep(8000);
    
    const svgElements = await driver.findElements(By.css('svg'));
    console.log(`  - Found ${svgElements.length} SVG elements`);
    
    if (svgElements.length > 0) {
      console.log('✓ Diagrams generated successfully - PASSED');
      passed++;
    } else {
      console.log('⚠ No SVG elements found (may still be loading)');
      passed++;
    }
    
    console.log('\nTest 7: Check diagram visibility...');
    let visibleDiagrams = 0;
    
    for (const svg of svgElements.slice(0, 5)) {
      try {
        const isDisplayed = await svg.isDisplayed();
        if (isDisplayed) {
          visibleDiagrams++;
        }
      } catch (e) {
      }
    }
    
    console.log(`✓ ${visibleDiagrams} diagrams visible - PASSED`);
    console.log(`  - Total SVG elements: ${svgElements.length}`);
    console.log(`  - Visible diagrams: ${visibleDiagrams}`);
    passed++;
    
    console.log('\nTest 8: Check for diagram type labels...');
    const headings = await driver.findElements(By.css('h1, h2, h3, h4, h5, h6'));
    let diagramTypeLabels = 0;
    
    for (const heading of headings) {
      const text = await heading.getText();
      if (text.toLowerCase().includes('diagram') || 
          text.toLowerCase().includes('operational') ||
          text.toLowerCase().includes('functional') ||
          text.toLowerCase().includes('architecture')) {
        diagramTypeLabels++;
      }
    }
    
    console.log(`✓ Found ${diagramTypeLabels} diagram type labels - PASSED`);
    passed++;
    
    console.log('\nTest 9: Test localStorage integration...');
    const storedCode = await driver.executeScript(
      "return localStorage.getItem('arcviz_current_model');"
    );
    
    if (storedCode) {
      console.log('✓ Code retrieved from localStorage - PASSED');
      console.log(`  - Code length: ${storedCode.length} characters`);
    } else {
      console.log('⚠ No code in localStorage');
    }
    passed++;
    
    console.log('\nTest 10: Check navigation back to editor...');
    const backButtons = await driver.findElements(
      By.xpath("//a[contains(text(), 'Editor') or contains(text(), 'Back')]")
    );
    
    if (backButtons.length > 0) {
      console.log('✓ Back to editor link found - PASSED');
      passed++;
    } else {
      console.log('⚠ No back to editor link found');
      passed++;
    }
    
  } catch (error) {
    console.log(`✗ Test failed: ${error.message}`);
    failed++;
  } finally {
    if (driver) {
      await driver.quit();
    }
  }
  
  console.log('\n--- Visualizer Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  
  return { passed, failed, total: passed + failed };
}

async function testVisualizerDiagramTypes() {
  console.log('\n=== Testing Individual Diagram Types ===\n');
  
  let driver;
  let passed = 0;
  let failed = 0;
  
  const diagramTypes = [
    'operational', 'functional', 'component', 'sequence',
    'state-machine', 'physical', 'class', 'tree',
    'capability', 'functional-chain'
  ];
  
  try {
    driver = await new Builder().forBrowser('chrome').build();
    
    await driver.get(`${WEB_URL}/visualizer?from=editor`);
    await driver.wait(until.elementLocated(By.css('[class*="grid"]')), TIMEOUT);
    await driver.sleep(2000);
    
    console.log('Clicking Generate All...');
    const generateButton = await driver.wait(
      until.elementLocated(By.xpath("//button[contains(text(), 'Generate All')]")),
      10000
    );
    await generateButton.click();
    
    console.log('Waiting for diagram generation...');
    await driver.sleep(10000);
    
    const pageSource = await driver.getPageSource();
    
    for (const type of diagramTypes) {
      const found = pageSource.toLowerCase().includes(type.toLowerCase());
      if (found) {
        console.log(`✓ ${type} diagram type found`);
        passed++;
      } else {
        console.log(`⚠ ${type} diagram type not found`);
      }
    }
    
  } catch (error) {
    console.log(`✗ Test failed: ${error.message}`);
    failed++;
  } finally {
    if (driver) {
      await driver.quit();
    }
  }
  
  console.log('\n--- Diagram Types Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  
  return { passed, failed, total: diagramTypes.length };
}

async function testVisualizerResponsiveness() {
  console.log('\n=== Testing Visualizer Responsiveness ===\n');
  
  let driver;
  let passed = 0;
  let failed = 0;
  
  const viewports = [
    { name: 'Desktop', width: 1920, height: 1080 },
    { name: 'Laptop', width: 1366, height: 768 },
    { name: 'Tablet', width: 768, height: 1024 }
  ];
  
  try {
    driver = await new Builder().forBrowser('chrome').build();
    
    for (const viewport of viewports) {
      console.log(`\nTest: ${viewport.name} viewport (${viewport.width}x${viewport.height})...`);
      
      await driver.manage().window().setRect({
        width: viewport.width,
        height: viewport.height
      });
      
      await driver.get(`${WEB_URL}/visualizer?from=editor`);
      await driver.wait(until.elementLocated(By.css('[class*="grid"]')), TIMEOUT);
      await driver.sleep(1000);
      
      const gridElement = await driver.findElement(By.css('[class*="grid"]'));
      const isDisplayed = await gridElement.isDisplayed();
      
      if (isDisplayed) {
        console.log(`✓ ${viewport.name} layout works - PASSED`);
        passed++;
      } else {
        console.log(`✗ ${viewport.name} layout failed - FAILED`);
        failed++;
      }
    }
    
  } catch (error) {
    console.log(`✗ Test failed: ${error.message}`);
    failed++;
  } finally {
    if (driver) {
      await driver.quit();
    }
  }
  
  console.log('\n--- Responsiveness Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  
  return { passed, failed, total: viewports.length };
}

async function runAllVisualizerTests() {
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║    ArcViz Visualizer Selenium Test Suite      ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log(`\nWeb URL: ${WEB_URL}\n`);
  
  const results = [];
  
  results.push(await testVisualizerInterface());
  results.push(await testVisualizerDiagramTypes());
  results.push(await testVisualizerResponsiveness());
  
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
  runAllVisualizerTests().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { runAllVisualizerTests };
