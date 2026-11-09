// The WebAssembly backend: answers the `arclang <subcommand>` calls of the
// extension with the compiler's wasm build, reading and writing the files
// those calls name on the in-memory file system (./vfs.ts). A subcommand the
// build does not carry yet answers with a clear message, never a crash.
import * as vscode from 'vscode';
import type { Backend, RunOptions, RunResult } from '../cli';
import * as vfs from './vfs';
import type { Wasm } from './wasm';

/** A command the server runs for signed-in visitors. */
export const NEEDS_SIGN_IN = (what: string): string =>
  `${what} runs online for signed-in visitors: sign in (free, a link by e-mail, no password) with "Sign in to keep your work" in the status bar, or use the desktop extension.`;

export const NOT_AVAILABLE = (what: string): string =>
  `${what} is not available in the browser yet: the web playground runs the compiler in WebAssembly, which carries the model, edits, checks, SysML v2 and the viewer for now. Use the desktop extension for this command.`;

import { probeRemote, runRemote, runsOnlineSignedIn, runsRemotely } from './remote';

const done = (stdout: string, code = 0): RunResult => ({ code, stdout, stderr: '' });
const failed = (stderr: string, code = 1): RunResult => ({ code, stdout: '', stderr });

/** The value following `flag` in `args` (`--format json` or `--format=json`). */
function option(args: string[], ...flags: string[]): string | undefined {
  for (const [index, arg] of args.entries()) {
    for (const flag of flags) {
      if (arg === flag) return args[index + 1];
      if (arg.startsWith(`${flag}=`)) return arg.slice(flag.length + 1);
    }
  }
  return undefined;
}

/** The positional arguments: what is neither a flag nor a flag's value. */
function positionals(args: string[], withValue: string[]): string[] {
  const out: string[] = [];
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--') continue;
    if (withValue.includes(arg)) {
      i++;
      continue;
    }
    if (!arg.startsWith('-')) out.push(arg);
  }
  return out;
}

/** An output: in the workspace when its path is there (the user chose it), else a temporary file of the extension. */
async function writeOut(path: string, text: string): Promise<void> {
  if (path === '/workspace' || path.startsWith('/workspace/')) await vscode.workspace.fs.writeFile(vscode.Uri.file(path), new TextEncoder().encode(text));
  else vfs.write(path, text);
}

/** A temporary file of the extension, else a file of the workspace. */
async function readModel(path: string | undefined): Promise<{ ok: true; text: string } | { ok: false; result: RunResult }> {
  if (!path) return { ok: false, result: failed('(no file)', 2) };
  if (vfs.isFile(path)) return { ok: true, text: vfs.readText(path) };
  try {
    return { ok: true, text: new TextDecoder().decode(await vscode.workspace.fs.readFile(vscode.Uri.file(path))) };
  } catch {
    return { ok: false, result: failed(`${path}: no such file`, 2) };
  }
}

const SEGMENT = /^[A-Za-z0-9_][A-Za-z0-9._ -]{0,99}$/;
const VERDICT_LIMITS = { files: 200, bytes: 4_000_000, depth: 6 };

/**
 * The files of a model's folder that its verdicts depend on, by path relative
 * to it: the verdicts the runs kept (`.arclang/sim-cache/*.json`) and the
 * Modelica sources their keys hash; null when the folder keeps no verdict
 * (the usual case: nothing is read then).
 */
