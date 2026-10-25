// The history of an element: the commits that changed it, found by the
// compiler from the semantic diff between successive revisions. The page
// lists them; picking one draws the model at that revision beside the
// current one.
import * as vscode from 'vscode';
import { run } from '../cli';
import { ElementHistory, parseHistory } from '../pure/semgit';
import { parseSnapshot } from '../pure/workbench';
import { documentDir } from '../workbench/source';
import { gateStandard } from './settings';
import { safeRevision } from './git';

export const HISTORY_LIMIT = 30;

export async function elementHistory(document: vscode.TextDocument, element: string): Promise<ElementHistory | string> {
  const result = await run(['history', document.uri.fsPath, element, '--json', '--limit', String(HISTORY_LIMIT)], { cwd: documentDir(document) });
  const parsed = parseHistory(result.stdout, result.stderr);
  return parsed.ok ? parsed.history : parsed.error;
}

/** The diagrams of the model as committed, with the layout file of that revision. */
export async function snapshotAt(document: vscode.TextDocument, revision: string): Promise<{ diagrams: unknown } | string> {
  if (!safeRevision(revision)) return `'${revision}' is not a revision`;
  const result = await run(['workbench', document.uri.fsPath, '--rev', revision, '--gate-standard', gateStandard()], { cwd: documentDir(document) });
  if (!result.stdout.trim()) return (result.stderr || 'no answer').trim();
  const snapshot = parseSnapshot(result.stdout);
  return snapshot.success && snapshot.diagrams ? { diagrams: snapshot.diagrams } : (snapshot.errors[0] ?? 'the model does not compile at that revision');
}
