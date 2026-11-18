import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Authorship, HOLD_MS } from '../../src/pure/authorship';

test('a change the assistant wrote is recognised once, and only shortly after it said so (issue #224)', () => {
  const who = new Authorship();
  assert.equal(who.takeAssistantChange('a.arc', 0), false, 'nobody said anything: the engineer typed it');
  who.assistantWrites('a.arc', 1000);
  assert.equal(who.takeAssistantChange('b.arc', 1500), false, 'another file');
  assert.equal(who.takeAssistantChange('a.arc', 1500), true);
  assert.equal(who.takeAssistantChange('a.arc', 1600), false, 'the next change is the engineer\'s again');
  who.assistantWrites('a.arc', 1000);
  assert.equal(who.takeAssistantChange('a.arc', 1000 + HOLD_MS + 1), false, 'too late to be that change');
});
