// Suspect traces and the model's own rules, inside the real VS Code: a rule
// of severity blocker is an error on the element that does not hold it; a
// reviewed trace whose requirement changes is a warning on the trace line,
// in Problems; its quick fix marks it reviewed again (new fingerprints in
// the file, the warning gone). The test case verifying that requirement is
// suspect too, on its own line, and its quick fix reviews the verification;
// ⌘Z takes that review back.
//   VSCODE_APP="/Applications/Visual Studio Code.app" PLAYWRIGHT=/path/to/playwright ARCLANG_BINARY=/path/to/arclang node vscode_suspect_test.js
const { launchVSCode } = require('./launch');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { spawnSync } = require('child_process');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const BINARY = process.env.ARCLANG_BINARY || 'arclang';
const SHOTS = process.env.SHOTS || null;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };

const MODEL = `model "Pack" {}

system_analysis "Needs" {
    requirement "Range" { id: "REQ-1" description: "Drives 300 km" }
}

physical_architecture "Hardware" {
    node "Pack" { id: "PN-1" mass: 450 kg }
    node "Cable" { id: "PN-2" }
}

rule "Every node declares its mass" { id: "R-MASS" kind: "PhysicalNode" require: "has(mass)" severity: "blocker" message: "a node without a mass cannot be budgeted" }

trace "PN-1" satisfies "REQ-1"
test_case "Range test" { id: "TC-1" verifies: ["REQ-1"] method: "test" procedure: "bench drive" }
`;

