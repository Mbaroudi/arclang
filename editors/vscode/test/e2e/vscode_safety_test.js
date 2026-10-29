// The safety report inside the real VS Code: the command builds it from the
// open model (hazards, FMEA by risk, a fault tree per hazard with its rate and
// budget) and opens it in the Markdown preview.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_safety_test.js
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail || ''}`); };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcsafety-'));
  const model = path.join(dir, 'station.arc');
  fs.copyFileSync(path.join(EXT, '..', '..', 'examples', 'energy', 'ev_charging_station.arc'), model);
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(5000);
  try {
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300);
    await win.keyboard.type('ArcLang: Safety report'); await win.waitForTimeout(600); await win.keyboard.press('Enter');
    let text = '';
    for (let i = 0; i < 40 && !/Safety report/.test(text); i++) {
      await win.waitForTimeout(500);
      for (const f of win.frames()) { try { const t = await f.evaluate(() => document.body ? document.body.innerText : ''); if (/Safety report: DC Fast-Charging Station/.test(t)) text = t; } catch { /* a frame going away */ } }
    }
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'safety-01-report.png') });
    report('the command opens the safety report of the model in the preview', /Safety report: DC Fast-Charging Station/.test(text));
    report('each hazard with its rate and verdict', /Electric shock at the DC output HAZ-001\s+ASIL-B\s+OR\s+2\s+35\s+100 \(ASIL-B\)\s+within/.test(text), (text.match(/Electric shock at the DC output HAZ-001[^\n]*/) || [''])[0]);
    report('a hazard of ASIL-A says it has no rate target', /no budget: ASIL-A sets no rate target/.test(text));
    report('the failure modes of each tree, with their element and rate', /DC contactor welded closed FMEA-001 of LC-PWR: 20 FIT/.test(text));
    report('the single points of failure are named', /Single points of failure: FMEA-001, FMEA-003/.test(text));
  } catch (error) {
    report('the test ran to its end', false, error.message);
  } finally {
    await app.close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
    const failed = rows.filter((r) => !r.ok).length;
    console.log(`${rows.length - failed}/${rows.length} passed`);
    process.exit(failed ? 1 : 0);
  }
})();
