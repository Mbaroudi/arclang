// Simulation for a beginner, inside the real VS Code, driven by Playwright
// through Electron, on a copy of the EV range example (examples/automotive):
//   1. Docker present: the status item names the tool, its click opens the
//      Simulation menu, "Check simulation setup" is a checklist with every
//      Docker step done;
//   2. a simulation run from the menu ends on the verdicts (standard pack
//      fail, long range pack and its envelope pass), in the status item and in the table;
//   3. no Docker and no omc (a PATH without them): the item says "needs
//      setup", the checklist shows what to do with a button for each step,
//      and running the simulations opens that checklist, not a raw error.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang node vscode_simulation_setup_test.js
// Needs Docker with the OpenModelica image for 1 and 2 (they are skipped
// when `docker version` fails). SHOTS=/dir keeps pictures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { spawnSync } = require('child_process');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(78)} ${detail}`); };
const dockerUp = spawnSync('docker', ['version', '--format', '{{.Server.Version}}']).status === 0;

function workspace() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcsimsetup-'));
  const source = path.join(EXT, '..', '..', 'examples', 'automotive');
  fs.mkdirSync(path.join(dir, 'physics'));
  const model = path.join(dir, 'ev_range_simulation.arc');
  fs.copyFileSync(path.join(source, 'ev_range_simulation.arc'), model);
  fs.copyFileSync(path.join(source, 'physics', 'EVPhysics.mo'), path.join(dir, 'physics', 'EVPhysics.mo'));
  return { dir, model };
}

async function launch({ dir, model }, env = {}, binary = process.env.ARCLANG_BINARY) {
  const settings = {
    'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off',
    'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'git.enabled': false, 'window.restoreWindows': 'none',
    'workbench.colorTheme': 'Default Light Modern', 'window.autoDetectColorScheme': false,
    ...(binary ? { 'arclang.binary': binary } : {}),
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({
    executablePath: path.join(APP, 'Contents/MacOS/Electron'),
    args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', dir, model],
    env: { ...process.env, ...env },
  });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1500, height: 950 }));
  await win.waitForTimeout(4500);
  return { app, win };
}

const until = async (win, test, tries = 60, ms = 500) => { for (let i = 0; i < tries; i++) { const v = await test(); if (v) return v; await win.waitForTimeout(ms); } return null; };
const statusItem = (win) => win.locator('.statusbar-item', { hasText: /Simulat/ }).first();
const itemText = async (win) => ((await statusItem(win).count()) ? (await statusItem(win).textContent()).replace(/\s+/g, ' ').trim() : null);
const shot = async (win, name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, name) }); };

async function checklistFrame(win, wanted) {
  for (let i = 0; i < 80; i++) {
    for (const f of win.frames()) {
      try { const text = await f.textContent('#headline', { timeout: 200 }); if (!wanted || wanted.test(text)) return f; } catch { /* another frame */ }
    }
    await win.waitForTimeout(400);
  }
  return null;
}
const states = async (frame) => Object.fromEntries(await frame.$$eval('li.item', (nodes) => nodes.map((n) => [n.dataset.item, n.dataset.state])));
const menuRows = (win) => win.locator('.quick-input-widget .quick-input-list .monaco-list-row').allTextContents();

async function openMenu(win) {
  await statusItem(win).click();
  await until(win, async () => (await menuRows(win)).length >= 6, 20, 250);
  return menuRows(win);
}
async function chooseInMenu(win, label) {
  await win.locator('.quick-input-widget .quick-input-list .monaco-list-row', { hasText: label }).first().click();
  await win.waitForTimeout(700);
}

