// The embedded chain's syntax in the real VS Code: the grammar colours the
// action language inside a guard and an equation, completion offers the
// machine's variables and the operators inside a guard, the blocks inside
// an equation, the keys of `execute:` and `io_binding` at the top level.
// Runs on Linux too (tools/embedded: xvfb, a linux-arm64 VS Code):
//   VSCODE_APP=/opt/vscode-cache/vscode-linux-arm64-1.144.1/code PLAYWRIGHT=/opt/pw/node_modules/playwright-core \
//   ARCLANG_BINARY=/target/debug/arclang SHOTS=/shots xvfb-run -a node vscode_embedded_test.js
const fs = require('fs'); const os = require('os'); const path = require('path');
const { launchVSCode } = require('./launch');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const MODEL = path.resolve(EXT, '../../examples/energy/battery-charger/battery_charger.arc');
const SHOTS = process.env.SHOTS || null;
const MOD = process.platform === 'darwin' ? 'Meta' : 'Control';
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcemb-'));
  const model = path.join(dir, 'battery_charger.arc');
  const text = fs.readFileSync(MODEL, 'utf8');
  fs.writeFileSync(model, text);
  const lineOf = (needle) => text.split('\n').findIndex((l) => l.includes(needle)) + 1;
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'editor.quickSuggestions': { other: true, strings: true }, 'editor.minimap.enabled': false, ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const linux = process.platform === 'linux' ? ['--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage'] : [];
  const { win, close } = await launchVSCode(APP, [...linux, `--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model], { width: 1600, height: 1000, timeoutMs: 120000 });
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const goTo = async (line) => { await win.keyboard.press(`${MOD}+g`); await win.waitForTimeout(400); await win.keyboard.type(String(line)); await win.keyboard.press('Enter'); await win.waitForTimeout(500); };
  const revert = async () => { await win.keyboard.press('Escape'); await win.keyboard.press(`${MOD}+Shift+p`); await win.waitForTimeout(400); await win.keyboard.type('File: Revert File'); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(1200); };
  const suggest = async () => { await win.keyboard.press('Control+Space'); await win.waitForTimeout(2500); return (await win.locator('.suggest-widget .monaco-list-row').allInnerTexts().catch(() => [])).map((t) => t.replace(/\s+/g, ' ').trim()); };
  try {
    await win.waitForTimeout(8000);
    const editor = win.locator('.monaco-editor .view-lines').first();
    await editor.click();
    // 1. The grammar: an equation's tokens are not one string colour.
    await goTo(lineOf('"setpoint = pid('));
    await win.waitForTimeout(800);
    const classes = await win.locator('.view-line', { hasText: 'setpoint = pid(' }).first().evaluate((line) => [...line.querySelectorAll('span span')].map((s) => `${s.className}:${s.textContent}`));
    const pid = classes.find((c) => /:pid$/.test(c)); const setpoint = classes.find((c) => /:setpoint\s*$/.test(c) || /:setpoint$/.test(c));
    report('an equation is coloured token by token (pid apart from the variables)', !!pid && !!setpoint && pid.split(':')[0] !== setpoint.split(':')[0], JSON.stringify(classes.slice(0, 8)));
    // 2. Completion in a guard: the variables, the operators' words.
    await goTo(lineOf('guard: "filtered >= v_absorption'));
    await win.keyboard.press('End'); for (let i = 0; i < 3; i++) await win.keyboard.press('ArrowLeft');
    await win.keyboard.type(' and fi', { delay: 30 });
    const inGuard = await suggest();
    report('in a guard, completion offers the machine\'s variables', inGuard.some((t) => /^filtered/.test(t)), JSON.stringify(inGuard.slice(0, 6)));
    await shot('embedded-01-guard-completion');
    await revert();
    // 3. In an equation: the blocks.
    await goTo(lineOf('"charge = integrator(current, 1)"'));
    await win.keyboard.press('End'); await win.keyboard.press('Enter');
    await win.keyboard.type('"x = de', { delay: 30 });
    const inEquation = await suggest();
    report('in an equation, completion offers the blocks (delay, integrator, pid)', inEquation.some((t) => /^delay/.test(t)), JSON.stringify(inEquation.slice(0, 6)));
    await revert();
    // 4. In execute: its keys.
    await goTo(lineOf('stop_time: 600 ms') > 0 ? text.split('\n').findIndex((l, i) => i > text.split('\n').findIndex((x) => x.includes('"TC-CHG-003"')) && l.includes('stop_time')) + 1 : 1);
    await win.keyboard.press('End'); await win.keyboard.press('Enter');
    await win.keyboard.type('tol', { delay: 30 });
    const inExecute = await suggest();
    report('inside execute: { }, completion offers its keys', inExecute.some((t) => /^tolerance/.test(t)), JSON.stringify(inExecute.slice(0, 6)));
    await shot('embedded-02-execute-completion');
    await revert();
    // 5. At the top level: io_binding.
    await win.keyboard.press(`${MOD}+End`); await win.keyboard.press('Enter'); await win.keyboard.press('Enter');
    await win.keyboard.type('io_b', { delay: 30 });
    const top = await suggest();
    report('at the top level, completion offers io_binding', top.some((t) => /^io_binding/.test(t)), JSON.stringify(top.slice(0, 6)));
    await revert();
    // The picture: the variables, the equations and a guard, coloured.
    await goTo(lineOf('variables: ['));
    await win.waitForTimeout(1000);
    await shot('embedded-03-model');
  } catch (error) {
    report('the test ran to its end', false, error.message);
  } finally {
    await close();
    fs.rmSync(dir, { recursive: true, force: true });
    const failed = rows.filter((r) => !r.ok).length;
    console.log(`${rows.length - failed}/${rows.length} passed`);
    process.exit(failed ? 1 : 0);
  }
})();
