import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  Checklist,
  Diagnosis,
  checklist,
  countOutcomes,
  countsText,
  nextStep,
  outcomesLog,
  parseDiagnosis,
  parseOutcomes,
  safeImageName,
  simView,
  simulates,
  simulationEnv,
  summaryText,
  toolLabel,
  toolMissing,
} from '../../src/pure/simulation';
import { checklistHtml, escapeHtml } from '../../src/pure/simulation-html';

const IMAGE = 'openmodelica/openmodelica:v1.27.1-minimal';
const diagnosis = (over: Partial<Diagnosis> = {}): Diagnosis => ({
  tool: 'docker', detail: `docker image ${IMAGE}`, docker_installed: true, docker_running: true, image: IMAGE, image_pulled: true,
  library_volume: 'arclang-openmodelica-libraries', msl_version: '4.1.0', library_installed: true, local_omc: false, ...over,
});
const env = { platform: 'darwin', modexInstalled: false };
const item = (list: Checklist, id: string) => list.items.find((i) => i.id === id)!;

test('the diagnosis is read from the compiler JSON, and anything else is refused', () => {
  assert.deepEqual(parseDiagnosis(JSON.stringify(diagnosis())), diagnosis());
  assert.equal(parseDiagnosis('not json'), null);
  assert.equal(parseDiagnosis('[]'), null);
  assert.equal(parseDiagnosis(JSON.stringify({ ...diagnosis(), tool: 'wine' })), null);
  assert.equal(parseDiagnosis(JSON.stringify({ ...diagnosis(), docker_running: 'yes' })), null);
  assert.equal(parseDiagnosis(JSON.stringify({ ...diagnosis(), tool: null, image_pulled: null, library_installed: null }))?.tool, null);
});

test('everything in place: ready, nothing to fix', () => {
  const list = checklist(diagnosis(), env);
  assert.equal(list.ready, true);
  assert.equal(list.firstRun, false);
  assert.match(list.headline, /^Ready/);
  for (const id of ['docker', 'daemon', 'image', 'library']) assert.equal(item(list, id).state, 'ok', id);
  assert.equal(nextStep(list), null);
});

test('no Docker, no omc: download Docker Desktop first, nothing else can be fixed', () => {
  const list = checklist(diagnosis({ tool: null, detail: 'OpenModelica not available', docker_installed: false, docker_running: false, image_pulled: null, library_installed: null }), env);
  assert.equal(list.ready, false);
  assert.match(list.headline, /needs setup/);
  assert.equal(item(list, 'docker').state, 'todo');
  assert.deepEqual(item(list, 'docker').fixes.map((f) => f.kind), ['openUrl']);
  assert.equal(item(list, 'daemon').state, 'unknown');
  assert.equal(item(list, 'image').state, 'unknown');
  assert.deepEqual(item(list, 'image').fixes, []);
  assert.equal(nextStep(list)?.item.id, 'docker');
});

test('Docker installed but stopped: start it (Docker Desktop on a Mac, a typed command on Linux)', () => {
  const stopped = diagnosis({ tool: null, docker_running: false, image_pulled: null, library_installed: null });
  assert.deepEqual(item(checklist(stopped, env), 'daemon').fixes.map((f) => f.kind), ['startDocker']);
  const linux = item(checklist(stopped, { ...env, platform: 'linux' }), 'daemon').fixes[0];
  assert.deepEqual(linux, { kind: 'terminal', label: 'Show the command in a terminal', command: 'sudo systemctl start docker' });
  assert.equal(item(checklist(stopped, env), 'daemon').state, 'todo');
});

test('image missing: pull it, with the command offered too; the library waits for it', () => {
  const list = checklist(diagnosis({ image_pulled: false, library_installed: null }), env);
  assert.equal(item(list, 'image').state, 'todo');
  assert.deepEqual(item(list, 'image').fixes, [
    { kind: 'pullImage', label: 'Download the image', image: IMAGE },
    { kind: 'terminal', label: 'Show the command in a terminal', command: `docker pull ${IMAGE}` },
  ]);
  assert.equal(item(list, 'library').state, 'unknown');
  assert.equal(nextStep(list)?.fix.kind, 'pullImage');
});

