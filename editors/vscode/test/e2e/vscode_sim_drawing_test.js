// The simulation log and the Modelica drawing, from the ArcLang commands
// inside the real VS Code, with Modex: on a copy of the EV range example,
// "Run simulations" writes a log that says when, with what, what each case
// checks, how far each value is from its bound and what OpenModelica
// printed; "Export Modelica" writes one file per class and offers to open
// the drawing; the configuration picked opens drawn in Modex (not the empty
// package), its folder added to modex.libraryRoots when the user says so.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang ARCLANG_OMC_DOCKER=openmodelica/openmodelica:v1.27.1-minimal \
//   node vscode_sim_drawing_test.js
// Needs Modex (MODEX_EXT, default ~/.vscode/extensions/modexai.modex-vscode-*) and the network
// once (Modex installs the Modelica Standard Library on its first start). SHOTS=/dir keeps pictures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(72)} ${detail}`); };

function modexExtension() {
  if (process.env.MODEX_EXT) return process.env.MODEX_EXT;
  const dir = path.join(os.homedir(), '.vscode', 'extensions');
  const found = fs.existsSync(dir) ? fs.readdirSync(dir).filter((n) => n.startsWith('modexai.modex-vscode')).sort().pop() : null;
  return found ? path.join(dir, found) : null;
}

async function frameWith(win, selector, text, tries = 80) {
  for (let i = 0; i < tries; i++) {
    for (const f of win.frames()) {
      try { if ((await f.locator(selector, { hasText: text }).count()) > 0) return f; } catch { /* a frame going away */ }
    }
    await win.waitForTimeout(500);
  }
  return null;
}

