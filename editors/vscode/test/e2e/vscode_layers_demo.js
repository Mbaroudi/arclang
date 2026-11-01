// A recorded demo of Arrange in the real VS Code (order, layers, groups, locks), keys sent as an
// AZERTY Mac sends them, with a caption on screen for each step. The window's frames are taken
// through the debugging protocol; ffmpeg makes arclang-arrange.webm and .mp4 of them under DEMO_DIR.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang DEMO_DIR=/path/to/out node vscode_layers_demo.js
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path'); const { execFileSync } = require('child_process');
const { azerty } = require('./azerty');
const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const OUT = process.env.DEMO_DIR || path.join(os.tmpdir(), 'arclang-demo');

/** The frames the window paints, kept with their time; `stop()` writes them and ffmpeg's list of them in `dir`. */
async function screencast(win, dir) {
  const cdp = await win.context().newCDPSession(win);
  const frames = [];
  cdp.on('Page.screencastFrame', ({ data, metadata, sessionId }) => { frames.push({ data, t: metadata.timestamp }); cdp.send('Page.screencastFrameAck', { sessionId }).catch(() => undefined); });
  await cdp.send('Page.startScreencast', { format: 'jpeg', quality: 90, maxWidth: 1600, maxHeight: 1000 });
  return async () => {
    await cdp.send('Page.stopScreencast').catch(() => undefined);
    if (!frames.length) return null;
    fs.mkdirSync(dir, { recursive: true });
    const lines = frames.map((f, i) => {
      const name = path.join(dir, `frame-${String(i).padStart(5, '0')}.jpg`);
      fs.writeFileSync(name, Buffer.from(f.data, 'base64'));
      const next = frames[i + 1] ? frames[i + 1].t : f.t + 1;
      return `file '${name}'\nduration ${(next - f.t).toFixed(3)}`;
    });
    // The concat demuxer takes the last frame's duration only when the frame is listed once more.
    const list = path.join(dir, 'frames.txt');
    fs.writeFileSync(list, `${lines.join('\n')}\nfile '${path.join(dir, `frame-${String(frames.length - 1).padStart(5, '0')}.jpg`)}'\n`);
    return list;
  };
}