async function verdictFiles(dir: string): Promise<Record<string, string> | null> {
  const read = async (path: string): Promise<[string, vscode.FileType][]> => {
    try {
      return await vscode.workspace.fs.readDirectory(vscode.Uri.file(path));
    } catch {
      return [];
    }
  };
  const cacheDir = `${dir}/.arclang/sim-cache`;
  const verdicts = (await read(cacheDir)).filter(([name, type]) => type & vscode.FileType.File && SEGMENT.test(name) && name.endsWith('.json'));
  if (!verdicts.length) return null;
  const out: Record<string, string> = {};
  let bytes = 0;
  const take = async (path: string, rel: string): Promise<boolean> => {
    const data = await vscode.workspace.fs.readFile(vscode.Uri.file(path));
    bytes += data.length;
    if (Object.keys(out).length >= VERDICT_LIMITS.files || bytes > VERDICT_LIMITS.bytes) return false;
    out[rel] = new TextDecoder().decode(data);
    return true;
  };
  for (const [name] of verdicts) if (!(await take(`${cacheDir}/${name}`, `.arclang/sim-cache/${name}`))) return out;
  const sources = async (path: string, rel: string, depth: number): Promise<boolean> => {
    if (depth > VERDICT_LIMITS.depth) return true;
    for (const [name, type] of await read(path)) {
      if (!SEGMENT.test(name) || name === 'node_modules') continue;
      const childRel = rel ? `${rel}/${name}` : name;
      if (type & vscode.FileType.Directory) {
        if (!(await sources(`${path}/${name}`, childRel, depth + 1))) return false;
      } else if (/\.(mo|fmu)$/i.test(name) && !(await take(`${path}/${name}`, childRel))) return false;
    }
    return true;
  };
  await sources(dir, '', 1);
  return out;
}

/** What this build answers: the subcommands, and for export and import the formats. */
const SUPPORTED = new Set(['--version', 'workbench', 'check', 'edit', 'format', 'metamodel', 'rules', 'transition', 'debug']);

