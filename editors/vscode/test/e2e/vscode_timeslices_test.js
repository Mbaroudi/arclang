// Timeslices and snapshots inside the real VS Code: written in a component,
// highlighted, shown in the properties pane of the component with when they
// are and the values it has then; a snapshot outside its timeslice is a problem.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_timeslices_test.js
// SHOTS=dir saves pictures.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

const MODEL = `model "Mission" {}

logical_architecture "Drone" {
    component "Drone" {
        id: "LC-DRONE"
        altitude: 0 m
        battery: 1.0
        timeslice "Takeoff" {
            description: "From the pad to cruise altitude"
            start: 0 s
            end: 40 s
            snapshot "Liftoff" { at: 0 s altitude: 0 m }
            snapshot "At cruise" { at: 40 s altitude: 120 m }
        }
        snapshot "Landed" { at: 25 min altitude: 0 m battery: 0.1 }
    }
}
`;

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcslices-'));
  const model = path.join(dir, 'model.arc');
  fs.writeFileSync(model, MODEL);
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  try {
    await command('View: Close Secondary Side Bar');
    const keyword = await win.locator('.view-line span', { hasText: /^timeslice$/ }).first().evaluate((n) => getComputedStyle(n).color).catch(() => '');
    const plain = await win.locator('.view-line span', { hasText: /^altitude/ }).first().evaluate((n) => getComputedStyle(n).color).catch(() => '');
    report('timeslice is highlighted as a keyword', !!keyword && keyword !== plain, `${keyword} vs ${plain}`);
    await command('ArcLang: Open the workbench');
    let page = null;
    for (let i = 0; i < 60 && !page; i++) { await win.waitForTimeout(500); for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) page = f; } catch { /* a frame going away */ } } }
    report('the workbench opens', !!page);
    if (!page) throw new Error('no workbench page');
    await win.waitForTimeout(1500);
    await page.locator('button, [role="tab"]', { hasText: /^Logical/ }).first().click(); await win.waitForTimeout(2000);
    if (await page.locator('.wb-right').isHidden()) await page.locator('.wb-toolrow button[title="Properties"]').click();
    const box = await page.locator('.arcviz-stage [data-id="LC-DRONE"]').first().boundingBox();
    await win.mouse.click(box.x + 24, box.y + 8); await win.waitForTimeout(1200);
    const section = page.locator('.wb-sec[data-section="life"]');
    report('the properties pane has a Timeslices and snapshots section', (await section.count()) === 1, (await section.locator('.wb-sec-head').textContent().catch(() => '')) || '');
    const item = async (id) => ((await section.locator(`li[data-id="${id}"]`).first().textContent().catch(() => '')) || '').replace(/\s+/g, ' ');
    report('the timeslice says when it is', /Takeoff.*start 0 s · end 40 s/.test(await item('LC-DRONE.Takeoff')), await item('LC-DRONE.Takeoff'));
    report('its snapshots sit under it with their values', /At cruise.*at 40 s.*Altitude 120 m/i.test(await item('LC-DRONE.Takeoff.At cruise')), await item('LC-DRONE.Takeoff.At cruise'));
    report('a snapshot of the component itself', /Landed.*at 25 min.*Battery 0\.1/i.test(await item('LC-DRONE.Landed')), await item('LC-DRONE.Landed'));
    await section.scrollIntoViewIfNeeded().catch(() => {});
    await shot('slices-01-properties');

    fs.writeFileSync(model, MODEL.replace('snapshot "At cruise" { at: 40 s', 'snapshot "At cruise" { at: 45 s'));
    await win.waitForTimeout(4000);
    await command('View: Focus Problems'); await win.waitForTimeout(1500);
    const problems = ((await win.locator('.markers-panel').textContent().catch(() => '')) || '').replace(/\s+/g, ' ');
    report('a snapshot outside its timeslice is a problem', /at 45 s is outside its timeslice 'Takeoff'/.test(problems), problems.slice(0, 200));
    await shot('slices-02-problem');
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
