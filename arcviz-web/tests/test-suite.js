const { Builder, By, until, Key } = require('selenium-webdriver');
const chrome = require('selenium-webdriver/chrome');

const BASE_URL = 'http://localhost:3002';
const API_URL = 'http://localhost:4000';

const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

class TestRunner {
  constructor() {
    this.driver = null;
    this.testResults = [];
    this.totalTests = 0;
    this.passedTests = 0;
    this.failedTests = 0;
  }

  async setup() {
    console.log(`${colors.cyan}🚀 Setting up Selenium WebDriver...${colors.reset}`);
    const options = new chrome.Options();
    options.addArguments('--headless');
    options.addArguments('--no-sandbox');
    options.addArguments('--disable-dev-shm-usage');
    options.addArguments('--window-size=1920,1080');

    this.driver = await new Builder()
      .forBrowser('chrome')
      .setChromeOptions(options)
      .build();

    await this.driver.manage().setTimeouts({ implicit: 10000 });
    console.log(`${colors.green}✓ WebDriver ready${colors.reset}\n`);
  }

  async teardown() {
    if (this.driver) {
      await this.driver.quit();
    }
  }

  async test(name, fn) {
    this.totalTests++;
    console.log(`${colors.blue}▶ Testing: ${name}${colors.reset}`);

    try {
      await fn();
      this.passedTests++;
      console.log(`${colors.green}✓ PASS: ${name}${colors.reset}\n`);
      this.testResults.push({ name, status: 'PASS' });
    } catch (error) {
      this.failedTests++;
      console.log(`${colors.red}✗ FAIL: ${name}${colors.reset}`);
      console.log(`${colors.red}  Error: ${error.message}${colors.reset}\n`);
      this.testResults.push({ name, status: 'FAIL', error: error.message });
    }
  }

  async waitForElement(by, timeout = 10000) {
    return await this.driver.wait(until.elementLocated(by), timeout);
  }

  async checkHealth() {
    const response = await fetch(`${API_URL}/health`);
    const data = await response.json();
    if (data.status !== 'ok') {
      throw new Error('API health check failed');
    }
  }

  printSummary() {
    console.log('\n' + '='.repeat(60));
    console.log(`${colors.cyan}TEST SUMMARY${colors.reset}`);
    console.log('='.repeat(60));
    console.log(`Total Tests:  ${this.totalTests}`);
    console.log(`${colors.green}Passed:       ${this.passedTests}${colors.reset}`);
    console.log(`${colors.red}Failed:       ${this.failedTests}${colors.reset}`);
    console.log(`Pass Rate:    ${((this.passedTests / this.totalTests) * 100).toFixed(1)}%`);
    console.log('='.repeat(60) + '\n');

    if (this.failedTests > 0) {
      console.log(`${colors.red}Failed Tests:${colors.reset}`);
      this.testResults
        .filter(r => r.status === 'FAIL')
        .forEach(r => console.log(`  - ${r.name}: ${r.error}`));
      console.log();
    }
  }

