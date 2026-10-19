// The workbench: the model tree in the side bar, the custom editor beside
// the source, the status bar item, and the commands of all three.
import * as vscode from 'vscode';
import { elementLabel, ViewerGraph } from '../pure/workbench';
import { applyRequests, refusalMessage } from './edits';
import { onDidSelectElement, WorkbenchPanel } from './panel';
import { forgetSnapshot, registerSnapshotRefresh, snapshotOf } from './snapshot';
import { isModelDocument } from './source';
import { registerStatusBar } from './status';
import { graphsDrawing, ModelItem, ModelTreeProvider, revealElement } from './tree';

/** The document to work on: the one given, the active model editor's, else the tree's. */
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

const VIEW_ORDER = ['oab', 'sab', 'lab', 'pab', 'cap', 'cdb', 'pbs', 'msm', 'es'];

/** The diagrams of a document as quick pick items, viewpoints in method order. */
function diagramPicks(graphs: ViewerGraph[]): (vscode.QuickPickItem & { graph: ViewerGraph })[] {
  const order = (g: ViewerGraph) => {
    const at = VIEW_ORDER.indexOf(g.arc.view);
    return at < 0 ? VIEW_ORDER.length : at;
  };
  return [...graphs]
    .sort((a, b) => order(a) - order(b) || a.arc.title.localeCompare(b.arc.title))
    .map((graph) => ({ label: `$(graph) ${graph.arc.title}`, description: graph.arc.view.toUpperCase(), detail: graph.id, graph }));
}

export function registerWorkbench(context: vscode.ExtensionContext): void {
  registerSnapshotRefresh(context);
  registerStatusBar(context);
  context.subscriptions.push(WorkbenchPanel.register(context));
  const tree = new ModelTreeProvider(context);
  const view = vscode.window.createTreeView('arclangModel', { treeDataProvider: tree, showCollapseAll: true });
  context.subscriptions.push(view);

  const register = (command: string, callback: (...args: unknown[]) => unknown) =>
    context.subscriptions.push(vscode.commands.registerCommand(command, callback));

  register('arclang.openWorkbench', async (uri?: unknown) => {
    const document = await documentFrom(uri, tree);
    if (!document) return;
    tree.follow(document);
    await WorkbenchPanel.open(document);
  });

  register('arclang.openDiagram', async (item?: unknown) => {
    const picked = item as Extract<ModelItem, { type: 'diagram' }> | undefined;
    if (!picked || picked.type !== 'diagram') return;
    const document = await vscode.workspace.openTextDocument(picked.uri);
    await WorkbenchPanel.open(document);
    WorkbenchPanel.for(document.uri)?.show(picked.graph.id);
  });

  register('arclang.pickDiagram', async (uri?: unknown) => {
    const document = await documentFrom(uri, tree);
    if (!document) return;
    const snapshot = await snapshotOf(document);
    const graphs = snapshot.diagrams?.graphs ?? [];
    if (graphs.length === 0) {
      vscode.window.showInformationMessage('ArcLang: this model has no diagram yet (does it compile?).');
      return;
    }
    const choice = await vscode.window.showQuickPick(diagramPicks(graphs), { placeHolder: `Open a diagram of ${document.uri.path.split('/').pop()} (${graphs.length})`, matchOnDescription: true, matchOnDetail: true });
    if (!choice) return;
    await WorkbenchPanel.open(document);
    WorkbenchPanel.for(document.uri)?.show(choice.graph.id);
  });

  register('arclang.pickElement', async (uri?: unknown) => {
    const document = await documentFrom(uri, tree);
    if (!document) return;
    const snapshot = await snapshotOf(document);
    const records = snapshot.elements?.elements ?? [];
    const items = records.map((e) => ({ label: e.name || e.id, description: e.kind, detail: e.id, element: e }));
    const choice = await vscode.window.showQuickPick(items, { placeHolder: `Go to an element of ${document.uri.path.split('/').pop()} (${records.length})`, matchOnDescription: true, matchOnDetail: true });
    if (!choice) return;
    const drawn = graphsDrawing(snapshot.diagrams?.graphs ?? [], choice.element);
    await revealElement(document, choice.element, { preserveFocus: drawn.length > 0 });
    if (drawn.length > 0) {
      await WorkbenchPanel.open(document);
      WorkbenchPanel.for(document.uri)?.show(drawn[0].id, choice.element.id);
    }
  });

  register('arclang.model.refresh', () => {
    if (tree.shown) forgetSnapshot(tree.shown.uri);
    if (tree.shown) void snapshotOf(tree.shown);
    tree.refresh();
  });

  // A click in the tree: the element is selected in the views and shown in the text, focus kept.
  register('arclang.model.select', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    const panel = WorkbenchPanel.for(element.uri);
    if (panel) {
      const snapshot = await snapshotOf(document);
      const drawn = graphsDrawing(snapshot.diagrams?.graphs ?? [], element.node.element);
      if (drawn.length > 0) panel.show(drawn[0].id, element.node.element.id);
      else panel.select(element.node.element.id);
    }
    await revealElement(document, element.node.element, { preserveFocus: true });
  });

  register('arclang.model.reveal', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    await revealElement(document, element.node.element);
  });

  register('arclang.model.openDiagram', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    const snapshot = await snapshotOf(document);
    const drawn = graphsDrawing(snapshot.diagrams?.graphs ?? [], element.node.element);
    if (drawn.length === 0) {
      vscode.window.showInformationMessage(`ArcLang: ${elementLabel(element.node.element)} is drawn on no diagram.`);
      return;
    }
    await WorkbenchPanel.open(document);
    WorkbenchPanel.for(document.uri)?.show(drawn[0].id, element.node.element.id);
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
    const document = await vscode.workspace.openTextDocument(element.uri);
    if (owned > 0) {
      const choice = await vscode.window.showWarningMessage(
        `Delete ${elementLabel(record)} and the ${owned} element${owned > 1 ? 's' : ''} it owns? The compiler refuses it when something else still refers to it. Undo takes it back.`,
        { modal: true },
        'Delete',
      );
      if (choice !== 'Delete') return;
    }
    const outcome = await applyRequests(document, [{ op: 'remove', element: record.id }]);
    if (!outcome.ok) vscode.window.showWarningMessage(refusalMessage(`delete ${record.id}`, outcome));
    else vscode.window.setStatusBarMessage(`ArcLang: deleted ${elementLabel(record)} (Undo in the editor takes it back)`, 8000);
  });

  register('arclang.model.copyId', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    await vscode.env.clipboard.writeText(element.node.element.id);
    vscode.window.setStatusBarMessage(`ArcLang: copied ${element.node.element.id}`, 3000);
  });

  register('arclang.model.ask', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const record = element.node.element;
    await vscode.commands.executeCommand('workbench.action.chat.open', {
      query: `@arclang /explain the element ${record.name} (${record.id}, ${record.kind}) of the open model: its role, what it exchanges with, and what the findings say about it.`,
    });
  });

  // A selection in the views is revealed in the tree, when the tree shows that model.
  context.subscriptions.push(
    onDidSelectElement(async ({ uri, id }) => {
      if (!id || !tree.shown || tree.shown.uri.toString() !== uri.toString() || !view.visible) return;
      const snapshot = await snapshotOf(tree.shown);
      const record = (snapshot.elements?.elements ?? []).find((e) => e.id === id);
      const item = record ? tree.itemOf(record.uuid) : undefined;
      if (item) await view.reveal(item, { select: true, focus: false, expand: false });
    }),
  );
}
