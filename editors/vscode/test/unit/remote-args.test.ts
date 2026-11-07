import assert from 'node:assert/strict';
import { test } from 'node:test';
import { outputTarget, planArguments, restorePaths } from '../../src/pure/remote-args';

test('the paths of a command become the job\'s: the folder, a temporary file, an output', () => {
  const plan = planArguments(['safety', '/workspace/ev/m.arc', '--standard', 'iso26262', '--report', '--output', '/tmp/arc-1/m.safety.md'], '/workspace');
  assert.deepEqual(plan.args, ['safety', 'w/ev/m.arc', '--standard', 'iso26262', '--report', '--output', 'o/1/m.safety.md']);
  assert.deepEqual(plan.outputs, [{ slot: 'o/1/', target: '/tmp/arc-1' }]);
  assert.deepEqual(plan.reads, []);
  const table = planArguments(['table', '/tmp/arc-2/model.arc', '-f', 'json', '--name=x'], '/workspace');
  assert.deepEqual(table.args, ['table', 't/1/model.arc', '-f', 'json', '--name=x']);
  assert.deepEqual(table.reads, [{ jobPath: 't/1/model.arc', source: '/tmp/arc-2/model.arc' }]);
  const inline = planArguments(['export', '/workspace/m.arc', '-f', 'modelica', '--output=/workspace/modelica'], '/workspace');
  assert.deepEqual(inline.args, ['export', 'w/m.arc', '-f', 'modelica', '--output=o/1/modelica']);
});

test('a folder smaller than the workspace maps to w/ too', () => {
  assert.deepEqual(planArguments(['gate', '/workspace/big/ev/m.arc'], '/workspace/big/ev').args, ['gate', 'w/m.arc']);
});

test('what the job wrote and printed comes back to the extension\'s paths', () => {
  const plan = planArguments(['export', '/workspace/ev/m.arc', '-f', 'modelica', '--split', '-o', '/workspace/ev/modelica'], '/workspace');
  assert.equal(outputTarget('o/1/modelica/EV/package.mo', plan.outputs), '/workspace/ev/modelica/EV/package.mo');
  assert.equal(outputTarget('w/ev/.arclang/x.json', plan.outputs), null);
  assert.equal(restorePaths('Output: o/1/modelica/EV\nmodel w/ev/m.arc', plan.back), 'Output: /workspace/ev/modelica/EV\nmodel /workspace/ev/m.arc');
});
