// The Modelica drawing in the real VS Code: a scenario of the data center
// drawn from OpenModelica's instance (its parts, their icons, the
// connections), then a part opened to its own diagram (the Standard
// Library's icons) and the path back.
//   VSCODE_APP=... PLAYWRIGHT=... ARCLANG_BINARY=... ARCLANG_OMC_DOCKER=... node vscode_modelica_drawing_test.js
const fs = require('fs'); const os = require('os'); const path = require('path');
const { launchVSCode } = require('./launch');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const REPO = path.resolve(EXT, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail || ''}`); };
const example = path.join(REPO, 'examples/datacenter/ai-liquid-cooling');

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcmo-'));
  fs.cpSync(example, path.join(dir, 'dc'), { recursive: true });
  const model = path.join(dir, 'dc', 'ai_liquid_cooling.arc');
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'arclang.binary': process.env.ARCLANG_BINARY || 'arclang' };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', dir, model], { width: 1600, height: 1000 });
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const page = async () => { for (const f of win.frames()) if (await f.locator('#stage').count().catch(() => 0)) return f; return null; };
  try {
    await win.waitForTimeout(6000);
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(400); await win.keyboard.type('ArcLang: Open the Modelica drawing'); await win.waitForTimeout(700); await win.keyboard.press('Enter');
    const quick = win.locator('.quick-input-widget input').first();
    for (let i = 0; i < 40 && !(await quick.isVisible().catch(() => false)); i++) await win.waitForTimeout(500);
    const offered = await win.locator('.quick-input-widget .monaco-list-row').allInnerTexts().catch(() => []);
    report('the scenarios and configurations are offered', offered.length >= 4 && offered.some((o) => /Design summer day/.test(o)), `${offered.length} offered`);
    await quick.fill('Design summer day'); await win.waitForTimeout(500); await win.keyboard.press('Enter');
    let view = null;
    for (let i = 0; i < 90 && !view; i++) { await win.waitForTimeout(1000); const f = await page(); if (f && (await f.locator('#stage svg g.component').count()) > 0) view = f; }
    report('the scenario is drawn', Boolean(view));
    if (!view) return;
    await shot('modelica-01-scenario');
    const parts = await view.locator('#stage svg g.component').evaluateAll((gs) => gs.map((g) => g.getAttribute('data-name')));
    report('its six parts', parts.length === 6 && parts.includes('coolantDistributionUnit'), parts.join(', '));
    report('the connections are drawn', (await view.locator('#stage svg path').count()) > 20);
    report('the summary names the class', /CoolingPhysics\.SummerDay/.test(await view.locator('#summary').innerText()));
    await view.locator('#stage svg g.component[data-name="coolantDistributionUnit"]').click();
    let inner = false;
    for (let i = 0; i < 60 && !inner; i++) { await win.waitForTimeout(1000); inner = /CoolingPhysics\.CDU/.test(await view.locator('#summary').innerText().catch(() => '')); }
    report('a part opens its own diagram', inner);
    await shot('modelica-02-cdu');
    const libraryParts = await view.locator('#stage svg g.component').evaluateAll((gs) => gs.map((g) => g.getAttribute('data-name')));
    report('with the Standard Library parts', libraryParts.includes('coolant') && libraryParts.includes('fromCoolant'), libraryParts.join(', '));
    report('the path shows where it is', (await view.locator('#crumbs').innerText()).replace(/\s+/g, ' ').includes('coolantDistributionUnit'));
    await view.locator('#crumbs button.crumb').first().click();
    let back = false;
    for (let i = 0; i < 20 && !back; i++) { await win.waitForTimeout(500); back = /SummerDay/.test(await view.locator('#summary').innerText()); }
    report('the path goes back', back);
  } finally {
    await close();
  }
  const failed = rows.filter((r) => !r.ok).length;
  console.log(`${rows.length - failed}/${rows.length} passed`);
  process.exit(failed ? 1 : 0);
})();
