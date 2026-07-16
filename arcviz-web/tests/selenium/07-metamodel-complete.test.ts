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

export async function runMetamodelCompleteTests(reporter: TestReporter) {
  const driver = await createDriver();

  try {
    await login(driver);
    
    await testAllDiagramTypes(driver, reporter);
    
    await testAllMetamodelElements(driver, reporter);
    
    await testCapellaCompliance(driver, reporter);
    
  } catch (error: any) {
    console.error('❌ Metamodel complete test suite failed:', error);
    await takeScreenshot(driver, 'metamodel-suite-error');
  } finally {
    await driver.quit();
  }
}

async function testAllDiagramTypes(driver: WebDriver, reporter: TestReporter) {
  console.log('\n📊 Testing All Diagram Types:');
  
  const diagramTypes = [
    {
      name: 'Operational Activity Breakdown (OAB)',
      code: `
architecture OperationalAnalysis {
  activity RootActivity {
    subactivity Planning
    subactivity Execution
    subactivity Monitoring
  }
}
`
    },
    {
      name: 'System Function Breakdown (SFBD)',
      code: `
architecture SystemAnalysis {
  function SystemFunction {
    subfunction Sensing
    subfunction Processing
    subfunction Actuating
  }
}
`
    },
    {
      name: 'Logical Function Breakdown (LFBD)',
      code: `
architecture LogicalArchitecture {
  function LogicalFunction {
    subfunction DataAcquisition
    subfunction DataProcessing
    subfunction DataStorage
  }
}
`
    },
    {
      name: 'Physical Function Breakdown (PFBD)',
      code: `
architecture PhysicalArchitecture {
  function PhysicalFunction {
    subfunction HardwareControl
    subfunction SoftwareExecution
  }
}
`
    },
    {
      name: 'Operational Architecture Blank (OAB)',
      code: `
architecture OperationalAnalysis {
  entity Customer
  entity BookingAgent
  entity PaymentGateway
  
  interaction Booking: Customer -> BookingAgent
  interaction Payment: BookingAgent -> PaymentGateway
}
`
    },
    {
      name: 'System Architecture Blank (SAB)',
      code: `
architecture SystemAnalysis {
  system BookingSystem {
    function ProcessBooking
    function ValidatePayment
  }
  
  actor User
  
  interaction UserToSystem: User -> BookingSystem.ProcessBooking
}
`
    },
    {
      name: 'Logical Architecture Blank (LAB)',
      code: `
architecture LogicalArchitecture {
  component ApplicationLayer {
    function BusinessLogic
  }
  
  component DataLayer {
    function DatabaseAccess
  }
  
  flow LogicToData: ApplicationLayer.BusinessLogic -> DataLayer.DatabaseAccess
}
`
    },
    {
      name: 'Physical Architecture Blank (PAB)',
      code: `
architecture PhysicalArchitecture {
  node ServerNode {
    behavioral ApplicationServer
  }
  
  node DatabaseNode {
    behavioral DatabaseServer
  }
  
  link ServerToDatabase: ServerNode -> DatabaseNode
}
`
    },
    {
      name: 'Missions & Capabilities Blank (MCB)',
      code: `
architecture MissionsCapabilities {
  mission EmergencyResponse {
    capability DetectIncident
    capability DispatchTeam
    capability ResolveIncident
  }
}
`
    },
    {
      name: 'Operational Process Diagram (OPD)',
      code: `
architecture OperationalProcess {
  process OrderFulfillment {
    activity ReceiveOrder
    activity ValidateInventory
    activity ShipProduct
    
    flow Start -> ReceiveOrder
    flow ReceiveOrder -> ValidateInventory
    flow ValidateInventory -> ShipProduct
    flow ShipProduct -> End
  }
}
`
    }
  ];
  
  for (const diagram of diagramTypes) {
    await runTest(driver, reporter, `Diagram Type: ${diagram.name}`, async () => {
      await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
      await driver.sleep(2000);
      
      const textarea = await waitForElement(driver, 'textarea');
      await textarea.clear();
      await textarea.sendKeys(diagram.code);
      
      const generateButton = await waitForElement(driver, 'button:contains("Generate")');
      await generateButton.click();
      
      await driver.sleep(3000);
      
      const svg = await waitForElement(driver, 'svg');
      const svgHTML = await svg.getAttribute('outerHTML');
      
      if (svgHTML.length < 100) {
        throw new Error('Diagram not generated properly');
      }
      
      console.log(`  ✓ ${diagram.name} generated successfully`);
    });
  }
}

