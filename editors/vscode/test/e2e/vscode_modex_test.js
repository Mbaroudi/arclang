// The Modelica export drawn by Modex inside the real VS Code, driven by
// Playwright through Electron: `arclang export -f modelica --split` writes the
// architecture as a library folder with Placement, Line, Icon and Diagram
// annotations; Modex (the Modelica editor, installed from the Marketplace)
// opens the .mo files of each model and draws boxes, labels and wires. The
// checks read the DOM of Modex's diagram webview, not only a picture.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang node vscode_modex_test.js
// Needs the Modex extension (MODEX_EXT, default ~/.vscode/extensions/modexai.modex-vscode-*)
// and the network once: Modex offers to install the Modelica Standard Library
// the first time it starts, and the test accepts. SHOTS=/dir keeps pictures.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { spawnSync } = require('child_process');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const ROOT = path.resolve(__dirname, '../../../..');
const SHOTS = process.env.SHOTS || null;
const BINARY = process.env.ARCLANG_BINARY || 'arclang';
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(72)} ${detail}`); };

function modexExtension() {
  if (process.env.MODEX_EXT) return process.env.MODEX_EXT;
  const dir = path.join(os.homedir(), '.vscode', 'extensions');
  const found = fs.existsSync(dir) ? fs.readdirSync(dir).filter((n) => n.startsWith('modexai.modex-vscode')).sort().pop() : null;
  return found ? path.join(dir, found) : null;
}

function online() {
  return spawnSync('curl', ['-sI', '--max-time', '8', 'https://libraries.openmodelica.org/index/v1/index.json']).status === 0;
}

// What each model must show: the files to open (relative to the workspace)
// and how many boxes and wires its text declares.
const CASES = [
  { name: 'EV range: configuration (4 nodes, 5 wires)', model: 'examples/automotive/ev_range_simulation.arc', open: 'EVRangeSimulation/Systems/Touring.mo' },
  { name: 'EV range: control unit node (own connectors, a controller)', model: 'examples/automotive/ev_range_simulation.arc', open: 'EVRangeSimulation/Physical/VehicleControlUnit.mo' },
  { name: 'Battery thermal loop: configuration (5 nodes, 8 wires)', model: 'tests/fixtures/modelica/thermal_loop.arc', open: 'BatteryThermalLoop/Systems/System.mo' },
  { name: 'Battery thermal loop: controller node (2 behaviors, a cycle)', model: 'tests/fixtures/modelica/thermal_loop.arc', open: 'BatteryThermalLoop/Physical/ThermalController.mo' },
  { name: 'EV product line: configuration', model: 'examples/automotive/ev_product_line.arc', open: 'EVProductLine/Systems/Touring.mo' },
  { name: 'ISR drone: system', model: 'examples/defense/isr_drone.arc', open: 'TacticalISRUnmannedAircraftSystem/Systems/System.mo' },
  { name: 'Flight control: system', model: 'examples/aerospace/flight_control_system.arc', open: 'PrimaryFlightControlSystem/Systems/System.mo' },
];

/** What the text of a model file declares: instance names, labels, connections. */
function declared(text) {
  const instances = [...text.matchAll(/^\s*(?:replaceable )?\S+ (\w+)(?: constrainedby \S+?(?:\([^)]*\))?)? "([^"]*)" annotation\(Placement/gm)].filter((m) => !/Interfaces\./.test(m[0]));
  return { names: instances.map((m) => m[1]), labels: instances.map((m) => m[2]), connects: (text.match(/^\s*connect\(/gm) || []).length, ports: (text.match(/Interfaces\.\w+ \w+ "[^"]*" annotation\(Placement/g) || []).length };
}

async function diagramFrame(win, label) {
  for (let i = 0; i < 80; i++) {
    for (const f of win.frames()) {
      try { if ((await f.locator('svg text', { hasText: label }).count()) > 0) return f; } catch { /* a frame going away */ }
    }
    await win.waitForTimeout(500);
  }
  return null;
}

async function snapshot(frame) {
  return frame.evaluate(() => ({
    texts: [...document.querySelectorAll('svg text')].map((t) => t.textContent.trim()),
    wires: document.querySelectorAll('g.mv-diagram-edge').length,
    boxes: [...document.querySelectorAll('svg rect')].map((r) => r.getBoundingClientRect()).filter((r) => r.width > 20 && r.height > 20),
  }));
}

(async () => {
  const modex = modexExtension();
  if (!modex) { console.log('SKIP Modex is not installed (MODEX_EXT, or ~/.vscode/extensions/modexai.modex-vscode-*)'); return; }
  if (!online()) { console.log('SKIP offline: Modex installs the Modelica Standard Library on first start'); return; }
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'mx-'));
  const ws = path.join(dir, 'w');
  fs.mkdirSync(ws);

  // 1. The exports, as the command line writes them.
  const wanted = new Map();
  for (const c of CASES) wanted.set(c.model, true);
  for (const model of wanted.keys()) {
    const run = spawnSync(BINARY, ['export', path.join(ROOT, model), '-f', 'modelica', '-o', ws, '--split'], { encoding: 'utf8' });
    report(`export ${path.basename(model)} (Modelica, one file per class)`, run.status === 0, run.status ? (run.stderr || run.stdout).slice(0, 160) : '');
  }
  const flat = path.join(dir, 'flat');
  const plain = spawnSync(BINARY, ['export', path.join(ROOT, 'examples/automotive/ev_range_simulation.arc'), '-f', 'modelica', '-o', flat, '--no-placement'], { encoding: 'utf8' });
  report('--no-placement writes the structure with no drawing', plain.status === 0 && !/Placement\(/.test(fs.readFileSync(path.join(flat, 'EVRangeSimulation.mo'), 'utf8')));

  // 2. The real VS Code with Modex, a fresh profile; the workspace is a library root.
  const settings = {
    'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off',
    'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none',
    'chat.disableAIFeatures': true, 'modex.libraryRoots': [ws],
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  fs.cpSync(modex, path.join(dir, 'x', path.basename(modex)), { recursive: true });
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, '--skip-welcome', '--skip-release-notes', ws] });
  const win = await app.firstWindow();
  try {
    await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1700, height: 1000 }));
    await win.waitForTimeout(10000);
    // The Modelica Standard Library prompt of the first start.
    const install = win.locator('.notifications-toasts .monaco-button:has-text("Install")').first();
    if (await install.count()) await install.click();
    else { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(500); await win.keyboard.type('Modex: Install Modelica Standard Library'); await win.waitForTimeout(1200); await win.keyboard.press('Enter'); }
    const library = path.join(dir, 'u', 'User', 'globalStorage', 'modexai.modex-vscode', 'managed-libraries');
    let installed = false;
    for (let i = 0; i < 120 && !installed; i++) { await win.waitForTimeout(1000); installed = fs.existsSync(library) && fs.readdirSync(library).some((n) => n.startsWith('Modelica') && fs.existsSync(path.join(library, n, 'package.mo'))); }
    report('the Modelica Standard Library is installed by Modex', installed);

    for (const c of CASES) {
      const text = fs.readFileSync(path.join(ws, c.open), 'utf8');
      const want = declared(text);
      await win.keyboard.press('Meta+P'); await win.waitForTimeout(500);
      await win.keyboard.type(path.basename(c.open) === 'System.mo' ? c.open : c.open); await win.waitForTimeout(1200); await win.keyboard.press('Enter');
      const probe = want.labels[0] || want.names[0];
      const frame = probe ? await diagramFrame(win, probe) : null;
      report(`${c.name}: the Modex diagram is open`, !!frame);
      if (!frame) { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `modex-${path.basename(c.open, '.mo')}-missing.png`) }); continue; }
      await win.waitForTimeout(2500);
      const seen = await snapshot(frame);
      const missingLabels = want.labels.filter((l) => !seen.texts.includes(l));
      report(`${c.name}: every component is drawn with its name`, want.names.length > 0 && missingLabels.length === 0 && want.names.every((n) => seen.texts.includes(n)), missingLabels.length ? `missing ${missingLabels.join(', ')}` : `${want.names.length} boxes`);
      report(`${c.name}: every connection is a wire`, seen.wires === want.connects, `${seen.wires} wires, ${want.connects} connects`);
      const overlap = seen.boxes.some((a, i) => seen.boxes.some((b, j) => j > i && a.left < b.right - 2 && b.left < a.right - 2 && a.top < b.bottom - 2 && b.top < a.bottom - 2 && !(a.left <= b.left && a.right >= b.right && a.top <= b.top && a.bottom >= b.bottom) && !(b.left <= a.left && b.right >= a.right && b.top <= a.top && b.bottom >= a.bottom)));
      report(`${c.name}: no box covers another`, !overlap);
      if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `modex-${c.open.replace(/[\\/]/g, '_').replace(/\.mo$/, '')}.png`) });
    }
  } catch (error) {
    report('the test ran to its end', false, String(error).slice(0, 240));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'modex-error.png') }).catch(() => undefined);
  } finally {
    await app.close().catch(() => undefined);
  }
  const failed = results.filter((ok) => !ok).length;
  console.log(`\n${results.length - failed}/${results.length} checks passed`);
  process.exit(failed ? 1 : 0);
})();
