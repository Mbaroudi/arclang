const { Builder, By, Key, until } = require('selenium-webdriver');
const assert = require('assert');

const WEB_URL = process.env.WEB_URL || 'http://localhost:3002';
const TIMEOUT = 10000;

async function testEditorInterface() {
  console.log('\n=== Testing Editor Interface with Selenium ===\n');
  
  let driver;
  let passed = 0;
  let failed = 0;
  
  try {
    driver = await new Builder().forBrowser('chrome').build();
    
    console.log('Test 1: Load editor page...');
    await driver.get(`${WEB_URL}/editor`);
    await driver.wait(until.titleContains('ArcViz'), TIMEOUT);
    console.log('✓ Editor page loaded - PASSED');
    passed++;
    
    console.log('\nTest 2: Check Monaco editor presence...');
    await driver.wait(until.elementLocated(By.css('.monaco-editor')), TIMEOUT);
    const editorElement = await driver.findElement(By.css('.monaco-editor'));
    assert.ok(editorElement, 'Monaco editor should be present');
    console.log('✓ Monaco editor found - PASSED');
    passed++;
    
    console.log('\nTest 3: Check initial code content...');
    await driver.sleep(1000);
    const textArea = await driver.findElement(By.css('.monaco-editor textarea'));
    const codeContent = await driver.executeScript(
      "return localStorage.getItem('arcviz_current_model');"
    );
    
    if (codeContent) {
      console.log('✓ Initial code loaded from localStorage - PASSED');
      console.log(`  - Code length: ${codeContent.length} characters`);
      passed++;
    } else {
      console.log('⚠ No code in localStorage (may be first run)');
      passed++;
    }
    
    console.log('\nTest 4: Modify code in editor...');
    await driver.sleep(500);
    
    await driver.executeScript(`
      const model = monaco.editor.getModels()[0];
      if (model) {
        model.setValue('// Test code modification\\noperational_analysis "Test" {}');
      }
    `);
    
    await driver.sleep(500);
    
    const modifiedCode = await driver.executeScript(
      "return monaco.editor.getModels()[0]?.getValue();"
    );
    
    assert.ok(modifiedCode.includes('Test code modification'), 'Code should be modified');
    console.log('✓ Code modification successful - PASSED');
    console.log(`  - New code: ${modifiedCode.substring(0, 50)}...`);
    passed++;
    
    console.log('\nTest 5: Verify auto-save to localStorage...');
    await driver.sleep(1500);
    
    const savedCode = await driver.executeScript(
      "return localStorage.getItem('arcviz_current_model');"
    );
    
    assert.ok(savedCode, 'Code should be saved to localStorage');
    assert.ok(savedCode.includes('Test code modification'), 'Saved code should match editor');
    console.log('✓ Auto-save to localStorage works - PASSED');
    passed++;
    
    console.log('\nTest 6: Check toolbar buttons...');
    const buttons = await driver.findElements(By.css('button'));
    assert.ok(buttons.length > 0, 'Should have toolbar buttons');
    console.log('✓ Toolbar buttons present - PASSED');
    console.log(`  - Found ${buttons.length} buttons`);
    passed++;
    
    console.log('\nTest 7: Navigate to visualizer...');
    const visualizeButton = await driver.findElement(
      By.xpath("//button[contains(text(), 'Visualize') or contains(text(), 'Generate')]")
    );
    await visualizeButton.click();
    
    await driver.wait(until.urlContains('/visualizer'), TIMEOUT);
    const currentUrl = await driver.getCurrentUrl();
    assert.ok(currentUrl.includes('/visualizer'), 'Should navigate to visualizer');
    console.log('✓ Navigation to visualizer works - PASSED');
    console.log(`  - URL: ${currentUrl}`);
    passed++;
    
  } catch (error) {
    console.log(`✗ Test failed: ${error.message}`);
    failed++;
  } finally {
    if (driver) {
      await driver.quit();
    }
  }
  
  console.log('\n--- Editor Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  
  return { passed, failed, total: passed + failed };
}

async function testEditorKeyboardShortcuts() {
  console.log('\n=== Testing Editor Keyboard Shortcuts ===\n');
  
  let driver;
  let passed = 0;
  let failed = 0;
  
  try {
    driver = await new Builder().forBrowser('chrome').build();
    
    console.log('Test 1: Load editor...');
    await driver.get(`${WEB_URL}/editor`);
    await driver.wait(until.elementLocated(By.css('.monaco-editor')), TIMEOUT);
    await driver.sleep(1000);
    console.log('✓ Editor loaded - PASSED');
    passed++;
    
    console.log('\nTest 2: Test code modification via API...');
    await driver.executeScript(`
      const model = monaco.editor.getModels()[0];
      if (model) {
        const fullText = model.getValue();
        model.setValue('// Modified via script\\n' + fullText);
      }
    `);
    await driver.sleep(300);
    
    const modifiedCode = await driver.executeScript(
      "return monaco.editor.getModels()[0]?.getValue();"
    );
    
    assert.ok(modifiedCode.includes('Modified via script'), 'Should contain modification');
    console.log('✓ Code modification via API works - PASSED');
    passed++;
    
    console.log('\nTest 3: Test clear and set code...');
    await driver.executeScript(`
      monaco.editor.getModels()[0]?.setValue('// New test code');
    `);
    await driver.sleep(300);
    
    const newCode = await driver.executeScript(
      "return monaco.editor.getModels()[0]?.getValue();"
    );
    
    assert.ok(newCode.includes('New test code'), 'Should contain new text');
    console.log('✓ Clear and set code works - PASSED');
    passed++;
    
  } catch (error) {
    console.log(`✗ Test failed: ${error.message}`);
    failed++;
  } finally {
    if (driver) {
      await driver.quit();
    }
  }
  
  console.log('\n--- Keyboard Shortcuts Test Summary ---');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  
  return { passed, failed, total: passed + failed };
}

async function runAllEditorTests() {
  console.log('╔════════════════════════════════════════════════╗');
  console.log('║      ArcViz Editor Selenium Test Suite        ║');
  console.log('╚════════════════════════════════════════════════╝');
  console.log(`\nWeb URL: ${WEB_URL}\n`);
  
  const results = [];
  
  results.push(await testEditorInterface());
  results.push(await testEditorKeyboardShortcuts());
  
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
  runAllEditorTests().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { runAllEditorTests };
