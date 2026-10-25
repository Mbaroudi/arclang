// draw.io's Arrange in the browser, through the real compiler: the stacking
// order (front, back, forward, backward) from the menu, Layout ▾ and the
// keys; groups (frame, move, ungroup, collapse, into a new component);
// locks; layers (new, move the selection to one, hide, lock, reorder); same
// size; clear waypoints; select lines and boxes. SHOTS=dir saves pictures.
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const path = require('path');
const BASE = `http://127.0.0.1:${process.env.PORT || 8766}`;
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok, detail }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(62)} ${detail || ''}`); };
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  const errors = []; page.on('pageerror', (e) => errors.push(String(e))); page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  await page.goto(`${BASE}/e2e.html`); await page.evaluate(() => localStorage.clear()); await page.evaluate(() => fetch('/reset')); await page.reload(); await page.waitForSelector('.arcviz-stage svg', { timeout: 30000 }); await page.waitForTimeout(600);
  const shot = async (name) => { if (SHOTS) await page.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const source = () => page.evaluate(() => fetch('/source').then((r) => r.text()));
  const count = () => page.evaluate(() => window.__results.length);
  const waitEdit = async (before) => { for (let i = 0; i < 60 && (await count()) === before; i++) await page.waitForTimeout(150); await page.waitForTimeout(1200); return (await count()) === before ? null : page.evaluate(() => window.__results.at(-1)); };
  const lastLayout = () => page.evaluate(() => window.__posted.filter((m) => m.type === 'layout').map((m) => m.file).at(-1) || {});
  const node = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first();
  const shape = async (id) => ((await node(id).count()) ? node(id).locator(':scope > .av-shape').first().boundingBox() : null);
  const order = () => page.evaluate(() => [...document.querySelectorAll('.arcviz-stage .av-node')].map((n) => n.dataset.id));
  const above = async (a, b) => { const o = await order(); return o.indexOf(a) > o.indexOf(b); };
  const click = async (id, options = {}) => { const b = await shape(id); for (const m of options.modifiers || []) await page.keyboard.down(m); await page.mouse.click(b.x + b.width - 30, b.y + 8); for (const m of options.modifiers || []) await page.keyboard.up(m); await page.waitForTimeout(350); };
  const blank = async () => { const st = await page.locator('.arcviz-stage').boundingBox(); await page.mouse.click(st.x + st.width - 20, st.y + st.height - 20); await page.waitForTimeout(250); };
  const drag = async (from, to) => { await page.mouse.move(from.x, from.y); await page.mouse.down(); await page.mouse.move(to.x, to.y, { steps: 14 }); await page.mouse.up(); await page.waitForTimeout(900); };
  const menu = async (id) => { const b = await shape(id); await page.mouse.click(b.x + b.width - 30, b.y + 8, { button: 'right' }); await page.waitForTimeout(400); };
  const menuItem = (label) => page.locator('.wb-menu:not([hidden]) button', { hasText: label }).first();
  const layoutItem = async (action) => { await page.locator('#arcviz-layout-menu').click(); await page.waitForTimeout(200); await page.locator(`.arcviz-menu-pop:not([hidden]) [data-action="${action}"]`).click(); await page.waitForTimeout(900); };
  const press = async (keys) => { await page.keyboard.press(keys); await page.waitForTimeout(900); };
  const toast = () => page.locator('.wb-toast:not([hidden])').textContent().catch(() => '');
  const fit = async () => { await page.locator('#arcviz-fit').click(); await page.waitForTimeout(900); };
  const prompt = async (text) => { const i = page.locator('.wb-prompt input').first(); await i.waitFor({ timeout: 3000 }); await i.fill(text); await i.press('Enter'); };

  await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await page.waitForTimeout(1200);
  // Two boxes that overlap: the actuator dragged over the camera.
  let act = await shape('LC-ACT'); const cam0 = await shape('LC-CAM');
  await drag({ x: act.x + act.width - 12, y: act.y + 8 }, { x: act.x + act.width - 12 - (act.x - cam0.x - 90), y: act.y + 8 - (act.y - cam0.y - 30) });
  act = await shape('LC-ACT');
  const overlapping = act.x < cam0.x + cam0.width && act.x + act.width > cam0.x && act.y < cam0.y + cam0.height && act.y + act.height > cam0.y;
  report('two boxes overlap, the later drawn on top', overlapping && (await above('LC-ACT', 'LC-CAM')), JSON.stringify({ act, cam0 }));
  await shot('arrange-00-overlap');

  // 1. Send to back from the context menu
  await menu('LC-ACT'); await menuItem('To back').click(); await page.waitForTimeout(900);
  report('Order > To back draws it under the box it overlaps', !(await above('LC-ACT', 'LC-CAM')), (await order()).join(','));
  report('the stacking is kept as order in the layout file', JSON.stringify((await lastLayout()).arrangements || []).includes('"order"'));
  // 2. Forward by its key (outside VS Code the page takes it)
  await click('LC-ACT'); await press('Meta+Alt+Shift+KeyF');
  report('alt-shift-cmd-F brings it one step forward, past the box it overlaps', await above('LC-ACT', 'LC-CAM'), (await order()).join(','));
  // 3. Backward from Layout, then to front and to back by their keys
  await click('LC-ACT'); await layoutItem('arrange-send-backward');
  report('Layout > Send backward takes it one step back', !(await above('LC-ACT', 'LC-CAM')));
  await click('LC-ACT'); await press('Meta+Shift+KeyF');
  report('shift-cmd-F brings it to the front', await above('LC-ACT', 'LC-CAM'));
  await click('LC-ACT'); await press('Meta+Shift+KeyB');
  report('shift-cmd-B sends it to the back', !(await above('LC-ACT', 'LC-CAM')));
  await click('LC-ACT'); await page.waitForTimeout(500);
  const mini = await page.locator('.wb-mini:not([hidden]) .wb-mini-button[aria-label^="Order"]').count();
  report('the floating toolbar has one Order button', mini === 1, `${mini}`);
  await shot('arrange-01-order');

  await fit();
  // 4. Group the camera and the controller; the group moves as one; ungroup
  await blank(); await click('LC-CAM'); await click('LC-CTL', { modifiers: ['Shift'] });
  await press('Meta+g');
  const frame = page.locator('.arcviz-stage .av-group').first();
  report('cmd-G frames the two boxes in a group', (await frame.count()) === 1 && /"groups"/.test(JSON.stringify(await lastLayout())), JSON.stringify((await lastLayout()).groups || {}));
  await shot('arrange-02-grouped');
  const f = await frame.locator(':scope > .av-shape').boundingBox(); const c1 = await shape('LC-CAM'); const t1 = await shape('LC-CTL');
  await drag({ x: f.x + f.width / 2, y: f.y + 6 }, { x: f.x + f.width / 2 + 40, y: f.y + 6 + 30 });
  const c2 = await shape('LC-CAM'); const t2 = await shape('LC-CTL');
  report('the group moves its members together', Math.abs((c2.x - c1.x) - (t2.x - t1.x)) < 2 && c2.y - c1.y > 15, `${Math.round(c2.x - c1.x)},${Math.round(c2.y - c1.y)} / ${Math.round(t2.x - t1.x)},${Math.round(t2.y - t1.y)}`);
  await click('LC-CAM');
  const picked = await page.evaluate(() => document.querySelector('.arcviz-stage .is-selected')?.dataset.id);
  report('a click on a member selects its group', /^group:/.test(picked || ''), picked);
  await menu('LC-CAM'); await menuItem('Collapse group').click(); await page.waitForTimeout(1000);
  report('a collapsed group is one box, its members not drawn', (await page.locator('.av-group.is-collapsed').count()) === 1 && (await node('LC-CAM').count()) === 0);
  await shot('arrange-03-collapsed');
  await page.locator('.av-group.is-collapsed').dblclick(); await page.waitForTimeout(1000);
  report('a double-click expands it', (await node('LC-CAM').count()) === 1);
  await click('LC-CAM'); await press('Meta+Shift+KeyG');
  report('shift-cmd-G ungroups', (await page.locator('.arcviz-stage .av-group').count()) === 0 && !JSON.stringify(await lastLayout()).includes('"groups"'));

  // 5. Lock: a drag does nothing, Delete is refused with the reason
  await blank(); await click('LC-CAM'); await press('Meta+l');
  report('cmd-L locks the box: a padlock on it', (await page.locator('.arcviz-stage [data-id="LC-CAM"].is-locked .av-lock').count()) === 1 && JSON.stringify(await lastLayout()).includes('"locked"'));
  const rel = async (id, to) => { const a = await shape(id); const b = await shape(to); return { x: a.x - b.x, y: a.y - b.y }; };
  const l0 = await rel('LC-CAM', 'LC-CTL'); const lb = await shape('LC-CAM');
  await drag({ x: lb.x + lb.width - 30, y: lb.y + 8 }, { x: lb.x + lb.width + 30, y: lb.y + 60 });
  const l1 = await rel('LC-CAM', 'LC-CTL');
  report('a locked box does not move (the others stay with it)', Math.abs(l1.x - l0.x) < 1 && Math.abs(l1.y - l0.y) < 1);
  const before = await source(); await click('LC-CAM'); await press('Delete');
  report('Delete on a locked box is refused, with the reason', (await source()) === before && /locked/.test(await toast()), await toast());
  await click('LC-CAM'); await press('Meta+l');
  report('cmd-L again unlocks it', (await page.locator('.arcviz-stage .is-locked').count()) === 0);

  // 6. Layers: a new one, the selection moved onto it (its tag), hidden, locked, ordered
  await blank(); await press('Meta+Shift+KeyL');
  report('shift-cmd-L opens the Layers panel with Default', (await page.locator('.wb-tl:not([hidden]) .wb-tl-row[data-layer=""]').count()) === 1);
  await page.locator('.wb-tl button', { hasText: 'New layer' }).click(); await prompt('Safety');
  await click('LC-ACT');
  let n = await count(); await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-move').click();
  const moved = await waitEdit(n); const tagged = await source();
  report('Move the selection to a layer sets its tag (one edit)', !!moved?.ok && /id: "LC-ACT"[\s\S]{0,200}tags: \["Safety"\]/.test(tagged), moved?.label || moved?.error || '');
  await page.waitForTimeout(600);
  const counted = await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-count').textContent();
  report('the layer counts its elements', counted === '1', counted);
  await shot('arrange-04-layers');
  await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-eye').click(); await page.waitForTimeout(1200);
  report('a hidden layer: its box and its lines are not drawn', (await node('LC-ACT').count()) === 0 && (await node('ce:SteerCommand').count()) === 0 && (await node('LC-CAM').count()) === 1);
  const exportTitle = await (async () => { await page.locator('#arcviz-export-menu').click(); await page.waitForTimeout(200); const t = await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="save-png"]').getAttribute('title'); await page.keyboard.press('Escape'); return t; })();
  report('the export menu says hidden layers are left out', /hidden layers are left out/.test(exportTitle || ''), exportTitle);
  await shot('arrange-05-hidden');
  await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-eye').click(); await page.waitForTimeout(1200);
  await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-lock').click(); await page.waitForTimeout(900);
  const a0 = await rel('LC-ACT', 'LC-CTL'); const ab = await shape('LC-ACT'); await drag({ x: ab.x + ab.width - 30, y: ab.y + 8 }, { x: ab.x + ab.width + 50, y: ab.y + 70 }); const a1 = await rel('LC-ACT', 'LC-CTL');
  report('a box on a locked layer does not move', Math.abs(a1.x - a0.x) < 1 && Math.abs(a1.y - a0.y) < 1);
  await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-lock').click(); await page.waitForTimeout(900);
  await click('LC-ACT'); await press('Meta+Shift+KeyF');
  const onTop = await above('LC-ACT', 'LC-CAM');
  await page.locator('.wb-tl-row[data-layer="Safety"] button[aria-label^="Move Safety down"]').click(); await page.waitForTimeout(1200);
  report('a layer moved under Default draws its box under the others', onTop && !(await above('LC-ACT', 'LC-CAM')) && JSON.stringify((await lastLayout()).layers || {}).includes('"order"'), JSON.stringify((await lastLayout()).layers));
  await page.locator('.wb-tl button[title^="Close"]').click();

  await fit();
  // 7. Same size as the last selected; clear waypoints; select lines and boxes
  await blank(); await click('LC-CAM'); await click('LC-CTL', { modifiers: ['Shift'] });
  await menu('LC-CTL');
  await page.locator('.wb-menu:not([hidden]) .wb-menu-row[aria-label="Same"] button', { hasText: 'Size' }).click(); await page.waitForTimeout(1100);
  const sc = await shape('LC-CAM'); const st = await shape('LC-CTL');
  report('Same size gives the first the size of the last selected', Math.abs(sc.width - st.width) < 2 && Math.abs(sc.height - st.height) < 2, `${Math.round(sc.width)}x${Math.round(sc.height)} vs ${Math.round(st.width)}x${Math.round(st.height)}`);
  await blank();
  const line = await node('ce:LaneFlow').evaluate((g) => { const l = g.querySelector('.av-line'); const p = l.getPointAtLength(l.getTotalLength() / 2); const m = l.getScreenCTM(); return { x: p.x * m.a + p.y * m.c + m.e, y: p.x * m.b + p.y * m.d + m.f }; });
  await page.mouse.click(line.x, line.y); await page.waitForTimeout(500);
  const diamond = page.locator('.arcviz-bend.is-new').first();
  if (await diamond.isVisible().catch(() => false)) { const d = await diamond.boundingBox(); await drag({ x: d.x + d.width / 2, y: d.y + d.height / 2 }, { x: d.x + d.width / 2, y: d.y + d.height / 2 - 70 }); await page.waitForTimeout(800); }
  const bent = JSON.stringify(await lastLayout()).includes('"points"');
  await page.waitForTimeout(500);
  await layoutItem('arrange-clear-waypoints');
  report('Clear waypoints straightens the selected line', bent && !JSON.stringify(await lastLayout()).includes('"points"'), `bent first: ${bent}`);
  await blank(); await press('Meta+Shift+KeyE');
  const lines = await page.locator('.av-edge.wb-multi').count();
  report('shift-cmd-E selects every line (2)', lines === 2 && /2 lines selected/.test(await toast()), `${lines}`);
  await press('Meta+Shift+KeyI');
  report('shift-cmd-I selects every box', /6 boxes selected/.test(await toast()), await toast());
  await press('Meta+Shift+KeyA');
  report('shift-cmd-A selects nothing', (await page.locator('.wb-multi').count()) === 0);

  await fit();
  // 8. Group into a new component: the model gains a component holding both
  await blank(); await click('LC-CAM'); await click('LC-CTL', { modifiers: ['Shift'] });
  await menu('LC-CTL'); n = await count(); await menuItem('Group into a new component').click(); await prompt('Sensing');
  const grouped = await waitEdit(n); const text = await source();
  const block = (text.split('component "Sensing"')[1] || '').split('component "Steering Actuator"')[0];
  report('Group into a new component: one edit, both inside it', !!grouped?.ok && /component "Camera"/.test(block) && /component "LKA Controller"/.test(block), grouped?.error || grouped?.label || '');
  await shot('arrange-06-into-component');

  report('no console error', errors.length === 0, errors.slice(0, 3).join(' | '));
  await browser.close();
  const failed = rows.filter((r) => !r.ok);
  console.log(`\n${rows.length - failed.length}/${rows.length} passed`);
  process.exit(failed.length ? 1 : 0);
})();
