import assert from 'node:assert/strict';
import { test } from 'node:test';
import { KEPT_LIMITS, keepTurn, readKept } from '../../src/pure/chat-history';

test('the conversation is kept turn by turn, the oldest dropped, long answers cut', () => {
  let kept = keepTurn([], { who: 'you', meta: 'You', text: 'why?' });
  kept = keepTurn(kept, { who: 'ai', meta: 'hosted · lock.arc', text: 'x'.repeat(KEPT_LIMITS.characters + 50) });
  kept = keepTurn(kept, { who: 'ai', meta: 'hosted', text: '   ' });
  assert.equal(kept.length, 2);
  assert.equal(kept[1].text.length, KEPT_LIMITS.characters + 1);
  for (let i = 0; i < KEPT_LIMITS.turns + 5; i++) kept = keepTurn(kept, { who: 'you', meta: 'You', text: `q${i}` });
  assert.equal(kept.length, KEPT_LIMITS.turns);
  assert.equal(kept[kept.length - 1].text, `q${KEPT_LIMITS.turns + 4}`);
});

test('a stored state is read back as turns, whatever else it holds left out', () => {
  assert.deepEqual(readKept(null), []);
  assert.deepEqual(readKept({ turns: 'no' }), []);
  const read = readKept({ turns: [{ who: 'you', meta: 'You', text: 'hello' }, { who: 'them', meta: 'x', text: 'y' }, { who: 'ai', meta: 'hosted', text: '' }, null, { who: 'ai', meta: 'hosted', text: 'answer' }] });
  assert.deepEqual(read, [{ who: 'you', meta: 'You', text: 'hello' }, { who: 'ai', meta: 'hosted', text: 'answer' }]);
});
