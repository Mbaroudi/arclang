import assert from 'node:assert/strict';
import { test } from 'node:test';
import { git, setGitGuard } from '../../src/semgit/git';

test('git is not run where it is refused, and the refusal is what the caller reads', async () => {
  setGitGuard(() => 'not in an untrusted workspace');
  try {
    const out = await git(['--version'], '.');
    assert.equal(out.code, 1);
    assert.equal(out.stdout, '');
    assert.equal(out.stderr, 'not in an untrusted workspace');
  } finally {
    setGitGuard(() => null);
  }
});

test('with no refusal, git runs as before', async () => {
  setGitGuard(() => null);
  const out = await git(['--version'], '.');
  assert.equal(out.code, 0);
  assert.match(out.stdout, /^git version/);
});
