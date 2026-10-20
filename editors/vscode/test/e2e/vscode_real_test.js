// The workbench inside the real VS Code (not a browser imitation of it):
// what the user sees and does. Needs Playwright and a VS Code application:
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright node vscode_real_test.js
// The compiler is the `arclang` on the PATH, or ARCLANG_BINARY. The keys of
// the draw.io checks are typed as on a French (AZERTY) Mac keyboard (azerty.js);
// SHOTS=/dir keeps pictures of the themes, the narrow panel and the gestures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const { azerty } = require('./azerty');
const EXT = path.resolve(__dirname, '../..');
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(56)} ${detail || ''}`); };

async function launch(dir, models, extra = {}) {
  const files = [].concat(models);
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}), ...extra };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', ...files] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(500); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  const page = await workbenchPage(win);
  await win.waitForTimeout(1200);
  return { app, win, page, command };
}

/** The frame of a workbench page (not one of `not`), once its drawing is there. */
async function workbenchPage(win, not = []) {
  for (let i = 0; i < 60; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (!not.includes(f) && await f.$('.arcviz-stage svg')) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

/** Placing, the arrangement kept, exports, full screen, drop, reconnect, menus, align. */
async function basics() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcvs-'));
  const model = path.join(dir, 'model.arc'); const layout = path.join(dir, 'model.layout.json');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  let { app, win, page } = await launch(dir, model);
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    const box = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first().boundingBox();
    const tool = (id) => page.locator(`#${id}`);
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await win.waitForTimeout(1500);

    // 1. Nothing hidden that should show, nothing shown that should hide
    const chrome = await page.evaluate(() => {
      const row = document.querySelector('.wb-toolrow').getBoundingClientRect();
      const cut = [...document.querySelectorAll('.wb-toolrow button')].filter((b) => { const r = b.getBoundingClientRect(); return r.width > 0 && (r.top < row.top - 1 || r.bottom > row.bottom + 1 || r.left < row.left - 1 || r.right > row.right + 1); }).map((b) => b.id || b.getAttribute('aria-label'));
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
    await tool('arcviz-export-menu').click(); await win.waitForTimeout(300);
    report('no Save layout in Export: the arrangement saves itself', !(await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="save-layout"]').count()));
    await win.keyboard.press('Escape');
    await win.keyboard.press('Escape');

    // 3. Exports through the save dialog
    for (const [label, file, magic] of [['save-png', 'out.png', '\x89PNG'], ['save-svg', 'out.svg', '<svg']]) {
      await tool('arcviz-export-menu').click(); await win.waitForTimeout(300);
      await page.locator(`.arcviz-menu-pop:not([hidden]) [data-action="${label}"]`).click(); await win.waitForTimeout(1200);
      const input = win.locator('.quick-input-widget input').first();
      if (await input.isVisible().catch(() => false)) { await input.fill(path.join(dir, file)); await input.press('Enter'); await win.waitForTimeout(1800); }
      const written = fs.existsSync(path.join(dir, file)) && fs.readFileSync(path.join(dir, file)).subarray(0, 4).toString('latin1').startsWith(magic.slice(0, 4));
      report(`${label} writes the file chosen in the save dialog`, written);
      await win.keyboard.press('Escape');
    }

    // 4. Full screen with every tool, Esc gives the window back
    const narrow = await page.evaluate(() => innerWidth);
    await tool('arcviz-fullscreen').click(); await win.waitForTimeout(2500);
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
    // The align stacked the two boxes: ⌘Z (typed as on an AZERTY Mac) puts them back first.
    await win.keyboard.press('Escape'); await (await azerty(win))('Meta+z'); await win.waitForTimeout(3000); const cam2 = await box('LC-CAM');
    await win.mouse.click(cam2.x + cam2.width / 2, cam2.y + 10); await win.waitForTimeout(500);
    await win.keyboard.press('Meta+d'); await win.keyboard.press('Meta+d'); await win.waitForTimeout(6000);
    await win.keyboard.press('Meta+s'); await win.waitForTimeout(800);
    const twice = fs.readFileSync(model, 'utf8');
    report('⌘D twice in a row makes two copies, neither lost', /id: "LC-CAM-2"/.test(twice) && /id: "LC-CAM-3"/.test(twice), `${(twice.match(/LC-CAM-\d/g) || []).join(',')} ${await page.evaluate(() => `${document.activeElement?.tagName}.${document.activeElement?.className} | ${document.querySelector('.wb-toast')?.textContent} | ${document.querySelector('.wb-multibar')?.hidden}`)}`);
  } catch (error) {
    report('no error', false, String(error));
  } finally {
    await app.close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

/** Helpers on one workbench page of a launched VS Code, keys typed as on an AZERTY Mac. */
function on(ctx) {
  const { win } = ctx;
  const h = {
    shape: (id) => ctx.page.locator(`.arcviz-stage [data-id="${id}"] > .av-shape`).first().boundingBox({ timeout: 4000 }).catch(() => null),
    key: async (k, wait = 2500) => { await ctx.press(k); await win.waitForTimeout(wait); },
    save: async () => { await ctx.press('Meta+s'); await win.waitForTimeout(900); },
    stage: () => ctx.page.locator('.arcviz-stage').boundingBox(),
    blank: async () => { const s = await h.stage(); return { x: s.x + 24, y: s.y + s.height - 24 }; },
    deselect: async () => { const b = await h.blank(); await win.mouse.click(b.x, b.y); await win.waitForTimeout(400); },
    click: async (id, options = {}) => { const b = await h.shape(id); if (!b) return null; await win.mouse.click(b.x + Math.min(30, b.width / 2), b.y + Math.min(9, b.height / 2), options); await win.waitForTimeout(500); return b; },
    shiftClick: async (id) => { await win.keyboard.down('Shift'); const b = await h.click(id); await win.keyboard.up('Shift'); await win.waitForTimeout(300); return b; },
    drag: async (from, to, wait = 2500) => { await win.mouse.move(from.x, from.y); await win.mouse.down(); await win.mouse.move(to.x, to.y, { steps: 15 }); await win.mouse.up(); await win.waitForTimeout(wait); },
    scale: () => ctx.page.evaluate(() => Number(/matrix\(([\d.]+)/.exec(getComputedStyle(document.querySelector('.arcviz-canvas')).transform)?.[1] || 1)),
    menu: async (x, y, label) => {
      await win.mouse.click(x, y, { button: 'right' }); await win.waitForTimeout(600);
      const item = ctx.page.locator('.wb-menu:not([hidden]) .wb-menu-item, .wb-menu:not([hidden]) .wb-menu-chip', { hasText: label }).first();
      if (!(await item.count())) { await ctx.press('Escape'); return false; }
      await item.click(); await win.waitForTimeout(3500); return true;
    },
    layer: async (name) => { await ctx.page.locator('.wb-layer', { hasText: name }).first().click(); await win.waitForTimeout(1800); },
    middle: async (id) => {
      const o = (await (await ctx.page.frameElement()).boundingBox()) ?? { x: 0, y: 0 };
      const p = await ctx.page.locator(`.arcviz-stage [data-id="${id}"]`).first().evaluate((g) => { const line = g.querySelector('.av-line'); const len = line.getTotalLength(); const q = line.getPointAtLength(len / 2); const m = line.getScreenCTM(); return { x: q.x * m.a + q.y * m.c + m.e, y: q.x * m.b + q.y * m.d + m.f }; });
      return { x: p.x + o.x, y: p.y + o.y };
    },
    clipboard: () => ctx.app.evaluate(({ clipboard }) => clipboard.readText()),
    shot: async (name) => { if (process.env.SHOTS) await win.screenshot({ path: path.join(process.env.SHOTS, `${name}.png`) }); },
  };
  return h;
}

/** draw.io's habits: one history, real keybindings, paste where asked, copies across models, Alt+drag, group moves. */
async function drawio() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcvs-drawio-'));
  const model = path.join(dir, 'model.arc'); const layout = path.join(dir, 'model.layout.json'); const other = path.join(dir, 'other.arc');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  fs.copyFileSync(path.join(EXT, '..', '..', 'examples', 'automotive', 'ev_product_line.arc'), other);
  const ctx = await launch(dir, model);
  const { app, win } = ctx;
  try {
    report('the workbench opens on the model (draw.io checks)', !!ctx.page && (await ctx.page.evaluate(() => document.getElementById('workbench')?.dataset.file)) === 'model.arc');
    if (!ctx.page) throw new Error('no workbench page');
    ctx.press = await azerty(win);
    const h = on(ctx);
    const src = () => fs.readFileSync(model, 'utf8');
    const lay = () => (fs.existsSync(layout) ? fs.readFileSync(layout, 'utf8') : '');
    const near = (a, b, tolerance) => Math.abs(a - b) <= tolerance;
    await h.layer('Logical'); await h.deselect();

    // 1. One history for the model and the arrangement, ⌘Z / ⌘⇧Z / ⌘Y on an AZERTY keyboard
    await h.save(); const s0 = src(); const l0 = lay();
    const act = await h.shape('LC-ACT');
    await h.drag({ x: act.x + 20, y: act.y + 10 }, { x: act.x + 120, y: act.y + 90 });
    const l1 = lay();
    const ctl = await h.shape('LC-CTL'); await win.mouse.dblclick(ctl.x + ctl.width / 2, ctl.y + 12); await win.waitForTimeout(700);
    await ctx.press('Meta+a'); await win.keyboard.type('Brain'); await ctx.press('Enter'); await win.waitForTimeout(3500); await h.save();
    const renamed = /component "Brain"/.test(src());
    await h.key('Meta+z'); await h.save();
    const undo1 = { renamed: /component "Brain"/.test(src()), moved: lay() === l1 };
    await h.key('Meta+z'); await h.save();
    const back = await h.shape('LC-ACT');
    report('⌘Z takes back a rename, then the move made before it', l1 !== l0 && renamed && !undo1.renamed && undo1.moved && src() === s0 && lay() === l0 && near(back.x, act.x, 4) && near(back.y, act.y, 4), JSON.stringify(undo1));
    await h.key('Meta+Shift+z'); await h.save(); const redo1 = lay() === l1 && src() === s0;
    await h.key('Meta+y'); await h.save(); const redo2 = /component "Brain"/.test(src());
    report('⌘⇧Z redoes the move, ⌘Y the rename', redo1 && redo2);
    await h.key('Meta+z'); await h.save();
    await h.click('LC-CAM'); await h.key('ArrowDown', 1500);
    await h.key('Meta+Shift+z'); await h.save();
    report('after a new step, ⌘⇧Z brings back nothing older', !/component "Brain"/.test(src()));
    await h.key('Meta+z'); await h.deselect();

    // 2. Every change of the arrangement is undone by one ⌘Z
    const undoes = async (name, gesture) => {
      await h.save(); const before = lay(); const text = src();
      const done = await gesture(); await win.waitForTimeout(1500);
      const changed = lay() !== before;
      await h.key('Meta+z', 3000); await h.save();
      report(`⌘Z undoes ${name}`, done !== false && changed && lay() === before && src() === text, done === false ? 'no gesture' : changed ? '' : 'nothing changed');
      await h.deselect();
    };
    await undoes('a nudge burst (Shift+arrows)', async () => { await h.click('LC-CAM'); await ctx.press('Shift+ArrowDown'); await ctx.press('Shift+ArrowDown'); });
    await undoes('a resize', async () => {
      const s = await h.shape('LC-CTL'); await win.mouse.click(s.x + 20, s.y + 8); await win.waitForTimeout(700);
      const g = await ctx.page.locator('.arcviz-grip').boundingBox().catch(() => null); if (!g) return false;
      await h.drag({ x: g.x + g.width / 2, y: g.y + g.height / 2 }, { x: g.x + 70, y: g.y + 40 });
    });
    await undoes('an alignment', async () => { const c = await h.click('LC-CAM'); await h.shiftClick('LC-ACT'); const t = await h.shape('LC-ACT'); return c && h.menu(t.x + t.width / 2, t.y + 10, 'Top'); });
    await undoes('a bend', async () => {
      const m = await h.middle('ce:LaneFlow'); await win.mouse.click(m.x, m.y); await win.waitForTimeout(700);
      const b = await ctx.page.locator('.arcviz-bend.is-new').first().boundingBox().catch(() => null); if (!b) return false;
      await h.drag({ x: b.x + b.width / 2, y: b.y + b.height / 2 }, { x: b.x + 10, y: b.y + 80 });
    });

    // 3. The text editor's ⌘Z is the text's
    await ctx.command('View: Focus First Editor Group'); await win.waitForTimeout(600);
    await ctx.press('Meta+ArrowDown'); await win.keyboard.type('// typed here'); await win.waitForTimeout(400); await ctx.press('Meta+s'); await win.waitForTimeout(800);
    const typed = /\/\/ typed here/.test(src());
    await ctx.press('Meta+z'); await win.waitForTimeout(600); await ctx.press('Meta+s'); await win.waitForTimeout(800);
    report('⌘Z in the text editor undoes the typing there', typed && !/\/\/ typed here/.test(src()));
    await h.deselect();

    // 4. Zoom keys zoom the sheet, not the window; ⌘0 and Shift+1 fit
    const zoomOf = () => app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].webContents.getZoomLevel());
    const z0 = await zoomOf(); const k0 = await h.scale();
    await h.key('Meta+=', 900); const k1 = await h.scale();
    await h.key('Meta+-', 900); const k2 = await h.scale();
    report('⌘= and ⌘− zoom the sheet, not the VS Code window', k1 > k0 * 1.05 && k2 < k1 * 0.95 && (await zoomOf()) === z0, `${k0.toFixed(2)} ${k1.toFixed(2)} ${k2.toFixed(2)}, window ${z0} -> ${await zoomOf()}`);
    await h.key('Meta+=', 600); await h.key('Meta+=', 900); await h.key('Meta+0', 1500); const fitted = await h.scale();
    await h.key('Meta+=', 900); await h.key('Shift+1', 1500); const again = await h.scale();
    report('⌘0 and Shift+1 fit the diagram (no Shift needed for ⌘0)', near(again, fitted, 0.01) && (await zoomOf()) === z0, `${fitted.toFixed(3)} ${again.toFixed(3)}`);

    // 5. Copy puts the source on the clipboard; paste lands 20,20 from the original; one ⌘Z takes both back
    await h.click('LC-ACT'); await h.key('Meta+c', 800);
    report('⌘C puts the ArcLang source of the selection on the clipboard', /^component "Steering Actuator" \{\n  id: "LC-ACT"/.test(await h.clipboard()));
    await h.key('Meta+v', 5000); await h.save();
    const scale = await h.scale(); const original = await h.shape('LC-ACT'); const copy = await h.shape('LC-ACT-2');
    const off = copy && original ? [Math.round((copy.x - original.x) / scale), Math.round((copy.y - original.y) / scale)] : null;
    await h.shot('drawio-paste');
    report('⌘V pastes the copy 20,20 from its original, kept in the layout file', !!off && near(off[0], 20, 3) && near(off[1], 20, 3) && /"LC-ACT-2"/.test(lay()), JSON.stringify(off));
    await h.key('Meta+z', 3000); await h.save();
    report('one ⌘Z takes back the paste and its place', !/LC-ACT-2/.test(src()) && !/"LC-ACT-2"/.test(lay()));
    const st = await h.stage(); const at = { x: st.x + st.width * 0.7, y: st.y + st.height * 0.82 };
    const menuPaste = await h.menu(at.x, at.y, 'Paste here'); await win.waitForTimeout(2000);
    const there = await h.shape('LC-ACT-2');
    report('Paste here in the menu puts the copy at the mouse', menuPaste && !!there && near(there.x, at.x, 6) && near(there.y, at.y, 6), there ? `${Math.round(there.x - at.x)},${Math.round(there.y - at.y)}` : 'no copy');
    await h.key('Meta+z', 3000); await h.deselect();

    // 6. Several pasted together keep what joins them; a cut pasted on the sheet goes to the layer
    await h.click('LC-CAM'); await h.shiftClick('LC-CTL'); await h.key('Meta+c', 800); await h.key('Meta+v', 6000); await h.save();
    report('a multi-selection pasted keeps its exchange, joined to the copies', /component_exchange "LaneFlow 2" \{\n    from_port: "LC-CAM-2\.lane"\n    to_port: "LC-CTL-2\.lane"/.test(src()));
    await h.shot('drawio-multipaste');
    await h.key('Meta+z', 3000); await h.deselect();
    await h.click('LF-CAP'); await h.key('Meta+x', 800); await h.deselect(); await h.key('Meta+v', 5000); await h.save();
    const camera = /component "Camera" \{[\s\S]*?\n  \}/.exec(src())?.[0] ?? '';
    report('a cut pasted on the empty sheet moves the element to the layer', !camera.includes('Capture frame') && (src().match(/function "Capture frame"/g) || []).length === 1);
    await h.key('Meta+z', 3000); await h.deselect();

    // 7. ArcLang copied in the text editor pastes on the canvas, with fresh ids
    await h.save(); const lines = src().split('\n'); const first = lines.findIndex((l) => l.includes('component "Steering Actuator"'));
    const last = lines.findIndex((l, i) => i > first && l === '  }');
    await ctx.command('View: Focus First Editor Group'); await win.waitForTimeout(500);
    await ctx.press('Control+g'); await win.waitForTimeout(400); await win.keyboard.type(String(first + 1)); await ctx.press('Enter'); await win.waitForTimeout(300);
    await ctx.press('Home'); await ctx.press('Home');
    for (let i = first; i <= last; i++) await ctx.press('Shift+ArrowDown');
    await ctx.press('Meta+c'); await win.waitForTimeout(400);
    const fromText = await h.clipboard();
    await h.deselect(); await h.key('Meta+v', 5000); await h.save();
    report('ArcLang copied from the text editor pastes with fresh ids', /^\s*component "Steering Actuator" \{/.test(fromText) && /component "Steering Actuator 2" \{\n    id: "LC-ACT-2"/.test(src()) && /id: "LF-ACT-2"/.test(src()), JSON.stringify(fromText.slice(0, 60)) + ' ' + (await ctx.page.locator('.wb-toast').textContent().catch(() => '')).slice(0, 120));
    await h.key('Meta+z', 3000); await h.deselect();

    // 8. ⌘+drag drops a copy where it is let go (draw.io; Alt+drag moves off the grid)
    const a0 = await h.shape('LC-ACT');
    await win.keyboard.down('Meta'); await h.drag({ x: a0.x + 20, y: a0.y + 10 }, { x: a0.x + 20, y: a0.y + 190 }, 300); await win.keyboard.up('Meta'); await win.waitForTimeout(5000); await h.save();
    const a1 = await h.shape('LC-ACT'); const a2 = await h.shape('LC-ACT-2'); const s1 = await h.scale();
    await h.shot('drawio-altdrag');
    report('⌘+drag drops a copy where it is let go', !!a2 && near(a2.x - a1.x, 0, 8) && near(a2.y - a1.y, 180, 14) && /id: "LC-ACT"/.test(src()), a2 ? `${Math.round(a2.x - a1.x)},${Math.round(a2.y - a1.y)} px; before ${Math.round(a0.x)},${Math.round(a0.y)} original ${Math.round(a1.x)},${Math.round(a1.y)} copy ${Math.round(a2.x)},${Math.round(a2.y)}; placed ${/"LC-ACT-2"/.test(lay())}` : 'no copy');
    await h.key('Meta+z', 3000); await h.deselect();

    // 9. Dragging one of several selected boxes moves them all, one ⌘Z puts them back
    await h.save(); const lb = lay();
    const c0 = await h.click('LC-CAM'); await h.shiftClick('LC-CTL'); const t0 = await h.shape('LC-CTL');
    await h.drag({ x: c0.x + 30, y: c0.y + 9 }, { x: c0.x + 30, y: c0.y + 129 }, 3500); await h.save();
    const c1 = await h.shape('LC-CAM'); const t1 = await h.shape('LC-CTL');
    await h.shot('drawio-groupdrag');
    report('dragging one of the selected boxes moves them all', near(c1.y - c0.y, 120, 12) && near(t1.y - t0.y, 120, 12), `${Math.round(c1.y - c0.y)} / ${Math.round(t1.y - t0.y)}`);
    await h.key('Meta+z', 3000); await h.save();
    const c2 = await h.shape('LC-CAM'); const t2 = await h.shape('LC-CTL');
    report('one ⌘Z puts the whole group back', near(c2.y, c0.y, 4) && near(t2.y, t0.y, 4) && lay() === lb);
    await h.deselect();

    // 10. A new diagram from the selection
    await h.click('LC-CAM'); await h.shiftClick('LC-ACT'); const ab = await h.shape('LC-ACT');
    const made = await h.menu(ab.x + ab.width / 2, ab.y + 10, 'New diagram from the selection'); await win.waitForTimeout(2000); await h.save();
    const diagram = /diagram "2 elements diagram" \{\n  view: lab\n  shows: \[([^\]]*)\]/.exec(src());
    report('New diagram from the selection shows the selected elements', made && !!diagram && /LC-CAM/.test(diagram[1]) && /LC-ACT/.test(diagram[1]) && !/LC-CTL/.test(diagram[1]), diagram ? diagram[1] : 'no diagram');
    await h.key('Meta+z', 3000); await h.layer('Logical');

    // 11. The box under the pointer is outlined, and only it
    const hovered = async (id, top) => { const b = await h.shape(id); await win.mouse.move(b.x + b.width / 2, top ? b.y + 8 : b.y + b.height / 2, { steps: 4 }); await win.waitForTimeout(300); return ctx.page.evaluate(() => [...document.querySelectorAll('.wb-hover')].map((n) => n.dataset.id).join(',')); };
    await ctx.press('Escape'); await h.deselect();
    const over = [await hovered('LC-ACT', true), await hovered('LF-CAP')];
    await h.shot('drawio-hover');
    report('the box under the pointer is outlined, not its container', over[0] === 'LC-ACT' && over[1] === 'LF-CAP', over.join(' / '));

    // 12. Across models: a copy here pastes into another model's workbench
    await h.click('LC-CAM'); await h.key('Meta+c', 800);
    await win.keyboard.press('Meta+p'); await win.waitForTimeout(500); await win.keyboard.type(other); await win.waitForTimeout(1200); await win.keyboard.press('Enter'); await win.waitForTimeout(2000);
    await ctx.command('ArcLang: Open the workbench');
    const second = await workbenchPage(win, [ctx.page]);
    report('another model opens its own workbench', !!second);
    if (second) {
      const before = fs.readFileSync(other, 'utf8');
      ctx.page = second; await h.layer('Logical'); await h.deselect();
      await h.key('Meta+v', 5000); await h.save();
      const after = fs.readFileSync(other, 'utf8');
      await h.shot('drawio-crossmodel');
      report('the copy pastes into the other model with its declarations', after !== before && /component "Camera" \{\n\s+id: "LC-CAM"/.test(after) && /function "Capture frame"/.test(after));
    }
  } catch (error) {
    report('no error (draw.io checks)', false, String(error).slice(0, 300));
  } finally {
    await app.close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

/** Two models open, the other one's text editor focused last: ⌘Z on the workbench changes only its own document. */
async function twoModels() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcvs-two-'));
  const model = path.join(dir, 'model.arc'); const other = path.join(dir, 'other.arc');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), other);
  const ctx = await launch(dir, model);
  const { app, win } = ctx;
  try {
    if (!ctx.page) throw new Error('no workbench page');
    ctx.press = await azerty(win);
    const h = on(ctx);
    const src = () => fs.readFileSync(model, 'utf8'); const otherText = () => fs.readFileSync(other, 'utf8');
    await h.layer('Logical'); await h.deselect(); await h.save();
    const s0 = src();
    // An edit of the model on the canvas.
    const ctl = await h.shape('LC-CTL'); await win.mouse.dblclick(ctl.x + ctl.width / 2, ctl.y + 12); await win.waitForTimeout(700);
    await ctx.press('Meta+a'); await win.keyboard.type('Brain'); await ctx.press('Enter'); await win.waitForTimeout(3500); await h.save();
    const renamed = /component "Brain"/.test(src());
    // The other model's text editor, typed in and saved, focused last.
    await ctx.command('View: Focus First Editor Group'); await win.waitForTimeout(500);
    await win.keyboard.press('Meta+p'); await win.waitForTimeout(500); await win.keyboard.type(other); await win.waitForTimeout(1200); await win.keyboard.press('Enter'); await win.waitForTimeout(1500);
    await ctx.press('Meta+ArrowDown'); await win.keyboard.type('// typed in the other model'); await win.waitForTimeout(400); await ctx.press('Meta+s'); await win.waitForTimeout(800);
    const o0 = otherText();
    await h.shot('two-models-before-undo');
    // ⌘Z twice on the workbench: its rename, then nothing of its own.
    await h.deselect(); await h.key('Meta+z', 3000); await h.deselect(); await h.key('Meta+z', 3000);
    await ctx.command('File: Save All'); await win.waitForTimeout(1200);
    report('two models: the edit was made on the canvas, the other typed in', renamed && /typed in the other model/.test(o0), JSON.stringify({ renamed, typed: /typed in the other model/.test(o0) }));
    report('⌘Z on the workbench takes back its own edit', src() === s0, src() === s0 ? '' : /component "Brain"/.test(src()) ? 'still renamed' : 'other change');
    report('⌘Z on the workbench never touches the other model', otherText() === o0, otherText() === o0 ? '' : /typed in the other model/.test(otherText()) ? 'changed otherwise' : 'its typing was undone');
  } catch (error) {
    report('no error (two models)', false, String(error).slice(0, 300));
  } finally {
    await app.close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

/** The themes (Light, Dark, High Contrast) and a narrow editor, with pictures. */
async function looks() {
  const themes = [['Default Light Modern', 'vscode-light', 'none'], ['Default Dark Modern', 'vscode-dark', 'invert'], ['Default High Contrast', 'vscode-high-contrast', 'invert']];
  for (const [theme, bodyClass, filter] of themes) {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcvs-look-'));
    const model = path.join(dir, 'model.arc'); fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
    const ctx = await launch(dir, model, { 'workbench.colorTheme': theme });
    const { app, win } = ctx;
    try {
      if (!ctx.page) throw new Error('no workbench page');
      ctx.press = await azerty(win); const h = on(ctx);
      await h.layer('Logical');
      const look = await ctx.page.evaluate(() => ({ body: document.body.className, filter: getComputedStyle(document.querySelector('.arcviz-stage svg.av-svg')).filter }));
      const slug = bodyClass.replace('vscode-', '');
      await h.shot(`theme-${slug}`);
      report(`${theme}: the sheet and the boxes follow the theme`, look.body.includes(bodyClass) && (filter === 'none' ? look.filter === 'none' : /invert/.test(look.filter)), `${look.body} | ${look.filter}`);
      if (bodyClass !== 'vscode-light') continue;
      // A narrow editor: the canvas keeps the width, the properties float over it.
      await ctx.command('View: Close Primary Side Bar'); await h.deselect();
      await ctx.command('View: Single Column Editor Layout'); await win.waitForTimeout(1500);
      for (const width of [600, 1240]) {
        // The window is sized until the webview is as wide as asked (the editor chrome takes the rest).
        let inner = 0;
        for (let attempt = 0; attempt < 3 && Math.abs(inner - width) > 20; attempt++) {
          inner = await ctx.page.evaluate(() => innerWidth);
          const bounds = await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].getBounds());
          await app.evaluate(({ BrowserWindow }, w) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: w, height: 900 }), bounds.width + width - inner);
          await win.waitForTimeout(2000);
          inner = await ctx.page.evaluate(() => innerWidth);
        }
        if (await ctx.page.locator('.wb-right').isHidden()) { await ctx.page.evaluate(() => document.querySelector('.wb-icon-button[title="Properties"]').click()); await win.waitForTimeout(800); }
        await h.click('LC-CAM');
        const panes = await ctx.page.evaluate(() => ({ stage: document.querySelector('.arcviz-stage').getBoundingClientRect().width, right: getComputedStyle(document.querySelector('.wb-right')).position, scroll: document.documentElement.scrollWidth > innerWidth }));
        await h.shot(`narrow-${width}`);
        report(`at ${width} px the canvas keeps the width${width < 900 ? ', the properties float over it' : ''}`, !panes.scroll && (width < 900 ? panes.right === 'absolute' && panes.stage > inner * 0.8 : panes.right !== 'absolute'), `${inner}px: stage ${Math.round(panes.stage)}, drawer ${panes.right}`);
      }
    } catch (error) {
      report(`no error (${theme})`, false, String(error).slice(0, 300));
    } finally {
      await app.close().catch(() => {});
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }
}

/** The window sized until the webview is `width` px wide (the editor chrome takes the rest). */
async function sizeTo(ctx, width) {
  const { app, win } = ctx;
  let inner = 0;
  for (let attempt = 0; attempt < 4 && Math.abs(inner - width) > 12; attempt++) {
    inner = await ctx.page.evaluate(() => innerWidth);
    const bounds = await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].getBounds());
    await app.evaluate(({ BrowserWindow }, w) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: w, height: 900 }), bounds.width + width - inner);
    await win.waitForTimeout(1800);
    inner = await ctx.page.evaluate(() => innerWidth);
  }
  return inner;
}

