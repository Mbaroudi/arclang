// The one place that runs the `arclang` binary.
import { execFile } from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';

export interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

export function binary(): string {
  return vscode.workspace.getConfiguration('arclang').get<string>('binary')?.trim() || 'arclang';
}

export interface RunOptions {
  cwd?: string;
  token?: vscode.CancellationToken;
}

export const UNTRUSTED_MESSAGE = 'ArcLang does not run the compiler on the files of an untrusted workspace. Trust the workspace (Workspaces: Manage Workspace Trust) to get diagnostics, commands, diagrams and tools.';

/** Whether the compiler may be run on the workspace's files. */
export function workspaceTrusted(): boolean {
  return vscode.workspace.isTrusted;
}

/**
 * Where the compiler runs: the native `arclang` binary (desktop), or the
 * WebAssembly build in a web worker (the browser). Every compiler call of
 * the extension goes through one of these via `run`.
 */
export interface Backend {
  readonly name: 'native' | 'wasm';
  run(args: string[], options: RunOptions): Promise<RunResult>;
}

let backend: Backend | null = null; // web hook

/** Replaces the native backend (the web entry installs the WebAssembly one). */
export function setBackend(next: Backend): void { // web hook
  backend = next;
}

/** The backend in use: the native binary unless the web entry set another. */
export function currentBackend(): Backend { // web hook
  return backend ?? nativeBackend;
}

/**
 * Runs `arclang <args>`, never through a shell; a missing binary is exit
 * code 127 and an untrusted workspace exit code 126, never a throw.
 */
export function run(args: string[], options: RunOptions = {}): Promise<RunResult> {
  if (!workspaceTrusted()) return Promise.resolve({ code: 126, stdout: '', stderr: UNTRUSTED_MESSAGE });
  return currentBackend().run(args, options);
}

function runNative(args: string[], options: RunOptions): Promise<RunResult> {
  return new Promise((resolve) => {
    const child = execFile(
      binary(),
      args,
      { cwd: options.cwd, maxBuffer: 256 * 1024 * 1024, env: process.env },
      (error, stdout, stderr) => {
        if (error && (error as NodeJS.ErrnoException).code === 'ENOENT') {
          resolve({ code: 127, stdout: '', stderr: `${binary()}: command not found` });
          return;
        }
        const code = error && typeof error.code === 'number' ? error.code : error ? 1 : 0;
        resolve({ code, stdout: String(stdout), stderr: String(stderr) });
      },
    );
    options.token?.onCancellationRequested(() => child.kill());
  });
}

const nativeBackend: Backend = { name: 'native', run: runNative };

/**
 * Writes `text` in a fresh private temporary directory (mode 0700, unguessable
 * name) and runs `use` on its path and its directory; the directory is removed
 * afterwards. Compiler runs on such a file take the directory as working
 * directory, so nothing of the workspace is read by relative paths.
 */
export async function withTempFile<T>(text: string, name: string, use: (file: string, dir: string) => Promise<T>): Promise<T> {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-'));
  const file = path.join(dir, name);
  try {
    fs.writeFileSync(file, text, { mode: 0o600 });
    return await use(file, dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

/** A fresh private temporary directory (mode 0700); the caller removes it. */
export function tempDir(): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-'));
}

let channel: vscode.OutputChannel | null = null;

export function output(): vscode.OutputChannel {
  if (!channel) channel = vscode.window.createOutputChannel('ArcLang');
  return channel;
}

/** Shows a command and its result in the ArcLang output channel. */
export function report(title: string, result: RunResult, reveal = true): void {
  const out = output();
  out.appendLine(`$ ${title}`);
  if (result.stdout) out.append(result.stdout.endsWith('\n') ? result.stdout : result.stdout + '\n');
  if (result.stderr) out.append(result.stderr.endsWith('\n') ? result.stderr : result.stderr + '\n');
  out.appendLine(result.code === 0 ? '(exit 0)' : `(exit ${result.code})`);
  out.appendLine('');
  if (reveal) out.show(true);
}

/** The message of a failed run: stderr, else stdout, else a generic line. */
export function failureText(result: RunResult): string {
  return (result.stderr || result.stdout || 'arclang failed').trim();
}
