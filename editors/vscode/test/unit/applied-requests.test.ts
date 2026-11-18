import test from 'node:test';
import assert from 'node:assert/strict';
import { APPLIED_LIMITS, withApplied, withoutApplied } from '../../src/pure/applied-requests';

test('the requests that changed the model are kept with their answer; a refinement joins its change (issue #228)', () => {
  let list = withApplied([], 's1', 'Add a radar', 'Added component "Radar".');
  list = withApplied(list, 's2', 'Rename it', 'Renamed.');
  list = withApplied(list, 's1', 'Put it at the front', 'Moved to the front.');
  assert.deepEqual(list, [
    { session: 's2', question: 'Rename it', answer: 'Renamed.' },
    { session: 's1', question: 'Add a radar\nPut it at the front', answer: 'Moved to the front.' },
  ]);
  assert.deepEqual(withApplied(list, 's3', '  ', 'x'), list);
});

test('a change that is undone is forgotten, and the list stays bounded', () => {
  const list = withApplied(withApplied([], 's1', 'a', 'b'), 's2', 'c', 'd');
  assert.deepEqual(withoutApplied(list, 's1').map((r) => r.session), ['s2']);
  let many: ReturnType<typeof withApplied> = [];
  for (let i = 0; i < 50; i++) many = withApplied(many, `s${i}`, 'q'.repeat(5000), 'a'.repeat(5000));
  assert.equal(many.length, APPLIED_LIMITS.requests);
  assert.equal(many[0].session, 's30');
  assert.equal(many[0].question.length, APPLIED_LIMITS.question + 1);
  assert.equal(many[0].answer.length, APPLIED_LIMITS.answer + 1);
});