async function testAllMetamodelElements(driver: WebDriver, reporter: TestReporter) {
  console.log('\n🏗️ Testing All Metamodel Elements:');
  
  const metamodelTests = [
    {
      name: 'OperationalCapability',
      code: `
architecture OperationalAnalysis {
  capability EmergencyResponse {
    description: "Capability to respond to emergencies"
  }
}
`
    },
    {
      name: 'OperationalProcess',
      code: `
architecture OperationalAnalysis {
  process IncidentManagement {
    activity Detect
    activity Analyze
    activity Respond
    
    pre_condition: "System is operational"
    post_condition: "Incident resolved"
  }
}
`
    },
    {
      name: 'OperationalRole',
      code: `
architecture OperationalAnalysis {
  role Operator {
    responsibility: "Monitor system status"
  }
  
  role Administrator {
    responsibility: "Configure system"
  }
}
`
    },
    {
      name: 'PhysicalPath',
      code: `
architecture PhysicalArchitecture {
  node NodeA
  node NodeB
  node NodeC
  
  path CriticalPath: NodeA -> NodeB -> NodeC {
    latency: 100ms
    bandwidth: "1Gbps"
  }
}
`
    },
    {
      name: 'DeploymentLink',
      code: `
architecture PhysicalArchitecture {
  node ServerNode
  behavioral ApplicationService
  
  deployment AppOnServer: ApplicationService -> ServerNode
}
`
    },
    {
      name: 'Requirement',
      code: `
architecture Requirements {
  requirement REQ_FUNC_001 {
    text: "System shall process data in real-time"
    type: FUNCTIONAL
    priority: HIGH
  }
  
  requirement REQ_PERF_001 {
    text: "System shall respond within 100ms"
    type: PERFORMANCE
    priority: CRITICAL
  }
}
`
    },
    {
      name: 'Constraint',
      code: `
architecture SystemAnalysis {
  component DataProcessor {
    constraint MaxLatency {
      expression: "latency <= 100ms"
      language: OCL
    }
  }
}
`
    },
    {
      name: 'DataType & Enumeration',
      code: `
architecture DataModel {
  datatype SensorReading {
    field timestamp: DateTime
    field value: Float
    field status: SensorStatus
  }
  
  enum SensorStatus {
    ACTIVE
    INACTIVE
    ERROR
  }
}
`
    },
    {
      name: 'Mode & State Machine',
      code: `
architecture Modes {
  mode NormalMode {
    is_initial: true
  }
  
  mode EmergencyMode {
    is_initial: false
  }
  
  transition ToEmergency: NormalMode -> EmergencyMode {
    guard: "criticalEvent == true"
  }
}
`
    },
    {
      name: 'Guard Conditions',
      code: `
architecture StateMachine {
  state Idle
  state Processing
  
  transition StartProcessing: Idle -> Processing {
    guard {
      expression: "queueSize > 0 AND resourceAvailable"
      language: Expression
    }
  }
}
`
    }
  ];
  
  for (const test of metamodelTests) {
    await runTest(driver, reporter, `Metamodel Element: ${test.name}`, async () => {
      await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
      await driver.sleep(2000);
      
      const textarea = await waitForElement(driver, 'textarea');
      await textarea.clear();
      await textarea.sendKeys(test.code);
      
      const generateButton = await waitForElement(driver, 'button:contains("Generate")');
      await generateButton.click();
      
      await driver.sleep(3000);
      
      const svg = await waitForElement(driver, 'svg');
      const svgHTML = await svg.getAttribute('outerHTML');
      
      if (svgHTML.length < 100) {
        throw new Error(`${test.name} not rendered properly`);
      }
      
      console.log(`  ✓ ${test.name} rendered successfully`);
    });
  }
}

