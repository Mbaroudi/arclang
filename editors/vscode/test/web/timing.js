const { chromium } = require('/Users/malek/node_modules/playwright');
(async () => {
  const browser = await chromium.launch({ args: ['--host-resolver-rules=MAP *.vscode-cdn.net ~NOTFOUND'] });
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await ctx.newPage();
  async function diagram() { const end = Date.now() + 60000; while (Date.now() < end) { for (const f of page.frames()) { try { if (await f.$('.arcviz-stage svg')) return; } catch {} } await page.waitForTimeout(100); } }
  for (let i = 0; i < 4; i++) {
    const t = Date.now();
    if (i === 0) await page.goto((process.env.BASE || 'http://localhost:8790/playground/vscode/')); else await page.reload();
    await page.waitForFunction(() => window.arclangPlayground && window.arclangPlayground.ready);
    const ready = Date.now() - t; await diagram();
    console.log(i === 0 ? 'cold' : 'warm', 'ready', ready, 'diagram', Date.now() - t, JSON.stringify(await page.evaluate(() => window.arclangPlayground.timing)));
    await page.waitForTimeout(1500);
  }
  await browser.close();
})();