test('library missing: the tool works, the first run is slow, and the library can be installed now', () => {
  const list = checklist(diagnosis({ library_installed: false }), env);
  assert.equal(list.ready, true);
  assert.equal(list.firstRun, true);
  assert.match(list.headline, /first simulation installs the Modelica library/);
  assert.match(item(list, 'library').detail, /few minutes/);
  assert.equal(item(list, 'library').blocking, false);
  assert.deepEqual(item(list, 'library').fixes, [{ kind: 'installLibrary', label: 'Install it now', image: IMAGE, volume: 'arclang-openmodelica-libraries', msl: '4.1.0' }]);
  assert.equal(nextStep(list)?.item.id, 'library');
});

test('a local omc makes Docker unnecessary', () => {
  const list = checklist(diagnosis({ tool: 'local', local_omc: true, docker_installed: false, docker_running: false, image_pulled: null, library_installed: null }), env);
  assert.equal(list.ready, true);
  assert.match(list.headline, /on this computer/);
  assert.equal(item(list, 'omc').state, 'ok');
  for (const id of ['docker', 'daemon', 'image', 'library']) assert.notEqual(item(list, id).state, 'todo', id);
  assert.equal(item(list, 'docker').state, 'na');
  assert.deepEqual(item(list, 'docker').fixes, []);
  assert.equal(nextStep(list), null);
});

test('without the compiler there is only one thing to do', () => {
  const list = checklist(null, env);
  assert.equal(list.ready, false);
  assert.equal(nextStep(list)?.fix.kind, 'installCompiler');
});

test('Modex is an optional extra with its install fix, and ticked once installed', () => {
  assert.deepEqual(item(checklist(diagnosis(), env), 'modex').fixes, [{ kind: 'installExtension', label: 'Install Modex', id: 'modexai.modex-vscode' }]);
  assert.equal(item(checklist(diagnosis(), env), 'modex').state, 'optional');
  const done = checklist(diagnosis(), { ...env, modexInstalled: true });
  assert.equal(item(done, 'modex').state, 'ok');
  assert.deepEqual(item(done, 'modex').fixes, []);
});

test('an image name that could pass for an option is never offered for a pull', () => {
  assert.equal(safeImageName(IMAGE), true);
  assert.equal(safeImageName('--privileged'), false);
  assert.equal(safeImageName('a b'), false);
  assert.equal(safeImageName('x;rm -rf'), false);
  const hostile = checklist(diagnosis({ image: '--privileged', image_pulled: false }), env);
  assert.deepEqual(item(hostile, 'image').fixes, []);
});

test('the settings become the environment variables the compiler reads', () => {
  assert.deepEqual(simulationEnv({ image: '', omc: '', timeout: 900 }), { ARCLANG_OMC_TIMEOUT: '900' });
  assert.deepEqual(simulationEnv({ image: ' my/om:1 ', omc: '/opt/om/bin/omc', timeout: 30.7 }), { ARCLANG_OMC_DOCKER: 'my/om:1', ARCLANG_OMC: '/opt/om/bin/omc', ARCLANG_OMC_TIMEOUT: '30' });
  assert.deepEqual(simulationEnv({ image: '', omc: '', timeout: 0 }), {});
});

// ---- The status bar state machine ----

const base = { relevant: true, checking: false, running: false, list: null as Checklist | null, toolLabel: null as string | null, last: null };

test('nothing to simulate: no item', () => {
  assert.equal(simView({ ...base, relevant: false }).state, 'hidden');
  assert.equal(simView({ ...base, relevant: false, running: true }).state, 'hidden');
});

test('the item goes checking, then ready or needs setup', () => {
  assert.equal(simView({ ...base, checking: true }).state, 'checking');
  const ready = checklist(diagnosis(), env);
  const view = simView({ ...base, list: ready, toolLabel: toolLabel(diagnosis()) });
  assert.equal(view.state, 'ready');
  assert.match(view.text, /Simulation: OpenModelica \(Docker\)/);
  assert.equal(simView({ ...base, list: checklist(diagnosis({ tool: 'local', local_omc: true }), env), toolLabel: 'OpenModelica (local)' }).text.includes('(local)'), true);
  const missing = simView({ ...base, list: checklist(diagnosis({ tool: null, docker_installed: false }), env) });
  assert.equal(missing.state, 'setup');
  assert.equal(missing.warning, true);
  assert.match(missing.text, /needs setup/);
});

