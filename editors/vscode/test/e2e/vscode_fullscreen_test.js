// Full screen from any ArcLang view, in the real VS Code: the button in the
// title bar of the model's text and of a PDF gives the view the window (side
// bars, panel and status bar hidden), Esc and the shortcut give it back; the
// workbench's own button and the title bar's agree.
//   VSCODE_APP=... PLAYWRIGHT=... PDF=report.pdf node vscode_fullscreen_test.js
const fs = require('fs'); const os = require('os'); const path = require('path');
const { launchVSCode } = require('./launch');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const REPO = path.resolve(EXT, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail || ''}`); };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcfs-'));
  fs.copyFileSync(path.join(REPO, 'examples/01-getting-started/acc-minimal/acc_minimal.arc'), path.join(dir, 'acc.arc'));
  if (process.env.PDF) fs.copyFileSync(process.env.PDF, path.join(dir, 'report.pdf'));
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'arclang.binary': process.env.ARCLANG_BINARY || 'arclang' };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', dir]);
  const open = async (file) => { await win.keyboard.press('Meta+P'); await win.waitForTimeout(500); await win.keyboard.type(file); await win.waitForTimeout(800); await win.keyboard.press('Enter'); await win.waitForTimeout(2500); };
  const chrome = () => win.evaluate(() => ['.part.activitybar', '.part.sidebar', '.part.statusbar'].map((s) => { const e = document.querySelector(s); return Boolean(e && e.offsetWidth && e.offsetHeight && getComputedStyle(e).display !== 'none' && getComputedStyle(e).visibility !== 'hidden'); }).some(Boolean));
  const button = (label) => win.locator(`.editor-actions a.action-label[aria-label^="${label}"]`).first();
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  try {
    await win.waitForTimeout(4000);
    await open('acc.arc');
    report('the model\'s text has a Full screen button', await button('Full screen').isVisible().catch(() => false));
    report('before: side bars and status bar shown', await chrome());
    await button('Full screen').click(); await win.waitForTimeout(1500);
    await shot('fs-01-text');
    report('Full screen hides side bars and status bar', !(await chrome()));
    report('the button becomes Exit full screen', await button('Exit full screen').isVisible().catch(() => false));
    await win.locator('.monaco-editor .view-lines').first().click(); await win.keyboard.press('Escape'); await win.waitForTimeout(1500);
    report('Esc gives the window back', await chrome());
    await win.keyboard.press('Meta+Alt+Enter'); await win.waitForTimeout(1500);
    report('Cmd+Alt+Enter enters full screen', !(await chrome()));
    await win.keyboard.press('Meta+Alt+Enter'); await win.waitForTimeout(1500);
    report('Cmd+Alt+Enter leaves it', await chrome());
    if (process.env.PDF) {
      await open('report.pdf');
      report('a PDF has the button', await button('Full screen').isVisible().catch(() => false));
      await button('Full screen').click(); await win.waitForTimeout(1500);
      await shot('fs-02-pdf');
      report('the PDF in full screen', !(await chrome()));
      await button('Exit full screen').click(); await win.waitForTimeout(1500);
      report('Exit full screen from the PDF', await chrome());
    }
  } finally {
    await close();
  }
  const failed = rows.filter((r) => !r.ok).length;
  console.log(`${rows.length - failed}/${rows.length} passed`);
  process.exit(failed ? 1 : 0);
})();
