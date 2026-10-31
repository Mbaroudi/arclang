// Monte Carlo results inside the real VS Code, driven by Playwright through
// Electron, on a copy of the EV range example whose two winter cases sample
// the outside air and the cruising speed: *ArcLang: Show Monte Carlo results*
// simulates every sample in OpenModelica, then a page shows, for each case,
// the probability shown against the one required, the spread of each margin,
// what moves it, and the failing samples; a failing sample replays on the
// diagram, and Python reads the same samples again.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang ARCLANG_PYTHON=/path/to/python-with-numpy \
//   ARCLANG_OMC_DOCKER=openmodelica/openmodelica:v1.27.1-minimal node vscode_montecarlo_test.js
// SHOTS=/dir keeps pictures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail}`); };

const SAMPLED = {
  'EVPhysics.CityRange': 'uncertain: ["vehicleBody.ambient ~ uniform(253.15 K, 303.15 K)", "\\"PN-VCU\\".speedController.target_speed ~ triangular(75 km/h, 95 km/h, 115 km/h)"] samples: 30 seed: 3 probability: 0.9',
  'EVPhysics.TouringRange': 'uncertain: ["vehicleBody.ambient ~ uniform(253.15 K, 293.15 K)", "\\"PN-VCU\\".speedController.target_speed ~ triangular(90 km/h, 100 km/h, 110 km/h)"] samples: 40 seed: 7 probability: 0.9',
};

/** The EV demo with its two winter cases sampled. */
function demo(dir) {
  const source = path.join(EXT, '..', '..', 'examples', 'automotive');
  fs.mkdirSync(path.join(dir, 'physics'));
  fs.copyFileSync(path.join(source, 'physics', 'EVPhysics.mo'), path.join(dir, 'physics', 'EVPhysics.mo'));
  let text = fs.readFileSync(path.join(source, 'ev_range_simulation.arc'), 'utf8');
  for (const [model, sampling] of Object.entries(SAMPLED)) {
    const plain = `simulate: { model: "${model}" configuration: "${model === 'EVPhysics.CityRange' ? 'CFG-CITY' : 'CFG-TOURING'}" stop_time: 5 h sources: ["physics/EVPhysics.mo"] }`;
    if (!text.includes(plain)) throw new Error(`the demo has no case simulating ${model}`);
    text = text.replace(plain, `${plain.slice(0, -2)} ${sampling} }`);
  }
  const model = path.join(dir, 'ev.arc');
  fs.writeFileSync(model, text);
  return model;
}

async function frameWith(win, selector, tries = 80) {
  for (let i = 0; i < tries; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (await f.$(selector)) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

(async () => {
  if (!process.env.ARCLANG_OMC_DOCKER && !process.env.ARCLANG_OMC) { console.log('SKIP set ARCLANG_OMC_DOCKER=<OpenModelica image> or ARCLANG_OMC'); return; }
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcmc-'));
  const model = demo(dir);
  const settings = {
    'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off',
    'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none',
    ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}),
    ...(process.env.ARCLANG_OMC_DOCKER ? { 'arclang.simulation.image': process.env.ARCLANG_OMC_DOCKER } : {}),
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  try {
    await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
    await win.waitForTimeout(4000);
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300);
    await win.keyboard.type('ArcLang: Show Monte Carlo results'); await win.waitForTimeout(600); await win.keyboard.press('Enter');

    // 1. 70 simulations, then the page.
    const page = await frameWith(win, '.case', 1200);
    report('the Monte Carlo page opens after the run', !!page);
    if (!page) throw new Error('no Monte Carlo page');
    await win.waitForTimeout(800);
    report('it shows the two sampled cases, not the sweep', (await page.locator('.case').count()) === 2);
    const touring = page.locator('.case', { hasText: 'TC-RANGE-LR' });
    const city = page.locator('.case', { hasText: 'TC-RANGE-STD' });
    report('the long range pack passes: its interval clears the requirement', (await touring.locator('.badge').textContent()) === 'PASS' && (await touring.locator('.band.shown').count()) === 1, (await touring.locator('.shows').textContent()).trim());
    report('the standard pack fails: some samples run out of charge', (await city.locator('.badge').textContent()) === 'FAIL' && (await city.locator('table.failing tbody tr').count()) > 0, (await city.locator('.shows').textContent()).trim());
    report('each case draws a histogram per margin', (await touring.locator('svg.histogram').count()) === 2 && (await city.locator('svg.histogram').count()) === 2);
    report('the failing bins of the city case are red, below the bound', (await city.locator('rect.bin.failing').count()) > 0 && (await city.locator('line.bound').count()) > 0);
    report('the distance of the touring case does not vary (ends on its bound)', /does not vary/.test(await touring.locator('.margin').first().textContent()));
    const speed = (await city.locator('.influences li', { hasText: 'target_speed' }).first().locator('.rho').textContent()).trim();
    report('the cruising speed moves the charge left the most, downwards', Number(speed) < -0.5, speed);
    await page.evaluate(() => window.scrollTo(0, 0));
    await shot('montecarlo-01-page');
    await city.scrollIntoViewIfNeeded(); await win.waitForTimeout(300);
    await shot('montecarlo-02-failing');

    // 2. Python reads the same samples again.
    if (process.env.ARCLANG_PYTHON) {
      await city.locator('details.statistics summary').click(); await win.waitForTimeout(300);
      await city.locator('button[data-engine="python"]').click();
      let done = false;
      for (let i = 0; i < 120 && !done; i++) { await win.waitForTimeout(500); done = (await city.locator('table.stats').count()) > 0 || (await city.locator('.status.error').count()) > 0; }
      const error = await city.locator('.status.error').textContent().catch(() => '');
      report('Python analyses the samples: PRCC with intervals', (await city.locator('table.stats').count()) > 0 && /PRCC/.test(await city.locator('table.stats').first().textContent()), error || '');
      await city.locator('details.statistics').scrollIntoViewIfNeeded(); await win.waitForTimeout(300);
      await shot('montecarlo-03-python');
    }

    // 3. A failing sample replays on the diagram.
    const label = (await city.locator('table.failing tbody tr td').first().textContent()).trim();
    await city.locator('table.failing button').first().click();
    const workbench = await frameWith(win, '.wb-replay', 80);
    report('a failing sample replays on the diagram', !!workbench);
    if (workbench) {
      await win.waitForTimeout(1500);
      const bar = (await workbench.locator('.wb-replay').textContent()).replace(/\s+/g, ' ');
      const sample = label.match(/^sample \d+/)[0];
      report('the replay is of that sample, and finds its failure', bar.includes(`(${sample} `) && /First failure at/.test(bar), bar.slice(0, 120));
      await shot('montecarlo-04-replay');
    }
  } catch (error) {
    report('the test ran to its end', false, String(error).slice(0, 200));
    await shot('montecarlo-error').catch(() => undefined);
  } finally {
    await app.close().catch(() => undefined);
  }
  const failed = results.filter((ok) => !ok).length;
  console.log(`\n${results.length - failed}/${results.length} checks passed`);
  process.exit(failed ? 1 : 0);
})();