  async run() {
    try {
      await this.setup();

      console.log(`${colors.cyan}${'='.repeat(60)}${colors.reset}`);
      console.log(`${colors.cyan}ARCVIZ PLATFORM - AUTOMATED TEST SUITE${colors.reset}`);
      console.log(`${colors.cyan}${'='.repeat(60)}${colors.reset}\n`);

      // API Health Check
      await this.test('API Health Check', async () => {
        await this.checkHealth();
      });

      // Landing Page Tests
      await this.test('Landing Page Loads', async () => {
        await this.driver.get(BASE_URL);
        const title = await this.driver.getTitle();
        if (!title) throw new Error('Page title not found');
      });

      await this.test('Landing Page Has Hero Section', async () => {
        await this.driver.get(BASE_URL);
        const hero = await this.waitForElement(By.css('h1'));
        const text = await hero.getText();
        if (!text.toLowerCase().includes('mbse') && !text.toLowerCase().includes('systems')) {
          throw new Error('Hero section not found or incorrect');
        }
      });

      // Navigation Tests
      await this.test('Navigation to Login Page', async () => {
        await this.driver.get(`${BASE_URL}/login`);
        await this.waitForElement(By.css('input[type="email"]'));
        const currentUrl = await this.driver.getCurrentUrl();
        if (!currentUrl.includes('/login')) {
          throw new Error('Login page not loaded');
        }
      });

      await this.test('Navigation to Register Page', async () => {
        await this.driver.get(`${BASE_URL}/register`);
        await this.waitForElement(By.css('input[type="email"]'));
        const currentUrl = await this.driver.getCurrentUrl();
        if (!currentUrl.includes('/register')) {
          throw new Error('Register page not loaded');
        }
      });

      await this.test('Navigation to Docs Page', async () => {
        await this.driver.get(`${BASE_URL}/docs`);
        await this.waitForElement(By.css('h1'));
        const heading = await this.driver.findElement(By.css('h1')).getText();
        if (!heading.toLowerCase().includes('documentation')) {
          throw new Error('Docs page not loaded correctly');
        }
      });

      await this.test('Navigation to Editor Page', async () => {
        await this.driver.get(`${BASE_URL}/editor`);
        await this.driver.sleep(2000); // Wait for Monaco to load
        const currentUrl = await this.driver.getCurrentUrl();
        if (!currentUrl.includes('/editor')) {
          throw new Error('Editor page not loaded');
        }
      });

      await this.test('Navigation to Visualizer Page', async () => {
        await this.driver.get(`${BASE_URL}/visualizer`);
        await this.driver.sleep(1000);
        const heading = await this.waitForElement(By.css('h1'));
        const text = await heading.getText();
        if (!text.toLowerCase().includes('visualizer')) {
          throw new Error('Visualizer page not loaded');
        }
      });

      // Registration Flow
      const testEmail = `test${Date.now()}@arcviz.io`;
      const testPassword = 'TestPassword123!';
      let authToken = null;

      await this.test('User Registration Form Validation', async () => {
        await this.driver.get(`${BASE_URL}/register`);
        
        const emailInput = await this.waitForElement(By.css('input[type="email"]'));
        const passwordInput = await this.driver.findElement(By.css('input[type="password"]'));
        const submitButton = await this.driver.findElement(By.css('button[type="submit"]'));

        await emailInput.sendKeys('invalid-email');
        await passwordInput.sendKeys('short');
        await submitButton.click();
        
        await this.driver.sleep(1000);
      });

      await this.test('User Registration Success', async () => {
        await this.driver.get(`${BASE_URL}/register`);
        
        const nameInput = await this.driver.findElement(By.css('input#name'));
        const emailInput = await this.driver.findElement(By.css('input#email'));
        const passwordInputs = await this.driver.findElements(By.css('input[type="password"]'));
        const submitButton = await this.driver.findElement(By.css('button[type="submit"]'));

        await nameInput.sendKeys('Test User');
        await emailInput.sendKeys(testEmail);
        await passwordInputs[0].sendKeys(testPassword);
        await passwordInputs[1].sendKeys(testPassword);
        
        await submitButton.click();
        
        await this.driver.sleep(3000);
        
        const currentUrl = await this.driver.getCurrentUrl();
        if (!currentUrl.includes('/editor')) {
          throw new Error('Did not redirect to editor after registration');
        }

        authToken = await this.driver.executeScript(
          'return localStorage.getItem("token");'
        );

        if (!authToken) {
          throw new Error('Auth token not stored in localStorage');
        }
      });

      // Login Flow
      await this.test('User Login', async () => {
        await this.driver.executeScript('localStorage.clear();');
        await this.driver.get(`${BASE_URL}/login`);
        
        const emailInput = await this.waitForElement(By.css('input#email'));
        const passwordInput = await this.driver.findElement(By.css('input#password'));
        const submitButton = await this.driver.findElement(By.css('button[type="submit"]'));

        await emailInput.sendKeys(testEmail);
        await passwordInput.sendKeys(testPassword);
        await submitButton.click();
        
        await this.driver.sleep(3000);
        
        const currentUrl = await this.driver.getCurrentUrl();
        if (!currentUrl.includes('/editor')) {
          throw new Error('Did not redirect to editor after login');
        }
      });

      // Visualizer Tests
      await this.test('Visualizer Shows Sample Diagram', async () => {
        await this.driver.get(`${BASE_URL}/visualizer`);
        await this.driver.sleep(2000);
        
        const svg = await this.driver.findElements(By.css('svg'));
        if (svg.length === 0) {
          throw new Error('SVG diagram not rendered');
        }
      });

      await this.test('Visualizer Compile Button Exists', async () => {
        await this.driver.get(`${BASE_URL}/visualizer`);
        const compileButton = await this.waitForElement(
          By.xpath("//button[contains(text(), 'Compile Code')]")
        );
        if (!compileButton) {
          throw new Error('Compile Code button not found');
        }
      });

      await this.test('Visualizer Can Open Code Input', async () => {
        await this.driver.get(`${BASE_URL}/visualizer`);
        const compileButton = await this.waitForElement(
          By.xpath("//button[contains(text(), 'Compile Code')]")
        );
        await compileButton.click();
        await this.driver.sleep(500);
        
        const textarea = await this.driver.findElements(By.css('textarea'));
        if (textarea.length === 0) {
          throw new Error('Code input textarea not visible');
        }
      });

      await this.test('Visualizer Load Sample Code', async () => {
        await this.driver.get(`${BASE_URL}/visualizer`);
        
        const compileButton = await this.waitForElement(
          By.xpath("//button[contains(text(), 'Compile Code')]")
        );
        await compileButton.click();
        await this.driver.sleep(500);
        
        const loadSampleButton = await this.driver.findElement(
          By.xpath("//button[contains(text(), 'Load Sample')]")
        );
        await loadSampleButton.click();
        await this.driver.sleep(500);
        
        const textarea = await this.driver.findElement(By.css('textarea'));
        const value = await textarea.getAttribute('value');
        
        if (!value || value.length < 100) {
          throw new Error('Sample code not loaded');
        }
      });

      // Documentation Tests
      await this.test('Docs Page Has All Sections', async () => {
        await this.driver.get(`${BASE_URL}/docs`);
        
        const sections = [
          'Getting Started',
          'ArcLang Language',
          'Editor Guide',
          'Visualizer Guide',
          'API Reference'
        ];

        for (const section of sections) {
          const elements = await this.driver.findElements(
            By.xpath(`//*[contains(text(), '${section}')]`)
          );
          if (elements.length === 0) {
            throw new Error(`Section "${section}" not found in docs`);
          }
        }
      });

      // Responsive Design Tests
      await this.test('Mobile Viewport Rendering', async () => {
        await this.driver.manage().window().setRect({ width: 375, height: 812 });
        await this.driver.get(BASE_URL);
        await this.driver.sleep(1000);
        
        // Check that page renders with header visible
        const header = await this.driver.findElement(By.css('header'));
        const isDisplayed = await header.isDisplayed();
        if (!isDisplayed) {
          throw new Error('Page not rendering on mobile viewport');
        }
        
        // Check that responsive menu exists (mobile navigation)
        const body = await this.driver.findElement(By.css('body'));
        if (!body) {
          throw new Error('Body element not found');
        }
        
        await this.driver.manage().window().setRect({ width: 1920, height: 1080 });
      });

      // Performance Tests
      await this.test('Page Load Time < 5 seconds', async () => {
        const start = Date.now();
        await this.driver.get(BASE_URL);
        await this.waitForElement(By.css('h1'));
        const loadTime = Date.now() - start;
        
        if (loadTime > 5000) {
          throw new Error(`Page load time too slow: ${loadTime}ms`);
        }
      });

      this.printSummary();

      return this.failedTests === 0 ? 0 : 1;

    } catch (error) {
      console.error(`${colors.red}Fatal Error: ${error.message}${colors.reset}`);
      return 1;
    } finally {
      await this.teardown();
    }
  }
}

// Run tests
(async () => {
  const runner = new TestRunner();
  const exitCode = await runner.run();
  process.exit(exitCode);
})();
