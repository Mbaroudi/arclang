// Time travel in the workbench: the timeline of a model (the commits that
// touched it, with its snapshots and branch heads), the model drawn as it
// was at any of them with the changes since another marked, and from there:
// restore the model or one element as it was (an edit of the file, undoable,
// never a rewrite of history), take a snapshot (an annotated tag), start a
// branch. Merging a branch into the current one and switching branch are
// commands; a merge's conflicts go to the Resolve conflicts view.
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { run, failureText, output } from '../cli';
import { LOG_FORMAT, parseLog, previousOf, shortDate, Timeline, TimelineCommit, validRefName } from '../pure/timeline';
import { WorkbenchPanel } from '../workbench/panel';
import { documentDir } from '../workbench/source';
import { sessionOf, startTimeTravel, stopCompare } from './compare';
import { currentBranch, git, relativeTo, repoTop, safeRevision } from './git';

/** The timeline of the model of `document`, or why there is none. */
export async function timelineOf(document: vscode.TextDocument): Promise<Timeline | string> {
  if (document.uri.scheme !== 'file') return 'the model is not saved in a git repository';
  const top = await repoTop(documentDir(document) ?? path.dirname(document.uri.fsPath));
  if (!top) return 'the model is not in a git repository';
  // The repository's path is a real one (/private/var on macOS): so must the file's be.
  let file = document.uri.fsPath;
  try {
    file = fs.realpathSync(file);
  } catch {
    // keep the path as given
  }
  const log = await git(['log', '-n', '300', `--format=${LOG_FORMAT}`, '--', relativeTo(top, file)], top);
  if (log.code !== 0) return log.stderr.trim() || 'git log failed';
  const commits = parseLog(log.stdout);
  if (commits.length === 0) return 'the model has no commit yet: commit it to travel in its history';
  return { branch: await currentBranch(top), commits };
}

const label = (c: TimelineCommit) => (c.tags[0] ? `${c.tags[0]} (${c.short})` : `${c.short} ${shortDate(c.date)}`);

/** Draw the model at commit `hash`, the changes since `base` (default: the commit before it) marked. */
async function show(panel: WorkbenchPanel, timeline: Timeline, hash: string, base?: string | null): Promise<void> {
  const index = timeline.commits.findIndex((c) => c.hash === hash);
  if (index < 0) return;
  const at = timeline.commits[index];
  const before = base === undefined ? previousOf(timeline.commits, index) : base ? timeline.commits.find((c) => c.hash === base) ?? null : null;
  const failure = await startTimeTravel(panel.documentOf, { ref: at.hash, label: label(at) }, before ? { ref: before.hash, label: label(before) } : null);
  panel.postPanes({ type: 'timeline-at', hash, base: before?.hash ?? null, error: failure });
}

async function saved(document: vscode.TextDocument): Promise<boolean> {
  if (!document.isDirty) return true;
  const choice = await vscode.window.showWarningMessage('ArcLang: the model has unsaved changes. Save it first?', { modal: true }, 'Save');
  return choice === 'Save' && document.save();
}

/** Restore the model (or one element) as it was at `hash`: one edit of the document, undoable, to review and commit. */
async function restore(panel: WorkbenchPanel, timeline: Timeline, hash: string, element?: string): Promise<void> {
  const document = panel.documentOf;
  const commit = timeline.commits.find((c) => c.hash === hash);
  if (!commit || !safeRevision(hash) || !(await saved(document))) return;
  const what = element ? `element ${element}` : 'the whole model';
  const choice = await vscode.window.showWarningMessage(`ArcLang: restore ${what} as it was at ${label(commit)}?`, { modal: true, detail: 'The file is edited (Undo takes it back); nothing is committed and the history is not rewritten.' }, 'Restore');
  if (choice !== 'Restore') return;
  const args = ['restore', document.uri.fsPath, '--rev', hash];
  if (element) args.push('--element', element);
  const result = await run(args, { cwd: documentDir(document) });
  if (result.code !== 0) return void vscode.window.showErrorMessage(`ArcLang: ${failureText(result).split('\n').pop()}`);
  stopCompare(document);
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), result.stdout);
  await vscode.workspace.applyEdit(edit);
  panel.postPanes({ type: 'timeline-restored', what, at: label(commit) });
}

async function snapshot(panel: WorkbenchPanel, timeline: Timeline, hash: string): Promise<void> {
  const top = await repoTop(documentDir(panel.documentOf) ?? '.');
  const commit = timeline.commits.find((c) => c.hash === hash);
  if (!top || !commit) return;
  const name = (await vscode.window.showInputBox({ title: `Snapshot of the model at ${label(commit)}`, prompt: 'Name (a git tag, never moved)', placeHolder: 'design-review-2', validateInput: (v) => (validRefName(v.trim()) ? null : 'Letters, digits, - _ . /') }))?.trim();
  if (!name) return;
  const note = (await vscode.window.showInputBox({ title: `Snapshot ${name}`, prompt: 'What this moment is (optional)', placeHolder: 'Model as reviewed with the customer' }))?.trim();
  const tagged = await git(['tag', '-a', name, '-m', note || name, hash], top);
  if (tagged.code !== 0) return void vscode.window.showErrorMessage(`ArcLang: ${tagged.stderr.trim() || 'the snapshot could not be taken'}`);
  void vscode.window.showInformationMessage(`ArcLang: snapshot ${name} taken on ${commit.short}.`);
  await openTimeline(panel);
}

