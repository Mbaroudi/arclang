// Navigation between the diagram and the text, and the New / Open entries, in
// the real VS Code. A single click on the diagram only selects (the editors, the
// groups, the cursor and the scroll of the text stay as they were; the text
// follows by a highlight when it is shown); a double-click goes to the source,
// opened beside the workbench; New model / layer / diagram and Open diagram.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_nav_test.js
// The state of the editors is read through the extension's test probe
// (ARCLANG_NAV_PROBE): visible editors, groups, selections, the highlight.
// Keys are AZERTY's (azerty.js); SHOTS=/dir keeps pictures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const { spawnSync } = require('child_process');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const ARCLANG = process.env.ARCLANG_BINARY || 'arclang';
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail || ''}`); };

const SETTINGS = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'window.menuStyle': 'custom', 'workbench.colorTheme': process.env.THEME || 'Default Light Modern', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };

async function launch(dir, files, extra = {}, open = true) {
  const settingsFile = path.join(dir, 'u', 'User', 'settings.json');
  fs.mkdirSync(path.dirname(settingsFile), { recursive: true });
  fs.writeFileSync(settingsFile, JSON.stringify({ ...SETTINGS, ...extra }));
  const probeFile = path.join(dir, 'probe.json');
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), env: { ...process.env, ARCLANG_NAV_PROBE: probeFile }, args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', dir, ...files] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1680, height: 1050 }));
  await win.waitForTimeout(4000);
  const press = await azerty(win);
  const command = async (name, settle = 700) => { await press('Meta+Shift+p'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(700); await press('Enter'); await win.waitForTimeout(settle); };
  const setSettings = async (more) => { fs.writeFileSync(settingsFile, JSON.stringify({ ...SETTINGS, ...extra, ...more })); await win.waitForTimeout(2000); };
  const probe = async () => { fs.rmSync(probeFile, { force: true }); await command('ArcLang: Dump the editor layout (test)', 500); for (let i = 0; i < 20 && !fs.existsSync(probeFile); i++) await win.waitForTimeout(150); return JSON.parse(fs.readFileSync(probeFile, 'utf8')); };
  await command('View: Close Secondary Side Bar');
  if (open) await command('ArcLang: Open the workbench', 1500);
  return { app, win, command, press, probe, setSettings };
}

async function workbenchPage(win, not = []) {
  for (let i = 0; i < 60; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (!not.includes(f) && await f.$('.arcviz-stage svg')) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const textEditors = (p) => p.visibleEditors.filter((e) => e.file.endsWith('.arc'));
const tabsOf = (p) => p.groups.flatMap((g) => g.tabs.map((t) => `${g.column}:${t.kind}:${t.label}`));
const groupWith = (p, kind) => p.groups.find((g) => g.tabs.some((t) => t.kind === kind));
/** The line of the declaration that holds `id: "<id>"`: the closest line above it that opens a block. */
const declOf = (text, id) => { const lines = text.split('\n'); let at = lines.findIndex((l) => l.includes(`id: "${id}"`)); while (at > 0 && !/\{\s*$/.test(lines[at])) at--; return at; };
const lineOf = (text, needle) => text.split('\n').findIndex((l) => l.includes(needle));

async function navigation() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcnav-'));
  const model = path.join(dir, 'model.arc');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const source = fs.readFileSync(model, 'utf8');
  const { app, win, command, press, probe, setSettings } = await launch(dir, [model]);
  try {
    const page = await workbenchPage(win);
    report('the workbench opens beside the text', !!page);
    if (!page) throw new Error('no workbench page');
    let p = await probe();
    report('… in a second editor group, the text kept in the first', textEditors(p).length === 1 && p.groups.length === 2 && groupWith(p, 'text')?.column === 1 && groupWith(p, 'workbench')?.column === 2, tabsOf(p).join(' | '));
    const boxOf = async (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first().boundingBox();
    const click = async (id, n = 1) => { const b = await boxOf(id); if (n === 2) await win.mouse.dblclick(b.x + 24, b.y + 8); else await win.mouse.click(b.x + 24, b.y + 8); await win.waitForTimeout(1200); };
    await page.locator('.wb-layer', { hasText: 'System' }).first().click(); await win.waitForTimeout(1500);
    if (await page.locator('.wb-right').isHidden()) await page.locator('.wb-toolrow button[title="Properties"]').click();

    // 1. A single click: the selection only
    await click('SF-EST'); // the workbench has the focus now
    const before = await probe();
    await click('SF-DET');
    const after = await probe();
    report('a single click: the properties pane follows', (await page.locator('.wb-name-input').inputValue()) === 'Detect lane markings');
    report('a single click: same editors, columns and selection in the text', same(before.visibleEditors, after.visibleEditors), JSON.stringify(after.visibleEditors.map((e) => [e.column, e.selection, e.visibleTop])));
    report('a single click: same groups and tabs, same active editor', same(before.groups, after.groups) && before.activeEditor === after.activeEditor, tabsOf(after).join(' | '));
    const declaration = lineOf(source, 'function "Detect lane markings"');
    report('followSelection highlight: the declaration is decorated in the text', !!after.highlight && after.highlight.startLine === declaration, JSON.stringify(after.highlight) + ' want line ' + declaration);
    report('… and the cursor of the text editor is not on it', !same(after.visibleEditors[0].selection.slice(0, 2), [declaration, 0]) || before.visibleEditors[0].selection[0] === declaration);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-1-click-highlight.png') });
    await page.locator('.arcviz-stage').click({ position: { x: 6, y: 6 } }); await win.waitForTimeout(800);
    p = await probe();
    report('the highlight goes with the selection when it is cleared', p.highlight === null, JSON.stringify(p.highlight));

    // 2. followSelection: off
    await setSettings({ 'arclang.workbench.followSelection': 'off' });
    const offBefore = await probe();
    await click('SF-DET');
    const off = await probe();
    report('followSelection off: nothing happens in the text', off.highlight === null && same(offBefore.visibleEditors, off.visibleEditors) && same(offBefore.groups, off.groups));
    await setSettings({});

    // 3. A single click with the text closed: nothing is opened
    await command('View: Close Editors in Other Groups'); await win.waitForTimeout(800);
    await page.locator('.arcviz-stage').click({ position: { x: 6, y: 6 } }); await win.waitForTimeout(500);
    const alone = await probe();
    report('the workbench alone: no text editor is visible', textEditors(alone).length === 0, tabsOf(alone).join(' | '));
    await click('SF-EST');
    const aloneAfter = await probe();
    report('… a single click opens nothing', textEditors(aloneAfter).length === 0 && same(alone.groups.map((g) => g.tabs), aloneAfter.groups.map((g) => g.tabs)), tabsOf(aloneAfter).join(' | '));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-2-workbench-alone.png') });

    // 4. A double-click goes to the source, beside the workbench
    await click('SF-DET', 2);
    const dbl = await probe();
    const text = textEditors(dbl)[0];
    report('a double-click opens the text', !!text, tabsOf(dbl).join(' | '));
    report('… in a group of its own, beside the workbench', !!text && groupWith(dbl, 'text') && groupWith(dbl, 'workbench') && groupWith(dbl, 'text') !== groupWith(dbl, 'workbench'), tabsOf(dbl).join(' | '));
    report('… the workbench tab is still there, active in its group', !!groupWith(dbl, 'workbench')?.tabs.find((t) => t.kind === 'workbench' && t.active) && groupWith(dbl, 'workbench').tabs.length === 1);
    report('… the declaration is selected in the text, which has the focus', !!text && text.selection[0] === declaration && dbl.activeEditor === model, JSON.stringify(text && text.selection));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-3-double-click-beside.png') });

    // 5. The text is shown: reused, focused, selected there
    await click('SF-EST'); // back to the workbench
    await click('SF-EST', 2);
    const reuse = await probe();
    const est = declOf(source, 'SF-EST');
    report('a double-click reuses the text editor shown', textEditors(reuse).length === 1 && reuse.groups.length === dbl.groups.length && reuse.groups.every((g) => g.tabs.length === 1), tabsOf(reuse).join(' | '));
    report('… and focuses it, the declaration selected', reuse.activeEditor === model && textEditors(reuse)[0].selection[0] === est, JSON.stringify(textEditors(reuse)[0].selection) + ' want ' + est);

    // 6. doubleClick: rename keeps draw.io's gesture
    await setSettings({ 'arclang.workbench.doubleClick': 'rename' });
    await click('SF-EST'); await click('SF-DET', 2);
    const rename = await page.locator('.wb-direct-edit').count();
    report('doubleClick rename: a double-click renames in place', rename === 1);
    await press('Escape'); await win.waitForTimeout(500);
    await setSettings({});
    await click('SF-DET', 2);
    report('… and back to source: no rename box', (await page.locator('.wb-direct-edit').count()) === 0);

    // 7. The Source button and Go to source with the workbench alone
    await page.locator('.arcviz-stage').click({ position: { x: 6, y: 6 } }); await win.waitForTimeout(500);
    await command('View: Close Editors in Other Groups'); await win.waitForTimeout(800);
    const gone = await probe();
    report('the text is closed again', textEditors(gone).length === 0);
    const notes = await win.locator('.notification-list-item').allTextContents(); if (notes.length) console.log('     notifications:', notes.map((n) => n.replace(/\s+/g, ' ').slice(0, 160)).join(' || '));
    await command('Notifications: Clear All Notifications', 400);
    await page.locator('.wb-problems-head button', { hasText: 'Source' }).click(); await win.waitForTimeout(1500);
    const src = await probe();
    report('the Source button opens the text in a second group, both visible', textEditors(src).length === 1 && src.groups.length >= 2 && groupWith(src, 'text') && groupWith(src, 'workbench') && groupWith(src, 'text') !== groupWith(src, 'workbench'), tabsOf(src).join(' | '));
    await page.locator('.arcviz-stage').click({ position: { x: 6, y: 6 } }); await win.waitForTimeout(500);
    await command('View: Close Editors in Other Groups'); await win.waitForTimeout(800);
    await click('SF-DET');
    const unfold = page.locator('.wb-sec[data-section="source"], .wb-link', { hasText: 'Go to source' }).first();
    const link = page.getByText('Go to source').first();
    if (await link.count()) { await link.click(); await win.waitForTimeout(1500); }
    const goto = await probe();
    report("the properties' Go to source opens the text beside, selected", (await link.count()) > 0 && textEditors(goto).length === 1 && textEditors(goto)[0].selection[0] === declaration, JSON.stringify(textEditors(goto)[0] && textEditors(goto)[0].selection));
    if (!(await link.count()) || textEditors(goto).length !== 1) { console.log('     debug', await link.count(), tabsOf(goto).join(' | '), JSON.stringify(goto.visibleEditors)); if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-debug-goto.png') }); }
    void unfold;
  } finally {
    await app.close().catch(() => undefined);
  }
}

async function creation() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcnew-'));
  fs.mkdirSync(path.join(dir, 'models'));
  const { app, win, command, press, probe } = await launch(dir, [], {}, false);
  const check = (file) => spawnSync(ARCLANG, ['check', file], { encoding: 'utf8' });
  const saveAs = async (file) => { const input = win.locator('.quick-input-widget input').first(); await input.waitFor({ timeout: 8000 }); await input.fill(file); await win.waitForTimeout(500); await press('Enter'); await win.waitForTimeout(2500); };
  const pickItem = async (label) => { await win.waitForTimeout(600); await win.locator('.quick-input-list .monaco-list-row', { hasText: label }).first().click(); await win.waitForTimeout(1200); };
  try {
    // New model: blank
    await command('ArcLang: New model…', 900);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-4-new-model.png') });
    await pickItem('Blank model');
    const blank = path.join(dir, 'models', 'blank.arc');
    await saveAs(blank);
    const page = await workbenchPage(win);
    const p1 = await probe();
    report('New model, blank: the file is written and compiles', fs.existsSync(blank) && check(blank).status === 0, check(blank).stdout.slice(-80));
    report('… the text and the workbench open side by side', textEditors(p1).length === 1 && p1.groups.length >= 2 && p1.groups.every((g) => g.tabs.length >= 1), tabsOf(p1).join(' | '));
    void page;

    // New model: Arcadia skeleton
    await command('ArcLang: New model…', 900);
    await pickItem('Arcadia skeleton');
    const skeleton = path.join(dir, 'models', 'skeleton.arc');
    await saveAs(skeleton);
    await win.waitForTimeout(3000); if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-5a-skeleton-open.png') });
    for (const f of win.frames()) { const t = await f.evaluate(() => document.body && document.body.innerText.slice(0, 200)).catch(() => ''); if (/Operational|workbench|No |empty/i.test(t)) console.log('     frame:', JSON.stringify(t)); }
    const wb2 = await workbenchPage(win, [page]);
    report('New model, Arcadia skeleton: compiles, holds the four levels', fs.existsSync(skeleton) && check(skeleton).status === 0 && ['operational_analysis', 'system_analysis', 'logical_architecture', 'physical_architecture'].every((k) => fs.readFileSync(skeleton, 'utf8').includes(k)), check(skeleton).stdout.slice(-80));
    const p2 = await probe();
    report('… opened beside the text', !!wb2 && textEditors(p2).some((e) => path.basename(e.file) === 'skeleton.arc') && p2.groups.length >= 2, tabsOf(p2).join(' | '));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-5-skeleton.png') });

    // New layer on the skeleton
    await wb2.locator('.arcviz-stage').click({ position: { x: 10, y: 10 } }).catch(() => undefined);
    await command('ArcLang: New layer…', 900);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-6-new-layer.png') });
    const offered = await win.locator('.quick-input-list .monaco-list-row').allTextContents();
    report('New layer offers only what the model lacks', offered.length > 0 && !offered.some((t) => /Logical/.test(t)) && offered.some((t) => /EPBS/.test(t)) && offered.some((t) => /Requirements/.test(t)), offered.map((t) => t.slice(0, 14)).join(' | '));
    await pickItem('Requirements');
    await win.waitForTimeout(2500);
    const docText = async () => { const pr = await probe(); return pr.text.find((t) => path.basename(t.file) === 'skeleton.arc'); };
    const withLayer = await docText();
    report('New layer Requirements adds its block to the document', withLayer.lines > 17, `${withLayer.lines} lines`);
    await command('ArcLang: New layer…', 900);
    await pickItem('EPBS');
    await win.waitForTimeout(2500);
    const withEpbs = await docText();
    report('New layer EPBS adds a block (the document grows)', withEpbs.lines > withLayer.lines, `${withLayer.lines} -> ${withEpbs.lines}`);
    await wb2.locator('.arcviz-stage').click({ position: { x: 10, y: 10 } }).catch(() => undefined);
    await press('Meta+z'); await win.waitForTimeout(2500);
    const undone = await docText();
    report('⌘Z removes the layer just added (one undo)', undone.lines === withLayer.lines, `${withEpbs.lines} -> ${undone.lines}`);

    // New model from an example, then a new diagram and the list of diagrams by layer
    await command('ArcLang: New model…', 900);
    await pickItem('From an example');
    await pickItem('Lane Keeping Assist');
    const lane = path.join(dir, 'models', 'lane_keeping.arc');
    await saveAs(lane);
    const wb3 = await workbenchPage(win, [page, wb2]);
    report('New model from an example: the example compiles and opens', fs.existsSync(lane) && check(lane).status === 0 && !!wb3, '');
    await wb3.locator('.arcviz-stage').click({ position: { x: 10, y: 10 } }).catch(() => undefined);
    await command('ArcLang: New diagram…', 900);
    await pickItem('Logical architecture');
    await win.waitForTimeout(600);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-7a-new-diagram-elements.png') });
    if (await win.locator('.quick-input-widget .quick-input-check-all, .quick-input-widget .quick-input-count').count()) { await win.locator('.quick-input-widget').getByText('OK', { exact: true }).click(); await win.waitForTimeout(900); }
    const name = win.locator('.quick-input-widget input').first();
    await name.waitFor({ timeout: 5000 });
    await name.fill('Brand new'); await press('Enter'); await win.waitForTimeout(4000);
    const p3 = await probe();
    report('New diagram opens a tab of its own, named by the diagram', p3.groups.some((g) => g.tabs.some((t) => t.kind === 'diagram' && /Brand new/.test(t.label))), tabsOf(p3).join(' | '));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-7-new-diagram.png') });
    await command('Notifications: Clear All Notifications', 300);
    await command('ArcLang: Open a diagram…', 1000);
    const list = (await win.locator('.quick-input-list .monaco-list-row').allTextContents()).map((t) => t.replace(/\s+/g, ' '));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'nav-8-open-diagram.png') });
    report('Open diagram lists by layer, declared and automatic', list.some((t) => /Logical architecture/.test(t)) && list.some((t) => /Brand new.*declared/.test(t)) && list.some((t) => /Logical Architecture.*automatic/i.test(t)), list.slice(0, 8).join(' | '));
    const buttons = await win.locator('.quick-input-list .monaco-list-row .monaco-action-bar .action-label').count();
    report('… each with an "Open in new tab" button', buttons >= 3, String(buttons));
    await win.locator('.quick-input-list .monaco-list-row', { hasText: 'Lane keeping chain' }).first().click(); await win.waitForTimeout(2000);
    const active = await wb3.locator('.wb-layer.is-active').first().textContent().catch(() => '');
    report('… picking one shows it in the workbench', /Logical/.test(active || ''), active);
    await command('ArcLang: Open a diagram…', 1000);
    await win.keyboard.type('Lane keeping chain'); await win.waitForTimeout(700);
    await win.locator('.quick-input-list .monaco-list-row', { hasText: 'Lane keeping chain' }).first().hover(); await win.waitForTimeout(300);
    await win.locator('.quick-input-list .monaco-list-row', { hasText: 'Lane keeping chain' }).first().locator('.action-label').first().click(); await win.waitForTimeout(3000);
    const p4 = await probe();
    report('… the button opens it in a tab of its own', p4.groups.some((g) => g.tabs.some((t) => t.kind === 'diagram' && /Lane keeping chain/.test(t.label))), tabsOf(p4).join(' | '));
  } finally {
    await app.close().catch(() => undefined);
  }
}

(async () => {
  await navigation();
  await creation();
  const failed = rows.filter((r) => !r.ok);
  console.log(`\n${rows.length - failed.length}/${rows.length} passed`);
  process.exit(failed.length ? 1 : 0);
})().catch((error) => { console.error(error); process.exit(1); });
