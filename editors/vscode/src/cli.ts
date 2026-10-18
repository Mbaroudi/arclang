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

/** Runs `arclang <args>`; a missing binary is exit code 127, never a throw. */
export function run(args: string[], options: RunOptions = {}): Promise<RunResult> {
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

/** Writes `text` in a fresh temporary directory and runs `use` on its path. */
export async function withTempFile<T>(text: string, name: string, use: (file: string) => Promise<T>): Promise<T> {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-'));
  const file = path.join(dir, name);
  try {
    fs.writeFileSync(file, text);
    return await use(file);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
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