(async () => {
  if (dockerUp) {
    const w = workspace();
    const { app, win } = await launch(w);
    try {
      // 1. The status item and the menu.
      const text = await until(win, async () => { const t = await itemText(win); return t && /OpenModelica \(Docker\)/.test(t) ? t : null; }, 40);
      report('the status item names the tool found: OpenModelica (Docker)', !!text, text || '');
      await shot(win, 'sim-status-item.png');
      const rows = await openMenu(win);
      const wanted = ['Run simulations on this model', 'Check my setup', 'Set up OpenModelica', 'Export Modelica', 'Open the Modelica drawing', 'Open the documentation'];
      report('its click opens the Simulation menu with the six entries', wanted.every((w2) => rows.some((r) => r.includes(w2))), rows.map((r) => r.replace(/\s+/g, ' ').slice(0, 30)).join(' | '));
      await shot(win, 'sim-menu.png');
      // 2. The checklist with Docker present.
      await chooseInMenu(win, 'Check my setup');
      const frame = await checklistFrame(win, /Ready/);
      report('"Check my setup" opens a checklist that says Ready', !!frame);
      if (frame) {
        const s = await states(frame);
        report('Docker, the daemon, the image and the library are all done', ['docker', 'daemon', 'image', 'library'].every((k) => s[k] === 'ok'), JSON.stringify(s));
        report('Modex is offered as an optional install, with its button', s.modex === 'optional' && (await frame.locator('button[data-fix="modex:0"]').count()) === 1);
        report('no fix is asked of a ready machine for Docker', (await frame.locator('li[data-item="docker"] button, li[data-item="image"] button').count()) === 0);
        await shot(win, 'sim-checklist-docker.png');
      }
      // 3. A real run, from the menu, on the EV example.
      await win.keyboard.press('Escape');
      await win.locator('.tab', { hasText: 'ev_range_simulation.arc' }).first().click();
      await win.waitForTimeout(500);
      await openMenu(win);
      await chooseInMenu(win, 'Run simulations on this model');
      const running = await until(win, async () => { const t = await itemText(win); return t && /Simulating/.test(t) ? t : null; }, 40, 250);
      report('while it runs the item says Simulating…', !!running, running || '');
      const toast = await until(win, async () => { const t = (await win.locator('.notification-toast').allTextContents()).join(' ').replace(/\s+/g, ' '); return /fail, \d+ pass|simulated case/.test(t) ? t : null; }, 400, 1000);
      report('the run ends on "1 fail, 2 pass."', !!toast && /1 fail, 2 pass/.test(toast), (toast || '').slice(0, 120));
      const done = await itemText(win);
      report('the status item shows the verdicts', !!done && /Simulation: 1 fail, 2 pass/.test(done), done || '');
      await shot(win, 'sim-result.png');
      await win.locator('.notification-toast a, .notification-toast .monaco-button', { hasText: 'Show the verdicts' }).first().click().catch(() => undefined);
      let table = null;
      for (let i = 0; i < 60 && !table; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('#tables .tb-row')) { table = f; break; } } catch { /* gone */ } } }
      if (table) {
        const verified = await table.textContent('.tb-row[data-id="REQ-RANGE"] [data-key="verified_by"]');
        report('the table lists standard pack fail and long range pack pass', /standard pack\s*fail/.test(verified) && /long range pack\s*pass/.test(verified), verified.replace(/\s+/g, ' ').slice(0, 100));
        await shot(win, 'sim-verdict-table.png');
      } else report('the verdict table opens from the notification', false);
    } catch (error) {
      report('the Docker part ran to its end', false, String(error).slice(0, 200));
      await shot(win, 'sim-error-docker.png').catch(() => undefined);
    } finally { await app.close().catch(() => undefined); }
  } else console.log('SKIP the Docker part: no Docker daemon answers');

  // 4. No Docker, no omc.
  const w = workspace();
  // VS Code merges the login shell's PATH into its processes, so a PATH given to Electron is lost: the
  // compiler is started through a wrapper that removes docker and omc from its own PATH.
  const wrapper = path.join(w.dir, 'arclang-nodocker.sh');
  fs.writeFileSync(wrapper, `#!/bin/sh\nunset ARCLANG_OMC ARCLANG_OMC_DOCKER\nPATH=/usr/bin:/bin exec "${process.env.ARCLANG_BINARY || 'arclang'}" "$@"\n`, { mode: 0o755 });
  const { app, win } = await launch(w, {}, wrapper);
  try {
    const text = await until(win, async () => { const t = await itemText(win); return t && /needs setup/.test(t) ? t : null; }, 40);
    report('without Docker nor omc the item says needs setup', !!text, text || '');
    await shot(win, 'sim-status-needs-setup.png');
    await openMenu(win);
    await chooseInMenu(win, 'Check my setup');
    const frame = await checklistFrame(win, /needs setup/);
    report('the checklist says Simulation needs setup', !!frame);
    if (frame) {
      const s = await states(frame);
      report('Docker is to do, the next steps unknown, omc optional', s.docker === 'todo' && s.daemon === 'unknown' && s.image === 'unknown' && s.omc === 'optional', JSON.stringify(s));
      const button = await frame.locator('button[data-fix="docker:0"]').textContent();
      report('the Docker step has a one-click fix: Download Docker Desktop', /Download Docker Desktop/.test(button), button);
      await shot(win, 'sim-checklist-no-tool.png');
    }
    // Running the simulations opens the same help, not a raw error.
    await win.locator('.tab', { hasText: 'ev_range_simulation.arc' }).first().click();
    await win.waitForTimeout(500);
    await openMenu(win);
    await chooseInMenu(win, 'Run simulations on this model');
    const guided = await checklistFrame(win, /needs setup/);
    const banner = guided ? await guided.textContent('.banner').catch(() => '') : '';
    report('running without a tool opens the guided setup, with its explanation', !!guided && /needs OpenModelica first/.test(banner), banner);
    const errors = (await win.locator('.notification-toast.notification-error').allTextContents()).join(' ');
    report('and shows no raw error', errors === '', errors.slice(0, 100));
  } catch (error) {
    report('the no-tool part ran to its end', false, String(error).slice(0, 200));
    await shot(win, 'sim-error-notool.png').catch(() => undefined);
  } finally { await app.close().catch(() => undefined); }

  const failed = results.filter((ok) => !ok).length;
  console.log(`\n${results.length - failed}/${results.length} checks passed`);
  process.exit(failed ? 1 : 0);
})();