export function createWasmBackend(wasm: Wasm): Backend {
  function unavailable(args: string[]): string | null {
    const [command, ...rest] = args;
    // Online (arclang.fr, signed in), the server runs what this build does not.
    if (runsRemotely(command) && !local(args)) return null;
    if (runsOnlineSignedIn(command) && !local(args)) return NEEDS_SIGN_IN(`'arclang ${command}'`);
    if (command === 'export' || command === 'import') {
      const format = option(rest, '-f', '--format');
      const known = command === 'export' ? format === 'sys-ml' || format === 'sysml' : format === 'sysml';
      return known ? null : NOT_AVAILABLE(`'arclang ${command} -f ${format ?? '…'}'`);
    }
    return SUPPORTED.has(command ?? '') ? null : NOT_AVAILABLE(`'arclang ${command ?? ''}'`);
  }

  /** Whether this build answers the command itself. */
  function local(args: string[]): boolean {
    const [command, ...rest] = args;
    if (command === 'export' || command === 'import') {
      const format = option(rest, '-f', '--format');
      return command === 'export' ? format === 'sys-ml' || format === 'sysml' : format === 'sysml';
    }
    // The workbench of a revision reads the history: online.
    if (command === 'workbench' && rest.some((a) => a === '--rev' || a.startsWith('--rev='))) return false;
    return SUPPORTED.has(command ?? '');
  }

  async function run(args: string[], options: RunOptions): Promise<RunResult> {
    const [command, ...rest] = args;
    if (!local(args)) await probeRemote();
    if (!local(args) && runsRemotely(command)) return runRemote(args, options);
    try {
      switch (command) {
        case '--version':
          return done(`arclang ${wasm.version()} (webassembly)\n`);
        case 'workbench': {
          const model = await readModel(positionals(rest, ['--gate-standard'])[0]);
          if (!model.ok) return model.result;
          const standard = option(rest, '--gate-standard') ?? 'iso26262';
          // The verdicts the runs kept beside the model, with the Modelica sources their keys hash: the gate and the coverage say what the runs said.
          const beside = options.cwd ? await verdictFiles(options.cwd) : null;
          return done(beside ? wasm.workbench_with(model.text, standard, rest.includes('--exports'), JSON.stringify(beside)) : wasm.workbench(model.text, standard, rest.includes('--exports')));
        }
        case 'check': {
          const model = await readModel(positionals(rest, [])[0]);
          if (!model.ok) return model.result;
          const answer = wasm.check_json(model.text);
          return { code: (JSON.parse(answer) as { success: boolean }).success ? 0 : 1, stdout: answer, stderr: '' };
        }
        case 'edit': {
          const model = await readModel(positionals(rest, ['--request'])[0]);
          if (!model.ok) return model.result;
          const answer = wasm.edit(model.text, option(rest, '--request') ?? '{}');
          return { code: (JSON.parse(answer) as { success: boolean }).success ? 0 : 1, stdout: answer, stderr: '' };
        }
        case 'format': {
          const model = await readModel(positionals(rest, [])[0]);
          if (!model.ok) return model.result;
          const answer = JSON.parse(wasm.format(model.text)) as { success: boolean; formatted?: string; error?: string };
          return answer.success ? done(answer.formatted ?? '') : failed(answer.error ?? 'not formatted');
        }
        case 'export': {
          const format = option(rest, '-f', '--format');
          const out = option(rest, '-o', '--output');
          if (format !== 'sys-ml' && format !== 'sysml') return failed(unavailable(args) ?? '', 78);
          const model = await readModel(positionals(rest, ['-f', '--format', '-o', '--output'])[0]);
          if (!model.ok) return model.result;
          const answer = JSON.parse(wasm.export_sysml(model.text)) as { success: boolean; text?: string; error?: string };
          if (!answer.success) return failed(answer.error ?? 'the model could not be exported');
          if (out) await writeOut(out, answer.text ?? '');
          return done(out ? '' : (answer.text ?? ''));
        }
        case 'import': {
          const format = option(rest, '-f', '--format');
          const out = option(rest, '-o', '--output');
          if (format !== 'sysml') return failed(unavailable(args) ?? '', 78);
          const model = await readModel(positionals(rest, ['-f', '--format', '-o', '--output'])[0]);
          if (!model.ok) return model.result;
          const answer = JSON.parse(wasm.import_sysml(model.text)) as { success: boolean; source?: string; error?: string; notes?: string[] };
          if (!answer.success) return failed(answer.error ?? 'the SysML v2 text could not be imported');
          if (out) await writeOut(out, answer.source ?? '');
          return done((answer.notes ?? []).map((note) => `  note: ${note}\n`).join(''));
        }
        case 'transition': {
          const model = await readModel(positionals(rest, ['--step', '--into'])[0]);
          if (!model.ok) return model.result;
          const plan = rest.includes('--plan');
          const answer = JSON.parse(wasm.transition(model.text, option(rest, '--step') ?? '', option(rest, '--into') ?? '', plan)) as { success: boolean; plans?: unknown; source?: string; error?: string };
          if (!answer.success) return failed(answer.error ?? 'the transition was refused');
          return done(plan ? JSON.stringify(answer.plans) : (answer.source ?? ''));
        }
        case 'debug': {
          // What a model runs step by step; the run itself is the inline debug adapter (debug-adapter.ts).
          if (!rest.includes('--list')) return failed(NOT_AVAILABLE(`'arclang debug' outside the debugger`), 78);
          const model = await readModel(positionals(rest, [])[0]);
          if (!model.ok) return model.result;
          const answer = wasm.debug_runnables(model.text);
          return answer === 'null' ? failed('the model does not compile') : done(answer);
        }
        case 'metamodel':
          return done(option(rest, '--format') === 'json' ? wasm.metamodel() : wasm.metamodel_markdown());
        case 'rules':
          return done(option(rest, '--format') === 'json' ? wasm.rules() : wasm.rules_markdown());
        default:
          return failed(unavailable(args) ?? NOT_AVAILABLE(`'arclang ${command ?? ''}'`), 78);
      }
    } catch (error) {
      return failed(`the WebAssembly compiler failed on '${command}': ${error instanceof Error ? error.message : String(error)}`, 70);
    }
  }
  return { name: 'wasm', run, unavailable };
}
