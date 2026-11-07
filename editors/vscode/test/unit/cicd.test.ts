import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  ageText,
  describeFiles,
  ghMessage,
  ghProblem,
  githubSlug,
  initArgs,
  parseDryRun,
  parseRun,
  parseWritten,
  pollSeconds,
  providerFromRemote,
  PROVIDERS,
  runListArgs,
  safeBranch,
  statusView,
} from '../../src/pure/cicd';

test('the provider comes from the remote, and an unknown host is not guessed', () => {
  assert.equal(providerFromRemote('https://github.com/a/b.git'), 'github');
  assert.equal(providerFromRemote('git@github.com:a/b.git'), 'github');
  assert.equal(providerFromRemote('https://gitlab.com/a/b'), 'gitlab');
  assert.equal(providerFromRemote('git@gitlab.example.org:a/b.git'), 'gitlab');
  assert.equal(providerFromRemote('https://dev.azure.com/o/p/_git/r'), 'azure');
  assert.equal(providerFromRemote('https://o.visualstudio.com/p/_git/r'), 'azure');
  assert.equal(providerFromRemote('https://git.example.org/a/b.git'), null);
  assert.deepEqual(PROVIDERS.map((p) => p.id), ['github', 'gitlab', 'azure', 'jenkins']);
});

test('the slug of a GitHub remote', () => {
  assert.equal(githubSlug('https://github.com/acme/rocket.git'), 'acme/rocket');
  assert.equal(githubSlug('git@github.com:acme/rocket.git'), 'acme/rocket');
  assert.equal(githubSlug('https://gitlab.com/acme/rocket.git'), null);
});

test('arclang ci init is asked with the provider, the standard and the flags', () => {
  assert.deepEqual(initArgs({ provider: 'github', standard: 'iso26262', dryRun: true }), ['ci', 'init', '--provider', 'github', '--standard', 'iso26262', '--dry-run']);
  assert.deepEqual(initArgs({ provider: 'all', standard: 'do178c', force: true }), ['ci', 'init', '--provider', 'all', '--standard', 'do178c', '--force']);
});

const DRY = `--- .github/workflows/arclang.yml
name: ArcLang
on:
  pull_request:

--- .gitlab/arclang.gitlab-ci.yml (exists: would be refused without --force)
stages:
  - test
GitLab: .gitlab-ci.yml exists and is left alone; add to it:
    include:
      - local: .gitlab/arclang.gitlab-ci.yml
Dry run: nothing was written.
`;

test('a dry run is read as files with their state, and the notes apart', () => {
  const { files, notes } = parseDryRun(DRY);
  assert.deepEqual(files.map((f) => [f.path, f.state]), [['.github/workflows/arclang.yml', 'new'], ['.gitlab/arclang.gitlab-ci.yml', 'exists']]);
  assert.equal(files[0].content, 'name: ArcLang\non:\n  pull_request:\n');
  assert.equal(files[1].content, 'stages:\n  - test\n');
  assert.ok(notes.some((n) => n.startsWith('GitLab: ')));
  assert.ok(notes.some((n) => n.includes('include:')));
  assert.equal(describeFiles(files), '+ .github/workflows/arclang.yml\n! .gitlab/arclang.gitlab-ci.yml  (exists)');
});

test('what was written is read from the created and overwrote lines', () => {
  const written = parseWritten('created .github/workflows/arclang.yml\noverwrote Jenkinsfile\ngithub: Commit it\n');
  assert.deepEqual(written, [{ verb: 'created', path: '.github/workflows/arclang.yml' }, { verb: 'overwrote', path: 'Jenkinsfile' }]);
});

const RUN = (over: Record<string, string> = {}) => JSON.stringify([{ status: 'completed', conclusion: 'success', name: 'ArcLang', workflowName: 'ArcLang', displayTitle: 'fix', url: 'https://github.com/a/b/actions/runs/1', headSha: 'abcdef0123456', headBranch: 'feat', event: 'push', createdAt: '2026-10-25T10:00:00Z', ...over }]);

test('the latest run becomes a status; only a success is shown as passing', () => {
  const view = (over: Record<string, string>) => statusView(parseRun(RUN(over)), 'feat');
  assert.equal(view({}).state, 'passed');
  assert.ok(view({}).text.includes('$(check)'));
  assert.equal(view({ conclusion: 'failure' }).state, 'failed');
  assert.equal(view({ conclusion: 'timed_out' }).state, 'failed');
  assert.equal(view({ status: 'in_progress', conclusion: '' }).state, 'running');
  assert.equal(view({ status: 'queued', conclusion: '' }).state, 'running');
  for (const c of ['canceled', 'skipped', 'neutral', 'stale', '']) assert.equal(view({ conclusion: c }).state, 'unknown', c);
  assert.ok(view({ conclusion: 'failure' }).tooltip.includes('abcdef0'));
});

test('no run, or text that is not a run list, is no status rather than a pass', () => {
  assert.equal(parseRun('[]'), null);
  assert.equal(parseRun('not json'), null);
  assert.equal(parseRun('{"a":1}'), null);
  assert.equal(statusView(null, 'main').state, 'none');
});

test('the gh command line asks for one run of the workflow on the branch', () => {
  const args = runListArgs('arclang.yml', 'feat/x');
  assert.deepEqual(args.slice(0, 7), ['run', 'list', '--workflow', 'arclang.yml', '--branch', 'feat/x', '--limit']);
  assert.ok(args.includes('--json'));
  assert.equal(safeBranch('feat/x'), true);
  assert.equal(safeBranch('--web'), false);
  assert.equal(safeBranch('a b'), false);
  assert.equal(safeBranch(''), false);
});

test('a gh that is missing or not logged in is told in words', () => {
  assert.equal(ghProblem({ code: 127, stdout: '', stderr: '' }), 'missing');
  assert.equal(ghProblem({ code: 4, stdout: '', stderr: 'To get started with GitHub CLI, please run:  gh auth login' }), 'not-logged-in');
  assert.equal(ghProblem({ code: 1, stdout: '', stderr: 'none of the git remotes configured for this repository point to a known GitHub host' }), 'no-repo');
  assert.equal(ghProblem({ code: 1, stdout: '', stderr: 'boom' }), 'other');
  assert.match(ghMessage('missing'), /not installed/);
  assert.match(ghMessage('not-logged-in'), /gh auth login/);
  assert.match(ghMessage('not-logged-in'), /stores no token/);
  assert.match(ghMessage('other', 'boom\nmore'), /boom/);
});

test('the polling interval has a floor', () => {
  assert.equal(pollSeconds(undefined), 120);
  assert.equal(pollSeconds(5), 30);
  assert.equal(pollSeconds(300), 300);
  assert.equal(pollSeconds('x'), 120);
});

test('how long ago a run was', () => {
  const now = Date.parse('2026-10-25T12:00:00Z');
  assert.equal(ageText('2026-10-25T11:59:30Z', now), 'just now');
  assert.equal(ageText('2026-10-25T11:30:00Z', now), '30 min ago');
  assert.equal(ageText('2026-10-25T07:00:00Z', now), '5 h ago');
  assert.equal(ageText('2026-10-20T12:00:00Z', now), '5 days ago');
  assert.equal(ageText('nope', now), '');
});
