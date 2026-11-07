// The requirements table as a tree, inside the real VS Code: a requirement
// under another (`parent:`) sits indented under its row, a row folds the
// rows under it, and + on a row writes a new requirement under it
// (`REQ-1.3`, `parent: "REQ-1"`) in the source.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_requirements_tree_test.js
// SHOTS=dir saves pictures.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

const MODEL = `model "Car" {}

requirements {
    requirement "REQ-1" { description: "The car protects its occupants" }
    requirement "REQ-1.1" { description: "The car stops from 100 km/h within 40 m" parent: "REQ-1" }
    requirement "REQ-1.1.1" { description: "The wheels do not lock" parent: "REQ-1.1" }
    requirement "REQ-1.2" { description: "Six airbags protect the front seats" parent: "REQ-1" }
    requirement "REQ-2" { description: "The car drives 400 km on one charge" }
}
`;

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arctree-'));
  const model = path.join(dir, 'model.arc');
  fs.writeFileSync(model, MODEL);
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  try {
    await command('View: Close Secondary Side Bar');
    await command('ArcLang: Requirement coverage table');
    let table = null;
    for (let i = 0; i < 60 && !table; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('#tables .tb-row')) table = f; } catch { /* a frame going away */ } } }
    report('the coverage table opens', !!table);
    if (!table) throw new Error('no table view');
    await win.waitForTimeout(800);
    const order = () => table.locator('.tb-row').evaluateAll((rs) => rs.sort((a, b) => a.getBoundingClientRect().top - b.getBoundingClientRect().top).map((r) => r.getAttribute('data-id')));
    report('each requirement follows its parent', JSON.stringify(await order()) === JSON.stringify(['REQ-1', 'REQ-1.1', 'REQ-1.1.1', 'REQ-1.2', 'REQ-2']), (await order()).join(' '));
    const indent = (id) => table.locator(`.tb-row[data-id="${id}"] .tb-tree`).first().evaluate((n) => n.getBoundingClientRect().width);
    report('a row is indented by its depth', (await indent('REQ-1.1.1')) > (await indent('REQ-1.1')) && (await indent('REQ-1.1')) > (await indent('REQ-1')), `${await indent('REQ-1')} < ${await indent('REQ-1.1')} < ${await indent('REQ-1.1.1')}`);
    await shot('tree-01-table');

    await table.click('.tb-row[data-id="REQ-1"] .tb-twisty'); await win.waitForTimeout(500);
    report('folding REQ-1 hides the three rows under it', JSON.stringify(await order()) === JSON.stringify(['REQ-1', 'REQ-2']), (await order()).join(' '));
    await shot('tree-02-folded');
    await table.click('.tb-row[data-id="REQ-1"] .tb-twisty'); await win.waitForTimeout(500);
    report('unfolding shows them again', (await order()).length === 5);

    await table.hover('.tb-row[data-id="REQ-1"]');
    await table.click('.tb-row[data-id="REQ-1"] .tb-add-child'); await win.waitForTimeout(400);
    await table.fill('.tb-ask-input', 'The seat belts pretension in a crash');
    await shot('tree-03-asking');
    await table.press('.tb-ask-input', 'Enter');
    let made = false;
    for (let i = 0; i < 30 && !made; i++) { await win.waitForTimeout(300); made = (await order()).includes('REQ-1.3'); }
    report('+ adds REQ-1.3 under REQ-1, drawn at once', made && (await order()).indexOf('REQ-1.3') === 4, (await order()).join(' '));
    await command('File: Save All'); await win.waitForTimeout(800);
    const text = fs.readFileSync(model, 'utf8');
    report('the source says it, under its parent', /requirement "REQ-1\.3" \{\s*description: "The seat belts pretension in a crash"\s*parent: "REQ-1"\s*\}/.test(text), (text.match(/requirement "REQ-1\.3"[^}]*\}/) || [''])[0]);
    await shot('tree-04-added');
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
