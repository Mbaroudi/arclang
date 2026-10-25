// The live tables inside the real VS Code, driven by Playwright through
// Electron, on a copy of the flagship example: the coverage table opens in
// its own tab, filters, a cell edited in place changes the source and ⌘Z
// (typed as on a French AZERTY Mac keyboard) takes it back, a trace is
// created from a matrix and removed again, CSV and Excel exports go through
// the save dialog (the workbook read back with openpyxl), Copy puts TSV on
// the clipboard.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang OPENPYXL_PYTHON=/venv/bin/python node vscode_tables_test.js
// SHOTS=/dir keeps pictures (light, dark, high contrast, the matrix).
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { execFileSync } = require('child_process');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(60)} ${detail}`); };

async function launch(dir, model, extra = {}) {
  const settings = {
    'files.simpleDialog.enable': true, 'window.dialogStyle': 'custom', 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false,
    'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none',
    ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}), ...extra,
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const command = async (name, pick = []) => {
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(600); await win.keyboard.press('Enter'); await win.waitForTimeout(700);
    for (const choice of pick) { await win.keyboard.type(choice); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(700); }
  };
  await command('View: Close Secondary Side Bar');
  return { app, win, command };
}

/** The frame of a table view (not one of `not`), once its rows are there. */
async function tableFrame(win, selector = '.tb-row', not = []) {
  for (let i = 0; i < 60; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (!not.includes(f) && await f.$(`#tables ${selector}`)) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

const rowIds = (frame) => frame.$$eval('.tb-row', (rows) => rows.sort((a, b) => a.getBoundingClientRect().top - b.getBoundingClientRect().top).map((r) => r.dataset.id));

