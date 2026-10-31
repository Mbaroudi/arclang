// The impact analysis inside the real VS Code, driven by Playwright through
// Electron, on a copy of the flagship example (a git repository of its own):
// a component analysed from the command palette, its requirement and test
// rows, the outline on another layer's diagram and the count on its tab; a
// what-if delete (the gate PASS -> FAIL, a new blocker) discarded, the file
// unchanged; a what-if latency of 80 ms on a function of the chain (the
// budget blocker), exported to Excel and read back with openpyxl, applied as
// one undo step (⌘Z typed as on an AZERTY Mac keyboard), and "Impact of my
// changes" since HEAD.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang node vscode_impact_test.js
// SHOTS=/dir keeps pictures.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { execFileSync } = require('child_process');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(64)} ${detail}`); };

async function launch(dir, model, extra = {}) {
  const settings = {
    'files.simpleDialog.enable': true, 'window.dialogStyle': 'custom', 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false,
    'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'git.enabled': false, 'git.openRepositoryInParentFolders': 'never', 'window.restoreWindows': 'none',
    ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}), ...extra,
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1700, height: 1000 });
  await win.waitForTimeout(4000);
  const palette = async (name) => {
    // Focus in the text: a key typed in a webview reaches the page, not always the palette.
    await win.locator('.tab', { hasText: 'aeb.arc' }).first().click().catch(() => undefined); await win.waitForTimeout(300);
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(800);
  };
  /** A picker is open and has its items (a command may compute before it asks). */
  const ready = async () => {
    for (let i = 0; i < 40; i++) {
      if (await win.locator('.quick-input-widget .quick-input-list .monaco-list-row').count().catch(() => 0)) return;
      await win.waitForTimeout(250);
    }
  };
  /** A command, then each pick typed and confirmed. */
  const command = async (name, picks = []) => {
    await palette(name);
    for (const pick of picks) { await ready(); await win.keyboard.type(pick); await win.waitForTimeout(600); await win.keyboard.press('Enter'); await win.waitForTimeout(800); }
  };
  await command('View: Close Secondary Side Bar');
  return { close, win, command, palette, ready };
}

