// A state invariant from the scenario palette, inside the real VS Code: a
// message selected, the State tool asks what the lifeline is in, the messages
// are wrapped in `mode "Lifeline" "Name" { ... }` and the box is drawn on the
// lifeline; an unknown state is refused with the compiler's reason; ⌘Z.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_invariant_test.js
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
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcinv-'));
  const model = path.join(dir, 'model.arc');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const { close, win, page, press } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const source = async () => { await press('Meta+s'); await win.waitForTimeout(900); return fs.readFileSync(model, 'utf8'); };
  const settle = (ms = 3000) => win.waitForTimeout(ms);
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    await page.locator('button, [role="tab"]', { hasText: /^Scenarios/ }).first().click(); await settle(2500);
    const inside = await page.locator('.arcviz-stage svg').first().evaluate((n) => { const r = n.getBoundingClientRect(); return { x: r.x, y: r.y }; });
    const outside = await page.locator('.arcviz-stage svg').first().boundingBox();
    const offset = { x: outside.x - inside.x, y: outside.y - inside.y };
    const message = page.locator('.arcviz-stage .av-edge', { hasText: 'steering correction' }).first();
    const middle = () => message.evaluate((g) => { const line = g.querySelector('.av-line'); const len = line.getTotalLength(); const p = line.getPointAtLength(len / 2); const m = line.getScreenCTM(); return { x: p.x * m.a + p.y * m.c + m.e, y: p.x * m.b + p.y * m.d + m.f }; });
    const ask = async (answer) => {
      const m = await middle(); await win.mouse.click(m.x + offset.x, m.y + offset.y); await win.waitForTimeout(800);
      await page.locator('[data-tool="invariant"]').first().click(); await win.waitForTimeout(500);
      const input = page.locator('.wb-prompt input').first();
      const label = await page.locator('.wb-prompt label').first().innerText().catch(() => '');
      if (await input.isVisible().catch(() => false)) { await input.fill(answer); await input.press('Enter'); await settle(4000); }
      return label;
    };

    // 1. Refused: a mode the model does not declare, with the reason
    const before = await source();
    const label = await ask('LC-CTL: mode Dreaming');
    report('the State tool asks what the lifeline is in during the message', /State of LC-ACT during "steering correction"/.test(label), label);
    const toast = await page.locator('.wb-toast').first().innerText().catch(() => '');
    report('an unknown mode is refused with the compiler\'s reason', /Dreaming/.test(toast) && (await source()) === before, toast.slice(0, 120));
    await press('Escape'); await settle(800);

    // 2. Written and drawn: the controller is correcting during the message
    await ask('LC-CTL: mode Correcting');
    const after = await source();
    report('the message is wrapped in the mode on the lifeline', /  mode "LC-CTL" "Correcting" \{\n    message "LC-CTL" -> "LC-ACT" "steering correction"\n  \}/.test(after), (after.match(/mode "LC-CTL"[^\n]*/) || [''])[0]);
    const boxes = await page.locator('.arcviz-stage .av-invariant').evaluateAll((gs) => gs.map((g) => g.textContent.trim()));
    report('a box is drawn on the lifeline for it', boxes.some((t) => /Correcting/.test(t)), JSON.stringify(boxes));
    await shot('invariant-01-drawn');

    // 3. Undone
    await press('Meta+z'); await settle(4000);
    report('⌘Z takes it back', (await source()) === before);
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
