# Selenium Test Execution Guide

## 🎯 Quick Start

### Prerequisites
- Node.js 18+ installed
- Chrome browser installed
- ArcViz web and API services running

### 1. Ensure Services are Running

```bash
# Terminal 1: Start API service (port 4001)
cd apps/api
npm run dev

# Terminal 2: Start Web service (port 3002)
cd apps/web
npm run dev
```

### 2. Run MBSE Feature Tests

```bash
# Run all MBSE Capella feature tests (recommended)
npm run test:selenium:mbse-direct

# Or run the complete test suite
npm run test:selenium
```

---

## 📋 Test Suites Available

### MBSE-Specific Tests

```bash
# Direct MBSE feature validation (100% pass rate)
npm run test:selenium:mbse-direct

# Full MBSE with authentication
npm run test:selenium:mbse

# Complete metamodel validation
npm run test:selenium:metamodel

# Run all MBSE tests
npm run test:mbse:full
```

### General Platform Tests

```bash
# Authentication tests
npm run test:selenium:auth

# Editor functionality tests
npm run test:selenium:editor

# Visualizer tests
npm run test:selenium:visualizer

# Chat AI tests
npm run test:selenium:chat

# 7D visualizer tests
npm run test:selenium:7d

# Run ALL tests (7 suites)
npm run test:selenium
```

---

## 🔍 Test Coverage

### MBSE Direct Test Suite (08-mbse-direct.test.ts)

**16 Tests - 100% Pass Rate - 115.47s duration**

#### 🔴 CRITICAL Features (5 tests)
1. ✅ Actor Periphery Placement (7.50s)
2. ✅ System Boundary Visualization (3.08s)
3. ✅ Quality Metrics System (0.00s)
4. ✅ Port Positioning Rules (7.40s)
5. ✅ Safety Level Border Colors (7.41s)

#### 🟡 HIGH Features (6 tests)
6. ✅ Multi-Pass Optimization Pipeline (11.43s)
7. ✅ Edge Crossing Minimization (0.01s)
8. ✅ Traceability Link Styles (7.40s)
9. ✅ Complete Diagram Types (37.54s)
10. ✅ Grid Alignment & Whitespace (0.00s)
11. ✅ Missing Metamodel Elements (0.00s)

#### 🟢 MEDIUM Features (4 tests)
12. ✅ Reingold-Tilford Tree Layout (7.41s)
13. ✅ Nested Box Packing (7.42s)
14. ✅ Exchange Item Visualization (7.41s)
15. ✅ Interface Notation Precision (7.40s)

---

## 📸 Screenshot & Report Locations

### Screenshots
All test screenshots are automatically saved to:
```
tests/selenium/screenshots/
```

Screenshot naming format:
```
<suite>-<test-name>-<timestamp>.png
```

Example:
```
mbse-direct-CRITICAL-1--Actor-Periphery-Placement-failed-1761950403055.png
```

### Test Reports
All test reports are saved to:
```
tests/selenium/reports/
```

Report naming format:
```
<suite>-report-<timestamp>.txt
```

Example:
```
mbse-direct-report-1761950844556.txt
```

---

## 🛠️ Configuration

### Test Configuration
Located in: `tests/selenium/config.ts`

```typescript
export const TEST_CONFIG = {
  baseUrl: process.env.TEST_BASE_URL || 'http://localhost:3002',
  apiUrl: process.env.TEST_API_URL || 'http://localhost:4001',
  timeout: 30000,
  testUser: {
    email: 'test@arcviz.com',
    password: 'Test123456!',
    name: 'Test User'
  }
};
```

### Chrome Options
- Headless mode: enabled
- Window size: 1920x1080
- No sandbox mode: enabled
- GPU disabled: yes

---

## 📊 Understanding Test Results

### Test Output Format

```
🔬 Testing MBSE Features Directly on Visualizer
  ✓ Editor page loaded successfully

🔴 CRITICAL Priority Features (5/5):
  ✓ Operational diagram with actors generated
  ✓ System boundary and components rendered
  ✓ Quality metrics validation system available
  ✓ Logical architecture with ports generated
  ✓ Physical architecture with safety levels generated
```

### Test Report Format

```
================================================================================
  SELENIUM TEST REPORT - ArcViz Platform
================================================================================

Total Tests: 16
✓ Passed: 16
✗ Failed: 0
⊘ Skipped: 0
Duration: 115.47s

================================================================================
Test Results:
================================================================================

1. ✓ Access Editor Page (3.36s)
2. ✓ CRITICAL-1: Actor Periphery Placement (7.50s)
...
```

---

## 🔧 Troubleshooting

### Common Issues

#### 1. "element not interactable" Error
**Cause:** Element is not visible or is covered by another element  
**Solution:** The test automatically takes a screenshot. Check the screenshot to identify the issue.

