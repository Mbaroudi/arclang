// The OpenModelica simulation verdicts inside the real VS Code, driven by
// Playwright through Electron, on a copy of the EV range example: `arclang
// verify` simulates both cases in the OpenModelica Docker image (the standard
// pack fails its minimum state of charge, the long-range pack passes), then
// the requirement coverage table in VS Code shows each verdict next to its
// test case (read from the cache beside the saved file, though the table is
// computed on a temporary copy of the editor text) and a blocker in the gate.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang \
//   ARCLANG_OMC_DOCKER=openmodelica/openmodelica:v1.27.1-minimal node vscode_simulation_test.js
// SHOTS=/dir keeps pictures.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { spawnSync } = require('child_process');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const BINARY = process.env.ARCLANG_BINARY || 'arclang';
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail}`); };

async function tableFrame(win) {
  for (let i = 0; i < 80; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (await f.$('#tables .tb-row')) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

(async () => {
  if (!process.env.ARCLANG_OMC_DOCKER && !process.env.ARCLANG_OMC) { console.log('SKIP set ARCLANG_OMC_DOCKER=<OpenModelica image> or ARCLANG_OMC'); return; }
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcsim-'));
  const source = path.join(EXT, '..', '..', 'examples', 'automotive');
  fs.mkdirSync(path.join(dir, 'physics'));
  const model = path.join(dir, 'ev.arc');
  fs.copyFileSync(path.join(source, 'ev_range_simulation.arc'), model);
  fs.copyFileSync(path.join(source, 'physics', 'EVPhysics.mo'), path.join(dir, 'physics', 'EVPhysics.mo'));

  // 1. The simulations, as the command line runs them.
  const run = spawnSync(BINARY, ['verify', model], { encoding: 'utf8', timeout: 540000 });
  const out = run.stdout + run.stderr;
  report('verify simulates in OpenModelica (a tool is named)', /Tool: (docker image|omc)/.test(out) && /OpenModelica 1\./.test(out), (out.match(/Tool:.*/) || [''])[0]);
  report('the standard pack fails its minimum state of charge', /✗ TC-RANGE-STD[\s\S]*FAIL min\(batteryPack\.soc\)/.test(out));
  report('the long-range pack passes both checks', /✓ TC-RANGE-LR[\s\S]*PASS/.test(out));
  report('the verdicts are cached next to the model', fs.existsSync(path.join(dir, '.arclang', 'sim-cache', 'TC-RANGE-STD.json')));

  // 2. The same verdicts in VS Code.
  const settings = {
    'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off',
    'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none',
    ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}),
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  try {
    await win.waitForTimeout(4000);
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300);
    await win.keyboard.type('ArcLang: Requirement coverage table'); await win.waitForTimeout(600); await win.keyboard.press('Enter');
    const table = await tableFrame(win);
    report('the coverage table opens', !!table);
    if (!table) throw new Error('no table view');
    const verified = await table.textContent('.tb-row[data-id="REQ-RANGE"] [data-key="verified_by"]');
    report('each test case carries its simulation verdict', /standard pack\s*fail/.test(verified) && /long range pack\s*pass/.test(verified), verified.replace(/\s+/g, ' ').slice(0, 140));
    const gate = await table.textContent('.tb-gate');
    report('the gate badge counts the requirement as a blocker', /[1-9]\d* blocker/.test(gate), gate.replace(/\s+/g, ' ').trim());
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-simulation-coverage.png') });
  } catch (error) {
    report('the test ran to its end', false, String(error).slice(0, 200));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-simulation-error.png') }).catch(() => undefined);
  } finally {
    await close().catch(() => undefined);
  }
  const failed = results.filter((ok) => !ok).length;
  console.log(`\n${results.length - failed}/${results.length} checks passed`);
  process.exit(failed ? 1 : 0);
})();