async function workbenchPage(win) {
  for (let i = 0; i < 60; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) return f; } catch { /* going away */ } } }
  return null;
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcdemo-'));
  const model = path.join(dir, 'model.arc'); fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'workbench.colorTheme': 'Default Dark Modern', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true }); fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  const stopRecording = await screencast(win, path.join(OUT, 'raw'));
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  await command('View: Close Secondary Side Bar'); await command('ArcLang: Open the workbench');
  const page = await workbenchPage(win); const press = await azerty(win);
  const caption = (text, keys) => win.evaluate(([t, k]) => {
    let box = document.getElementById('demo-caption');
    if (!box) { box = document.createElement('div'); box.id = 'demo-caption'; box.style.cssText = 'position:fixed;left:50%;bottom:34px;transform:translateX(-50%);z-index:99999;padding:10px 20px;border-radius:10px;background:rgba(20,22,28,.92);color:#fff;font:600 19px system-ui;display:flex;gap:14px;align-items:center;pointer-events:none;box-shadow:0 6px 24px rgba(0,0,0,.5)'; document.body.appendChild(box); }
    box.replaceChildren(); const s = document.createElement('span'); s.textContent = t; box.appendChild(s);
    if (k) { const kb = document.createElement('kbd'); kb.textContent = k; kb.style.cssText = 'padding:3px 9px;border-radius:6px;background:#3b82f6;font:700 17px ui-monospace,monospace'; box.appendChild(kb); }
  }, [text, keys || '']);
  const wait = (ms) => win.waitForTimeout(ms);
  try {
    const node = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first();
    const shape = (id) => node(id).locator(':scope > .av-shape').first().boundingBox();
    const tool = (id) => page.locator(`#${id}`);
    const drag = async (from, to, steps = 28) => { await win.mouse.move(from.x, from.y); await win.mouse.down(); await win.mouse.move(to.x, to.y, { steps }); await win.mouse.up(); await wait(1800); };
    const click = async (id, shift) => { const b = await shape(id); if (shift) await win.keyboard.down('Shift'); await win.mouse.click(b.x + b.width - 30, b.y + 8); if (shift) await win.keyboard.up('Shift'); await wait(700); };
    const blank = async () => { const s = await page.locator('.arcviz-stage').boundingBox(); await win.mouse.click(s.x + s.width - 30, s.y + s.height - 30); await wait(500); };
    const fit = async () => { await tool('arcviz-fit').click(); await wait(1500); };
    const prompt = async (text) => { const i = page.locator('.wb-prompt input').first(); await i.waitFor({ timeout: 4000 }); await i.pressSequentially(text, { delay: 90 }); await wait(500); await i.press('Enter'); await wait(3000); };
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await wait(2000); await fit();
    await caption('ArcLang workbench: draw.io-style Arrange, on a real model'); await wait(3500);

    await caption('Two boxes overlap');
    let act = await shape('LC-ACT'); const cam = await shape('LC-CAM');
    await drag({ x: act.x + act.width - 12, y: act.y + 8 }, { x: act.x + act.width - 12 - (act.x - cam.x - 90), y: act.y + 8 - (act.y - cam.y - 30) }); await wait(1500);
    await click('LC-ACT'); await caption('Send to back', '⇧⌘B'); await wait(800); await press('Meta+Shift+b'); await wait(3000);
    await click('LC-ACT'); await caption('Bring forward: one step past the box it overlaps', '⌥⇧⌘F'); await wait(800); await press('Meta+Alt+Shift+f'); await wait(3000);
    await click('LC-ACT'); await caption('Undo: one history for the model and the arrangement', '⌘Z'); await wait(800); await press('Meta+z'); await wait(3500);
    await click('LC-ACT'); await caption('Bring to front', '⇧⌘F'); await wait(800); await press('Meta+Shift+f'); await wait(3000);

    await blank(); await caption('Layers: one per tag of the model', '⇧⌘L'); await wait(800); await press('Meta+Shift+l'); await wait(2500);
    await caption('New layer "Safety"'); await page.locator('.wb-tl button', { hasText: 'New layer' }).click(); await prompt('Safety');
    await click('LC-ACT'); await caption('Move the selection to the layer: it sets the element\'s tag in the model'); await wait(1500);
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-move').click(); await wait(4500);
    await caption('Hide the layer: its boxes and exchanges go, and exports leave them out'); await wait(1200);
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-eye').click(); await wait(3500);
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-eye').click(); await wait(2500);
    await caption('Lock the layer: nothing on it can be moved'); await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-lock').click(); await wait(2000);
    const lb = await shape('LC-ACT'); await drag({ x: lb.x + lb.width - 30, y: lb.y + 8 }, { x: lb.x + lb.width + 40, y: lb.y + 80 }); await wait(1500);
    await page.locator('.wb-tl-row[data-layer="Safety"] .wb-tl-lock').click(); await wait(1200);
    await caption('Reorder layers: a higher layer is drawn over the others'); await page.locator('.wb-tl-row[data-layer="Safety"] button[aria-label^="Move Safety down"]').click(); await wait(3000);
    await page.locator('.wb-tl button[title^="Close"]').click(); await wait(800);

    await fit(); await blank(); await click('LC-CAM'); await click('LC-CTL', true);
    await caption('Group the selected boxes', '⌘G'); await wait(800); await press('Meta+g'); await wait(3000);
    const gf = await page.locator('.arcviz-stage .av-group > .av-shape').boundingBox();
    await caption('The group moves as one'); await drag({ x: gf.x + gf.width / 2, y: gf.y + 6 }, { x: gf.x + gf.width / 2 + 60, y: gf.y + 6 + 50 }); await wait(1500);
    await caption('Collapse it to one box'); await click('LC-CAM'); const rc = page.locator('.arcviz-stage .is-selected').first();
    const sb = await page.locator('.arcviz-stage .av-group > .av-shape').boundingBox(); await win.mouse.click(sb.x + sb.width / 2, sb.y + 6, { button: 'right' }); await wait(800);
    await page.locator('.wb-menu:not([hidden]) button', { hasText: 'Collapse group' }).first().click(); await wait(3500);
    await page.locator('.av-group.is-collapsed').dblclick(); await wait(2500);
    await click('LC-CAM'); await caption('Ungroup', '⇧⌘G'); await wait(800); await press('Meta+Shift+g'); await wait(3000);

    await fit(); await blank(); await click('LC-CAM'); await click('LC-CTL', true);
    await caption('Group into a new component: a real change of the model, one undo'); await wait(1500);
    const cb = await shape('LC-CTL'); await win.mouse.click(cb.x + cb.width - 30, cb.y + 8, { button: 'right' }); await wait(1000);
    await page.locator('.wb-menu:not([hidden]) button', { hasText: 'Group into a new component' }).first().click(); await prompt('Sensing'); await wait(3500);
    await caption('Undo', '⌘Z'); await wait(800); await press('Meta+z'); await wait(3500);

    await fit(); await blank(); await click('LC-CAM'); await caption('Lock an element', '⌘L'); await wait(800); await press('Meta+l'); await wait(2500);
    await caption('Delete is refused, and says why'); await wait(800); await press('Delete'); await wait(4000);
    await click('LC-CAM'); await press('Meta+l'); await wait(1500);
    await blank(); await caption('Select every line', '⇧⌘E'); await wait(800); await press('Meta+Shift+e'); await wait(3000);
    await caption('ArcLang: models, diagrams and layout that never lie'); await wait(4000);
  } finally {
    const list = await stopRecording(); await close().catch(() => {});
    if (list) {
      const input = ['-y', '-f', 'concat', '-safe', '0', '-i', list, '-vf', 'pad=ceil(iw/2)*2:ceil(ih/2)*2', '-fps_mode', 'vfr'];
      try {
        execFileSync('ffmpeg', [...input, '-c:v', 'libvpx-vp9', '-b:v', '0', '-crf', '32', path.join(OUT, 'arclang-arrange.webm')], { stdio: 'ignore' });
        execFileSync('ffmpeg', [...input, '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-crf', '20', '-movflags', '+faststart', path.join(OUT, 'arclang-arrange.mp4')], { stdio: 'ignore' });
        console.log('video in', OUT);
      } catch { console.log('frames in', path.dirname(list), '(ffmpeg makes the video of them)'); }
    } else console.log('no video recorded');
  }
})();
