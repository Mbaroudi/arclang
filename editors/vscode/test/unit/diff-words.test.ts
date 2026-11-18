import assert from 'node:assert/strict';
import { test } from 'node:test';
import { changedPart, refineDiff } from '../../src/pure/diff-words';

const marked = (text: string, range?: [number, number]) => (range ? text.slice(range[0], range[1]) : null);

test('the part of a line that changed is the words between what stayed', () => {
  const before = '    description: "Controls the brake"';
  const after = '    description: "Controls the brake pressure"';
  const part = changedPart(before, after)!;
  assert.equal(before.slice(...part.before), '');
  assert.equal(after.slice(...part.after), ' pressure');
  const word = changedPart('  asil: "ASIL_B"', '  asil: "ASIL_D"')!;
  assert.equal('  asil: "ASIL_B"'.slice(...word.before), 'ASIL_B');
  assert.equal('  asil: "ASIL_D"'.slice(...word.after), 'ASIL_D');
  assert.equal(changedPart('same', 'same'), null);
  assert.equal(changedPart('  component "A" {}', '  function "B" { id: "x" }') === null, false);
  assert.equal(changedPart('alpha', 'omega beta'), null, 'nothing in common: nothing to mark');
});

test('in a diff, a removed line and the line added in its place say what changed; other lines are left whole', () => {
  const lines = refineDiff(['@@ -3,4 +3,5 @@', '   component "Brake" {', '-    asil: "ASIL_B"', '-    description: "Old"', '+    asil: "ASIL_D"', '+    description: "Old and new"', '   }', '-  removed alone', '+  added one', '+  added two']);
  assert.deepEqual(lines.map((l) => l.kind), ['hunk', 'ctx', 'del', 'del', 'add', 'add', 'ctx', 'del', 'add', 'add']);
  assert.equal(marked(lines[2].text, lines[2].changed), 'ASIL_B');
  assert.equal(marked(lines[4].text, lines[4].changed), 'ASIL_D');
  assert.equal(marked(lines[5].text, lines[5].changed), ' and new');
  assert.equal(lines[3].changed, undefined, 'text only added: nothing to mark in the line removed');
  assert.equal(lines[7].changed, undefined, 'one line replaced by two: no pair');
  assert.equal(lines[8].changed, undefined);
});