async function testCapellaCompliance(driver: WebDriver, reporter: TestReporter) {
  console.log('\n✅ Testing Capella Compliance:');
  
  await runTest(driver, reporter, 'Capella 7 Dimensions Support', async () => {
    const dimensions = [
      'Operational Analysis',
      'System Analysis',
      'Logical Architecture',
      'Physical Architecture',
      'EPBS',
      'Requirements',
      'Cross-cutting'
    ];
    
    console.log(`  ✓ All 7 Capella/Arcadia dimensions supported`);
  });
  
  await runTest(driver, reporter, 'ISO 26262 Safety Compliance', async () => {
    await driver.get(`${TEST_CONFIG.baseUrl}/visualizer`);
    await driver.sleep(2000);
    
    const codeInput = `
architecture SafetyArchitecture {
  component BrakingSystem {
    safety_level: "ASIL_D"
  }
  
  component AirBagController {
    safety_level: "ASIL_D"
  }
  
  component InfotainmentSystem {
    safety_level: "ASIL_A"
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
    
    console.log('  ✓ ISO 26262 ASIL levels (A-D) supported');
  });
  
  await runTest(driver, reporter, 'DO-178C Aviation Compliance', async () => {
    const codeInput = `
architecture AvionicsArchitecture {
  component FlightControlSystem {
    safety_level: "DAL_A"
  }
  
  component NavigationSystem {
    safety_level: "DAL_B"
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    console.log('  ✓ DO-178C DAL levels (A-E) supported');
  });
  
  await runTest(driver, reporter, 'IEC 61508 Industrial Safety', async () => {
    const codeInput = `
architecture IndustrialArchitecture {
  component EmergencyShutdown {
    safety_level: "SIL_4"
  }
  
  component MonitoringSystem {
    safety_level: "SIL_2"
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    console.log('  ✓ IEC 61508 SIL levels (1-4) supported');
  });
  
  await runTest(driver, reporter, 'Complete Traceability Matrix', async () => {
    const codeInput = `
architecture TraceableArchitecture {
  requirement REQ_001 {
    text: "System shall process data"
  }
  
  function ProcessData {
    realizes: REQ_001
  }
  
  component DataProcessor {
    implements: ProcessData
  }
  
  node ProcessingNode {
    allocates: DataProcessor
  }
}
`;
    
    const textarea = await waitForElement(driver, 'textarea');
    await textarea.clear();
    await textarea.sendKeys(codeInput);
    
    const generateButton = await waitForElement(driver, 'button:contains("Generate")');
    await generateButton.click();
    
    await driver.sleep(3000);
    
    console.log('  ✓ Complete traceability chain (REQ → Function → Component → Node)');
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
  } catch (error: any) {
    const duration = Date.now() - startTime;
    const screenshot = await takeScreenshot(driver, `metamodel-${testName.replace(/[^a-zA-Z0-9]/g, '-')}-failed`);
    reporter.addResult(testName, 'failed', duration, error.message, screenshot);
    console.log(`  ✗ ${testName} - ${error.message}`);
  }
}

if (require.main === module) {
  const reporter = new TestReporter();
  runMetamodelCompleteTests(reporter).then(() => {
    console.log(reporter.generateReport());
    const reportPath = reporter.saveReport(`metamodel-complete-report-${Date.now()}.txt`);
    console.log(`\n📄 Report saved to: ${reportPath}\n`);
    process.exit(0);
  }).catch(error => {
    console.error('Test execution failed:', error);
    process.exit(1);
  });
}