/** The toolbar in every theme and width: one line, nothing cut, menus by mouse and keyboard, the zoom level, undo and redo. */
async function toolbar() {
  const themes = [['Default Light Modern', 'light'], ['Default Dark Modern', 'dark'], ['Default High Contrast', 'hc']];
  // THEMES=light,dark,hc runs some of them.
  for (const [theme, slug] of themes.filter(([, s]) => !process.env.THEMES || process.env.THEMES.split(',').includes(s))) {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcvs-tools-'));
    const model = path.join(dir, 'model.arc'); fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
    const ctx = await launch(dir, model, { 'workbench.colorTheme': theme });
    const { win } = ctx;
    try {
      if (!ctx.page) throw new Error('no workbench page');
      ctx.press = await azerty(win); const h = on(ctx); const page = ctx.page;
      const id = (name) => page.locator(`#${name}`);
      const open = () => page.locator('.arcviz-menu-pop:not([hidden])');
      const action = (name) => page.locator(`.arcviz-menu-pop:not([hidden]) [data-action="${name}"]`);
      const crop = async (name, height = 140) => {
        if (!process.env.SHOTS) return;
        const frame = await (await page.frameElement()).boundingBox();
        await win.screenshot({ path: path.join(process.env.SHOTS, `${name}.png`), clip: { x: frame.x, y: frame.y, width: frame.width, height } });
      };
      await h.layer('Logical');
      await ctx.command('View: Close Primary Side Bar'); await h.deselect();
      await ctx.command('View: Single Column Editor Layout'); await win.waitForTimeout(1500);
      for (const width of [1240, 800, 600, 420]) {
        const inner = await sizeTo(ctx, width);
        const row = await page.evaluate(() => {
          const bar = document.querySelector('.wb-toolrow').getBoundingClientRect();
          const shown = [...document.querySelectorAll('.wb-toolrow button')].filter((b) => b.getBoundingClientRect().width > 0);
          const cut = shown.filter((b) => { const r = b.getBoundingClientRect(); return r.top < bar.top - 1 || r.bottom > bar.bottom + 1 || r.left < bar.left - 1 || r.right > bar.right + 1; }).map((b) => b.id || b.getAttribute('aria-label'));
          const more = document.getElementById('arcviz-more');
          return { cut, rows: new Set(shown.map((b) => Math.round(b.getBoundingClientRect().top))).size, more: !more.closest('.arcviz-menu').hidden, compact: document.querySelector('.arcviz-tools').classList.contains('is-compact') };
        });
        report(`${slug} ${width}: the toolbar is one line, every button fully visible`, row.cut.length === 0 && row.rows === 1 && (width > 900 ? !row.more : true), `${inner}px ${row.cut.join(',')} compact ${row.compact} more ${row.more}`);
        await crop(`vscode-${slug}-${width}`);
        if (width === 420) {
          await id('arcviz-more').click(); await win.waitForTimeout(400);
          report(`${slug} 420: the tools that do not fit are in "⋯"`, (await action('save-svg').count()) === 1 && (await action('fold-all').count()) === 1);
          await crop(`vscode-${slug}-420-more`, 560);
          await ctx.press('Escape'); await win.waitForTimeout(300);
        }
      }
      await sizeTo(ctx, 1240);
      // Menus by mouse: Layout, Align (two boxes selected), Export
      await id('arcviz-layout-menu').click(); await win.waitForTimeout(400);
      report(`${slug}: Layout opens by mouse`, (await action('reset-layout').count()) === 1 && (await action('arrange').count()) === 0);
      await crop(`vscode-${slug}-menu-layout`, 300);
      await action('fold-all').click(); await win.waitForTimeout(1500);
      report(`${slug}: a Layout item runs and closes the menu`, (await open().count()) === 0 && (await page.locator('.arcviz-stage .av-node').count()) > 0);
      await id('arcviz-layout-menu').click(); await win.waitForTimeout(300); await action('unfold-all').click(); await win.waitForTimeout(1500);
      report(`${slug}: Align is off until two boxes are selected`, (await id('arcviz-align-menu').getAttribute('aria-disabled')) === 'true');
      // Camera and actuator sit on different rows: Align top puts them on one.
      const gap = async () => { const [p, q] = [await h.shape('LC-CAM'), await h.shape('LC-ACT')]; return p && q ? Math.round(q.y - p.y) : NaN; };
      const before = await gap();
      await h.click('LC-CAM'); await h.shiftClick('LC-ACT');
      await id('arcviz-align-menu').click(); await win.waitForTimeout(400);
      report(`${slug}: Align opens with two boxes selected`, (await action('align-top').count()) === 1);
      await crop(`vscode-${slug}-menu-align`, 380);
      await action('align-top').click(); await win.waitForTimeout(2500);
      const aligned = await gap();
      report(`${slug}: Align top lines the boxes up`, Math.abs(before) > 20 && Math.abs(aligned) < 2, `${before} -> ${aligned}`);
      // Undo and redo buttons: the history's labels, then the gesture undone and redone
      const undoTitle = await id('arcviz-undo').getAttribute('title');
      report(`${slug}: Undo names the last change, with its shortcut`, !(await id('arcviz-undo').isDisabled()) && /^Undo .+ \(⌘Z\)$/.test(undoTitle), undoTitle);
      await id('arcviz-undo').click(); await win.waitForTimeout(3500);
      const undone = await gap();
      report(`${slug}: the Undo button puts the boxes back`, Math.abs(undone - before) < 3 && !(await id('arcviz-redo').isDisabled()), `${undone} | ${await id('arcviz-redo').getAttribute('title')}`);
      await id('arcviz-redo').click(); await win.waitForTimeout(3500);
      const redone = await gap();
      report(`${slug}: the Redo button aligns them again`, Math.abs(redone) < 2, `${redone}`);
      await h.deselect();
      // Export by keyboard: Enter opens on the first item, arrows move, Esc closes and gives the focus back
      await id('arcviz-export-menu').focus(); await ctx.press('Enter'); await win.waitForTimeout(400);
      const first = await page.evaluate(() => document.activeElement?.dataset.action);
      await ctx.press('ArrowDown'); const second = await page.evaluate(() => document.activeElement?.dataset.action);
      await crop(`vscode-${slug}-menu-export`, 260);
      await ctx.press('Escape'); await win.waitForTimeout(300);
      const back = await page.evaluate(() => document.activeElement?.id);
      report(`${slug}: Export by keyboard (Enter, arrows, Esc)`, first === 'save-svg' && second === 'save-png' && (await open().count()) === 0 && back === 'arcviz-export-menu', `${first} ${second} -> ${back}`);
      // The zoom level follows the buttons, the presets and the keys
      const level = async () => (await id('arcviz-zoom-level').textContent()).replace(/\s/g, ' ').trim();
      await id('arcviz-zoom-level').click(); await win.waitForTimeout(300); await action('zoom-75').click(); await win.waitForTimeout(600);
      const l75 = await level();
      await id('arcviz-zoom-in').click(); await win.waitForTimeout(600);
      const lIn = await level();
      await page.locator('.arcviz-stage').click({ position: { x: 6, y: 6 } }); await h.key('-', 800);
      const lKey = await level();
      report(`${slug}: the zoom level shows the zoom (preset, +, the - key)`, l75 === '75 %' && lIn === '94 %' && lKey === '75 %', `${l75} ${lIn} ${lKey}`);
      // The panel toggles: a pressed state, the shortcut in the tooltip
      await page.locator('.wb-minimap-toggle').click(); await win.waitForTimeout(500);
      report(`${slug}: the minimap toggle shows it is on, Shift+M in its tooltip`, (await page.locator('.wb-minimap-toggle').getAttribute('aria-pressed')) === 'true' && /⇧M|Shift\+M/.test(await page.locator('.wb-minimap-toggle').getAttribute('title')));
      await page.locator('.wb-minimap-toggle').click();
    } catch (error) {
      report(`no error (toolbar, ${theme})`, false, String(error).slice(0, 300));
    } finally {
      await ctx.app.close().catch(() => {});
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }
}

// ONLY=basics|drawio|two|looks|toolbar runs one part.
(async () => {
  const only = process.env.ONLY;
  if (!only || only === 'basics') await basics();
  if (!only || only === 'drawio') await drawio();
  if (!only || only === 'two') await twoModels();
  if (!only || only === 'looks') await looks();
  if (!only || only === 'toolbar') await toolbar();
  const failed = rows.filter((r) => !r.ok); console.log(`\n${rows.length - failed.length}/${rows.length} checks in VS Code`);
  process.exit(failed.length ? 1 : 0);
})();

