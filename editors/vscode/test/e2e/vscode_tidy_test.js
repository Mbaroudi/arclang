// Tidying a diagram up after a move by hand, inside the real VS Code: a box
// dragged over another makes the diagram harder to read, so the workbench
// offers to tidy it up; Tidy up removes the overlap keeping the order of the
// boxes, writes the layout file, and Cmd+Z brings the move back; Keep my
// layout leaves it; Always tidy up sets arclang.workbench.reorganize and
// tidies the next such move by itself; the Layout menu offers the same.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_tidy_test.js
// SHOTS=dir saves pictures. Keys are sent as a French (AZERTY) Mac keyboard sends them.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail || ''}`); };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arctidy-'));
  const model = path.join(dir, 'model.arc'); const layoutFile = path.join(dir, 'model.layout.json');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const userSettings = path.join(dir, 'u', 'User', 'settings.json');
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.dirname(userSettings), { recursive: true });
  fs.writeFileSync(userSettings, JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(600); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
  const press = await azerty(win);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const settle = (ms = 2500) => win.waitForTimeout(ms);
  const box = (id) => page.locator(`.arcviz-stage [data-id="${id}"] > .av-shape`).first().boundingBox();
  const overlapping = (a, b) => a.x < b.x + b.width - 2 && b.x < a.x + a.width - 2 && a.y < b.y + b.height - 2 && b.y < a.y + a.height - 2;
  const offer = () => page.locator('.arcviz-tidy-offer');
  // ACT dragged over the right half of CAM, held by its bottom-right corner so the pointer is let go beside CAM, not on it.
  const dragOver = async () => {
    const cam = await box('LC-CAM'); const act = await box('LC-ACT');
    const to = { x: cam.x + cam.width * 0.5, y: cam.y + 12 };
    const from = { x: act.x + act.width - 8, y: act.y + act.height - 8 };
    await win.mouse.move(from.x, from.y); await win.mouse.down();
    await win.mouse.move(from.x + (to.x - act.x), from.y + (to.y - act.y), { steps: 16 }); await win.mouse.up();
    await settle();
  };
  // Where the line SteerCommand ends, in the window: it must reach the Actuator box.
  const lineEnd = (label) => page.locator('.arcviz-stage .av-edge', { hasText: label }).first().locator('.av-line').first().evaluate((line) => {
    const p = line.getPointAtLength(line.getTotalLength()); const m = line.getScreenCTM();
    return { x: p.x * m.a + p.y * m.c + m.e, y: p.x * m.b + p.y * m.d + m.f };
  });
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await settle(1500);
    const order0 = ['LC-CAM', 'LC-CTL', 'LC-ACT'];
    const startAt = await Promise.all(order0.map(box));
    report('the logical view shows the three components apart', !startAt.some((a, i) => startAt.some((b, j) => j > i && overlapping(a, b))));

    // 1. A move that makes the diagram harder to read: the offer.
    await dragOver();
    const moved = [await box('LC-CTL'), await box('LC-ACT')];
    await shot('tidy-00-moved');
    // Both in the webview's own coordinates.
    const end = await lineEnd('SteerCommand').catch(() => null);
    const actInside = await page.locator('.arcviz-stage [data-id="LC-ACT"] > .av-shape').first().evaluate((n) => { const r = n.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; });
    const near = (p, b) => p && p.x > b.x - 30 && p.x < b.x + b.width + 30 && p.y > b.y - 30 && p.y < b.y + b.height + 30;
    report('the line to the moved box follows it', near(end, actInside), JSON.stringify(end && [Math.round(end.x), Math.round(end.y), Math.round(actInside.x), Math.round(actInside.y)]));
    const cam1 = await box('LC-CAM');
    report('the box was let go over the other (not into it)', overlapping(cam1, moved[1]), JSON.stringify([cam1, moved[1]].map((b) => [Math.round(b.x), Math.round(b.y)])));
    const offered = await offer().count() > 0;
    const text = offered ? await offer().innerText() : '';
    report('the workbench offers to tidy the diagram up, saying why', offered && /harder to read: 1 box overlaps another/.test(text), text.replace(/\s+/g, ' ').slice(0, 120));
    await shot('tidy-01-offer');

    // 2. Tidy up: no overlap, the order kept, the layout file written.
    const layoutBefore = fs.existsSync(layoutFile) ? fs.readFileSync(layoutFile, 'utf8') : '';
    await page.locator('.arcviz-tidy-offer .arcviz-tidy-yes').click(); await settle(3000);
    const after = await Promise.all(order0.map(box));
    const none = !after.some((a, i) => after.some((b, j) => j > i && overlapping(a, b)));
    report('Tidy up removes the overlap', none, JSON.stringify(after.map((b) => [Math.round(b.x), Math.round(b.y)])));
    report('the offer is gone', (await offer().count()) === 0);
    const layoutAfter = fs.existsSync(layoutFile) ? fs.readFileSync(layoutFile, 'utf8') : '';
    report('the tidy layout is written to the layout file', layoutAfter !== layoutBefore && /LC-ACT|LC-CTL|LC-CAM/.test(layoutAfter));
    await shot('tidy-02-tidied');

    // 3. Cmd+Z brings the move by hand back.
    await page.locator('.arcviz-stage').click({ position: { x: 20, y: 20 } }); await win.waitForTimeout(300);
    await press('Meta+z'); await settle(2500);
    const undone = [await box('LC-CAM'), await box('LC-ACT')];
    report('Cmd+Z undoes the tidy-up: the boxes overlap again', overlapping(undone[0], undone[1]));

    // 4. Keep my layout: nothing moves, the offer closes.
    await press('Meta+z'); await settle(2500); // back to the row
    await dragOver();
    const kept = [await box('LC-CAM'), await box('LC-ACT')];
    await page.locator('.arcviz-tidy-offer .arcviz-tidy-keep').click(); await settle(800);
    const still = [await box('LC-CAM'), await box('LC-ACT')];
    report('Keep my layout leaves the boxes where they are', (await offer().count()) === 0 && Math.abs(still[1].x - kept[1].x) < 1 && overlapping(still[0], still[1]));

    // 5. Always tidy up: the setting, then the next such move tidied by itself.
    await press('Meta+z'); await settle(2500);
    await dragOver();
    await page.locator('.arcviz-tidy-offer .arcviz-tidy-always').click(); await settle(3500);
    const saved = JSON.parse(fs.readFileSync(userSettings, 'utf8'))['arclang.workbench.reorganize'];
    report('Always tidy up sets arclang.workbench.reorganize to always', saved === 'always', String(saved));
    // Two steps back: the tidy-up, then the move by hand; the boxes apart again.
    await press('Meta+z'); await settle(3000); await press('Meta+z'); await settle(3000);
    const apart = await Promise.all(order0.map(box));
    report('two undos bring the boxes back apart', !apart.some((a, i) => apart.some((b, j) => j > i && overlapping(a, b))));
    await dragOver(); await settle(4000);
    const auto = await Promise.all(order0.map(box));
    const said = await page.locator('.arcviz-status').first().textContent().catch(() => '');
    report('then a move that overlaps is tidied up by itself, no question', (await offer().count()) === 0 && !auto.some((a, i) => auto.some((b, j) => j > i && overlapping(a, b))), `${said} ${JSON.stringify(auto.map((b) => [Math.round(b.x), Math.round(b.y)]))}`);
    await shot('tidy-03-always');

    // 6. The Layout menu: the tidy-up and the three choices.
    await page.locator('#arcviz-layout-menu, .arcviz-layout button').first().click(); await win.waitForTimeout(500);
    const items = await page.locator('[role="menuitem"], [role="menuitemcheckbox"], [role="menuitemradio"]').allInnerTexts();
    report('Layout offers Tidy up and the three choices', ['Tidy up, keeping my arrangement', 'ask to tidy up', 'always tidy up', 'never ask'].every((t) => items.some((i) => i.includes(t))), items.filter((i) => /tidy|never/i.test(i)).join(' | '));
    await press('Escape');
  } catch (error) {
    report('the test ran to its end', false, String(error).slice(0, 240));
    await shot('tidy-error').catch(() => undefined);
  } finally {
    await close().catch(() => undefined);
  }
  const failed = rows.filter((r) => !r.ok).length;
  console.log(`\n${rows.length - failed}/${rows.length} checks passed`);
  process.exit(failed ? 1 : 0);
})();
