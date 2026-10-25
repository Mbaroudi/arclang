// draw.io's Arrange inside the real VS Code, keys sent as a French (AZERTY) Mac sends them:
// send to back / bring to front and forward / backward (DOM order, the layout file, ⌘Z); the
// Layers panel (new layer, the selection moved to it as a tag, hidden: boxes, exchanges and the
// SVG export lose it, locked, reordered); group, move, collapse, ungroup, group into a new
// component; lock; same size; clear waypoints; select edges; the state kept across a reload.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_layers_test.js
// SHOTS=dir saves pictures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail || ''}`); };

async function workbenchPage(win, not = []) {
  for (let i = 0; i < 60; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (!not.includes(f) && await f.$('.arcviz-stage svg')) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

async function launch(dir, model) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  return { app, win, command, page: await workbenchPage(win), press: await azerty(win) };
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclayers-'));
  const model = path.join(dir, 'model.arc'); const layoutFile = path.join(dir, 'model.layout.json');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const ctx = await launch(dir, model);
  const { app, win, command, press } = ctx; let page = ctx.page;
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const layout = () => (fs.existsSync(layoutFile) ? fs.readFileSync(layoutFile, 'utf8') : '');
  const source = async () => { await press('Meta+s'); await win.waitForTimeout(900); return fs.readFileSync(model, 'utf8'); };
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    const settle = (ms = 1800) => win.waitForTimeout(ms);
    const node = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first();
    const shape = async (id) => ((await node(id).count()) ? node(id).locator(':scope > .av-shape').first().boundingBox() : null);
    const order = () => page.evaluate(() => [...document.querySelectorAll('.arcviz-stage .av-node')].map((n) => n.dataset.id));
    const above = async (a, b) => { const o = await order(); return o.indexOf(a) > o.indexOf(b); };
    const tool = (id) => page.locator(`#${id}`);
    const drag = async (from, to, steps = 14) => { await win.mouse.move(from.x, from.y); await win.mouse.down(); await win.mouse.move(to.x, to.y, { steps }); await win.mouse.up(); await settle(); };
    const click = async (id, shift) => { const b = await shape(id); if (shift) await win.keyboard.down('Shift'); await win.mouse.click(b.x + b.width - 30, b.y + 8); if (shift) await win.keyboard.up('Shift'); await win.waitForTimeout(500); };
    const rightClick = async (id) => { const b = await shape(id); await win.mouse.click(b.x + b.width - 30, b.y + 8, { button: 'right' }); await win.waitForTimeout(500); };
    const menuItem = (label) => page.locator('.wb-menu:not([hidden]) button', { hasText: label }).first();
    const blank = async () => { const s = await page.locator('.arcviz-stage').boundingBox(); await win.mouse.click(s.x + s.width - 30, s.y + s.height - 30); await win.waitForTimeout(400); };
    const fit = async () => { await tool('arcviz-fit').click(); await settle(1200); };
    const toast = () => page.locator('.wb-toast:not([hidden])').textContent().catch(() => '');
    const rel = async (id, to) => { const a = await shape(id); const b = await shape(to); return { x: a.x - b.x, y: a.y - b.y }; };
    const prompt = async (text) => { const i = page.locator('.wb-prompt input').first(); await i.waitFor({ timeout: 4000 }); await i.fill(text); await i.press('Enter'); await settle(2500); };
    const hasOrder = () => /"order"\s*:\s*\{\s*"/.test(layout());

    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await settle(1800);
    await fit();
    // Two boxes that overlap: the actuator dragged over the camera.
    let act = await shape('LC-ACT'); const cam = await shape('LC-CAM');
    await drag({ x: act.x + act.width - 12, y: act.y + 8 }, { x: act.x + act.width - 12 - (act.x - cam.x - 90), y: act.y + 8 - (act.y - cam.y - 30) });
    act = await shape('LC-ACT'); const cam1 = await shape('LC-CAM');
    report('two boxes overlap, the later drawn on top', act.x < cam1.x + cam1.width && act.x + act.width > cam1.x && act.y < cam1.y + cam1.height && act.y + act.height > cam1.y && (await above('LC-ACT', 'LC-CAM')));
    await shot('layers-00-overlap');

    // 1. Send to back, bring to front, forward, backward: DOM order, the layout file, ⌘Z
    await click('LC-ACT'); await press('Meta+Shift+b'); await settle(1500);
    report('⇧⌘B sends it to the back: drawn under the camera', !(await above('LC-ACT', 'LC-CAM')), (await order()).join(','));
    report('the stacking is kept in the layout file', hasOrder());
    await press('Meta+z'); await settle(2500);
    report('⌘Z puts it back on top and the file loses the order', (await above('LC-ACT', 'LC-CAM')) && !hasOrder(), `${await above('LC-ACT', 'LC-CAM')} ${hasOrder()}`);
    await click('LC-ACT'); await press('Meta+Alt+Shift+b'); await settle(1500);
    report('⌥⇧⌘B sends it one step backward', !(await above('LC-ACT', 'LC-CAM')));
    await click('LC-ACT'); await press('Meta+Alt+Shift+f'); await settle(1500);
    report('⌥⇧⌘F brings it one step forward', await above('LC-ACT', 'LC-CAM'));
    await click('LC-ACT'); await press('Meta+Shift+b'); await settle(1200); await click('LC-ACT'); await press('Meta+Shift+f'); await settle(1500);
    report('⇧⌘F brings it to the front again', await above('LC-ACT', 'LC-CAM'));
    await shot('layers-01-order');

    // 2. The Layers panel: a new layer, the selection moved onto it (an edit of the tags), hidden, locked, ordered
    await blank(); await press('Meta+Shift+l'); await settle(900);
    report('⇧⌘L opens the Layers panel', (await page.locator('.wb-tl:not([hidden])').count()) === 1, `${await page.locator('.wb-tl').count()} ${await page.locator('.wb-tl').first().getAttribute('hidden')}`); await shot('layers-02a-panel-key');
    await page.locator('.wb-tl button', { hasText: 'New layer' }).click(); await prompt('Safety');
    await click('LC-ACT');
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-move').click(); await settle(3500);
    let text = await source();
    report('moving the selection to a layer adds the tag in the source', /id: "LC-ACT"[\s\S]{0,200}tags: \["Safety"\]/.test(text));
    await press('Meta+z'); await settle(2800); text = await source();
    report('⌘Z takes the tag back', !/tags: \["Safety"\]/.test(text));
    await press('Meta+Shift+z'); await settle(2800); text = await source();
    report('⇧⌘Z gives it again', /tags: \["Safety"\]/.test(text));
    await shot('layers-02-panel');
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-eye').click(); await settle(2200);
    report('a hidden layer: its box and its exchanges disappear', (await node('LC-ACT').count()) === 0 && (await node('ce:SteerCommand').count()) === 0 && (await node('LC-CAM').count()) === 1);
    await shot('layers-03-hidden');
    await openExport(page); await win.waitForTimeout(300);
    const tip = await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="save-png"]').getAttribute('title');
    await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="save-svg"]').click(); await win.waitForTimeout(1200);
    const input = win.locator('.quick-input-widget input').first();
    if (await input.isVisible().catch(() => false)) { await input.fill(path.join(dir, 'hidden.svg')); await input.press('Enter'); await win.waitForTimeout(1800); }
    const svgText = fs.existsSync(path.join(dir, 'hidden.svg')) ? fs.readFileSync(path.join(dir, 'hidden.svg'), 'utf8') : '';
    report('the SVG export leaves the hidden layer out', svgText.includes('Camera') && !svgText.includes('Steering Actuator') && !svgText.includes('SteerCommand'), `${svgText.length} bytes`);
    report('the export menu says so', /hidden layers are left out/.test(tip || ''), tip);
    await win.keyboard.press('Escape');
    const png = path.join(dir, 'hidden.png');
    await openExport(page); await win.waitForTimeout(300);
    await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="save-png"]').click(); await win.waitForTimeout(1200);
    if (await input.isVisible().catch(() => false)) { await input.fill(png); await input.press('Enter'); await win.waitForTimeout(2200); }
    report('the PNG export is written without it', fs.existsSync(png) && fs.readFileSync(png).subarray(1, 4).toString() === 'PNG');
    await win.keyboard.press('Escape');
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-eye').click(); await settle(2200);
    report('shown again, it is back', (await node('LC-ACT').count()) === 1);
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-lock').click(); await settle(1500);
    const a0 = await rel('LC-ACT', 'LC-CTL'); const ab = await shape('LC-ACT');
    await drag({ x: ab.x + ab.width - 30, y: ab.y + 8 }, { x: ab.x + ab.width + 40, y: ab.y + 80 });
    const a1 = await rel('LC-ACT', 'LC-CTL');
    report('a box on a locked layer does not move', Math.abs(a1.x - a0.x) < 1 && Math.abs(a1.y - a0.y) < 1);
    const before = await source(); await click('LC-ACT'); await press('Delete'); await settle(1200);
    report('Delete on it is refused with a message, the source unchanged', (await source()) === before && /locked/.test(await toast()), await toast());
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-lock').click(); await settle(1200);
    await page.locator('.wb-tl-row[data-layer="Safety"] button[aria-label^="Move Safety down"]').click(); await settle(2000);
    report('Safety moved under Default: kept in the layout file', /"order":\s*\[\s*"",\s*"Safety"\s*\]/.test(layout()), layout().replace(/\s+/g, ' ').slice(-120));
    await page.locator('.wb-tl button[title^="Close"]').click();

    // 3. Group, move, collapse, ungroup (⌘Z restores)
    await fit(); await blank(); await click('LC-CAM'); await click('LC-CTL', true);
    await press('Meta+g'); await settle(2000);
    report('⌘G frames the two boxes; the layout file holds the group, the model is unchanged', (await page.locator('.arcviz-stage .av-group').count()) === 1 && /"groups"/.test(layout()) && (await source()) === before);
    await shot('layers-04-group');
    const gf = await page.locator('.arcviz-stage .av-group > .av-shape').boundingBox(); const c0 = await shape('LC-CAM'); const t0 = await shape('LC-CTL');
    await drag({ x: gf.x + gf.width / 2, y: gf.y + 6 }, { x: gf.x + gf.width / 2 + 40, y: gf.y + 6 + 30 });
    const c1 = await shape('LC-CAM'); const t1 = await shape('LC-CTL');
    report('the group moves both members', Math.abs((c1.x - c0.x) - (t1.x - t0.x)) < 2 && c1.y - c0.y > 15, `${Math.round(c1.x - c0.x)},${Math.round(c1.y - c0.y)} / ${Math.round(t1.x - t0.x)},${Math.round(t1.y - t0.y)} frame ${JSON.stringify(gf)}`);
    await press('Meta+z'); await settle(2500);
    report('⌘Z puts the group back', Math.abs((await shape('LC-CAM')).x - c0.x) < 3);
    await click('LC-CAM'); await press('Meta+Shift+g'); await settle(1800);
    report('⇧⌘G ungroups', (await page.locator('.arcviz-stage .av-group').count()) === 0 && !/"groups"/.test(layout()));
    await press('Meta+z'); await settle(2200);
    report('⌘Z brings the group back', (await page.locator('.arcviz-stage .av-group').count()) === 1);
    await click('LC-CAM'); await press('Meta+Shift+g'); await settle(1800);

    // 4. Group into a new component: the source has it, one ⌘Z
    await fit(); await blank(); await click('LC-CAM'); await click('LC-CTL', true);
    await rightClick('LC-CTL'); await menuItem('Group into a new component').click(); await prompt('Sensing');
    text = await source(); const block = (text.split('component "Sensing"')[1] || '').split('component "Steering Actuator"')[0];
    report('Group into a new component: the model holds both inside it', /component "Camera"/.test(block) && /component "LKA Controller"/.test(block));
    await shot('layers-05-component');
    await press('Meta+z'); await settle(2800); text = await source();
    report('one ⌘Z takes the new component back', !/component "Sensing"/.test(text));

    // 5. Lock one element, same size, clear waypoints, select edges
    await fit(); await blank(); await click('LC-CAM'); await press('Meta+l'); await settle(1500);
    report('⌘L locks the box (padlock, kept in the file)', (await page.locator('.arcviz-stage [data-id="LC-CAM"].is-locked .av-lock').count()) === 1 && /"locked"/.test(layout()));
    const l0 = await rel('LC-CAM', 'LC-CTL'); const lb = await shape('LC-CAM');
    await drag({ x: lb.x + lb.width - 30, y: lb.y + 8 }, { x: lb.x + lb.width + 30, y: lb.y + 60 });
    const l1 = await rel('LC-CAM', 'LC-CTL');
    report('a locked box does not move', Math.abs(l1.x - l0.x) < 1 && Math.abs(l1.y - l0.y) < 1);
    await click('LC-CAM'); await press('Meta+l'); await settle(1200);
    await fit(); await blank(); await click('LC-CAM'); await click('LC-CTL', true);
    await rightClick('LC-CTL'); await page.locator('.wb-menu:not([hidden]) .wb-menu-row[aria-label="Same"] button', { hasText: 'Size' }).click(); await settle(2000);
    const sc = await shape('LC-CAM'); const st = await shape('LC-CTL');
    report('Same size: the first takes the size of the last selected', Math.abs(sc.width - st.width) < 2 && Math.abs(sc.height - st.height) < 2, `${Math.round(sc.width)}x${Math.round(sc.height)}`);
    await blank();
    const lm = await (async () => { const o = (await (await page.frameElement()).boundingBox()) ?? { x: 0, y: 0 }; const p = await node('ce:LaneFlow').evaluate((g) => { const l = g.querySelector('.av-line'); const q = l.getPointAtLength(l.getTotalLength() / 2); const m = l.getScreenCTM(); return { x: q.x * m.a + q.y * m.c + m.e, y: q.x * m.b + q.y * m.d + m.f }; }); return { x: p.x + o.x, y: p.y + o.y }; })();
    await win.mouse.click(lm.x, lm.y); await win.waitForTimeout(700);
    const dia = page.locator('.arcviz-bend.is-new').first();
    if (await dia.isVisible().catch(() => false)) { const d = await dia.boundingBox(); await drag({ x: d.x + d.width / 2, y: d.y + d.height / 2 }, { x: d.x + d.width / 2, y: d.y + d.height / 2 - 70 }, 10); }
    const bent = /"points"/.test(layout());
    await tool('arcviz-layout-menu').click(); await win.waitForTimeout(300);
    await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="arrange-clear-waypoints"]').click(); await settle(2000);
    report('Clear waypoints straightens the selected exchange', bent && !/"points"/.test(layout()), `bent first: ${bent}`);
    await blank(); await press('Meta+Shift+e'); await win.waitForTimeout(1000);
    const selected = await page.locator('.av-edge.wb-multi').count();
    report('⇧⌘E selects every exchange', selected === 2 && /2 lines selected/.test(await toast()), `${selected}`);
    await shot('layers-06-edges');
    await press('Meta+Shift+a'); await win.waitForTimeout(800);
    report('⇧⌘A selects nothing', (await page.locator('.wb-multi').count()) === 0);

    // 6. The state is kept: hide a layer and group, reload the webviews
    await blank(); await click('LC-CAM'); await click('LC-CTL', true); await press('Meta+g'); await settle(1800);
    await press('Meta+Shift+l'); await settle(900);
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-eye').click(); await settle(2000);
    const keptLayout = layout();
    const old = page;
    await command('Developer: Reload Webviews'); await win.waitForTimeout(2500);
    page = await workbenchPage(win, [old]);
    report('the workbench is back after a reload', !!page);
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await settle(2200);
    report('the hidden layer and the group are kept', (await page.locator('.arcviz-stage .av-group').count()) === 1 && (await page.locator('.arcviz-stage [data-id="LC-ACT"]').count()) === 0 && layout() === keptLayout);
    await shot('layers-07-reloaded');
  } catch (error) {
    report('the run completes', false, String(error).slice(0, 300)); await shot('layers-failure').catch(() => {});
  } finally {
    await app.close().catch(() => {});
    const failed = rows.filter((r) => !r.ok);
    console.log(`\n${rows.length - failed.length}/${rows.length} passed`);
    process.exit(failed.length ? 1 : 0);
  }
})();

/** Export is in the toolbar, or in its "⋯" menu when the row is too narrow for it (the entries are the same). */
async function openExport(page) {
  const direct = page.locator('#arcviz-export-menu');
  if (await direct.isVisible().catch(() => false)) await direct.click();
  else await page.locator('#arcviz-more').click();
}
