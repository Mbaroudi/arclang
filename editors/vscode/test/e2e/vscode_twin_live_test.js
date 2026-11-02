// A digital twin on live data in the real VS Code: the heat pump twin
// subscribed to an MQTT broker, the day replayed as JSON messages; the
// view follows each run of the twin, the banner says it is live, Stop ends it.
//   VSCODE_APP=... PLAYWRIGHT=... ARCLANG_BINARY=... ARCLANG_OMC_DOCKER=... MQTT_PYTHON=python-with-paho node vscode_twin_live_test.js
// (a broker on localhost:1883 that lets anonymous clients in)
const fs = require('fs'); const os = require('os'); const path = require('path');
const { spawn } = require('child_process');
const { launchVSCode } = require('./launch');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const REPO = path.resolve(EXT, '../..');
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(60)} ${detail || ''}`); };

const PUBLISH = `
import csv, json, sys, time
import paho.mqtt.client as mqtt
c = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2); c.connect("localhost", 1883); c.loop_start()
rows = [r for r in csv.reader(l for l in open(sys.argv[1]) if not l.startswith('#'))][1:]
for stamp, outdoor, indoor in rows:
    c.publish("house/042/sensors", json.dumps({"time": stamp, "outdoor_c": float(outdoor), "indoor_c": float(indoor)})); time.sleep(0.04)
time.sleep(1); c.loop_stop()
`;

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclive-'));
  fs.cpSync(path.join(REPO, 'examples/building/heat-pump'), path.join(dir, 'house'), { recursive: true });
  const model = path.join(dir, 'house', 'heat_pump_heating.arc');
  fs.writeFileSync(model, fs.readFileSync(model, 'utf8').replace('data: { file: "data/house-2026-01-14.csv" time: "timestamp" }', 'data: { mqtt: "mqtt://localhost:1883" topics: ["house/042/#"] window: 24 h refresh: 4 s record: "data/live.csv" }'));
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'arclang.binary': process.env.ARCLANG_BINARY || 'arclang', 'arclang.simulation.image': process.env.ARCLANG_OMC_DOCKER || '' };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', dir, model], { width: 1600, height: 1000 });
  const page = async () => { for (const f of win.frames()) if (await f.locator('#live').count().catch(() => 0)) return f; return null; };
  try {
    await win.waitForTimeout(6000);
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(400); await win.keyboard.type('ArcLang: Watch a digital twin live'); await win.waitForTimeout(700); await win.keyboard.press('Enter');
    let view = null;
    for (let i = 0; i < 30 && !view; i++) { await win.waitForTimeout(1000); view = await page(); }
    report('the live view opens', Boolean(view));
    if (!view) return;
    let connected = false;
    for (let i = 0; i < 20 && !connected; i++) { await win.waitForTimeout(500); connected = /localhost:1883|Live/.test(await view.locator('#live').innerText().catch(() => '')); }
    report('the banner says the broker is reached', connected, await view.locator('#live').innerText().catch(() => ''));
    const publisher = spawn(process.env.MQTT_PYTHON || 'python3', ['-c', PUBLISH, path.join(dir, 'house', 'data', 'house-2026-01-14.csv')], { stdio: 'ignore' });
    const seen = new Set();
    for (let i = 0; i < 60; i++) {
      await win.waitForTimeout(1000);
      const meta = await view.locator('.meta').first().innerText().catch(() => '');
      const n = /(\d+) samples/.exec(meta)?.[1];
      if (n) seen.add(Number(n));
      if (seen.has(289)) break;
    }
    publisher.kill();
    const counts = [...seen].sort((a, b) => a - b);
    report('the view follows the runs as the data arrives', counts.length >= 2 && counts[counts.length - 1] === 289, counts.join(' → '));
    report('the twin holds on the whole day', (await view.locator('section.twin .badge').first().innerText()).toLowerCase() === 'holds');
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'twin-live-01.png') });
    await view.locator('#stop').click();
    let stopped = false;
    for (let i = 0; i < 10 && !stopped; i++) { await win.waitForTimeout(500); stopped = /stopped/.test(await view.locator('#live').getAttribute('class')); }
    report('Stop ends the live run', stopped);
    const recorded = fs.existsSync(path.join(dir, 'house', 'data', 'live.csv')) ? fs.readFileSync(path.join(dir, 'house', 'data', 'live.csv'), 'utf8').split('\n').filter((l) => /^2026/.test(l)).length : 0;
    report('every sample was recorded', recorded === 289, `${recorded} rows`);
  } finally {
    await close();
  }
  const failed = rows.filter((r) => !r.ok).length;
  console.log(`${rows.length - failed}/${rows.length} passed`);
  process.exit(failed ? 1 : 0);
})();
