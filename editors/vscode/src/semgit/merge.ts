// The semantic merge driver and the conflicts it leaves. Installing it is
// `arclang merge --install` (after showing what it changes). A merge that
// leaves element-level markers is announced, and the Resolve conflicts view
// lists them by element with Accept ours, Accept theirs and Edit.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { run, output, failureText } from '../cli';
import { Choice, Conflict, parseConflicts, resolveConflict } from '../pure/semgit';
import { repoTop } from './git';

/** The folder git commands run in: the workspace folder of the active file, else the first. */
function workspaceDir(): string | undefined {
  const active = vscode.window.activeTextEditor?.document.uri;
  const folder = (active && vscode.workspace.getWorkspaceFolder(active)) ?? vscode.workspace.workspaceFolders?.[0];
  return folder?.uri.fsPath;
}

export async function installMergeDriver(): Promise<void> {
  const dir = workspaceDir();
  if (!dir || !(await repoTop(dir))) {
    void vscode.window.showInformationMessage('ArcLang: open a folder that is a git repository first.');
    return;
  }
  const configured = vscode.workspace.getConfiguration('arclang').get<string>('binary')?.trim();
  const args = ['merge', '--install'];
  if (configured) args.push('--binary', configured);
  const plan = await run([...args, '--dry-run'], { cwd: dir });
  if (plan.code !== 0) {
    void vscode.window.showErrorMessage(`ArcLang: ${failureText(plan).split('\n')[0]}`);
    return;
  }
  if (/already installed/.test(plan.stdout)) {
    void vscode.window.showInformationMessage('ArcLang: the semantic merge driver is already installed in this repository.');
    return;
  }
  const choice = await vscode.window.showInformationMessage('Install the ArcLang semantic merge driver in this repository?', { modal: true, detail: plan.stdout.trim() }, 'Install');
  if (choice !== 'Install') return;
  const done = await run([...args, '--yes'], { cwd: dir });
  output().appendLine(`$ arclang ${[...args, '--yes'].join(' ')}\n${done.stdout}${done.stderr}`);
  if (done.code !== 0) void vscode.window.showErrorMessage(`ArcLang: ${failureText(done).split('\n')[0]}`);
  else void vscode.window.showInformationMessage('ArcLang: git now merges .arc and .layout.json files by element.');
}

// --- The conflicts view -------------------------------------------------------

export interface FileConflicts {
  uri: vscode.Uri;
  conflicts: Conflict[];
}

type Node = { kind: 'file'; file: FileConflicts } | { kind: 'conflict'; file: FileConflicts; conflict: Conflict };

export class ConflictsView implements vscode.TreeDataProvider<Node> {
  private files: FileConflicts[] = [];
  private readonly changed = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this.changed.event;
  private announced = new Set<string>();

  get count(): number {
    return this.files.reduce((n, f) => n + f.conflicts.length, 0);
  }

  all(): FileConflicts[] {
    return this.files;
  }

  /** Look for markers in every model file of the workspace (the open ones as they are in the editor). */
  async scan(): Promise<void> {
    const found: FileConflicts[] = [];
    const uris = await vscode.workspace.findFiles('**/*.arc', '**/{node_modules,.git,target}/**', 500);
    for (const uri of uris) {
      const open = vscode.workspace.textDocuments.find((d) => d.uri.toString() === uri.toString());
      const text = open ? open.getText() : Buffer.from(await vscode.workspace.fs.readFile(uri)).toString('utf8');
      if (!text.includes('<<<<<<< ours')) continue;
      const conflicts = parseConflicts(text);
      if (conflicts.length) found.push({ uri, conflicts });
    }
    this.files = found;
    await vscode.commands.executeCommand('setContext', 'arclang.hasConflicts', found.length > 0);
    this.changed.fire();
    for (const file of found) {
      const key = `${file.uri.toString()}:${file.conflicts.length}`;
      if (this.announced.has(key)) continue;
      this.announced.add(key);
      void vscode.window
        .showWarningMessage(`ArcLang: the merge left ${file.conflicts.length} element conflict${file.conflicts.length === 1 ? '' : 's'} in ${path.basename(file.uri.fsPath)}.`, 'Resolve conflicts')
        .then((choice) => (choice ? vscode.commands.executeCommand('arclangConflicts.focus') : undefined));
    }
  }

