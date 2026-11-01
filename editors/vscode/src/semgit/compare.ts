// Compare mode: the model against a git revision. The compiler computes the
// semantic diff (`arclang diff --from`) and the snapshot of the old model
// (`arclang workbench --rev`); the workbench draws the current diagrams with
// the changes marked and the removed elements ghosted at their old place.
// Editing is off while the comparison is on.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { run, failureText } from '../cli';
import { CompareState, compareState, diffArgs, parseDiff, RichDiff, withGhosts } from '../pure/semgit';
import { parseSnapshot, Snapshot } from '../pure/workbench';
import { WorkbenchPanel } from '../workbench/panel';
import { documentDir } from '../workbench/source';
import { gateStandard } from './settings';

export interface Session {
  ref: string;
  label: string;
  diff: RichDiff;
  state: CompareState;
  old: Snapshot | null;
  /** Time travel: the model as it was at a revision, drawn instead of the model now. */
  at?: { ref: string; label: string; snapshot: Snapshot };
}

const sessions = new Map<string, Session>();
const emitter = new vscode.EventEmitter<vscode.Uri>();
export const onDidChangeCompare = emitter.event;

export function sessionOf(document: vscode.TextDocument): Session | undefined {
  return sessions.get(document.uri.toString());
}

export function compareActive(document: vscode.TextDocument): boolean {
  return sessions.has(document.uri.toString());
}

/** The snapshot as the page draws it in compare mode, and the messages that go with it. */
export function applyCompare(document: vscode.TextDocument, snapshot: Snapshot): { snapshot: Snapshot; messages: unknown[] } {
  const session = sessions.get(document.uri.toString());
  if (!session) return { snapshot, messages: [{ type: 'compare', state: null }] };
  // At a past moment the page draws that moment's model, read-only.
  const base = session.at?.snapshot ?? snapshot;
  let shown = base;
  let drawn: string[] = [];
  if (base.diagrams && session.old?.diagrams) {
    const removed = session.state.marks.filter((m) => m.status === 'removed');
    const ghosted = withGhosts(base.diagrams, session.old.diagrams, removed.map((m) => m.id), removed.map((m) => m.uuid));
    shown = { ...base, diagrams: ghosted.diagrams };
    drawn = ghosted.drawn;
  }
  return { snapshot: shown, messages: [{ type: 'compare', state: { ...session.state, ghostDrawn: drawn } }] };
}

async function saved(document: vscode.TextDocument): Promise<boolean> {
  if (!document.isDirty) return true;
  const choice = await vscode.window.showInformationMessage('ArcLang: the comparison reads the saved file. Save the model first?', { modal: true }, 'Save and compare');
  if (choice !== 'Save and compare') return false;
  return document.save();
}

/** Compare the model of `document` with a git revision and show it in the workbench. */
export async function startCompare(document: vscode.TextDocument, ref: string, label = ref): Promise<boolean> {
  if (document.uri.scheme !== 'file') {
    void vscode.window.showInformationMessage('ArcLang: compare needs a model saved in a git repository.');
    return false;
  }
  if (!(await saved(document))) return false;
  const file = document.uri.fsPath;
  const cwd = documentDir(document);
  const standard = gateStandard();
  const [diffRun, oldRun] = await vscode.window.withProgress({ location: vscode.ProgressLocation.Window, title: `ArcLang: comparing with ${label}` }, () =>
    Promise.all([run(diffArgs(file, ref, null, standard), { cwd }), run(['workbench', file, '--rev', ref, '--gate-standard', standard], { cwd })]),
  );
  // Exit 1 is "they differ", not a failure: the JSON tells.
  const parsed = parseDiff(diffRun.stdout, diffRun.stderr || failureText(diffRun));
  if (!parsed.ok) {
    void vscode.window.showErrorMessage(`ArcLang: cannot compare with ${label}. ${parsed.error.split('\n')[0]}`);
    return false;
  }
  const old = oldRun.stdout.trim() ? parseSnapshot(oldRun.stdout) : null;
  const session: Session = { ref, label, diff: parsed.diff, state: compareState(parsed.diff, ref, label), old: old && old.success ? old : null };
  sessions.set(document.uri.toString(), session);
  await vscode.commands.executeCommand('setContext', 'arclang.comparing', true);
  if (!WorkbenchPanel.for(document.uri)) await WorkbenchPanel.open(document);
  refreshPanels(document);
  emitter.fire(document.uri);
  return true;
}

/**
 * Time travel: the model as it was at revision `at`, the changes since
 * `base` marked on it (added, modified, removed ghosted). Read-only: what is
 * shown is git's, not the file.
 */
export async function startTimeTravel(document: vscode.TextDocument, at: { ref: string; label: string }, base: { ref: string; label: string } | null): Promise<string | null> {
  if (document.uri.scheme !== 'file') return 'time travel needs a model saved in a git repository';
  const file = document.uri.fsPath;
  const cwd = documentDir(document);
  const standard = gateStandard();
  // The first commit has nothing before it: compared with itself, nothing is marked.
  const since = base ?? at;
  const [atRun, diffRun, baseRun] = await vscode.window.withProgress({ location: vscode.ProgressLocation.Window, title: `ArcLang: the model at ${at.label}` }, () =>
    Promise.all([
      run(['workbench', file, '--rev', at.ref, '--gate-standard', standard], { cwd }),
      run(diffArgs(file, since.ref, at.ref, standard), { cwd }),
      run(['workbench', file, '--rev', since.ref, '--gate-standard', standard], { cwd }),
    ]),
  );
  const atSnapshot = atRun.stdout.trim() ? parseSnapshot(atRun.stdout) : null;
  if (!atSnapshot || !atSnapshot.success) return `the model does not compile at ${at.label}`;
  const parsed = parseDiff(diffRun.stdout, diffRun.stderr || failureText(diffRun));
  if (!parsed.ok) return parsed.error.split('\n')[0];
  const old = baseRun.stdout.trim() ? parseSnapshot(baseRun.stdout) : null;
  const label = base ? `${at.label} · since ${base.label}` : at.label;
  const state: CompareState = { ...compareState(parsed.diff, since.ref, label), timeTravel: true };
  const session: Session = { ref: since.ref, label, diff: parsed.diff, state, old: old && old.success ? old : null, at: { ...at, snapshot: atSnapshot } };
  sessions.set(document.uri.toString(), session);
  await vscode.commands.executeCommand('setContext', 'arclang.comparing', true);
  if (!WorkbenchPanel.for(document.uri)) await WorkbenchPanel.open(document);
  refreshPanels(document);
  emitter.fire(document.uri);
  return null;
}

export function stopCompare(document: vscode.TextDocument): void {
  if (!sessions.delete(document.uri.toString())) return;
  void vscode.commands.executeCommand('setContext', 'arclang.comparing', sessions.size > 0);
  refreshPanels(document);
  emitter.fire(document.uri);
}

function refreshPanels(document: vscode.TextDocument): void {
  for (const panel of WorkbenchPanel.all(document.uri)) panel.refreshView();
}

export function baseName(document: vscode.TextDocument): string {
  return path.basename(document.uri.fsPath);
}
