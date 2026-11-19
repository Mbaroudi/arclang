// The extension's public API: what it makes of the compiler's answers and of a burst of changes.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { debounced, parseWorkbenchAnswer } from '../../src/pure/api';

test('a workbench document is returned as it is', () => {
  const answer = parseWorkbenchAnswer(0, '{"success": true, "elements": [{"id": "A"}]}', '');
  assert.equal(answer.ok, true);
  if (answer.ok) assert.deepEqual(answer.workbench.elements, [{ id: 'A' }]);
});

test('a model that does not compile gives its errors, not a document', () => {
  const answer = parseWorkbenchAnswer(1, '{"success": false, "errors": ["line 3: unknown kind", {"line": 4}]}', '');
  assert.deepEqual(answer, { ok: false, errors: ['line 3: unknown kind', '{"line":4}'] });
  assert.deepEqual(parseWorkbenchAnswer(1, '{"success": false}', ''), { ok: false, errors: ['the model does not compile'] });
});

test('a compiler that is missing, untrusted or crashing is an answer with its text, never a throw', () => {
  assert.deepEqual(parseWorkbenchAnswer(127, '', 'arclang: command not found\n'), { ok: false, errors: ['arclang: command not found'] });
  assert.deepEqual(parseWorkbenchAnswer(126, '', ''), { ok: false, errors: ['the compiler answered nothing readable (exit 126)'] });
  assert.equal(parseWorkbenchAnswer(0, '[1, 2]', '').ok, false);
  assert.equal(parseWorkbenchAnswer(0, 'null', '').ok, false);
});

test('a burst of changes of one file is one call, and two files are two', () => {
  const pending: Array<{ run: () => void; at: number; cleared: boolean }> = [];
  const timers = {
    set: ((run: () => void, at: number) => { pending.push({ run, at, cleared: false }); return pending.length - 1; }) as unknown as typeof setTimeout,
    clear: ((id: number) => { pending[id].cleared = true; }) as unknown as typeof clearTimeout,
  };
  const calls: string[] = [];
  const burst = debounced<string>(300, (value) => calls.push(value), timers);
  burst.push('a', 'a.arc');
  burst.push('a', 'a.arc');
  burst.push('a', 'a.arc');
  burst.push('b', 'b.arc');
  for (const timer of pending) if (!timer.cleared) { timer.cleared = true; timer.run(); }
  assert.deepEqual([...calls].sort(), ['a.arc', 'b.arc']);
  burst.push('c', 'c.arc');
  burst.dispose();
  for (const timer of pending) if (!timer.cleared) { timer.cleared = true; timer.run(); }
  assert.deepEqual([...calls].sort(), ['a.arc', 'b.arc'], 'a disposed debounce calls nothing');
});
