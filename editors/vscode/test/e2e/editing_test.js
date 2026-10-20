// Changing what is already drawn, through the real compiler: reconnect,
// reverse, rename and edit an exchange; move a port; copy, cut, paste,
// duplicate; align, distribute, nudge, bend, snap; band selection; quick
// connect; the context menu, the floating toolbar and the shortcuts list.
// Every model change is one edit (one undo); refusals show their reason.
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const path = require('path');
const BASE = `http://127.0.0.1:${process.env.PORT || 8766}`;
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok, detail }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(58)} ${detail || ''}`); };
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  const errors = []; page.on('pageerror', (e) => errors.push(String(e))); page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  await page.goto(`${BASE}/e2e.html`); await page.evaluate(() => localStorage.clear()); await page.reload(); await page.waitForSelector('.arcviz-stage svg', { timeout: 30000 }); await page.waitForTimeout(600);
  const shot = async (name) => { if (SHOTS) await page.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const snapshot = () => page.evaluate(() => fetch('/snapshot').then((r) => r.json()));
  const source = () => page.evaluate(() => fetch('/source').then((r) => r.text()));
  const reset = async () => { await page.evaluate(() => fetch('/reset')); await page.evaluate(() => window.__push()); await page.waitForTimeout(1000); await page.keyboard.press('Escape'); };
  // Boxes back where the automatic layout puts them, so each check starts from the same drawing.
  const resetLayout = async () => { const b = page.locator('.wb-viewer-tools button', { hasText: 'Reset layout' }).first(); if (await b.count() && await b.isEnabled()) { await b.click(); await page.waitForTimeout(900); } };
  const count = () => page.evaluate(() => window.__results.length);
  const last = () => page.evaluate(() => window.__results.at(-1));
  const waitEdit = async (before) => { for (let i = 0; i < 60 && (await count()) === before; i++) await page.waitForTimeout(150); await page.waitForTimeout(900); return (await count()) === before ? null : last(); };
  const layouts = () => page.evaluate(() => window.__posted.filter((m) => m.type === 'layout').map((m) => m.file));
  const layer = async (label) => { await page.locator('.wb-layer', { hasText: label }).first().click(); await page.waitForTimeout(900); await resetLayout(); };
  const show = async (view) => { await page.evaluate((id) => window.postMessage({ type: 'show', view: id, select: null }, '*'), view); await page.waitForTimeout(1200); };
  const node = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first();
  const box = async (id) => ((await node(id).count()) ? node(id).locator(':scope > .av-shape').first().boundingBox() : null);
  const click = async (id, options = {}) => { const b = await box(id); if (!b) return false; await page.mouse.click(b.x + Math.min(30, b.width / 2), b.y + Math.min(10, b.height / 2), options); await page.waitForTimeout(300); return true; };
  const blank = async () => { const st = await page.locator('.arcviz-stage').boundingBox(); await page.mouse.click(st.x + 6, st.y + st.height - 6); await page.waitForTimeout(250); };
  const prompt = async (text) => { const i = page.locator('.wb-prompt input').first(); await i.waitFor({ timeout: 3000 }); if (text !== undefined) await i.fill(text); await i.press('Enter'); };
  const pick = async (text) => { const s = page.locator('.wb-pick select').first(); await s.waitFor({ timeout: 3000 }); const value = await s.evaluate((sel, t) => [...sel.options].find((o) => o.text.includes(t))?.value ?? null, text); if (value === null) return false; await s.selectOption(value); return true; };
  const edgePoint = (id, at) => node(id).evaluate((g, where) => { const lines = [...g.querySelectorAll('.av-line')]; const line = where === 'end' ? lines.at(-1) : lines[0]; const len = line.getTotalLength(); const p = line.getPointAtLength(where === 'start' ? 0 : where === 'end' ? len : len / 2); const m = line.getScreenCTM(); return { x: p.x * m.a + p.y * m.c + m.e, y: p.x * m.b + p.y * m.d + m.f }; }, at);
  const selectEdge = async (id) => { if (!(await node(id).count())) return false; const p = await edgePoint(id, 'middle'); await page.mouse.click(p.x, p.y); await page.waitForTimeout(350); return true; };
  const drag = async (from, to, options = {}) => { await page.mouse.move(from.x, from.y); await page.mouse.down(); await page.mouse.move(to.x, to.y, { steps: 12 }); if (options.during) await options.during(); await page.mouse.up(); await page.waitForTimeout(400); };
  const centreOf = (b) => ({ x: b.x + b.width / 2, y: b.y + b.height / 2 });
  const handle = async (end) => { const h = page.locator(`.arcviz-end[data-end="${end}"]`); return (await h.isVisible().catch(() => false)) ? centreOf(await h.boundingBox()) : null; };
  let shotTaken = false;
  const reconnectByDrag = async (edge, end, targetId) => {
    if (!(await selectEdge(edge))) return { error: `edge ${edge} not found` };
    const from = await handle(end); if (!from) return { error: 'no end handle' };
    const target = await box(targetId); if (!target) return { error: `${targetId} not drawn` };
    const n = await count(); let marked = [];
    await drag(from, { x: target.x + target.width / 2, y: target.y + Math.min(14, target.height / 2) }, { during: async () => { marked = await page.$$eval('.is-drop-target', (ns) => ns.map((x) => x.dataset.id)); if (!shotTaken) { shotTaken = true; await shot('reconnect-handles'); } } });
    return { result: await waitEdit(n), marked };
  };
  const menu = async (x, y) => { await page.mouse.click(x, y, { button: 'right' }); await page.waitForTimeout(300); return page.locator('.wb-menu:not([hidden])'); };
  const menuItem = (label) => page.locator('.wb-menu:not([hidden]) button', { hasText: label }).first();

  let r = {};
  // 1. Reconnect a component exchange by dragging its target end onto a box: a port is made there, one edit
  await reset(); r = {}; await layer('Logical');
  r = await reconnectByDrag('ce:LaneFlow', 'target', 'LC-ACT'); let text = await source();
  await selectEdge('ce:SteerCommand'); await shot('edge-selected');
  report('drag the end of a component exchange to a box reconnects it', !!r.result?.ok && /to_port: "LC-ACT\.lane"/.test(text) && /port in "lane"/.test(text.split('component "Steering Actuator"')[1] || '') && r.result.requests.length === 2, r.error || r.result?.error || '');
  report('the box it would land on is marked during the drag', (r.marked || []).includes('LC-ACT'), (r.marked || []).join(','));

  // 2. Functional exchange: the source end moved to another function's port
  await reset(); r = {}; await layer('System');
  r = await reconnectByDrag('fe:lane model', 'target', 'SF-CMD'); text = await source();
  report('drag the end of a functional exchange reconnects it', !!r.result?.ok && /to: "SF-CMD\.lane"/.test(text), r.error || r.result?.error || '');

  // 3. Interaction of the operational analysis
  await reset(); r = {}; await layer('Operational');
  r = await reconnectByDrag('int:Steering input', 'target', 'OA-VEH'); text = await source();
  report('drag the end of an interaction reconnects it', !!r.result?.ok && /interaction "Steering input" \{\s*from: "OA-DRV"\s*to: "OA-VEH"/.test(text), r.error || r.result?.error || '');

  // 4. A transition of a state machine (written on its line)
  await reset(); r = {}; await show('msm:LKA modes');
  r = await reconnectByDrag('tr:Off -> Armed', 'target', 'LKA modes.Correcting'); text = await source();
  report('drag the end of a transition reconnects it', !!r.result?.ok && /transition Off -> Correcting \{ trigger: "SF-DET" \}/.test(text), r.error || r.result?.error || '');

  // 5. A message of a scenario, to another lifeline
  await reset(); r = {}; await show('es:Lane departure');
  r = await reconnectByDrag('msg:1', 'target', 'Lane departure.LC-ACT'); text = await source();
  report('drag the end of a message reconnects it', !!r.result?.ok && /message "LC-CAM" -> "LC-ACT" "lane model"/.test(text), r.error || r.result?.error || '');

  // 6. A physical link reversed from the floating toolbar; a component exchange from the properties
  await reset(); r = {}; await layer('Physical'); await selectEdge('pl:ChassisCAN');
  let n = await count(); const reverseButton = page.locator('.wb-mini:not([hidden]) button[aria-label="Reverse"]');
  const hasMini = await reverseButton.count(); if (hasMini) { await reverseButton.click(); r = { result: await waitEdit(n) }; }
  text = await source();
  report('Reverse in the floating toolbar swaps a link\'s ends', hasMini > 0 && !!r.result?.ok && /from: "PN-CHS"\s*to: "PN-CAM"/.test(text), hasMini ? r.result?.error : 'no toolbar');
  await reset(); r = {}; await layer('Logical'); await selectEdge('ce:SteerCommand'); n = await count();
  await page.locator('.wb-ends button', { hasText: 'Reverse' }).click(); r = { result: await waitEdit(n) }; text = await source();
  report('Reverse in the properties swaps an exchange\'s ends', !!r.result?.ok && /from_port: "LC-ACT\.steer"\s*to_port: "LC-CTL\.steer"/.test(text), r.result?.error);

  // 7. A trace retargeted from the Relations pane (traces are not drawn as lines here)
  await reset(); r = {}; await layer('Logical'); await click('LC-CAM');
  await page.locator('.wb-right-tab', { hasText: 'Relations' }).click(); await page.waitForTimeout(200);
  const retarget = page.locator('.wb-retarget').first(); n = await count();
  if (await retarget.count()) { await retarget.click(); await pick('REQ-LKA-002'); r = { result: await waitEdit(n) }; }
  text = await source(); await page.locator('.wb-right-tab', { hasText: 'Properties' }).click();
  report('a trace is retargeted, its rationale kept', !!r.result?.ok && /trace "LC-CAM" satisfies "REQ-LKA-002" \{ rationale: "Delivers the lane model/.test(text), r.result ? r.result.error : 'no retarget button');

  // 8. Rename an exchange in place: double-click its label; F2 on a selected message
  await reset(); r = {}; await layer('Logical');
  const label = node('ce:LaneFlow').locator('.av-edge-label');
  const lb = await label.boundingBox(); await page.mouse.dblclick(lb.x + lb.width / 2, lb.y + lb.height / 2); await page.waitForTimeout(300);
  let field = page.locator('.wb-direct-edit'); let hasField = await field.count(); n = await count();
  if (hasField) { await field.fill('Lane data'); await field.press('Enter'); r = { result: await waitEdit(n) }; }
  text = await source();
  report('double-click on an exchange label renames it in place', hasField > 0 && !!r.result?.ok && /component_exchange "Lane data" \{/.test(text), hasField ? r.result?.error : 'no field');
  await reset(); r = {}; await show('es:Lane departure'); await selectEdge('msg:2'); await page.keyboard.press('F2');
  field = page.locator('.wb-direct-edit'); hasField = await field.count(); n = await count();
  if (hasField) { await field.fill('torque request'); await field.press('Enter'); r = { result: await waitEdit(n) }; }
  text = await source();
  report('F2 on a selected message renames it', hasField > 0 && !!r.result?.ok && /message "LC-CTL" -> "LC-ACT" "torque request"/.test(text), hasField ? r.result?.error : 'no field');

  // 9. Double-click on a box renames it (no more jump to the source)
  await reset(); r = {}; await layer('Logical'); { const b = await box('LC-ACT'); await page.mouse.dblclick(b.x + b.width / 2, b.y + 12); await page.waitForTimeout(300); }
  field = page.locator('.wb-direct-edit'); hasField = await field.count(); n = await count();
  if (hasField) { await field.fill('Steering motor'); await field.press('Enter'); r = { result: await waitEdit(n) }; }
  text = await source(); report('double-click on a box renames it in place', hasField > 0 && !!r.result?.ok && /component "Steering motor"/.test(text), hasField ? r.result?.error : 'no field');

  // 10. An attribute of an exchange from the properties
  await reset(); r = {}; await layer('Physical'); await selectEdge('pl:ChassisCAN');
  const protocol = page.locator('table.wb-props tr', { hasText: 'Protocol' }).locator('input').first(); n = await count();
  if (await protocol.count()) { await protocol.fill('Ethernet'); await protocol.press('Enter'); r = { result: await waitEdit(n) }; }
  text = await source();
  report('the properties of a link edit its attributes', !!r.result?.ok && /protocol: "Ethernet"/.test(text), (await protocol.count()) ? r.result?.error : `no Protocol row: ${(await page.locator('.wb-properties').textContent()).slice(0, 120)}`);

  // 11. A port moved to another component with Move to…, its exchange following
  await reset(); r = {}; await layer('Logical');
  { const port = await node('LC-CAM::lane').boundingBox(); const m = await menu(port.x + port.width / 2, port.y + port.height / 2); await m.waitFor(); }
  await menuItem('Move to').click(); n = await count(); const picked = await pick('Steering Actuator'); r = { result: picked ? await waitEdit(n) : null }; text = await source();
  report('Move to… moves a port, its exchange follows', !!r.result?.ok && /from_port: "LC-ACT\.lane"/.test(text) && /port out "lane"/.test(text.split('component "Steering Actuator"')[1] || ''), picked ? r.result?.error : 'no choice');

  // 12. Copy then paste into another component; cut then paste is a move; duplicate
  await reset(); r = {}; await layer('Logical'); await click('LF-CAP'); await page.keyboard.press('Meta+c'); await click('LC-CTL'); n = await count(); await page.keyboard.press('Meta+v'); r = { result: await waitEdit(n) }; text = await source();
  report('⌘C ⌘V pastes a copy into the selected box, ids made unique', !!r.result?.ok && /id: "LF-CAP-2"/.test(text.split('component "LKA Controller"')[1] || '') && /id: "LF-CAP"\n/.test(text), r.result?.error);
  await reset(); r = {}; await layer('Logical'); await click('LF-CAP'); await page.keyboard.press('Meta+x'); await click('LC-ACT'); n = await count(); await page.keyboard.press('Meta+v'); r = { result: await waitEdit(n) }; text = await source();
  report('⌘X ⌘V moves the element into the selected box', !!r.result?.ok && text.indexOf('function "Capture frame"') > text.indexOf('component "Steering Actuator"') && (text.match(/Capture frame/g) || []).length === 1, r.result?.error);
  await reset(); r = {}; await layer('Logical'); await click('LC-ACT'); n = await count(); await page.keyboard.press('Meta+d'); r = { result: await waitEdit(n) }; text = await source();
  report('⌘D duplicates with new ids', !!r.result?.ok && /id: "LC-ACT-2"/.test(text) && /id: "LF-ACT-2"/.test(text), r.result?.error);

  // 13. A refused gesture says why: a component cut and pasted into a function
  await reset(); r = {}; await layer('Logical'); await click('LC-CAM'); await page.keyboard.press('Meta+x'); await click('LF-ACT'); n = await count(); await page.keyboard.press('Meta+v'); r = { result: await waitEdit(n) };
  const toastText = await page.locator('.wb-toast').textContent().catch(() => '');
  report('a refused paste is refused with the compiler\'s reason', r.result && !r.result.ok && /Refused/.test(toastText || '') && (toastText || '').length > 20, (toastText || '').slice(0, 90));

  // 14. Band selection, then align left and distribute (layout only, through the layout file)
  await reset(); r = {}; await layer('Logical'); await blank();
  const stage = await page.locator('.arcviz-stage').boundingBox();
  const all = await Promise.all(['LC-CAM', 'LC-CTL', 'LC-ACT'].map(box));
  const band = { x1: Math.min(...all.map((b) => b.x)) - 20, y1: Math.min(...all.map((b) => b.y)) - 20, x2: Math.max(...all.map((b) => b.x + b.width)) + 20, y2: Math.max(...all.map((b) => b.y + b.height)) + 20 };
  await drag({ x: Math.max(stage.x + 2, band.x1), y: Math.max(stage.y + 2, band.y1) }, { x: band.x2, y: band.y2 });
  const bar = await page.locator('.wb-multibar').textContent().catch(() => '');
  report('a drag on the empty sheet selects the boxes it encloses', /3 selected/.test(bar || ''), bar || 'no bar');
  let before = (await layouts()).length;
  { const b = await box('LC-ACT'); const m = await menu(b.x + 20, b.y + 10); await m.waitFor(); }
  await shot('context-menu-many');
  await menuItem('Top').click(); await page.waitForTimeout(1500);
  const tops0 = await Promise.all(['LC-CAM', 'LC-CTL', 'LC-ACT'].map(async (id) => Math.round((await box(id)).y)));
  report('Align top lines the selection up, kept in the layout file', new Set(tops0).size === 1 && (await layouts()).length > before, tops0.join(','));
  before = (await layouts()).length;
  { const b = await box('LC-ACT'); const m = await menu(b.x + 20, b.y + 10); await m.waitFor(); }
  await menuItem('Horizontally').click(); await page.waitForTimeout(1500);
  await shot('align-distribute');
  const row = await Promise.all(['LC-CAM', 'LC-CTL', 'LC-ACT'].map(box)); row.sort((a, b) => a.x - b.x);
  const gaps = [row[1].x - row[0].x - row[0].width, row[2].x - row[1].x - row[1].width];
  report('Distribute horizontally leaves equal gaps', Math.abs(gaps[0] - gaps[1]) < 3 && (await layouts()).length > before, gaps.map(Math.round).join(','));
  await page.keyboard.press('Escape');

  // 15. Arrow keys nudge the selection (1, Shift: 10), in the layout file
  await reset(); r = {}; await layer('Logical'); await click('LC-ACT'); const at0 = await box('LC-ACT');
  const placeOf = (file) => file?.arrangements?.find((a) => a.view === 'lab' && !a.open)?.places?.['LC-ACT'] || { dx: 0, dy: 0 };
  const was = placeOf((await layouts()).at(-1));
  for (let i = 0; i < 3; i++) { await page.keyboard.press('ArrowRight'); await page.waitForTimeout(250); }
  await page.keyboard.press('Shift+ArrowDown'); await page.waitForTimeout(800);
  const at1 = await box('LC-ACT'); const scale = await page.evaluate(() => Number(/scale\(([\d.]+)\)/.exec(document.querySelector('.arcviz-canvas').style.transform)?.[1] || 1));
  const place = placeOf((await layouts()).at(-1)); const moved = { dx: Math.round(place.dx - was.dx), dy: Math.round(place.dy - was.dy) };
  report('arrow keys move the selection by 1, Shift+arrow by 10', moved.dx === 3 && moved.dy === 10 && Math.abs((at1.x - at0.x) / scale - 3) < 1.5, JSON.stringify(moved));

  // 16. Smart guides while dragging a box; Alt moves freely
  await reset(); r = {}; await layer('Logical'); const cam = await box('LC-CAM'); const act = await box('LC-ACT');
  let guide = false;
  await drag({ x: act.x + 20, y: act.y + 10 }, { x: act.x + 20 + (cam.x - act.x) + 3, y: act.y + 40 }, { during: async () => { guide = await page.locator('.arcviz-guide.is-vertical:not([hidden])').count() > 0; } });
  await page.waitForTimeout(1200); const snapped = await box('LC-ACT');
  report('a dragged box snaps to the edge of another, with a guide', guide && Math.abs(snapped.x - cam.x) < 1.5, `guide ${guide}, ${Math.round(snapped.x - cam.x)} px off`);

  // 17. Bend an exchange by its middle, kept in the layout file; double-click the bend removes it
  await reset(); r = {}; await layer('Logical'); await selectEdge('ce:SteerCommand');
  const diamond = page.locator('.arcviz-bend.is-new').first(); let bent = false; let removed = false;
  if (await diamond.isVisible().catch(() => false)) {
    const d = centreOf(await diamond.boundingBox()); await drag(d, { x: d.x, y: d.y + 70 }); await page.waitForTimeout(1200);
    const file = (await layouts()).at(-1); bent = !!file?.arrangements?.some((a) => a.edges?.['ce:SteerCommand']?.points?.length === 1);
    await selectEdge('ce:SteerCommand'); const waypoint = page.locator('.arcviz-bend.is-waypoint').first();
    if (await waypoint.isVisible().catch(() => false)) { const w = centreOf(await waypoint.boundingBox()); await page.mouse.dblclick(w.x, w.y); await page.waitForTimeout(1200); removed = !(await layouts()).at(-1)?.arrangements?.some((a) => a.edges?.['ce:SteerCommand']); }
  }
  report('dragging the middle of an exchange bends it, kept in the layout', bent);
  report('double-click on a bend removes it', removed);

  // 18. Quick connect: an arrow on the side of a box; dragged it connects, clicked it creates and connects
  await reset(); r = {}; await layer('Logical'); await page.mouse.move(5, 5);
  { const b = await box('LC-CAM'); await page.mouse.move(b.x + b.width / 2, b.y + 10); await page.waitForTimeout(300); }
  let arrow = page.locator('.arcviz-quick.is-east'); let arrowShown = await arrow.isVisible().catch(() => false);
  await shot('quick-connect');
  if (arrowShown) { const a = centreOf(await arrow.boundingBox()); const t = await box('LC-ACT'); n = await count(); await drag(a, { x: t.x + t.width / 2, y: t.y + 12 }); if (await page.locator('.wb-prompt input').count()) { await prompt('Camera to actuator'); r = { result: await waitEdit(n) }; } }
  text = await source();
  report('dragging the arrow of a box connects it', arrowShown && !!r.result?.ok && /component_exchange "Camera to actuator"/.test(text), arrowShown ? r.result?.error : 'no arrow');
  await reset(); r = {}; await layer('Logical'); await page.mouse.move(5, 5);
  { const b = await box('LC-ACT'); await page.mouse.move(b.x + b.width / 2, b.y + 10); await page.waitForTimeout(300); }
  arrow = page.locator('.arcviz-quick.is-south'); arrowShown = await arrow.isVisible().catch(() => false);
  if (arrowShown) { const a = centreOf(await arrow.boundingBox()); n = await count(); await page.mouse.click(a.x, a.y); await prompt('Torque sensor'); r = { result: await waitEdit(n) }; }
  text = await source();
  report('clicking the arrow creates a connected element, one edit', arrowShown && !!r.result?.ok && /component "Torque sensor" \{\s*id: "LC-1"/.test(text) && /component_exchange "Steering Actuator to Torque sensor"/.test(text), arrowShown ? r.result?.error : 'no arrow');

  // 19. The context menu of an exchange and of a box; the shortcuts list
  await reset(); r = {}; await layer('Logical');
  { const p = await edgePoint('ce:LaneFlow', 'middle'); const m = await menu(p.x, p.y); await m.waitFor(); }
  const edgeItems = await page.locator('.wb-menu:not([hidden]) .wb-menu-item').allTextContents();
  await shot('context-menu-edge');
  report('right-click on an exchange lists rename, reconnect, reverse, delete', ['Rename', 'Reconnect the source', 'Reconnect the target', 'Reverse', 'Go to source', 'Delete'].every((w) => edgeItems.some((t) => t.includes(w))), edgeItems.join(' | '));
  await menuItem('Reconnect the target').click(); n = await count(); const chosen = await pick('LC-ACT'); r = { result: chosen ? await waitEdit(n) : null }; text = await source();
  report('Reconnect the target… from the menu, chosen in a list', !!r.result?.ok && /to_port: "LC-ACT\.lane"/.test(text), chosen ? r.result?.error : 'no choice');
  await reset(); r = {}; await layer('Logical');
  { const b = await box('LC-CTL'); const m = await menu(b.x + b.width / 2, b.y + 12); await m.waitFor(); }
  const boxItems = await page.locator('.wb-menu:not([hidden]) .wb-menu-item').allTextContents();
  await shot('context-menu');
  report('right-click on a box lists its actions', ['Rename', 'Move to', 'Hide', 'Copy', 'Cut', 'Paste', 'Duplicate', 'Show related', 'New diagram around', 'Go to source', 'Delete'].every((w) => boxItems.some((t) => t.includes(w))), boxItems.length + ' items');
  await page.keyboard.press('ArrowDown'); await page.keyboard.press('Escape');
  report('the menu closes with Esc', !(await page.locator('.wb-menu:not([hidden])').count()));
  await click('LC-CTL'); await page.keyboard.press('Shift+?'); await page.waitForTimeout(200);
  const helpShown = await page.locator('.wb-help:not([hidden])').count(); const helpText = await page.locator('.wb-help').textContent();
  report('? lists the shortcuts', helpShown > 0 && /⌘D/.test(helpText) && /Double-click/.test(helpText));
  await page.keyboard.press('Escape');
  // 20. A refused cut keeps the clipboard: the same cut pastes elsewhere afterwards
  await reset(); r = {}; await layer('Logical'); await click('LC-CAM'); await page.keyboard.press('Meta+x'); await click('LF-ACT'); n = await count(); await page.keyboard.press('Meta+v'); r = { result: await waitEdit(n) };
  const refusedFirst = r.result && !r.result.ok;
  await click('LC-CTL'); n = await count(); await page.keyboard.press('Meta+v'); r = { result: await waitEdit(n) }; text = await source();
  report('a refused paste keeps the cut, which then pastes elsewhere', !!refusedFirst && !!r.result?.ok && text.indexOf('component "Camera"') > text.indexOf('component "LKA Controller"'), refusedFirst ? r.result?.error : 'the first paste was not refused');
  report('the cut marker goes once the cut is pasted', (await page.locator('.wb-cut').count()) === 0);

  // 21. The canvas keys leave other text alone; ports are not copied by the keyboard either
  await reset(); r = {}; await layer('Logical'); await click('LC-CTL');
  const swallowed = await page.evaluate(() => { const input = document.querySelector('.wb-name-input'); input.focus(); input.select(); const keys = ['c', 'a', 'd', 'x'].map((key) => { const e = new KeyboardEvent('keydown', { key, metaKey: true, bubbles: true, cancelable: true }); input.dispatchEvent(e); return e.defaultPrevented; }); input.blur(); return keys; });
  report('⌘C ⌘A ⌘D ⌘X in a properties field are the field\'s', swallowed.every((x) => !x), swallowed.join(','));
  { const port = await node('LC-CTL::lane').boundingBox(); await page.mouse.click(port.x + port.width / 2, port.y + port.height / 2); await page.waitForTimeout(300); }
  const portCopy = await page.evaluate(() => { const e = new KeyboardEvent('keydown', { key: 'c', metaKey: true, bubbles: true, cancelable: true }); document.querySelector('.arcviz-stage').dispatchEvent(e); return e.defaultPrevented; });
  report('⌘C on a port is not taken (ports move with Move to…)', !portCopy && !/Copied/.test(await page.locator('.wb-toast').textContent().catch(() => '')));

  // 22. Esc in a field is the field's: an armed tool stays armed
  await click('LC-CTL'); await page.locator('.wb-palette .wb-tool', { hasText: 'Component exchange' }).first().click();
  const armedBefore = await page.locator('.wb-tool.is-armed').count();
  await page.locator('.wb-name-input').focus(); await page.keyboard.press('Escape');
  report('Esc in a properties field leaves the armed tool armed', armedBefore > 0 && (await page.locator('.wb-tool.is-armed').count()) > 0);
  await page.locator('.arcviz-stage').focus(); await page.keyboard.press('Escape');

  // 23. Bends follow the line: two bends both kept; reversed with it; renamed with it; dropped on reconnect
  await reset(); r = {}; await layer('Logical');
  const bend = async (edge, dy) => { await selectEdge(edge); const d = page.locator('.arcviz-bend.is-new').first(); if (!(await d.isVisible().catch(() => false))) return false; const c = centreOf(await d.boundingBox()); await drag(c, { x: c.x + 30, y: c.y + dy }); await page.waitForTimeout(1200); return true; };
  const routeIn = (file, edge) => file?.arrangements?.find((a) => a.view === 'lab' && !a.open)?.edges?.[edge]?.points ?? [];
  await bend('ce:SteerCommand', 60); await bend('ce:SteerCommand', 40);
  const two = routeIn((await layouts()).at(-1), 'ce:SteerCommand');
  report('two bends in a row are both kept', two.length === 2, JSON.stringify(two));
  await selectEdge('ce:SteerCommand'); n = await count(); await page.locator('.wb-ends button', { hasText: 'Reverse' }).click(); r = { result: await waitEdit(n) }; await page.waitForTimeout(1500);
  const reversed = routeIn((await layouts()).at(-1), 'ce:SteerCommand');
  report('a reversed line keeps its bends, from its new source', !!r.result?.ok && reversed.length === 2 && reversed[0].x === two[1].x && reversed[0].y === two[1].y, JSON.stringify(reversed));
  await bend('ce:LaneFlow', 50);
  const labelBox = await node('ce:LaneFlow').locator('.av-edge-label').boundingBox(); await page.mouse.dblclick(labelBox.x + labelBox.width / 2, labelBox.y + labelBox.height / 2); await page.waitForTimeout(300);
  n = await count(); await page.locator('.wb-direct-edit').fill('Lane data'); await page.locator('.wb-direct-edit').press('Enter'); r = { result: await waitEdit(n) }; await page.waitForTimeout(1500);
  const renamedFile = (await layouts()).at(-1);
  report('a renamed line keeps its bends under its new name, none under the old', !!r.result?.ok && routeIn(renamedFile, 'ce:Lane data').length === 1 && routeIn(renamedFile, 'ce:LaneFlow').length === 0, JSON.stringify(renamedFile?.arrangements?.map((a) => Object.keys(a.edges || {}))));
  r = await reconnectByDrag('ce:Lane data', 'target', 'LC-ACT'); await page.waitForTimeout(1500);
  report('a reconnected line is drawn afresh, its bends dropped', !!r.result?.ok && routeIn((await layouts()).at(-1), 'ce:Lane data').length === 0, r.error || r.result?.error || '');

  // 24. A rename field open when the model changes underneath is closed, nothing written
  await reset(); r = {}; await layer('Logical'); await click('LC-CAM'); await page.keyboard.press('F2'); await page.waitForTimeout(200);
  const opened = await page.locator('.wb-direct-edit').count(); n = await count();
  await page.evaluate(() => fetch('/edit', { method: 'POST', body: JSON.stringify({ requests: [{ op: 'rename', element: 'LC-ACT', name: 'Elsewhere' }] }) }).then(() => window.__push()));
  await page.waitForTimeout(1500);
  const closed = (await page.locator('.wb-direct-edit').count()) === 0;
  report('a rename field is closed when the model changes under it', opened > 0 && closed && (await count()) === n && /closed/.test(await page.locator('.wb-toast').textContent().catch(() => '')));

  report('no page error', errors.length === 0, errors.slice(0, 2).join(' / '));
  const failed = rows.filter((x) => !x.ok); console.log(`\n${rows.length - failed.length}/${rows.length} editing gestures work`);
  await browser.close(); process.exit(failed.length ? 1 : 0);
})().catch((e) => { console.error(e); process.exit(2); });
