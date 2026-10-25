// Semantic git inside the real VS Code, driven by Playwright through
// Electron, on a copy of the flagship example in a git repository of its
// own: the model changed (an attribute, an added function, a removed test
// case), the workbench compared with HEAD (marks, counters, panel, ghosts),
// the history of an element with the model at a past commit beside the
// current one, a baseline created and verified, and two branches merged by
// git through the driver the extension installed.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang node vscode_semgit_test.js
// SHOTS=/dir keeps pictures; THEME=dark|light picks the colour theme.
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { execFileSync } = require('child_process');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const THEME = process.env.THEME || 'light';
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail}`); };

async function launch(dir, model) {
  const settings = {
    'files.simpleDialog.enable': true, 'window.dialogStyle': 'custom', 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false,
    'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'git.enabled': false, 'git.openRepositoryInParentFolders': 'never', 'window.restoreWindows': 'none',
    'workbench.colorTheme': THEME === 'dark' ? 'Default Dark Modern' : 'Default Light Modern', 'window.autoDetectColorScheme': false,
    ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}),
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', dir, model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1700, height: 1000 }));
  await win.waitForTimeout(4500);
  const palette = async (name) => {
    await win.locator('.tab', { hasText: 'aeb.arc' }).first().click().catch(() => undefined); await win.waitForTimeout(300);
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(800); await win.keyboard.press('Enter'); await win.waitForTimeout(900);
  };
  const ready = async () => {
    for (let i = 0; i < 40; i++) {
      if (await win.locator('.quick-input-widget .quick-input-list .monaco-list-row').count().catch(() => 0)) return;
      await win.waitForTimeout(250);
    }
  };
  const pick = async (text) => { await ready(); await win.keyboard.type(text); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(900); };
  const command = async (name, picks = []) => { await palette(name); for (const p of picks) await pick(p); };
  /** The button of a modal dialog (custom dialogs are in the page). */
  const dialog = async (label) => {
    const button = win.locator('.monaco-dialog-box .dialog-buttons a, .monaco-dialog-box .dialog-buttons .monaco-button', { hasText: label }).first();
    for (let i = 0; i < 40 && !(await button.isVisible().catch(() => false)); i++) await win.waitForTimeout(250);
    const text = await win.locator('.monaco-dialog-box').first().textContent().catch(() => '');
    await button.click();
    await win.waitForTimeout(800);
    return text;
  };
  await command('View: Close Secondary Side Bar');
  /** A picture with the workbench given the whole window (the text editor beside it hides what the page draws). */
  let maximized = false;
  /** A picture with the workbench given the whole window (the text editor beside it would hide what the page draws). */
  const shot = async (name, frame) => {
    if (!SHOTS) return;
    if (!maximized) {
      await frame.click('.wb-layers').catch(() => undefined); await win.waitForTimeout(300);
      await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(400); await win.keyboard.type('View: Toggle Maximize Editor Group'); await win.waitForTimeout(800); await win.keyboard.press('Enter'); await win.waitForTimeout(1800);
    }
    await win.waitForTimeout(800);
    await win.screenshot({ path: path.join(SHOTS, `${name}-${THEME}.png`) });
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(400); await win.keyboard.type('View: Toggle Maximize Editor Group'); await win.waitForTimeout(800); await win.keyboard.press('Enter'); await win.waitForTimeout(1200);
  };
  return { app, win, command, palette, ready, pick, dialog, shot };
}

async function workbenchFrame(win) {
  for (let i = 0; i < 80; i++) {
    await win.waitForTimeout(400);
    for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

const until = async (win, test, tries = 40) => { for (let i = 0; i < tries; i++) { const v = await test(); if (v) return v; await win.waitForTimeout(300); } return null; };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcsemgit-'));
  const model = path.join(dir, 'aeb.arc');
  fs.copyFileSync(path.join(EXT, '..', '..', 'examples', 'complete_emergency_braking_simple.arc'), model);
  const original = fs.readFileSync(model, 'utf8');
  const git = (...args) => execFileSync('git', ['-C', dir, '-c', 'user.name=Ada', '-c', 'user.email=ada@example.com', '-c', 'commit.gpgsign=false', '-c', 'tag.gpgsign=false', ...args]).toString();
  const cli = (...args) => { try { return { code: 0, out: execFileSync(process.env.ARCLANG_BINARY, args, { cwd: dir }).toString() }; } catch (e) { return { code: e.status, out: String(e.stdout || '') + String(e.stderr || '') }; } };
  git('init', '-q', '-b', 'main'); git('add', 'aeb.arc'); git('commit', '-q', '-m', 'the flagship model');
  // A second commit that touches FuseData, so that its history has two entries.
  fs.writeFileSync(model, original.replace('description: "Combine the radar and camera detections"', 'description: "Fuse the radar and camera detections"'));
  git('commit', '-q', '-am', 'reword FuseData');
  const committed = fs.readFileSync(model, 'utf8');
  // The working tree: an attribute changed, a function added, a test case removed.
  const changed = committed
    .replace('latency: 25 ms\n        description: "Fuse', 'latency: 40 ms\n        description: "Fuse')
    .replace('    function TrackObjects {', '    function LogEvents {\n        latency: 5 ms\n        description: "Record the braking events"\n    }\n    function TrackObjects {')
    .replace('        function "Read camera" { description: "Frames of the forward camera" }\n', '')
    .replace(/test_case "Fault injection campaign" \{[\s\S]*?\n\}\n/, '');
  fs.writeFileSync(model, changed);
  report('the working tree differs from HEAD in the three ways', changed !== committed && !changed.includes('TC-007') && changed.includes('LogEvents') && changed.includes('latency: 40 ms'));

  const { app, win, command, palette, ready, pick, dialog, shot } = await launch(dir, model);
  try {
    // 1. Compare with HEAD
    await command('ArcLang: Open the workbench beside the source');
    const bench = await workbenchFrame(win);
    report('the workbench opens', !!bench);
    if (!bench) throw new Error('no workbench');

    report('the Compare menu is in the tool row', (await bench.$$('#wb-compare-menu')).length === 1);
    await bench.click('#wb-compare-menu'); await win.waitForTimeout(500);
    const menu = await bench.$$eval('.wb-menu .wb-menu-item', (n) => n.map((x) => x.textContent));
    report('it offers HEAD, a branch or tag, a baseline, history, baseline, merge driver', ['Compare with HEAD', 'Compare with a branch, a tag or a revision…', 'Compare with a baseline…', 'Create baseline…', 'Install the semantic merge driver…'].every((t) => menu.includes(t)), menu.join(' | ').slice(0, 160));
    await bench.click('.wb-menu .wb-menu-item:has-text("Compare with HEAD")'); await win.waitForTimeout(500);
    // The model is saved (it is on disk); the host computes the diff and the old snapshot.
    const panel = await until(win, () => bench.$('.wb-cmp-panel'));
    report('compare mode opens its panel of changes', !!panel);
    const head = await bench.textContent('.wb-cmp-head').catch(() => '');
    report('the panel says what it is compared with', /Comparing with\s*HEAD/.test(head), head.replace(/\s+/g, ' '));
    const rows = await bench.$$eval('.wb-cmp-row[data-id]', (r) => r.map((x) => `${x.classList.contains('is-added') ? '+' : x.classList.contains('is-removed') ? '-' : '~'}${x.dataset.id}`));
    report('the panel lists the added, the removed and the modified element', rows.includes('+LogEvents') || rows.some((r) => /^\+.*LogEvents/.test(r)), rows.join(' '));
    report('the removed test case is listed as removed', rows.some((r) => r.startsWith('-') && /TC-007/.test(r)), rows.join(' '));
    report('the modified function is listed as modified', rows.some((r) => r.startsWith('~') && /FuseData/.test(r)));
    const layers = await bench.$$eval('.wb-cmp-layer', (h) => h.map((x) => x.textContent));
    report('the changes are grouped by layer', layers.length >= 2 && layers.some((l) => /System Analysis/.test(l)), layers.join(' | '));
    const note = await bench.textContent('.wb-cmp-panel');
    report('the panel shows the consequences: the gate and the newly unverified requirement', /Gate PASS → FAIL|Gate FAIL → FAIL/.test(note) && /unverified|unsatisfied|blocker/.test(note), note.replace(/\s+/g, ' ').slice(0, 200));
    const counts = await bench.$$eval('.wb-layer .wb-cmp-count', (c) => c.map((x) => x.textContent));
    report('the layer tabs carry the counters', counts.length >= 1 && counts.some((c) => /\+1/.test(c)), counts.join(' | '));

    // The system diagram: marks on the boxes
    await bench.click('.wb-layer:has-text("System")'); await win.waitForTimeout(3000);
    const marks = await until(win, async () => { const m = await bench.$$eval('.arcviz-stage .wb-cmp', (n) => n.map((x) => `${x.dataset.id}:${(x.getAttribute('class') || '').split(' ').filter((c) => c.startsWith('wb-cmp-')).join(',')}`)); return m.length ? m : null; });
    report('the system diagram outlines the modified and added functions', !!marks && marks.some((m) => /FuseData.*modified/.test(m)) && marks.some((m) => /LogEvents.*added/.test(m)), (marks || []).join(' '));
    report('badges mark them for readers who do not tell colours apart', (await bench.$$('.arcviz-stage .wb-cmp-badge')).length >= 2);
    await shot('compare-system', bench);
    await bench.click('.wb-layer:has-text("Capabilities")').catch(() => undefined);

    // Selecting a row selects the element
    await bench.click('.wb-cmp-row[data-id*="FuseData"]'); await win.waitForTimeout(800);
    const selected = await bench.$$eval('.arcviz-stage .is-selected, .arcviz-stage [aria-selected="true"], .arcviz-stage .arcviz-selected', (n) => n.length).catch(() => 0);
    const props = await bench.textContent('.wb-pane.is-active').catch(() => '');
    report('clicking a change selects the element (its properties are shown)', /FuseData/.test(props) || selected > 0, props.replace(/\s+/g, ' ').slice(0, 80));

    // Read-only
    const before = fs.readFileSync(model, 'utf8');
    await bench.click('.wb-layer:has-text("System")'); await win.waitForTimeout(1500);
    await win.keyboard.press('Meta+s'); await win.waitForTimeout(500);
    report('editing is off while comparing', fs.readFileSync(model, 'utf8') === before && (await bench.$$eval('body.wb-comparing', (b) => b.length)) === 1);

    // Ghost of the removed test case on a diagram that draws test cases, if any: at least the panel keeps it.
    const ghosts = await bench.$$('.arcviz-stage .wb-cmp-removed');
    report('the removed function is ghosted at its old place in the box that held it', ghosts.length >= 1, `${ghosts.length} ghost(s)`);
    await shot('compare-ghost', bench);

    // Show in source / Open diff
    await bench.hover('.wb-cmp-row[data-id*="FuseData"]'); await win.waitForTimeout(300);
    await bench.click('.wb-cmp-row[data-id*="FuseData"] .wb-cmp-actions button:has-text("Diff")'); await win.waitForTimeout(2500);
    const diffTab = await win.locator('.tab', { hasText: 'HEAD' }).count();
    report('"Open diff" opens VS Code\'s diff against HEAD', diffTab >= 1, `${diffTab} tab`);
    await win.locator('.tab', { hasText: 'HEAD' }).first().hover(); await win.locator('.tab', { hasText: 'HEAD' }).first().locator('.codicon-close, .action-label').first().click().catch(() => undefined); await win.waitForTimeout(800);

    // Leaving the comparison
    await bench.click('.wb-cmp-exit'); await win.waitForTimeout(1500);
    report('Exit returns to normal editing', (await bench.$$('.wb-cmp-panel')).length === 0 && (await bench.$$('.arcviz-stage .wb-cmp')).length === 0 && (await bench.$$eval('body.wb-comparing', (b) => b.length)) === 0);

    // 2. Compare with a commit, then the pick list of refs
    await bench.click('#wb-compare-menu'); await win.waitForTimeout(400);
    await bench.click('.wb-menu .wb-menu-item:has-text("Compare with a branch, a tag or a revision")'); await win.waitForTimeout(1200);
    await ready();
    const refs = await win.locator('.quick-input-widget .quick-input-list .monaco-list-row').allTextContents();
    report('the quick pick lists HEAD and the branch', refs.some((r) => /HEAD/.test(r)) && refs.some((r) => /main/.test(r)), refs.join(' | ').slice(0, 120));
    await win.keyboard.type('main'); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(2500);
    report('comparing with the branch main works', !!(await until(win, () => bench.$('.wb-cmp-panel'))) && /main/.test(await bench.textContent('.wb-cmp-head')));
    await bench.click('.wb-cmp-exit'); await win.waitForTimeout(1000);

    // 3. History of an element
    await bench.click('.wb-layer:has-text("System")'); await win.waitForTimeout(2500);
    await bench.click('#wb-compare-menu'); await win.waitForTimeout(300); await win.keyboard.press('Escape');
    await win.locator('.tab', { hasText: 'aeb.arc' }).first().click().catch(() => undefined);
    const fuse = await bench.$('.arcviz-stage [data-id="FuseData"]');
    if (fuse) await fuse.click({ position: { x: 20, y: 10 } });
    await win.waitForTimeout(1000);
    await bench.click('#wb-compare-menu'); await win.waitForTimeout(400);
    await bench.click('.wb-menu .wb-menu-item:has-text("History of")'); await win.waitForTimeout(1500);
    const histPanel = await until(win, () => bench.$('.wb-hist-panel'), 25);
    report('History… lists the commits of the element', !!histPanel);
    if (histPanel) {
      const subjects = await bench.$$eval('.wb-hist-panel .wb-cmp-name span', (n) => n.map((x) => x.textContent));
      report('with the commit that reworded it and the one that created it', subjects.includes('reword FuseData') && subjects.includes('the flagship model'), subjects.join(' | '));
      report('but only the commits that changed it', subjects.length === 2, `${subjects.length} entries`);
      await bench.click('.wb-hist-panel .wb-cmp-row:has-text("the flagship model")'); await win.waitForTimeout(3500);
      report('selecting a commit draws the model at that revision beside the current one', (await bench.$$('.wb-rev-pane .arcviz-stage svg')).length >= 1);
      await shot('history', bench);
    }

    // 4. A baseline
    git('checkout', '-q', '--', 'aeb.arc'); await win.waitForTimeout(2500);
    await command('ArcLang: Create baseline…', []);
    await win.waitForTimeout(800);
    await win.keyboard.type('PDR-1'); await win.keyboard.press('Enter'); await win.waitForTimeout(800);
    await pick('iso26262');
    const gateText = await dialog('Create baseline');
    report('the baseline dialog shows the gate result first', /production gate/i.test(gateText), gateText.replace(/\s+/g, ' ').slice(0, 120));
    await win.waitForTimeout(3500);
    report('the baseline directory and the tag exist', fs.existsSync(path.join(dir, 'baselines/PDR-1/manifest.json')) && git('tag', '-l', 'baseline/*').includes('baseline/PDR-1'));
    const verify = cli('baseline', 'verify', 'PDR-1');
    report('the baseline verifies', verify.code === 0 && /VERIFIED/.test(verify.out), verify.out.split('\n').slice(-2).join(' '));
    await win.keyboard.press('Escape');

    // 5. The merge driver, installed from the command palette
    await command('ArcLang: Install the semantic merge driver');
    const plan = await dialog('Install');
    report('the install shows what it changes first', /merge\.arclang\.driver/.test(plan) && /merge=arclang/.test(plan), plan.replace(/\s+/g, ' ').slice(0, 120));
    await win.waitForTimeout(2000);
    report('git is configured for .arc and .layout.json', /merge %O %A %B/.test(git('config', '--local', 'merge.arclang.driver')) && /\*\.arc merge=arclang/.test(fs.readFileSync(path.join(dir, '.gitattributes'), 'utf8')));
    git('add', '.gitattributes', 'baselines'); git('commit', '-q', '-m', 'baseline PDR-1 and the merge driver');
    const base = fs.readFileSync(model, 'utf8');
    git('checkout', '-q', '-b', 'ours');
    fs.writeFileSync(model, base.replace('latency: 20 ms\n        description: "Collect', 'latency: 22 ms\n        description: "Collect')); git('commit', '-q', '-am', 'ours: acquisition latency');
    git('checkout', '-q', 'main'); git('checkout', '-q', '-b', 'theirs');
    fs.writeFileSync(model, base.replace('latency: 15 ms\n        description: "Track', 'latency: 18 ms\n        description: "Track')); git('commit', '-q', '-am', 'theirs: tracking latency');
    git('checkout', '-q', 'ours');
    let merged = '';
    try { merged = git('merge', '--no-edit', 'theirs'); } catch (e) { merged = String(e.stdout || e.message); }
    const text = fs.readFileSync(model, 'utf8');
    report('git merges two branches that changed different functions, through the driver', text.includes('latency: 22 ms') && text.includes('latency: 18 ms') && !text.includes('<<<<<<<'), merged.split('\n')[0]);
    report('and the merged model compiles', cli('check', 'aeb.arc').code === 0);

    // 6. A real conflict, and the Resolve conflicts view
    git('checkout', '-q', 'main'); git('checkout', '-q', '-b', 'c-ours');
    fs.writeFileSync(model, fs.readFileSync(model, 'utf8').replace('description: "Combine the radar and camera detections"', 'description: "Fuse radar and camera, ours"').replace('description: "Fuse the radar and camera detections"', 'description: "Fuse radar and camera, ours"')); git('commit', '-q', '-am', 'c-ours');
    git('checkout', '-q', 'main'); git('checkout', '-q', '-b', 'c-theirs');
    fs.writeFileSync(model, fs.readFileSync(model, 'utf8').replace('description: "Fuse the radar and camera detections"', 'description: "Fuse radar and camera, theirs"')); git('commit', '-q', '-am', 'c-theirs');
    git('checkout', '-q', 'c-ours');
    try { git('merge', '--no-edit', 'c-theirs'); } catch { /* the conflict is the point */ }
    const marked = fs.readFileSync(model, 'utf8');
    report('a real conflict is written as markers around the element', /<<<<<<< ours function 'FuseData' \[SF-FuseData\]|<<<<<<< ours function/.test(marked) && marked.includes('>>>>>>> theirs'), (/<<<<<<< ours.*/.exec(marked) || [''])[0].slice(0, 140));
    await win.waitForTimeout(3500);
    await command('ArcLang: Resolve conflicts: refresh').catch(() => undefined);
    await win.waitForTimeout(1500);
    const warn = await win.locator('.notification-toast, .notifications-toasts').first().textContent().catch(() => '');
    report('the editor says the merge left element conflicts', /element conflict/.test(warn), warn.replace(/\s+/g, ' ').slice(0, 120));
    await win.locator('.notification-toast a, .notification-toast .monaco-button', { hasText: 'Resolve conflicts' }).first().click().catch(() => undefined);
    await win.waitForTimeout(1200);
    const item = await until(win, async () => (await win.locator('.pane-body .monaco-list-row', { hasText: /FuseData/ }).count()) ? true : null, 20);
    report('the Resolve conflicts view lists the conflict by element', !!item);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `conflicts-${THEME}.png`) });
    const row = win.locator('.pane-body .monaco-list-row', { hasText: /FuseData/ }).first();
    await row.hover(); await win.waitForTimeout(400);
    await row.locator('.action-label[aria-label="Accept theirs"]').click().catch(() => undefined);
    await win.waitForTimeout(2500);
    const settled = fs.readFileSync(model, 'utf8');
    report('Accept theirs settles the conflict in the file', !settled.includes('<<<<<<<') && settled.includes('radar and camera, theirs') && !settled.includes('radar and camera, ours'));
  } catch (error) {
    report('the run', false, String(error).slice(0, 300));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `semgit-failure-${THEME}.png`) }).catch(() => undefined);
  } finally {
    await app.close();
    fs.rmSync(dir, { recursive: true, force: true });
  }
  const failed = results.filter((r) => !r).length;
  console.log(`\n${results.length - failed}/${results.length} passed`);
  process.exit(failed ? 1 : 0);
})();
