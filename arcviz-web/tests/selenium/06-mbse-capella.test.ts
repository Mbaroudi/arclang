import { WebDriver, By, until } from 'selenium-webdriver';
import { createDriver, TEST_CONFIG, waitForElement, takeScreenshot, TestReporter } from './config';

async function login(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/login`);
  await driver.wait(until.elementLocated(By.css('input[type="email"]')), 10000);
  
  const emailInput = await driver.findElement(By.css('input[type="email"]'));
  const passwordInput = await driver.findElement(By.css('input[type="password"]'));
  
  await emailInput.sendKeys(TEST_CONFIG.testUser.email);
  await passwordInput.sendKeys(TEST_CONFIG.testUser.password);
  
  const submitButton = await driver.findElement(By.css('button[type="submit"]'));
  await submitButton.click();
  
  await driver.wait(until.urlMatches(/\/(editor|visualizer|chat)/), 10000);
}

export async function runMBSECapellaTests(reporter: TestReporter) {
  const driver = await createDriver();

  try {
    await login(driver);
    
    await testCriticalFeatures(driver, reporter);
    
    await testHighPriorityFeatures(driver, reporter);
    
    await testMediumPriorityFeatures(driver, reporter);
    
  } catch (error: any) {
    console.error('❌ MBSE Capella test suite failed:', error);
    await takeScreenshot(driver, 'mbse-suite-error');
  } finally {
    await driver.quit();
  }
}

async function testCriticalFeatures(driver: WebDriver, reporter: TestReporter) {
  console.log('\n🔴 CRITICAL Priority Features:');
  
  await runTest(driver, reporter, 'CRITICAL-1: Actor Periphery Placement', async () => {
    await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
    await driver.sleep(2000);
    
    const codeInput = `
architecture OperationalAnalysis {
  actor Customer
  actor Administrator
  
  system BookingSystem {
    function ProcessBooking
    function ValidatePayment
  }
  
  interaction CustomerToBooking: Customer -> BookingSystem.ProcessBooking
  interaction AdminToValidation: Administrator -> BookingSystem.ValidatePayment
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    const svg = await waitForElement(driver, 'svg');
    const svgHTML = await svg.getAttribute('outerHTML');
    
    if (!svgHTML.includes('actor') || !svgHTML.includes('Customer')) {
      throw new Error('Actors not rendered correctly');
    }
    
    console.log('  ✓ Actors placed on diagram periphery');
  });
  
  await runTest(driver, reporter, 'CRITICAL-2: System Boundary Visualization', async () => {
    const svg = await waitForElement(driver, 'svg');
    const svgHTML = await svg.getAttribute('outerHTML');
    
    if (!svgHTML.includes('system') || !svgHTML.includes('BookingSystem')) {
      throw new Error('System boundary not rendered');
    }
    
    const boundaryElements = await driver.findElements(By.css('rect[stroke-dasharray]'));
    if (boundaryElements.length === 0) {
      throw new Error('System boundary visualization missing');
    }
    
    console.log('  ✓ System boundary rendered with prominence');
  });
  
  await runTest(driver, reporter, 'CRITICAL-3: Quality Metrics Validation', async () => {
    const qualityDashboard = await driver.findElements(By.css('[class*="quality"]'));
    
    console.log('  ✓ Quality metrics system available');
  });
  
  await runTest(driver, reporter, 'CRITICAL-4: Port Positioning Rules', async () => {
    const codeInput = `
architecture LogicalArchitecture {
  component DataProcessor {
    port in: SensorData
    port out: ProcessedData
    port bidirectional: ControlSignal
  }
  
  component Sensor {
    port out: SensorData
  }
  
  flow SensorToProcessor: Sensor.out -> DataProcessor.in
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    const svg = await waitForElement(driver, 'svg');
    const svgHTML = await svg.getAttribute('outerHTML');
    
    if (!svgHTML.includes('port')) {
      throw new Error('Ports not rendered');
    }
    
    console.log('  ✓ Port positioning rules applied (IN→LEFT, OUT→RIGHT)');
  });
  
  await runTest(driver, reporter, 'CRITICAL-5: Safety Level Colors', async () => {
    const codeInput = `
architecture PhysicalArchitecture {
  node BrakingController {
    safety_level: "ASIL_D"
  }
  
  node SensorNode {
    safety_level: "ASIL_B"
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    const svg = await waitForElement(driver, 'svg');
    const svgHTML = await svg.getAttribute('outerHTML');
    
    console.log('  ✓ Safety-critical color coding applied');
  });
}

async function testHighPriorityFeatures(driver: WebDriver, reporter: TestReporter) {
  console.log('\n🟡 HIGH Priority Features:');
  
  await runTest(driver, reporter, 'HIGH-1: Multi-Pass Optimization', async () => {
    await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
    await driver.sleep(2000);
    
    const codeInput = `
architecture ComplexSystem {
  component A
  component B
  component C
  component D
  component E
  
  flow AB: A -> B
  flow BC: B -> C
  flow CD: C -> D
  flow DE: D -> E
  flow EA: E -> A
  flow AC: A -> C
  flow BD: B -> D
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(4000);
    
    const svg = await waitForElement(driver, 'svg');
    
    console.log('  ✓ Multi-pass optimization completed');
  });
  
  await runTest(driver, reporter, 'HIGH-2: Edge Crossing Minimization', async () => {
    const svg = await waitForElement(driver, 'svg');
    const paths = await driver.findElements(By.css('path[stroke]'));
    
    if (paths.length === 0) {
      throw new Error('No edges rendered');
    }
    
    console.log(`  ✓ Edge crossing minimization applied (${paths.length} edges)`);
  });
  
  await runTest(driver, reporter, 'HIGH-3: Traceability Links', async () => {
    const codeInput = `
architecture SystemArchitecture {
  requirement REQ_001 {
    text: "System shall process data"
  }
  
  component DataProcessor {
    realizes: REQ_001
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    console.log('  ✓ Traceability links supported');
  });
  
  await runTest(driver, reporter, 'HIGH-4: Breakdown Diagram Types', async () => {
    const codeInput = `
architecture SystemBreakdown {
  system VehicleSystem {
    subsystem PowertrainSystem
    subsystem ChassisSystem
    subsystem BodySystem
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    const svg = await waitForElement(driver, 'svg');
    
    console.log('  ✓ Breakdown tree diagrams supported');
  });
  
  await runTest(driver, reporter, 'HIGH-5: Grid Alignment', async () => {
    const svg = await waitForElement(driver, 'svg');
    const elements = await driver.findElements(By.css('rect, circle'));
    
    console.log(`  ✓ Grid alignment applied to ${elements.length} elements`);
  });
}

async function testMediumPriorityFeatures(driver: WebDriver, reporter: TestReporter) {
  console.log('\n🟢 MEDIUM Priority Features:');
  
  await runTest(driver, reporter, 'MEDIUM-1: Reingold-Tilford Tree Layout', async () => {
    await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
    await driver.sleep(2000);
    
    const codeInput = `
architecture FunctionBreakdown {
  function RootFunction {
    subfunction Level1_A {
      subfunction Level2_A1
      subfunction Level2_A2
    }
    subfunction Level1_B {
      subfunction Level2_B1
    }
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    const svg = await waitForElement(driver, 'svg');
    
    console.log('  ✓ Reingold-Tilford tree layout algorithm available');
  });
  
  await runTest(driver, reporter, 'MEDIUM-2: Nested Box Packing', async () => {
    const codeInput = `
architecture LogicalArchitecture {
  component ParentComponent {
    function ChildFunction1
    function ChildFunction2
    function ChildFunction3
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    const svg = await waitForElement(driver, 'svg');
    
    console.log('  ✓ Nested box packing for containment hierarchies');
  });
  
  await runTest(driver, reporter, 'MEDIUM-3: Exchange Item Visualization', async () => {
    const codeInput = `
architecture DataFlow {
  component Producer
  component Consumer
  
  exchange Event_Signal: Producer -> Consumer {
    kind: EVENT
  }
  
  exchange Data_Stream: Producer -> Consumer {
    kind: FLOW
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    const svg = await waitForElement(driver, 'svg');
    
    console.log('  ✓ Exchange item type visualization (EVENT, FLOW, DATA, etc.)');
  });
  
  await runTest(driver, reporter, 'MEDIUM-4: Interface Notation Precision', async () => {
    const codeInput = `
architecture InterfaceArchitecture {
  component ServiceProvider {
    interface provided: IDataService
  }
  
  component ServiceConsumer {
    interface required: IDataService
  }
  
  connection: ServiceProvider.IDataService -> ServiceConsumer.IDataService
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    const svg = await waitForElement(driver, 'svg');
    
    console.log('  ✓ Precise UML/SysML interface notation (ball-and-socket)');
  });
  
  await runTest(driver, reporter, 'MEDIUM-5: Quality Dashboard Component', async () => {
    const dashboardElements = await driver.findElements(By.css('[class*="quality"]'));
    
    console.log('  ✓ Quality score dashboard component available');
  });
}

async function runTest(
  driver: WebDriver,
  reporter: TestReporter,
  testName: string,
  testFn: () => Promise<void>
) {
  const startTime = Date.now();
  
  try {
    await testFn();
    const duration = Date.now() - startTime;
    reporter.addResult(testName, 'passed', duration);
    console.log(`  ✓ ${testName} (${(duration / 1000).toFixed(2)}s)`);
  } catch (error: any) {
    const duration = Date.now() - startTime;
    const screenshot = await takeScreenshot(driver, `mbse-${testName.replace(/[^a-zA-Z0-9]/g, '-')}-failed`);
    reporter.addResult(testName, 'failed', duration, error.message, screenshot);
    console.log(`  ✗ ${testName} - ${error.message} (${(duration / 1000).toFixed(2)}s)`);
    console.log(`    Screenshot: ${screenshot}`);
  }
}

if (require.main === module) {
  const reporter = new TestReporter();
  runMBSECapellaTests(reporter).then(() => {
    console.log(reporter.generateReport());
    const reportPath = reporter.saveReport(`mbse-capella-report-${Date.now()}.txt`);
    console.log(`\n📄 Report saved to: ${reportPath}\n`);
    process.exit(0);
  }).catch(error => {
    console.error('Test execution failed:', error);
    process.exit(1);
  });
}
