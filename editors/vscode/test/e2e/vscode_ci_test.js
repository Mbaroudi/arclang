// CI/CD inside the real VS Code, driven by Playwright through Electron, on a
// git repository of its own (a GitHub remote, the flagship model) and a stub
// `gh` that the test controls (so that no network and no login are needed):
//   - ArcLang: Configure CI: the quick pick of providers (the remote's ticked), a
//     preview of the file before anything is written, then the file written and
//     opened; a second run shows the file as existing and replaces it only
//     after a confirmation (cancelled first: the file is untouched);
//   - the status bar item: passed, failed, running, cancelled (not a pass),
//     `gh` missing, `gh` not logged in;
//   VSCODE_APP=".../Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright \
//   ARCLANG_BINARY=/path/to/arclang node vscode_ci_test.js
// SHOTS=/dir keeps pictures; THEME=dark|light picks the colour theme.
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { execFileSync } = require('child_process');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const SHOTS = process.env.SHOTS || null;
const THEME = process.env.THEME || 'light';
const results = [];
const report = (name, ok, detail = '') => { results.push(ok); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(74)} ${detail}`); };

const RUN = (over) => JSON.stringify([{ status: 'completed', conclusion: 'success', name: 'ArcLang', workflowName: 'ArcLang', displayTitle: 'the model', url: 'https://github.com/acme/rocket/actions/runs/42', headSha: '0123456789abcdef', headBranch: 'main', event: 'push', createdAt: new Date().toISOString(), ...over }]);

async function launch(dir, model, extraSettings = {}) {
  const settings = {
    'files.simpleDialog.enable': true, 'window.dialogStyle': 'custom', 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false,
    'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'git.enabled': false, 'git.openRepositoryInParentFolders': 'never', 'window.restoreWindows': 'none',
    'workbench.colorTheme': THEME === 'dark' ? 'Default Dark Modern' : 'Default Light Modern', 'window.autoDetectColorScheme': false,
    'arclang.ci.gh': path.join(dir, 'bin', 'gh'), 'arclang.ci.pollSeconds': 30,
    ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}),
    ...extraSettings,
  };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', dir, model], { width: 1500, height: 950 });
  await win.waitForTimeout(4500);
  const ready = async () => {
    for (let i = 0; i < 40; i++) {
      if (await win.locator('.quick-input-widget .quick-input-list .monaco-list-row').count().catch(() => 0)) return;
      await win.waitForTimeout(250);
    }
  };
  const palette = async (name) => {
    await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(400); await win.keyboard.type(name); await win.waitForTimeout(900); await win.keyboard.press('Enter'); await win.waitForTimeout(900);
  };
  /** Types into the open quick pick and takes the first match. */
  const pick = async (text) => { await ready(); await win.keyboard.type(text); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(900); };
  const dialog = async (label) => {
    const button = win.locator('.monaco-dialog-box .dialog-buttons a, .monaco-dialog-box .dialog-buttons .monaco-button', { hasText: label }).first();
    for (let i = 0; i < 40 && !(await button.isVisible().catch(() => false)); i++) await win.waitForTimeout(250);
    const text = await win.locator('.monaco-dialog-box').first().textContent().catch(() => '');
    await button.click();
    await win.waitForTimeout(800);
    return text;
  };
  const toasts = async () => (await win.locator('.notification-toast').allTextContents().catch(() => [])).join(' | ').replace(/\s+/g, ' ');
  /** The status bar item of ArcLang CI: its text's codicon and its tooltip. */
  const ciItem = async () => {
    const item = win.locator('.statusbar-item', { hasText: 'ArcLang CI' }).first();
    if (!(await item.count())) return null;
    const icon = await item.locator('.codicon').first().getAttribute('class').catch(() => '') || '';
    return { icon: (/codicon-([a-z-]+)/.exec(icon.replace('codicon-modifier-spin', '')) || [])[1] || '', label: await item.getAttribute('aria-label').catch(() => '') };
  };
  return { close, win, palette, pick, dialog, ready, toasts, ciItem };
}

const until = async (win, test, tries = 40) => { for (let i = 0; i < tries; i++) { const v = await test(); if (v) return v; await win.waitForTimeout(400); } return null; };

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcci-'));
  const model = path.join(dir, 'aeb.arc');
  fs.copyFileSync(path.join(EXT, '..', '..', 'examples', 'complete_emergency_braking_simple.arc'), model);
  const git = (...args) => execFileSync('git', ['-C', dir, '-c', 'user.name=Ada', '-c', 'user.email=ada@example.com', '-c', 'commit.gpgsign=false', ...args]).toString();
  git('init', '-q', '-b', 'main'); git('remote', 'add', 'origin', 'https://github.com/acme/rocket.git'); git('add', 'aeb.arc'); git('commit', '-q', '-m', 'the flagship model');
  // A stub `gh`: its answer is the mode the test writes, every call is logged.
  fs.mkdirSync(path.join(dir, 'bin'), { recursive: true });
  for (const [mode, over] of Object.entries({ success: {}, failure: { conclusion: 'failure' }, running: { status: 'in_progress', conclusion: '' }, cancelled: { conclusion: 'cancelled' } })) fs.writeFileSync(path.join(dir, `run-${mode}.json`), RUN(over));
  fs.writeFileSync(path.join(dir, 'bin', 'gh'), `#!/bin/bash\necho "$@" >> "${dir}/calls.log"\nmode="$(cat "${dir}/gh-mode")"\nif [ "$mode" = nologin ]; then echo "To get started with GitHub CLI, please run:  gh auth login" >&2; exit 4; fi\ncat "${dir}/run-$mode.json"\n`, { mode: 0o755 });
  const mode = (m) => fs.writeFileSync(path.join(dir, 'gh-mode'), m);
  const calls = () => (fs.existsSync(path.join(dir, 'calls.log')) ? fs.readFileSync(path.join(dir, 'calls.log'), 'utf8').split('\n').filter(Boolean) : []);
  mode('success');
  const workflow = path.join(dir, '.github', 'workflows', 'arclang.yml');

  const { close, win, palette, pick, dialog, ready, toasts, ciItem } = await launch(dir, model);
  try {
    // 1. No workflow yet: no status item, and gh is not asked.
    await win.waitForTimeout(2500);
    report('without the workflow there is no status item and gh is not called', (await ciItem()) === null && calls().length === 0, `calls ${calls().length}`);

    // 2. Configure CI: the providers, the preview, then the write.
    await palette('ArcLang: Configure CI');
    await ready();
    const providers = await win.locator('.quick-input-list .monaco-list-row').allTextContents();
    report('the quick pick offers GitHub Actions, GitLab CI, Azure DevOps and Jenkins', ['GitHub Actions', 'GitLab CI', 'Azure DevOps', 'Jenkins'].every((p) => providers.some((t) => t.includes(p))), providers.map((t) => t.replace(/\s+/g, ' ').slice(0, 40)).join(' | '));
    const ticked = await win.locator('.quick-input-list .monaco-list-row[aria-checked="true"]').allTextContents();
    report('the provider of the git remote (GitHub) is ticked', ticked.length === 1 && ticked[0].includes('GitHub Actions'), ticked.join('|'));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `ci-providers-${THEME}.png`) });
    await win.keyboard.press('Enter'); await win.waitForTimeout(3500);
    await ready();
    const files = await win.locator('.quick-input-list .monaco-list-row').allTextContents();
    report('the preview lists the file to write and the Write and Cancel choices', files.some((t) => t.includes('.github/workflows/arclang.yml')) && files.some((t) => /Write 1 file/.test(t)) && files.some((t) => /Cancel/.test(t)), files.map((t) => t.replace(/\s+/g, ' ').slice(0, 60)).join(' | '));
    report('nothing is written yet', !fs.existsSync(workflow));
    await win.keyboard.press('Enter'); await win.waitForTimeout(1800); // the first item: the file
    const tab = await until(win, async () => (await win.locator('.tab', { hasText: 'arclang.yml' }).count()) ? true : null, 10);
    const previewText = await win.locator('.editor-instance .view-lines').first().textContent().catch(() => '');
    report('picking the file opens its preview', !!tab && /ArcLang gate|name: ArcLang/.test(previewText.replace(/ /g, ' ')), previewText.replace(/\s+/g, ' ').slice(0, 80));
    report('the preview is still not a file on disk', !fs.existsSync(workflow));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `ci-preview-${THEME}.png`) });
    await ready();
    await pick('Write 1 file');
    await until(win, () => (fs.existsSync(workflow) ? true : null), 20);
    const written = fs.existsSync(workflow) ? fs.readFileSync(workflow, 'utf8') : '';
    report('Write puts the workflow in the repository', written.includes('name: ArcLang gate') && written.includes('arclang ci run'), `${written.length} bytes`);
    report('the other providers were not written', !fs.existsSync(path.join(dir, 'Jenkinsfile')) && !fs.existsSync(path.join(dir, 'azure-pipelines.yml')));
    await win.waitForTimeout(1500);
    const message = await toasts();
    report('the editor says the files are not committed and not run', /not committed, not run/.test(message), message.slice(0, 160));

    // 3. The status bar item follows the stub's runs.
    await palette('ArcLang: Refresh the CI pipeline status');
    const passed = await until(win, async () => { const i = await ciItem(); return i && i.icon === 'check' ? i : null; }, 15);
    report('a successful run is a check in the status bar', !!passed, JSON.stringify(passed));
    report('gh was asked for the workflow on the branch, one run, as JSON', calls().some((c) => /^run list --workflow arclang.yml --branch main --limit 1 --json/.test(c)), calls().slice(-1)[0]);
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `ci-status-passed-${THEME}.png`) });
    const state = async (m, icon) => { mode(m); await palette('ArcLang: Refresh the CI pipeline status'); return until(win, async () => { const i = await ciItem(); return i && i.icon === icon ? i : null; }, 15); };
    report('a failed run is an error', !!(await state('failure', 'error')));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `ci-status-failed-${THEME}.png`) });
    report('a run in progress is a spinner', !!(await state('running', 'sync')));
    report('a cancelled run is a question mark, never a pass', !!(await state('cancelled', 'question')));

    // 4. gh not logged in, then gh missing: said in words, no token asked.
    mode('nologin'); await palette('ArcLang: Refresh the CI pipeline status');
    const nologin = await until(win, async () => { const i = await ciItem(); return i && i.icon === 'question' && /gh auth login/.test(i.label) ? i : null; }, 15);
    report('gh not logged in: a question mark whose tooltip says "gh auth login"', !!nologin, (nologin && nologin.label || '').slice(0, 100));
    await win.locator('.statusbar-item', { hasText: 'ArcLang CI' }).first().click(); await win.waitForTimeout(1200);
    const warning = await toasts();
    report('clicking it explains, and no token is stored', /gh auth login/.test(warning) && /stores no token/.test(warning), warning.slice(0, 160));
    await win.keyboard.press('Escape');
    const settingsPath = path.join(dir, 'u', 'User', 'settings.json');
    const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
    fs.writeFileSync(settingsPath, JSON.stringify({ ...settings, 'arclang.ci.gh': path.join(dir, 'bin', 'no-such-gh') }));
    await win.waitForTimeout(2500); await palette('ArcLang: Refresh the CI pipeline status');
    const missing = await until(win, async () => { const i = await ciItem(); return i && i.icon === 'question' && /not installed/.test(i.label) ? i : null; }, 15);
    report('gh missing: a question mark whose tooltip says it is not installed', !!missing, (missing && missing.label || '').slice(0, 100));
    fs.writeFileSync(settingsPath, JSON.stringify(settings));
    mode('success'); await win.waitForTimeout(2500); await palette('ArcLang: Refresh the CI pipeline status');
    report('and it recovers when gh is back', !!(await until(win, async () => { const i = await ciItem(); return i && i.icon === 'check' ? i : null; }, 15)));

    // 5. Configure CI again: the file exists, it is replaced only after a confirmation.
    fs.writeFileSync(workflow, `# edited by hand\n${written}`);
    await palette('ArcLang: Configure CI'); await ready(); await win.keyboard.press('Enter'); await win.waitForTimeout(3500); await ready();
    const again = await win.locator('.quick-input-list .monaco-list-row').allTextContents();
    report('the preview says the file exists and would be replaced', again.some((t) => /exists: would be replaced/.test(t)), again.map((t) => t.replace(/\s+/g, ' ').slice(0, 70)).join(' | '));
    await pick('Write 1 file');
    const sure = await dialog('Cancel').catch(() => '');
    report('a modal asks before replacing it', /Replace them\?/.test(sure) && /arclang\.yml/.test(sure), String(sure).replace(/\s+/g, ' ').slice(0, 120));
    await win.waitForTimeout(800);
    await ready().catch(() => undefined);
    report('cancelling the confirmation leaves the file as it was', fs.readFileSync(workflow, 'utf8').startsWith('# edited by hand'));
    await pick('Write 1 file');
    await dialog('Replace');
    await until(win, () => (!fs.readFileSync(workflow, 'utf8').startsWith('# edited by hand') ? true : null), 20);
    report('confirming replaces it', !fs.readFileSync(workflow, 'utf8').startsWith('# edited by hand') && fs.readFileSync(workflow, 'utf8').includes('ArcLang gate'));
    const real = (() => { try { return execFileSync('git', ['-C', dir, 'status', '--porcelain', '-uall']).toString(); } catch { return ''; } })();
    report('nothing was committed or pushed', /arclang\.yml/.test(real) && execFileSync('git', ['-C', dir, 'log', '--oneline']).toString().trim().split('\n').length === 1, real.trim().split('\n').join(' ; '));
  } catch (error) {
    report('the run', false, String(error).slice(0, 300));
    if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `ci-failure-${THEME}.png`) }).catch(() => undefined);
  } finally {
    await close();
  }

  // Not covered here: an untrusted workspace. In Restricted Mode the extension host of this harness
  // never finishes starting the development extension (nothing of ArcLang activates), so a check
  // there would pass for the wrong reason. The guard is `workspaceTrusted()` before every gh call
  // and before the command (src/cicd/index.ts).
  fs.rmSync(dir, { recursive: true, force: true });
  const failed = results.filter((r) => !r).length;
  console.log(`\n${results.length - failed}/${results.length} passed`);
  process.exit(failed ? 1 : 0);
})();
