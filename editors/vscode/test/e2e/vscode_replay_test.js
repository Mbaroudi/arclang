// A simulation replayed on the diagram, inside the real VS Code: the aircraft
// network after the loss of its generator is simulated in OpenModelica, then
// *Replay a simulation on the diagram* shows each part's values under its box
// at the instant of a time bar; *First failure* goes to the instant the
// essential bus drops under 24 V, and outlines that part in red.
//   VSCODE_APP=... PLAYWRIGHT=... ARCLANG_BINARY=... ARCLANG_OMC_DOCKER=openmodelica/openmodelica:v1.27.1-minimal node vscode_replay_test.js
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { spawnSync } = require('child_process');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const REPO = path.resolve(EXT, '../..');
const BINARY = process.env.ARCLANG_BINARY || 'arclang';
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcreplay-'));
  const model = path.join(dir, 'aircraft.arc');
  fs.copyFileSync(path.join(REPO, 'examples/aerospace/aircraft_electrical_network.arc'), model);
  fs.mkdirSync(path.join(dir, 'physics'));
  fs.copyFileSync(path.join(REPO, 'examples/aerospace/physics/ElectricalNetworkPhysics.mo'), path.join(dir, 'physics/ElectricalNetworkPhysics.mo'));
  const verified = spawnSync(BINARY, ['verify', model, '--test', 'TC-LOSS-25'], { encoding: 'utf8', env: process.env });
  report('the case is simulated by OpenModelica first', /short by/.test(verified.stdout) && fs.existsSync(path.join(dir, '.arclang/sim-results/TC-LOSS-25/0.csv')), (verified.stdout.match(/FAIL[^\n]*/) || [verified.stderr.slice(-200)])[0]);
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'arclang.binary': BINARY };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', dir, model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(5000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(600); await win.keyboard.press('Enter'); await win.waitForTimeout(900); };
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  try {
    await command('View: Close Secondary Side Bar');
    await command('ArcLang: Replay a simulation on the diagram');
    await win.waitForTimeout(2500);
    await win.keyboard.type('Generator loss, 25 Ah'); await win.waitForTimeout(500); await win.keyboard.press('Enter');
    let page = null;
    for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.wb-replay')) page = f; } catch { /* a frame going away */ } } }
    report('the workbench opens with the replay bar', !!page);
    if (!page) throw new Error('no replay bar');
    await win.waitForTimeout(1500);
    const tag = async (id) => ((await page.locator(`.av-node[data-id="${id}"] .wb-replay-tag-text`).allTextContents().catch(() => [])).join(' | ')).replace(/\s+/g, ' ');
    report('each part shows its values under its box', /voltage 27\.\d+ V/.test(await tag('PN-ESS')) && /soc 0\.8/.test(await tag('PN-BAT')), `${await tag('PN-ESS')} | ${await tag('PN-BAT')}`);
    report('nothing has failed at the start', (await page.locator('.wb-replay-fail').count()) === 0);
    const jump = page.locator('.wb-replay-jump');
    report('the bar offers the first failure', /First failure at 37 min 26 s/.test(await jump.textContent()), await jump.textContent());
    await shot('replay-01-start');
    await jump.click(); await win.waitForTimeout(800);
    report('at the first failure the essential bus is outlined in red', (await page.locator('.av-node.wb-replay-fail[data-id="PN-ESS"]').count()) === 1);
    report('its value is under 24 V and the bar names the requirement', /voltage 2[34]\.\d+ V/.test(await tag('PN-ESS')) && /REQ-ESS-30MIN/.test(await page.locator('.wb-replay-status').textContent()), `${await tag('PN-ESS')} | ${await page.locator('.wb-replay-status').textContent()}`);
    report('the clock reads the instant', /^37 min 26 s \//.test(await page.locator('.wb-replay-time').textContent()), await page.locator('.wb-replay-time').textContent());
    await shot('replay-02-failure');
    // Play from the start: time moves on its own.
    await page.locator('.wb-replay-slider').fill('0'); await win.waitForTimeout(300);
    await page.locator('.wb-replay-play').click(); await win.waitForTimeout(2500);
    const moved = await page.locator('.wb-replay-time').textContent();
    report('Play moves through the run', !/^0 s \//.test(moved), moved);
    await page.locator('.wb-replay-close').click(); await win.waitForTimeout(600);
    report('× stops the replay and clears the boxes', (await page.locator('.wb-replay').count()) === 0 && (await page.locator('.wb-replay-tag').count()) === 0);
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
