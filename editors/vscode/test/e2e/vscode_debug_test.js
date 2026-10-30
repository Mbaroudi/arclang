// The state machine debugger inside the real VS Code: a breakpoint (F9) on a
// mode of the model, the debugger launched from the command palette, events
// typed in the debug console, a guard set there, the stop on the breakpoint,
// the active modes as the call stack, and the mode lit on the workbench diagram.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_debug_test.js
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
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
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
  await win.waitForTimeout(1500);
  return { app, win, page, press: await azerty(win) };
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcdebug-'));
  const model = path.join(dir, 'model.arc');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const { app, win, page, press } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const settle = (ms = 2500) => win.waitForTimeout(ms);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(400); await win.keyboard.type(name); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(900); };
  const consoleType = async (text) => {
    await command('Debug: Focus on Debug Console View');
    await win.keyboard.type(text); await win.keyboard.press('Enter'); await settle(2500);
  };
  const consoleText = () => win.locator('.repl').first().innerText().catch(() => '');
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    await page.locator('button, [role="tab"]', { hasText: /^Modes/ }).first().click(); await settle();

    // 1. A breakpoint on `mode Correcting` (line 171), set with F9 in the text
    await command('View: Focus First Editor Group');
    await win.keyboard.press('Control+g'); await win.waitForTimeout(400); await win.keyboard.type('171'); await win.keyboard.press('Enter'); await win.waitForTimeout(500);
    await win.keyboard.press('F9'); await settle(800);
    const glyph = await win.locator('.codicon-debug-breakpoint, .codicon-debug-breakpoint-unverified').count();
    report('F9 puts a breakpoint in the margin of the model', glyph > 0, `${glyph}`);

    // 2. Run: the initial mode is entered and the session stops on entry
    await command('ArcLang: Run the state machine (debugger)'); await settle(5000);
    const started = await consoleText();
    report('the debugger runs the machine of the model', /Running LKA modes/.test(started) && /active: Off/.test(started), started.split('\n').slice(0, 3).join(' | '));
    const verified = await win.locator('.codicon-debug-breakpoint').count();
    report('the breakpoint on the mode is verified', verified > 0);

    // 3. Events typed in the console; the guard is a condition set there
    await consoleType('SF-DET');
    report('an event typed in the console fires its transition', /SF-DET: fired Off -> Armed/.test(await consoleText()));
    await consoleType('SF-EST');
    report('a guard not set: nothing fires, and it says which', /nothing fired.*'departure imminent'/.test(await consoleText()));
    await consoleType('set departure imminent true');
    await consoleType('SF-EST'); await settle(1500);
    const log = await consoleText();
    report('set, the transition fires', /SF-EST: fired Armed -> Correcting/.test(log));
    await shot('debug-01-breakpoint');

    // 4. Stopped on the breakpoint: the call stack, the line, the diagram
    const stack = await win.locator('.debug-viewlet, .debug-pane').first().innerText().catch(() => '');
    report('stopped on the breakpoint, the active mode is the call stack, with the reason', /mode Correcting/.test(stack) && /entered Correcting/.test(stack), stack.replace(/\s+/g, ' ').slice(0, 120));
    const paused = await win.locator('.codicon-debug-stackframe').count();
    report('the editor shows the line of the mode it stopped on', paused > 0);
    const lit = await page.locator('.arcviz-stage .wb-debug-active').evaluateAll((ns) => ns.map((n) => n.dataset.id));
    report('the active mode is lit on the workbench diagram', lit.includes('LKA modes.Correcting'), JSON.stringify(lit));
    const fired = await page.locator('.arcviz-stage .wb-debug-fired').count();
    report('and the transition just fired is drawn bold', fired === 1);

    // 5. Stop: the diagram is no longer lit
    await win.keyboard.press('Shift+F5'); await settle(3500);
    await shot('debug-02-stopped');
    report('the session ends', (await win.locator('.debug-toolbar').count()) === 0 || !(await win.locator('.debug-toolbar').first().isVisible().catch(() => false)));
    report('stopped, the diagram is no longer lit', (await page.locator('.arcviz-stage .wb-debug-active').count()) === 0);
  } catch (error) {
    report('the test ran to its end', false, error.message);
  } finally {
    await app.close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
    const failed = rows.filter((r) => !r.ok).length;
    console.log(`${rows.length - failed}/${rows.length} passed`);
    process.exit(failed ? 1 : 0);
  }
})();
