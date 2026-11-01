// The parametric view, inside the real VS Code: each constraint with its
// expression, the value of each side and whether it holds, bound to the
// values it reads inside their elements; a value changed in the text turns
// the constraint violated on the diagram.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_parametric_test.js
// SHOTS=dir saves pictures. Keys are sent as a French (AZERTY) Mac sends them.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

const MODEL = `model "Charger" {}

system_analysis "Needs" {
    function "Detect" { id: "SF-DET" latency: 10 ms }
    function "Stop" { id: "SF-STOP" latency: 40 ms }
    functional_chain "Emergency stop" { id: "FC-STOP" involves: ["SF-DET", "SF-STOP"] latency_budget: 100 ms }
}

physical_architecture "Hardware" {
    node "Cabinet" { id: "PN-PWR" voltage: 1000 V rated_current: 500 A power: 350 kW }
}

constraint "Stop within budget" { id: "CST-STOP" assert: sum("FC-STOP", latency) <= "FC-STOP".latency_budget }
constraint "Rating covers the power" { id: "CST-PWR" assert: "PN-PWR".voltage * "PN-PWR".rated_current >= "PN-PWR".power }
`;

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
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcpar-'));
  const model = path.join(dir, 'model.arc');
  fs.writeFileSync(model, MODEL);
  const { close, win, page } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const settle = (ms = 3000) => win.waitForTimeout(ms);
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    const tab = page.locator('button, [role="tab"]', { hasText: /^Parametric/ }).first();
    report('the layer bar offers the Parametric view', (await tab.count()) > 0);
    await tab.click(); await settle(2500);
    const node = (id) => page.locator(`.arcviz-stage .av-node[data-id="${id}"]`).first();
    const text = async (id) => ((await node(id).textContent().catch(() => '')) || '').replace(/\s+/g, ' ');
    report('the power constraint shows its sides in kW and holds', /500 kW against 350 kW/.test(await text('CST-PWR')) && /holds/.test(await text('CST-PWR')), (await text('CST-PWR')).slice(0, 160));
    // Each value is a box of its own, drawn inside the cabinet.
    const cabinet = await node('PN-PWR').locator(':scope > .av-shape').first().boundingBox();
    const values = [];
    for (const id of ['PN-PWR.voltage', 'PN-PWR.rated_current', 'PN-PWR.power']) {
      const box = await node(id).locator(':scope > .av-shape').first().boundingBox().catch(() => null);
      const inside = !!box && box.x >= cabinet.x && box.y >= cabinet.y && box.x + box.width <= cabinet.x + cabinet.width && box.y + box.height <= cabinet.y + cabinet.height;
      values.push({ id, inside, text: await text(id) });
    }
    report('the cabinet holds the three values it reads', values.every((v) => v.inside) && /= 1000 V/.test(values[0].text) && /= 350 kW/.test(values[2].text), values.map((v) => `${v.text}${v.inside ? '' : ' (outside)'}`).join(' | '));
    const bindings = await page.locator('.arcviz-stage .av-edge[data-kind="binding"]').count();
    report('every value is bound to its constraint', bindings === 6, `${bindings} bindings`);
    await shot('parametric-01-view');
    // The chain budget shrinks in the text: the constraint is violated.
    fs.writeFileSync(model, MODEL.replace('latency_budget: 100 ms', 'latency_budget: 30 ms'));
    await settle(5000);
    report('a budget made too small turns the constraint violated', /violated/.test(await text('CST-STOP')) && /50 ms against 30 ms/.test(await text('CST-STOP')), (await text('CST-STOP')).slice(0, 160));
    await shot('parametric-02-violated');
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
