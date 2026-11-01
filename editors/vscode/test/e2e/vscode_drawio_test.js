// draw.io's gestures on the diagram, inside the real VS Code: an end of an
// exchange snaps onto a box or a port near the pointer, lines go around the
// boxes after a move, a line drawn straight or curved, a port slid along its
// box or moved onto another, a name dragged off its line, a box resized from
// any side, the grid (Alt to place freely). Needs Playwright and VS Code:
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright node vscode_drawio_test.js
// The compiler is the `arclang` on the PATH, or ARCLANG_BINARY. SHOTS=dir saves pictures.
// Keys are sent as a French (AZERTY) Mac keyboard sends them.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail || ''}`); };

// What a finger on the key labelled X of an AZERTY Mac sends (Playwright's Meta+z arrives as ⌘W there).
const AZ = { a: 'KeyQ', q: 'KeyA', z: 'KeyW', w: 'KeyZ', m: 'Semicolon' };
async function azerty(win) {
  const cdp = await win.context().newCDPSession(win);
  const MOD = { Alt: [1, 'Alt', 'AltLeft', 18], Control: [2, 'Control', 'ControlLeft', 17], Meta: [4, 'Meta', 'MetaLeft', 91], Shift: [8, 'Shift', 'ShiftLeft', 16] };
  return async (combo) => {
    const parts = combo.split('+'); const last = parts.pop(); let mods = 0;
    for (const p of parts) { const m = MOD[p]; mods |= m[0]; await cdp.send('Input.dispatchKeyEvent', { type: 'rawKeyDown', modifiers: mods, key: m[1], code: m[2], windowsVirtualKeyCode: m[3] }); }
    const named = { Escape: 27, Enter: 13 };
    const spec = /^[a-z]$/.test(last) ? { key: last, code: AZ[last] || `Key${last.toUpperCase()}`, keyCode: last.toUpperCase().charCodeAt(0) } : { key: last, code: last, keyCode: named[last] };
    const text = !(mods & 6) && spec.key.length === 1 ? spec.key : undefined;
    await cdp.send('Input.dispatchKeyEvent', { type: text ? 'keyDown' : 'rawKeyDown', modifiers: mods, key: spec.key, code: spec.code, windowsVirtualKeyCode: spec.keyCode, text, unmodifiedText: text });
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', modifiers: mods, key: spec.key, code: spec.code, windowsVirtualKeyCode: spec.keyCode });
    for (const p of parts.reverse()) { const m = MOD[p]; mods &= ~m[0]; await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', modifiers: mods, key: m[1], code: m[2], windowsVirtualKeyCode: m[3] }); }
  };
}

async function launch(dir, model) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'arclang.workbench.doubleClick': 'rename', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(500); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
  await win.waitForTimeout(1200);
  return { close, win, page, press: await azerty(win) };
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcdraw-'));
  const model = path.join(dir, 'model.arc'); const layoutFile = path.join(dir, 'model.layout.json');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const { close, win, page, press } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const layout = () => (fs.existsSync(layoutFile) ? fs.readFileSync(layoutFile, 'utf8') : '');
  const source = async () => { await press('Meta+s'); await win.waitForTimeout(900); return fs.readFileSync(model, 'utf8'); };
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    // Where the webview's own coordinates sit in the window: measured on a box seen both ways.
    let offset = { x: 0, y: 0 };
    const calibrate = async () => {
      const inside = await page.locator('.arcviz-stage svg').first().evaluate((n) => { const r = n.getBoundingClientRect(); return { x: r.x, y: r.y }; });
      const outside = await page.locator('.arcviz-stage svg').first().boundingBox();
      offset = { x: outside.x - inside.x, y: outside.y - inside.y };
    };
    const box = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first().boundingBox();
    const shape = (id) => page.locator(`.arcviz-stage [data-id="${id}"] > .av-shape`).first().boundingBox();
    const centre = (b) => ({ x: b.x + b.width / 2, y: b.y + b.height / 2 });
    const settle = (ms = 2200) => win.waitForTimeout(ms);
    const drag = async (from, to, options = {}) => { await win.mouse.move(from.x, from.y); await win.mouse.down(); await win.mouse.move(to.x, to.y, { steps: options.steps || 14 }); if (options.during) await options.during(); await win.mouse.up(); await settle(options.wait); };
    const sheetPoint = (id, where) => page.locator(`.arcviz-stage [data-id="${id}"]`).first().evaluate((g, where) => { const line = g.querySelector('.av-line'); const len = line.getTotalLength(); const p = line.getPointAtLength(where === 'start' ? 0 : where === 'end' ? len : len / 2); const m = line.getScreenCTM(); return { x: p.x * m.a + p.y * m.c + m.e, y: p.x * m.b + p.y * m.d + m.f }; }, where);
    const edgeAt = async (id, where) => { const p = await sheetPoint(id, where); return { x: p.x + offset.x, y: p.y + offset.y }; };
    const selectEdge = async (id) => { const m = await edgeAt(id, 'middle'); await win.mouse.click(m.x, m.y); await win.waitForTimeout(700); };
    const blank = async () => { const s = await page.locator('.arcviz-stage').boundingBox(); await win.mouse.click(s.x + s.width - 30, s.y + s.height - 30); await win.waitForTimeout(400); };
    // The boxes a line runs through (inside them by more than their border), in sheet units.
    const crossings = (edge) => page.evaluate((edge) => {
      const d = document.querySelector(`.arcviz-stage [data-id="${edge}"] .av-line`).getAttribute('d');
      const nums = d.match(/-?\d+(\.\d+)?/g).map(Number); const pts = []; for (let i = 0; i + 1 < nums.length; i += 2) pts.push([nums[i], nums[i + 1]]);
      const hits = [];
      for (const n of document.querySelectorAll('.arcviz-stage .av-node[data-kind="logical_component"]')) {
        const r = n.querySelector(':scope > .av-shape'); const x = +r.getAttribute('x'), y = +r.getAttribute('y'), w = +r.getAttribute('width'), h = +r.getAttribute('height');
        for (let i = 0; i + 1 < pts.length; i++) { const [x1, y1] = pts[i], [x2, y2] = pts[i + 1]; let hit = false; for (let t = 0.02; t < 1; t += 0.02) { const px = x1 + (x2 - x1) * t, py = y1 + (y2 - y1) * t; if (px > x + 3 && px < x + w - 3 && py > y + 3 && py < y + h - 3) hit = true; } if (hit) { hits.push(n.dataset.id); break; } }
      }
      return hits;
    }, edge);

    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await settle(1500);
    await page.locator('#arcviz-fullscreen').click(); await settle(3500);
    await calibrate();
    await shot('drawio-00-start');

    // 1. Routing around boxes: Camera dragged down beside the actuator
    let b = await box('LC-CAM');
    await drag({ x: b.x + 20, y: b.y + 10 }, { x: b.x + 20, y: b.y + 230 }, { steps: 20 });
    const hits = await crossings('ce:LaneFlow');
    report('a moved box: LaneFlow goes around the other boxes', hits.length === 0, `crosses ${hits.join(',') || 'nothing'}`);
    const lane = await page.locator('.arcviz-stage [data-id="ce:LaneFlow"] .av-line').getAttribute('d');
    report('LaneFlow is still drawn at right angles', !/NaN/.test(lane) && (lane.match(/L/g) || []).length >= 1, lane.slice(0, 60));
    // Attachment: the line still starts on Camera's port and ends on the controller's.
    const near = (p, q) => Math.hypot(p.x - q.x, p.y - q.y) < 14;
    const startOk = near(await edgeAt('ce:LaneFlow', 'start'), centre(await box('LC-CAM::lane')));
    const endOk = near(await edgeAt('ce:LaneFlow', 'end'), centre(await box('LC-CTL::lane')));
    report('LaneFlow stays attached to both of its ports', startOk && endOk, `${startOk} ${endOk}`);
    await shot('drawio-01-routed');

    // 2. Grid: a box let go lands on the 10-unit grid; with Alt, where it is let go
    const gridX = async (id) => +(await page.locator(`.arcviz-stage [data-id="${id}"] > .av-shape`).first().getAttribute('x'));
    b = await box('LC-ACT');
    await drag({ x: b.x + 20, y: b.y + 10 }, { x: b.x + 20 + 37, y: b.y + 10 + 23 });
    const onGrid = await gridX('LC-ACT');
    report('a dragged box lands on the grid', Math.abs(onGrid / 10 - Math.round(onGrid / 10)) < 0.01, `x = ${onGrid}`);
    b = await box('LC-ACT');
    await win.keyboard.down('Alt'); await drag({ x: b.x + 20, y: b.y + 10 }, { x: b.x + 20 + 13, y: b.y + 10 }); await win.keyboard.up('Alt');
    const free = await gridX('LC-ACT');
    report('with Alt held it goes exactly where it is let go', Math.abs(free / 10 - Math.round(free / 10)) > 0.05, `x = ${free}`);

    // 3. Resize from every side: eight handles; the left one keeps what the box holds in place
    await blank();
    const wide0 = await page.evaluate(() => innerWidth);
    let s = await shape('LC-CTL'); await win.mouse.click(s.x + 30, s.y + 8); await win.waitForTimeout(800);
    // The selection follows into the text only out of full screen: shown, the text would take half the window.
    const wide1 = await page.evaluate(() => innerWidth);
    report('a box selected in full screen keeps the full screen', wide1 === wide0, `${wide0} -> ${wide1}`);
    const handles = await page.locator('.arcviz-resize:not([hidden])').count();
    report('the selected box shows eight resize handles', handles === 8, `${handles}`);
    const inner0 = await shape('LF-PLAN'); const ref0 = await shape('LC-ACT');
    const nw = await page.locator('.arcviz-resize.is-nw').boundingBox();
    await drag(centre(nw), { x: centre(nw).x - 60, y: centre(nw).y - 40 });
    // Relative to another box: a box grown past the top of the sheet moves the whole drawing down.
    const ref1 = await shape('LC-ACT'); const rel = (a, r0, r1) => ({ x: a.x - (r1.x - r0.x), y: a.y - (r1.y - r0.y), width: a.width, height: a.height });
    const s1 = rel(await shape('LC-CTL'), ref0, ref1); const inner1 = rel(await shape('LF-PLAN'), ref0, ref1);
    report('the top-left handle grows the box up and to the left', s1.x < s.x - 40 && s1.y < s.y - 25 && s1.width > s.width + 40 && s1.height > s.height + 25, `${Math.round(s1.x - s.x)},${Math.round(s1.y - s.y)} +${Math.round(s1.width - s.width)}x${Math.round(s1.height - s.height)}`);
    report('what the box holds stays where it was', Math.abs(inner1.x - inner0.x) < 6 && Math.abs(inner1.y - inner0.y) < 6, `${Math.round(inner1.x - inner0.x)},${Math.round(inner1.y - inner0.y)}`);
    report('the size is kept in the layout file', /"LC-CTL":\s*\{[^}]*"dw"/.test(layout()));
    const w = await page.locator('.arcviz-resize.is-w').boundingBox();
    await drag(centre(w), { x: centre(w).x + 600, y: centre(w).y });
    const s2 = await shape('LC-CTL'); const inner2 = await shape('LF-PLAN');
    report('shrunk past what it holds, the box stops at its content', s2.x <= inner2.x && s2.x + s2.width >= inner2.x + inner2.width && s2.width >= 40, `${Math.round(s2.width)} wide`);
    await shot('drawio-03-resized');

    // 4. Line styles from the selected exchange, kept in the layout file
    await blank(); await selectEdge('ce:SteerCommand');
    const bar = page.locator('.arcviz-edge-style:not([hidden])');
    report('a selected exchange offers its line styles', (await bar.count()) === 1, JSON.stringify(await page.evaluate(() => ({ selected: [...document.querySelectorAll('.is-selected')].map((n) => n.dataset.id), ends: document.querySelectorAll('.arcviz-end:not([hidden])').length, bars: document.querySelectorAll('.arcviz-edge-style').length }))));
    await shot('drawio-04-selected');
    const inToolbar = await page.evaluate(() => { const b = document.querySelector('.arcviz-edge-style:not([hidden])'); return !!b && !!b.closest('.wb-mini') && !document.querySelector('.wb-mini').hidden; });
    report('the line styles ride in the selection toolbar, not under it', inToolbar);
    await page.locator('.arcviz-edge-style-option[data-style="curved"]').click(); await settle();
    report('curved: the line is drawn as a curve', (await page.locator('.arcviz-stage [data-id="ce:SteerCommand"][data-style="curved"]').count()) === 1);
    report('the style is kept in the layout file', /"ce:SteerCommand":\s*\{[^}]*"style": "curved"/.test(layout()));
    await shot('drawio-04-curved');
    await selectEdge('ce:SteerCommand');
    await page.locator('.arcviz-edge-style-option[data-style="straight"]').click(); await settle();
    const straight = await page.locator('.arcviz-stage [data-id="ce:SteerCommand"] .av-line').getAttribute('d');
    report('straight: one run from end to end', (straight.match(/[LQ]/g) || []).length === 1, straight.slice(0, 60));
    await selectEdge('ce:SteerCommand');
    await page.locator('.arcviz-edge-style-option[data-style="orthogonal"]').click(); await settle();
    report('back to right angles, the style leaves the file', !/"style"/.test(layout()));

    // 5. A port slid along its box, kept in the layout file; the exchange follows it
    await blank();
    let p = await box('LC-CTL::steer');
    await drag(centre(p), { x: centre(p).x, y: centre(p).y + 30 }, { steps: 10 });
    let p2 = await box('LC-CTL::steer');
    report('a port slides along the border of its box', Math.abs(p2.y - p.y - 30) < 10 && Math.abs(p2.x - p.x) < 4, `${Math.round(p2.x - p.x)},${Math.round(p2.y - p.y)}`);
    report('its place is kept in the layout file', /"ports":\s*\{\s*"LC-CTL::steer"/.test(layout()));
    report('its exchange leaves from where it now is', near(await edgeAt('ce:SteerCommand', 'start'), centre(p2)));
    // Away into the empty sheet beside the box (not onto another box, which would move the port there).
    p = p2; await drag(centre(p), { x: centre(p).x + 160, y: centre(p).y + 20 }, { steps: 10 });
    p2 = await box('LC-CTL::steer'); const ctl = await shape('LC-CTL');
    const onBorder = Math.min(Math.abs(centre(p2).x - ctl.x), Math.abs(centre(p2).x - ctl.x - ctl.width), Math.abs(centre(p2).y - ctl.y), Math.abs(centre(p2).y - ctl.y - ctl.height)) < 8;
    report('pulled away from its box, a port stays on its border', onBorder, JSON.stringify(centre(p2)));
    await shot('drawio-05-port');

    // 6. A name dragged off its line, kept in the layout file; the sheet does not move
    await blank();
    const label = page.locator('.arcviz-stage [data-id="ce:LaneFlow"] .av-edge-label').first();
    const l0 = await label.boundingBox(); const cam0 = await box('LC-CAM');
    await drag(centre(l0), { x: centre(l0).x + 50, y: centre(l0).y + 40 }, { steps: 10 });
    const l1 = await label.boundingBox(); const cam1 = await box('LC-CAM');
    report('a name is dragged off its line', Math.abs(l1.x - l0.x - 50) < 10 && Math.abs(l1.y - l0.y - 40) < 10, `${Math.round(l1.x - l0.x)},${Math.round(l1.y - l0.y)}`);
    report('the sheet stays where it was', Math.abs(cam1.x - cam0.x) < 1 && Math.abs(cam1.y - cam0.y) < 1);
    report('the offset of the name is kept in the layout file', /"label":\s*\{\s*"dx"/.test(layout()));
    await shot('drawio-06-label');

    // 7. Snap: an end dragged near a box lands on it, and over its body too
    await blank(); await selectEdge('ce:SteerCommand');
    const end = await page.locator('.arcviz-end[data-end="target"]').boundingBox();
    const cam = await shape('LC-CAM'); const capture = await shape('LF-CAP');
    const before = await source();
    await win.mouse.move(centre(end).x, centre(end).y); await win.mouse.down();
    await win.mouse.move(cam.x - 12, cam.y + cam.height * 0.25, { steps: 15 }); await win.waitForTimeout(300);
    const nearMarks = await page.evaluate(() => ({ box: [...document.querySelectorAll('.is-snap-box')].map((n) => n.dataset.id), target: [...document.querySelectorAll('.is-drop-target')].map((n) => n.dataset.id) }));
    await shot('drawio-07-snap-near');
    report('12 px outside a box, its outline lights and it is the target', nearMarks.box.includes('LC-CAM') && nearMarks.target.includes('LC-CAM'), JSON.stringify(nearMarks));
    await win.mouse.move(centre(capture).x, centre(capture).y, { steps: 6 }); await win.waitForTimeout(300);
    const bodyMarks = await page.evaluate(() => [...document.querySelectorAll('.is-drop-target')].map((n) => n.dataset.id));
    report('over the body of the box (on what it holds), still the box', bodyMarks.includes('LC-CAM'), bodyMarks.join(','));
    await win.mouse.up(); await settle(4000);
    const after = await source();
    report('let go there, it reconnects, a port made for it', after !== before && /to_port: "LC-CAM\.steer"/.test(after) && /id: "LC-CAM"[\s\S]{0,200}port in "steer"/.test(after), (after.match(/component_exchange "SteerCommand" \{[\s\S]*?\}/) || [''])[0].replace(/\s+/g, ' '));
    await shot('drawio-07-snapped');

    // 8. A port let go on another box moves there (the model's Move)
    await blank();
    const lanePort = await box('LC-CAM::lane'); const act = await shape('LC-ACT');
    const text0 = await source();
    await drag(centre(lanePort), { x: act.x + act.width / 2, y: act.y + act.height - 10 }, { steps: 18, wait: 4000 });
    const text1 = await source();
    const actBlock = (text1.split('component "Steering Actuator"')[1] || '').split('component_exchange')[0];
    report('a port let go on another box is moved into it', text1 !== text0 && /port out "lane"/.test(actBlock), text1 === text0 ? 'source unchanged' : '');
    await shot('drawio-08-port-moved');

    // 9. Reset layout: back to the automatic layout, every arrangement gone from the file
    await page.locator('#arcviz-layout-menu').click().catch(() => {}); await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="reset-layout"]').click().catch(() => {});
    await settle(2500);
    const reset = layout();
    report('Reset layout drops the arrangement of the view', !/"LC-CTL"/.test(reset) && !/"ports"/.test(reset) && !/"label"/.test(reset), reset.slice(0, 120).replace(/\s+/g, ' '));
  } catch (error) {
    report('the run completes', false, String(error).slice(0, 300)); await shot('drawio-failure').catch(() => {});
  } finally {
    await close().catch(() => {});
    const failed = rows.filter((r) => !r.ok);
    console.log(`\n${rows.length - failed.length}/${rows.length} passed`);
    process.exit(failed.length ? 1 : 0);
  }
})();