(async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcsus-'));
  const model = path.join(dir, 'pack.arc');
  fs.writeFileSync(model, MODEL);
  const reviewed = spawnSync(BINARY, ['trace', model, '--review'], { encoding: 'utf8' });
  const stamped = fs.readFileSync(model, 'utf8');
  const stamp = (/reviewed: "([0-9a-f ]+)"/.exec(stamped) || [])[1];
  const caseStamp = (/reviewed: \["REQ-1 ([0-9a-f ]+)"\]/.exec(stamped) || [])[1];
  const settings = { 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', 'files.autoSave': 'off', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}) };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const { win, close } = await launchVSCode(APP, [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', '--new-window', model], { width: 1600, height: 1000 });
  await win.waitForTimeout(6000);
  const press = await azerty(win);
  const command = async (name) => { await press('Meta+Shift+p'); await win.waitForTimeout(400); await win.keyboard.type(name); await win.waitForTimeout(700); await win.keyboard.press('Enter'); await win.waitForTimeout(900); };
  const shot = async (name) => { if (SHOTS) await win.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
  const problems = async () => { await command('Problems: Focus on Problems View'); await win.waitForTimeout(1200); return (await win.locator('.markers-panel').first().innerText().catch(() => '')).replace(/\s+/g, ' '); };
  try {
    report('the trace and the verification were reviewed by the command line', reviewed.status === 0 && !!stamp && !!caseStamp, `${stamp} / ${caseStamp} ${reviewed.stderr || ''}`.trim());
    // 1. The model's rule: an error on the node without a mass.
    await win.waitForTimeout(2500);
    let listed = await problems();
    report('a blocker rule is an error in Problems, on the node that does not hold it', /rule 'R-MASS' \(blocker\): 'PN-2': a node without a mass cannot be budgeted/.test(listed) && /\[Ln 9/.test(listed), listed.slice(0, 200));
    report('no trace is suspect yet', !/suspect:/.test(listed));
    await shot('suspect-01-rule');
    // 2. The requirement changes (outside the editor): the trace turns suspect, on its line.
    fs.writeFileSync(model, stamped.replace('Drives 300 km', 'Drives 320 km'));
    await win.waitForTimeout(4000);
    listed = await problems();
    report('the changed requirement makes the trace suspect, on the trace line', /suspect: trace 'PN-1' satisfies 'REQ-1': 'REQ-1' changed since the trace was reviewed/.test(listed) && /\[Ln 14/.test(listed), listed.slice(0, 260));
    report('and the verification of its test case, on the test case line', /suspect: verification 'TC-1' verifies 'REQ-1': 'REQ-1' changed since the verification was reviewed/.test(listed) && /\[Ln 15/.test(listed), listed.slice(0, 400));
    await shot('suspect-02-suspect');
    // 3. Its quick fix marks it reviewed again.
    const editor = win.locator('.monaco-editor .view-lines').first();
    await editor.click();
    await win.keyboard.press('Control+g'); await win.waitForTimeout(400); await win.keyboard.type('14'); await win.keyboard.press('Enter'); await win.waitForTimeout(600);
    // ⌘. is not the same key on every layout: the command palette opens the quick fixes.
    await command('Quick Fix...'); await win.waitForTimeout(1800);
    const offered = (await win.locator('.action-widget .monaco-list-row, .context-view .action-item').allInnerTexts().catch(() => [])).map((t) => t.replace(/\s+/g, ' '));
    const fix = offered.findIndex((t) => /Mark the trace PN-1 satisfies REQ-1 as reviewed/.test(t));
    report('the quick fix offers to mark the trace reviewed', fix >= 0, JSON.stringify(offered.slice(0, 6)));
    await shot('suspect-03-quickfix');
    if (fix >= 0) {
      // The first row is the group header; the first action is selected: walk down to the fix.
      const header = /^Quick Fix$/.test(offered[0] || '') ? 1 : 0;
      for (let i = header; i < fix; i += 1) { await win.keyboard.press('ArrowDown'); await win.waitForTimeout(150); }
      await win.keyboard.press('Enter');
      await win.waitForTimeout(4000);
    }
    const after = fs.readFileSync(model, 'utf8');
    const newStamp = (/reviewed: "([0-9a-f ]+)"/.exec(after) || [])[1];
    report('the file holds new fingerprints for the requirement end', !!newStamp && newStamp !== stamp && newStamp.split(' ')[0] === stamp.split(' ')[0], `${stamp} -> ${newStamp}`);
    listed = await problems();
    report('the trace warning is gone, the verification stays', !/suspect: trace/.test(listed) && /suspect: verification/.test(listed), listed.slice(0, 200));
    await shot('suspect-04-reviewed');
    // 4. The verification's quick fix, on the test case line.
    await editor.click();
    await win.keyboard.press('Control+g'); await win.waitForTimeout(400); await win.keyboard.type('15'); await win.keyboard.press('Enter'); await win.waitForTimeout(600);
    await command('Quick Fix...'); await win.waitForTimeout(1800);
    const offers = (await win.locator('.action-widget .monaco-list-row, .context-view .action-item').allInnerTexts().catch(() => [])).map((t) => t.replace(/\s+/g, ' '));
    const verify = offers.findIndex((t) => /Mark the verification TC-1 verifies REQ-1 as reviewed/.test(t));
    report('the quick fix offers to mark the verification reviewed', verify >= 0, JSON.stringify(offers.slice(0, 6)));
    await shot('suspect-05-verification');
    if (verify >= 0) {
      const header = /^Quick Fix$/.test(offers[0] || '') ? 1 : 0;
      for (let i = header; i < verify; i += 1) { await win.keyboard.press('ArrowDown'); await win.waitForTimeout(150); }
      await win.keyboard.press('Enter');
      await win.waitForTimeout(4000);
    }
    const final = fs.readFileSync(model, 'utf8');
    const newCase = (/reviewed: \["REQ-1 ([0-9a-f ]+)"\]/.exec(final) || [])[1];
    report('the test case holds new fingerprints for the requirement', !!newCase && newCase !== caseStamp && newCase.split(' ')[0] === caseStamp.split(' ')[0], `${caseStamp} -> ${newCase}`);
    const check = spawnSync(BINARY, ['trace', model, '--suspects'], { encoding: 'utf8' });
    report('the compiler finds nothing suspect any more', /1 of 1 trace\(s\) and 1 of 1 verification\(s\) reviewed, 0 suspect/.test(check.stdout), check.stdout.trim());
    listed = await problems();
    report('no suspect warning is left in Problems', !/suspect:/.test(listed), listed.slice(0, 160));
    await shot('suspect-06-reviewed');
    // 5. ⌘Z takes the last review back in the editor.
    await editor.click(); await press('Meta+z'); await win.waitForTimeout(2500);
    const text = await win.evaluate(() => document.querySelector('.monaco-editor .view-lines')?.innerText || '');
    await shot('suspect-07-undo');
    // The trace line holds the new requirement fingerprint too: read the test case line only.
    const caseLine = (text.split('\n').find((l) => l.includes('test_case')) || '').replace(/\u00a0/g, ' ');
    report('⌘Z takes the review back', caseLine.includes(caseStamp) && !caseLine.includes(newCase), caseLine.slice(0, 200));
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
