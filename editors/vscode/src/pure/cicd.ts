// CI/CD in the editor, without VS Code: the providers `arclang ci init`
// writes for, how its dry run and its output are read, and how the latest
// GitHub Actions run (from `gh run list --json`) becomes a status line.
// Pure functions, tested with node:test.

export type ProviderId = 'github' | 'gitlab' | 'azure' | 'jenkins';

export interface Provider {
  id: ProviderId;
  label: string;
  /** The file a pipeline of this provider is in, for the description. */
  file: string;
}

export const PROVIDERS: readonly Provider[] = [
  { id: 'github', label: 'GitHub Actions', file: '.github/workflows/arclang.yml' },
  { id: 'gitlab', label: 'GitLab CI', file: '.gitlab/arclang.gitlab-ci.yml' },
  { id: 'azure', label: 'Azure DevOps', file: 'azure-pipelines.yml' },
  { id: 'jenkins', label: 'Jenkins', file: 'Jenkinsfile' },
];

export const STANDARDS = ['iso26262', 'do178c', 'iec61508'] as const;
export type Standard = (typeof STANDARDS)[number];

/** The provider a git remote URL points at; the compiler tells it the same way. */
export function providerFromRemote(url: string): ProviderId | null {
  const u = url.toLowerCase();
  if (u.includes('github.')) return 'github';
  if (u.includes('gitlab')) return 'gitlab';
  if (u.includes('dev.azure.com') || u.includes('visualstudio.com')) return 'azure';
  return null;
}

/** `owner/repo` of a GitHub remote, or null. */
export function githubSlug(url: string): string | null {
  const match = /github\.[a-z.]+[/:]([^/\s]+)\/([^/\s]+?)(?:\.git)?\/?$/i.exec(url.trim());
  return match ? `${match[1]}/${match[2]}` : null;
}

export interface InitOptions {
  provider: ProviderId | 'all';
  standard: Standard;
  force?: boolean;
  dryRun?: boolean;
}

/** The arguments of `arclang ci init`. One provider per call keeps each preview its own. */
export function initArgs(options: InitOptions): string[] {
  const args = ['ci', 'init', '--provider', options.provider, '--standard', options.standard];
  if (options.dryRun) args.push('--dry-run');
  if (options.force) args.push('--force');
  return args;
}

export type FileState = 'new' | 'exists' | 'overwritten';

export interface PlannedFile {
  path: string;
  content: string;
  /** `exists`: the compiler would refuse it without --force. */
  state: FileState;
}

/**
 * What `arclang ci init --dry-run` prints: `--- <path>[ (exists: ...)]` and the
 * content of each file, then notes, then "Dry run: nothing was written."
 */
export function parseDryRun(stdout: string): { files: PlannedFile[]; notes: string[] } {
  const files: PlannedFile[] = [];
  const notes: string[] = [];
  let current: { path: string; lines: string[]; state: FileState } | null = null;
  const close = () => {
    if (current) files.push({ path: current.path, content: current.lines.join('\n').replace(/\n+$/, '\n'), state: current.state });
    current = null;
  };
  for (const line of stdout.split('\n')) {
    const header = /^--- (\S+?)(?: \(exists: ([^)]*)\))?$/.exec(line);
    if (header) {
      close();
      const exists = header[2];
      current = { path: header[1], lines: [], state: exists === undefined ? 'new' : /overwritten/.test(exists) ? 'overwritten' : 'exists' };
    } else if (/^Dry run:/.test(line)) {
      close();
    } else if (current) {
      current.lines.push(line);
    } else if (line.trim()) {
      notes.push(line);
    }
  }
  close();
  // A note printed after the last file (GitLab's include hint) is read as part of its content: split it back.
  const last = files[files.length - 1];
  if (last) {
    const at = last.content.indexOf('\nGitLab: ');
    if (at >= 0) {
      notes.push(...last.content.slice(at + 1).split('\n').filter((l) => l.trim()));
      last.content = last.content.slice(0, at + 1);
    }
  }
  return { files, notes };
}

/** `created <path>` / `overwrote <path>` lines of a real run. */
export function parseWritten(stdout: string): { verb: 'created' | 'overwrote'; path: string }[] {
  const out: { verb: 'created' | 'overwrote'; path: string }[] = [];
  for (const line of stdout.split('\n')) {
    const match = /^(created|overwrote) (\S.*)$/.exec(line.trim());
    if (match) out.push({ verb: match[1] as 'created' | 'overwrote', path: match[2] });
  }
  return out;
}

export function describeFiles(files: PlannedFile[]): string {
  return files.map((f) => `${f.state === 'new' ? '+' : '!'} ${f.path}${f.state === 'new' ? '' : '  (exists)'}`).join('\n');
}

// ---------------------------------------------------------------- the status

