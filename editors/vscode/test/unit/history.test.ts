import assert from 'node:assert/strict';
import { test } from 'node:test';
import { COALESCE_MS, History } from '../../src/pure/history';

test('model and layout steps are undone and redone in the order they were made', () => {
  const h = new History();
  h.recordLayout('move', { a: 0 }, { a: 1 }, 0);
  h.recordModel('rename', 'v0', 'v1', 5000);
  assert.equal(h.undo()?.kind, 'model');
  assert.equal(h.undo()?.kind, 'layout');
  assert.equal(h.undo(), null);
  assert.equal(h.redo()?.kind, 'layout');
  assert.equal(h.redo()?.kind, 'model');
  assert.equal(h.redo(), null);
});

test('a new step forgets what was undone: redo never brings back an older edit', () => {
  const h = new History();
  h.recordModel('rename', 'v0', 'v1', 0);
  h.undo();
  h.recordLayout('move', 1, 2, 10000);
  assert.equal(h.canRedo, false);
  assert.equal(h.undo()?.label, 'move');
  assert.equal(h.undo(), null, 'the undone rename is not undone twice');
});

test('nudges in a burst are one step; a placement joins the edit it follows', () => {
  const h = new History();
  h.recordLayout('nudge', 0, 1, 0);
  h.recordLayout('nudge', 1, 2, COALESCE_MS - 1);
  h.recordLayout('nudge', 2, 3, 3 * COALESCE_MS);
  assert.deepEqual(h.undo(), { kind: 'layout', label: 'nudge', before: 2, after: 3, at: 3 * COALESCE_MS });
  assert.deepEqual(h.undo(), { kind: 'layout', label: 'nudge', before: 0, after: 2, at: COALESCE_MS - 1 });
  h.recordModel('paste', 'v0', 'v1', 100000);
  h.recordLayout('place', 'l0', 'l1', 101000, true);
  h.recordLayout('place more', 'l1', 'l2', 102000, true);
  const paste = h.undo();
  assert.ok(paste?.kind === 'model' && paste.layout?.before === 'l0' && paste.layout.after === 'l2', JSON.stringify(paste));
  h.recordLayout('none', 'same', 'same', 200000);
  assert.equal(h.canUndo, false, 'a change to the same arrangement is no step');
});

test('the text changed elsewhere: the history is dropped, a document undo is redone by the document', () => {
  const h = new History();
  h.recordModel('rename', 'v0', 'v1', 0);
  h.clear();
  assert.equal(h.canUndo || h.canRedo, false);
  h.recordNative(1);
  assert.equal(h.redo()?.kind, 'native');
});
