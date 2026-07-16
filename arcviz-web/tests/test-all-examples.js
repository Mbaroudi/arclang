const { Builder, By, until } = require('selenium-webdriver')
const chrome = require('selenium-webdriver/chrome')
const fs = require('fs')
const path = require('path')

const EXAMPLES = [
  {
    name: 'Flight Control System',
    path: '/Users/malek/Arclang/examples/aerospace/flight_control_system.arc',
    minNodes: 3,
    minEdges: 2
  },
  {
    name: 'ACC Complete Architecture',
    path: '/Users/malek/Arclang/examples/automotive/acc_complete_architecture.arc',
    minNodes: 5,
    minEdges: 3
  },
  {
    name: 'ACC Minimal',
    path: '/Users/malek/Arclang/examples/automotive/acc_minimal.arc',
    minNodes: 2,
    minEdges: 1
  },
  {
    name: 'Adaptive Cruise Control',
    path: '/Users/malek/Arclang/examples/automotive/adaptive_cruise_control.arc',
    minNodes: 3,
    minEdges: 2
  },
  {
    name: 'ACC from Capella',
    path: '/Users/malek/Arclang/examples/automotive/acc_from_capella.arc',
    minNodes: 2,
    minEdges: 1
  },
  {
    name: 'Remote Start Architecture',
    path: '/Users/malek/Arclang/examples/automotive/remote_start/remote_start_architecture.arc',
    minNodes: 3,
    minEdges: 2
  },
  {
    name: 'Mission Computer',
    path: '/Users/malek/Arclang/examples/defense/mission_computer.arc',
    minNodes: 3,
    minEdges: 2
  },
  {
    name: 'Pluxee Analytics',
    path: '/Users/malek/Arclang/examples/business/pluxee_analytics.arc',
    minNodes: 2,
    minEdges: 1
  },
  {
    name: 'Data Platform Migration',
    path: '/Users/malek/Arclang/examples/business/data_platform_migration/data_platform_migration.arc',
    minNodes: 2,
    minEdges: 1
  }
]

const VISUALIZER_URL = 'http://localhost:3002/visualizer'
const TIMEOUT = 30000

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function testExample(driver, example) {
  console.log(`\n${'='.repeat(60)}`)
  console.log(`Testing: ${example.name}`)
  console.log(`${'='.repeat(60)}`)

  try {
    if (!fs.existsSync(example.path)) {
      throw new Error(`File not found: ${example.path}`)
    }

    const code = fs.readFileSync(example.path, 'utf-8')
    console.log(`✓ Loaded code (${code.length} bytes)`)

    await driver.get(VISUALIZER_URL)
    await driver.wait(until.elementLocated(By.css('textarea, .monaco-editor')), TIMEOUT)
    console.log('✓ Page loaded')

    const textarea = await driver.findElement(By.css('textarea'))
    await textarea.clear()
    await textarea.sendKeys(code)
    console.log('✓ Code inserted into editor')

    await sleep(500)

    const compileButton = await driver.findElement(By.xpath('//button[contains(text(), "Compile")]'))
    await compileButton.click()
    console.log('✓ Compile button clicked')

    await sleep(3000)

    const logs = await driver.manage().logs().get('browser')
    const consoleOutput = logs.map(entry => entry.message).join('\n')

    const hasError = consoleOutput.toLowerCase().includes('error') && 
                     !consoleOutput.includes('0 errors')
    
    if (hasError) {
      const errorLogs = logs.filter(entry => 
        entry.level.name === 'SEVERE' || 
        entry.message.toLowerCase().includes('error')
      )
      throw new Error(`Compilation errors:\n${errorLogs.map(e => e.message).join('\n')}`)
    }

    const statsElement = await driver.findElement(By.xpath('//*[contains(text(), "Components:") or contains(text(), "Requirements:")]'))
    const statsText = await statsElement.getText()
    console.log(`✓ Stats found: ${statsText}`)

    const svgElements = await driver.findElements(By.css('svg'))
    if (svgElements.length === 0) {
      await driver.takeScreenshot().then(image => {
        fs.writeFileSync(`error-${example.name.replace(/\s+/g, '-')}.png`, image, 'base64')
      })
      throw new Error('No SVG diagram found')
    }
    console.log(`✓ Diagram SVG rendered (${svgElements.length} svg elements)`)

    const nodes = await driver.findElements(By.css('svg .node, svg rect[rx="8"]'))
    const edges = await driver.findElements(By.css('svg .edge, svg path[stroke]'))
    
    console.log(`✓ Found ${nodes.length} nodes and ${edges.length} edges`)

    if (nodes.length < example.minNodes) {
      throw new Error(`Expected at least ${example.minNodes} nodes, got ${nodes.length}`)
    }

    if (edges.length < example.minEdges) {
      throw new Error(`Expected at least ${example.minEdges} edges, got ${edges.length}`)
    }

    await driver.takeScreenshot().then(image => {
      const filename = `screenshot-${example.name.replace(/\s+/g, '-')}.png`
      fs.writeFileSync(filename, image, 'base64')
      console.log(`✓ Screenshot saved: ${filename}`)
    })

    console.log(`\n✅ ${example.name}: PASSED`)
    return { success: true, example: example.name }

  } catch (error) {
    console.error(`\n❌ ${example.name}: FAILED`)
    console.error(`Error: ${error.message}`)
    
    try {
      await driver.takeScreenshot().then(image => {
        const filename = `error-${example.name.replace(/\s+/g, '-')}.png`
        fs.writeFileSync(filename, image, 'base64')
        console.log(`Error screenshot saved: ${filename}`)
      })
    } catch (screenshotError) {
      console.error('Failed to save error screenshot:', screenshotError.message)
    }

    return { success: false, example: example.name, error: error.message }
  }
}

async function runTests() {
  console.log('Starting ArcViz Examples Test Suite')
  console.log(`Testing ${EXAMPLES.length} examples`)
  
  const options = new chrome.Options()
  options.addArguments('--headless')
  options.addArguments('--disable-gpu')
  options.addArguments('--no-sandbox')
  options.addArguments('--disable-dev-shm-usage')
  options.addArguments('--window-size=1920,1080')
  options.setLoggingPrefs({ browser: 'ALL' })

  const driver = await new Builder()
    .forBrowser('chrome')
    .setChromeOptions(options)
    .build()

  const results = []

  try {
    for (const example of EXAMPLES) {
      const result = await testExample(driver, example)
      results.push(result)
      
      await sleep(1000)
    }

  } finally {
    await driver.quit()
  }

  console.log('\n' + '='.repeat(60))
  console.log('TEST SUMMARY')
  console.log('='.repeat(60))
  
  const passed = results.filter(r => r.success).length
  const failed = results.filter(r => !r.success).length
  
  console.log(`\nTotal: ${results.length}`)
  console.log(`Passed: ${passed} ✅`)
  console.log(`Failed: ${failed} ❌`)
  
  if (failed > 0) {
    console.log('\nFailed tests:')
    results.filter(r => !r.success).forEach(r => {
      console.log(`  ❌ ${r.example}: ${r.error}`)
    })
  }
  
  console.log('\n' + '='.repeat(60))
  
  process.exit(failed > 0 ? 1 : 0)
}

runTests().catch(error => {
  console.error('Fatal error:', error)
  process.exit(1)
})
