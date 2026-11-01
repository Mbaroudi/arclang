import assert from 'node:assert/strict';
import { test } from 'node:test';
import { decorations, parseLog, previousOf, shortDate, validRefName } from '../../src/pure/timeline';

const US = '\u001f';
const RS = '\u001e';
const record = (hash: string, subject: string, parents: string, refs: string) => [hash, hash.slice(0, 7), '2026-11-01T10:00:00+01:00', 'Malek', subject, parents, refs].join(US) + RS + '\n';

test('the log of the model reads as commits, oldest first, with snapshots, heads and merges', () => {
  const log =
    record('c'.repeat(40), 'Merge feature', `${'b'.repeat(40)} ${'f'.repeat(40)}`, 'HEAD -> main, tag: v2, origin/main') +
    record('b'.repeat(40), 'Faster controller', 'a'.repeat(40), 'tag: design-review-1') +
    record('a'.repeat(40), 'Initial model', '', '');
  const commits = parseLog(log);
  assert.deepEqual(commits.map((c) => c.subject), ['Initial model', 'Faster controller', 'Merge feature']);
  assert.deepEqual(commits[1].tags, ['design-review-1']);
  assert.deepEqual(commits[2].heads, ['main'], 'a remote branch is not a head here');
  assert.deepEqual(commits[2].tags, ['v2']);
  assert.equal(commits[2].merge, true);
  assert.equal(commits[0].merge, false);
  assert.equal(previousOf(commits, 0), null, 'the first commit has nothing before it');
  assert.equal(previousOf(commits, 2)?.subject, 'Faster controller');
  assert.deepEqual(parseLog('not a log'), []);
});

test('decorations, dates and names', () => {
  assert.deepEqual(decorations('HEAD -> feature/x, tag: v1.0, origin/feature/x, main'), { tags: ['v1.0'], heads: ['feature/x', 'main'] });
  assert.match(shortDate('2026-11-01T10:05:00Z'), /^2026-11-01 \d\d:05$/);
  assert.equal(shortDate('soon'), 'soon');
  assert.ok(validRefName('design-review-2'));
  assert.ok(validRefName('feature/new-braking'));
  for (const bad of ['', '-x', 'a..b', 'a b', 'x/', 'x.lock', 'a//b']) assert.ok(!validRefName(bad), bad);
});