export interface Run {
  status: string;
  conclusion: string;
  name: string;
  workflow: string;
  title: string;
  url: string;
  sha: string;
  branch: string;
  event: string;
  createdAt: string;
}

/** The fields `gh run list --json` is asked for. */
export const RUN_FIELDS = 'status,conclusion,name,workflowName,displayTitle,url,headSha,headBranch,event,createdAt';

export function runListArgs(workflowFile: string, branch: string): string[] {
  return ['run', 'list', '--workflow', workflowFile, '--branch', branch, '--limit', '1', '--json', RUN_FIELDS];
}

/** The latest run of the list, or null when there is none (or the text is not a run list). */
export function parseRun(json: string): Run | null {
  let data: unknown;
  try {
    data = JSON.parse(json);
  } catch {
    return null;
  }
  const first = Array.isArray(data) ? (data[0] as Record<string, unknown> | undefined) : undefined;
  if (!first || typeof first !== 'object') return null;
  const text = (key: string) => (typeof first[key] === 'string' ? (first[key] as string) : '');
  return {
    status: text('status'),
    conclusion: text('conclusion'),
    name: text('name'),
    workflow: text('workflowName'),
    title: text('displayTitle'),
    url: text('url'),
    sha: text('headSha'),
    branch: text('headBranch'),
    event: text('event'),
    createdAt: text('createdAt'),
  };
}

export type StatusState = 'passed' | 'failed' | 'running' | 'none' | 'unknown';

export interface StatusView {
  state: StatusState;
  /** Status-bar text, with its codicon. */
  text: string;
  tooltip: string;
}

const FAILED = new Set(['failure', 'timed_out', 'startup_failure', 'action_required']);

/** A run as a status line. A run that did not succeed is never shown as passing. */
export function statusView(run: Run | null, branch: string): StatusView {
  if (!run) return { state: 'none', text: '$(circle-slash) ArcLang CI', tooltip: `No ArcLang CI run for ${branch} yet.` };
  const where = `${run.workflow || run.name} on ${run.branch || branch} (${run.event}), ${run.sha.slice(0, 7)}`;
  if (run.status !== 'completed') return { state: 'running', text: '$(sync~spin) ArcLang CI', tooltip: `Running: ${where}` };
  if (run.conclusion === 'success') return { state: 'passed', text: '$(check) ArcLang CI', tooltip: `Passed: ${where}` };
  if (FAILED.has(run.conclusion)) return { state: 'failed', text: '$(error) ArcLang CI', tooltip: `Failed (${run.conclusion}): ${where}` };
  // cancelled, skipped, neutral, stale: nothing was proved.
  return { state: 'unknown', text: '$(question) ArcLang CI', tooltip: `${run.conclusion || run.status}: ${where}` };
}

export interface GhOutcome {
  code: number;
  stdout: string;
  stderr: string;
}

export type GhProblem = 'missing' | 'not-logged-in' | 'no-repo' | 'other';

/** Why `gh run list` gave no run list: said in words, never as a stack. */
export function ghProblem(result: GhOutcome): GhProblem {
  if (result.code === 127) return 'missing';
  const text = result.stderr.toLowerCase();
  if (/gh auth login|not logged in|authentication|gh_token/.test(text)) return 'not-logged-in';
  if (/none of the git remotes|could not resolve to a repository|no git remotes|not a git repository/.test(text)) return 'no-repo';
  return 'other';
}

export function ghMessage(problem: GhProblem, detail = ''): string {
  switch (problem) {
    case 'missing':
      return 'The GitHub CLI (gh) is not installed: the status of the pipeline cannot be read. Install it from https://cli.github.com.';
    case 'not-logged-in':
      return 'The GitHub CLI is not logged in: run "gh auth login" in a terminal. ArcLang stores no token.';
    case 'no-repo':
      return 'This repository has no GitHub remote the GitHub CLI can read.';
    default:
      return `The GitHub CLI could not list the runs${detail ? `: ${detail.split('\n')[0]}` : '.'}`;
  }
}

/** `ci` for a branch that cannot be a name for `gh`. */
export function safeBranch(branch: string): boolean {
  return branch.trim() !== '' && !branch.startsWith('-') && !/\s/.test(branch);
}

export const MIN_POLL_SECONDS = 30;

export function pollSeconds(setting: unknown): number {
  const n = typeof setting === 'number' && Number.isFinite(setting) ? Math.floor(setting) : 120;
  return Math.max(MIN_POLL_SECONDS, n);
}

export function ageText(iso: string, now: number): string {
  const then = Date.parse(iso);
  if (!Number.isFinite(then)) return '';
  const seconds = Math.max(0, Math.round((now - then) / 1000));
  if (seconds < 90) return 'just now';
  if (seconds < 5400) return `${Math.round(seconds / 60)} min ago`;
  if (seconds < 172800) return `${Math.round(seconds / 3600)} h ago`;
  return `${Math.round(seconds / 86400)} days ago`;
}