async function branch(panel: WorkbenchPanel, timeline: Timeline, hash: string): Promise<void> {
  const top = await repoTop(documentDir(panel.documentOf) ?? '.');
  const commit = timeline.commits.find((c) => c.hash === hash);
  if (!top || !commit) return;
  const name = (await vscode.window.showInputBox({ title: `New branch from ${label(commit)}`, prompt: 'Branch name', placeHolder: 'feature/new-braking-mode', validateInput: (v) => (validRefName(v.trim()) ? null : 'Letters, digits, - _ . /') }))?.trim();
  if (!name) return;
  const made = await git(['branch', name, hash], top);
  if (made.code !== 0) return void vscode.window.showErrorMessage(`ArcLang: ${made.stderr.trim()}`);
  const choice = await vscode.window.showInformationMessage(`ArcLang: branch ${name} created at ${commit.short}.`, 'Switch to it');
  if (choice === 'Switch to it') await switchTo(panel.documentOf, name);
  else await openTimeline(panel);
}

/** Open the timeline on the workbench of `panel`. */
export async function openTimeline(panel: WorkbenchPanel): Promise<void> {
  const timeline = await timelineOf(panel.documentOf);
  panel.postPanes(typeof timeline === 'string' ? { type: 'timeline', error: timeline } : { type: 'timeline', timeline, at: sessionOf(panel.documentOf)?.at?.ref ?? null });
}

export async function handleTimeline(panel: WorkbenchPanel, op: string, hash?: string, base?: string | null, element?: string): Promise<void> {
  if (op === 'open') return openTimeline(panel);
  if (op === 'now') {
    stopCompare(panel.documentOf);
    return;
  }
  const timeline = await timelineOf(panel.documentOf);
  if (typeof timeline === 'string' || !hash) return;
  if (op === 'show') return show(panel, timeline, hash, base);
  if (op === 'restore') return restore(panel, timeline, hash, element);
  if (op === 'snapshot') return snapshot(panel, timeline, hash);
  if (op === 'branch') return branch(panel, timeline, hash);
}

// --- Branches: switch, merge ----------------------------------------------------

async function branches(top: string): Promise<string[]> {
  const out = await git(['for-each-ref', '--format=%(refname:short)', 'refs/heads'], top);
  return out.code === 0 ? out.stdout.split('\n').map((l) => l.trim()).filter(Boolean) : [];
}

async function switchTo(document: vscode.TextDocument, name: string): Promise<void> {
  const top = await repoTop(documentDir(document) ?? '.');
  if (!top || !(await saved(document))) return;
  const switched = await git(['switch', name], top);
  if (switched.code !== 0) return void vscode.window.showErrorMessage(`ArcLang: ${switched.stderr.trim()}`);
  stopCompare(document);
  void vscode.window.showInformationMessage(`ArcLang: on branch ${name}.`);
  const panel = WorkbenchPanel.for(document.uri);
  if (panel) await openTimeline(panel);
}

export async function switchBranch(document: vscode.TextDocument): Promise<void> {
  const top = await repoTop(documentDir(document) ?? '.');
  if (!top) return void vscode.window.showInformationMessage('ArcLang: the model is not in a git repository.');
  const now = await currentBranch(top);
  const picked = await vscode.window.showQuickPick((await branches(top)).filter((b) => b !== now).map((b) => ({ label: `$(git-branch) ${b}`, name: b })), { title: `Switch branch (now on ${now ?? 'a detached head'})` });
  if (picked) await switchTo(document, picked.name);
}

/** Merge a branch into the one checked out, by element (the semantic merge driver); conflicts go to the Resolve conflicts view. */
export async function mergeBranch(document: vscode.TextDocument): Promise<void> {
  const top = await repoTop(documentDir(document) ?? '.');
  if (!top) return void vscode.window.showInformationMessage('ArcLang: the model is not in a git repository.');
  if (!(await saved(document))) return;
  const now = await currentBranch(top);
  const picked = await vscode.window.showQuickPick((await branches(top)).filter((b) => b !== now).map((b) => ({ label: `$(git-merge) ${b}`, name: b })), { title: `Merge a branch into ${now}`, placeHolder: 'Elements changed on one side only merge by themselves' });
  if (!picked) return;
  const driver = await git(['config', '--get', 'merge.arclang.driver'], top);
  if (driver.code !== 0) {
    const install = await vscode.window.showWarningMessage('ArcLang: the semantic merge driver is not installed in this repository: git would merge the model line by line.', { modal: true }, 'Install it and merge', 'Merge line by line');
    if (!install) return;
    if (install === 'Install it and merge') await vscode.commands.executeCommand('arclang.merge.install');
  }
  const merged = await git(['merge', '--no-ff', '--no-edit', picked.name], top);
  output().appendLine(`$ git merge --no-ff --no-edit ${picked.name}\n${merged.stdout}${merged.stderr}`);
  stopCompare(document);
  if (merged.code === 0) {
    void vscode.window.showInformationMessage(`ArcLang: ${picked.name} merged into ${now}.`);
  } else {
    void vscode.window.showWarningMessage(`ArcLang: ${picked.name} conflicts with ${now} on some elements: resolve them one by one.`, 'Resolve conflicts').then((choice) => {
      if (choice) void vscode.commands.executeCommand('arclangConflicts.focus');
    });
  }
  const panel = WorkbenchPanel.for(document.uri);
  if (panel) await openTimeline(panel);
}
