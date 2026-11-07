// Notes on a diagram, inside the real VS Code: a note of the file is drawn on
// the view of the elements it is about, tied to them by dashed lines; the Note
// tool writes one about the selected element; Delete removes it, ⌘Z brings it back.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_note_test.js
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
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcnote-'));
  const model = path.join(dir, 'model.arc');
  const reference = fs.readFileSync(path.join(EXT, 'media', 'reference.arc'), 'utf8');
  fs.writeFileSync(model, `${reference.trimEnd()}\nnote "The camera delivers a lane model every 33 ms, within the chain budget" { id: "N-1" about: ["LC-CAM"] }\n`);
  const { close, win, page, press } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const source = async () => { await press('Meta+s'); await win.waitForTimeout(900); return fs.readFileSync(model, 'utf8'); };
  const settle = (ms = 3000) => win.waitForTimeout(ms);
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    await page.locator('button, [role="tab"]', { hasText: /^Logical/ }).first().click(); await settle(2500);
    const node = (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first();
    const shape = (id) => node(id).locator(':scope > .av-shape').first().boundingBox();

    // 1. Drawn: the note beside the camera, tied to it
    const text = await node('N-1').textContent().catch(() => '');
    report('the note is drawn on the logical view with its text', /lane model ?every 33 ms/.test(text || ''), (text || '').slice(0, 60));
    const tie = await page.locator('.arcviz-stage [data-id="ann:N-1 LC-CAM"] .av-line').first().evaluate((l) => getComputedStyle(l).strokeDasharray).catch(() => null);
    report('a dashed line ties it to the camera', !!tie && tie !== 'none', String(tie));
    await shot('note-01-drawn');

    // 2. The Note tool writes one about the selected controller
    const ctl = await shape('LC-CTL'); await win.mouse.click(ctl.x + ctl.width - 30, ctl.y + 8); await win.waitForTimeout(700);
    await page.locator('[data-tool="note"]').first().click(); await win.waitForTimeout(500);
    const input = page.locator('.wb-prompt input').first();
    const asked = await input.isVisible().catch(() => false);
    report('the Note tool asks for the text of a note about the selection', asked, asked ? await page.locator('.wb-prompt label').first().innerText() : '');
    if (asked) { await input.fill('Plans the correction within one frame'); await input.press('Enter'); await settle(4000); }
    const written = await source();
    report('the note is written at the top level, about the controller', /\nnote "Plans the correction within one frame" \{ about: \["LC-CTL"\] \}\n/.test(written), (written.match(/note "Plans[^\n]*/) || [''])[0]);
    const created = await page.locator('.arcviz-stage .av-node').evaluateAll((ns) => ns.filter((n) => /Plans the correction/.test(n.textContent)).length);
    report('and drawn beside it at once', created === 1);
    await shot('note-02-created');

    // 3. Delete removes the selected note; ⌘Z brings it back
    const n1 = await shape('N-1'); await win.mouse.click(n1.x + 12, n1.y + n1.height - 6); await win.waitForTimeout(700);
    const kind = await page.locator('.wb-properties').innerText().catch(() => '');
    report('clicked, the note is selected', /Note/.test(kind), kind.split('\n').slice(0, 2).join(' | '));
    await press('Backspace'); await settle(4000);
    const removed = await source();
    report('Delete removes its declaration, the other note stays', !/N-1/.test(removed) && /Plans the correction/.test(removed));
    report('and it leaves the diagram', (await node('N-1').count()) === 0);
    await press('Meta+z'); await settle(4000);
    report('⌘Z brings it back', /id: "N-1"/.test(await source()));
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
