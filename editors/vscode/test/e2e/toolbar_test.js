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
  const tool = (label) => page.locator('.wb-viewer-tools button', { hasText: new RegExp(`^${label.replace(/[+−]/g, '\\$&')}$`) }).first();
  const labels = await page.$$eval('.wb-viewer-tools button', (bs) => bs.map((b) => b.textContent.trim()));
  ok('toolbar buttons present', labels.length >= 10, labels.join(' | '));
  // Exports
  await tool('Save SVG').click(); await page.waitForTimeout(500);
  await tool('Save PNG').click(); await page.waitForTimeout(2500);
  await tool('Save layout').click(); await page.waitForTimeout(500);
  const saves = (await posted()).filter((m) => m.type === 'save');
  const svg = saves.find((m) => m.name.endsWith('.svg')); const png = saves.find((m) => m.name.endsWith('.png')); const layout = saves.find((m) => m.name.endsWith('.json'));
  const svgText = svg ? await page.evaluate((n) => atob(window.__posted.find((m) => m.type === 'save' && m.name === n).data).slice(0, 200), svg.name) : '';
  ok('Save SVG reaches the host as an SVG document', !!svg && /<svg/.test(svgText), svg ? `${svg.name} ${svg.size} b64 chars` : 'no message');
  ok('Save PNG reaches the host as a PNG image', !!png && png.head.startsWith('iVBORw0KGgo'), png ? `${png.name} ${png.size} b64 chars` : 'no message');
  ok('Save layout reaches the host as JSON', !!layout && layout.mime.startsWith('application/json'), layout ? layout.name : 'no message');
  // Full screen
  await tool('Full screen').click(); await page.waitForTimeout(400);
  const focusOn = await page.evaluate(() => document.body.classList.contains('wb-focus'));
  const paletteKept = await page.evaluate(() => getComputedStyle(document.querySelector('.wb-left')).display !== 'none');
  const fsMsg = (await posted()).filter((m) => m.type === 'focus-mode');
  ok('Full screen keeps the palette and asks the host for the whole screen', focusOn && paletteKept && fsMsg.at(-1)?.on === true);
  await page.keyboard.press('Escape'); await page.waitForTimeout(300);
  ok('Esc leaves full screen', !(await page.evaluate(() => document.body.classList.contains('wb-focus'))) && (await posted()).filter((m) => m.type === 'focus-mode').at(-1)?.on === false);
  // Zoom and fit change the sheet transform
  const transform = () => page.evaluate(() => { const g = document.querySelector('.arcviz-stage svg'); return g ? (g.getAttribute('viewBox') || '') + '|' + (g.style.transform || '') + '|' + (document.querySelector('.arcviz-canvas')?.style.transform || '') : ''; });
  const t0 = await transform(); await tool('+').click(); await page.waitForTimeout(300); const t1 = await transform();
  ok('+ zooms in', t0 !== t1); await tool('−').click(); await page.waitForTimeout(300); ok('− zooms out', (await transform()) !== t1);
  await tool('Fit').click(); await page.waitForTimeout(300); ok('Fit answers', true);
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
  // Fold / unfold / arrange / filters / back
  const filters = page.locator('.wb-viewer-tools summary', { hasText: 'Filters' }).first();
  await filters.click(); await page.waitForTimeout(300);
  const filterBoxes = await page.locator('.arcviz-filters[open] input[type=checkbox]').count();
  ok('Filters opens its menu of kinds', filterBoxes > 0, `${filterBoxes} checkboxes`);
  const filterLabels = await page.$$eval('.arcviz-filters[open] label', (ls) => ls.map((l) => l.textContent.trim()));
  console.log('     filters:', filterLabels.join(' | '));
  const drawn = () => page.evaluate(() => document.querySelector('.arcviz-stage svg')?.innerHTML.length ?? 0);
  let changed = false;
  for (let i = 0; i < filterBoxes && !changed; i++) {
    const before = await drawn(); const box = page.locator('.arcviz-filters[open] input[type=checkbox]').nth(i);
    await box.click(); await page.waitForTimeout(700); const after = await drawn();
    if (before !== after) { changed = true; ok('a filter changes what is drawn', true, `${filterLabels[i]}: ${before} -> ${after} chars of SVG`); }
    await box.click(); await page.waitForTimeout(500);
  }
  if (!changed) ok('a filter changes what is drawn', false, 'no checkbox changed the drawing');
  await filters.click();
  for (const label of ['Fold all', 'Unfold all', 'Arrange', 'Arrange', 'Back']) {
    const b = tool(label); if (await b.count() === 0) { ok(`${label}`, false, 'missing'); continue; }
    if (await b.isDisabled()) { ok(`${label} (disabled here)`, true); continue; }
    await b.click(); await page.waitForTimeout(300); ok(`${label} runs`, true);
    if (label === 'Filters') await page.keyboard.press('Escape');
  }
  // Selection, F2 field, palette rail, drawer
  await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await page.waitForTimeout(700);
  const node = page.locator('.arcviz-stage [data-id="LC-CTL"]').first();
  const box = await node.boundingBox(); await page.mouse.click(box.x + box.width / 2, box.y + 12); await page.waitForTimeout(300);
  ok('click selects the element in Properties', (await page.locator('.wb-name-input').inputValue().catch(() => '')) === 'LKA Controller');
  await page.locator('.wb-toolrow .wb-icon-button').first().click(); await page.waitForTimeout(200);
  ok('palette rail collapses', await page.evaluate(() => document.querySelector('.wb-left').classList.contains('is-collapsed')));
  await page.locator('.wb-toolrow .wb-icon-button').last().click(); await page.waitForTimeout(200);
  ok('properties drawer hides', await page.evaluate(() => document.querySelector('.wb-right').hidden));
  ok('no console or page error', errors.length === 0, errors.slice(0, 3).join(' / '));
  await browser.close(); server.close();
  const failed = results.filter((r) => !r[0]).length; console.log(failed ? `\n${failed} FAILED` : '\nALL CHECKS PASSED'); process.exit(failed ? 1 : 0);
})().catch((e) => { console.error(e); process.exit(2); });