  getTreeItem(node: Node): vscode.TreeItem {
    if (node.kind === 'file') {
      const item = new vscode.TreeItem(path.basename(node.file.uri.fsPath), vscode.TreeItemCollapsibleState.Expanded);
      item.description = `${node.file.conflicts.length} conflict${node.file.conflicts.length === 1 ? '' : 's'}`;
      item.resourceUri = node.file.uri;
      item.contextValue = 'arclangConflictFile';
      return item;
    }
    const c = node.conflict;
    const item = new vscode.TreeItem(c.label || c.element, vscode.TreeItemCollapsibleState.None);
    item.description = c.reason;
    item.tooltip = new vscode.MarkdownString(`**${c.label}**\n\n${c.reason}\n\n**Ours**\n\`\`\`\n${c.ours.join('\n')}\n\`\`\`\n**Theirs**\n\`\`\`\n${c.theirs.join('\n')}\n\`\`\``);
    item.iconPath = new vscode.ThemeIcon('git-merge');
    item.contextValue = 'arclangConflict';
    item.command = { command: 'arclang.conflict.edit', title: 'Edit', arguments: [node] };
    return item;
  }

  getChildren(node?: Node): Node[] {
    if (!node) return this.files.map((file) => ({ kind: 'file', file }));
    return node.kind === 'file' ? node.file.conflicts.map((conflict) => ({ kind: 'conflict', file: node.file, conflict })) : [];
  }
}

function nodeOf(arg: unknown): Extract<Node, { kind: 'conflict' }> | null {
  const node = arg as Node | undefined;
  return node && node.kind === 'conflict' ? node : null;
}

async function accept(view: ConflictsView, arg: unknown, choice: Choice): Promise<void> {
  const node = nodeOf(arg);
  if (!node) return;
  const document = await vscode.workspace.openTextDocument(node.file.uri);
  // The marker block as it is now in the editor (an earlier choice moved the lines).
  const current = parseConflicts(document.getText()).find((c) => c.element === node.conflict.element && c.reason === node.conflict.reason);
  if (!current) return void view.scan();
  const text = resolveConflict(document.getText(), current, choice);
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), text);
  await vscode.workspace.applyEdit(edit);
  await document.save();
  await view.scan();
}

async function edit(arg: unknown): Promise<void> {
  const node = nodeOf(arg);
  if (!node) return;
  const document = await vscode.workspace.openTextDocument(node.file.uri);
  const current = parseConflicts(document.getText()).find((c) => c.element === node.conflict.element && c.reason === node.conflict.reason) ?? node.conflict;
  const editor = await vscode.window.showTextDocument(document, { preview: false });
  const range = new vscode.Range(current.start, 0, current.end, document.lineAt(Math.min(current.end, document.lineCount - 1)).text.length);
  editor.selection = new vscode.Selection(range.start, range.start);
  editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
}

export function registerConflicts(context: vscode.ExtensionContext): ConflictsView {
  const view = new ConflictsView();
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  context.subscriptions.push(vscode.window.createTreeView('arclangConflicts', { treeDataProvider: view, showCollapseAll: true }));
  register('arclang.conflicts.refresh', () => view.scan());
  register('arclang.conflict.acceptOurs', (arg?: unknown) => accept(view, arg, 'ours'));
  register('arclang.conflict.acceptTheirs', (arg?: unknown) => accept(view, arg, 'theirs'));
  register('arclang.conflict.edit', (arg?: unknown) => edit(arg));
  let timer: NodeJS.Timeout | undefined;
  const later = () => {
    clearTimeout(timer);
    timer = setTimeout(() => void view.scan(), 600);
  };
  const watcher = vscode.workspace.createFileSystemWatcher('**/*.arc');
  context.subscriptions.push(watcher, watcher.onDidChange(later), watcher.onDidCreate(later), watcher.onDidDelete(later), vscode.workspace.onDidSaveTextDocument((d) => d.fileName.endsWith('.arc') && later()), vscode.workspace.onDidOpenTextDocument((d) => d.fileName.endsWith('.arc') && later()));
  void view.scan();
  return view;
}

