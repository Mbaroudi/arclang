import assert from 'node:assert/strict';
import { test } from 'node:test';
import { quickFixTitle, reviewArgs, suspectOf } from '../../src/pure/suspect';

test('a suspect diagnostic names its trace', () => {
  const message = "suspect: trace 'PN-BATT' satisfies 'REQ-RANGE': 'REQ-RANGE' changed since the trace was reviewed; check that it still holds, then `arclang trace <model> --review --from PN-BATT --to REQ-RANGE`";
  assert.deepEqual(suspectOf(message), { from: 'PN-BATT', type: 'satisfies', to: 'REQ-RANGE' });
  assert.equal(suspectOf("rule 'R' (blocker): 'PN-1': x"), undefined);
  assert.equal(quickFixTitle({ from: 'PN-BATT', type: 'satisfies', to: 'REQ-RANGE' }), 'Mark the trace PN-BATT satisfies REQ-RANGE as reviewed');
});

test('the review prints the source, for one trace or for all', () => {
  assert.deepEqual(reviewArgs('/m/ev.arc', { from: 'A', type: 'satisfies', to: 'B' }), ['trace', '/m/ev.arc', '--review', '--from', 'A', '--to', 'B', '--print']);
  assert.deepEqual(reviewArgs('/m/ev.arc'), ['trace', '/m/ev.arc', '--review', '--print']);
});
