// Importing and saving in the real VS Code, through the visible buttons: the
// model view's welcome (a Capella model, given by its .aird), the explorer's
// menu on a folder of SysML v2 files, the model view's Import button
// (ReqIF into the open model), the workbench's File menu (a SysML v2 API
// server: `arclang serve`), a file dropped on the sheet; then Save (● while
// unsaved, ⌘S, the file written, ⌘Z still there), Save as SysML v2 read
// back by `arclang import`, and auto save.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_import_test.js
// Keys are sent as a French (AZERTY) Mac keyboard sends them (azerty.js);
// SHOTS=/dir keeps pictures of each step.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const { execFileSync, spawn } = require('child_process');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const REPO = path.resolve(EXT, '../..');
const BIN = process.env.ARCLANG_BINARY || path.join(REPO, 'target/release/arclang');
const SHOTS = process.env.SHOTS || null;
const PORT = Number(process.env.API_PORT || 5594);
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail || ''}`); };
const shot = async (win, name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };

function workspace() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcimport-'));
  fs.cpSync(path.join(REPO, 'tests/fixtures/capella/test7_0'), path.join(dir, 'cap'), { recursive: true });
  fs.mkdirSync(path.join(dir, 'sysfolder'));
  for (const f of fs.readdirSync(path.join(REPO, 'tests/fixtures/sysml_abstract_syntax')).filter((f) => f.endsWith('.sysml'))) fs.copyFileSync(path.join(REPO, 'tests/fixtures/sysml_abstract_syntax', f), path.join(dir, 'sysfolder', f));
  fs.mkdirSync(path.join(dir, 'req'));
  fs.copyFileSync(path.join(REPO, 'tests/fixtures/reqif/Doors_10.reqif'), path.join(dir, 'req', 'Doors_10.reqif'));
  return dir;
}

async function launch(dir, home) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'window.menuStyle': 'custom', 'files.autoSave': 'off', 'workbench.colorTheme': process.env.THEME || 'Default Light Modern', 'arclang.binary': BIN };
  fs.mkdirSync(path.join(home, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(home, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${home}/u`, `--extensions-dir=${home}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', dir] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1680, height: 1050 }));
  await win.waitForTimeout(4000);
  const press = await azerty(win);
  const command = async (name) => { await press('Meta+Shift+p'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(700); await press('Enter'); await win.waitForTimeout(700); };
  return { app, win, press, command, settings };
}

/** The frame of a webview holding `selector`, not one of `not`. */
async function frameWith(win, selector, not = [], tries = 80) {
  for (let i = 0; i < tries; i++) {
    for (const f of win.frames()) { try { if (!not.includes(f) && await f.$(selector)) return f; } catch { /* a frame going away */ } }
    await win.waitForTimeout(500);
  }
  return null;
}

async function quickInput(win) {
  const input = win.locator('.quick-input-widget input').first();
  for (let i = 0; i < 40 && !(await input.isVisible().catch(() => false)); i++) await win.waitForTimeout(250);
  return input;
}
/** A quick pick or a simple dialog: the text typed, Enter. */
async function answer(win, text, wait = 900) {
  const input = await quickInput(win);
  if (text !== null) await input.fill(text);
  await win.waitForTimeout(500);
  await input.press('Enter');
  await win.waitForTimeout(wait);
}

/** The report of the last import: its verdict and counts (a new report replaces the old one's page). */
async function readReport(win, previous) {
  for (let i = 0; i < 120; i++) {
    const frame = await frameWith(win, '#verdict', [], 1);
    if (frame) {
      const verdict = await frame.locator('#verdict').textContent().catch(() => '');
      if (verdict && verdict !== previous) {
        const num = async (sel) => Number(await frame.locator(sel).textContent().catch(() => 'NaN'));
        return { frame, verdict, imported: await num('#imported'), leftOut: await num('#left-out'), metrics: await frame.locator('#metrics').textContent().catch(() => '') };
      }
    }
    await win.waitForTimeout(500);
  }
  return null;
}

(async () => {
  const dir = workspace();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'arcimport-home-'));
  const served = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'arcserve-')), 'aeb.arc');
  fs.copyFileSync(path.join(REPO, 'examples/complete_emergency_braking_simple.arc'), served);
  const { app, win, press, command, settings } = await launch(dir, home);
  const server = spawn(BIN, ['serve', '--port', String(PORT), served], { stdio: 'ignore' });
  try {
    // 1. No model yet: the model view welcomes with the imports.
    await command('ArcLang: Focus on Model View'); await win.waitForTimeout(2500);
    const welcome = win.locator('.welcome-view-content .monaco-button');
    const buttons = await welcome.allTextContents();
    report('an empty workspace: the model view offers the imports', ['Import a Capella model', 'Import SysML v2', 'Import requirements (ReqIF)', 'New model'].every((b) => buttons.includes(b)), buttons.join(' | '));
    await shot(win, 'import-welcome');

    // 2. A Capella model, chosen by its .aird, from the welcome
    await welcome.filter({ hasText: 'Import a Capella model' }).first().click();
    await answer(win, path.join(dir, 'cap', 'Model Test 7.0.aird'));
    await answer(win, path.join(dir, 'capella.arc'), 1500);
    let r = await readReport(win, null);
    report('Capella (.aird): the report says the model compiles', !!r && /capella\.arc was written and compiles/.test(r.verdict), r && `${r.verdict} imported ${r.imported}, left out ${r.leftOut}`);
    const checkErrors = JSON.parse(execFileSync(BIN, ['check', '--json', path.join(dir, 'capella.arc')], { encoding: 'utf8' })).findings.filter((f) => f.severity === 'error').length;
    report('… and states the error findings the check reports', checkErrors === 0 ? !/error finding/.test(r.verdict) : new RegExp(`reports ${checkErrors} error finding`).test(r.verdict), `${checkErrors} error findings`);
    report('… with what was imported and what was left out', !!r && r.imported > 500 && r.leftOut > 0, r && `${r.imported} / ${r.leftOut}`);
    report('… and read through the .capella beside the .aird', !!r && /read through Model Test 7\.0\.capella/.test(await r.frame.locator('body').textContent()));
    await shot(win, 'import-capella-report');
    await r.frame.locator('button[data-action="workbench"]').click();
    const capPage = await frameWith(win, '.arcviz-stage svg');
    report('Open workbench shows the imported model', !!capPage);
    await win.waitForTimeout(1500);
    await shot(win, 'import-capella-workbench');

    // 3. A folder of SysML v2 files, from the explorer's menu
    await command('View: Show Explorer'); await win.waitForTimeout(1200);
    await win.locator('.explorer-folders-view .monaco-list-row', { hasText: 'sysfolder' }).first().click({ button: 'right' }); await win.waitForTimeout(700);
    const folderItem = win.locator('.context-view .action-item', { hasText: 'Import SysML v2 folder as one model' }).first();
    const hasFolderItem = (await folderItem.count()) > 0;
    report('the explorer offers "Import SysML v2 folder as one model" on the folder', hasFolderItem);
    await shot(win, 'import-explorer-menu');
    if (hasFolderItem) {
      await folderItem.click();
      await answer(win, path.join(dir, 'sysml.arc'), 1500);
      r = await readReport(win, r?.verdict);
      report('SysML v2 folder: the report says the model compiles', !!r && /sysml\.arc was written and compiles/.test(r.verdict), r && `${r.verdict} imported ${r.imported}, left out ${r.leftOut}`);
      report('… the five files read as one model', !!r && /Every \.sysml and \.kerml file under sysfolder/.test(await r.frame.locator('body').textContent()));
      await shot(win, 'import-sysml-report');
    } else await press('Escape');

    // 4. ReqIF into the open model, from the model view's Import button
    await press('Meta+p'); await win.waitForTimeout(400); await win.keyboard.type('capella.arc'); await win.waitForTimeout(900); await press('Enter'); await win.waitForTimeout(1500);
    await command('ArcLang: Focus on Model View'); await win.waitForTimeout(1200);
    const importButton = win.locator('.pane-header .actions-container .action-label[aria-label^="Import a model"]').first();
    report('the model view has an Import button in its title bar', (await importButton.count()) > 0);
    await importButton.click(); await win.waitForTimeout(600);
    await answer(win, 'Requirements (ReqIF)');
    await answer(win, path.join(dir, 'req', 'Doors_10.reqif'));
    const where = await win.locator('.quick-input-widget .quick-input-list').textContent().catch(() => '');
    report('ReqIF with a model open: into it, or a new model', /Into capella\.arc/.test(where) && /A new model/.test(where), where.slice(0, 80));
    await answer(win, 'Into', 2500);
    r = await readReport(win, r?.verdict);
    report('ReqIF: the requirements are added to the open model, which compiles', !!r && /requirements were added to capella\.arc, which compiles/.test(r.verdict), r && `${r.verdict} ${r.metrics.replace(/\s+/g, ' ')}`);
    await shot(win, 'import-reqif-report');
    await press('Meta+p'); await win.waitForTimeout(400); await win.keyboard.type('capella.arc'); await win.waitForTimeout(900); await press('Enter'); await win.waitForTimeout(1200);
    await press('Meta+s'); await win.waitForTimeout(1500);
    const merged = fs.readFileSync(path.join(dir, 'capella.arc'), 'utf8');
    report('… one edit of the model, saved with ⌘S', /Requirements imported from Doors_10\.reqif[\s\S]*req "REQ-IMPORTED-001"/.test(merged));

    // 5. A SysML v2 API server, from the workbench's File menu (the workbench of capella.arc brought back to the front)
    await press('Meta+p'); await win.waitForTimeout(400); await win.keyboard.type('capella.arc'); await win.waitForTimeout(900); await press('Enter'); await win.waitForTimeout(1200);
    await command('ArcLang: Open the workbench beside the source'); await win.waitForTimeout(2000);
    await capPage.locator('#wb-file-menu').click(); await win.waitForTimeout(500);
    await shot(win, 'import-file-menu');
    await capPage.locator('.wb-menu-item', { hasText: 'Import a model' }).click(); await win.waitForTimeout(800);
    await answer(win, 'SysML v2 API server');
    await answer(win, `http://localhost:${PORT}/api/sysml-v2`);
    await answer(win, '', 1500); // no token
    await answer(win, null, 1500); // the one project
    await answer(win, path.join(dir, 'api.arc'), 2000);
    r = await readReport(win, r?.verdict);
    report('API server: the commit is imported and compiles', !!r && /api\.arc was written and compiles/.test(r.verdict), r && `${r.verdict} imported ${r.imported}, left out ${r.leftOut}`);
    report('… its elements read from the server', !!r && /elements of the commit .* read from http:\/\/localhost/.test(await r.frame.locator('body').textContent()));
    await shot(win, 'import-api-report');

    // 6. A ReqIF file dropped on the sheet is offered for import
    await capPage.evaluate((uri) => {
      const sheet = document.querySelector('.wb-sheet');
      const data = new DataTransfer();
      data.setData('text/uri-list', uri);
      sheet.dispatchEvent(new DragEvent('drop', { dataTransfer: data, bubbles: true, cancelable: true }));
    }, `file://${encodeURI(path.join(dir, 'req', 'Doors_10.reqif'))}`);
    await win.waitForTimeout(1200);
    const toast = await win.locator('.notifications-toasts').textContent().catch(() => '');
    report('a .reqif dropped on the sheet is offered for import', /import Doors_10\.reqif as an ArcLang model\?/.test(toast), toast.slice(0, 90));
    await shot(win, 'import-drop-offer');
    await command('Notifications: Clear All Notifications');

    // 7. Save: ● while unsaved, ⌘S writes the file, ⌘Z still there
    await r.frame.locator('button[data-action="workbench"]').click(); await win.waitForTimeout(2500);
    const page = await frameWith(win, '.arcviz-stage svg', [capPage]);
    if (!page) throw new Error('no workbench on api.arc');
    const save = page.locator('#wb-save');
    report('a saved model: the button says Saved, disabled', (await save.textContent()).includes('Saved') && await save.isDisabled());
    if (await page.locator('.wb-right').isHidden()) await page.locator('.wb-toolrow button[title="Properties"]').click();
    const box = page.locator('.arcviz-stage g[data-id]').filter({ has: page.locator('rect') }).nth(1);
    const bb = await box.boundingBox();
    await win.mouse.click(bb.x + 20, bb.y + 8); await win.waitForTimeout(900);
    const nameInput = page.locator('.wb-name-input');
    const before = await nameInput.inputValue();
    const renamed = `${before} Renamed`;
    await nameInput.fill(renamed); await press('Enter'); await win.waitForTimeout(3500);
    const api = path.join(dir, 'api.arc');
    const dirtyShown = await save.evaluate((b) => b.classList.contains('is-dirty') && !b.querySelector('.wb-dirty-dot').hidden && !b.disabled);
    const tabDirty = await win.locator('.tabs-container .tab.dirty', { hasText: 'api.arc' }).count();
    report('an edit on the canvas: ● on the Save button and on the tab', dirtyShown && tabDirty > 0, `${before} → ${renamed}`);
    report('… the file on disk is not written yet', !fs.readFileSync(api, 'utf8').includes(renamed));
    await shot(win, 'save-dirty');
    await win.mouse.click(bb.x + bb.width + 160, bb.y + bb.height + 160); await win.waitForTimeout(500);
    await press('Meta+s'); await win.waitForTimeout(1500);
    report('⌘S (AZERTY) on the canvas writes the file', fs.readFileSync(api, 'utf8').includes(renamed));
    report('… and the ● goes', await save.isDisabled() && (await save.textContent()).includes('Saved') && (await win.locator('.tabs-container .tab.dirty', { hasText: 'api.arc' }).count()) === 0);
    await shot(win, 'save-saved');
    await press('Meta+z'); await win.waitForTimeout(3000);
    report('⌘Z after the save takes the rename back (the history is kept)', !(await page.locator('.arcviz-stage').textContent()).includes(renamed) && await save.evaluate((b) => b.classList.contains('is-dirty')));
    await page.locator('#wb-save').click(); await win.waitForTimeout(1500);
    report('the Save button saves it', !fs.readFileSync(api, 'utf8').includes(renamed) && await save.isDisabled());

    // 8. Save as SysML v2, read back by the compiler
    await page.locator('#wb-file-menu').click(); await win.waitForTimeout(400);
    await page.locator('.wb-menu-item', { hasText: 'Save as SysML v2' }).click(); await win.waitForTimeout(800);
    await answer(win, path.join(dir, 'api.sysml'), 2500);
    const sysml = path.join(dir, 'api.sysml');
    let readBack = false; let why = 'no file';
    if (fs.existsSync(sysml)) {
      try { execFileSync(BIN, ['import', sysml, '-f', 'sysml', '-o', path.join(dir, 'roundtrip.arc')], { stdio: 'pipe' }); readBack = true; } catch (e) { why = String(e.stderr || e.message).slice(0, 200); }
    }
    report('Save as SysML v2 writes a file arclang import -f sysml reads back', readBack, readBack ? `${fs.statSync(sysml).size} bytes` : why);

    // 9. Auto save on: the button says Saved, disabled
    fs.writeFileSync(path.join(home, 'u', 'User', 'settings.json'), JSON.stringify({ ...settings, 'files.autoSave': 'afterDelay' }));
    await win.waitForTimeout(2500);
    report('auto save on: Saved, disabled, the tooltip says why', await save.isDisabled() && /Auto save is on/.test(await save.getAttribute('title')));
    await shot(win, 'save-autosave');
  } catch (error) {
    report('the test ran to its end', false, error instanceof Error ? error.stack.split('\n').slice(0, 3).join(' ') : String(error));
    await shot(win, 'import-failure');
  } finally {
    server.kill();
    await app.close();
    const failed = rows.filter((x) => !x.ok).length;
    console.log(`\n${rows.length - failed}/${rows.length} passed`);
    process.exit(failed ? 1 : 0);
  }
})();
