// Every palette tool of every view, through the real compiler.
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const BASE = `http://127.0.0.1:${process.env.PORT || 8766}`;
const rows = []; const report = (view, tool, ok, detail) => { rows.push({ view, tool, ok, detail }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${view.padEnd(4)} ${tool.padEnd(22)} ${detail || ''}`); };
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  const errors = []; page.on('pageerror', (e) => errors.push(String(e)));
  await page.goto(`${BASE}/e2e.html`); await page.waitForSelector('.arcviz-stage svg', { timeout: 30000 }); await page.waitForTimeout(600);
  const snapshot = () => page.evaluate(() => fetch('/snapshot').then((r) => r.json()));
  const source = () => page.evaluate(() => fetch('/source').then((r) => r.text()));
  const reset = async () => { await page.evaluate(() => fetch('/reset')); await page.evaluate(() => window.__push()); await page.waitForTimeout(900); };
  const lastResult = () => page.evaluate(() => window.__results.at(-1));
  const resultsCount = () => page.evaluate(() => window.__results.length);
  async function showLayer(view) {
    const graphs = (await snapshot()).diagrams.graphs; const g = graphs.find((x) => x.arc.view === view); if (!g) return null;
    await page.evaluate((id) => { const v = document.querySelector('#workbench'); }, g.id);
    const label = { oab: 'Operational', sab: 'System', lab: 'Logical', pab: 'Physical', cap: 'Capabilities', cdb: 'Data', msm: 'Modes', es: 'Scenarios' }[view];
    await page.locator('.wb-layer', { hasText: label }).first().click(); await page.waitForTimeout(800); return g;
  }
  async function clickNode(id) {
    const node = page.locator(`.arcviz-stage [data-id="${id}"]`).first(); if (await node.count() === 0) return false;
    const box = await node.boundingBox(); if (!box) return false;
    await page.mouse.click(box.x + Math.min(30, box.width / 2), box.y + Math.min(10, box.height / 2)); await page.waitForTimeout(250); return true;
  }
  async function answerPrompt(text) { const input = page.locator('.wb-prompt input').first(); await input.waitFor({ timeout: 3000 }); await input.fill(text); await input.press('Enter'); }
  async function waitResult(before) { for (let i = 0; i < 40 && (await resultsCount()) === before; i++) await page.waitForTimeout(150); await page.waitForTimeout(700); return lastResult(); }
  async function useTool(view, label, { select, connect, name }) {
    await reset(); const g = await showLayer(view); if (!g) return report(view, label, false, 'view absent');
    await page.keyboard.press('Escape');
    if (select) { if (!(await clickNode(select))) return report(view, label, false, `cannot select ${select}`); }
    const tool = page.locator('.wb-tool', { hasText: new RegExp(`^.?${label}`) }).filter({ hasText: label }).first();
    if (await tool.count() === 0) return report(view, label, false, 'tool missing');
    const before = await resultsCount();
    await tool.click(); await page.waitForTimeout(200);
    if (connect) { if (!(await clickNode(connect[0]))) return report(view, label, false, `no source ${connect[0]}`); if (!(await clickNode(connect[1]))) return report(view, label, false, `no target ${connect[1]}`); }
    await answerPrompt(name);
    const result = await waitResult(before);
    if (!result || result === undefined || (await resultsCount()) === before) return report(view, label, false, 'no edit sent');
    if (!result.ok) return report(view, label, false, `refused: ${String(result.error).split('\n')[0].slice(0, 140)}`);
    const text = await source(); const written = text.includes(name.replace(/[^A-Za-z0-9_]+/g, '_')) || text.includes(name);
    const snap = await snapshot();
    report(view, label, written && snap.success, written ? (snap.success ? 'written, compiles' : 'written, does NOT compile') : 'not in the text');
  }
  const ids = async () => (await snapshot()).elements.elements;
  const first = async (kind, n = 0) => (await ids()).filter((e) => e.kind === kind)[n]?.id;
  // OAB
  await useTool('oab', 'Actor', { name: 'Passenger' });
  await useTool('oab', 'Entity', { name: 'Road authority' });
  await useTool('oab', 'Activity', { select: await first('OperationalEntity'), name: 'Monitor road' });
  await useTool('oab', 'Capability', { name: 'Assist the driver' });
  await useTool('oab', 'Interaction', { connect: [await first('Actor'), await first('OperationalActivity')], name: 'Takeover request' });
  // SAB
  await useTool('sab', 'Actor', { name: 'Fleet operator' });
  await useTool('sab', 'Function', { name: 'Log events' });
  await useTool('sab', 'Component', { name: 'Recorder' });
  await useTool('sab', 'Port in', { select: await first('SystemFunction'), name: 'cmd_in' });
  await useTool('sab', 'Port out', { select: await first('SystemFunction'), name: 'cmd_out' });
  await useTool('sab', 'Mission', { name: 'Safe mobility' });
  await useTool('sab', 'Capability', { name: 'Lane change' });
  await useTool('sab', 'Functional chain', { name: 'Logging chain' });
  await useTool('sab', 'Functional exchange', { connect: [await first('SystemFunction', 0), await first('SystemFunction', 2)], name: 'health status' });
  // LAB
  await useTool('lab', 'Component', { name: 'Supervisor' });
  await useTool('lab', 'Function', { select: await first('LogicalComponent'), name: 'Supervise' });
  await useTool('lab', 'Port in', { select: await first('LogicalComponent'), name: 'diag_in' });
  await useTool('lab', 'Port out', { select: await first('LogicalComponent'), name: 'diag_out' });
  await useTool('lab', 'Interface', { name: 'Diagnostics' });
  await useTool('lab', 'Component exchange', { connect: [await first('LogicalComponent', 0), await first('LogicalComponent', 2)], name: 'Diag link' });
  await useTool('lab', 'Functional exchange', { connect: [await first('LogicalFunction', 0), await first('LogicalFunction', 2)], name: 'diag data' });
  // PAB
  await useTool('pab', 'Node', { name: 'Gateway ECU' });
  await useTool('pab', 'Behavior component', { select: await first('PhysicalNode'), name: 'Gateway SW' });
  await useTool('pab', 'Hardware component', { select: await first('PhysicalNode'), name: 'CAN transceiver' });
  await useTool('pab', 'Port in', { select: await first('PhysicalNode'), name: 'eth_in' });
  await useTool('pab', 'Physical link', { connect: [await first('PhysicalNode', 0), await first('PhysicalNode', 1)], name: 'Ethernet backbone' });
  await useTool('pab', 'Physical exchange', { connect: [await first('PhysicalNode', 0), await first('PhysicalNode', 1)], name: 'Video stream' });
  // CAP, CDB
  await useTool('cap', 'Mission', { name: 'Zero accidents' });
  await useTool('cap', 'Capability', { name: 'Emergency stop' });
  await useTool('cdb', 'Class', { name: 'Track' });
  await useTool('cdb', 'Enumeration', { name: 'Gear' });
  await useTool('cdb', 'Data type', { name: 'Speed' });
  await useTool('cdb', 'Exchange item', { name: 'TrackList' });
  // MSM, ES
  const states = (await ids()).filter((e) => e.kind === 'State');
  await useTool('msm', 'State', { name: 'Degraded' });
  await useTool('msm', 'Mode', { name: 'Maintenance' });
  await useTool('msm', 'Transition', { connect: [states[0]?.id, states[1]?.id], name: 'fault cleared' });
  const graph = (await snapshot()).diagrams.graphs.find((g) => g.arc.view === 'es');
  const lifelines = (graph?.children || []).map((n) => n.id);
  await useTool('es', 'Message', { connect: [lifelines[0], lifelines[1]], name: 'status request' });
  const failed = rows.filter((r) => !r.ok);
  console.log(`\n${rows.length - failed.length}/${rows.length} tools work${errors.length ? `; page errors: ${errors.slice(0, 3).join(' / ')}` : ''}`);
  require('fs').writeFileSync(__dirname + '/palette_results.json', JSON.stringify(rows, null, 1));
  await browser.close(); process.exit(failed.length ? 1 : 0);
})().catch((e) => { console.error(e); process.exit(2); });
