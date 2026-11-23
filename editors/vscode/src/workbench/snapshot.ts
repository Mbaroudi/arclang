// The snapshot of a document (`arclang workbench`), computed on the text in
// the editor (saved or not), once per version, and handed to whoever asks:
// the model tree, the workbench panel, the assistant.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { asFile, run, withTempFile, workspaceTrusted, UNTRUSTED_MESSAGE } from '../cli';
import { PerVersion } from '../pure/versioned';
import { Metamodel, parseSnapshot, Snapshot } from '../pure/workbench';
import { arcTextOf, documentDir, isModelDocument } from './source';

const snapshots = new PerVersion<Snapshot>();
/** The folders whose verdicts are watched, and how one more is (set once the refresh is registered). */
const watched = new Map<string, vscode.Disposable>();
let watchVerdicts: (folder: string) => void = () => undefined;
/** Documents whose verdicts changed while their snapshot was being computed. */
const overtaken = new Set<string>();
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
    const result = await run(['workbench', file, '--gate-standard', gateStandard(), ...asFile(document)], { cwd: dir });
    if (result.code !== 0 && !result.stdout.trim()) return failed(result.stderr.trim() || `arclang exited with ${result.code}`);
    return parseSnapshot(result.stdout);
  });
}

/** The snapshot of the document's current text; cached by document version. */
export function snapshotOf(document: vscode.TextDocument): Promise<Snapshot> {
  const key = document.uri.toString();
  const version = document.version;
  const cached = snapshots.cached(key, version);
  if (cached) return Promise.resolve(cached);
  if (document.uri.scheme === 'file') watchVerdicts(path.dirname(document.uri.fsPath));
  // Every outcome is cached for the version, a refusal too: the views refresh
  // when a snapshot arrives, and one that is never cached is asked again, and again.
  // A snapshot under way for an older version is not the answer for this one: it gets its own.
  const { work, started } = snapshots.get(
    key,
    version,
    () => {
      overtaken.delete(key);
      return compute(document);
    },
    // Verdicts written meanwhile are not in it: it is given, not kept.
    () => !overtaken.delete(key),
  );
  if (started) work.then(() => emitter.fire(document.uri)).catch(() => undefined);
  return work;
}

/** The last snapshot computed for a document, if any, without computing one. */
export function lastSnapshot(document: vscode.TextDocument): Snapshot | null {
  return snapshots.last(document.uri.toString()) ?? null;
}

export function forgetSnapshot(uri: vscode.Uri): void {
  snapshots.forget(uri.toString());
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

/**
 * The verdicts a command wrote beside a model (`.arclang/sim-cache`, by a
 * simulation, a SIL or an HDL run in a terminal) change what the gate and the
 * requirements say of it, though its text did not change: the snapshots of the
 * documents of that folder are forgotten and their views told.
 */
function verdictsChanged(folder: string): void {
  for (const document of vscode.workspace.textDocuments) {
    if (!isModelDocument(document) || document.uri.scheme !== 'file' || path.dirname(document.uri.fsPath) !== folder) continue;
    const key = document.uri.toString();
    if (snapshots.busy(key)) overtaken.add(key);
    snapshots.forget(key);
    // Told whether a snapshot was kept or not: a view may still show one that
    // was dropped since (a setting changed), and would never ask again.
    emitter.fire(document.uri);
  }
}

/** Recompute snapshots as documents change, after a short pause in typing. */
export function registerSnapshotRefresh(context: vscode.ExtensionContext, delayMs = 400): void {
  const timers = new Map<string, NodeJS.Timeout>();
  const settle = new Map<string, NodeJS.Timeout>();
  // A run writes several verdicts: the views are told once it has.
  const changedIn = (folder: string) => () => {
    const previous = settle.get(folder);
    if (previous) clearTimeout(previous);
    settle.set(folder, setTimeout(() => {
      settle.delete(folder);
      verdictsChanged(folder);
    }, delayMs));
  };
  // The folder of each model a snapshot was asked for is watched, in a
  // workspace folder or not.
  watchVerdicts = (folder: string) => {
    if (watched.has(folder)) return;
    const watcher = vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(vscode.Uri.file(folder), '.arclang/sim-cache/**'));
    const changed = changedIn(folder);
    watched.set(folder, vscode.Disposable.from(watcher, watcher.onDidChange(changed), watcher.onDidCreate(changed), watcher.onDidDelete(changed)));
  };
  context.subscriptions.push(
    new vscode.Disposable(() => {
      watchVerdicts = () => undefined;
      for (const watcher of watched.values()) watcher.dispose();
      watched.clear();
    }),
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
      const uris = snapshots.keys().map((key) => vscode.Uri.parse(key));
      snapshots.clear();
      for (const uri of uris) emitter.fire(uri);
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('arclang.binary')) metamodelCache = null;
      // A preference of the page (what follows a move by hand) changes no snapshot: no redraw of every view.
      if (event.affectsConfiguration('arclang') && !event.affectsConfiguration('arclang.workbench.reorganize')) snapshots.clear();
    }),
  );
}
