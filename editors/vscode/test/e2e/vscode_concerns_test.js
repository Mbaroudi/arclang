// Concerns, a requirement's subject and its constraints, inside the real VS
// Code: the Requirements view draws the concern a requirement frames, the
// subject it is about (with the requirements about it), and the constraints
// it requires and assumes, and the Logical view lists on the car the
// requirements about it; a constraint that is no comparison is a problem.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_concerns_test.js
// SHOTS=dir saves pictures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

const MODEL = `model "Car" {}

system_analysis "Needs" {
    concern "Driver safety" { id: "CON-SAFETY" stakeholders: ["Driver", "Insurer"] description: "The driver is not harmed in a crash" }
    requirement "Mass" {
        id: "REQ-MASS"
        description: "The car weighs at most 1200 kg"
        frames: ["CON-SAFETY"]
        subject: "LC-CAR"
        require: ["mass <= 1200 kg"]
        assume: ["ambient.T >= -20 °C"]
    }
    requirement "Braking" {
        id: "REQ-BRAKE"
        description: "The car stops from 100 km/h within 40 m"
        frames: ["CON-SAFETY"]
        subject: "LC-CAR"
        require: ["stopping_distance <= 40 m"]
    }
}

logical_architecture "Logic" {
    component "Car" { id: "LC-CAR" }
}
`;

async function launch(dir, model) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: ['Electron', 'Code'].map((name) => path.join(APP, 'Contents/MacOS', name)).find((exe) => fs.existsSync(exe)), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
  await win.waitForTimeout(1500);
  return { app, win, page, command };
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcconcern-'));
  const model = path.join(dir, 'model.arc');
  fs.writeFileSync(model, MODEL);
  const { app, win, page, command } = await launch(dir, model);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  try {
    report('the workbench opens with its diagram', !!page);
    if (!page) throw new Error('no workbench page');
    await page.locator('button, [role="tab"]', { hasText: /^Requirements/ }).first().click(); await win.waitForTimeout(2500);
    const node = (id) => page.locator(`.arcviz-stage .av-node[data-id="${id}"]`).first();
    const ids = await page.locator('.arcviz-stage .av-node').evaluateAll((ns) => ns.map((n) => n.getAttribute('data-id')));
    report('the requirements, the concern and the subject are drawn', ['REQ-MASS', 'REQ-BRAKE', 'CON-SAFETY', 'LC-CAR'].every((id) => ids.includes(id)), ids.join(' '));
    report('the concern is a concern box', (await node('CON-SAFETY').getAttribute('data-kind')) === 'concern', await node('CON-SAFETY').getAttribute('data-kind'));
    const text = async (id) => ((await node(id).textContent()) || '').replace(/\s+/g, ' ');
    report('the concern shows its stakeholders', /stakeholders: Driver, Insurer/.test(await text('CON-SAFETY')), await text('CON-SAFETY'));
    const mass = await text('REQ-MASS');
    report('the requirement shows its subject, frames, require and assume', /subject: LC-CAR/.test(mass) && /require/.test(mass) && /mass <= 1200 kg/.test(mass) && /assume/.test(mass) && /ambient\.T >= -20 °C/.test(mass), mass);
    report('the subject lists the requirements about it', /requirements\s*REQ-MASS\s*REQ-BRAKE/.test(await text('LC-CAR')), await text('LC-CAR'));
    const lines = await page.locator('.arcviz-stage .av-edge').evaluateAll((es) => es.map((e) => (e.querySelector('.av-edge-label')?.textContent || '').replace(/\s+/g, ' ').trim()));
    report('two frames lines and two subject lines are drawn', lines.filter((l) => /^frames/.test(l)).length === 2 && lines.filter((l) => /^subject/.test(l)).length === 2, lines.join(' | '));
    await shot('concerns-01-view');

    // In the Logical view, the car lists the requirements about it.
    await page.locator('button, [role="tab"]', { hasText: /^Logical/ }).first().click(); await win.waitForTimeout(2500);
    report('the Logical view lists the requirements about the car in its box', /requirements\s*REQ-MASS\s*REQ-BRAKE/.test(await text('LC-CAR')), await text('LC-CAR'));
    await shot('concerns-01b-logical');

    // A constraint that is no comparison: the compiler says so in Problems.
    fs.writeFileSync(model, MODEL.replace('"stopping_distance <= 40 m"', '"stopping_distance"'));
    await win.waitForTimeout(4000);
    await command('View: Focus Problems'); await win.waitForTimeout(1500);
    const problems = ((await win.locator('.markers-panel').textContent().catch(() => '')) || '').replace(/\s+/g, ' ');
    report('a require that is no comparison is a problem naming the requirement', /REQ-BRAKE/.test(problems) && /comparison/.test(problems), problems.slice(0, 200));
    await shot('concerns-02-problem');
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
