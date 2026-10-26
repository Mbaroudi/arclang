// The snapshot of a document (`arclang workbench`), computed on the text in
// the editor (saved or not), once per version, and handed to whoever asks:
// the model tree, the workbench panel, the assistant.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { run, withTempFile, workspaceTrusted, UNTRUSTED_MESSAGE } from '../cli';
import { Metamodel, parseSnapshot, Snapshot } from '../pure/workbench';
import { arcTextOf, documentDir, isModelDocument } from './source';

interface Cached {
  version: number;
  snapshot: Snapshot;
}

const cache = new Map<string, Cached>();
const pending = new Map<string, Promise<Snapshot>>();
const emitter = new vscode.EventEmitter<vscode.Uri>();

/** Fires when a document's snapshot changed. */
export const onDidChangeSnapshot = emitter.event;

function failed(message: string): Snapshot {
  return { success: false, errors: [message], warnings: [], stats: null, diagrams: null, document: null, elements: null, findings: [], gate: null };
}

/** The gate standard setting, as the CLI spells it. */
function gateStandard(): string {
  return vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262');
}

async function compute(document: vscode.TextDocument): Promise<Snapshot> {
  if (!workspaceTrusted()) return failed(UNTRUSTED_MESSAGE);
  // A SysML v2 document is imported first; the snapshot is of the imported model.
  const source = await arcTextOf(document);
  if (!source.ok) return failed(source.error);
  const dir = documentDir(document);
  const name = path.basename(document.uri.fsPath).replace(/\.sysml$/i, '.arc') || 'model.arc';
  return withTempFile(source.text, name, async (file) => {
    const result = await run(['workbench', file, '--gate-standard', gateStandard()], { cwd: dir });
    if (result.code !== 0 && !result.stdout.trim()) return failed(result.stderr.trim() || `arclang exited with ${result.code}`);
    return parseSnapshot(result.stdout);
  });
}

/** The snapshot of the document's current text; cached by document version. */
export function snapshotOf(document: vscode.TextDocument): Promise<Snapshot> {
  const key = document.uri.toString();
  const cached = cache.get(key);
  if (cached && cached.version === document.version) return Promise.resolve(cached.snapshot);
  const running = pending.get(key);
  if (running) return running;
  const version = document.version;
  // Every outcome is cached for the version, a refusal too: the views refresh
  // when a snapshot arrives, and one that is never cached is asked again, and again.
  const work = (async () => {
    const snapshot = await compute(document);
    cache.set(key, { version, snapshot });
    return snapshot;
  })();
  pending.set(key, work);
  work.finally(() => pending.delete(key)).catch(() => undefined);
  work.then(() => emitter.fire(document.uri)).catch(() => undefined);
  return work;
}

/** The last snapshot computed for a document, if any, without computing one. */
export function lastSnapshot(document: vscode.TextDocument): Snapshot | null {
  return cache.get(document.uri.toString())?.snapshot ?? null;
}

export function forgetSnapshot(uri: vscode.Uri): void {
  cache.delete(uri.toString());
}

let metamodelCache: { key: string; value: Metamodel | null } | null = null;

/** The metamodel as JSON, once per binary setting; null when the compiler cannot print it. */
export async function metamodelJson(): Promise<Metamodel | null> {
  const key = vscode.workspace.getConfiguration('arclang').get<string>('binary', 'arclang');
  if (metamodelCache && metamodelCache.key === key) return metamodelCache.value;
  const result = await run(['metamodel', '--format', 'json']);
  let value: Metamodel | null = null;
  if (result.code === 0) {
    try {
      const parsed = JSON.parse(result.stdout) as Metamodel;
      if (Array.isArray(parsed.kinds)) value = parsed;
    } catch {
      value = null;
    }
  }
  metamodelCache = { key, value };
  return value;
}

/** Recompute snapshots as documents change, after a short pause in typing. */
export function registerSnapshotRefresh(context: vscode.ExtensionContext, delayMs = 400): void {
  const timers = new Map<string, NodeJS.Timeout>();
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((event) => {
      if (!isModelDocument(event.document)) return;
      const key = event.document.uri.toString();
      const previous = timers.get(key);
      if (previous) clearTimeout(previous);
      timers.set(
        key,
        setTimeout(() => {
          timers.delete(key);
          void snapshotOf(event.document);
        }, delayMs),
      );
    }),
    vscode.workspace.onDidCloseTextDocument((document) => forgetSnapshot(document.uri)),
    // What was refused while the workspace was restricted is asked again once it is trusted.
    vscode.workspace.onDidGrantWorkspaceTrust(() => {
      const uris = [...cache.keys()].map((key) => vscode.Uri.parse(key));
      cache.clear();
      for (const uri of uris) emitter.fire(uri);
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('arclang.binary')) metamodelCache = null;
      if (event.affectsConfiguration('arclang')) cache.clear();
    }),
  );
}
