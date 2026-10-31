// A document generated from the model, inside the real VS Code: *Generate a
// document* writes the design review of the aircraft network as Word (read
// back by macOS's own reader), then a template of the user's as Markdown,
// opened in the editor with its directives filled.
//   VSCODE_APP=... PLAYWRIGHT=... ARCLANG_BINARY=... node vscode_document_test.js
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
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcdoc-'));
  const model = path.join(dir, 'aircraft.arc');
  fs.copyFileSync(path.join(REPO, 'examples/aerospace/aircraft_electrical_network.arc'), model);
  const template = path.join(dir, 'brief.md');
  fs.writeFileSync(template, '# {{model}}: battery brief\n\nThe light battery: {{value LC-BAT-25 capacity}}.\n\n{{elements LogicalComponent where "has(capacity)" columns capacity}}\n\n{{gate}}\n');
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'arclang.binary': BINARY };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', dir, model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(5000);
  const quick = win.locator('.quick-input-widget input').first();
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(600); await win.keyboard.press('Enter'); await win.waitForTimeout(1200); };
  const choose = async (text) => { await quick.fill(text); await win.waitForTimeout(500); await quick.press('Enter'); await win.waitForTimeout(1200); };
  const path_ = async (file) => { for (let i = 0; i < 20; i++) { if (await quick.isVisible().catch(() => false)) { await quick.fill(file); await win.waitForTimeout(300); await quick.press('Enter'); return; } await win.waitForTimeout(500); } };
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  try {
    // 1. The design review, as Word.
    await command('ArcLang: Generate a document');
    await choose('Design review');
    await choose('Word');
    const docx = path.join(dir, 'aircraft-review.docx');
    await path_(docx);
    for (let i = 0; i < 20 && !fs.existsSync(docx); i++) await win.waitForTimeout(500);
    report('the design review is written as Word', fs.existsSync(docx), docx);
    const text = fs.existsSync(docx) ? spawnSync('textutil', ['-convert', 'txt', '-stdout', docx], { encoding: 'utf8' }).stdout : '';
    report('Word reads it: the gate, its blockers and the coverage table', /Production gate \(ISO26262\): FAIL, \d\/3 requirements verified/.test(text) && /REQ-ESS-30MIN/.test(text) && /Load shedding/.test(text), text.split('\n').slice(0, 4).join(' / '));
    await shot('document-01-word');
    await win.keyboard.press('Escape');
    // 2. A template of mine, as Markdown, opened in the editor.
    await command('ArcLang: Generate a document');
    await choose('A template of mine');
    await path_(template);
    await win.waitForTimeout(800);
    await choose('Markdown');
    const md = path.join(dir, 'brief.out.md');
    await path_(md);
    for (let i = 0; i < 20 && !fs.existsSync(md); i++) await win.waitForTimeout(500);
    const written = fs.existsSync(md) ? fs.readFileSync(md, 'utf8') : '';
    report('the template is filled from the model', /^# Aircraft electrical network: battery brief\n\nThe light battery: 25 Ah\./.test(written) && /\| LC-BAT-40 \| 40 Ah battery \| 40 Ah \|/.test(written), written.split('\n').slice(0, 3).join(' / '));
    await win.waitForTimeout(800);
    const toast = win.locator('.notifications-toasts .notification-toast', { hasText: 'wrote brief.out.md' }).first();
    report('a toast says what was written and offers to open it', await toast.isVisible().catch(() => false));
    await toast.locator('a.monaco-button, .monaco-button', { hasText: 'Open' }).first().click().catch(() => {});
    await win.waitForTimeout(1500);
    const tab = await win.locator('.tab', { hasText: 'brief.out.md' }).count();
    report('Open shows it in the editor', tab > 0);
    await shot('document-02-markdown');
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
