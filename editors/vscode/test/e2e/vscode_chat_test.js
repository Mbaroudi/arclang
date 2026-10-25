// The AI chat view inside the real VS Code (desktop), against the fake
// provider: it opens in the secondary side bar, sees the model and the
// selection, shows a compiler-checked proposal, Apply writes it as one
// undoable edit; and on a .mo file the answer is labelled not verified.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=... node vscode_chat_test.js
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path'); const { spawnSync } = require('child_process');
const { startFakeServer, arclangScenario } = require('../ai/fake-llm-server.js');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const BUILT = path.resolve(EXT, '../../target/release/arclang');
const ARCLANG = process.env.ARCLANG_BINARY || (fs.existsSync(BUILT) ? BUILT : 'arclang');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail || ''}`); };
const compiles = (file) => { const r = spawnSync(ARCLANG, ['check', '--json', file], { encoding: 'utf8' }); try { return JSON.parse(r.stdout).errors.length === 0; } catch { return false; } };
const until = async (win, test, ms = 20000) => { const end = Date.now() + ms; while (Date.now() < end) { try { if (await test()) return true; } catch { /* redrawing */ } await win.waitForTimeout(250); } return false; };

(async () => {
  const fake = await startFakeServer((request) => (/Modelica assistant/.test(request.all) ? { text: 'It integrates the current.\n```modelica\nmodel X\nend X;\n```' } : arclangScenario(request)));
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcchat-'));
  const model = path.join(dir, 'model.arc'); const mo = path.join(dir, 'battery.mo');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  fs.writeFileSync(mo, 'model Battery\n  Real soc(start = 1);\nequation\n  der(soc) = -0.001;\nend Battery;\n');
  const settings = {
    'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none',
    'arclang.binary': ARCLANG, 'arclang.ai.provider': 'openai-compatible', 'arclang.ai.endpoint': `${fake.url}/v1`, 'arclang.ai.model': 'fake-arclang', 'arclang.ai.contextScope': 'selectionAndNeighbours',
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const exe = ['Electron', 'Code'].map((n) => path.join(APP, 'Contents/MacOS', n)).find((p) => fs.existsSync(p));
  const app = await _electron.launch({ executablePath: exe, args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--disable-workspace-trust', model, mo] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const press = await azerty(win);
  const shot = async (name) => { if (SHOTS) { fs.mkdirSync(SHOTS, { recursive: true }); await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); } };
  const command = async (name) => { await press('Meta+Shift+p'); await win.waitForTimeout(400); await win.keyboard.type(name); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(700); };
  const chatFrame = async () => { for (const f of win.frames()) { try { if (await f.$('#chat .chat-input')) return f; } catch { /* a frame going away */ } } return null; };
  try {
    await until(win, () => win.evaluate(() => [...document.querySelectorAll('.statusbar-item')].some((n) => /OpenAI-compatible/.test(n.textContent))), 30000);
    await command('ArcLang: Open the AI chat');
    let chat = null; await until(win, async () => (chat = await chatFrame()), 20000);
    report('the chat view opens (secondary side bar)', !!chat);
    if (!chat) throw new Error('no chat view');
    // The model is the active editor: select the camera by the keyboard in the text.
    await command('ArcLang: Open the workbench').catch?.(() => {});
    await win.keyboard.press('Escape');
    await until(win, async () => /model\.arc/.test(await chat.locator('.chat-context').innerText()), 15000);
    report('the chat sees the model file', /model\.arc/.test(await chat.locator('.chat-context').innerText()), (await chat.locator('.chat-context').innerText()).replace(/\s+/g, ' '));
    await chat.locator('.chat-input').fill('Add a monitoring component that watches the camera and the controller.');
    await chat.locator('.chat-send').click();
    const apply = chat.locator('.chat-proposal .primary', { hasText: 'Apply' });
    await apply.waitFor({ timeout: 60000 });
    report('the answer is a compiler-checked proposal with a diff', /Checked by the compiler/.test(await chat.locator('.chat-checked').innerText()) && (await chat.locator('.chat-diff').count()) === 1);
    await shot('chat-1-proposal');
    await apply.click(); await win.waitForTimeout(2500);
    await win.keyboard.press('Meta+s'); await win.waitForTimeout(1500);
    const text = fs.readFileSync(model, 'utf8');
    report('Apply wrote the component and the model still compiles', /Health Monitor/.test(text) && compiles(model));
    await shot('chat-2-applied');
    // Undo, as the editor does it, takes the whole proposal back.
    await chat.locator('.chat-proposal button', { hasText: 'Undo' }).click(); await win.waitForTimeout(1500);
    await win.keyboard.press('Meta+s'); await win.waitForTimeout(1200);
    report('Undo takes the whole proposal back in one step', !/Health Monitor/.test(fs.readFileSync(model, 'utf8')));
    // A Modelica file.
    await win.locator('.tab', { hasText: 'battery.mo' }).first().click(); await win.waitForTimeout(1500);
    await until(win, async () => /battery\.mo/.test(await chat.locator('.chat-context').innerText()), 10000);
    await chat.locator('.chat-input').fill('Explain this model.');
    await shot('chat-3a-before-modelica');
    await chat.locator('.chat-send').click();
    await chat.locator('.chat-unverified').waitFor({ timeout: 40000 });
    report('the Modelica answer is labelled not verified', /Not verified: there is no Modelica compiler in the browser/.test(await chat.locator('.chat-unverified').innerText()));
    await shot('chat-3-modelica');
  } catch (error) {
    report('no error', false, String(error).slice(0, 300));
  } finally {
    await app.close(); await fake.close();
  }
  const failed = rows.filter((r) => !r.ok).length;
  console.log(failed ? `${failed} FAILED` : 'all ok');
  process.exit(failed ? 1 : 0);
})();
