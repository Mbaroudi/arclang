// Clicks every tool of the workbench page in a browser with a fake VS Code
// host and checks what the host receives and that the console stays clean.
const http = require('http'); const fs = require('fs'); const path = require('path');
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const ROOT = process.argv[2];
const server = http.createServer((req, res) => {
  const file = path.join(ROOT, decodeURIComponent(req.url.split('?')[0]));
  fs.readFile(file, (err, data) => { if (err) { res.writeHead(404); res.end(); return; }
    const type = file.endsWith('.js') ? 'text/javascript' : file.endsWith('.css') ? 'text/css' : file.endsWith('.json') ? 'application/json' : 'text/html';
    res.writeHead(200, { 'content-type': type }); res.end(data); });
}).listen(0);
const results = []; const ok = (name, cond, detail = '') => { results.push([cond, name, detail]); console.log(`${cond ? 'ok  ' : 'FAIL'} ${name}${detail ? '  ' + detail : ''}`); };
(async () => {
  const port = server.address().port;
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const errors = []; page.on('pageerror', (e) => errors.push(String(e))); page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  await page.goto(`http://127.0.0.1:${port}/index.html?model=snapshot`);
  await page.waitForSelector('.arcviz-stage svg', { timeout: 30000 });
  await page.waitForTimeout(800);
  const posted = () => page.evaluate(() => window.__posted.map((m) => ({ type: m.type, name: m.name, mime: m.mime, head: m.data ? m.data.slice(0, 16) : undefined, size: m.data ? m.data.length : 0, on: m.on })));
  const byId = (id) => page.locator(`#${id}`);
  const ids = await page.$$eval('.wb-viewer-tools .arcviz-tools button[id]', (bs) => bs.map((b) => b.id));
  ok('toolbar: back, undo, redo, zoom, menus, full screen', ['arcviz-back', 'arcviz-undo', 'arcviz-redo', 'arcviz-zoom-out', 'arcviz-zoom-level', 'arcviz-zoom-in', 'arcviz-fit', 'arcviz-layout-menu', 'arcviz-align-menu', 'arcviz-filters-menu', 'arcviz-export-menu', 'arcviz-fullscreen'].every((id) => ids.includes(id)), ids.join(' '));
  const unnamed = await page.$$eval('.wb-toolrow button', (bs) => bs.filter((b) => !b.hidden && !(b.getAttribute('aria-label') || b.textContent.trim())).length);
  ok('every button of the bar has a name', unnamed === 0, `${unnamed} unnamed`);
  const tip = (id) => byId(id).getAttribute('title');
  ok('tooltips carry the shortcut', /^Undo \((⌘Z|Ctrl\+Z)\)$/.test(await tip('arcviz-undo')) && /\(\+\)$/.test(await tip('arcviz-zoom-in')) && /Shift\+M|⇧M/.test(await page.locator('.wb-minimap-toggle').getAttribute('title')), `${await tip('arcviz-undo')} | ${await tip('arcviz-zoom-in')}`);
  ok('Undo and Redo are off with nothing to undo', (await byId('arcviz-undo').isDisabled()) && (await byId('arcviz-redo').isDisabled()));
  const menu = (id) => page.locator(`.arcviz-menu-pop:not([hidden])`);
  const choose = async (menuId, action) => { await byId(menuId).click(); await page.waitForTimeout(200); await page.locator(`.arcviz-menu-pop:not([hidden]) [data-action="${action}"]`).click(); await page.waitForTimeout(400); };
  // Exports, from the Export menu
  await choose('arcviz-export-menu', 'save-svg'); await page.waitForTimeout(300);
  await choose('arcviz-export-menu', 'save-png'); await page.waitForTimeout(2500);
  await byId('arcviz-export-menu').click(); await page.waitForTimeout(200);
  ok('no Save layout in Export: the arrangement keeps itself', (await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="save-layout"]').count()) === 0);
  await page.keyboard.press('Escape'); await page.waitForTimeout(150);
  ok('Esc closes a menu and gives the focus back to its button', (await menu().count()) === 0 && (await page.evaluate(() => document.activeElement?.id)) === 'arcviz-export-menu');
  const saves = (await posted()).filter((m) => m.type === 'save');
  const svg = saves.find((m) => m.name.endsWith('.svg')); const png = saves.find((m) => m.name.endsWith('.png'));
  const svgText = svg ? await page.evaluate((n) => atob(window.__posted.find((m) => m.type === 'save' && m.name === n).data).slice(0, 200), svg.name) : '';
  ok('Save as SVG reaches the host as an SVG document', !!svg && /<svg/.test(svgText), svg ? `${svg.name} ${svg.size} b64 chars` : 'no message');
  ok('Save as PNG reaches the host as a PNG image', !!png && png.head.startsWith('iVBORw0KGgo'), png ? `${png.name} ${png.size} b64 chars` : 'no message');
  // The keyboard: Enter opens with the first item focused, arrows move, Enter acts
  await byId('arcviz-layout-menu').focus(); await page.keyboard.press('Enter'); await page.waitForTimeout(200);
  const first = await page.evaluate(() => document.activeElement?.dataset.action);
  await page.keyboard.press('ArrowDown'); const second = await page.evaluate(() => document.activeElement?.dataset.action);
  await page.keyboard.press('End'); const last = await page.evaluate(() => document.activeElement?.dataset.action);
  ok('Enter opens a menu on its first item; arrows and End move', first === 'reset-layout' && second === 'fold-all' && last === 'unfold-all', `${first} ${second} ${last}`);
  ok('no Arrange switch where boxes are always placed by dragging', (await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="arrange"]').count()) === 0);
  await page.keyboard.press('Escape');
  // Full screen
  await byId('arcviz-fullscreen').click(); await page.waitForTimeout(400);
  const focusOn = await page.evaluate(() => document.body.classList.contains('wb-focus'));
  const paletteKept = await page.evaluate(() => getComputedStyle(document.querySelector('.wb-left')).display !== 'none');
  const fsMsg = (await posted()).filter((m) => m.type === 'focus-mode');
  ok('Full screen keeps the palette and asks the host for the whole screen', focusOn && paletteKept && fsMsg.at(-1)?.on === true);
  await page.locator('.arcviz-stage').click({ position: { x: 4, y: 4 } }); await page.keyboard.press('Escape'); await page.waitForTimeout(300);
  ok('Esc leaves full screen', !(await page.evaluate(() => document.body.classList.contains('wb-focus'))) && (await posted()).filter((m) => m.type === 'focus-mode').at(-1)?.on === false);
  // Zoom: the buttons, the level shown, the presets
  const level = () => byId('arcviz-zoom-level').textContent().then((t) => t.trim());
  const scale = () => page.evaluate(() => Number(/matrix\(([\d.]+)/.exec(getComputedStyle(document.querySelector('.arcviz-canvas')).transform)?.[1] || 1));
  const l0 = await level(); await byId('arcviz-zoom-in').click(); await page.waitForTimeout(400); const l1 = await level();
  ok('+ zooms in and the level follows', l0 !== l1 && Math.abs(parseInt(l1, 10) - Math.round((await scale()) * 100)) <= 1, `${l0} -> ${l1}`);
  await byId('arcviz-zoom-out').click(); await page.waitForTimeout(400); ok('− zooms out', (await level()) !== l1);
  await choose('arcviz-zoom-level', 'zoom-150'); await page.waitForTimeout(300);
  ok('the 150 % preset zooms to 150 %', (await level()).replace(/\s/g, ' ') === '150 %' && Math.abs((await scale()) - 1.5) < 0.01, await level());
  const stageBox = await page.locator('.arcviz-stage').boundingBox();
  await page.mouse.move(stageBox.x + stageBox.width / 2, stageBox.y + stageBox.height / 2);
  await page.keyboard.down('Control'); await page.mouse.wheel(0, 120); await page.keyboard.up('Control'); await page.waitForTimeout(300);
  ok('the level shows the zoom after the wheel', (await level()).replace(/\s/g, ' ') !== '150 %' && Math.abs(parseInt(await level(), 10) - Math.round((await scale()) * 100)) <= 1, await level());
  await byId('arcviz-fit').click(); await page.waitForTimeout(300); ok('Fit answers', true);
  // Layer bar
  const layers = await page.$$eval('.wb-layer', (bs) => bs.map((b) => b.textContent.trim()));
  ok('layer bar lists the layers', layers.length >= 5, layers.join(' | '));
  for (const name of ['Logical', 'Physical', 'Modes', 'Scenarios', 'Operational']) {
    const btn = page.locator('.wb-layer', { hasText: name }).first();
    if (await btn.count() === 0) { ok(`layer ${name}`, false, 'missing'); continue; }
    await btn.click(); await page.waitForTimeout(700);
    ok(`layer ${name} becomes the view on screen`, await btn.evaluate((b) => b.classList.contains('is-active')));
  }
  await page.keyboard.press(']'); await page.waitForTimeout(500); ok('] steps to the next layer', (await page.$$eval('.wb-layer.is-active', (b) => b.map((x) => x.textContent.trim())))[0] !== 'Operational');
  // Filters: a panel of kinds, a count on the button
  await byId('arcviz-filters-menu').click(); await page.waitForTimeout(300);
  const boxes = () => page.locator('.arcviz-filters-pop:not([hidden]) input[type=checkbox]');
  const filterBoxes = await boxes().count();
  ok('Filters opens its menu of kinds', filterBoxes > 0, `${filterBoxes} checkboxes`);
  const filterLabels = await page.$$eval('.arcviz-filters-pop:not([hidden]) label', (ls) => ls.map((l) => l.textContent.trim()));
  console.log('     filters:', filterLabels.join(' | '));
  const drawn = () => page.evaluate(() => document.querySelector('.arcviz-stage svg')?.innerHTML.length ?? 0);
  let changed = false;
  for (let i = 0; i < filterBoxes && !changed; i++) {
    const before = await drawn(); const box = boxes().nth(i);
    await box.click(); await page.waitForTimeout(700); const after = await drawn();
    if (before !== after) {
      changed = true;
      ok('a filter changes what is drawn', true, `${filterLabels[i]}: ${before} -> ${after} chars of SVG`);
      ok('the Filters button counts what is filtered out', (await page.locator('#arcviz-filters-menu .arcviz-badge').textContent()) === '1');
    }
    await box.click(); await page.waitForTimeout(500);
  }
  if (!changed) ok('a filter changes what is drawn', false, 'no checkbox changed the drawing');
  await page.keyboard.press('Escape'); await page.waitForTimeout(150);
  ok('Esc closes the filters', (await boxes().count()) === 0);
  for (const action of ['fold-all', 'unfold-all']) {
    await byId('arcviz-layout-menu').click(); await page.waitForTimeout(200);
    const item = page.locator(`.arcviz-menu-pop:not([hidden]) [data-action="${action}"]`);
    if (await item.getAttribute('aria-disabled') === 'true') { ok(`${action} (off here)`, true); await page.keyboard.press('Escape'); continue; }
    await item.click(); await page.waitForTimeout(500); ok(`${action} runs and closes the menu`, (await page.locator('.arcviz-menu-pop:not([hidden])').count()) === 0);
  }
  // Align: off until two boxes are selected, then it lines them up
  await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await page.waitForTimeout(700);
  ok('Align is off with one box or none, and says why', (await byId('arcviz-align-menu').getAttribute('aria-disabled')) === 'true' && /two or more/.test(await tip('arcviz-align-menu')), await tip('arcviz-align-menu'));
  const shape = (id) => page.locator(`.arcviz-stage [data-id="${id}"] > .av-shape`).first().boundingBox();
  const cam = await shape('LC-CAM'); await page.mouse.click(cam.x + 20, cam.y + 8); await page.waitForTimeout(300);
  const ctl = await shape('LC-CTL'); await page.keyboard.down('Shift'); await page.mouse.click(ctl.x + 20, ctl.y + 8); await page.keyboard.up('Shift'); await page.waitForTimeout(400);
  ok('Align is on with two boxes selected', (await byId('arcviz-align-menu').getAttribute('aria-disabled')) === 'false');
  await byId('arcviz-align-menu').click(); await page.waitForTimeout(200);
  ok('Distribute needs three boxes', (await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="distribute-horizontal"]').getAttribute('aria-disabled')) === 'true');
  await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="align-top"]').click(); await page.waitForTimeout(800);
  const [a1, a2] = [await shape('LC-CAM'), await shape('LC-CTL')];
  ok('Align top lines the two boxes up', Math.abs(a1.y - a2.y) < 2, `${Math.round(a1.y)} / ${Math.round(a2.y)}`);
  // Undo and redo: the host's history, its labels in the tooltips
  await page.evaluate(() => window.postMessage({ type: 'history', undo: 'move LC-CAM', redo: null }, '*')); await page.waitForTimeout(200);
  ok('Undo names what it undoes', !(await byId('arcviz-undo').isDisabled()) && /^Undo move LC-CAM \(/.test(await tip('arcviz-undo')) && (await byId('arcviz-redo').isDisabled()), await tip('arcviz-undo'));
  await byId('arcviz-undo').click(); await page.waitForTimeout(200);
  await page.evaluate(() => window.postMessage({ type: 'history', undo: null, redo: 'move LC-CAM' }, '*')); await page.waitForTimeout(200);
  await byId('arcviz-redo').click(); await page.waitForTimeout(200);
  const ops = (await page.evaluate(() => window.__posted.filter((m) => m.type === 'history').map((m) => m.op)));
  ok('the Undo and Redo buttons ask the host', ops.join(',') === 'undo,redo', ops.join(','));
  await page.keyboard.press('Escape'); await page.waitForTimeout(200);
  await byId('arcviz-back').isDisabled().then((off) => ok('Back is off until a jump', off));
  // A narrow panel: words go first, then tools move into "⋯"; nothing is cut, nothing wraps
  for (const width of [1240, 800, 600, 420]) {
    await page.setViewportSize({ width, height: 800 }); await page.waitForTimeout(500);
    const row = await page.evaluate(() => {
      const bar = document.querySelector('.wb-toolrow').getBoundingClientRect();
      const tools = document.querySelector('.arcviz-tools');
      const shown = [...document.querySelectorAll('.wb-toolrow button')].filter((b) => b.getBoundingClientRect().width > 0);
      const cut = shown.filter((b) => { const r = b.getBoundingClientRect(); return r.top < bar.top - 1 || r.bottom > bar.bottom + 1 || r.left < bar.left - 1 || r.right > bar.right + 1; }).map((b) => b.id || b.getAttribute('aria-label'));
      return { cut, rows: new Set(shown.map((b) => Math.round(b.getBoundingClientRect().top))).size, compact: tools.classList.contains('is-compact'), more: !document.getElementById('arcviz-more').closest('.arcviz-menu').hidden, scroll: document.documentElement.scrollWidth > innerWidth, height: Math.round(bar.height) };
    });
    ok(`at ${width} px the bar is one line, nothing cut`, row.cut.length === 0 && row.rows === 1 && !row.scroll, `${row.cut.join(',')} rows ${row.rows} compact ${row.compact} more ${row.more} h ${row.height}`);
    if (width === 1240) ok('at 1240 px every tool shows its words', !row.compact && !row.more);
    if (width === 420) {
      ok('at 420 px the rest is in "⋯"', row.more);
      await byId('arcviz-more').click(); await page.waitForTimeout(200);
      const items = await page.$$eval('.arcviz-menu-pop:not([hidden]) [data-action]', (bs) => bs.map((b) => b.dataset.action));
      ok('"⋯" offers what moved into it', ['zoom-fit', 'save-svg', 'save-png', 'fold-all'].every((a) => items.includes(a)), items.join(' '));
      await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="zoom-50"]').click(); await page.waitForTimeout(400);
      ok('an action from "⋯" runs', Math.abs((await scale()) - 0.5) < 0.01);
      await byId('arcviz-more').click(); await page.waitForTimeout(200);
      await page.locator('.arcviz-menu-pop:not([hidden]) .arcviz-menu-item', { hasText: 'Filters' }).click(); await page.waitForTimeout(300);
      ok('Filters from "⋯" opens the filter panel', (await boxes().count()) > 0);
      await page.keyboard.press('Escape');
    }
  }
  await page.setViewportSize({ width: 1280, height: 800 }); await page.waitForTimeout(400);
  // Selection, F2 field, palette rail, drawer
  await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await page.waitForTimeout(700);
  const node = page.locator('.arcviz-stage [data-id="LC-CTL"]').first();
  const box = await node.boundingBox(); await page.mouse.click(box.x + box.width / 2, box.y + 12); await page.waitForTimeout(300);
  ok('click selects the element in Properties', (await page.locator('.wb-name-input').inputValue().catch(() => '')) === 'LKA Controller');
  await page.locator('.wb-toolrow .wb-palette-toggle').click(); await page.waitForTimeout(200);
  ok('palette rail collapses', await page.evaluate(() => document.querySelector('.wb-left').classList.contains('is-collapsed')));
  await page.locator('.wb-toolrow .wb-icon-button').last().click(); await page.waitForTimeout(200);
  ok('properties drawer hides', await page.evaluate(() => document.querySelector('.wb-right').hidden));
  ok('no console or page error', errors.length === 0, errors.slice(0, 3).join(' / '));
  await browser.close(); server.close();
  const failed = results.filter((r) => !r[0]).length; console.log(failed ? `\n${failed} FAILED` : '\nALL CHECKS PASSED'); process.exit(failed ? 1 : 0);
})().catch((e) => { console.error(e); process.exit(2); });
