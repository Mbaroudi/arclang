// The compiler online, for the browser build: a command the WebAssembly
// compiler does not have (the gate, tables, impact, safety, diffs, exports
// and imports...) runs on the server when the page hosting VS Code offers it
// (arclang.fr, signed in: the command `arclangWeb.compiler.run`).
//
// The job carries the workspace (or the model's folder when the workspace is
// large) under `w/`, the temporary files the command names under `t/<n>/`,
// and each output it asks for under `o/<n>/` (pure/remote-args.ts). The
// command runs at the job's root, and what comes
// back is put where the extension expects it: an output in the in-memory
// file system or in the workspace, the job's paths in the printed text
// turned back into the extension's.
import * as vscode from 'vscode';
import type { RunOptions, RunResult } from '../cli';
import { insidePath as inside, outputTarget, planArguments, restorePaths } from '../pure/remote-args';
import { onlineRun } from '../simulation/online-results';
import * as vfs from './vfs';

const HOST_COMMAND = 'arclangWeb.compiler.run';
const ROOT = '/workspace';
/** What the server runs (it checks again, and refuses what it must). */
const REMOTE = new Set(['gate', 'table', 'safety', 'impact', 'diff', 'diagram', 'export', 'import', 'document', 'trace', 'set', 'rename', 'verify', 'debug', 'history', 'restore', 'baseline', 'workbench']);
/** Commands that read the history: the site adds the user's repository, whose top is the workspace. */
const readsHistory = (args: string[]): boolean => {
  const [command, ...rest] = args;
  const flags = rest.map((a) => a.split('=')[0]);
  return ['history', 'restore', 'baseline'].includes(command) || (command === 'workbench' && flags.includes('--rev')) || (command === 'impact' && flags.includes('--since')) || (command === 'diff' && (flags.includes('--from') || flags.includes('--to')));
};
/** An empty file that makes an output's folder exist in the job (the compiler does not create it); never sent back. */
const OUTPUT_FOLDER = '__arclang_output_folder';
const SENT = /\.(arc|mo|mos|csv|tsv|json|txt|md|reqif|reqifz|xml|capella|aird|afm|melodymodeller|sysml|kerml|docx|xlsx|html|plantuml|puml|mmd|mermaid)$/i;
const SEGMENT = /^[A-Za-z0-9_][A-Za-z0-9._ -]{0,99}$/;
const LIMITS = { files: 400, bytes: 8_000_000 };

let available = false;
/** Whether the visitor is signed in (the page says so); a page that does not say is taken as signed in, as before. */
let signedIn = true;

/** Asks whether the page offers the compiler online (again until it does: it registers after the account is known), and whether the visitor is signed in. */
export async function probeRemote(): Promise<boolean> {
  const commands = await vscode.commands.getCommands(true);
  available ||= commands.includes(HOST_COMMAND);
  if (commands.includes('arclangWeb.account.signedIn')) {
    signedIn = await Promise.resolve(vscode.commands.executeCommand<boolean>('arclangWeb.account.signedIn')).then((v) => v === true, () => false);
  }
  return available;
}

/** Whether a subcommand can run online here: offered by the page, and the visitor signed in (else nothing is sent, and no notice pops up for a command run in the background). */
export function runsRemotely(command: string | undefined): boolean {
  return available && signedIn && !!command && REMOTE.has(command);
}

/** Whether a subcommand would run online for a signed-in visitor (what to tell one who is not). */
export function runsOnlineSignedIn(command: string | undefined): boolean {
  return available && !signedIn && !!command && REMOTE.has(command);
}

const toBase64 = (bytes: Uint8Array): string => {
  let text = '';
  for (let i = 0; i < bytes.length; i += 0x8000) text += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(text);
};
const fromBase64 = (text: string): Uint8Array => Uint8Array.from(atob(text), (c) => c.charCodeAt(0));
const posixDir = (p: string) => p.slice(0, Math.max(0, p.lastIndexOf('/'))) || '/';

/** The files of a folder of the workspace that a command may read, within the limits; null when too large. */
async function folderFiles(dir: string): Promise<Map<string, Uint8Array> | null> {
  const out = new Map<string, Uint8Array>();
  let bytes = 0;
  const walk = async (path: string): Promise<boolean> => {
    let entries: [string, vscode.FileType][];
    try {
      entries = await vscode.workspace.fs.readDirectory(vscode.Uri.file(path));
    } catch {
      return true;
    }
    for (const [name, type] of entries) {
      if (!SEGMENT.test(name) || name === 'node_modules') continue;
      const child = `${path}/${name}`;
      if (type & vscode.FileType.Directory) {
        if (!(await walk(child))) return false;
      } else if (SENT.test(name)) {
        const data = await vscode.workspace.fs.readFile(vscode.Uri.file(child));
        bytes += data.length;
        if (out.size >= LIMITS.files || bytes > LIMITS.bytes) return false;
        out.set(child, data);
      }
    }
    return true;
  };
  return (await walk(dir)) ? out : null;
}

