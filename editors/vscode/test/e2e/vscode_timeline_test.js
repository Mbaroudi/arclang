// Time travel inside the real VS Code, on a git history of the reference
// model: a snapshot, a branch merged back, another snapshot. The timeline
// bar lists the commits with their snapshots and the merge; a past commit
// is drawn as it was with the changes since the commit before marked (an
// element added in green), or since any moment chosen; an element restored
// from the past is an edit of the file; a merge of a conflicting branch
// lists the element in dispute in the Resolve conflicts view.
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang node vscode_timeline_test.js
// SHOTS=/dir keeps pictures.
const { launchVSCode } = require('./launch');
const { azerty } = require('./azerty');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { execFileSync } = require('child_process');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const BINARY = process.env.ARCLANG_BINARY || 'arclang';
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(66)} ${detail}`); };

/** A repository whose model has a history: snapshot, branch, merge, snapshot. */
function history(dir) {
  let clock = Date.parse('2026-10-20T09:00:00Z');
  const git = (...args) => {
    clock += 3 * 3600 * 1000;
    const when = new Date(clock).toISOString();
    return execFileSync('git', ['-c', 'user.name=Malek Baroudi', '-c', 'user.email=dev@example.com', '-c', 'commit.gpgsign=false', '-c', 'tag.gpgsign=false', ...args], { cwd: dir, env: { ...process.env, GIT_AUTHOR_DATE: when, GIT_COMMITTER_DATE: when }, encoding: 'utf8' });
  };
  const model = path.join(dir, 'lane_keeping.arc');
  const edit = (from, to) => fs.writeFileSync(model, fs.readFileSync(model, 'utf8').replace(from, to));
  git('init', '-q', '-b', 'main');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  git('add', '.'); git('commit', '-q', '-m', 'Initial lane keeping architecture');
  edit('id: "LC-CAM"\n    safety_level: "ASIL-B"', 'id: "LC-CAM"\n    safety_level: "ASIL-D"');
  git('commit', '-q', '-am', 'Raise the camera to ASIL-D');
  git('tag', '-a', 'design-review-1', '-m', 'Model as reviewed with the customer');
  git('switch', '-q', '-c', 'feature/driver-monitoring');
  edit('  component "LKA Controller" {', '  component "Driver Monitor" {\n    id: "LC-DMS"\n    safety_level: "ASIL-B"\n  }\n  component "LKA Controller" {');
  git('commit', '-q', '-am', 'Add driver monitoring');
  git('switch', '-q', 'main');
  edit('id: "LC-ACT"\n    safety_level: "ASIL-B"', 'id: "LC-ACT"\n    safety_level: "ASIL-C"');
  git('commit', '-q', '-am', 'Steering actuator to ASIL-C');
  git('merge', '-q', '--no-ff', '-m', 'Merge driver monitoring', 'feature/driver-monitoring');
  git('tag', '-a', 'v1.0', '-m', 'First release of the model');
  // A branch that will conflict with main on the actuator.
  git('switch', '-q', '-c', 'feature/actuator-asil-d');
  edit('safety_level: "ASIL-C"', 'safety_level: "ASIL-D"');
  git('commit', '-q', '-am', 'Actuator to ASIL-D');
  git('switch', '-q', 'main');
  execFileSync(BINARY, ['merge', '--install', '--yes', '--binary', BINARY], { cwd: dir });
  git('add', '.gitattributes'); git('commit', '-q', '-m', 'Merge the model by element');
  return model;
}

async function workbenchFrame(win, selector) {
  for (let i = 0; i < 80; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (await f.$(selector)) return f; } catch { /* a frame going away */ } }
  }
  return null;
}

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'tl-'));
  const repo = path.join(dir, 'r'); fs.mkdirSync(repo);
  const model = history(repo);
  const settings = {
    'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off',
    'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'window.dialogStyle': 'custom',
    'chat.disableAIFeatures': true, 'git.enabled': false, 'arclang.binary': BINARY,
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', repo, model], { width: 1700, height: 1000 });
  const press = await azerty(win);
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const command = async (name) => { await press('Meta+Shift+p'); await win.waitForTimeout(400); await win.keyboard.type(name); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(800); };
  try {
    await win.waitForTimeout(5000);
    if (await win.locator('.part.sidebar').isVisible().catch(() => false)) await press('Meta+b');
    await command('ArcLang: Timeline');
    const page = await workbenchFrame(win, '.wb-timeline');
    report('the timeline opens on the workbench', !!page);
    if (!page) throw new Error('no timeline');
    // The workbench alone in the window, the drawing at its size.
    await win.locator('.tab', { hasText: 'lane_keeping.arc' }).last().click(); await win.waitForTimeout(300);
    await press('Meta+k'); await press('Meta+m'); await win.waitForTimeout(1200);
    await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await win.waitForTimeout(1200);
    if (process.env.DEBUG_WIDTHS) console.log(await page.evaluate(() => ['.wb-timeline', '.wb-time-row', '.wb-time-rail-wrap', '.wb-time-rail', '.wb-time-track', '.wb-time-step'].map((s) => { const e = document.querySelector(s); const c = getComputedStyle(e); return `${s}: w=${Math.round(e.getBoundingClientRect().width)} flex=${c.flex} display=${c.display} width=${c.width} max=${c.maxWidth}`; }).join('\n')));
    const dots = await page.locator('.wb-time-dot').count();
    report('one notch per commit of the model, the merge as a diamond', dots === 5 && (await page.locator('.wb-time-dot.is-merge').count()) === 1, `${dots} commits`);
    report('the snapshots are flags on their commits', (await page.locator('.wb-time-tag').allTextContents()).join(',') === 'design-review-1,v1.0');

    // The commit that added the driver monitor, against the one before it.
    const slider = page.locator('.wb-time-slider');
    const indexOf = async (subject) => {
      for (let i = 0; i < 8; i++) {
        await slider.fill(String(i)); await win.waitForTimeout(150);
        if ((await page.locator('.wb-time-subject').textContent()) === subject) return i;
      }
      return -1;
    };
    const added = await indexOf('Add driver monitoring');
    report('the slider walks the commits', added >= 0, `at ${added}`);
    let ready = false;
    for (let i = 0; i < 60 && !ready; i++) { await win.waitForTimeout(500); ready = (await page.locator('.wb-cmp-panel .wb-cmp-title').textContent().catch(() => '')).startsWith('At '); }
    report('the drawing is the model at that commit, read-only', ready && (await page.locator('.wb-cmp-panel').textContent()).includes('A past moment'));
    await win.waitForTimeout(1500);
    const dms = page.locator('.arcviz-stage [data-id="LC-DMS"]');
    report('the element that commit added is on the drawing, marked added', (await dms.count()) > 0 && /added/.test(await dms.first().getAttribute('class') ?? ''), await dms.first().getAttribute('class').catch(() => ''));
    await shot('timeline-01-added');

    // The merge, against the snapshot design-review-1.
    const merge = await indexOf('Merge driver monitoring');
    await win.waitForTimeout(400);
    const option = await page.locator('.wb-time-base option', { hasText: 'design-review-1' }).getAttribute('value');
    await page.locator('.wb-time-base').selectOption(option);
    ready = false;
    for (let i = 0; i < 60 && !ready; i++) { await win.waitForTimeout(500); ready = (await page.locator('.wb-cmp-panel .wb-cmp-title').textContent().catch(() => '')).includes('since design-review-1'); }
    report('any moment compares with any other (since the snapshot)', ready, `merge at ${merge}`);
    await win.waitForTimeout(1500);
    const panelText = (await page.locator('.wb-cmp-panel').textContent()).replace(/\s+/g, ' ');
    report('since the review: the monitor added and the actuator modified', /Driver Monitor/.test(panelText) && /Steering Actuator/.test(panelText), panelText.slice(0, 160));
    await shot('timeline-02-since-snapshot');

    // Restore the actuator as it was at the review.
    const review = await indexOf('Raise the camera to ASIL-D');
    await win.waitForTimeout(2500);
    await page.locator('.arcviz-stage [data-id="LC-ACT"] > .av-shape').first().click({ force: true, position: { x: 30, y: 8 } }); await win.waitForTimeout(800);
    const restoreButton = page.locator('.wb-time-actions button', { hasText: 'Restore Steering Actuator' });
    report('back in time, "changes since" falls back to the commit before', (await page.locator('.wb-time-base').inputValue()) === '');
    report('the selected element can be restored from that moment', (await restoreButton.count()) === 1, `review at ${review}`);
    await restoreButton.click(); await win.waitForTimeout(800);
    await win.locator('.monaco-dialog-box .monaco-button', { hasText: 'Restore' }).first().click();
    await win.waitForTimeout(2500);
    await press('Meta+k'); await press('Meta+m'); await win.waitForTimeout(800);
    const tab = win.locator('.tab', { hasText: 'lane_keeping.arc' }).first();
    const dirty = /dirty/.test(await tab.getAttribute('class') ?? '');
    report('the restore is an edit of the file, not yet saved', dirty);
    await shot('timeline-03-restored');
    await press('Meta+s'); await win.waitForTimeout(1500);
    const saved = fs.readFileSync(model, 'utf8');
    report('the actuator is back to ASIL-B, the rest of today kept', /id: "LC-ACT"\n    safety_level: "ASIL-B"/.test(saved) && saved.includes('LC-DMS'));
    execFileSync('git', ['-c', 'user.name=t', '-c', 'user.email=t@t', 'commit', '-q', '-am', 'Actuator back to ASIL-B'], { cwd: repo });

    // A merge that conflicts on that element.
    await command('ArcLang: Merge a branch into the current one');
    await win.waitForTimeout(600);
    await win.keyboard.type('actuator'); await win.waitForTimeout(500); await win.keyboard.press('Enter');
    let conflicted = false;
    for (let i = 0; i < 40 && !conflicted; i++) { await win.waitForTimeout(500); conflicted = fs.readFileSync(model, 'utf8').includes('<<<<<<< ours'); }
    report('git merges by element and marks the element in dispute', conflicted);
    await win.waitForTimeout(1000);
    const notice = win.locator('.notifications-toasts .monaco-button', { hasText: 'Resolve conflicts' }).first();
    if (await notice.count()) await notice.click();
    await win.waitForTimeout(1500);
    const view = (await win.locator('.pane', { hasText: 'Resolve conflicts' }).first().textContent().catch(() => '')) || '';
    report('the Resolve conflicts view names the element', /Steering Actuator|LC-ACT/.test(view), view.replace(/\s+/g, ' ').slice(0, 120));
    await shot('timeline-04-conflict');
  } catch (error) {
    report('the test ran to its end', false, String(error).slice(0, 240));
    await shot('timeline-error').catch(() => undefined);
  } finally {
    await close();
  }
  const failed = results.filter((ok) => !ok).length;
  console.log(`\n${results.length - failed}/${results.length} checks passed`);
  process.exit(failed ? 1 : 0);
})();
