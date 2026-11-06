import assert from 'node:assert/strict';
import { test } from 'node:test';
import { applies, complete, Ladder, lineOf, markOf, nextCommands, onModel, reached, stageTitle, summarize, withBinary, withModel } from '../../src/pure/realization';
import { parseSnapshot } from '../../src/pure/workbench';

const stage = (name: string, state: Ladder['stages'][number]['state'], cases: { id: string; state: string; reason?: string }[] = [], target = '') => ({ stage: name, state, cases, target });

const window: Ladder = {
  requirement: 'SYS-PW-001',
  name: 'Auto direction from the position',
  stages: [stage('simulation', 'n/a'), stage('sil', 'pass', [{ id: 'TC-PW-001', state: 'pass' }], 'arclang sil (cc)'), stage('pil', 'stale', [{ id: 'TC-PW-001', state: 'stale', reason: 'its PIL verdict is stale; run `arclang pil model.arc --test TC-PW-001`' }], 'zephyr/litex_vexriscv'), stage('hdl', 'missing'), stage('soc', 'missing'), stage('board', 'missing'), stage('twin', 'n/a')],
  next: 'arclang pil model.arc --test TC-PW-001',
};
const comfort: Ladder = {
  requirement: 'REQ-COMFORT',
  name: 'Comfort on the design day',
  stages: [stage('simulation', 'pass'), stage('sil', 'n/a'), stage('pil', 'n/a'), stage('hdl', 'n/a'), stage('soc', 'n/a'), stage('board', 'n/a'), stage('twin', 'pass', [], 'data/house.csv')],
  next: '',
};
const idle: Ladder = { requirement: 'REQ-DOC', name: 'Documented', stages: ['simulation', 'sil', 'pil', 'hdl', 'soc', 'board', 'twin'].map((s) => stage(s, 'n/a')), next: '' };
const failing: Ladder = { requirement: 'REQ-X', name: 'X', stages: [stage('simulation', 'fail', [{ id: 'TC-X', state: 'fail', reason: 'short by 0.08' }]), ...['sil', 'pil', 'hdl', 'soc', 'board', 'twin'].map((s) => stage(s, 'n/a'))], next: 'simulation TC-X: the simulation verdict fails; see the case' };

test('the marks are those arclang check prints, and a ladder reads as its line', () => {
  assert.equal(['pass', 'fail', 'error', 'stale', 'missing', 'n/a'].map(markOf).join(''), '✓✗✗~○–');
  assert.equal(lineOf(window), 'SYS-PW-001 "Auto direction from the position"  simulation – · SIL ✓ · PIL ~ · HDL ○ · SoC ○ · board ○ · twin –');
  assert.equal(lineOf({ ...idle, name: 'REQ-DOC' }), 'REQ-DOC  simulation – · SIL – · PIL – · HDL – · SoC – · board – · twin –');
});

test('a requirement reaches its highest fresh pass, is complete when every stage passes, and is idle with no stage', () => {
  assert.equal(reached(window), 'sil');
  assert.equal(complete(window), false);
  assert.equal(reached(comfort), 'twin');
  assert.equal(complete(comfort), true);
  assert.equal(applies(idle), false);
  assert.equal(reached(idle), null);
});

test('the summary counts the requirements on the ladder by what holds them', () => {
  const summary = summarize([window, comfort, idle, failing]);
  assert.equal(summary.requirements, 4);
  assert.equal(summary.climbing, 3);
  assert.equal(summary.complete, 1);
  assert.equal(summary.failing, 1);
  assert.equal(summary.stale, 1);
  assert.equal(summary.missing, 0);
  assert.equal(summary.byStage.sil.pass, 1);
  assert.equal(summary.byStage.twin.pass, 1);
});

test('the next commands are listed once each, in the order of the stages, with the requirements waiting on them', () => {
  const other: Ladder = { ...window, requirement: 'SYS-PW-003' };
  const sim: Ladder = { ...comfort, requirement: 'REQ-ENERGY', stages: [stage('simulation', 'missing'), ...comfort.stages.slice(1)], next: 'arclang verify model.arc --test TC-DAY' };
  const next = nextCommands([window, other, sim, failing, idle]);
  assert.deepEqual(
    next.map((n) => [n.stage, n.command, n.requirements]),
    [
      ['simulation', 'arclang verify model.arc --test TC-DAY', ['REQ-ENERGY']],
      ['pil', 'arclang pil model.arc --test TC-PW-001', ['SYS-PW-001', 'SYS-PW-003']],
    ],
  );
});

test('the configured binary replaces the bare arclang of a command, quoted when it needs it', () => {
  assert.equal(withBinary('arclang sil model.arc --test T', 'arclang'), 'arclang sil model.arc --test T');
  assert.equal(withBinary('arclang sil model.arc --test T', '/opt/arclang/bin/arclang'), '/opt/arclang/bin/arclang sil model.arc --test T');
  assert.equal(withBinary('arclang sil model.arc', '/Users/me/my tools/arclang'), '"/Users/me/my tools/arclang" sil model.arc');
  assert.equal(withBinary('make all', '/opt/arclang'), 'make all');
});

test('a stage tells its cases and their reasons', () => {
  const title = stageTitle(window.stages[2]);
  assert.match(title, /^PIL: ran, but the model.*changed since · zephyr\/litex_vexriscv\n/);
  assert.match(title, /TC-PW-001: stale — its PIL verdict is stale/);
});

test('the snapshot keeps the ladder the compiler sends, and has none from an older compiler', () => {
  const withLadder = parseSnapshot(JSON.stringify({ success: true, errors: [], warnings: [], realization: [window] }));
  assert.equal(withLadder.realization?.length, 1);
  assert.equal(withLadder.realization?.[0].requirement, 'SYS-PW-001');
  const older = parseSnapshot(JSON.stringify({ success: true, errors: [], warnings: [] }));
  assert.deepEqual(older.realization, []);
});

test('a next command computed on a copy runs on the real model file', () => {
  assert.equal(withModel('arclang sil /tmp/arclang-x1/power_window.arc --test TC-PW-001', '/work/pw/power_window.arc'), 'arclang sil /work/pw/power_window.arc --test TC-PW-001');
  assert.equal(withModel('arclang soc --port <serial port> <model> --test TC-1', '/work/pw/model.arc'), 'arclang soc --port <serial port> /work/pw/model.arc --test TC-1');
  assert.equal(withModel('arclang twin /tmp/x/hp.arc --twin TW-1', '/my models/hp.arc'), 'arclang twin "/my models/hp.arc" --twin TW-1');
  assert.equal(withModel('simulation TC-X: the simulation verdict fails; see the case', '/work/m.arc'), 'simulation TC-X: the simulation verdict fails; see the case');
  assert.equal(onModel([window], '/work/pw/model.arc')[0].next, 'arclang pil /work/pw/model.arc --test TC-PW-001');
});
