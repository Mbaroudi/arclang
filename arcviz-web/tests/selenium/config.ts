import { Builder, WebDriver, until, By } from 'selenium-webdriver';
import chrome from 'selenium-webdriver/chrome';

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

export async function createDriver(): Promise<WebDriver> {
  const options = new chrome.Options();
  options.addArguments('--headless');
  options.addArguments('--no-sandbox');
  options.addArguments('--disable-dev-shm-usage');
  options.addArguments('--disable-gpu');
  options.addArguments('--window-size=1920,1080');

  const driver = await new Builder()
    .forBrowser('chrome')
    .setChromeOptions(options)
    .build();

  await driver.manage().setTimeouts({ implicit: TEST_CONFIG.timeout });
  
  return driver;
}

export async function waitForElement(driver: WebDriver, selector: string, timeout = TEST_CONFIG.timeout) {
  return await driver.wait(
    until.elementLocated(By.css(selector)),
    timeout,
    `Element ${selector} not found within ${timeout}ms`
  );
}

export async function waitForUrl(driver: WebDriver, urlPattern: string | RegExp, timeout = TEST_CONFIG.timeout) {
  return await driver.wait(
    until.urlMatches(new RegExp(urlPattern)),
    timeout,
    `URL did not match ${urlPattern} within ${timeout}ms`
  );
}

export async function takeScreenshot(driver: WebDriver, name: string) {
  const screenshot = await driver.takeScreenshot();
  const fs = require('fs');
  const path = require('path');
  
  const screenshotDir = path.join(__dirname, 'screenshots');
  if (!fs.existsSync(screenshotDir)) {
    fs.mkdirSync(screenshotDir, { recursive: true });
  }
  
  const filePath = path.join(screenshotDir, `${name}-${Date.now()}.png`);
  fs.writeFileSync(filePath, screenshot, 'base64');
  
  return filePath;
}

export class TestReporter {
  private results: Array<{
    test: string;
    status: 'passed' | 'failed' | 'skipped';
    duration: number;
    error?: string;
    screenshot?: string;
  }> = [];

  private startTime = Date.now();

  addResult(test: string, status: 'passed' | 'failed' | 'skipped', duration: number, error?: string, screenshot?: string) {
    this.results.push({ test, status, duration, error, screenshot });
  }

  generateReport(): string {
    const totalDuration = Date.now() - this.startTime;
    const passed = this.results.filter(r => r.status === 'passed').length;
    const failed = this.results.filter(r => r.status === 'failed').length;
    const skipped = this.results.filter(r => r.status === 'skipped').length;

    let report = '\n' + '='.repeat(80) + '\n';
    report += '  SELENIUM TEST REPORT - ArcViz Platform\n';
    report += '='.repeat(80) + '\n\n';
    report += `Total Tests: ${this.results.length}\n`;
    report += `✓ Passed: ${passed}\n`;
    report += `✗ Failed: ${failed}\n`;
    report += `⊘ Skipped: ${skipped}\n`;
    report += `Duration: ${(totalDuration / 1000).toFixed(2)}s\n\n`;

    report += '='.repeat(80) + '\n';
    report += 'Test Results:\n';
    report += '='.repeat(80) + '\n\n';

    this.results.forEach((result, index) => {
      const icon = result.status === 'passed' ? '✓' : result.status === 'failed' ? '✗' : '⊘';
      report += `${index + 1}. ${icon} ${result.test} (${(result.duration / 1000).toFixed(2)}s)\n`;
      if (result.error) {
        report += `   Error: ${result.error}\n`;
      }
      if (result.screenshot) {
        report += `   Screenshot: ${result.screenshot}\n`;
      }
      report += '\n';
    });

    report += '='.repeat(80) + '\n';

    return report;
  }

  saveReport(filename: string) {
    const fs = require('fs');
    const path = require('path');
    const reportDir = path.join(__dirname, 'reports');
    
    if (!fs.existsSync(reportDir)) {
      fs.mkdirSync(reportDir, { recursive: true });
    }
    
    const filePath = path.join(reportDir, filename);
    fs.writeFileSync(filePath, this.generateReport());
    
    return filePath;
  }
}