async function saveTo(win, file) {
  const input = win.locator('.quick-input-widget input').first();
  for (let i = 0; i < 20 && !(await input.isVisible().catch(() => false)); i++) await win.waitForTimeout(250);
  await input.fill(file); await input.press('Enter'); await win.waitForTimeout(2500);
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arctables-'));
  const model = path.join(dir, 'aeb.arc');
  fs.copyFileSync(path.join(EXT, '..', '..', 'examples', 'complete_emergency_braking_simple.arc'), model);
  const original = fs.readFileSync(model, 'utf8');
  const { app, win, command } = await launch(dir, model);
  const press = await azerty(win);
  try {
    // 1. The coverage table opens in its own editor tab
    await command('ArcLang: Requirement coverage table');
    const table = await tableFrame(win);
    report('the coverage table opens in its own tab', !!table);
    if (!table) throw new Error('no table view');
    const tab = await win.locator('.tab', { hasText: 'Requirement coverage' }).count();
    report('the tab is an editor tab named after the table', tab >= 1);
    const ids = await rowIds(table);
    report('one row per requirement of the model', ids.length === 7 && ids[0] === 'REQ-AEB-001', ids.join(' '));
    report('satisfied by: links to the components', /RadarSensor/.test(await table.textContent('.tb-row[data-id="REQ-AEB-001"] [data-key="satisfied_by"]')));
    report('verified by: the test cases with their verdict', /Radar range verification\s*pending/.test(await table.textContent('.tb-row[data-id="REQ-AEB-001"] [data-key="verified_by"]')));
    report('the gate badge says what the gate says', /Gate ISO26262/.test(await table.textContent('.tb-gate')));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-coverage-light.png') });

    // 2. Filter
    await table.fill('.tb-filter', 'priority=medium'); await win.waitForTimeout(500);
    const filtered = await rowIds(table);
    report('the filter keeps the rows asked for', filtered.length === 1 && filtered[0] === 'REQ-AEB-005', filtered.join(' '));
    await table.fill('.tb-filter', ''); await win.waitForTimeout(400);

    // 3. A cell edited in place changes the source; ⌘Z takes it back
    const cell = '.tb-row[data-id="REQ-AEB-005"] [data-key="text"]';
    await table.dblclick(cell);
    await table.fill(`${cell} .tb-editor`, 'The driver shall be warned 1.5 s before autonomous braking');
    await table.press(`${cell} .tb-editor`, 'Enter');
    let changed = false;
    for (let i = 0; i < 30 && !changed; i++) { await win.waitForTimeout(300); changed = /1\.5 s before/.test(await table.textContent(cell)); }
    report('the edited cell shows the compiled value', changed);
    await command('File: Save All');
    const edited = fs.readFileSync(model, 'utf8');
    report('the source holds the edit', edited.includes('description: "The driver shall be warned 1.5 s before autonomous braking"') && !edited.includes('description: "The driver shall be warned before autonomous braking is applied"'));
    await table.click('.tb-status'); await win.waitForTimeout(300);
    await press('Meta+z'); await win.waitForTimeout(2000);
    await command('File: Save All');
    const undone = fs.readFileSync(model, 'utf8');
    report('⌘Z (AZERTY) in the table takes the edit back in the source', undone === original, undone === original ? '' : 'text differs');
    report('the table shows the text back', /before autonomous braking is applied/.test(await table.textContent(cell)));

    // 4. Exports through the save dialog
    await table.click('.tb-button:has-text("CSV")');
    await saveTo(win, path.join(dir, 'coverage.csv'));
    const csv = fs.existsSync(path.join(dir, 'coverage.csv')) ? fs.readFileSync(path.join(dir, 'coverage.csv'), 'utf8') : '';
    report('Export CSV writes the table', csv.startsWith('Req id,Name,Requirement text') && csv.split('\r\n').filter(Boolean).length === 8, csv.split('\r\n')[0]);
    await table.click('.tb-button:has-text("Excel")');
    await saveTo(win, path.join(dir, 'coverage.xlsx'));
    const book = path.join(dir, 'coverage.xlsx');
    // The Python that reads it back: OPENPYXL_PYTHON, else python3 when it has openpyxl.
    const python = process.env.OPENPYXL_PYTHON || (() => { try { execFileSync('python3', ['-I', '-c', 'import openpyxl']); return 'python3'; } catch { return null; } })();
    let read = null; let why = '';
    if (!fs.existsSync(book)) why = 'no workbook written (the save dialog or the writer)';
    else if (!python) why = 'no Python with openpyxl: set OPENPYXL_PYTHON';
    else {
      const script = 'import json,sys,openpyxl\nwb=openpyxl.load_workbook(sys.argv[1])\nws=wb.active\nprint(json.dumps({"title":ws.title,"rows":ws.max_row,"cols":ws.max_column,"head":[c.value for c in ws[1]],"a2":ws["A2"].value,"frozen":ws.freeze_panes,"filter":ws.auto_filter.ref}))';
      try { read = JSON.parse(execFileSync(python, ['-I', '-c', script, book]).toString()); } catch (error) { why = `openpyxl refused it: ${String(error).slice(0, 160)}`; }
    }
    report('Export Excel writes a workbook openpyxl reads', !!read && read.title === 'Requirement coverage' && read.rows === 8 && read.head[0] === 'Req id' && read.a2 === 'REQ-AEB-001' && read.frozen === 'A2' && !!read.filter, why || JSON.stringify(read));

    // 5. Copy: TSV on the clipboard
    execFileSync('/bin/sh', ['-c', 'printf nothing | pbcopy']);
    await table.click('.tb-button:has-text("Copy")'); await win.waitForTimeout(800);
    const clip = execFileSync('pbpaste').toString();
    report('Copy puts the table on the clipboard as TSV', clip.startsWith('Req id\tName\t') && clip.trim().split('\n').length === 8, clip.split('\n')[0]);

    // 6. A traceability matrix: a click creates a trace, a click on it removes it
    await command('ArcLang: New table…', ['Traceability matrix', 'Whole model']);
    const matrix = await tableFrame(win, '.tb-mrow', [table]);
    report('a new traceability matrix opens beside', !!matrix);
    if (!matrix) throw new Error('no matrix');
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-matrix.png') });
    const coordinates = await matrix.evaluate(() => {
      const columns = [...document.querySelectorAll('.tb-mcol')].map((c) => c.textContent.trim());
      const rows = [...document.querySelectorAll('.tb-mrow')].map((r) => r.querySelector('.tb-mname').textContent.trim());
      return { columns, rows };
    });
    const r = coordinates.rows.findIndex((n) => n.includes('Driver warning'));
    const c = coordinates.columns.findIndex((n) => n === 'RadarSensor');
    report('the matrix has the requirements and what they are traced with', r >= 0 && c >= 0, `${coordinates.rows.length}x${coordinates.columns.length}`);
    await matrix.click(`.tb-mrow[data-row="${r}"] .tb-mcell[data-col="${c}"]`);
    let created = false;
    for (let i = 0; i < 30 && !created; i++) { await win.waitForTimeout(300); created = (await matrix.$$(`.tb-mrow[data-row="${r}"] .tb-mcell.tb-mfilled[data-col="${c}"]`)).length === 1; }
    report('a click on an empty cell creates the trace, the cell fills', created);
    await command('File: Save All');
    report('the trace is in the source', /trace "(LC-)?RadarSensor" satisfies "REQ-AEB-005"/.test(fs.readFileSync(model, 'utf8')));
    await matrix.click(`.tb-mrow[data-row="${r}"] .tb-mcell[data-col="${c}"]`); await win.waitForTimeout(800);
    const confirm = win.locator('.monaco-dialog-box .monaco-button', { hasText: 'Remove' });
    const asked = await confirm.isVisible().catch(() => false);
    report('a click on a filled cell asks before removing', asked);
    if (SHOTS && asked) await win.screenshot({ path: path.join(SHOTS, 'vscode-matrix-remove.png') });
    if (asked) await confirm.click();
    await win.waitForTimeout(2500);
    await command('File: Save All');
    report('confirmed, the trace is gone from the source', fs.readFileSync(model, 'utf8') === original);

    // 7. Themes: the settings file changed, as a user picking a theme does
    if (SHOTS) {
      const settingsFile = path.join(dir, 'u', 'User', 'settings.json');
      const settings = JSON.parse(fs.readFileSync(settingsFile, 'utf8'));
      for (const [theme, file] of [['Default Dark Modern', 'vscode-coverage-dark.png'], ['Default High Contrast', 'vscode-coverage-hc.png'], ['Default Light Modern', null]]) {
        fs.writeFileSync(settingsFile, JSON.stringify({ ...settings, 'workbench.colorTheme': theme }));
        await win.waitForTimeout(2500);
        if (file) {
          await win.locator('.tab', { hasText: 'Requirement coverage' }).first().click(); await win.waitForTimeout(1500);
          await win.screenshot({ path: path.join(SHOTS, file) });
        }
      }
    }
  } catch (error) {
    report('the run went to its end', false, String(error && error.message));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-failure.png') }).catch(() => undefined);
  } finally {
    await app.close();
  }
  const failed = results.filter((ok) => !ok).length;
  console.log(failed ? `${failed} FAILED` : `all ${results.length} table checks passed in VS Code`);
  process.exit(failed ? 1 : 0);
})();
