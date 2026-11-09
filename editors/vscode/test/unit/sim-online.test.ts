import assert from 'node:assert/strict';
import { test } from 'node:test';
import { ONLINE_LIMITS, leftText, pickFiles, sendablePath, walkable, sentAsBytes, cosimCases, cosimLog, cosimText, executes } from '../../src/pure/sim-online';

test('only what a simulation reads is sent, never a dot file nor the compiler cache', () => {
  for (const ok of ['ev.arc', 'lib/Plant.mo', 'data/drive cycle.csv', 'run.mos', 'data/house.parquet']) assert.ok(sendablePath(ok), ok);
  for (const bad of ['.arclang/sim-cache/TC.json', 'notes/.secret.mo', 'image.png', 'node_modules/x/y.mo', 'a/../b.arc', 'x.arc.', '']) assert.equal(sendablePath(bad), false, bad);
  assert.ok(walkable('lib'));
  assert.equal(walkable('.git'), false);
  assert.equal(walkable('node_modules'), false);
});

test('the model goes first, the others by path, within the limits', () => {
  const picked = pickFiles('ev.arc', [
    { path: 'z.mo', size: 10 },
    { path: 'ev.arc', size: 100 },
    { path: 'a/Plant.mo', size: 10 },
    { path: 'big.csv', size: ONLINE_LIMITS.bytes },
  ]);
  assert.deepEqual(picked.take, ['ev.arc', 'a/Plant.mo', 'z.mo']);
  assert.equal(picked.left, 1);
  assert.match(pickFiles('mon modèle.arc', [{ path: 'mon modèle.arc', size: 1 }]).error ?? '', /cannot be sent/);
  assert.match(pickFiles('ev.arc', [{ path: 'ev.arc', size: ONLINE_LIMITS.bytes + 1 }]).error ?? '', /larger/);
});

test('the runs left read naturally', () => {
  assert.equal(leftText(19), ' · 19 online runs left today');
  assert.equal(leftText(1), ' · 1 online run left today');
  assert.equal(leftText(undefined), '');
});

test('a twin\'s Parquet measurements and a .mat result go as bytes, the rest as text', () => {
  for (const bytes of ['data/house.parquet', 'res.MAT']) assert.ok(sentAsBytes(bytes), bytes);
  for (const text of ['ev.arc', 'lib/Plant.mo', 'data/house.csv']) assert.equal(sentAsBytes(text), false, text);
});

test('a model with test cases that execute: is co-simulated online; their verdicts are said and logged', () => {
  assert.equal(executes('test_case "T" {\n  execute: { machine: "M" inputs: [] }\n}'), true);
  assert.equal(executes('// execute: { }\ntest_case "T" { simulate: { model: "P" } }'), false);
  const cases = cosimCases([
    { test_case: 'TC-1', name: 'Release to hold', verifies: ['REQ-1'], verdict: 'pass', reason: '' },
    { test_case: 'TC-2', name: 'Creep', verifies: [], verdict: 'fail', reason: 'speed 4 km/h > 3 km/h' },
    { test_case: 'TC-3', name: 'Other', verdict: 'skipped' },
    { name: 'no id' },
  ]);
  assert.deepEqual(cases.map((c) => c.test_case), ['TC-1', 'TC-2']);
  assert.equal(cosimText(cases), '2 co-simulations (OMSimulator): 1 pass, 1 fail.');
  assert.equal(cosimText(cases.slice(0, 1)), '1 co-simulation (OMSimulator) passes.');
  assert.equal(cosimText([]), '');
  assert.match(cosimLog(cases), /FAIL  TC-2  Creep\n\s+speed 4 km\/h > 3 km\/h/);
  assert.deepEqual(cosimCases(null), []);
});
