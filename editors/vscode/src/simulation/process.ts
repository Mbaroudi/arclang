// Programs the setup steps run (docker, open): always an argument array and
// never a shell, so nothing the user or a setting contains is interpreted.
import { spawn } from 'node:child_process';
import type * as vscode from 'vscode';

export interface Captured {
  /** null when the program was killed (cancelled). */
  code: number | null;
  output: string;
  /** The program does not exist. */
  missing: boolean;
}

export function capture(command: string, args: string[], options: { cwd?: string; token?: vscode.CancellationToken; onData?: (text: string) => void } = {}): Promise<Captured> {
  return new Promise((resolve) => {
    let output = '';
    let missing = false;
    const child = spawn(command, args, { cwd: options.cwd, shell: false, stdio: ['ignore', 'pipe', 'pipe'] });
    const take = (chunk: Buffer) => {
      const text = chunk.toString();
      output += text;
      options.onData?.(text);
    };
    child.stdout.on('data', take);
    child.stderr.on('data', take);
    child.on('error', (error: NodeJS.ErrnoException) => {
      missing = error.code === 'ENOENT';
      output += `${error.message}\n`;
      resolve({ code: missing ? 127 : 1, output, missing });
    });
    child.on('close', (code) => resolve({ code, output, missing }));
    options.token?.onCancellationRequested(() => child.kill());
  });
}

/** Starts a program and lets it run on its own (Docker Desktop). */
export function launch(command: string, args: string[]): Promise<boolean> {
  return new Promise((resolve) => {
    const child = spawn(command, args, { shell: false, detached: true, stdio: 'ignore' });
    child.on('error', () => resolve(false));
    child.on('spawn', () => {
      child.unref();
      resolve(true);
    });
  });
}
