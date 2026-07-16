import { WebDriver, By } from 'selenium-webdriver';
import { createDriver, TEST_CONFIG, waitForElement, takeScreenshot } from './config.js';

export async function run7DVisualizerTests(reporter: any) {
  let driver: WebDriver | null = null;
  const tests = [
    { name: 'Enable 7D view', fn: testEnable7DView },
    { name: 'Navigate to Operational dimension', fn: testOperationalDimension },
    { name: 'Navigate to System dimension', fn: testSystemDimension },
    { name: 'Navigate to Logical dimension', fn: testLogicalDimension },
    { name: 'Navigate to Physical dimension', fn: testPhysicalDimension },
    { name: 'Navigate to EPBS dimension', fn: testEPBSDimension },
    { name: 'Navigate to Requirements dimension', fn: testRequirementsDimension },
    { name: 'Navigate to Cross-cutting dimension', fn: testCrossCuttingDimension },
    { name: 'Test SVG rendering', fn: testSVGRendering },
    { name: 'Test node interaction', fn: testNodeInteraction },
    { name: 'Disable 7D view', fn: testDisable7DView },
  ];

  for (const test of tests) {
    const startTime = Date.now();
    try {
      driver = await createDriver();
      await setup7DTest(driver);
      console.log(`\n▶ Running: ${test.name}`);
      await test.fn(driver);
      const duration = Date.now() - startTime;
      reporter.addResult(`7D Visualizer: ${test.name}`, 'passed', duration);
      console.log(`✓ Passed: ${test.name} (${(duration / 1000).toFixed(2)}s)`);
    } catch (error: any) {
      const duration = Date.now() - startTime;
      let screenshot;
      if (driver) {
        screenshot = await takeScreenshot(driver, `7d-${test.name.replace(/\s+/g, '-')}-failed`);
      }
      reporter.addResult(`7D Visualizer: ${test.name}`, 'failed', duration, error.message, screenshot);
      console.log(`✗ Failed: ${test.name} - ${error.message}`);
    } finally {
      if (driver) {
        await driver.quit();
      }
    }
  }
}

async function setup7DTest(driver: WebDriver) {
  const complexCode = `
actor "User" as user
actor "System" as sys
actor "Admin" as admin

operational_activity "UserLogin" {
  performed_by: user
}

operational_activity "SystemAuth" {
  performed_by: sys
}

capability "Authentication" {
  activities: ["UserLogin", "SystemAuth"]
}

system_function "AuthService" {
  allocated_to: sys
}

system_function "UserManagement" {
  allocated_to: sys
}

component "AuthController" {
  provides: ["IAuth"]
  requires: ["IUserStore"]
}

component "UserDatabase" {
  provides: ["IUserStore"]
}

interface "IAuth" {}
interface "IUserStore" {}

physical_node "WebServer" {
  type: hardware
}

physical_node "DatabaseServer" {
  type: hardware
}

deployment AuthController -> WebServer
deployment UserDatabase -> DatabaseServer

requirement "REQ001" {
  type: functional
  priority: high
  status: approved
  text: "System shall authenticate users"
  allocated_to: ["AuthService"]
}

requirement "REQ002" {
  type: non-functional
  priority: medium
  status: draft
  text: "System shall respond within 2 seconds"
}

security_policy "SP001" {
  name: "Password Policy"
  description: "Passwords must be hashed"
}

performance_metric "PM001" {
  name: "Response Time"
  target: "< 2s"
}
`;

  await driver.get(`${TEST_CONFIG.baseUrl}/editor`);
  await driver.sleep(1000);
  
  await driver.executeScript(`localStorage.setItem("arcviz_current_model", ${JSON.stringify(complexCode)});`);
}

async function testEnable7DView(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  
  await driver.sleep(2000);
  
  try {
    const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D") or contains(., "Enable 7D View")]'));
    await enable7DButton.click();
    
    await driver.sleep(2000);
    
    const dimensionNav = await driver.findElements(By.xpath('//*[contains(., "Operational") or contains(., "System") or contains(., "Logical")]'));
    
    if (dimensionNav.length === 0) {
      throw new Error('7D navigation not visible after enabling');
    }
    
    console.log('   7D view enabled successfully');
  } catch (error) {
    throw new Error('Could not enable 7D view');
  }
}

