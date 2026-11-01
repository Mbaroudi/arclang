// Semantic git in the editor: compare the model with a revision, the history
// of an element, baselines, the semantic merge driver and the conflicts it
// leaves. The workbench page asks for them with `semgit-*` messages
// (semgit hook in src/workbench/panel.ts); the same are commands.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { SemgitFromWorkbench } from '../pure/semgit-messages';
import { GitRef } from '../pure/semgit';
import { WorkbenchPanel } from '../workbench/panel';
import { isModelDocument, documentDir } from '../workbench/source';
import { createBaseline, listBaselines, verifyBaseline } from './baseline';
import { applyCompare, compareActive, sessionOf, startCompare, stopCompare } from './compare';
import { listRefs, repoTop, safeRevision, showAt } from './git';
import { elementHistory, snapshotAt } from './history';
import { installMergeDriver, registerConflicts } from './merge';
import { handleTimeline, mergeBranch, openTimeline, switchBranch } from './timeline';

export { applyCompare, compareActive };

const REVISION_SCHEME = 'arclang-rev';

/** `arclang-rev:/<abs path>?<revision>`: the committed text of a file, for VS Code's diff. */
class RevisionText implements vscode.TextDocumentContentProvider {
  async provideTextDocumentContent(uri: vscode.Uri): Promise<string> {
    const revision = decodeURIComponent(uri.query);
    if (!safeRevision(revision)) return '';
    const file = uri.path;
    const top = await repoTop(path.dirname(file));
    return top ? ((await showAt(top, revision, file)) ?? '') : '';
  }
}

const revisionUri = (file: string, revision: string) => vscode.Uri.from({ scheme: REVISION_SCHEME, path: file, query: encodeURIComponent(revision) });

/** The model the command is about: the workbench in front, else the editor, else the first model open. */
function targetDocument(arg?: unknown): vscode.TextDocument | null {
  if (arg instanceof vscode.Uri) return vscode.workspace.textDocuments.find((d) => d.uri.toString() === arg.toString()) ?? null;
  const editor = vscode.window.activeTextEditor?.document;
  return WorkbenchPanel.activeDocument() ?? (editor && isModelDocument(editor) ? editor : null) ?? vscode.workspace.textDocuments.find((d) => isModelDocument(d) && d.uri.scheme === 'file') ?? null;
}

const kindIcon: Record<GitRef['kind'], string> = { head: '$(history)', branch: '$(git-branch)', remote: '$(cloud)', tag: '$(tag)', baseline: '$(verified)' };

/** The quick pick of what to compare with. */
export async function pickRef(document: vscode.TextDocument): Promise<{ ref: string; label: string } | null> {
  const top = await repoTop(documentDir(document) ?? '.');
  if (!top) {
    void vscode.window.showInformationMessage('ArcLang: the model is not in a git repository.');
    return null;
  }
  const refs = await listRefs(top);
  type Item = vscode.QuickPickItem & { ref?: string; other?: boolean };
  const items: Item[] = refs.map((r) => ({ label: `${kindIcon[r.kind]} ${r.name}`, description: r.kind === 'head' ? '' : r.kind, detail: r.detail, ref: r.name }));
  items.push({ label: '$(edit) Another revision…', detail: 'a commit hash, HEAD~3, a ref', other: true });
  const picked = await vscode.window.showQuickPick(items, { title: `Compare ${path.basename(document.uri.fsPath)} with…`, placeHolder: 'A branch, a tag, a baseline or HEAD', matchOnDescription: true, matchOnDetail: true });
  if (!picked) return null;
  if (picked.other) {
    const typed = (await vscode.window.showInputBox({ prompt: 'Revision', placeHolder: 'main, v1.0, HEAD~2, a1b2c3d' }))?.trim();
    return typed && safeRevision(typed) ? { ref: typed, label: typed } : null;
  }
  return { ref: picked.ref!, label: picked.ref! };
}

async function compareWith(arg?: unknown, ref?: string): Promise<void> {
  const document = targetDocument(arg);
  if (!document) return void vscode.window.showInformationMessage('ArcLang: open a .arc model first.');
  const picked = ref ? { ref, label: ref } : await pickRef(document);
  if (picked) await startCompare(document, picked.ref, picked.label);
}

async function compareWithBaseline(arg?: unknown): Promise<void> {
  const document = targetDocument(arg);
  if (!document) return void vscode.window.showInformationMessage('ArcLang: open a .arc model first.');
  const all = await listBaselines(documentDir(document) ?? '.');
  if (all.length === 0) return void vscode.window.showInformationMessage('ArcLang: this repository has no baseline yet. "ArcLang: Create baseline…" makes one.');
  const picked = await vscode.window.showQuickPick(
    all.map((b) => ({ label: `$(verified) ${b.name}`, description: `${b.standard} · ${b.gate}`, detail: `${b.commit.slice(0, 10)} ${b.created}`, name: b.name })),
    { title: 'Compare with a baseline', placeHolder: 'The model as it was when the baseline was made' },
  );
  if (picked) await startCompare(document, `baseline/${picked.name}`, `baseline ${picked.name}`);
}

async function openDiff(document: vscode.TextDocument, revision: string, other?: string): Promise<void> {
  const left = revisionUri(document.uri.fsPath, revision);
  const right = other ? revisionUri(document.uri.fsPath, other) : document.uri;
  await vscode.commands.executeCommand('vscode.diff', left, right, `${path.basename(document.uri.fsPath)}: ${revision} ↔ ${other ?? 'working tree'}`);
}

