// A PDF opened in the real VS Code is shown as a document: its pages drawn,
// the zoom and the fit working (the simulation report of the unit tests).
//   VSCODE_APP=... PLAYWRIGHT=... PDF=report.pdf node vscode_pdf_test.js
const fs = require('fs'); const os = require('os'); const path = require('path');
const { launchVSCode } = require('./launch');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(60)} ${detail || ''}`); };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcpdf-'));
  const pdf = path.join(dir, 'report.pdf');
  fs.copyFileSync(process.env.PDF, pdf);
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none' };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', dir]);
  try {
    // Opened as a user does once VS Code is up: Quick Open.
    await win.waitForTimeout(4000);
    await win.keyboard.press('Meta+P'); await win.waitForTimeout(500); await win.keyboard.type('report.pdf'); await win.waitForTimeout(800); await win.keyboard.press('Enter');
    let view = null;
    for (let i = 0; i < 40 && !view; i++) {
      await win.waitForTimeout(500);
      for (const frame of win.frames()) if (await frame.locator('.sheet canvas').count().catch(() => 0)) view = frame;
    }
    report('the PDF opens in the PDF view, its pages drawn', Boolean(view));
    if (view) {
      report('three pages', (await view.locator('.sheet').count()) === 3 && (await view.locator('#count').innerText()) === '/ 3');
      const fitted = await view.locator('#zoom').innerText();
      await view.locator('#in').click(); await win.waitForTimeout(600);
      const zoomed = await view.locator('#zoom').innerText();
      report('zoom in goes up a step from the fitted width', zoomed !== fitted, `${fitted} → ${zoomed}`);
      await view.locator('#width').click(); await win.waitForTimeout(600);
      report('fit the width again', (await view.locator('#zoom').innerText()) === fitted);
      const blank = await view.locator('.sheet canvas').first().evaluate((canvas) => { const data = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data; for (let i = 0; i < data.length; i += 4) if (data[i] < 200) return false; return true; });
      report('the first page has ink on it', !blank);
    }
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'pdf-01-view.png') });
  } finally {
    await close();
  }
  const failed = rows.filter((r) => !r.ok).length;
  console.log(`${rows.length - failed}/${rows.length} passed`);
  process.exit(failed ? 1 : 0);
})();