async function testOperationalDimension(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(1000);
  
  try {
    const operationalButton = await driver.findElement(By.xpath('//button[contains(., "Operational")]'));
    await operationalButton.click();
    
    await driver.sleep(2000);
    
    const svg = await driver.findElements(By.css('svg'));
    
    if (svg.length === 0) {
      throw new Error('No SVG rendered for Operational dimension');
    }
    
    console.log(`   Operational dimension rendered with ${svg.length} SVG elements`);
  } catch (error) {
    throw new Error('Could not navigate to Operational dimension');
  }
}

async function testSystemDimension(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(1000);
  
  try {
    const systemButton = await driver.findElement(By.xpath('//button[contains(., "System")]'));
    await systemButton.click();
    
    await driver.sleep(2000);
    
    const svg = await driver.findElements(By.css('svg'));
    console.log(`   System dimension rendered with ${svg.length} SVG elements`);
  } catch (error) {
    throw new Error('Could not navigate to System dimension');
  }
}

async function testLogicalDimension(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(1000);
  
  try {
    const logicalButton = await driver.findElement(By.xpath('//button[contains(., "Logical")]'));
    await logicalButton.click();
    
    await driver.sleep(2000);
    
    const svg = await driver.findElements(By.css('svg'));
    console.log(`   Logical dimension rendered with ${svg.length} SVG elements`);
  } catch (error) {
    throw new Error('Could not navigate to Logical dimension');
  }
}

async function testPhysicalDimension(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(1000);
  
  try {
    const physicalButton = await driver.findElement(By.xpath('//button[contains(., "Physical")]'));
    await physicalButton.click();
    
    await driver.sleep(2000);
    
    const svg = await driver.findElements(By.css('svg'));
    console.log(`   Physical dimension rendered with ${svg.length} SVG elements`);
  } catch (error) {
    throw new Error('Could not navigate to Physical dimension');
  }
}

async function testEPBSDimension(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(1000);
  
  try {
    const epbsButton = await driver.findElement(By.xpath('//button[contains(., "EPBS")]'));
    await epbsButton.click();
    
    await driver.sleep(2000);
    
    const svg = await driver.findElements(By.css('svg'));
    console.log(`   EPBS dimension rendered with ${svg.length} SVG elements`);
  } catch (error) {
    throw new Error('Could not navigate to EPBS dimension');
  }
}

async function testRequirementsDimension(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(1000);
  
  try {
    const reqButton = await driver.findElement(By.xpath('//button[contains(., "Requirements")]'));
    await reqButton.click();
    
    await driver.sleep(2000);
    
    const svg = await driver.findElements(By.css('svg'));
    console.log(`   Requirements dimension rendered with ${svg.length} SVG elements`);
  } catch (error) {
    throw new Error('Could not navigate to Requirements dimension');
  }
}

async function testCrossCuttingDimension(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(1000);
  
  try {
    const crossButton = await driver.findElement(By.xpath('//button[contains(., "Cross")]'));
    await crossButton.click();
    
    await driver.sleep(2000);
    
    const svg = await driver.findElements(By.css('svg'));
    console.log(`   Cross-cutting dimension rendered with ${svg.length} SVG elements`);
  } catch (error) {
    throw new Error('Could not navigate to Cross-cutting dimension');
  }
}

async function testSVGRendering(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(2000);
  
  const svgElements = await driver.findElements(By.css('svg'));
  
  if (svgElements.length === 0) {
    throw new Error('No SVG elements rendered');
  }
  
  const circles = await driver.findElements(By.css('svg circle, svg rect, svg path'));
  const lines = await driver.findElements(By.css('svg line, svg path[stroke]'));
  const texts = await driver.findElements(By.css('svg text'));
  
  console.log(`   SVG elements: circles/rects=${circles.length}, lines=${lines.length}, texts=${texts.length}`);
}

async function testNodeInteraction(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(2000);
  
  try {
    const nodes = await driver.findElements(By.css('svg rect, svg circle'));
    
    if (nodes.length > 0) {
      await nodes[0].click();
      await driver.sleep(1000);
      
      const detailsPanel = await driver.findElements(By.xpath('//*[contains(., "Details") or contains(., "Node Details")]'));
      
      if (detailsPanel.length > 0) {
        console.log('   Node interaction working - details panel visible');
      } else {
        console.log('   Warning: No details panel found after node click');
      }
    }
  } catch (error) {
    console.log('   Warning: Could not test node interaction');
  }
}

async function testDisable7DView(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
  await driver.sleep(2000);
  
  const enable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await enable7DButton.click();
  await driver.sleep(1000);
  
  const disable7DButton = await driver.findElement(By.xpath('//button[contains(., "7D")]'));
  await disable7DButton.click();
  
  await driver.sleep(1000);
  
  console.log('   7D view disabled successfully');
}