export function isSemgitMessage(message: unknown): message is SemgitFromWorkbench {
  return typeof (message as { type?: unknown } | null)?.type === 'string' && String((message as { type: string }).type).startsWith('semgit-');
}

export async function handleSemgitMessage(panel: WorkbenchPanel, message: SemgitFromWorkbench): Promise<void> {
  const document = panel.documentOf;
  switch (message.type) {
    case 'semgit-compare':
      if (message.op === 'exit') return stopCompare(document);
      if (message.op === 'baseline') return compareWithBaseline(document.uri);
      return compareWith(document.uri, message.ref);
    case 'semgit-refresh':
      return void panel.refreshView();
    case 'semgit-open-diff': {
      const session = sessionOf(document);
      if (session) await openDiff(document, session.ref);
      return;
    }
    case 'semgit-history': {
      const history = await elementHistory(document, message.id);
      panel.postPanes(typeof history === 'string' ? { type: 'history', error: history, id: message.id } : { type: 'history', history });
      return;
    }
    case 'semgit-history-show': {
      const shown = await snapshotAt(document, message.hash);
      panel.postPanes(typeof shown === 'string' ? { type: 'history-revision', hash: message.hash, error: shown } : { type: 'history-revision', hash: message.hash, diagrams: shown.diagrams });
      return;
    }
    case 'semgit-history-diff': {
      await openDiff(document, `${message.hash}~1`, message.hash);
      return;
    }
    case 'semgit-baseline':
      if (message.op === 'create') {
        const name = await createBaseline(document);
        if (name) panel.postPanes({ type: 'baseline-created', name });
      } else await compareWithBaseline(document.uri);
      return;
    case 'semgit-merge-install':
      return installMergeDriver();
    case 'semgit-timeline':
      return handleTimeline(panel, message.op, message.hash, message.base, message.element);
  }
}

export function registerSemgit(context: vscode.ExtensionContext): void {
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  context.subscriptions.push(vscode.workspace.registerTextDocumentContentProvider(REVISION_SCHEME, new RevisionText()));
  register('arclang.compare', (arg?: unknown, ref?: unknown) => compareWith(arg, typeof ref === 'string' ? ref : undefined));
  register('arclang.compare.baseline', (arg?: unknown) => compareWithBaseline(arg));
  register('arclang.compare.exit', (arg?: unknown) => {
    const document = targetDocument(arg);
    if (document) stopCompare(document);
  });
  register('arclang.compare.openDiff', async (arg?: unknown) => {
    const document = targetDocument(arg);
    const session = document && sessionOf(document);
    if (document && session) await openDiff(document, session.ref);
    else if (document) {
      const picked = await pickRef(document);
      if (picked) await openDiff(document, picked.ref);
    }
  });
  register('arclang.history', async (arg?: unknown, element?: unknown) => {
    const document = targetDocument(arg);
    if (!document) return;
    const id = typeof element === 'string' ? element : (arg as { id?: string } | undefined)?.id ?? WorkbenchPanel.for(document.uri)?.selectedId;
    if (!id) return void vscode.window.showInformationMessage('ArcLang: select an element in the workbench first.');
    if (!WorkbenchPanel.for(document.uri)) await WorkbenchPanel.open(document);
    await handleSemgitMessage(WorkbenchPanel.for(document.uri)!, { type: 'semgit-history', id });
  });
  register('arclang.baseline.create', async (arg?: unknown) => {
    const document = targetDocument(arg);
    if (document) await createBaseline(document);
  });
  register('arclang.baseline.verify', async (arg?: unknown) => {
    const document = targetDocument(arg);
    if (!document) return;
    const all = await listBaselines(documentDir(document) ?? '.');
    const picked = await vscode.window.showQuickPick(all.map((b) => ({ label: b.name, description: `${b.standard} · ${b.gate}` })), { title: 'Verify a baseline' });
    if (picked) await verifyBaseline(documentDir(document) ?? '.', picked.label);
  });
  register('arclang.merge.install', () => installMergeDriver());
  register('arclang.timeline', async (arg?: unknown) => {
    const document = targetDocument(arg);
    if (!document) return void vscode.window.showInformationMessage('ArcLang: open a .arc model first.');
    if (!WorkbenchPanel.for(document.uri)) await WorkbenchPanel.open(document);
    const panel = WorkbenchPanel.for(document.uri);
    if (panel) setTimeout(() => void openTimeline(panel), 600);
  });
  register('arclang.branch.switch', async (arg?: unknown) => {
    const document = targetDocument(arg);
    if (document) await switchBranch(document);
  });
  register('arclang.branch.merge', async (arg?: unknown) => {
    const document = targetDocument(arg);
    if (document) await mergeBranch(document);
  });
  registerConflicts(context);
  // Editing a file under comparison ends it: what is compared is no longer what is on disk.
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((document) => {
      // A past moment is git's, not the file's: saving leaves it as it is.
      if (compareActive(document) && !sessionOf(document)!.at) void startCompare(document, sessionOf(document)!.ref, sessionOf(document)!.label);
    }),
    vscode.workspace.onDidCloseTextDocument((document) => stopCompare(document)),
  );
}
