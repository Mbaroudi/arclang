// The SysML v2 views, inside the real VS Code: the drone in a General View
// (definitions and parts with their keyword, how the air vehicle is made),
// its hardware in an Interconnection View (the altitude loop inside the
// flight controller, ports, named connections), its system analysis in an
// Action Flow View (items in and out, flows, the steps of the loop in order).
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_sysml_views_test.js
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

async function launch(dir, model) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
  await win.waitForTimeout(1500);
  return { close, win, page, press: await azerty(win) };
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcsv2-'));
  const model = path.join(dir, 'drone.arc');
  fs.copyFileSync(path.resolve(EXT, '../../examples/aerospace/drone_endurance.arc'), model);
  const { close, win, page } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const settle = (ms = 2500) => win.waitForTimeout(ms);
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    const tab = (name) => page.locator('button, [role="tab"]', { hasText: new RegExp(`^${name}`) }).first();
    const node = (id) => page.locator(`.arcviz-stage .av-node[data-id="${id}"]`).first();
    const text = async (id) => ((await node(id).textContent().catch(() => '')) || '').replace(/\s+/g, ' ');
    report('the layer bar offers the three SysML v2 views', (await tab('General View').count()) > 0 && (await tab('Interconnection').count()) > 0 && (await tab('Action Flow').count()) > 0);
    // General View
    await tab('General View').click(); await settle();
    report('the General View says what each element is', /«part» Battery/.test(await text('PN-BATT')) && /voltage : ElectricPotentialValue = 44.4 \[V\]/.test(await text('PN-BATT')), (await text('PN-BATT')).slice(0, 120));
    const made = await page.locator('.arcviz-stage .av-edge[data-kind="composition"]').count();
    report('and how the air vehicle is made', made >= 6, `${made} compositions`);
    await shot('sysml-01-general');
    // Interconnection View
    await tab('Interconnection').click(); await settle();
    const fc = await node('PN-FC').locator(':scope > .av-shape').first().boundingBox();
    const loop = await node('BC-ALT').locator(':scope > .av-shape').first().boundingBox();
    report('the altitude loop is drawn inside the flight controller', !!fc && !!loop && loop.x > fc.x && loop.y > fc.y && loop.x + loop.width < fc.x + fc.width);
    const labels = await page.locator('.arcviz-stage .av-edge[data-kind="connection"] .av-edge-label').allTextContents();
    report('its ten connections are drawn and named', labels.length === 10 && labels.includes('Power harness plus'), labels.slice(0, 4).join(', '));
    await shot('sysml-02-interconnection');
    // Action Flow View
    await tab('Action Flow').click(); await settle();
    const flows = await page.locator('.arcviz-stage .av-edge[data-kind="flow"] .av-edge-label').allTextContents();
    report('the flows between the items of the actions', flows.includes('altitude') && flows.includes('thrust demand'), flows.join(', '));
    const then = await page.locator('.arcviz-stage .av-edge[data-kind="succession"]').count();
    report('the steps of the altitude loop follow one another', then === 2 && /«action»\s*step1 :> SF-ALT/.test(await text('FC-ALT.step1')), `${then} successions`);
    await shot('sysml-03-action-flow');
  } catch (error) {
    report('the test ran to its end', false, error.message);
  } finally {
    await close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
    const failed = rows.filter((r) => !r.ok).length;
    console.log(`${rows.length - failed}/${rows.length} passed`);
    process.exit(failed ? 1 : 0);
  }
})();
