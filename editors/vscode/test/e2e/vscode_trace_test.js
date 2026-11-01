// A trace drawn on a diagram, inside the real VS Code: a trace between two
// elements the view draws is a dashed line labelled by its type; selected, it
// is the trace; an end dragged onto another box reconnects it (the trace line
// of the source changes, nothing else); Delete removes it, ⌘Z brings it back.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_trace_test.js
// SHOTS=dir saves pictures. Keys are sent as a French (AZERTY) Mac sends them.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail || ''}`); };

async function launch(dir, model) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
  await win.waitForTimeout(1500);
  return { close, win, page, press: await azerty(win) };
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arctrace-'));
  const model = path.join(dir, 'model.arc');
  const reference = fs.readFileSync(path.join(EXT, 'media', 'reference.arc'), 'utf8');
  fs.writeFileSync(model, `${reference.trimEnd()}\ntrace "LC-CAM" allocates "LC-CTL"\n`);
  const { close, win, page, press } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const source = async () => { await press('Meta+s'); await win.waitForTimeout(900); return fs.readFileSync(model, 'utf8'); };
  const settle = (ms = 3000) => win.waitForTimeout(ms);
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    // The components are on the logical view.
    await page.locator('button, [role="tab"]', { hasText: /^Logical/ }).first().click(); await settle(2500);
    const inside = await page.locator('.arcviz-stage svg').first().evaluate((n) => { const r = n.getBoundingClientRect(); return { x: r.x, y: r.y }; });
    const outside = await page.locator('.arcviz-stage svg').first().boundingBox();
    const offset = { x: outside.x - inside.x, y: outside.y - inside.y };
    const traceEdge = () => page.locator('.arcviz-stage [data-id^="trc:"]');
    const middle = (locator) => locator.first().evaluate((g) => { const line = g.querySelector('.av-line'); const len = line.getTotalLength(); const p = line.getPointAtLength(len / 2); const m = line.getScreenCTM(); return { x: p.x * m.a + p.y * m.c + m.e, y: p.x * m.b + p.y * m.d + m.f }; });
    const shape = (id) => page.locator(`.arcviz-stage [data-id="${id}"] > .av-shape`).first().boundingBox();

    // 1. Drawn: a dashed line from the camera to the controller, labelled by its type
    const drawn = await traceEdge().evaluateAll((gs) => gs.map((g) => ({ id: g.dataset.id, dash: getComputedStyle(g.querySelector('.av-line')).strokeDasharray, text: g.textContent.trim() })));
    report('the trace between two drawn components is a dashed line', drawn.length === 1 && drawn[0].id === 'trc:LC-CAM allocates LC-CTL' && drawn[0].dash !== 'none', JSON.stringify(drawn));
    report('its label is its type', drawn.length === 1 && /allocates/.test(drawn[0].text), drawn[0]?.text);
    await shot('trace-01-drawn');

    // 2. Selected: the properties show the trace
    let m = await middle(traceEdge()); await win.mouse.click(m.x + offset.x, m.y + offset.y); await win.waitForTimeout(900);
    const kind = await page.locator('.wb-properties').innerText().catch(() => '');
    report('clicked, it is selected as the trace, its type shown', /Trace/.test(kind) && /allocates/.test(kind), kind.split('\n').slice(0, 3).join(' | '));

    // 3. Reconnected: its target end dragged onto the actuator
    const end = page.locator('.arcviz-end[data-end="target"]');
    const endShown = await end.isVisible().catch(() => false);
    report('selected, its ends can be dragged', endShown);
    const before = await source();
    if (endShown) {
      const e = await end.boundingBox(); const actuator = await shape('LC-ACT');
      await win.mouse.move(e.x + e.width / 2, e.y + e.height / 2); await win.mouse.down();
      await win.mouse.move(actuator.x + actuator.width / 2, actuator.y + 14, { steps: 16 }); await win.waitForTimeout(300);
      await shot('trace-02-dragging');
      await win.mouse.up(); await settle(4000);
    }
    const after = await source();
    const changed = before.split('\n').filter((line, i) => line !== after.split('\n')[i]);
    report('let go on another component, the trace goes there', /\ntrace "LC-CAM" allocates "LC-ACT"\n/.test(after) && !/allocates "LC-CTL"/.test(after), (after.match(/trace "LC-CAM" allocates[^\n]*/) || [''])[0]);
    report('nothing else in the file changed', changed.length === 1 && before.split('\n').length === after.split('\n').length, `${changed.length} line(s)`);
    await shot('trace-03-reconnected');

    // 4. Deleted: Delete on the selected trace removes its line; ⌘Z brings it back
    await settle(1500);
    m = await middle(traceEdge()); await win.mouse.click(m.x + offset.x, m.y + offset.y); await win.waitForTimeout(800);
    await press('Backspace'); await settle(4000);
    const removed = await source();
    report('Delete removes the trace line, the rest stays', !/allocates/.test(removed) && removed.split('\n').length === after.split('\n').length - 1);
    report('and the line is gone from the diagram', (await traceEdge().count()) === 0);
    await press('Meta+z'); await settle(4000);
    const back = await source();
    report('⌘Z brings it back', /trace "LC-CAM" allocates "LC-ACT"/.test(back));
    await shot('trace-04-undo');
  } catch (error) {
    report('the test ran to its end', false, error.message);
  } finally {
    await close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
    const failed = rows.filter((r) => !r.ok).length;
    console.log(`${rows.length - failed}/${rows.length} passed`);
    process.exit(failed ? 1 : 0);
  }
})();
