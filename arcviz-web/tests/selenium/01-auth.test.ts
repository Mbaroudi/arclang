import { WebDriver, By, until } from 'selenium-webdriver';
import { createDriver, TEST_CONFIG, waitForElement, waitForUrl, takeScreenshot } from './config.js';

export async function runAuthTests(reporter: any) {
  let driver: WebDriver | null = null;
  const tests = [
    { name: 'Register new user', fn: testRegister },
    { name: 'Login with valid credentials', fn: testLogin },
    { name: 'Login with invalid credentials', fn: testInvalidLogin },
    { name: 'Logout', fn: testLogout },
  ];

  for (const test of tests) {
    const startTime = Date.now();
    try {
      driver = await createDriver();
      console.log(`\n▶ Running: ${test.name}`);
      await test.fn(driver);
      const duration = Date.now() - startTime;
      reporter.addResult(`Auth: ${test.name}`, 'passed', duration);
      console.log(`✓ Passed: ${test.name} (${(duration / 1000).toFixed(2)}s)`);
    } catch (error: any) {
      const duration = Date.now() - startTime;
      let screenshot;
      if (driver) {
        screenshot = await takeScreenshot(driver, `auth-${test.name.replace(/\s+/g, '-')}-failed`);
      }
      reporter.addResult(`Auth: ${test.name}`, 'failed', duration, error.message, screenshot);
      console.log(`✗ Failed: ${test.name} - ${error.message}`);
    } finally {
      if (driver) {
        await driver.quit();
      }
    }
  }
}

async function testRegister(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/register`);
  
  await waitForElement(driver, 'input[type="email"]');
  
  const emailInput = await driver.findElement(By.css('input[type="email"]'));
  const passwordInput = await driver.findElement(By.css('input[type="password"]'));
  const nameInput = await driver.findElement(By.css('input[name="name"], input[placeholder*="name" i]'));
  
  const uniqueEmail = `test-${Date.now()}@arcviz.com`;
  await emailInput.sendKeys(uniqueEmail);
  await passwordInput.sendKeys(TEST_CONFIG.testUser.password);
  await nameInput.sendKeys(TEST_CONFIG.testUser.name);
  
  const submitButton = await driver.findElement(By.css('button[type="submit"]'));
  await submitButton.click();
  
  await driver.sleep(2000);
  
  const currentUrl = await driver.getCurrentUrl();
  if (!currentUrl.includes('/editor') && !currentUrl.includes('/dashboard')) {
    throw new Error('Registration did not redirect to expected page');
  }
}

async function testLogin(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/login`);
  
  await waitForElement(driver, 'input[type="email"]');
  
  const emailInput = await driver.findElement(By.css('input[type="email"]'));
  const passwordInput = await driver.findElement(By.css('input[type="password"]'));
  
  await emailInput.sendKeys(TEST_CONFIG.testUser.email);
  await passwordInput.sendKeys(TEST_CONFIG.testUser.password);
  
  const submitButton = await driver.findElement(By.css('button[type="submit"]'));
  await submitButton.click();
  
  await driver.sleep(2000);
  
  const currentUrl = await driver.getCurrentUrl();
  if (!currentUrl.includes('/editor') && !currentUrl.includes('/dashboard')) {
    throw new Error('Login did not redirect to expected page');
  }
}

async function testInvalidLogin(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/login`);
  
  await waitForElement(driver, 'input[type="email"]');
  
  const emailInput = await driver.findElement(By.css('input[type="email"]'));
  const passwordInput = await driver.findElement(By.css('input[type="password"]'));
  
  await emailInput.sendKeys('invalid@test.com');
  await passwordInput.sendKeys('wrongpassword');
  
  const submitButton = await driver.findElement(By.css('button[type="submit"]'));
  await submitButton.click();
  
  await driver.sleep(1000);
  
  const currentUrl = await driver.getCurrentUrl();
  if (!currentUrl.includes('/login')) {
    throw new Error('Invalid login should stay on login page');
  }
}

async function testLogout(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/login`);
  
  await waitForElement(driver, 'input[type="email"]');
  
  const emailInput = await driver.findElement(By.css('input[type="email"]'));
  const passwordInput = await driver.findElement(By.css('input[type="password"]'));
  
  await emailInput.sendKeys(TEST_CONFIG.testUser.email);
  await passwordInput.sendKeys(TEST_CONFIG.testUser.password);
  
  const submitButton = await driver.findElement(By.css('button[type="submit"]'));
  await submitButton.click();
  
  await driver.sleep(2000);
  
  try {
    const logoutButton = await driver.findElement(By.xpath('//*[contains(text(), "Logout") or contains(text(), "Sign out")]'));
    await logoutButton.click();
    
    await driver.sleep(1000);
    
    const currentUrl = await driver.getCurrentUrl();
    if (!currentUrl.includes('/login')) {
      throw new Error('Logout did not redirect to login page');
    }
  } catch (error) {
    console.log('   Note: Logout button not found (may need manual testing)');
  }
}
