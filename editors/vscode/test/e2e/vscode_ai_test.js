// The AI features inside the real VS Code, against the local fake provider
// (test/ai/fake-llm-server.js, OpenAI-compatible): no network, no key, no cost.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/node_modules/playwright node vscode_ai_test.js
// The compiler is ARCLANG_BINARY, else the repository's release build, else the PATH. SHOTS=dir keeps pictures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path'); const { spawnSync } = require('child_process');
const { startFakeServer, arclangScenario } = require('../ai/fake-llm-server.js');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const BUILT = path.resolve(EXT, '../../target/release/arclang');
const ARCLANG = process.env.ARCLANG_BINARY || (fs.existsSync(BUILT) ? BUILT : 'arclang');
const SHOTS = process.env.SHOTS || null;
const FAKE_KEY = 'sk-e2e-fake-key-0123456789abcdef';
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail || ''}`); };

// Keys as a finger on a French (AZERTY) Mac sends them: Playwright's own
// press('Meta+z') arrives as ⌘W there. Each key goes through CDP.
async function azerty(win) {
  const cdp = await win.context().newCDPSession(win);
  const AZ = { a: 'KeyQ', q: 'KeyA', z: 'KeyW', w: 'KeyZ', m: 'Semicolon' };
  const MOD = { Alt: [1, 'Alt', 'AltLeft', 18], Control: [2, 'Control', 'ControlLeft', 17], Meta: [4, 'Meta', 'MetaLeft', 91], Shift: [8, 'Shift', 'ShiftLeft', 16] };
  return async (combo) => {
    const parts = combo.split('+'); const last = parts.pop();
    let mods = 0;
    for (const p of parts) { const m = MOD[p]; mods |= m[0]; await cdp.send('Input.dispatchKeyEvent', { type: 'rawKeyDown', modifiers: mods, key: m[1], code: m[2], windowsVirtualKeyCode: m[3] }); }
    const named = { Escape: 27, Enter: 13, Delete: 46, Backspace: 8, ArrowDown: 40, ArrowUp: 38 };
    const spec = /^[a-z]$/i.test(last) ? { key: (mods & 8) ? last.toUpperCase() : last.toLowerCase(), code: AZ[last.toLowerCase()] || `Key${last.toUpperCase()}`, keyCode: last.toUpperCase().charCodeAt(0) } : { key: last, code: last, keyCode: named[last] };
    const text = !(mods & 6) && spec.key.length === 1 ? spec.key : undefined;
    await cdp.send('Input.dispatchKeyEvent', { type: text ? 'keyDown' : 'rawKeyDown', modifiers: mods, key: spec.key, code: spec.code, windowsVirtualKeyCode: spec.keyCode, text, unmodifiedText: text });
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', modifiers: mods, key: spec.key, code: spec.code, windowsVirtualKeyCode: spec.keyCode });
    for (const p of parts.reverse()) { const m = MOD[p]; mods &= ~m[0]; await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', modifiers: mods, key: m[1], code: m[2], windowsVirtualKeyCode: m[3] }); }
  };
}

const compiles = (file) => { const r = spawnSync(ARCLANG, ['check', '--json', file], { encoding: 'utf8' }); try { return JSON.parse(r.stdout).errors.length === 0; } catch { return false; } };
const until = async (win, test, ms = 20000) => { const end = Date.now() + ms; while (Date.now() < end) { try { if (await test()) return true; } catch { /* the page is redrawing */ } await win.waitForTimeout(250); } return false; };

(async () => {
  const fake = await startFakeServer(arclangScenario);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcai-'));
  const model = path.join(dir, 'model.arc');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  const settings = {
    'files.simpleDialog.enable': true, 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none',
    'arclang.binary': ARCLANG,
    'arclang.ai.provider': 'openai-compatible', 'arclang.ai.endpoint': `${fake.url}/v1`, 'arclang.ai.model': 'fake-arclang',
    'arclang.ai.suggestOnEdit': 'afterEachEdit', 'arclang.ai.contextScope': 'selectionAndNeighbours',
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const exe = ['Electron', 'Code'].map((n) => path.join(APP, 'Contents/MacOS', n)).find((p) => fs.existsSync(p));
  const app = await _electron.launch({ executablePath: exe, args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--disable-workspace-trust', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const press = await azerty(win);
  const shot = async (name) => { if (SHOTS) { fs.mkdirSync(SHOTS, { recursive: true }); await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); } };
  const command = async (name) => { await press('Meta+Shift+p'); await win.waitForTimeout(400); await win.keyboard.type(name); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(700); };
  const source = () => fs.readFileSync(model, 'utf8');
  try {
    // 0. A key for the provider, through the command: it reaches the server, never the log.
    await command('ArcLang: Set AI API key');
    await win.keyboard.type('OpenAI-compatible'); await win.waitForTimeout(400); await win.keyboard.press('Enter'); await win.waitForTimeout(500);
    await win.keyboard.type(FAKE_KEY); await win.keyboard.press('Enter'); await win.waitForTimeout(800);
    await command('View: Close Secondary Side Bar');
    await command('ArcLang: Open the workbench');
    let page = null;
    for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* frame going away */ } } }
    report('the workbench opens', !!page);
    if (!page) throw new Error('no workbench page');
    const status = await win.evaluate(() => [...document.querySelectorAll('.statusbar-item')].map((n) => n.textContent.trim()).find((t) => /OpenAI-compatible/.test(t)) || '');
    report('the status bar shows the provider and the model', /OpenAI-compatible server · fake-arclang/.test(status), status);
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await win.waitForTimeout(1800);
    const box = async (id) => page.locator(`.arcviz-stage [data-id="${id}"]`).first().boundingBox();

    // 1. Two boxes selected, ⌘I, a request, the proposal checked by the compiler, Apply, ⌘Z
    const cam = await box('LC-CAM'); const ctl = await box('LC-CTL');
    await win.mouse.click(cam.x + cam.width / 2, cam.y + 10); await win.waitForTimeout(500);
    await win.keyboard.down('Shift'); await win.mouse.click(ctl.x + ctl.width / 2, ctl.y + 10); await win.keyboard.up('Shift'); await win.waitForTimeout(700);
    const mini = await page.locator('.wb-mini:not([hidden]) .wb-mini-button', { hasText: '✦' }).count();
    report('the floating toolbar of the selection has Ask AI', mini > 0);
    await press('Meta+i'); await win.waitForTimeout(800);
    let opened = await page.locator('.wb-ai').isVisible().catch(() => false);
    report('⌘I on two selected boxes opens the Ask AI prompt by them', opened);
    if (!opened) { await page.locator('.wb-mini-button', { hasText: '✦' }).first().click(); await win.waitForTimeout(600); opened = await page.locator('.wb-ai').isVisible(); }
    const about = await page.locator('.wb-ai-about').textContent().catch(() => '');
    report('the prompt names the selection', /Camera/.test(about) && /LKA Controller/.test(about), about);
    await page.locator('.wb-ai-input').fill('add a monitoring component connected to both');
    await shot('ai-1-prompt');
    await page.locator('.wb-ai-input').press('Enter');
    const proposed = await until(win, () => page.locator('.wb-ai-apply').isVisible(), 30000);
    report('the proposal comes back, checked by the compiler', proposed);
    const diff = (await page.locator('.wb-ai-diff').textContent().catch(() => '')) || '';
    report('the preview shows the diff of the source', /component "Health Monitor"/.test(diff) && /CameraHealth/.test(diff), `${diff.length} characters`);
    const marked = await page.evaluate(() => [...document.querySelectorAll('.wb-ai-mark')].map((n) => n.dataset.id));
    report('the changed boxes are marked on the diagram', marked.includes('LC-CAM') && marked.includes('LC-CTL'), marked.join(','));
    const created = await page.locator('.wb-ai-created').textContent().catch(() => '');
    report('the new elements are listed', /LC-MON/.test(created), created);
    report('nothing is applied before Apply', !/LC-MON/.test(source()) && !(await win.evaluate(() => document.title)).startsWith('●'));
    await shot('ai-2-proposal');
    await page.locator('.wb-ai button', { hasText: 'Open diff' }).click(); await win.waitForTimeout(1500);
    const diffEditor = await win.locator('.monaco-diff-editor').first().isVisible().catch(() => false);
    report('Open diff shows VS Code\'s diff view', diffEditor);
    await shot('ai-3-diff');
    await page.locator('.wb-ai-apply').click();
    const applied = await until(win, async () => /Applied: the AI proposal/.test((await page.locator('.wb-toast').textContent()) || ''), 15000);
    await page.locator('.arcviz-stage').click({ position: { x: 6, y: 6 } }); await press('Meta+s'); await win.waitForTimeout(1200);
    const afterApply = source();
    report('Apply writes the change into the model', applied && /id: "LC-MON"/.test(afterApply) && /component_exchange "CameraHealth"/.test(afterApply));
    report('the applied model compiles', compiles(model));
    await shot('ai-4-applied');
    await page.locator('.arcviz-stage').click({ position: { x: 6, y: 6 } }); await press('Meta+z'); await win.waitForTimeout(1500); await press('Meta+s'); await win.waitForTimeout(1200);
    report('⌘Z undoes the whole AI change in one step', !/LC-MON/.test(source()) && !/CameraHealth/.test(source()) && /id: "LC-CAM"/.test(source()));
    const sent = fake.requests.filter((r) => /monitoring component/.test(r.lastUser));
    report('the provider got the selection\'s source, the rules and the tool', sent.length > 0 && /component \\"Camera\\"/.test(sent[0].all) && /DATA about the system/.test(sent[0].all) && /propose_edits/.test(sent[0].all));
    report('the key went in the Authorization header', sent.length > 0 && sent[0].headers.authorization === `Bearer ${FAKE_KEY}`);

    // 2. A text selection in the .arc editor: the Camera block, "raise to ASIL-D"
    await press('Meta+p'); await win.waitForTimeout(400); await win.keyboard.type('model.arc'); await win.waitForTimeout(600); await win.keyboard.press('Enter'); await win.waitForTimeout(1500);
    const focusText = async () => { await command('View: Focus First Editor Group'); await win.waitForTimeout(400); };
    await focusText();
    const camLine = source().split('\n').findIndex((l) => l.includes('component "Camera"')) + 1;
    await press('Control+g'); await win.waitForTimeout(300); await win.keyboard.type(String(camLine)); await win.keyboard.press('Enter'); await win.waitForTimeout(400);
    for (let i = 0; i < 4; i++) await win.keyboard.press('Shift+ArrowDown');
    await win.waitForTimeout(300);
    await shot('ai-5a-selected');
    await press('Meta+i'); await win.waitForTimeout(1500);
    await shot('ai-5b-cmd-i');
    console.log('quick input:', await win.locator('.quick-input-widget').first().textContent().catch(() => '(none)'));
    let picker = /Ask AI about/.test((await win.locator('.quick-input-widget:visible .quick-input-title').textContent().catch(() => '')) || '');
    report('⌘I on a text selection of the model opens Ask AI', picker);
    if (!picker) { await command('ArcLang: Ask AI about the selection'); picker = true; }
    await win.keyboard.type('raise the safety level to ASIL-D'); await win.waitForTimeout(400); await win.keyboard.press('Enter');
    const toast = win.locator('.notification-toast', { hasText: 'ArcLang AI' });
    const asked = await until(win, () => toast.first().isVisible(), 30000);
    await win.waitForTimeout(800);
    await shot('ai-5-text-ask');
    const textDiff = await win.locator('.monaco-diff-editor').first().isVisible().catch(() => false);
    report('the text ask shows the diff and Apply / Refine / Reject', asked && textDiff && (await toast.locator('.monaco-button', { hasText: 'Apply' }).count()) > 0);
    await toast.locator('.monaco-button', { hasText: 'Apply' }).first().click(); await win.waitForTimeout(2000);
    await press('Meta+s'); await win.waitForTimeout(1000);
    report('Apply raises the camera to ASIL-D in the source', /id: "LC-CAM"\n    safety_level: "ASIL-D"/.test(source()) && compiles(model));
    const textRequest = fake.requests.filter((r) => /ASIL-D/.test(r.lastUser)).pop();
    report('the selected text and its element went to the provider', !!textRequest && /The text the user selected/.test(textRequest.all) && /### LogicalComponent LC-CAM/.test(textRequest.all));

    // 3. An edit that breaks a trace: the Suggestions pane proposes a fix, compiled, applied
    await focusText();
    console.log('lines on disk:', source().split('\n').length);
    await press('Meta+f'); await win.waitForTimeout(400); await win.keyboard.type('trace "LC-CAM" satisfies "REQ-LKA-001"'); await win.keyboard.press('Enter'); await win.waitForTimeout(300); await press('Escape'); await win.waitForTimeout(300);
    // The match stays selected; the editor takes the keys again (the find box may keep them otherwise).
    await command('View: Focus Active Editor Group'); await win.waitForTimeout(300);
    await shot('ai-6a-at-trace');
    await press('Meta+Shift+k'); await win.waitForTimeout(500);
    await shot('ai-6b-deleted');
    await press('Meta+s'); await win.waitForTimeout(800);
    report('the trace line is deleted in the text', !/trace "LC-CAM" satisfies "REQ-LKA-001"/.test(source()));
    // Back to the workbench tab.
    await win.locator('.tab', { hasText: 'model.arc' }).filter({ has: win.locator('.codicon, .file-icon') }).first().click().catch(() => {});
    for (const t of await win.locator('.tabs-container .tab').all()) { if (/workbench|model\.arc/i.test((await t.getAttribute('aria-label')) || '') && (await t.getAttribute('aria-label')).includes('ArcLang')) await t.click(); }
    await command('ArcLang: Open the workbench'); await win.waitForTimeout(1500);
    await page.locator('.wb-ai-tab').click().catch(() => {});
    const cut = await page.evaluate(() => { const r = document.querySelector('.wb-right'); return r ? r.scrollWidth - r.clientWidth : 0; });
    report('the drawer with the AI tab fits its width', cut <= 1, `${cut}px over`);
    const card = page.locator('.wb-ai-card', { hasText: 'Trace the camera' });
    const suggested = await until(win, () => card.first().isVisible(), 30000);
    const consequences = (await page.locator('.wb-ai-consequences').textContent().catch(() => '')) || '';
    report('after the edit, the pane lists what it broke', /LC-CAM satisfies REQ-LKA-001 is gone/.test(consequences) || /LC-CAM has no trace/.test(consequences), consequences.slice(0, 120));
    report('the AI proposes a change, marked as an AI proposal', suggested && /Proposals of an AI/.test((await page.locator('.wb-ai-hint').textContent()) || ''));
    await shot('ai-6-suggestion');
    await card.locator('.wb-ai-apply').first().click(); await win.waitForTimeout(2500);
    await page.locator('.arcviz-stage').click({ position: { x: 6, y: 6 } }); await press('Meta+s'); await win.waitForTimeout(1200);
    report('applying the suggestion puts the trace back', /trace "LC-CAM" satisfies "REQ-LKA-001"/.test(source()) && compiles(model));
    const suggestRequest = fake.requests.find((r) => /propose_changes/.test(r.all));
    report('the suggestion request carried the consequences and impact', !!suggestRequest && /Its consequences found by the compiler/.test(suggestRequest.all) && /arclang impact/.test(suggestRequest.all));

    // 4. The log names provider, model and endpoint, never the key
    await command('ArcLang: Show the AI log'); await win.waitForTimeout(1500);
    const log = ((await win.locator('.output .view-lines, .panel .view-lines').first().textContent().catch(() => '')) || '').replace(/\s+/g, ' ');
    await shot('ai-7-log');
    report('the AI log lists the requests without the key', /openai-compatible fake-arclang at http:\/\/127\.0\.0\.1/.test(log) && !log.includes(FAKE_KEY), `${log.length} characters`);
    report('no request reached any other server', fake.requests.every((r) => r.format === 'openai'));
  } catch (error) {
    report('no error', false, String(error && error.stack || error));
    await shot('ai-error');
  } finally {
    await app.close().catch(() => {});
    await fake.close();
    fs.rmSync(dir, { recursive: true, force: true });
  }
  const failed = rows.filter((r) => !r.ok); console.log(`\n${rows.length - failed.length}/${rows.length} AI checks in VS Code`);
  process.exit(failed.length ? 1 : 0);
})();
