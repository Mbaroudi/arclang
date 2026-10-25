// The table view in a browser with a fake VS Code host, on tables the real
// compiler computed (`arclang table -f json`): rendering, sorting, filters,
// virtual rows for thousands of rows, a cell edited in place, matrix clicks,
// copy and exports reaching the host, both themes. SHOTS=dir saves pictures.
const http = require('http'); const fs = require('fs'); const path = require('path');
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const ROOT = process.argv[2]; const SHOTS = process.env.SHOTS;
const server = http.createServer((req, res) => {
  const file = path.join(ROOT, decodeURIComponent(req.url.split('?')[0]));
  fs.readFile(file, (err, data) => { if (err) { res.writeHead(404); res.end(); return; }
    const type = file.endsWith('.js') ? 'text/javascript' : file.endsWith('.css') ? 'text/css' : file.endsWith('.json') ? 'application/json' : 'text/html';
    res.writeHead(200, { 'content-type': type }); res.end(data); });
}).listen(0);
let failures = 0;
const ok = (name, cond, detail = '') => { if (!cond) failures++; console.log(`${cond ? 'ok  ' : 'FAIL'} ${name}${detail ? '  ' + detail : ''}`); };
(async () => {
  const port = server.address().port;
  const browser = await chromium.launch();
  const errors = [];
  const open = async (table, theme = 'light', viewport = { width: 1440, height: 820 }) => {
    const page = await browser.newPage({ viewport });
    page.on('pageerror', (e) => errors.push(String(e))); page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
    await page.goto(`http://127.0.0.1:${port}/tables.html?table=${table}&theme=${theme}`);
    await page.waitForSelector('.tb-row, .tb-mrow, .tb-none', { timeout: 15000 });
    return page;
  };
  const posted = (page, type) => page.evaluate((t) => window.__posted.filter((m) => m.type === t), type);
  const ids = (page) => page.$$eval('.tb-row', (rows) => rows.sort((a, b) => a.getBoundingClientRect().top - b.getBoundingClientRect().top).map((r) => r.dataset.id));

  // Coverage table
  const page = await open('coverage');
  ok('every requirement is a row', JSON.stringify(await ids(page)) === JSON.stringify(['REQ-1', 'REQ-2', 'REQ-3', 'REQ-4']), (await ids(page)).join(' '));
  ok('the header says the gate is blocked', /blocker/.test(await page.textContent('.tb-gate')));
  ok('a failing row and a gap row are marked', (await page.$$('.tb-row-fail')).length === 1 && (await page.$$('.tb-row-gap')).length === 1);
  ok('links are chips with the verdict', /Range test\s*pass/.test(await page.textContent('.tb-row[data-id="REQ-1"] [data-key="verified_by"]')));
  if (SHOTS) await page.screenshot({ path: path.join(SHOTS, 'browser-coverage-light.png') });
  // Sort
  await page.click('.tb-th:has-text("Req id")'); await page.click('.tb-th:has-text("Req id")');
  ok('a header click sorts, again reverses', (await ids(page))[0] === 'REQ-4', (await ids(page)).join(' '));
  await page.click('.tb-th:has-text("Req id")');
  // Filter
  await page.fill('.tb-filter', 'is:gap'); await page.waitForTimeout(250);
  ok('is:gap keeps the gaps and failures', JSON.stringify(await ids(page)) === JSON.stringify(['REQ-2', 'REQ-4']), (await ids(page)).join(' '));
  await page.fill('.tb-filter', 'priority=high -is:fail'); await page.waitForTimeout(250);
  ok('column=text and negation', JSON.stringify(await ids(page)) === JSON.stringify(['REQ-1']), (await ids(page)).join(' '));
  await page.fill('.tb-filter', ''); await page.waitForTimeout(250);
  const satisfiedFilter = page.locator('.tb-col-filter').nth(4);
  await satisfiedFilter.fill('brake'); await page.waitForTimeout(250);
  ok('a column filter', JSON.stringify(await ids(page)) === JSON.stringify(['REQ-2']), (await ids(page)).join(' '));
  ok('the column filter keeps its focus while typing', await satisfiedFilter.evaluate((n) => document.activeElement === n));
  await satisfiedFilter.fill(''); await page.waitForTimeout(250);
  await page.check('.tb-toggle input'); await page.waitForTimeout(150);
  ok('only gaps (a pending verdict is none)', (await ids(page)).length === 2, (await ids(page)).join(' '));
  await page.uncheck('.tb-toggle input'); await page.waitForTimeout(150);
  // Edit in place
  await page.dblclick('.tb-row[data-id="REQ-3"] [data-key="text"]');
  await page.fill('.tb-row[data-id="REQ-3"] .tb-editor', 'Warn the driver 2 s before braking');
  await page.keyboard.press('Enter');
  const edits = await posted(page, 'edit');
  ok('a cell edit reaches the host as a set request', edits.length === 1 && edits[0].edit.key === 'description' && edits[0].value === '"Warn the driver 2 s before braking"', JSON.stringify(edits[0] || {}));
  await page.dblclick('.tb-row[data-id="REQ-3"] [data-key="priority"]');
  ok('an enumeration is edited with its values', (await page.$$eval('.tb-row[data-id="REQ-3"] select.tb-editor option', (o) => o.map((x) => x.value))).includes('Critical'));
  await page.selectOption('.tb-row[data-id="REQ-3"] select.tb-editor', 'Critical');
  ok('picking a value edits it', (await posted(page, 'edit')).at(-1)?.value === 'Critical');
  await page.dblclick('.tb-row[data-id="REQ-1"] [data-key="text"]'); await page.keyboard.press('Escape');
  ok('Escape leaves the cell unchanged', (await posted(page, 'edit')).length === 2);
  // Links reveal
  await page.click('.tb-row[data-id="REQ-1"] .tb-link >> nth=0');
  ok('a link reveals its element', (await posted(page, 'reveal')).at(-1)?.id === 'LC-RADAR');
  // Copy and exports
  await page.click('.tb-button:has-text("Copy")');
  const copied = (await posted(page, 'copied')).at(-1);
  ok('Copy hands the rows shown as TSV', !!copied && copied.rows === 4 && copied.tsv.split('\n')[0].split('\t').length === 9, copied ? copied.tsv.split('\n')[0] : 'none');
  await page.click('.tb-button:has-text("CSV")'); await page.click('.tb-button:has-text("Excel")');
  ok('exports reach the host', JSON.stringify((await posted(page, 'export')).map((m) => m.format)) === '["csv","xlsx"]');
  // Column picker
  await page.click('.tb-columns summary');
  await page.locator('.tb-columns-item:has(code:text-is("gate")) input').uncheck();
  const spec = (await posted(page, 'spec')).at(-1);
  ok('the column picker asks for the columns', !!spec && !spec.spec.columns.includes('gate') && spec.spec.columns[0] === 'id', JSON.stringify(spec?.spec.columns));
  await page.close();

  // Dark theme and high contrast
  const dark = await open('coverage', 'dark');
  const bg = await dark.evaluate(() => getComputedStyle(document.body).backgroundColor);
  ok('the dark theme follows VS Code', bg === 'rgb(31, 31, 31)', bg);
  if (SHOTS) await dark.screenshot({ path: path.join(SHOTS, 'browser-coverage-dark.png') });
  await dark.close();
  const hc = await open('coverage', 'hc');
  ok('high contrast outlines the pills', (await hc.$eval('.tb-pill', (n) => getComputedStyle(n).borderTopStyle)) === 'solid');
  await hc.close();

  // Matrix
  const matrix = await open('satisfaction');
  ok('the matrix draws its rows and its turned column names', (await matrix.$$('.tb-mrow')).length === 4 && (await matrix.$$('.tb-mcol')).length === 4);
  ok('an empty row is marked', (await matrix.$$('.tb-mrow.tb-mempty')).length === 1);
  await matrix.click('.tb-mrow[data-row="3"] .tb-mcell[data-col="3"]');
  const add = (await posted(matrix, 'add-trace')).at(-1);
  ok('a click on an empty cell creates a trace', !!add && add.row === 3 && add.column === 3 && add.kind === 'satisfies', JSON.stringify(add));
  await matrix.click('.tb-mrow[data-row="0"] .tb-mcell[data-col="0"]');
  ok('a click on a filled cell asks to remove it', (await posted(matrix, 'remove-trace')).at(-1)?.row === 0);
  await matrix.hover('.tb-mrow[data-row="1"] .tb-mcell[data-col="2"]');
  ok('the crossing of the pointer is lit', (await matrix.$$('.tb-mcol.tb-lit')).length === 1 && (await matrix.$$('.tb-mrow.tb-lit')).length === 1);
  if (SHOTS) await matrix.screenshot({ path: path.join(SHOTS, 'browser-matrix-light.png') });
  await matrix.close();
  const n2 = await open('interfaces', 'dark');
  ok('an N² chart marks its diagonal', (await n2.$$('.tb-mself')).length === 4);
  if (SHOTS) await n2.screenshot({ path: path.join(SHOTS, 'browser-n2-dark.png') });
  await n2.close();
  const icd = await open('icd');
  ok('the ICD shows the protocol of the port', /CAN FD/.test(await icd.textContent('.tb-row[data-id="Targets"] [data-key="protocol"]')));
  if (SHOTS) await icd.screenshot({ path: path.join(SHOTS, 'browser-icd-light.png') });
  await icd.close();

  // Thousands of rows: drawn only where the view is
  const big = await open('coverage');
  await big.evaluate(() => {
    const t = window.__table;
    const rows = [];
    for (let i = 0; i < 6000; i++) { const r = t.rows[i % t.rows.length]; rows.push({ ...r, id: `REQ-${String(i).padStart(5, '0')}`, uuid: `u${i}`, cells: { ...r.cells, id: { text: `REQ-${String(i).padStart(5, '0')}` } } }); }
    window.postMessage({ type: 'table', table: { ...t, rows }, computedMs: 40, file: 'big.arc', history: { undo: null, redo: null } }, '*');
  });
  await big.waitForTimeout(400);
  const drawn = (await big.$$('.tb-row')).length;
  ok('6000 rows: only those in view are in the page', drawn > 10 && drawn < 120, `${drawn} drawn`);
  const started = Date.now();
  await big.$eval('.tb-scroll', (s) => { s.scrollTop = s.scrollHeight; });
  await big.waitForTimeout(200);
  ok('scrolled to the end, the last row is there', (await big.$$('.tb-row[data-id="REQ-05999"]')).length === 1, `${Date.now() - started} ms`);
  ok('the header stays on top while scrolling', (await big.$eval('.tb-header', (h) => h.getBoundingClientRect().top - h.closest('.tb-scroll').getBoundingClientRect().top)) === 0);
  await big.close();

  ok('no console error', errors.length === 0, errors.join(' | '));
  await browser.close(); server.close();
  console.log(failures ? `${failures} FAILED` : 'all table view checks passed');
  process.exit(failures ? 1 : 0);
})().catch((e) => { console.error(e); process.exit(1); });