(async () => {
  if (!process.env.ARCLANG_OMC_DOCKER && !process.env.ARCLANG_OMC) { console.log('SKIP set ARCLANG_OMC_DOCKER=<OpenModelica image> or ARCLANG_OMC'); return; }
  const modex = modexExtension();
  if (!modex) { console.log('SKIP Modex is not installed'); return; }
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcdraw-'));
  const ws = path.join(dir, 'w');
  const source = path.join(EXT, '..', '..', 'examples', 'automotive');
  fs.mkdirSync(path.join(ws, 'physics'), { recursive: true });
  const model = path.join(ws, 'ev.arc');
  fs.copyFileSync(path.join(source, 'ev_range_simulation.arc'), model);
  fs.copyFileSync(path.join(source, 'physics', 'EVPhysics.mo'), path.join(ws, 'physics', 'EVPhysics.mo'));

  const settings = {
    'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off',
    'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'chat.disableAIFeatures': true,
    ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}),
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  fs.cpSync(modex, path.join(dir, 'x', path.basename(modex)), { recursive: true });
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', ws] });
  const win = await app.firstWindow();
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(400); await win.keyboard.type(name); await win.waitForTimeout(800); await win.keyboard.press('Enter'); await win.waitForTimeout(900); };
  const button = (label) => win.locator(`.notifications-toasts .monaco-button:has-text("${label}")`).first();
  const clickWhenShown = async (label, seconds) => {
    for (let i = 0; i < seconds * 2; i++) { if (await button(label).count()) { await button(label).click(); return true; } await win.waitForTimeout(500); }
    return false;
  };
  try {
    await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1700, height: 1000 }));
    await win.waitForTimeout(8000);
    // Modex's first start: the Modelica Standard Library.
    const install = win.locator('.notifications-toasts .monaco-button:has-text("Install")').first();
    if (await install.count()) await install.click();
    else await command('Modex: Install Modelica Standard Library');
    const library = path.join(dir, 'u', 'User', 'globalStorage', 'modexai.modex-vscode', 'managed-libraries');
    let installed = false;
    for (let i = 0; i < 180 && !installed; i++) { await win.waitForTimeout(1000); installed = fs.existsSync(library) && fs.readdirSync(library).some((n) => n.startsWith('Modelica') && fs.existsSync(path.join(library, n, 'package.mo'))); }
    report('Modex has the Modelica Standard Library', installed);
    await command('Notifications: Clear All Notifications');

    // 1. The simulations, and their log.
    await win.keyboard.press('Meta+P'); await win.waitForTimeout(500); await win.keyboard.type('ev.arc'); await win.waitForTimeout(800); await win.keyboard.press('Enter'); await win.waitForTimeout(2500);
    await command('ArcLang: Run simulations');
    const shown = await clickWhenShown('Show the log', 300);
    report('the run ends on its verdicts, with "Show the log"', shown);
    await win.waitForTimeout(1500);
    const channel = await win.locator('.part.panel select.monaco-select-box').first().evaluate((s) => s.options[s.selectedIndex]?.text || '').catch(() => '');
    report('the log is its own output channel, "ArcLang Simulation"', channel === 'ArcLang Simulation', channel);
    // What the channel holds, as VS Code writes it to the window's logs.
    const channelFile = (root) => {
      for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
        const full = path.join(root, entry.name);
        if (entry.isDirectory()) { const found = channelFile(full); if (found) return found; } else if (/ArcLang Simulation\.log$/.test(entry.name)) return full;
      }
      return null;
    };
    const logFile = channelFile(path.join(dir, 'u', 'logs'));
    const log = logFile ? fs.readFileSync(logFile, 'utf8') : '';
    report('the log says when and which file', /^── \d\d:\d\d:\d\d {2}arclang verify ev\.arc/m.test(log), (log.split('\n')[0] || '').slice(0, 60));
    report('the log names the tool and its version', /Tool: .*OpenModelica 1\.\d+/.test(log), (log.match(/Tool:.*/) || [''])[0]);
    report('each case: its name, requirement, configuration and model', /✗ FAIL {2}TC-RANGE-STD "Winter range, standard pack"\n {6}verifies REQ-RANGE · configuration City \(CFG-CITY\) · model EVPhysics\.CityRange/.test(log));
    report('a failing value says how far it is from its bound', /FAIL min\(batteryPack\.soc\) = 0\.02659 .*short by 0\.07341/.test(log));
    report('a value that holds says what it has to spare', /final\(vehicleBody\.distance\) = 319\.9 km .*19\.91 km to spare/.test(log));
    report('what OpenModelica printed is in the log', /OpenModelica log \(\d+ lines\):\n {8}\| @@version OpenModelica/.test(log) && /The simulation finished successfully/.test(log));
    report('the run ends on its count and time', /1 fail, 1 pass\. Simulated in \d+\.\d s\./.test(log));
    await win.keyboard.press('Meta+End');
    await shot('sim-log');
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, 'sim-log.txt'), log);
    await command('Notifications: Clear All Notifications');

    // 2. The export, one file per class, and the drawing of a configuration.
    await win.keyboard.press('Meta+P'); await win.waitForTimeout(500); await win.keyboard.type('ev.arc'); await win.waitForTimeout(800); await win.keyboard.press('Enter'); await win.waitForTimeout(1200);
    await command('ArcLang: Export Modelica');
    const offered = await clickWhenShown('Open the drawing', 60);
    report('the export is split and offers to open the drawing', offered && fs.existsSync(path.join(ws, 'modelica', 'EVRangeSimulation', 'Systems', 'City.mo')));
    await win.waitForTimeout(1200);
    const picks = await win.locator('.quick-input-widget .monaco-list-row').allInnerTexts().catch(() => []);
    report('the drawing asks which configuration', picks.some((t) => /^City/.test(t)) && picks.some((t) => /^Touring/.test(t)), picks.map((t) => t.split('\n')[0]).join(', '));
    await win.keyboard.type('City'); await win.waitForTimeout(500); await win.keyboard.press('Enter');
    const added = await clickWhenShown('Add it', 30);
    await win.waitForTimeout(1500);
    const wsSettings = path.join(ws, '.vscode', 'settings.json');
    const roots = fs.existsSync(wsSettings) ? JSON.parse(fs.readFileSync(wsSettings, 'utf8'))['modex.libraryRoots'] : null;
    report('it offers to add the folder to modex.libraryRoots, for the workspace', added && Array.isArray(roots) && roots.includes(path.join(ws, 'modelica')), JSON.stringify(roots));
    const frame = await frameWith(win, 'svg text', 'Battery pack');
    report('the configuration opens drawn in Modex, not as an empty package', !!frame);
    if (frame) {
      await win.waitForTimeout(2500);
      const seen = await frame.evaluate(() => ({ texts: [...document.querySelectorAll('svg text')].map((t) => t.textContent.trim()), wires: document.querySelectorAll('g.mv-diagram-edge').length }));
      const parts = ['Battery pack', 'Drive unit', 'Vehicle body', 'Vehicle control unit'];
      report('the four nodes are drawn with their names', parts.every((p) => seen.texts.includes(p)), parts.filter((p) => !seen.texts.includes(p)).join(', '));
      report('the five connections are wires', seen.wires === 5, `${seen.wires} wires`);
    }
    await shot('sim-drawing-city');
    // Again: the root is known, no question, Touring opens.
    await command('Notifications: Clear All Notifications');
    await command('ArcLang: Open the Modelica drawing');
    await win.waitForTimeout(800);
    await win.keyboard.type('Touring'); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(2500);
    report('a known library root is not asked again', (await button('Add it').count()) === 0);
    const tab = await win.locator('.tabs-container .tab.active').first().innerText().catch(() => '');
    report('the other configuration opens from the command', /Touring/.test(tab), tab.replace(/\s+/g, ' '));
    await shot('sim-drawing-touring');
  } catch (error) {
    report('the test ran to its end', false, String(error).slice(0, 240));
    await shot('sim-drawing-error').catch(() => undefined);
  } finally {
    await app.close().catch(() => undefined);
  }
  const failed = results.filter((ok) => !ok).length;
  console.log(`\n${results.length - failed}/${results.length} checks passed`);
  process.exit(failed ? 1 : 0);
})();
