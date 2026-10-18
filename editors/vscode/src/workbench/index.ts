// The workbench: the model tree in the side bar, the panel beside the
// source, and the commands of both. Wired here, in one place.
import * as vscode from 'vscode';
import { elementLabel } from '../pure/workbench';
import { applyRequests, refusalMessage } from './edits';
import { WorkbenchPanel } from './panel';
import { forgetSnapshot, registerSnapshotRefresh, snapshotOf } from './snapshot';
import { isModelDocument } from './source';
import { ModelItem, ModelTreeProvider, revealElement } from './tree';

/** The document to work on: the one given, the active .arc editor's, else the tree's. */
async function documentFrom(uri: unknown, tree: ModelTreeProvider): Promise<vscode.TextDocument | null> {
  if (uri instanceof vscode.Uri) return vscode.workspace.openTextDocument(uri);
  const active = vscode.window.activeTextEditor;
  if (active && isModelDocument(active.document)) return active.document;
  if (tree.shown) return tree.shown;
  const panelDocument = vscode.workspace.textDocuments.find((d) => WorkbenchPanel.for(d.uri));
  if (panelDocument) return panelDocument;
  vscode.window.showInformationMessage('ArcLang: open a .arc or .sysml file first.');
  return null;
}

function elementItem(item: unknown): Extract<ModelItem, { type: 'element' }> | null {
  if (typeof item === 'object' && item !== null && (item as ModelItem).type === 'element') return item as Extract<ModelItem, { type: 'element' }>;
  return null;
}

export function registerWorkbench(context: vscode.ExtensionContext): void {
  registerSnapshotRefresh(context);
  const tree = new ModelTreeProvider(context);
  const view = vscode.window.createTreeView('arclangModel', { treeDataProvider: tree, showCollapseAll: true });
  context.subscriptions.push(view);

  const register = (command: string, callback: (...args: unknown[]) => unknown) =>
    context.subscriptions.push(vscode.commands.registerCommand(command, callback));

  register('arclang.openWorkbench', async (uri?: unknown) => {
    const document = await documentFrom(uri, tree);
    if (!document) return;
    tree.follow(document);
    WorkbenchPanel.open(context, document);
  });

  register('arclang.model.refresh', () => {
    if (tree.shown) forgetSnapshot(tree.shown.uri);
    if (tree.shown) void snapshotOf(tree.shown);
    tree.refresh();
  });

  register('arclang.model.reveal', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    await revealElement(document, element.node.element);
    WorkbenchPanel.for(element.uri)?.select(element.node.element.id);
  });

  register('arclang.model.rename', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const record = element.node.element;
    const name = await vscode.window.showInputBox({ prompt: `New name of ${elementLabel(record)}`, value: record.name, validateInput: (v) => (v.trim() ? null : 'a name is needed') });
    if (name === undefined || name.trim() === record.name) return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    const outcome = await applyRequests(document, [{ op: 'rename', element: record.id, name: name.trim() }]);
    if (!outcome.ok) vscode.window.showWarningMessage(refusalMessage(`rename ${record.id}`, outcome));
  });

  register('arclang.model.delete', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const record = element.node.element;
    const owned = element.node.children.length;
    const choice = await vscode.window.showWarningMessage(
      `Delete ${elementLabel(record)}${owned ? ` and the ${owned} element${owned > 1 ? 's' : ''} it owns` : ''}? The compiler refuses it when something else still refers to it. Undo takes it back.`,
      { modal: true },
      'Delete',
    );
    if (choice !== 'Delete') return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    const outcome = await applyRequests(document, [{ op: 'remove', element: record.id }]);
    if (!outcome.ok) vscode.window.showWarningMessage(refusalMessage(`delete ${record.id}`, outcome));
  });

  register('arclang.model.ask', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const record = element.node.element;
    await vscode.commands.executeCommand('workbench.action.chat.open', {
      query: `@arclang /explain the element ${record.name} (${record.id}, ${record.kind}) of the open model: its role, what it exchanges with, and what the findings say about it.`,
    });
  });
}
