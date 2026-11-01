// A folder opened in Restricted Mode, in the real VS Code: the compiler is not
// run, the model views say so once, and the extension host does not loop on
// refreshing them (it used to: a refusal was never remembered, so each refresh
// asked again, until the host's event queue overflowed with "Invalid array length").
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright node vscode_untrusted_test.js
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(78)} ${detail}`); };

function hostLogs(userData) {
  const found = [];
  const walk = (dir) => { for (const e of fs.existsSync(dir) ? fs.readdirSync(dir, { withFileTypes: true }) : []) { const p = path.join(dir, e.name); if (e.isDirectory()) walk(p); else if (e.name.startsWith('exthost') && e.name.endsWith('.log')) found.push(p); } };
  walk(path.join(userData, 'logs'));
  return found.map((p) => fs.readFileSync(p, 'utf8')).join('\n');
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcuntrusted-'));
  const model = path.join(dir, 'ev.arc');
  fs.copyFileSync(path.join(EXT, '..', '..', 'examples', 'automotive', 'ev_range_simulation.arc'), model);
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify({
    'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off',
    'workbench.startupEditor': 'none', 'git.enabled': false, 'window.restoreWindows': 'none',
    'security.workspace.trust.enabled': true, 'security.workspace.trust.startupPrompt': 'never', 'security.workspace.trust.banner': 'never',
  }));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', dir, model]);
  await win.waitForTimeout(6000);
  // The ArcLang view is the one that asks for the snapshot of the open model.
  await win.keyboard.press('F1');
  await win.keyboard.type('View: Show ArcLang');
  await win.waitForTimeout(800);
  await win.keyboard.press('Enter');
  await win.waitForTimeout(3000);
  // Switching the editor makes the model view follow the document (and ask for its snapshot).
  fs.writeFileSync(path.join(dir, 'other.arc'), 'model "Other" { description: "x" }\n');
  for (const name of ['other.arc', 'ev.arc']) {
    await win.keyboard.press('F1');
    await win.keyboard.type(`Go to File`);
    await win.waitForTimeout(600);
    await win.keyboard.press('Enter');
    await win.waitForTimeout(600);
    await win.keyboard.type(name);
    await win.waitForTimeout(800);
    await win.keyboard.press('Enter');
    await win.waitForTimeout(3000);
  }
  await win.waitForTimeout(8000);
  if (process.env.SHOT) await win.screenshot({ path: process.env.SHOT });
  const restricted = await win.locator('.statusbar-item', { hasText: /Restricted Mode/ }).count();
  report('the workspace is in Restricted Mode', restricted > 0);
  const log = hostLogs(path.join(dir, 'u'));
  const overflow = (log.match(/Invalid array length/g) || []).length;
  report('the extension host does not loop on refreshing the model views', overflow === 0, `${overflow} errors`);
  const settled = await win.locator('.pane-body', { hasText: /No model: ArcLang does not run/ }).count();
  report('the model view says why it shows nothing, instead of loading for ever', settled > 0);
  const tabs = await win.locator('.tabs-container .tab', { hasText: /other\.arc/ }).count();
  report('the editor still answers: the second model opened', tabs > 0);
  const simulation = await win.locator('.statusbar-item', { hasText: /Simulat/ }).count();
  report('no Simulation item in a restricted workspace', simulation === 0);
  await close();
  fs.rmSync(dir, { recursive: true, force: true });
  const failed = results.filter((ok) => !ok).length;
  console.log(`\n${results.length - failed}/${results.length} passed`);
  process.exit(failed ? 1 : 0);
})().catch((e) => { console.error(e); process.exit(1); });
