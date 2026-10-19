// Editing gestures beyond the palette, through the real compiler.
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const BASE = `http://127.0.0.1:${process.env.PORT || 8766}`;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok, detail }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(46)} ${detail || ''}`); };
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  const errors = []; page.on('pageerror', (e) => errors.push(String(e)));
  await page.goto(`${BASE}/e2e.html`); await page.waitForSelector('.arcviz-stage svg', { timeout: 30000 }); await page.waitForTimeout(600);
  const snapshot = () => page.evaluate(() => fetch('/snapshot').then((r) => r.json()));
  const source = () => page.evaluate(() => fetch('/source').then((r) => r.text()));
  const reset = async () => { await page.evaluate(() => fetch('/reset')); await page.evaluate(() => window.__push()); await page.waitForTimeout(1000); };
  const count = () => page.evaluate(() => window.__results.length);
  const last = () => page.evaluate(() => window.__results.at(-1));
  const waitEdit = async (before) => { for (let i = 0; i < 50 && (await count()) === before; i++) await page.waitForTimeout(150); await page.waitForTimeout(900); return (await count()) === before ? null : last(); };
  const layer = async (label) => { await page.locator('.wb-layer', { hasText: label }).first().click(); await page.waitForTimeout(900); };
  const box = async (id) => { const n = page.locator(`.arcviz-stage [data-id="${id}"]`).first(); return (await n.count()) ? n.boundingBox() : null; };
  const click = async (id) => { const b = await box(id); if (!b) return false; await page.mouse.click(b.x + Math.min(30, b.width / 2), b.y + Math.min(10, b.height / 2)); await page.waitForTimeout(300); return true; };
  const activeLayer = () => page.$$eval('.wb-layer.is-active', (b) => b.map((x) => x.textContent.trim())[0]);
  const prompt = async (text, keep) => { const i = page.locator('.wb-prompt input').first(); await i.waitFor({ timeout: 3000 }); if (!keep) await i.fill(text); await i.press('Enter'); };

  // 1. Trace from the palette
  await reset(); await layer('Logical'); let n = await count();
  await page.locator('.wb-tool', { hasText: 'Trace' }).first().click(); await click('LC-CAM'); await click('LC-CTL'); await prompt('realizes', true);
  let r = await waitEdit(n); let text = await source();
  report('Trace tool writes a realizes trace', !!r?.ok && /trace "LC-CAM" realizes "LC-CTL"/.test(text), r?.ok ? '' : r?.error);

  // 2. Hide from a viewpoint view: a new named diagram without the element, shown
  await reset(); await layer('Logical'); n = await count(); await click('LC-ACT'); await page.keyboard.press('Meta+h');
  r = await waitEdit(n); text = await source(); await page.waitForTimeout(800);
  const custom = /diagram "Logical Architecture \(custom\)" \{[^}]*shows: \["LC-CAM", "LC-CTL"\]/s.test(text);
  report('⌘H on a viewpoint creates a diagram without it', !!r?.ok && custom, r?.ok ? '' : r?.error);
  const actGone = !(await box('LC-ACT')) && !!(await box('LC-CAM'));
  report('the new diagram is on screen without the element', actGone);

  // 3. Hide inside a named diagram: out of shows
  await reset(); await layer('Logical'); n = await count();
  await page.evaluate(() => { const v = [...document.querySelectorAll('.arcviz-pick select')][0]; });
  const named = (await snapshot()).diagrams.graphs.find((g) => g.id === 'lab:Lane keeping chain');
  await page.evaluate((id) => window.postMessage({ type: 'show', view: id, select: null }, '*'), named.id); await page.waitForTimeout(1200);
  await click('LC-ACT'); n = await count(); await page.keyboard.press('Meta+h');
  r = await waitEdit(n); text = await source();
  report('⌘H in a named diagram takes it out of shows', !!r?.ok && /diagram "Lane keeping chain" \{[^}]*shows: \["LC-CAM", "LC-CTL"\]/s.test(text), r?.ok ? '' : r?.error);

  // 4. Show related, then new diagram around an element (properties buttons)
  await page.evaluate((id) => window.postMessage({ type: 'show', view: id, select: null }, '*'), named.id); await page.waitForTimeout(1200);
  await click('LC-CTL'); n = await count(); await page.locator('.wb-actions button', { hasText: 'Show related' }).click();
  r = await waitEdit(n); text = await source();
  report('Show related adds the neighbours to shows', !!r?.ok && /diagram "Lane keeping chain" \{[^}]*shows: \[[^\]]*"LC-ACT"/s.test(text), r?.ok ? '' : (r?.error || 'no edit'));
  await reset(); await layer('Logical'); await click('LC-CTL'); n = await count(); await page.locator('.wb-actions button', { hasText: 'New diagram' }).click();
  r = await waitEdit(n); text = await source(); await page.waitForTimeout(800);
  report('New diagram around an element, shown', !!r?.ok && /diagram "LKA Controller context" \{[^}]*context: true/s.test(text) && String(await activeLayer()).startsWith('Logical'), r?.ok ? '' : r?.error);

  // 5. Rename in place (F2)
  await reset(); await layer('Logical'); await click('LC-CAM'); await page.keyboard.press('F2');
  const direct = page.locator('.wb-direct-edit'); const hasDirect = await direct.count();
  if (hasDirect) { n = await count(); await direct.fill('Front camera'); await direct.press('Enter'); r = await waitEdit(n); }
  text = await source(); report('F2 renames in place on the canvas', hasDirect > 0 && /component "Front camera"/.test(text), hasDirect ? (r?.ok ? '' : r?.error) : 'no field');

  // 6. Find in diagram
  await page.keyboard.press('Meta+f'); const find = page.locator('.wb-find input'); await find.fill('steer');
  await page.waitForTimeout(400); const findCount = await page.locator('.wb-find-count').textContent();
  report('⌘F finds in the diagram', /of/.test(findCount || '') || /1/.test(findCount || ''), findCount || '');
  await find.press('Escape');

  // 7. Properties: set an attribute, rename by the name field
  await reset(); await layer('Logical'); await click('LC-CTL');
  const general = page.locator('.wb-props-tab', { hasText: 'All' }); await general.click(); await page.waitForTimeout(200);
  n = await count(); const desc = page.locator('table.wb-props tr', { hasText: 'Description' }).locator('textarea, input').first();
  if (await desc.count()) { await desc.fill('Plans the steering correction'); await desc.evaluate((e) => e.blur()); r = await waitEdit(n); }
  else { await page.locator('.wb-props-body .wb-link', { hasText: 'Add property' }).click(); await page.locator('.wb-add select').selectOption('description'); const f = page.locator('table.wb-props textarea').last(); await f.fill('Plans the steering correction'); await f.evaluate((e) => e.blur()); r = await waitEdit(n); }
  text = await source(); report('Properties sets an attribute', !!r?.ok && /Plans the steering correction/.test(text), r?.ok ? '' : r?.error);
  n = await count(); const nameField = page.locator('.wb-name-input'); await nameField.fill('Lane controller'); await nameField.press('Enter'); r = await waitEdit(n);
  text = await source(); report('Properties renames by the name field', !!r?.ok && /component "Lane controller"/.test(text), r?.ok ? '' : r?.error);

  // 8. Delete an element (key) and an exchange (edge)
  await reset(); await layer('Logical'); n = await count();
  { const st = await page.locator('.arcviz-stage').boundingBox(); await page.mouse.click(st.x + 8, st.y + 8); await page.waitForTimeout(300); }
  await page.locator('.wb-tool', { hasText: 'Component' }).first().click(); await prompt('Spare'); await waitEdit(n); await page.waitForTimeout(600);
  const spare = (await snapshot()).elements.elements.find((e) => e.name === 'Spare');
  await click(spare.id); n = await count(); await page.keyboard.press('Delete'); await page.waitForTimeout(300); if (await page.locator('.wb-toast button', { hasText: 'Delete with' }).count()) await page.locator('.wb-toast button', { hasText: 'Delete with' }).click(); r = await waitEdit(n); text = await source();
  report('Delete removes the selected element', !!r?.ok && !/component "Spare"/.test(text), r?.ok ? '' : r?.error);
  const edge = page.locator('.arcviz-stage [data-id="ce:SteerCommand"]').first();
  if (await edge.count()) { const pt = await edge.evaluate((g) => { const path = g.querySelector('path'); const len = path.getTotalLength(); const p = path.getPointAtLength(len / 2); const m = path.getScreenCTM(); return { x: p.x * m.a + p.y * m.c + m.e, y: p.x * m.b + p.y * m.d + m.f }; }); await page.mouse.click(pt.x, pt.y); await page.waitForTimeout(300); n = await count(); await page.keyboard.press('Delete'); r = await waitEdit(n); text = await source();
    report('Delete removes the selected exchange', !!r?.ok && !/component_exchange "SteerCommand"/.test(text), r?.ok ? '' : (r?.error || 'no edit')); }
  else report('Delete removes the selected exchange', false, 'edge not found');

  // 9. Drag-to-connect from the handle, move by drag
  await reset(); await layer('Logical'); await page.mouse.move(10, 10);
  const from = await box('LC-CAM'); const to = await box('LC-ACT');
  await page.mouse.move(from.x + from.width / 2, from.y + 10); await page.waitForTimeout(250);
  const handle = page.locator('.arcviz-handle').first(); let dragged = false;
  if (await handle.count() && await handle.isVisible()) { const hb = await handle.boundingBox(); n = await count();
    await page.mouse.move(hb.x + hb.width / 2, hb.y + hb.height / 2); await page.mouse.down(); await page.mouse.move(to.x + to.width / 2, to.y + 10, { steps: 10 }); await page.mouse.up(); await page.waitForTimeout(300);
    if (await page.locator('.wb-prompt input').count()) { await prompt('Camera to actuator'); r = await waitEdit(n); dragged = true; } }
  text = await source(); report('drag from the handle connects (component exchange)', dragged && !!r?.ok && /component_exchange "Camera to actuator"/.test(text), dragged ? (r?.ok ? '' : r?.error) : 'no handle or no prompt');

  report('no page error', errors.length === 0, errors.slice(0, 2).join(' / '));
  const failed = rows.filter((x) => !x.ok); console.log(`\n${rows.length - failed.length}/${rows.length} gestures work`);
  await browser.close(); process.exit(failed.length ? 1 : 0);
})().catch((e) => { console.error(e); process.exit(2); });
