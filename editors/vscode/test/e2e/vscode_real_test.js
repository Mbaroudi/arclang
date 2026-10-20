// The workbench inside the real VS Code (not a browser imitation of it):
// what the user sees and does. Needs Playwright and a VS Code application:
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright node vscode_real_test.js
// The compiler is the `arclang` on the PATH, or ARCLANG_BINARY.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(56)} ${detail || ''}`); };

async function launch(dir, model) {
  const settings = { 'files.simpleDialog.enable': true, 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(500); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
  await win.waitForTimeout(1200);
  return { app, win, page };
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcvs-'));
  const model = path.join(dir, 'model.arc'); const layout = path.join(dir, 'model.layout.json');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  let { app, win, page } = await launch(dir, model);
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    const box = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first().boundingBox();
    const tool = (label) => page.locator('.wb-viewer-tools button', { hasText: label }).first();
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await win.waitForTimeout(1500);

    // 1. Nothing hidden that should show, nothing shown that should hide
    const chrome = await page.evaluate(() => {
      const row = document.querySelector('.wb-toolrow').getBoundingClientRect();
      const cut = [...document.querySelectorAll('.wb-viewer-tools button')].filter((b) => { const r = b.getBoundingClientRect(); return r.width > 0 && (r.top < row.top - 1 || r.bottom > row.bottom + 1); }).map((b) => b.textContent.trim());
      return { cut, findShown: getComputedStyle(document.querySelector('.wb-find')).display !== 'none' };
    });
    report('every toolbar button is fully visible', chrome.cut.length === 0, chrome.cut.join(', '));
    report('the find box is closed until asked for', !chrome.findShown);

    // 2. Free placement: a box goes where it is let go, kept next to the model
    const before = await box('LC-ACT');
    await win.mouse.move(before.x + 15, before.y + 10); await win.mouse.down(); await win.mouse.move(before.x + 75, before.y + 150, { steps: 12 }); await win.mouse.up();
    await win.waitForTimeout(2500);
    const after = await box('LC-ACT');
    report('a dragged box stays where it is let go', Math.abs(after.x - before.x - 60) < 12 && Math.abs(after.y - before.y - 140) < 12, `${Math.round(after.x - before.x)},${Math.round(after.y - before.y)}`);
    const kept = fs.existsSync(layout) && /"LC-ACT"/.test(fs.readFileSync(layout, 'utf8'));
    report('the arrangement is written to model.layout.json', kept);

    // 2b. Resize: the grip on the corner of the selected box
    const shape = () => page.locator('.arcviz-stage [data-id="LC-CAM"] > .av-shape').first().boundingBox();
    const small = await shape(); await win.mouse.click(small.x + 20, small.y + 8); await win.waitForTimeout(800);
    const grip = page.locator('.arcviz-grip');
    const gripShown = await grip.isVisible().catch(() => false);
    if (gripShown) { const g = await grip.boundingBox(); await win.mouse.move(g.x + g.width / 2, g.y + g.height / 2); await win.mouse.down(); await win.mouse.move(g.x + 90, g.y + 50, { steps: 10 }); await win.mouse.up(); await win.waitForTimeout(2500); }
    const big = await shape();
    report('the grip of the selected box resizes it', gripShown && big.width > small.width * 1.5 && big.height > small.height * 1.3, `${Math.round(small.width)}x${Math.round(small.height)} -> ${Math.round(big.width)}x${Math.round(big.height)}`);
    report('the size is written to model.layout.json', /"LC-CAM":\s*\{[^}]*"dw"/.test(fs.readFileSync(layout, 'utf8')));
    report('no Save layout button: the arrangement saves itself', !(await page.locator('.wb-viewer-tools button', { hasText: 'Save layout' }).count()));
    await win.keyboard.press('Escape');

    // 3. Exports through the save dialog
    for (const [label, file, magic] of [['Save PNG', 'out.png', '\x89PNG'], ['Save SVG', 'out.svg', '<svg']]) {
      await tool(label).click(); await win.waitForTimeout(1200);
      const input = win.locator('.quick-input-widget input').first();
      if (await input.isVisible().catch(() => false)) { await input.fill(path.join(dir, file)); await input.press('Enter'); await win.waitForTimeout(1800); }
      const written = fs.existsSync(path.join(dir, file)) && fs.readFileSync(path.join(dir, file)).subarray(0, 4).toString('latin1').startsWith(magic.slice(0, 4));
      report(`${label} writes the file chosen in the save dialog`, written);
      await win.keyboard.press('Escape');
    }

    // 4. Full screen with every tool, Esc gives the window back
    const narrow = await page.evaluate(() => innerWidth);
    await tool('Full screen').click(); await win.waitForTimeout(2500);
    const full = await page.evaluate(() => ({ w: innerWidth, palette: getComputedStyle(document.querySelector('.wb-left')).display, tools: getComputedStyle(document.querySelector('.wb-toolrow')).display }));
    report('full screen gives the workbench the window, tools kept', full.w > narrow * 1.5 && full.palette !== 'none' && full.tools !== 'none', `${narrow} -> ${full.w}`);
    await page.locator('.arcviz-stage').click({ position: { x: 4, y: 4 } }); await win.keyboard.press('Escape'); await win.waitForTimeout(2000);
    report('Esc leaves full screen', (await page.evaluate(() => innerWidth)) < full.w * 0.8);

    // 5. Reopened: the arrangement is back
    await app.close();
    // VS Code lets go of its profile a moment after its window closes.
    for (let i = 0; i < 40 && fs.existsSync(path.join(dir, 'u', 'code.lock')); i++) await new Promise((r) => setTimeout(r, 250));
    await new Promise((r) => setTimeout(r, 1500));
    ({ app, win, page } = await launch(dir, model));
    if (!page) { await win.screenshot({ path: path.join(os.tmpdir(), 'arcvs-reopen.png') }); throw new Error('the workbench did not open again'); }
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await win.waitForTimeout(1500);
    const camera = await box('LC-CAM'); const actuator = await box('LC-ACT');
    report('a reopened model shows the boxes where they were left', actuator.y - camera.y > 120, `${Math.round(actuator.y - camera.y)}`);
    const reopened = await shape();
    report('a reopened model shows the boxes at the size they were left', Math.abs(reopened.width / reopened.height - big.width / big.height) < 0.2, `${Math.round(reopened.width)}x${Math.round(reopened.height)}`);

    // 6. A box let go on another goes into it (the text changes)
    const from = await box('LC-ACT'); const to = await box('LC-CTL');
    await win.mouse.move(from.x + 15, from.y + 10); await win.mouse.down(); await win.mouse.move(to.x + to.width / 2, to.y + 12, { steps: 15 });
    const marked = await page.evaluate(() => [...document.querySelectorAll('.is-drop-target')].map((n) => n.dataset.id));
    await win.mouse.up(); await win.waitForTimeout(3500);
    report('the box it would go into is marked during the drag', marked.includes('LC-CTL'), marked.join(','));
    await win.keyboard.press('Meta+s'); await win.waitForTimeout(800);
    const text = fs.readFileSync(model, 'utf8');
    report('let go on a component, it is moved into it', /component "LKA Controller" \{[\s\S]*?\n    component "Steering Actuator" \{\n      id: "LC-ACT"/.test(text));

    // 7. Reconnect: the end of the selected exchange dragged onto another box
    const shots = process.env.SHOTS || null;
    const middle = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first().evaluate((g) => { const line = g.querySelector('.av-line'); const len = line.getTotalLength(); const p = line.getPointAtLength(len / 2); const m = line.getScreenCTM(); return { x: p.x * m.a + p.y * m.c + m.e, y: p.x * m.b + p.y * m.d + m.f }; });
    // Frame coordinates of the webview are window coordinates offset by the frame's position.
    const offset = await (await page.frameElement()).boundingBox().catch(() => ({ x: 0, y: 0 }));
    const inWindow = (p) => ({ x: p.x + (offset?.x ?? 0), y: p.y + (offset?.y ?? 0) });
    const frameBox = async (locator) => { const b = await locator.boundingBox(); return b; };
    let m = inWindow(await middle('ce:LaneFlow')); await win.mouse.click(m.x, m.y); await win.waitForTimeout(800);
    const end = page.locator('.arcviz-end[data-end="target"]');
    const endShown = await end.isVisible().catch(() => false);
    if (endShown) {
      const e = await frameBox(end); const actuator = await box('LC-ACT');
      await win.mouse.move(e.x + e.width / 2, e.y + e.height / 2); await win.mouse.down();
      await win.mouse.move(actuator.x + actuator.width / 2, actuator.y + 12, { steps: 15 });
      if (shots) await win.screenshot({ path: path.join(shots, 'vscode-reconnect.png') });
      await win.mouse.up(); await win.waitForTimeout(3500);
    }
    await win.keyboard.press('Meta+s'); await win.waitForTimeout(800);
    report('the end of an exchange dragged onto a box reconnects it', endShown && /to_port: "LC-ACT\.lane"/.test(fs.readFileSync(model, 'utf8')), endShown ? '' : 'no end handle');

    // 8. The context menu of a box
    const cam = await box('LC-CAM');
    await win.mouse.click(cam.x + cam.width / 2, cam.y + 10, { button: 'right' }); await win.waitForTimeout(600);
    const items = await page.locator('.wb-menu:not([hidden]) .wb-menu-item').allTextContents();
    if (shots) await win.screenshot({ path: path.join(shots, 'vscode-context-menu.png') });
    report('right-click on a box opens its menu, not VS Code\'s', ['Rename', 'Move to', 'Duplicate', 'Delete'].every((w) => items.some((t) => t.includes(w))), items.length + ' items');
    await win.keyboard.press('Escape'); await win.waitForTimeout(300);

    // 9. Align: two boxes selected (click, Shift+click), Align top in the menu, kept in the layout file
    const ctl = await box('LC-CTL');
    await win.mouse.click(cam.x + cam.width / 2, cam.y + 10); await win.waitForTimeout(400);
    await win.keyboard.down('Shift'); await win.mouse.click(ctl.x + ctl.width / 2, ctl.y + 10); await win.keyboard.up('Shift'); await win.waitForTimeout(500);
    await win.mouse.click(ctl.x + ctl.width / 2, ctl.y + 10, { button: 'right' }); await win.waitForTimeout(600);
    const top = page.locator('.wb-menu:not([hidden]) .wb-menu-chip', { hasText: 'Bottom' }).first();
    const hasAlign = await top.count();
    if (hasAlign) { await top.click(); await win.waitForTimeout(2500); }
    if (shots) await win.screenshot({ path: path.join(shots, 'vscode-align.png') });
    const [a1, a2] = [await page.locator('.arcviz-stage [data-id="LC-CAM"] > .av-shape').first().boundingBox(), await page.locator('.arcviz-stage [data-id="LC-CTL"] > .av-shape').first().boundingBox()];
    report('Align bottom in the menu lines two boxes up, in the layout file', hasAlign > 0 && Math.abs(a1.y + a1.height - a2.y - a2.height) < 2 && /"LC-CAM"|"LC-CTL"/.test(fs.readFileSync(layout, 'utf8')), `${Math.round(a1.y + a1.height)} / ${Math.round(a2.y + a2.height)}`);

    // 10. Two gestures in quick succession both land: ⌘D twice without waiting
    await win.keyboard.press('Escape'); const cam2 = await box('LC-CAM');
    await win.mouse.click(cam2.x + cam2.width / 2, cam2.y + 10); await win.waitForTimeout(500);
    await win.keyboard.press('Meta+d'); await win.keyboard.press('Meta+d'); await win.waitForTimeout(6000);
    await win.keyboard.press('Meta+s'); await win.waitForTimeout(800);
    const twice = fs.readFileSync(model, 'utf8');
    report('⌘D twice in a row makes two copies, neither lost', /id: "LC-CAM-2"/.test(twice) && /id: "LC-CAM-3"/.test(twice), `${(twice.match(/LC-CAM-\d/g) || []).join(',')}`);
  } catch (error) {
    report('no error', false, String(error));
  } finally {
    await app.close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
  const failed = rows.filter((r) => !r.ok); console.log(`\n${rows.length - failed.length}/${rows.length} checks in VS Code`);
  process.exit(failed.length ? 1 : 0);
})();
