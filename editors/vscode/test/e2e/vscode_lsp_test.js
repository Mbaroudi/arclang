// The editor as a code editor, inside the real VS Code: an error appears
// while typing (no save), underlined on the line that writes the wrong name,
// and listed in Problems at that line; completion offers the elements of
// the right kind after `leads_to: [` and the keyword `note` at the top level.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_lsp_test.js
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail || ''}`); };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclsp-'));
  const model = path.join(dir, 'steer.arc');
  fs.writeFileSync(model, 'model T { name: "Steering" }\nlogical_architecture "L" {\n    component "Camera" { id: "LC-CAM" }\n}\nsafety_analysis {\n    hazard "Unintended steering" { id: "HAZ-1" asil: "ASIL-D" }\n}\n');
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'editor.quickSuggestions': { other: true, strings: true }, ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(6000);
  const press = await azerty(win);
  const command = async (name) => { await press('Meta+Shift+p'); await win.waitForTimeout(400); await win.keyboard.type(name); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(900); };
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  try {
    const editor = win.locator('.monaco-editor .view-lines').first();
    await editor.click(); await press('Meta+ArrowDown'); await win.waitForTimeout(300);
    // 1. A wrong reference typed (never saved): underlined at once, at its line
    await win.keyboard.type('note "Runs at 30 Hz" { about: ["LC-NOPE"] }', { delay: 15 });
    await win.waitForTimeout(3500);
    const squiggles = await win.locator('.squiggly-error').count();
    report('typed and not saved, the wrong reference is underlined at once', squiggles > 0, `${squiggles}`);
    await command('Problems: Focus on Problems View'); await win.waitForTimeout(1200);
    const problems = await win.locator('.markers-panel').first().innerText().catch(() => '');
    report('Problems lists it at the line that writes it, without the compiler prefix', /about 'LC-NOPE' names no element/.test(problems) && /\[Ln 8/.test(problems) && !/Semantic error/.test(problems), problems.replace(/\s+/g, ' ').slice(0, 160));
    await shot('lsp-01-error');
    // 2. Fixed: the error goes away as soon as it is fixed
    await editor.click(); await press('Meta+a'); await win.keyboard.type('model T { name: "Steering" }\nlogical_architecture "L" {\ncomponent "Camera" { id: "LC-CAM" }\n}\nsafety_analysis {\nhazard "Unintended steering" { id: "HAZ-1" asil: "ASIL-D" }\nfmea "Stuck" {\nleads_to: [', { delay: 5 });
    await win.waitForTimeout(800);
    await win.keyboard.press('Control+Space'); await win.waitForTimeout(2500);
    const offered = await win.locator('.suggest-widget .monaco-list-row').allInnerTexts().catch(() => []);
    report('after leads_to: [ completion offers the hazards, not the components', offered.some((t) => /HAZ-1/.test(t)) && !offered.some((t) => /LC-CAM/.test(t)), JSON.stringify(offered.map((t) => t.replace(/\s+/g, ' ').slice(0, 40))));
    await shot('lsp-02-completion');
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