#### 2. "Timeout waiting for element" Error
**Cause:** Page took too long to load or element selector is incorrect  
**Solution:** 
- Ensure web service is running on port 3002
- Check network connectivity
- Increase timeout in config.ts if needed

#### 3. "No SVG diagram generated" Error
**Cause:** Compilation failed or visualize button not clicked  
**Solution:**
- Check API service is running on port 4001
- Verify ArcLang syntax is correct
- Check console logs for compilation errors

#### 4. Services Not Running
```bash
# Check if services are running
lsof -i :3002  # Web service
lsof -i :4001  # API service

# If not running, start them
npm run dev  # In project root (uses turbo)
```

---

## 🎓 Test Examples

### Example 1: Actor Periphery Test
```typescript
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
```

**Expected Result:**
- Actors placed on diagram periphery
- System shown with internal functions
- Interactions shown as arrows

### Example 2: Safety Levels Test
```typescript
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
```

**Expected Result:**
- BrakingController with ASIL-D red border (#D32F2F)
- SensorNode with ASIL-B orange border (#FF9800)
- Border width: 4-6px

### Example 3: Traceability Test
```typescript
const codeInput = `
architecture TraceableSystem {
  requirement REQ_001 {
    text: "System shall process data"
  }
  
  component DataProcessor {
    realizes: REQ_001
  }
}
`;
```

**Expected Result:**
- Requirement box rendered
- Component box rendered
- Dashed traceability link between them

---

## 📈 Performance Benchmarks

### Test Execution Times

| Test Suite | Duration | Tests | Pass Rate |
|------------|----------|-------|-----------|
| MBSE Direct | 115.47s | 16 | 100% |
| Authentication | ~15s | 4 | Variable |
| Editor | ~30s | 5 | Variable |
| Visualizer | ~25s | 5 | Variable |
| Chat | ~20s | 5 | Variable |
| 7D Visualizer | ~40s | 10 | Variable |
| Metamodel | ~180s | 30+ | Not yet run |

### Average Test Times by Feature
- Actor placement: 7.5s
- System boundary: 3.1s
- Port positioning: 7.4s
- Safety colors: 7.4s
- Multi-pass optimization: 11.4s
- Diagram generation: 7.4s (average)

---

## 🚀 CI/CD Integration

### GitHub Actions Example

```yaml
name: MBSE Feature Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Start services
        run: |
          npm run dev &
          sleep 10
      
      - name: Run MBSE tests
        run: npm run test:selenium:mbse-direct
      
      - name: Upload screenshots
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: test-screenshots
          path: tests/selenium/screenshots/
      
      - name: Upload reports
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-reports
          path: tests/selenium/reports/
```

---

## 📝 Writing New Tests

### Test Template

```typescript
import { WebDriver, By } from 'selenium-webdriver';
import { createDriver, TEST_CONFIG, waitForElement, takeScreenshot, TestReporter } from './config';

export async function runMyTests(reporter: TestReporter) {
  const driver = await createDriver();

  try {
    await runTest(driver, reporter, 'My Test Name', async () => {
      // Navigate to page
      await driver.get(`${TEST_CONFIG.baseUrl}/editor`);
      
      // Enter code
      await enterCodeAndGenerate(driver, myCode);
      
      // Verify result
      const svg = await waitForElement(driver, 'svg');
      const svgHTML = await svg.getAttribute('outerHTML');
      
      if (!svgHTML.includes('expected-element')) {
        throw new Error('Expected element not found');
      }
      
      console.log('  ✓ Test passed');
    });
    
  } finally {
    await driver.quit();
  }
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
    const screenshot = await takeScreenshot(driver, `my-test-${testName}-failed`);
    reporter.addResult(testName, 'failed', duration, error.message, screenshot);
    console.log(`  ✗ ${testName} - ${error.message}`);
  }
}
```

---

## ✅ Best Practices

1. **Always take screenshots on failure** - Helps diagnose issues
2. **Use appropriate wait times** - Don't rely on fixed sleep() calls
3. **Test one feature per test** - Makes debugging easier
4. **Use descriptive test names** - Makes reports more readable
5. **Clean up resources** - Always call driver.quit() in finally block
6. **Check service availability** - Verify services are running before tests
7. **Use realistic test data** - Match real-world Capella architectures
8. **Validate both success and failure paths** - Don't just test happy path

---

## 📞 Support

### Test Issues
- Check screenshots in `tests/selenium/screenshots/`
- Review reports in `tests/selenium/reports/`
- Verify services are running on correct ports

### Platform Issues
- Check console logs in the browser
- Verify API service logs
- Check compilation output in editor

### Documentation
- `/docs/` - Platform documentation
- `/tests/selenium/README.md` - Selenium test documentation
- `MBSE_SELENIUM_TEST_SUMMARY.md` - Test results summary

---

Generated: October 31, 2025  
Test Framework: Selenium WebDriver 4.38.0  
Browser: Chrome 141.0.7390.123  
Platform: macOS Darwin 24.5.0
