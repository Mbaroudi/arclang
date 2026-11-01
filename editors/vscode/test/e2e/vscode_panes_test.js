// The panes inside the real VS Code: the properties pane (a quantity, an
// enumeration and a reference edited, a refusal shown as it is typed, ⌘Z),
// an inherited value redefined and reset, the model tree (icons, findings,
// the filter, a node dragged onto the diagram) and a diagram opened in a
// tab of its own beside another, the selection kept the same in both.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_panes_test.js
// Keys are sent as a French (AZERTY) Mac keyboard sends them (azerty.js);
// SHOTS=/dir keeps pictures of the pane, the tree and the tabs.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(60)} ${detail || ''}`); };

async function launch(dir, model, extra = {}) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'arclang.workbench.doubleClick': 'rename', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'window.menuStyle': 'custom', 'workbench.colorTheme': process.env.THEME || 'Default Light Modern', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}), ...extra };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', dir, model], { width: 1680, height: 1050 });
  await win.waitForTimeout(4000);
  const press = await azerty(win);
  const command = async (name) => { await press('Meta+Shift+p'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(600); await press('Enter'); await win.waitForTimeout(600); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  const page = await workbenchPage(win);
  await win.waitForTimeout(1200);
  return { close, win, page, command, press };
}

/** The frame of a workbench page (not one of `not`), once its drawing is there. */
async function workbenchPage(win, not = []) {
  for (let i = 0; i < 60; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (!not.includes(f) && await f.$('.arcviz-stage svg')) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcpanes-'));
  const model = path.join(dir, 'model.arc');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const { close, win, page, command, press } = await launch(dir, model);
  const text = () => fs.readFileSync(model, 'utf8');
  const save = async () => { await press('Meta+s'); await win.waitForTimeout(900); };
  try {
    report('the workbench opens', !!page);
    if (!page) throw new Error('no workbench page');
    const offset = async (frame) => (await (await frame.frameElement()).boundingBox()) ?? { x: 0, y: 0 };
    const boxIn = async (frame, id) => { const b = await frame.locator(`.arcviz-stage [data-id="${id}"]`).first().boundingBox(); return b; };
    const clickBox = async (frame, id) => { const b = await boxIn(frame, id); await frame.locator('body').evaluate(() => undefined); await win.mouse.click(b.x + 24, b.y + 8); await win.waitForTimeout(900); };
    const row = (key, frame = page) => frame.locator(`.wb-properties tr[data-key="${key}"]`).first();
    const unfold = async (section) => { const s = page.locator(`.wb-sec[data-section="${section}"]`); if (await s.evaluate((n) => n.classList.contains('is-folded')).catch(() => false)) await s.locator('.wb-sec-head').click(); };
    // The drawer floats closed in a narrow editor: opened, as a reader would.
    if (await page.locator('.wb-right').isHidden()) await page.locator('.wb-toolrow button[title="Properties"]').click();
    await page.locator('.wb-layer', { hasText: 'System' }).first().click(); await win.waitForTimeout(1500);

    // 1. A quantity: refused as it is typed, applied on Enter, ⌘Z takes it back
    await clickBox(page, 'SF-DET');
    report('a click shows the element in the properties pane', (await page.locator('.wb-name-input').inputValue()) === 'Detect lane markings');
    const latency = row('latency').locator('input.wb-quantity-value');
    await latency.click(); await latency.fill('5 kg'); await win.waitForTimeout(500);
    const refusal = await row('latency').locator('.wb-field-msg').textContent();
    report('a mass typed for a time is refused as it is typed', /kg measures mass/.test(refusal), refusal);
    await press('Enter'); await win.waitForTimeout(1500); await save();
    report('… and nothing is written', /latency: 30 ms/.test(text()));
    // Values go in with fill (the text, whatever the keyboard layout); the keys are AZERTY's.
    await latency.fill('45'); await press('Enter'); await win.waitForTimeout(3500); await save();
    report('a quantity is applied on Enter as one edit', /"SF-DET"[\s\S]*?latency: 45 ms/.test(text()));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-pane-quantity.png') });
    await clickBox(page, 'SF-EST'); await clickBox(page, 'SF-DET');
    await press('Meta+z'); await win.waitForTimeout(3000); await save();
    report('⌘Z (AZERTY) on the canvas takes the edit back', /"SF-DET"[\s\S]*?latency: 30 ms/.test(text()));

    // 2. An enumeration and a reference
    await unfold('safety');
    await row('safety_level').locator('select').selectOption('ASIL-C'); await win.waitForTimeout(3500); await save();
    report('an enumeration is chosen from its values', /"SF-DET"[\s\S]*?safety_level: "ASIL-C"/.test(text()));
    const is = row('is').locator('input.wb-picker-input');
    await is.click(); await is.fill('Driver'); await win.waitForTimeout(800);
    const kindRefusal = await row('is').locator('.wb-field-msg').textContent();
    report('a reference to the wrong kind is refused as it is typed', /points at a type/.test(kindRefusal), kindRefusal);
    await press('Escape'); await win.waitForTimeout(400);
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await win.waitForTimeout(1500);
    await clickBox(page, 'LC-CAM'); await unfold('traces');
    await page.locator('.wb-sec[data-section="traces"] .wb-trace-kind').selectOption('satisfies');
    const traceTo = page.locator('.wb-trace-add input.wb-picker-input');
    await traceTo.click(); await traceTo.fill('REQ-LKA-002'); await win.waitForTimeout(500); await press('Enter'); await win.waitForTimeout(3500); await save();
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-trace.png') });
    report('a reference picked in the search is written (a trace)', /trace "LC-CAM" satisfies "REQ-LKA-002"/.test(text()));

    // 3. Redefine and reset an inherited value
    await page.locator('.wb-layer', { hasText: 'System' }).first().click(); await win.waitForTimeout(1500);
    await clickBox(page, 'SF-DET'); await unfold('inherited');
    report('the value of the type shows greyed, from the type', (await row('period').evaluate((r) => r.classList.contains('is-inherited'))) && /Periodic task/.test(await row('period').textContent()));
    await row('period').locator('button', { hasText: 'Redefine' }).click(); await win.waitForTimeout(400);
    const own = row('period').locator('input.wb-quantity-value'); await own.fill('10'); await press('Enter'); await win.waitForTimeout(3500); await save();
    report('Redefine writes the element\'s own value', /"SF-DET"[\s\S]*?period: 10 ms/.test(text()));
    await unfold('inherited');
    await row('period').locator('button[aria-label="Reset to inherited"]').click(); await win.waitForTimeout(3500); await save();
    const fn = /function "Detect lane markings" \{[^}]*\}/.exec(text())?.[0] ?? '';
    report('Reset to inherited takes it away', !/period/.test(fn) && (await row('period').evaluate((r) => r.classList.contains('is-inherited'))));
    if (SHOTS) { await unfold('inherited'); await win.screenshot({ path: path.join(SHOTS, 'vscode-pane-inherited.png') }); }

    // 4. The model tree: icons, the filter
    await command('ArcLang: Focus on Model View'); await win.waitForTimeout(1500);
    const tree = win.locator('.pane-body .monaco-list[aria-label*="Model"], [id="workbench.view.extension.arclang"] .monaco-list').first();
    const icons = await win.$$eval('.monaco-list-row .custom-view-tree-node-item-icon', (n) => n.map((x) => getComputedStyle(x).backgroundImage).filter((b) => /kinds\/[a-z_]+\.svg/.test(b)).map((b) => b.match(/kinds\/([a-z_]+)\.svg/)[1]));
    report('tree nodes carry the icon of their kind', new Set(icons).size >= 4, [...new Set(icons)].join(','));
    const layerRow = win.locator('.monaco-list-row', { hasText: 'Logical Architecture' }).first();
    report('layers list their diagrams', (await win.locator('.monaco-list-row', { hasText: 'Lane keeping chain' }).count()) > 0);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-tree.png') });
    await command('ArcLang: Filter the model tree'); await win.keyboard.type('camera'); await win.waitForTimeout(1500); await press('Enter'); await win.waitForTimeout(800);
    const shown = await win.locator('.monaco-list-row .label-name').allTextContents();
    report('the filter keeps the matching elements and the path to them', shown.includes('Camera') && shown.includes('Camera ECU') && !shown.includes('LKA Controller'), shown.slice(0, 8).join(', '));
    await command('ArcLang: Clear the filter of the model tree'); await win.waitForTimeout(1200);
    void tree; void layerRow;

    // 5. A diagram in a tab of its own, beside the workbench
    await win.locator('.monaco-list-row', { hasText: 'Lane keeping chain' }).first().click({ button: 'right' }); await win.waitForTimeout(600);
    const menuItem = win.locator('.context-view .action-item', { hasText: 'Open in a new tab' }).first();
    const menuShown = await menuItem.count();
    if (menuShown) await menuItem.click(); else await press('Escape');
    await win.waitForTimeout(4000);
    const second = await workbenchPage(win, [page]);
    const tabs = await win.locator('.tabs-container .tab').allTextContents();
    report('"Open in a new tab" opens the diagram in a tab titled model — diagram', menuShown > 0 && !!second && tabs.some((t) => t.includes('Lane Keeping Assist — Lane keeping chain')), tabs.join(' | '));
    if (!second) throw new Error('no second page');
    const pinned = await second.locator('.wb-pin-title').textContent().catch(() => '');
    report('the tab shows that diagram only', /Lane keeping chain/.test(pinned) && (await second.locator('.wb-layers').isHidden()));
    const firstFrame = await (await page.frameElement()).boundingBox(); const secondFrame = await (await second.frameElement()).boundingBox();
    report('the two diagrams sit side by side', Math.abs(firstFrame.x - secondFrame.x) > 200, `${Math.round(firstFrame.x)} / ${Math.round(secondFrame.x)}`);
    // 6. The selection syncs between the tabs and the text
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await win.waitForTimeout(1500);
    await clickBox(second, 'LC-ACT'); await win.waitForTimeout(1500);
    report('a selection in one tab is the selection of the other', (await page.locator('.wb-name-input').inputValue()) === 'Steering Actuator');
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-two-tabs.png') });

    // 7. A tree node dragged onto a declared diagram is added to what it shows
    // The filter brings the node into view (the tree draws only the rows on screen).
    await command('ArcLang: Filter the model tree'); await win.keyboard.type('ECU'); await win.waitForTimeout(1500); await press('Enter'); await win.waitForTimeout(800);
    const shows = (id) => new RegExp(`diagram "Lane keeping chain" \\{[^}]*shows: \\[[^\\]]*"${id}"`, 's').test(text());
    // 7a. The node's context menu: Show in the diagram (the page of the diagram in front)
    await clickBox(second, 'LC-CAM');
    await win.locator('.monaco-list-row', { hasText: 'Camera ECU' }).first().click({ button: 'right' }); await win.waitForTimeout(800);
    const show = win.locator('.context-view .action-item', { hasText: 'Show in the diagram' }).first();
    const hasShow = await show.count();
    if (hasShow) { await show.click(); await win.waitForTimeout(3500); await command('File: Save All'); } else await press('Escape');
    report('"Show in the diagram" on a tree node adds it to the declared diagram', hasShow > 0 && shows('PN-CAM'));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-shown.png') });
    // 7b. What VS Code hands the webview of a tree drag with Shift held: the item's resource
    // URI (its fragment encoded again) in its list of resources. The page reads the element from it.
    {
      await second.evaluate(() => {
        const sheet = document.querySelector('.wb-sheet');
        const data = new DataTransfer();
        data.setData('ResourceURLs', JSON.stringify(['arclang-node:/uuid?e%3D0%26w%3D0%26i%3D0%26own%3D0#arclang-element%3DPN-CHS']));
        data.setData('application/vnd.code.tree.arclang.model', JSON.stringify({ id: 'arclang.model', itemHandles: ['0/0:Chassis ECU'] }));
        sheet.dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: data }));
        sheet.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: data }));
      });
      await win.waitForTimeout(3500);
      await command('File: Save All'); await win.waitForTimeout(1000);
    }
    const said = await second.evaluate(() => [...document.querySelectorAll('.wb-toast-text')].map((n) => n.textContent).join(' | ')).catch(() => '');
    report('a tree drop as VS Code delivers it (Shift held) adds the node to the declared diagram', shows('PN-CHS'), shows('PN-CHS') ? '' : said + ' / ' + ((text().match(/diagram "Lane keeping chain" \{[^}]*\}/) || [''])[0]).replace(/\s+/g, ' '));
    // 7c. A drag from the tree onto the page, last (a drag VS Code keeps may hold the keys). VS Code decides whether a webview
    // receives a drag from a view: what happens is said, not counted.
    const from = await win.locator('.monaco-list-row', { hasText: 'Chassis ECU' }).first().boundingBox();
    const stage = await second.locator('.arcviz-stage').boundingBox();
    if (from && stage) {
      await win.mouse.move(from.x + 40, from.y + from.height / 2); await win.mouse.down();
      await win.keyboard.down('Shift');
      await win.mouse.move(stage.x + stage.width / 2, stage.y + stage.height / 2, { steps: 20 }); await win.waitForTimeout(400);
      await win.mouse.up(); await win.keyboard.up('Shift'); await win.waitForTimeout(3500);
    }
    const delivered = await second.evaluate(() => [...document.querySelectorAll('.wb-toast-text')].some((n) => /Chassis ECU is on this diagram/.test(n.textContent || ''))).catch(() => false);
    console.log(`note a tree node dragged onto the page (Shift held): ${delivered ? 'delivered to the webview' : 'not delivered to the webview by this synthetic drag'}`);
  } catch (error) {
    report('the run went through', false, String(error).slice(0, 200));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'vscode-panes-error.png') }).catch(() => undefined);
  } finally {
    await close();
  }
  const failed = rows.filter((r) => !r.ok).length;
  console.log(`\n${rows.length - failed}/${rows.length} checks passed`);
  process.exit(failed ? 1 : 0);
})();
