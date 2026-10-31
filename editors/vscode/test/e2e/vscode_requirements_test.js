// The requirements view, inside the real VS Code: the requirements, what
// derives from and refines them, what satisfies them and the test cases
// verifying them, each with its verdict; the Derive tool writes a
// `derives` trace between two requirements; the Requirement tool, with a
// requirement selected, writes a new one derived from it; ⌘Z takes it back.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_requirements_test.js
// SHOTS=dir saves pictures. Keys are sent as a French (AZERTY) Mac sends them.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

const MODEL = `model "Pack" {}

system_analysis "Needs" {
    requirement "Range" { id: "REQ-RANGE" description: "The car drives 300 km on one charge" }
    requirement "Cell temperature" { id: "REQ-CELL" description: "Cells stay under 55 °C" }
    requirement "Charge" { id: "REQ-CHARGE" description: "Charges in 30 min" parent: "REQ-RANGE" }
    requirement "Cooling" { id: "REQ-COOL" description: "The loop removes 5 kW" }
}

physical_architecture "Hardware" {
    node "Battery" { id: "PN-BATT" mass: 450 kg }
}

test_case "Range drive" { id: "TC-RANGE" verifies: ["REQ-RANGE", "REQ-CELL"] method: "test" verdict: "pass" }
test_case "Bench charge" { id: "TC-CHARGE" verifies: ["REQ-CHARGE"] method: "test" verdict: "fail" }

trace "REQ-CELL" derives "REQ-RANGE"
trace "PN-BATT" satisfies "REQ-RANGE"
`;

async function launch(dir, model) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
  await win.waitForTimeout(1500);
  return { app, win, page, press: await azerty(win) };
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcreq-'));
  const model = path.join(dir, 'model.arc');
  fs.writeFileSync(model, MODEL);
  const { app, win, page, press } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const source = async () => { await press('Meta+s'); await win.waitForTimeout(900); return fs.readFileSync(model, 'utf8'); };
  const settle = (ms = 3000) => win.waitForTimeout(ms);
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    const tab = page.locator('button, [role="tab"]', { hasText: /^Requirements/ }).first();
    report('the layer bar offers the Requirements view', (await tab.count()) > 0);
    await tab.click(); await settle(2500);
    const node = (id) => page.locator(`.arcviz-stage .av-node[data-id="${id}"]`).first();
    const shape = (id) => node(id).locator(':scope > .av-shape').first().boundingBox();
    const ids = await page.locator('.arcviz-stage .av-node').evaluateAll((ns) => ns.map((n) => n.getAttribute('data-id')));
    report('the four requirements, the two cases and the battery are drawn', ['REQ-RANGE', 'REQ-CELL', 'REQ-CHARGE', 'REQ-COOL', 'TC-RANGE', 'TC-CHARGE', 'PN-BATT'].every((id) => ids.includes(id)), ids.join(' '));
    const badge = async (id) => (await node(id).locator('.av-verdict').textContent().catch(() => '')) || '';
    report('each requirement shows its verdict', /verified/.test(await badge('REQ-RANGE')) && /fails/.test(await badge('REQ-CHARGE')) && /not verified/.test(await badge('REQ-COOL')), `${await badge('REQ-RANGE')} / ${await badge('REQ-CHARGE')} / ${await badge('REQ-COOL')}`);
    const lines = await page.locator('.arcviz-stage .av-edge').evaluateAll((es) => es.map((e) => (e.querySelector('.av-edge-label')?.textContent || '').replace(/\s+/g, ' ').trim()));
    report('derivation, satisfaction, verification and containment are drawn', lines.some((l) => /^derives/.test(l)) && lines.some((l) => /^satisfies/.test(l)) && lines.filter((l) => /^verifies/.test(l)).length === 3 && (await page.locator('.arcviz-stage .av-edge[data-kind="containment"]').count()) === 1, lines.join(' | '));
    report('a failing verification reads ✗, a passing one ✓', lines.some((l) => /verifies ✗/.test(l)) && lines.some((l) => /verifies ✓/.test(l)));
    await shot('requirements-01-view');

    // The Derive tool: from Cooling to Cell temperature, nothing asked.
    const derive = page.locator('[data-tool="derive"]').first();
    report('the palette has Derive, Refine and Satisfy', (await derive.count()) > 0 && (await page.locator('[data-tool="refine"]').count()) > 0 && (await page.locator('[data-tool="satisfy"]').count()) > 0);
    await derive.click(); await win.waitForTimeout(500);
    const cool = await shape('REQ-COOL'); await win.mouse.click(cool.x + cool.width - 20, cool.y + 10); await win.waitForTimeout(600);
    const cell = await shape('REQ-CELL'); await win.mouse.click(cell.x + cell.width - 20, cell.y + 10); await settle(4500);
    let text = await source();
    report('Derive writes the trace between the two requirements', /trace "REQ-COOL" derives "REQ-CELL"/.test(text), (text.match(/trace "REQ-COOL"[^\n]*/) || [''])[0]);
    report('and it is drawn at once', (await page.locator('.arcviz-stage .av-edge').evaluateAll((es) => es.filter((e) => /REQ-COOL/.test(e.getAttribute('data-source') || '') && /derives/.test(e.querySelector('.av-edge-label')?.textContent || '')).length)) === 1);
    await shot('requirements-02-derived');

    // The Requirement tool with Range selected: the new one derives from it.
    const range = await shape('REQ-RANGE'); await win.mouse.click(range.x + range.width - 20, range.y + 10); await win.waitForTimeout(700);
    await page.locator('[data-tool="requirement"]').first().click(); await win.waitForTimeout(500);
    const input = page.locator('.wb-prompt input').first();
    const label = (await page.locator('.wb-prompt label').first().innerText().catch(() => '')) || '';
    report('the Requirement tool says the new requirement derives from the one selected', /derived from Range/.test(label), label);
    if (await input.isVisible().catch(() => false)) { await input.fill('The pack holds 75 kWh'); await input.press('Enter'); await settle(4500); }
    text = await source();
    const made = /requirement "([^"]+)" \{\s*description: "The pack holds 75 kWh"/.exec(text);
    report('it is written with its derives trace', !!made && new RegExp(`trace "${made[1]}" derives "REQ-RANGE"`).test(text), made ? made[1] : text.slice(-300));
    await shot('requirements-03-new');
    await press('Meta+z'); await settle(4000);
    report('⌘Z takes the new requirement back', !/The pack holds 75 kWh/.test(await source()));
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