async function readAny(path: string): Promise<Uint8Array | null> {
  if (vfs.isFile(path)) {
    const data = vfs.read(path);
    return typeof data === 'string' ? new TextEncoder().encode(data) : data;
  }
  try {
    return await vscode.workspace.fs.readFile(vscode.Uri.file(path));
  } catch {
    return null;
  }
}

async function writeBack(path: string, data: Uint8Array): Promise<void> {
  if (inside(path, ROOT)) await vscode.workspace.fs.writeFile(vscode.Uri.file(path), data);
  else vfs.write(path, data);
}

/** Runs `arclang args` online; the result as the native compiler would have given it. */
export async function runRemote(args: string[], options: RunOptions): Promise<RunResult> {
  const failed = (stderr: string, code = 1): RunResult => ({ code, stdout: '', stderr });
  // The workspace, or the folder the command works in when the workspace is too large.
  const firstModel = args.slice(1).find((a) => a.startsWith(`${ROOT}/`) && a.endsWith('.arc'));
  let base = ROOT;
  let files = await folderFiles(ROOT);
  if (!files) {
    if (readsHistory(args)) return failed(`ArcLang: the workspace is larger than the compiler online takes with its history (${LIMITS.files} files, ${LIMITS.bytes / 1e6} MB).`, 78);
    base = options.cwd && inside(options.cwd, ROOT) ? options.cwd : firstModel ? posixDir(firstModel) : ROOT;
    files = await folderFiles(base);
    if (!files) return failed(`ArcLang: the folder ${base} is larger than the compiler online takes (${LIMITS.files} files, ${LIMITS.bytes / 1e6} MB).`, 78);
  }
  const job: Record<string, { base64: string }> = {};
  for (const [path, data] of files) job[`w${path.slice(base.length)}`] = { base64: toBase64(data) };
  // The verdicts of the last online simulation of a model, as the desktop keeps them beside it.
  for (const path of files.keys()) {
    const run = path.endsWith('.arc') ? onlineRun(path) : undefined;
    for (const [name, text] of Object.entries(run?.cache ?? {})) job[`w${posixDir(path).slice(base.length)}/.arclang/sim-cache/${name}`] = { base64: toBase64(new TextEncoder().encode(text)) };
  }

  // The arguments: each path turned into the job's (see pure/remote-args.ts).
  const plan = planArguments(args, base);
  for (const { slot } of plan.outputs) job[`${slot}${OUTPUT_FOLDER}`] = { base64: '' };
  for (const { jobPath, source } of plan.reads) {
    const data = await readAny(source);
    if (data) job[jobPath] = { base64: toBase64(data) };
  }

  const reply = await vscode.commands.executeCommand<{ ok: true; answer: Record<string, unknown> } | { ok: false; error: string } | null>(HOST_COMMAND, { cwd: '', args: plan.args, files: job });
  if (!reply) return failed(`ArcLang: 'arclang ${args[0]}' runs online for signed-in visitors. Sign in, then run it again.`, 77);
  if (!reply.ok) return failed(`ArcLang: ${reply.error}`, 78);
  const answer = reply.answer as { code?: number | null; stdout?: string; stderr?: string; files?: Record<string, string>; timedOut?: boolean; truncated?: boolean };

  // The outputs, where the extension asked for them; the job's paths in the text turned back.
  for (const [rel, data] of Object.entries(answer.files ?? {})) {
    if (rel.endsWith(`/${OUTPUT_FOLDER}`)) continue;
    // An output asked for, or a file the command wrote in the folder sent (a baseline's), never a hidden one.
    const target = outputTarget(rel, plan.outputs) ?? (rel.startsWith('w/') && !rel.split('/').some((s) => s.startsWith('.')) ? `${base}${rel.slice(1)}` : null);
    if (target) await writeBack(target, fromBase64(data));
  }
  const restore = (text: string) => restorePaths(text, plan.back);
  const note = answer.timedOut ? '\nArcLang: the command took longer than the compiler online allows.' : answer.truncated ? '\nArcLang: some files written by the command were too large to be sent back.' : '';
  return { code: typeof answer.code === 'number' ? answer.code : 70, stdout: restore(answer.stdout ?? ''), stderr: restore(answer.stderr ?? '') + note };
}
