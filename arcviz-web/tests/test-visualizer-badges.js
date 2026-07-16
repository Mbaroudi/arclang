const { Builder, By, until } = require('selenium-webdriver');
const assert = require('assert');

const WEB_URL = process.env.WEB_URL || 'http://localhost:3002';

async function testVisualizerBadges() {
  const driver = await new Builder().forBrowser('chrome').build();
  
  try {
    console.log('=== Testing Visualizer Badges ===\n');
    
    // Step 1: Go to editor first and check localStorage
    console.log('Step 1: Opening editor...');
    await driver.get(`${WEB_URL}/editor`);
    await driver.sleep(2000);
    
    // Check what's in localStorage
    const codeInStorage = await driver.executeScript(`
      return localStorage.getItem('arcviz_current_model') || 'NOT FOUND';
    `);
    
    console.log(`✓ Code in localStorage: ${codeInStorage.length} chars`);
    console.log(`✓ First 200 chars: ${codeInStorage.substring(0, 200)}\n`);
    
    // Step 2: Go to visualizer
    console.log('Step 2: Opening visualizer...');
    await driver.get(`${WEB_URL}/visualizer?from=editor`);
    await driver.sleep(3000);
    
    // Wait for diagrams to load
    console.log('Step 3: Waiting for diagrams to generate...');
    await driver.wait(until.elementLocated(By.css('[class*="grid"]')), 15000);
    await driver.sleep(2000);
    
    // Step 4: Check for badges
    console.log('Step 4: Checking for badges...\n');
    
    const badges = await driver.findElements(By.xpath("//span[contains(text(), 'Sample') or contains(text(), 'Your Code')]"));
    
    if (badges.length === 0) {
      console.log('⚠️  NO BADGES FOUND!');
      console.log('Checking page content...\n');
      
      const pageText = await driver.findElement(By.css('body')).getText();
      console.log('Page text preview:', pageText.substring(0, 500));
      
      // Check if there's an error message
      const hasError = pageText.includes('No code found') || pageText.includes('failed');
      if (hasError) {
        console.log('\n❌ ERROR: Visualizer shows error message');
        console.log('Possible issue: localStorage not working or code not being sent\n');
      }
      
      // Take screenshot
      const screenshot = await driver.takeScreenshot();
      require('fs').writeFileSync('/tmp/visualizer-no-badges.png', screenshot, 'base64');
      console.log('Screenshot saved: /tmp/visualizer-no-badges.png\n');
      
    } else {
      console.log(`✓ Found ${badges.length} badges\n`);
      
      // Count badge types
      let yourCodeCount = 0;
      let sampleCount = 0;
      
      for (let badge of badges) {
        const text = await badge.getText();
        if (text === 'Your Code') yourCodeCount++;
        if (text === 'Sample') sampleCount++;
      }
      
      console.log(`  • "Your Code" badges: ${yourCodeCount}`);
      console.log(`  • "Sample" badges: ${sampleCount}\n`);
      
      // Find diagram cards and their badges
      console.log('Diagram details:');
      const cards = await driver.findElements(By.css('[class*="group"][class*="cursor-pointer"]'));
      
      for (let i = 0; i < Math.min(cards.length, 10); i++) {
        try {
          const card = cards[i];
          const cardText = await card.getText();
          const lines = cardText.split('\n');
          const diagramName = lines[0] || 'Unknown';
          
          // Check if this card has a badge
          const cardBadges = await card.findElements(By.xpath(".//span[contains(text(), 'Sample') or contains(text(), 'Your Code')]"));
          const badgeText = cardBadges.length > 0 ? await cardBadges[0].getText() : 'NO BADGE';
          
          console.log(`  ${i + 1}. ${diagramName}: ${badgeText}`);
        } catch (e) {
          console.log(`  ${i + 1}. Error reading card: ${e.message}`);
        }
      }
    }
    
    // Step 5: Check API call in browser console
    console.log('\nStep 5: Checking browser console logs...');
    const logs = await driver.manage().logs().get('browser');
    const relevantLogs = logs.filter(entry => 
      entry.message.includes('Visualizer') || 
      entry.message.includes('generate-all') ||
      entry.message.includes('localStorage')
    );
    
    if (relevantLogs.length > 0) {
      console.log('Browser console:');
      relevantLogs.forEach(entry => {
        console.log(`  ${entry.level.name}: ${entry.message.substring(0, 200)}`);
      });
    }
    
    // Step 6: Check toast notifications
    console.log('\nStep 6: Checking toast notifications...');
    const toasts = await driver.findElements(By.css('[class*="toast"]'));
    if (toasts.length > 0) {
      console.log(`Found ${toasts.length} toast(s):`);
      for (let toast of toasts) {
        const toastText = await toast.getText();
        console.log(`  • ${toastText}`);
      }
    } else {
      console.log('No toast notifications found');
    }
    
    console.log('\n=== Test Complete ===');
    
  } catch (error) {
    console.error('Test failed:', error);
    
    // Take screenshot on error
    try {
      const screenshot = await driver.takeScreenshot();
      require('fs').writeFileSync('/tmp/visualizer-error.png', screenshot, 'base64');
      console.log('Error screenshot saved: /tmp/visualizer-error.png');
    } catch (e) {
      // ignore
    }
  } finally {
    await driver.quit();
  }
}

testVisualizerBadges().catch(console.error);
