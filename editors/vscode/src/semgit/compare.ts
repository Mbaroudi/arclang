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
  let shown = snapshot;
  let drawn: string[] = [];
  if (snapshot.diagrams && session.old?.diagrams) {
    const removed = session.state.marks.filter((m) => m.status === 'removed');
    const ghosted = withGhosts(snapshot.diagrams, session.old.diagrams, removed.map((m) => m.id), removed.map((m) => m.uuid));
    shown = { ...snapshot, diagrams: ghosted.diagrams };
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
