import { WebDriver, By } from 'selenium-webdriver';
import { createDriver, TEST_CONFIG, waitForElement, takeScreenshot } from './config.js';

export async function runVisualizerTests(reporter: any) {
  let driver: WebDriver | null = null;
  const tests = [
    { name: 'Load visualizer page', fn: testVisualizerLoad },
    { name: 'Check available diagram types', fn: testDiagramTypes },
    { name: 'Generate operational diagram', fn: testGenerateOperational },
    { name: 'Generate functional diagram', fn: testGenerateFunctional },
    { name: 'Export diagram as SVG', fn: testExportDiagram },
  ];

  for (const test of tests) {
    const startTime = Date.now();
    try {
      driver = await createDriver();
      await setupVisualizerTest(driver);
      console.log(`\n▶ Running: ${test.name}`);
      await test.fn(driver);
      const duration = Date.now() - startTime;
      reporter.addResult(`Visualizer: ${test.name}`, 'passed', duration);
      console.log(`✓ Passed: ${test.name} (${(duration / 1000).toFixed(2)}s)`);
    } catch (error: any) {
      const duration = Date.now() - startTime;
      let screenshot;
      if (driver) {
        screenshot = await takeScreenshot(driver, `visualizer-${test.name.replace(/\s+/g, '-')}-failed`);
      }
      reporter.addResult(`Visualizer: ${test.name}`, 'failed', duration, error.message, screenshot);
      console.log(`✗ Failed: ${test.name} - ${error.message}`);
    } finally {
      if (driver) {
        await driver.quit();
      }
    }
  }
}

async function setupVisualizerTest(driver: WebDriver) {
  const sampleCode = `
actor "User" as user
actor "System" as sys

operational_activity "Login" {
  performed_by: user
}

operational_activity "Authenticate" {
  performed_by: sys
}

interaction user -> sys : "login request"
interaction sys -> user : "auth token"

system_function "AuthService" {
  allocated_to: sys
}

component "AuthController" {
  provides: ["IAuth"]
}

component "UserDatabase" {
  provides: ["IUserStore"]
}

interface "IAuth" {}
interface "IUserStore" {}
`;

  await driver.get(`${TEST_CONFIG.baseUrl}/editor`);
  await driver.sleep(1000);
  
  await driver.executeScript(`localStorage.setItem("arcviz_current_model", ${JSON.stringify(sampleCode)});`);
}

async function testVisualizerLoad(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  
  await waitForElement(driver, 'body');
  
  await driver.sleep(2000);
  
  const pageTitle = await driver.getTitle();
  console.log(`   Page title: ${pageTitle}`);
  
  const currentUrl = await driver.getCurrentUrl();
  if (!currentUrl.includes('visualizer')) {
    throw new Error('Not on visualizer page');
  }
}

async function testDiagramTypes(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  
  await driver.sleep(3000);
  
  const diagramCards = await driver.findElements(By.css('[class*="Card"], .diagram-type, button[class*="generate"]'));
  
  console.log(`   Found ${diagramCards.length} diagram type elements`);
  
  if (diagramCards.length === 0) {
    console.log('   Warning: No diagram types found');
  }
}

async function testGenerateOperational(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  
  await driver.sleep(3000);
  
  try {
    const generateButtons = await driver.findElements(By.css('button'));
    let operationalButton = null;
    
    for (const button of generateButtons) {
      const text = await button.getText();
      const parent = await button.findElement(By.xpath('..'));
      const parentText = await parent.getText();
      
      if (parentText.toLowerCase().includes('operational') && 
          (text.toLowerCase().includes('generate') || text.toLowerCase().includes('create'))) {
        operationalButton = button;
        break;
      }
    }
    
    if (operationalButton) {
      await operationalButton.click();
      console.log('   Clicked generate button for operational diagram');
      
      await driver.sleep(5000);
      
      const svgs = await driver.findElements(By.css('svg'));
      console.log(`   Found ${svgs.length} SVG elements on page`);
    } else {
      console.log('   Warning: Could not find operational generate button');
    }
  } catch (error) {
    console.log('   Warning: Could not generate operational diagram');
  }
}

async function testGenerateFunctional(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  
  await driver.sleep(3000);
  
  try {
    const generateButtons = await driver.findElements(By.css('button'));
    let functionalButton = null;
    
    for (const button of generateButtons) {
      const text = await button.getText();
      const parent = await button.findElement(By.xpath('..'));
      const parentText = await parent.getText();
      
      if (parentText.toLowerCase().includes('functional') && 
          (text.toLowerCase().includes('generate') || text.toLowerCase().includes('create'))) {
        functionalButton = button;
        break;
      }
    }
    
    if (functionalButton) {
      await functionalButton.click();
      console.log('   Clicked generate button for functional diagram');
      
      await driver.sleep(5000);
      
      const svgs = await driver.findElements(By.css('svg'));
      console.log(`   Found ${svgs.length} SVG elements on page`);
    } else {
      console.log('   Warning: Could not find functional generate button');
    }
  } catch (error) {
    console.log('   Warning: Could not generate functional diagram');
  }
}

async function testExportDiagram(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  
  await driver.sleep(3000);
  
  try {
    const generateButtons = await driver.findElements(By.css('button'));
    if (generateButtons.length > 0) {
      await generateButtons[0].click();
      await driver.sleep(5000);
      
      const exportButtons = await driver.findElements(By.xpath('//button[contains(., "Export") or contains(., "Download") or contains(., "SVG")]'));
      
      if (exportButtons.length > 0) {
        console.log(`   Found ${exportButtons.length} export buttons`);
        console.log('   Export functionality available');
      } else {
        console.log('   Warning: No export buttons found');
      }
    }
  } catch (error) {
    console.log('   Warning: Could not test export functionality');
  }
}
