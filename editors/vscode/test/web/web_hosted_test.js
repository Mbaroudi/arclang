// The hosted provider end to end in a browser, on the site's own nginx and AI
// gateway (the Docker image), with a fake Anthropic behind the gateway
// (AI_UPSTREAM): the visitor asks without any key, the consent is asked once,
// the request goes to the site's /api/ai and nowhere else, the answer streams
// back. Run the container first:
//   docker run -d -p 8790:8080 -e ANTHROPIC_API_KEY=sk-not-real -e AI_UPSTREAM=http://host.docker.internal:8798 arclang-site
//   node test/ai/fake-llm-server.js 8798
//   BASE=http://localhost:8790/playground/vscode/ PLAYWRIGHT=... node test/web/web_hosted_test.js
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const BASE = process.env.BASE || 'http://localhost:8790/playground/vscode/';
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail || ''}`); };

(async () => {
  const browser = await chromium.launch({ args: ['--host-resolver-rules=MAP *.vscode-cdn.net ~NOTFOUND'] });
  const page = await (await browser.newContext({ viewport: { width: 1600, height: 950 } })).newPage();
  const origin = new URL(BASE).origin;
  const hosts = new Set(); const ai = [];
  page.on('request', (r) => { const u = new URL(r.url()); if (/^https?:$/.test(u.protocol)) { hosts.add(u.origin); if (u.pathname.startsWith('/api/ai')) ai.push({ path: u.pathname, key: Object.keys(r.headers()).filter((h) => /key|authorization/i.test(h)) }); } });
  await page.goto(BASE);
  await page.waitForFunction(() => window.arclangPlayground && window.arclangPlayground.ready, null, { timeout: 120000 });
  let chat = null;
  for (let i = 0; i < 60 && !chat; i++) { for (const f of page.frames()) { try { if (await f.$('#chat .chat-input')) chat = f; } catch { /* */ } } await page.waitForTimeout(300); }
  report('the page opens with the hosted provider and no key to set', !!chat && /hosted/i.test(await chat.locator('.chat-provider').innerText()) && (await chat.locator('.chat-banner').isHidden()));
  await chat.locator('.chat-input').fill('Please explain this model.');
  await chat.locator('.chat-send').click();
  // On a real host the first request to a provider of someone else asks first (a local gateway, as here, does not).
  const dialog = page.locator('.monaco-dialog-box');
  if (await dialog.isVisible({ timeout: 3000 }).catch(() => false)) {
    report('the first request asks for consent, naming the destination', /ArcLang hosted/.test(await dialog.innerText()) && /\/api\/ai/.test(await dialog.innerText()));
    await dialog.getByText('Send', { exact: true }).first().click();
  }
  await chat.locator('.chat-turn.ai .chat-body p').first().waitFor({ timeout: 40000 });
  await page.waitForTimeout(1500);
  const answer = await chat.locator('.chat-turn.ai').last().innerText();
  report('the answer comes back through the gateway, streamed', answer.length > 20, answer.replace(/\s+/g, ' ').slice(0, 100));
  report('the request went to the site\'s /api/ai, with no key in it', ai.length >= 1 && ai.every((a) => a.key.length === 0), JSON.stringify(ai.slice(0, 2)));
  const strangers = [...hosts].filter((h) => h !== origin && !/vscode-cdn\.net$/.test(h));
  report('the page talked to no other host', strangers.length === 0, strangers.join(', '));
  await browser.close();
  const failed = rows.filter((r) => !r.ok).length;
  console.log(failed ? `${failed} FAILED` : 'all ok');
  process.exit(failed ? 1 : 0);
})();