test('running wins over everything; the verdict count follows the run', () => {
  const ready = checklist(diagnosis(), env);
  assert.equal(simView({ ...base, list: ready, running: true }).state, 'running');
  const failing = simView({ ...base, list: ready, last: { pass: 1, fail: 1, other: 0 } });
  assert.equal(failing.state, 'result');
  assert.equal(failing.text, '$(error) Simulation: 1 fail, 1 pass');
  assert.equal(failing.error, true);
  const passing = simView({ ...base, list: ready, last: { pass: 2, fail: 0, other: 0 } });
  assert.equal(passing.text, '$(pass) Simulation: 2 pass');
  assert.equal(passing.error, false);
  assert.equal(simView({ ...base, list: ready, last: { pass: 1, fail: 0, other: 1 } }).text, '$(error) Simulation: 1 inconclusive, 1 pass');
});

test('a tool that went missing after a run shows the setup, not a stale verdict', () => {
  const gone = checklist(diagnosis({ tool: null, docker_running: false }), env);
  assert.equal(simView({ ...base, list: gone, last: { pass: 2, fail: 0, other: 0 } }).state, 'setup');
});

// ---- Which models call for the item ----

test('a model that simulates, or has a physics port, is recognised; a comment is not', () => {
  assert.equal(simulates('test_case "x" { simulate: { model: "A.B" stop_time: 1 s } }'), true);
  assert.equal(simulates('port in p { physics: electrical }'), true);
  assert.equal(simulates('// simulate: nothing\nmodel M {}'), false);
  assert.equal(simulates('logical_component "simulated" {}'), false);
});

// ---- The verdicts ----

const VERDICTS = JSON.stringify([
  { test_case: 'TC-RANGE-STD', model: 'EVPhysics.StdRange', verdict: 'fail', reason: '', values: [{ value: 'min(batteryPack.soc) = 0.1', expect: '>= 0.2', holds: false }] },
  { test_case: 'TC-RANGE-LR', model: 'EVPhysics.LrRange', verdict: 'pass', reason: '', values: [{ value: 'min(batteryPack.soc) = 0.3', expect: '>= 0.2', holds: true }] },
]);

test('the verdicts of arclang verify --json are counted and logged', () => {
  const outcomes = parseOutcomes(VERDICTS)!;
  assert.deepEqual(countOutcomes(outcomes), { pass: 1, fail: 1, other: 0 });
  assert.equal(summaryText(countOutcomes(outcomes)), '1 fail, 1 pass.');
  assert.match(outcomesLog(outcomes), /FAIL {2}TC-RANGE-STD\n {6}FAIL min\(batteryPack.soc\) = 0.1 {3}\(>= 0.2\)\nPASS {2}TC-RANGE-LR/);
  assert.equal(summaryText({ pass: 3, fail: 0, other: 0 }), 'All 3 simulated cases pass.');
  assert.equal(summaryText({ pass: 0, fail: 0, other: 0 }), 'No test case in this model simulates.');
  assert.equal(countsText({ pass: 0, fail: 0, other: 0 }), 'no case');
  assert.equal(parseOutcomes('{}'), null);
  assert.equal(parseOutcomes('nope'), null);
});

test('a missing OpenModelica is told apart from a failing case', () => {
  const inconclusive = parseOutcomes(JSON.stringify([{ test_case: 'T', verdict: 'inconclusive', reason: 'OpenModelica not available: no omc', values: [] }]))!;
  assert.equal(toolMissing(inconclusive, ''), true);
  assert.equal(toolMissing(parseOutcomes(VERDICTS)!, ''), false);
  assert.equal(toolMissing([], 'Error: OpenModelica not available: x'), true);
});

// ---- The page ----

test('the checklist page escapes what it shows and names a button for each fix', () => {
  const list = checklist(diagnosis({ tool: null, docker_installed: false, image: '<img onerror=x>' }), env);
  const html = checklistHtml(list, 'N0NCE', 'vscode-resource:', 'banner <b>');
  assert.match(html, /script-src 'nonce-N0NCE'/);
  assert.match(html, /data-fix="docker:0">Download Docker Desktop/);
  assert.match(html, /data-fix="modex:0">Install Modex/);
  assert.equal(html.includes('<img onerror'), false);
  assert.equal(html.includes('banner <b>'), false);
  assert.match(html, /data-item="docker" data-state="todo"/);
  assert.equal(escapeHtml(`<a href="x">&'`), '&lt;a href=&quot;x&quot;&gt;&amp;&#39;');
});
