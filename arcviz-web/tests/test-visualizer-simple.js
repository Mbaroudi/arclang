const { Builder, By } = require('selenium-webdriver');
const chrome = require('selenium-webdriver/chrome');

async function test() {
  const options = new chrome.Options();
  options.addArguments('--headless');
  options.addArguments('--no-sandbox');
  options.addArguments('--window-size=1920,1080');

  const driver = await new Builder()
    .forBrowser('chrome')
    .setChromeOptions(options)
    .build();

  try {
    console.log('Loading visualizer...');
    await driver.get('http://localhost:3002/visualizer');
    await driver.sleep(3000);  // Wait for initial render
    
    const svgs = await driver.findElements(By.css('svg'));
    console.log('Total SVG elements:', svgs.length);
    
    for (let i = 0; i < svgs.length; i++) {
      const size = await svgs[i].getRect();
      console.log(`SVG ${i}: ${size.width}x${size.height}`);
      if (size.width > 500) {
        console.log('✓ Found large diagram SVG!');
        const rects = await driver.findElements(By.css('svg rect'));
        console.log('✓ Diagram has', rects.length, 'rectangles (nodes)');
        return;
      }
    }
    
    console.log('✗ No large diagram SVG found');
    
  } finally {
    await driver.quit();
  }
}

test();
