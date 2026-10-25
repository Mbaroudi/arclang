// The ArcLang web playground (VS Code for the Web on static files): the page
// loads, the web extension activates in its worker, the sample opens in the
// workbench, a palette tool edits the text, undo takes it back, `check`
// reports a broken model, and the warm reload is measured.
//   BASE=http://127.0.0.1:8765/ PLAYWRIGHT=/path/to/playwright node web_playground_test.js
// SHOTS=/dir keeps pictures. The page must be served with the COOP/COEP
// headers of site/nginx.conf (the test server in tools/ does).
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const BASE = process.env.BASE || 'http://127.0.0.1:8765/';
const SHOTS = process.env.SHOTS;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail || ''}`); };

async function workbenchFrame(page, timeout = 60000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    for (const f of page.frames()) { try { if (await f.$('.arcviz-stage svg')) return f; } catch { /* a frame going away */ } }
    await page.waitForTimeout(300);
  }
  return null;
}
const shot = async (page, name) => { if (SHOTS) await page.screenshot({ path: `${SHOTS}/${name}.png` }); };

(async () => {
  const browser = await chromium.launch({ args: ['--host-resolver-rules=MAP *.vscode-cdn.net ~NOTFOUND'] });
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await context.newPage();
  const errors = []; page.on('pageerror', (e) => errors.push(String(e)));
  const foreign = new Set(); const origin = new URL(BASE).origin;
  page.on('request', (r) => { const u = new URL(r.url()); // vscode-cdn.net names are the webviews' virtual resource origin, answered by the page's own service worker (the browser below cannot resolve them).
    if (/^https?:$/.test(u.protocol) && u.origin !== origin && !u.hostname.endsWith('.vscode-cdn.net')) foreign.add(u.origin); });
  // What a first visit downloads, compressed as the server would (gzip -6; nginx's gzip is on for these types).
  const zlib = require('node:zlib'); const bodies = [];
  page.on('response', (r) => { if (r.url().startsWith(origin) && r.status() === 200) bodies.push(r.body().then((b) => ({ url: r.url(), raw: b.length, gz: zlib.gzipSync(b, { level: 6 }).length })).catch(() => null)); });

  // 1. cold load
  const t0 = Date.now();
  await page.goto(BASE, { waitUntil: 'load' });
  await page.waitForFunction(() => window.arclangPlayground && window.arclangPlayground.ready, null, { timeout: 120000 });
  const ready = Date.now() - t0;
  report('the page loads and the workbench is ready', true, `${ready} ms`);
  let frame = await workbenchFrame(page);
  const firstDiagram = Date.now() - t0;
  report('the sample opens in the workbench with its diagram', !!frame, `first diagram after ${firstDiagram} ms`);
  if (!frame) { await shot(page, 'web-fail'); await browser.close(); process.exit(1); }
  await shot(page, 'web-1-opened');

  // 2. the extension runs in the worker: its status bar item, its language, its diagnostics
  const status = await page.locator('.statusbar').innerText();
  report('the extension is active (gate status in the status bar)', /Gate/.test(status), status.replace(/\s+/g, ' ').slice(0, 80));
  const lang = await page.evaluate(() => window.arclangPlayground.monaco.editor.getModels().find((m) => m.uri.path === '/workspace/starter.arc')?.getLanguageId());
  report('an .arc file is the ArcLang language', lang === 'arclang', lang);

  // 3. a palette tool adds an element: the text changes and recompiles
  const before = await page.evaluate(() => window.arclangPlayground.modelText('/workspace/starter.arc'));
  await frame.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await page.waitForTimeout(800);
  await frame.locator('.wb-tool', { hasText: 'Component' }).first().click(); await page.waitForTimeout(300);
  const input = frame.locator('.wb-prompt input').first();
  await input.waitFor({ timeout: 5000 }); await input.fill('Webcam'); await input.press('Enter');
  let after = before;
  for (let i = 0; i < 40 && after === before; i++) { await page.waitForTimeout(250); after = await page.evaluate(() => window.arclangPlayground.modelText('/workspace/starter.arc')); }
  report('a palette tool adds an element: the source text changes', after !== before && /Webcam/.test(after));
  frame = (await workbenchFrame(page)) || frame; await page.waitForTimeout(1200);
  const drawn = await frame.locator('.arcviz-stage [data-id], .arcviz-stage text', { hasText: 'Webcam' }).count().catch(() => 0);
  report('the diagram recompiled and shows the new element', drawn > 0 || (await frame.locator('.arcviz-stage').innerText().catch(() => '')).includes('Webcam'));
  await shot(page, 'web-2-added');

  // 4. undo
  await frame.locator('#arcviz-undo').click(); await page.waitForTimeout(1500);
  const undone = await page.evaluate(() => window.arclangPlayground.modelText('/workspace/starter.arc'));
  report('undo takes the element back', undone === before && !/Webcam/.test(undone));

  // 5. check reports a deliberately broken model
  await page.evaluate(() => window.arclangPlayground.setModelText('/workspace/starter.arc', 'model Broken {\n  version: \n'));
  let markers = [];
  for (let i = 0; i < 40 && !markers.some((m) => m.severity >= 8); i++) { await page.waitForTimeout(300); markers = await page.evaluate(() => window.arclangPlayground.markers('/workspace/starter.arc')); }
  report('check reports a broken model as a problem', markers.length > 0 && markers.some((m) => m.severity >= 8), markers[0] && markers[0].message.slice(0, 80));
  await shot(page, 'web-3-broken');

  // 6. the workspace is kept in IndexedDB: a reload brings the edited text back, and is timed
  await page.evaluate((text) => window.arclangPlayground.setModelText('/workspace/starter.arc', text + '\n// kept in this browser\n'), before);
  await page.waitForTimeout(1500);
  const downloads = (await Promise.all(bodies)).filter(Boolean);
  const t1 = Date.now();
  await page.reload({ waitUntil: 'load' });
  await page.waitForFunction(() => window.arclangPlayground && window.arclangPlayground.ready, null, { timeout: 120000 });
  const warmReady = Date.now() - t1;
  const frame2 = await workbenchFrame(page);
  const warmDiagram = Date.now() - t1;
  const kept = await page.evaluate(() => window.arclangPlayground.modelText('/workspace/starter.arc'));
  report('the edited workspace survives a reload (IndexedDB)', /kept in this browser/.test(kept || ''));
  report('the warm reload shows the diagram again', !!frame2, `ready ${warmReady} ms, first diagram ${warmDiagram} ms`);

  report('no request left for another host', foreign.size === 0, [...foreign].join(', '));
  report('no page error', errors.length === 0, errors.slice(0, 2).join(' | ').slice(0, 200));
  const raw = downloads.reduce((a, d) => a + d.raw, 0); const gz = downloads.reduce((a, d) => a + d.gz, 0);
  console.log(`cold: ready ${ready} ms, first diagram ${firstDiagram} ms; warm reload: ready ${warmReady} ms, first diagram ${warmDiagram} ms`);
  console.log(`first visit: ${downloads.length} files, ${(raw / 1e6).toFixed(1)} MB raw, ${(gz / 1e6).toFixed(1)} MB gzip`);
  if (process.env.BIGGEST) for (const d of downloads.sort((a, b) => b.gz - a.gz).slice(0, 15)) console.log(`  ${(d.gz / 1e3).toFixed(0).padStart(6)} kB gz  ${d.url.replace(origin, '')}`);
  await browser.close();
  const failed = rows.filter((r) => !r.ok).length;
  console.log(failed ? `${failed} FAILED` : 'all ok');
  process.exit(failed ? 1 : 0);
})();
