// A digital twin in the real VS Code: the heat pump example's twin, its
// physics driven by a day of outdoor temperatures in OpenModelica and
// compared to the measured room temperature, shown with its verdict, its
// curves and the requirement it checks on the run.
//   VSCODE_APP=... PLAYWRIGHT=... ARCLANG_BINARY=... ARCLANG_OMC_DOCKER=... node vscode_twin_test.js
const fs = require('fs'); const os = require('os'); const path = require('path');
const { launchVSCode } = require('./launch');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const REPO = path.resolve(EXT, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(60)} ${detail || ''}`); };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arctwin-'));
  fs.cpSync(path.join(REPO, 'examples/building/heat-pump'), path.join(dir, 'house'), { recursive: true });
  const model = path.join(dir, 'house', 'heat_pump_heating.arc');
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'arclang.binary': process.env.ARCLANG_BINARY || 'arclang', 'arclang.simulation.image': process.env.ARCLANG_OMC_DOCKER || '' };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', dir, model], { width: 1600, height: 1000 });
  try {
    await win.waitForTimeout(6000);
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(400); await win.keyboard.type('ArcLang: Run the digital twins'); await win.waitForTimeout(700); await win.keyboard.press('Enter');
    let view = null;
    for (let i = 0; i < 120 && !view; i++) { await win.waitForTimeout(1000); for (const f of win.frames()) if (await f.locator('section.twin').count().catch(() => 0)) view = f; }
    report('the twin view opens with the twin', Boolean(view));
    if (!view) return;
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'twin-01.png') });
    report('the twin holds', (await view.locator('section.twin .badge').first().innerText()).toLowerCase() === 'holds');
    report('its comparison is drawn: measured, model, band', (await view.locator('.chart svg path.measured').count()) === 1 && (await view.locator('.chart svg path.simulated').count()) === 1 && (await view.locator('.chart svg path.band').count()) === 1);
    const numbers = await view.locator('.numbers').first().innerText();
    report('the share within the tolerance is given', /99\.\d % \(99 % required\)/.test(numbers.replace(/\s+/g, ' ')), numbers.replace(/\s+/g, ' ').slice(0, 90));
    report('the requirement is checked on the run', /min\("PN-SPACE"\.room\.T\) >= 20 °C/.test(await view.locator('.expects').innerText()));
    report('the data and its span are named', /house-2026-01-14\.csv · 289 samples over 24 h/.test(await view.locator('.meta').first().innerText()));
  } finally {
    await close();
  }
  const failed = rows.filter((r) => !r.ok).length;
  console.log(`${rows.length - failed}/${rows.length} passed`);
  process.exit(failed ? 1 : 0);
})();