/** The frame of the impact view (not one of `not`), once its rows are there. */
async function impactFrame(win, not = []) {
  for (let i = 0; i < 80; i++) {
    await win.waitForTimeout(400);
    for (const f of win.frames()) { try { if (!not.includes(f) && await f.$('#impact .ip-row, #impact .ip-error')) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

async function workbenchFrame(win) {
  for (let i = 0; i < 80; i++) {
    await win.waitForTimeout(400);
    for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

/** Wait until the view's title (its h1) matches. */
async function titled(frame, pattern) {
  for (let i = 0; i < 60; i++) {
    const text = await frame.textContent('.ip-title').catch(() => '');
    if (pattern.test(text) && !(await frame.$('#impact.is-stale'))) return text;
    await frame.page().waitForTimeout(300);
  }
  return await frame.textContent('.ip-title').catch(() => '');
}

const rowIds = (frame) => frame.$$eval('.ip-row', (rows) => rows.map((r) => r.dataset.id));

async function saveTo(win, file) {
  const input = win.locator('.quick-input-widget input').first();
  for (let i = 0; i < 20 && !(await input.isVisible().catch(() => false)); i++) await win.waitForTimeout(250);
  await input.fill(file); await input.press('Enter'); await win.waitForTimeout(2500);
}

function openpyxl(book, script) {
  const python = process.env.OPENPYXL_PYTHON || (() => { try { execFileSync('python3', ['-I', '-c', 'import openpyxl']); return 'python3'; } catch { return null; } })();
  if (!python) return { error: 'no Python with openpyxl: set OPENPYXL_PYTHON' };
  try { return JSON.parse(execFileSync(python, ['-I', '-c', script, book]).toString()); } catch (error) { return { error: String(error).slice(0, 200) }; }
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcimpact-'));
  const model = path.join(dir, 'aeb.arc');
  fs.copyFileSync(path.join(EXT, '..', '..', 'examples', 'complete_emergency_braking_simple.arc'), model);
  const original = fs.readFileSync(model, 'utf8');
  const git = (...args) => execFileSync('git', ['-C', dir, '-c', 'user.name=t', '-c', 'user.email=t@t', '-c', 'commit.gpgsign=false', ...args]);
  git('init', '-q'); git('add', 'aeb.arc'); git('commit', '-q', '-m', 'model');
  const { close, win, command, palette, ready } = await launch(dir, model);
  const press = await azerty(win);
  try {
    // 1. A sensor component analysed from the command palette
    await command('Impact analysis: analyse an element', ['RadarSensor']);
    const view = await impactFrame(win);
    report('the impact view opens in its own tab', !!view);
    if (!view) throw new Error('no impact view');
    const title = await titled(view, /Impact of /);
    report('the tab and the header name the element', /Impact of LC-RadarSensor/.test(title) && (await win.locator('.tab', { hasText: 'Impact of LC-RadarSensor' }).count()) >= 1, title);
    const ids = await rowIds(view);
    report('the requirement it satisfies is a row', ids.includes('REQ-AEB-001'), `${ids.length} rows`);
    report('the test that verifies it is a row, two steps away', ids.includes('TC-001'));
    const why = await view.textContent('.ip-row[data-id="TC-001"] .c-why');
    report('the row says why, step by step', /LC-RadarSensor.*satisfies.*REQ-AEB-001.*verified by.*TC-001/.test(why), why.replace(/\s+/g, ' '));
    const sev = await view.textContent('.ip-row[data-id="REQ-AEB-001"] .ip-pill');
    report('an ASIL-D requirement is a blocker', sev === 'blocker', sev);
    const chips = await view.textContent('.ip-chips');
    report('the header counts blockers, majors, minors and the gate', /\d+ blockers?/.test(chips) && /\d+ major/.test(chips) && /gate PASS/.test(chips), chips.replace(/\s+/g, ' '));
    report('the graph draws the set around the element', (await view.$$('.ip-node')).length > 10 && (await view.$$('.ip-node.sev-seed')).length === 1);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'impact-component.png') });

    // filters
    await view.click('.ip-toggle:has-text("Blockers")'); await win.waitForTimeout(400);
    const blockers = await rowIds(view);
    report('the severity filter keeps the blockers', blockers.length > 0 && blockers.length < ids.length && (await view.$$eval('.ip-row .ip-pill', (p) => p.every((x) => x.textContent === 'blocker'))));
    await view.click('.ip-toggle:has-text("Blockers")'); await win.waitForTimeout(300);

    // 2. The outline on another layer's diagram
    await command('ArcLang: Open the workbench beside the source');
    const bench = await workbenchFrame(win);
    report('the workbench opens', !!bench);
    if (!bench) throw new Error('no workbench');
    await bench.click('.wb-layer:has-text("Physical")'); await win.waitForTimeout(2500);
    let marked = [];
    for (let i = 0; i < 20 && !marked.length; i++) { marked = await bench.$$eval('.arcviz-stage .wb-impact', (n) => n.map((x) => `${x.dataset.id}:${x.className.baseVal || x.className}`)); if (!marked.length) await win.waitForTimeout(400); }
    report('the physical diagram outlines the node it is deployed on', marked.some((m) => /FrontRadarECU/.test(m)), marked.slice(0, 4).join(' '));
    const badge = await bench.$$eval('.wb-layer .wb-impact-count', (b) => b.map((x) => x.textContent));
    report('the layer tabs carry the counts', badge.length >= 3, badge.join(','));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'impact-highlight.png') });

    // 3. What if: delete, then Discard
    await command('Impact analysis: what if', ['SystemSupervisor']);
    await ready(); await win.keyboard.type('Delete'); await win.waitForTimeout(500); await win.keyboard.press('Enter');
    const whatif = await impactFrame(win);
    await titled(whatif, /What if SystemSupervisor|What if LC-SystemSupervisor/);
    for (let i = 0; i < 40 && !(await whatif.$('.ip-whatif')); i++) await win.waitForTimeout(300);
    const gate = await whatif.textContent('.ip-gate').catch(() => '');
    report('the gate goes from PASS to FAIL', /PASS\s*→\s*FAIL/.test(gate), gate.replace(/\s+/g, ' '));
    const newBlockers = await whatif.$$eval('.ip-blockers li.is-new', (l) => l.map((x) => x.textContent));
    report('a requirement is a new blocker', newBlockers.some((t) => /REQ-AEB-007/.test(t)), newBlockers.join(' | ').slice(0, 120));
    const cov = await whatif.textContent('.ip-cards').catch(() => '');
    report('the coverage says it is no longer satisfied', /REQ-AEB-007\s*covered\s*→\s*unsatisfied/.test(cov.replace(/\s+/g, ' ')));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'impact-whatif-delete.png') });
    await whatif.click('.ip-secondary:has-text("Discard")'); await win.waitForTimeout(2500);
    await command('File: Save All');
    report('Discard leaves the file unchanged', fs.readFileSync(model, 'utf8') === original);
    report('and the whatif panel is gone', !(await whatif.$('.ip-whatif')));

    // 4. What if the latency of a function of the chain is 80 ms
    await palette('Impact analysis: what if'); await win.waitForTimeout(1500);
    await ready(); await win.keyboard.type('FuseData'); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(800);
    await ready(); await win.keyboard.type('Change an attribute'); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800);
    await ready(); await win.keyboard.type('latency'); await win.waitForTimeout(500); await win.keyboard.press('Enter'); await win.waitForTimeout(800);
    await win.waitForTimeout(1200); await press('Meta+a'); await win.keyboard.type('80 ms'); await win.waitForTimeout(300); await win.keyboard.press('Enter');
    await titled(whatif, /latency = 80 ms/);
    for (let i = 0; i < 40 && !(await whatif.$('.ip-bar.is-over')); i++) await win.waitForTimeout(300);
    const gate2 = await whatif.textContent('.ip-gate').catch(() => '');
    report('the budget blocker flips the gate', /PASS\s*→\s*FAIL/.test(gate2), gate2.replace(/\s+/g, ' '));
    const nb = await whatif.$$eval('.ip-blockers li.is-new', (l) => l.map((x) => x.textContent));
    report('the new blocker is the chain over its budget', nb.some((t) => /EmergencyBrakingChain.*exceeds the 100\.0 ms budget/.test(t)), nb[0]?.slice(0, 140));
    const budgets = await whatif.textContent('.ip-cards');
    report('the budget bar shows 145 ms against 100 ms', /90 ms/.test(budgets) && /145 ms/.test(budgets) && /budget 100 ms/.test(budgets));
    report('the constraint flips', /Braking chain within budget\s*holds\s*→\s*violated/.test(budgets.replace(/\s+/g, ' ')));
    const chainRow = await whatif.textContent('.ip-row[data-id="FC-001"] .ip-pill').catch(() => null);
    report('the chain is a blocker in the table', chainRow === 'blocker', String(chainRow));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'impact-whatif-latency.png') });
    await command('File: Save All');
    report('nothing is written while the what-if is on screen', fs.readFileSync(model, 'utf8') === original);

    // 5. Excel export, read back
    await whatif.click('.ip-btn:has-text("Excel")');
    await saveTo(win, path.join(dir, 'impact.xlsx'));
    const read = openpyxl(path.join(dir, 'impact.xlsx'), 'import json,sys,openpyxl\nwb=openpyxl.load_workbook(sys.argv[1])\nimpact=wb["Impact"]\nprint(json.dumps({"sheets":wb.sheetnames,"head":[c.value for c in impact[1]],"first":[c.value for c in impact[2]],"ids":[r[0].value for r in impact.iter_rows(min_row=2)],"frozen":impact.freeze_panes,"budget":[[c.value for c in r] for r in wb["Budgets"].iter_rows(min_row=2)] if "Budgets" in wb.sheetnames else None}))');
    report('the workbook has the summary, impact, gate, budgets sheets', !read.error && ['Summary', 'Impact', 'Gate', 'Budgets', 'Coverage', 'Changes'].every((s) => read.sheets.includes(s)), read.error || read.sheets.join(','));
    report('the impact sheet lists the elements, blockers first', !read.error && read.head[0] === 'Element' && read.first[5] === 'blocker' && read.ids.includes('FC-001') && read.frozen === 'A2', read.error || String(read.first));
    report('the budget sheet carries before and after', !read.error && read.budget && read.budget[0][3] === '90 ms' && read.budget[0][4] === '145 ms', JSON.stringify(read.budget));

    // 6. Apply: one undo step
    await whatif.click('.ip-primary:has-text("Apply the change")'); await win.waitForTimeout(3500);
    await command('File: Save All');
    const applied = fs.readFileSync(model, 'utf8');
    report('Apply writes the change to the model', /function AcquireSensorData[\s\S]*function FuseData \{\s*latency: 80 ms/.test(applied) && applied !== original);
    const afterApply = await titled(whatif, /applied change/);
    report('the view turns to the impact of the applied change', /Impact of the applied change/.test(afterApply) && !(await whatif.$('.ip-whatif')), afterApply);

    // 7. Impact of my changes since HEAD
    await command('Impact analysis: my changes', ['Since HEAD']);
    await titled(whatif, /since HEAD/);
    for (let i = 0; i < 40 && !(await whatif.$('.ip-seed')); i++) await win.waitForTimeout(300);
    const seeds = await whatif.$$eval('.ip-seed', (s) => s.map((x) => x.textContent));
    report('"my changes" finds the function changed since HEAD', seeds.some((t) => /FuseData/.test(t)), seeds.join(' | '));
    const gate3 = await whatif.textContent('.ip-gate-chip').catch(() => '');
    report('and the gate before and after against HEAD', /PASS\s*→\s*FAIL/.test(gate3), gate3);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'impact-mine.png') });

    // 8. ⌘Z in the text (AZERTY) takes the applied change back in one step
    await win.locator('.tab', { hasText: 'aeb.arc' }).first().click(); await win.waitForTimeout(800);
    await win.locator('.monaco-editor').first().click({ position: { x: 300, y: 300 } }); await win.waitForTimeout(300);
    await press('Meta+z'); await win.waitForTimeout(2000);
    await command('File: Save All');
    report('one ⌘Z undoes the whole applied change', fs.readFileSync(model, 'utf8') === original);

    // 9. Themes
    if (SHOTS) {
      await win.locator('.tab', { hasText: 'Impact' }).first().click(); await win.waitForTimeout(1500);
      const settingsFile = path.join(dir, 'u', 'User', 'settings.json');
      const settings = JSON.parse(fs.readFileSync(settingsFile, 'utf8'));
      for (const [theme, file] of [['Default Dark Modern', 'impact-dark.png'], ['Default High Contrast', 'impact-hc.png'], ['Default Light Modern', null]]) {
        fs.writeFileSync(settingsFile, JSON.stringify({ ...settings, 'workbench.colorTheme': theme }));
        await win.waitForTimeout(3000);
        if (file) { await win.locator('.tab', { hasText: 'Impact' }).first().click(); await win.waitForTimeout(1500); await win.screenshot({ path: path.join(SHOTS, file) }); }
      }
    }
  } catch (error) {
    report('the run went to its end', false, String(error && error.message));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, 'impact-failure.png') }).catch(() => undefined);
  } finally {
    await close();
  }
  const failed = results.filter((ok) => !ok).length;
  console.log(failed ? `${failed} FAILED` : `all ${results.length} impact checks passed in VS Code`);
  process.exit(failed ? 1 : 0);
})();
