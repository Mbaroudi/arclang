const { Builder, By, until } = require('selenium-webdriver');
const chrome = require('selenium-webdriver/chrome');

const BASE_URL = 'http://localhost:3002';

const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  cyan: '\x1b[36m',
  yellow: '\x1b[33m',
};

async function testCompilation() {
  console.log(`${colors.cyan}🚀 Testing ArcLang Compilation${colors.reset}\n`);

  const options = new chrome.Options();
  options.addArguments('--headless');
  options.addArguments('--no-sandbox');
  options.addArguments('--disable-dev-shm-usage');
  options.addArguments('--window-size=1920,1080');

  const driver = await new Builder()
    .forBrowser('chrome')
    .setChromeOptions(options)
    .build();

  try {
    await driver.manage().setTimeouts({ implicit: 10000 });

    // First register/login a user
    console.log(`${colors.cyan}➤ Registering test user...${colors.reset}`);
    const testEmail = `test${Date.now()}@arcviz.io`;
    const testPassword = 'TestPassword123!';
    
    await driver.get(`${BASE_URL}/register`);
    await driver.sleep(1000);
    
    const nameInput = await driver.findElement(By.css('input#name'));
    const emailInput = await driver.findElement(By.css('input#email'));
    const passwordInputs = await driver.findElements(By.css('input[type="password"]'));
    
    await nameInput.sendKeys('Test User');
    await emailInput.sendKeys(testEmail);
    await passwordInputs[0].sendKeys(testPassword);
    await passwordInputs[1].sendKeys(testPassword);
    
    const submitButton = await driver.findElement(By.css('button[type="submit"]'));
    await submitButton.click();
    await driver.sleep(2000);

    // Navigate to visualizer
    console.log(`${colors.cyan}➤ Loading visualizer page...${colors.reset}`);
    await driver.get(`${BASE_URL}/visualizer`);
    await driver.sleep(2000);

    // Click "Compile Code" button
    console.log(`${colors.cyan}➤ Opening code input panel...${colors.reset}`);
    const compileButton = await driver.findElement(
      By.xpath("//button[contains(text(), 'Compile Code')]")
    );
    await compileButton.click();
    await driver.sleep(500);

    // Click "Load Sample" button
    console.log(`${colors.cyan}➤ Loading sample code...${colors.reset}`);
    const loadSampleButton = await driver.findElement(
      By.xpath("//button[contains(text(), 'Load Sample')]")
    );
    await loadSampleButton.click();
    await driver.sleep(500);

    // Get the code from textarea
    const textarea = await driver.findElement(By.css('textarea'));
    const sampleCode = await textarea.getAttribute('value');
    console.log(`${colors.cyan}➤ Sample code loaded (${sampleCode.length} characters)${colors.reset}`);
    
    // Print first few lines of sample code
    const firstLines = sampleCode.split('\n').slice(0, 5).join('\n');
    console.log(`${colors.yellow}Sample code preview:\n${firstLines}${colors.reset}\n`);

    // Click "Compile & Visualize" button
    console.log(`${colors.cyan}➤ Compiling code...${colors.reset}`);
    const compileVisualizeButton = await driver.findElement(
      By.xpath("//button[contains(text(), 'Compile & Visualize')]")
    );
    await compileVisualizeButton.click();
    
    // Wait for compilation to complete (up to 10 seconds)
    await driver.sleep(5000);

    // Check browser console for errors and logs
    const logs = await driver.manage().logs().get('browser');
    const errors = logs.filter(log => log.level.name === 'SEVERE');
    const allLogs = logs.filter(log => log.message.includes('compile') || log.message.includes('diagram') || log.message.includes('nodes'));
    
    if (errors.length > 0) {
      console.log(`${colors.yellow}Browser console errors:${colors.reset}`);
      errors.forEach(log => console.log(`  ${log.message}`));
    }
    
    if (allLogs.length > 0) {
      console.log(`${colors.cyan}Browser console logs (compile/diagram):${colors.reset}`);
      allLogs.slice(0, 5).forEach(log => console.log(`  ${log.message.substring(0, 200)}`));
    }

    // Get the stats display to see if compilation succeeded
    const statsText = await driver.findElements(By.xpath("//*[contains(text(), 'Components')]"));
    if (statsText.length > 0) {
      const text = await statsText[0].getText();
      console.log(`${colors.cyan}Stats display: ${text}${colors.reset}`);
    }

    // Check for error toast
    const toasts = await driver.findElements(By.css('[role="alert"]'));
    if (toasts.length > 0) {
      const toastText = await toasts[0].getText();
      console.log(`${colors.yellow}Toast message: ${toastText}${colors.reset}`);
      
      if (toastText.toLowerCase().includes('error') || toastText.toLowerCase().includes('failed')) {
        console.log(`${colors.red}✗ COMPILATION FAILED${colors.reset}`);
        console.log(`${colors.red}Error: ${toastText}${colors.reset}\n`);
        return false;
      }
    }

    // Check if diagram was rendered (look for large SVG - the diagram viewer)
    const svgs = await driver.findElements(By.css('svg'));
    console.log(`${colors.cyan}➤ Found ${svgs.length} SVG elements${colors.reset}`);
    
    if (svgs.length === 0) {
      console.log(`${colors.red}✗ NO SVG ELEMENTS FOUND${colors.reset}\n`);
      return false;
    }

    // Find the diagram SVG (should be large, e.g., 1600x1200)
    let diagramSvg = null;
    for (const svg of svgs) {
      const size = await svg.getRect();
      if (size.width > 500) {  // Diagram SVG should be much larger than icons
        diagramSvg = svg;
        console.log(`${colors.cyan}➤ Found diagram SVG: ${size.width}x${size.height}${colors.reset}`);
        break;
      }
    }
    
    if (!diagramSvg) {
      console.log(`${colors.red}✗ NO LARGE DIAGRAM SVG FOUND (all SVGs are small icons)${colors.reset}\n`);
      
      // Take a screenshot to debug
      const screenshot = await driver.takeScreenshot();
      require('fs').writeFileSync('/tmp/visualizer-debug.png', screenshot, 'base64');
      console.log(`${colors.yellow}📸 Screenshot saved to /tmp/visualizer-debug.png${colors.reset}\n`);
      
      return false;
    }

    // Check if SVG has nodes (look for rectangles)
    const rects = await driver.findElements(By.css('svg rect'));
    console.log(`${colors.cyan}➤ Found ${rects.length} rect elements in SVG${colors.reset}`);

    // Check for SVG groups which contain nodes
    const groups = await driver.findElements(By.css('svg g.node'));
    console.log(`${colors.cyan}➤ Found ${groups.length} node groups in SVG${colors.reset}`);

    // Check for any SVG children
    const allSvgChildren = await driver.findElements(By.css('svg *'));
    console.log(`${colors.cyan}➤ Total SVG child elements: ${allSvgChildren.length}${colors.reset}`);

    if (rects.length === 0 && groups.length === 0) {
      console.log(`${colors.yellow}⚠ WARNING: No diagram nodes rendered${colors.reset}\n`);
      return false;
    }

    console.log(`${colors.green}✓ COMPILATION SUCCESSFUL${colors.reset}`);
    console.log(`${colors.green}✓ Diagram rendered with ${rects.length} nodes${colors.reset}\n`);
    return true;

  } catch (error) {
    console.error(`${colors.red}✗ TEST ERROR: ${error.message}${colors.reset}\n`);
    return false;
  } finally {
    await driver.quit();
  }
}

// Run the test
(async () => {
  const success = await testCompilation();
  process.exit(success ? 0 : 1);
})();
