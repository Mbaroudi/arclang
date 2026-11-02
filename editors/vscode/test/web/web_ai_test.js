// The AI chat in the web playground, against the fake LLM (an OpenAI-
// compatible server, no network, no key): a question on an .arc selection
// gives a compiler-checked proposal, Apply is one edit and Ctrl/Cmd+Z takes it
// back; a question on a .mo file is labeled "not verified"; the key goes to
// the secret storage and to no host but the provider's.
//   BASE=http://127.0.0.1:8765/ FAKE_PORT=8799 PLAYWRIGHT=... node web_ai_test.js
// The page must be served with a content security policy whose connect-src
// lets the page reach http://127.0.0.1:$FAKE_PORT (what a visitor who chose a
// local provider gets from the server's cookie map).
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const { startFakeServer } = require('../ai/fake-llm-server');
const { azerty } = require('../e2e/azerty'); // keys as a finger on this Mac's French keyboard sends them
const BASE = process.env.BASE || 'http://127.0.0.1:8765/';
const FAKE_PORT = Number(process.env.FAKE_PORT || 8799);
const SHOTS = process.env.SHOTS;
const KEY = 'sk-test-FAKE-KEY-for-the-browser-test-0001';
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail || ''}`); };

const script = (request) => {
  const all = request.all;
  if (/Modelica assistant of the ArcLang playground/.test(all)) return { text: 'The pack integrates the current into the state of charge, and heats up through R0.\n\n```modelica\nmodel Battery2\n  Real x(start = 1);\nequation\n  der(x) = -0.01;\nend Battery2;\n```' };
  if (/propose_edits/.test(all)) return { text: 'I give the detector a description.', toolCalls: [{ name: 'propose_edits', input: { explanation: 'The Threat Detector gets a description saying what it does.', requests: [{ op: 'set', element: 'LC-001', key: 'description', value: '"Detects threats on the road"' }] } }] };
  return { text: 'ok' };
};

async function frameWith(page, selector, timeout = 30000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    for (const f of page.frames()) { try { if (await f.$(selector)) return f; } catch { /* a frame going away */ } }
    await page.waitForTimeout(250);
  }
  return null;
}

(async () => {
  const fake = await startFakeServer(script, FAKE_PORT);
  const browser = await chromium.launch({ args: ['--host-resolver-rules=MAP *.vscode-cdn.net ~NOTFOUND'] });
  const context = await browser.newContext({ viewport: { width: 1600, height: 950 } });
  const page = await context.newPage();
  const errors = []; page.on('pageerror', (e) => errors.push(String(e)));
  const calls = []; // every request of the page: [origin, hasKey]
  page.on('request', (r) => { const u = new URL(r.url()); if (/^https?:$/.test(u.protocol)) calls.push({ origin: u.origin, key: JSON.stringify(r.headers()).includes(KEY) || (r.postData() || '').includes(KEY) || r.url().includes(KEY) }); });
  const origin = new URL(BASE).origin;
  const fakeOrigin = `http://127.0.0.1:${FAKE_PORT}`;
  const api = (fn, arg) => page.evaluate(fn, arg);

  await page.goto(BASE);
  await page.waitForFunction(() => window.arclangPlayground && window.arclangPlayground.ready, null, { timeout: 120000 });
  const chat = await frameWith(page, '#chat .chat-input');
  report('the chat view is in the secondary side bar', !!chat);
  if (!chat) { await browser.close(); await fake.close(); process.exit(1); }
  const hosted = await chat.locator('.chat-provider').innerText();
  report('the page defaults to the ArcLang hosted provider', /hosted/i.test(hosted), hosted);

  // The visitor brings a key and an OpenAI-compatible server (here the fake LLM).
  await api(async (fakeUrl) => {
    const cfg = window.arclangPlayground.vscode.workspace.getConfiguration('arclang.ai');
    const G = window.arclangPlayground.vscode.ConfigurationTarget.Global;
    await cfg.update('provider', 'openai-compatible', G);
    await cfg.update('endpoint', `${fakeUrl}/v1`, G);
    await cfg.update('model', 'fake-model', G);
    await cfg.update('confirmBeforeSending', false, G);
  }, fakeOrigin);
  // The command returns when the input is answered: not awaited here.
  await page.evaluate(() => { void window.arclangPlayground.vscode.commands.executeCommand('arclang.ai.setApiKey', 'openai-compatible'); });
  const box = page.locator('.quick-input-widget input[type=password], .quick-input-widget input').first();
  await box.waitFor({ timeout: 10000 });
  await box.fill(KEY); await box.press('Enter');
  await page.waitForTimeout(800);
  report('the key was typed in the secret input of the workbench', true);

  // 1. Ask on an .arc selection.
  await api(() => {
    const m = window.arclangPlayground.monaco;
    const editor = m.editor.getEditors().find((e) => e.getModel() && e.getModel().uri.path === '/workspace/starter.arc');
    editor.focus(); editor.setSelection(new m.Selection(22, 1, 27, 4));
  });
  await page.waitForTimeout(800);
  const ctx = await chat.locator('.chat-context').innerText();
  report('the chat sees the active file and the selection', /starter\.arc/.test(ctx) && /selected/.test(ctx), ctx.replace(/\s+/g, ' '));
  const before = await api(() => window.arclangPlayground.modelText('/workspace/starter.arc'));
  await chat.locator('.chat-input').fill('Describe the selected component.');
  await chat.locator('.chat-send').click();
  await chat.locator('.chat-proposal .primary', { hasText: 'Apply' }).waitFor({ timeout: 40000 });
  const checked = await chat.locator('.chat-checked').innerText();
  report('the answer is a proposal checked by the compiler on a copy', /Checked by the compiler/.test(checked), checked);
  const diff = await chat.locator('.chat-diff').innerText();
  report('the diff of the model source is shown', /Detects threats/.test(diff) && /^\+/m.test(diff));
  report('nothing is applied before Apply', (await api(() => window.arclangPlayground.modelText('/workspace/starter.arc'))) === before);
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/web-ai-1-proposal.png` });
  await chat.locator('.chat-proposal .primary', { hasText: 'Apply' }).click();
  await page.waitForTimeout(1500);
  const applied = await api(() => window.arclangPlayground.modelText('/workspace/starter.arc'));
  report('Apply changes the model', /description: "Detects threats on the road"/.test(applied));
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/web-ai-2-applied.png` });
  // Undo: the editor's own, as for any edit.
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/web-ai-2b-before-undo.png` });
  await page.locator('.editor-instance .monaco-editor .view-lines').first().click({ position: { x: 300, y: 30 } });
  const press = await azerty(page);
  await press('Meta+z'); await page.waitForTimeout(800);
  let undone = await api(() => window.arclangPlayground.modelText('/workspace/starter.arc'));
  report('the keyboard undo takes the applied proposal back in one step', undone === before, undone === before ? '' : `before ${before.length} chars, after undo ${undone.length}, has description: ${/Detects threats/.test(undone)}; focus: ${await api(() => document.activeElement && document.activeElement.className)}`);

  // 2. A Modelica file: explained, labeled not verified.
  await api(() => { void window.arclangPlayground.vscode.commands.executeCommand('vscode.open', window.arclangPlayground.vscode.Uri.file('/workspace/battery.mo')); });
  await page.waitForTimeout(1500);
  const lang = await api(() => window.arclangPlayground.monaco.editor.getModels().find((m) => m.uri.path === '/workspace/battery.mo').getLanguageId());
  report('a .mo file is the Modelica language', lang === 'modelica', lang);
  const ctxMo = await chat.locator('.chat-context').innerText();
  report('the chat context says answers are not verified', /Modelica/.test(ctxMo) && /not verified/.test(ctxMo), ctxMo.replace(/\s+/g, ' '));
  await chat.locator('.chat-input').fill('Explain this model.');
  await chat.locator('.chat-send').click();
  await chat.locator('.chat-unverified').waitFor({ timeout: 40000 });
  const label = await chat.locator('.chat-unverified').innerText();
  report('the Modelica answer is labeled not verified', /Not verified: there is no Modelica compiler in the browser/.test(label), label.slice(0, 120));
  report('the label says where verification would run', /full workspace tier/.test(label));
  report('the answer offers its code to the editor', (await chat.locator('.chat-code button', { hasText: 'Insert at cursor' }).count()) === 1);
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/web-ai-3-modelica.png` });
  const sentMo = fake.requests.filter((r) => /Modelica assistant/.test(r.all)).pop();
  report('the file text and the Modelica rules were what the model received', !!sentMo && /der\(soc\) = -i \/ Q/.test(sentMo.all));

  // 3. The key.
  const withKey = fake.requests.filter((r) => String(r.headers.authorization || '').includes(KEY));
  report('the fake provider got the key as a bearer token (and only there)', withKey.length >= 1);
  const leaked = calls.filter((c) => c.key && c.origin !== fakeOrigin);
  report('the key appears in no request to another host', leaked.length === 0, JSON.stringify(leaked.slice(0, 3)));
  const stored = await page.evaluate(async (key) => {
    const found = [];
    for (const info of await indexedDB.databases()) {
      const db = await new Promise((resolve, reject) => { const r = indexedDB.open(info.name); r.onsuccess = () => resolve(r.result); r.onerror = () => reject(r.error); });
      for (const name of db.objectStoreNames) {
        const all = await new Promise((resolve) => { const q = db.transaction(name).objectStore(name).getAll(); q.onsuccess = () => resolve(q.result); q.onerror = () => resolve([]); });
        const text = JSON.stringify(all, (k, v) => (v instanceof Uint8Array ? Array.from(v) : v));
        if (text.includes(key)) found.push(`${info.name}/${name}`);
      }
      db.close();
    }
    for (const k of Object.keys(localStorage)) if ((localStorage.getItem(k) || '').includes(key)) found.push(`localStorage/${k}`);
    return found;
  }, KEY);
  report('the key is in no setting and no workspace file', !stored.some((s) => /settings|userdata|workspace/i.test(s)), stored.join(', ') || 'stored obfuscated or encrypted by the secret service');
  const settingsText = await page.evaluate(() => JSON.stringify(window.arclangPlayground.vscode.workspace.getConfiguration('arclang.ai')));
  report('the settings do not hold the key', !settingsText.includes(KEY));
  const strangers = [...new Set(calls.map((c) => c.origin))].filter((o) => o !== origin && o !== fakeOrigin && !/vscode-cdn\.net$/.test(o));
  report('the page talked to no other host', strangers.length === 0, strangers.join(', '));
  report('no page error', errors.length === 0, errors.slice(0, 2).join(' | ').slice(0, 200));
  await browser.close(); await fake.close();
  const failed = rows.filter((r) => !r.ok).length;
  console.log(failed ? `${failed} FAILED` : 'all ok');
  process.exit(failed ? 1 : 0);
})();
