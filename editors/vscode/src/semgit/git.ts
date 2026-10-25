// The git the semantic-git commands need: where the repository is, which
// refs there are to compare with, the committed text of a file. The compiler
// does the semantic work; this only asks git for names and text.
import { execFile } from 'node:child_process';
import * as path from 'node:path';
import { GitRef, parseRefs } from '../pure/semgit';

export interface GitResult {
  code: number;
  stdout: string;
  stderr: string;
}

export function git(args: string[], cwd: string): Promise<GitResult> {
  return new Promise((resolve) => {
    execFile('git', args, { cwd, maxBuffer: 64 * 1024 * 1024 }, (error, stdout, stderr) => {
      resolve({ code: error && typeof error.code === 'number' ? error.code : error ? 1 : 0, stdout: String(stdout), stderr: String(stderr) });
    });
  });
}

/** The top of the repository a directory is in, or null. */
export async function repoTop(dir: string): Promise<string | null> {
  const out = await git(['rev-parse', '--show-toplevel'], dir);
  return out.code === 0 ? out.stdout.trim() : null;
}

export async function currentBranch(dir: string): Promise<string | null> {
  const out = await git(['rev-parse', '--abbrev-ref', 'HEAD'], dir);
  return out.code === 0 ? out.stdout.trim() : null;
}

/** Everything a model can be compared with: HEAD, branches, baselines, tags, remote branches. */
export async function listRefs(dir: string): Promise<GitRef[]> {
  const out = await git(['for-each-ref', '--format=%(refname)%09%(objectname:short)%09%(subject)', 'refs/heads', 'refs/tags', 'refs/remotes'], dir);
  if (out.code !== 0) return [];
  return parseRefs(out.stdout, await currentBranch(dir));
}

/** The path of a file inside its repository, with `/` separators. */
export function relativeTo(top: string, file: string): string {
  return path.relative(top, file).split(path.sep).join('/');
}

/** The text of a file at a revision, or null when the revision has none. */
export async function showAt(top: string, revision: string, file: string): Promise<string | null> {
  const out = await git(['show', `${revision}:${relativeTo(top, file)}`], top);
  return out.code === 0 ? out.stdout : null;
}

/** A revision that cannot be taken for an option or a path. */
export function safeRevision(revision: string): boolean {
  return revision.trim() !== '' && !revision.startsWith('-') && !revision.includes(':') && !/\s/.test(revision);
}
