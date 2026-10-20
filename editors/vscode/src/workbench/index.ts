// The workbench: the model tree in the side bar, the custom editor beside
// the source, the status bar item, and the commands of all three.
import * as vscode from 'vscode';
import { CanvasCommand, elementLabel, VIEW_OF_LAYER, ViewerGraph } from '../pure/workbench';
import { applyRequests, refusalMessage } from './edits';
import { onDidSelectElement, WorkbenchPanel } from './panel';
import { forgetSnapshot, registerSnapshotRefresh, snapshotOf } from './snapshot';
import { isModelDocument } from './source';
import { RelationItem, RelationsProvider } from './relations';
import { registerStatusBar } from './status';
import { graphsDrawing, ModelItem, ModelTreeProvider, revealElement } from './tree';
import { ExportArgs, pickerItems, themeStore } from './diagram-export';

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
  const relations = new RelationsProvider(context);
  const relationsView = vscode.window.createTreeView('arclangRelations', { treeDataProvider: relations });
  context.subscriptions.push(view, relationsView);

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

  // The canvas's keys, bound in VS Code so they follow the keyboard layout (⌘Z on AZERTY is ⌘Z).
  register('arclang.workbench.undo', () => WorkbenchPanel.activePanel()?.undo());
  register('arclang.workbench.redo', () => WorkbenchPanel.activePanel()?.redo());
  const canvasKeys: [string, CanvasCommand][] = [['copy', 'copy'], ['cut', 'cut'], ['paste', 'paste'], ['duplicate', 'duplicate'], ['selectAll', 'select-all'], ['delete', 'delete'], ['zoomIn', 'zoom-in'], ['zoomOut', 'zoom-out'], ['fit', 'fit']];
  for (const [name, command] of canvasKeys) register(`arclang.workbench.${name}`, () => WorkbenchPanel.activePanel()?.canvas(command));

  // The diagram on screen as an image or SVG on the clipboard, as slides, and its colour theme.
  const activeOr = (verb: string) => {
    const panel = WorkbenchPanel.activePanel();
    if (!panel) void vscode.window.showInformationMessage(`ArcLang: open a workbench to ${verb}.`);
    return panel;
  };
  register('arclang.diagram.copyImage', () => activeOr('copy its diagram')?.diagramAction('copy-image'));
  register('arclang.diagram.copySvg', () => activeOr('copy its diagram')?.diagramAction('copy-svg'));
  register('arclang.diagram.exportPowerPoint', (args?: unknown) => activeOr('export its diagrams')?.diagramAction('export-pptx', (args ?? undefined) as ExportArgs | undefined));
  register('arclang.diagram.editTheme', () => activeOr('edit a theme on its diagram')?.diagramAction('edit-theme'));
  register('arclang.diagram.chooseTheme', async (key?: unknown) => {
    const store = themeStore(context);
    const entries = await store.list();
    const { entry: current } = await store.active(entries);
    const wanted = typeof key === 'string' ? entries.find((e) => e.key === key || e.id === key) : undefined;
    const picked = wanted ? { entry: wanted } : await vscode.window.showQuickPick(pickerItems(entries, current.key), { placeHolder: 'Colour theme of every diagram, on every layer' });
    if (!picked) return;
    if (!picked.entry.ok) {
      void vscode.window.showErrorMessage(`ArcLang: the theme "${picked.entry.name}" is refused: ${picked.entry.errors.join('; ')}`);
      return;
    }
    await store.choose(picked.entry);
  });

  register('arclang.layer.next', () => WorkbenchPanel.activeDocument() && WorkbenchPanel.for(WorkbenchPanel.activeDocument()!.uri)?.stepLayer(1));
  register('arclang.layer.previous', () => WorkbenchPanel.activeDocument() && WorkbenchPanel.for(WorkbenchPanel.activeDocument()!.uri)?.stepLayer(-1));

  // A layer of the tree opens the diagram of that layer.
  register('arclang.openLayer', async (item?: unknown) => {
    const layer = item as Extract<ModelItem, { type: 'layer' }> | undefined;
    if (!layer || layer.type !== 'layer') return;
    const document = await vscode.workspace.openTextDocument(layer.uri);
    const snapshot = await snapshotOf(document);
    const view = VIEW_OF_LAYER[layer.layer];
    const graph = (snapshot.diagrams?.graphs ?? []).find((g) => g.arc.view === view);
    await WorkbenchPanel.open(document);
    if (graph) WorkbenchPanel.for(document.uri)?.show(graph.id);
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
    relations.show(document, element.node.element);
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

  // The native context menu of the canvas: the argument carries the element's id.
  const canvasTarget = async (arg: unknown): Promise<{ document: vscode.TextDocument; id: string; name: string; kind: string } | null> => {
    const context = (typeof arg === 'object' && arg !== null ? arg : {}) as { elementId?: string; elementName?: string; elementKind?: string };
    const document = WorkbenchPanel.activeDocument() ?? tree.shown;
    if (!document || !context.elementId) return null;
    return { document, id: context.elementId, name: context.elementName ?? context.elementId, kind: context.elementKind ?? '' };
  };
  register('arclang.canvas.reveal', async (arg?: unknown) => {
    const target = await canvasTarget(arg);
    if (target) await revealElement(target.document, { id: target.id, name: target.name });
  });
  register('arclang.canvas.tree', async (arg?: unknown) => {
    const target = await canvasTarget(arg);
    if (!target) return;
    tree.follow(target.document);
    const snapshot = await snapshotOf(target.document);
    const record = (snapshot.elements?.elements ?? []).find((e) => e.id === target.id);
    await vscode.commands.executeCommand('arclangModel.focus');
    const item = record ? tree.itemOf(record.uuid) : undefined;
    if (item) await view.reveal(item, { select: true, focus: true, expand: true });
  });
  register('arclang.canvas.ask', async (arg?: unknown) => {
    const target = await canvasTarget(arg);
    if (!target) return;
    await vscode.commands.executeCommand('workbench.action.chat.open', {
      query: `@arclang /explain the element ${target.name} (${target.id}, ${target.kind}) of the open model: its role, what it exchanges with, and what the findings say about it.`,
    });
  });
  register('arclang.canvas.copyId', async (arg?: unknown) => {
    const target = await canvasTarget(arg);
    if (target) await vscode.env.clipboard.writeText(target.id);
  });
  register('arclang.canvas.rename', async (arg?: unknown) => {
    const target = await canvasTarget(arg);
    if (target) WorkbenchPanel.for(target.document.uri)?.diagramOp('rename', target.id);
  });
  for (const [command, op] of [['arclang.canvas.hide', 'hide'], ['arclang.canvas.related', 'related'], ['arclang.canvas.newDiagram', 'new']] as const) {
    register(command, async (arg?: unknown) => {
      const target = await canvasTarget(arg);
      if (target) WorkbenchPanel.for(target.document.uri)?.diagramOp(op, target.id);
    });
  }
  register('arclang.canvas.delete', async (arg?: unknown) => {
    const target = await canvasTarget(arg);
    if (!target) return;
    const snapshot = await snapshotOf(target.document);
    const record = (snapshot.elements?.elements ?? []).find((e) => e.id === target.id);
    const owned = record ? (snapshot.elements?.elements ?? []).filter((e) => e.owner === record.uuid).length : 0;
    if (owned > 0) {
      const choice = await vscode.window.showWarningMessage(`Delete ${target.name} and the ${owned} element${owned > 1 ? 's' : ''} it owns? Undo takes it back.`, { modal: true }, 'Delete');
      if (choice !== 'Delete') return;
    }
    const outcome = await applyRequests(target.document, [{ op: 'remove', element: target.id }]);
    if (!outcome.ok) vscode.window.showWarningMessage(refusalMessage(`delete ${target.id}`, outcome));
  });

  // A selection in the views is revealed in the tree, when the tree shows that model.
  context.subscriptions.push(
    onDidSelectElement(async ({ uri, id }) => {
      const document = vscode.workspace.textDocuments.find((d) => d.uri.toString() === uri.toString());
      if (!document) return;
      const snapshot = await snapshotOf(document);
      const record = id ? (snapshot.elements?.elements ?? []).find((e) => e.id === id) : undefined;
      relations.show(document, record ?? null);
      if (!record || !tree.shown || tree.shown.uri.toString() !== uri.toString() || !view.visible) return;
      const item = tree.itemOf(record.uuid);
      if (item) await view.reveal(item, { select: true, focus: false, expand: false });
    }),
  );

  // A step in the relations view: the related element becomes the selection everywhere.
  register('arclang.relations.step', async (item?: unknown) => {
    const picked = item as RelationItem | undefined;
    if (!picked || picked.type !== 'entry' || !picked.entry.other) return;
    const document = await vscode.workspace.openTextDocument(picked.entry.uri);
    const other = picked.entry.other;
    relations.show(document, other);
    const panel = WorkbenchPanel.for(document.uri);
    if (panel) {
      const snapshot = await snapshotOf(document);
      const drawn = graphsDrawing(snapshot.diagrams?.graphs ?? [], other);
      if (drawn.length > 0) panel.show(drawn[0].id, other.id);
      else panel.select(other.id);
    }
    await revealElement(document, other, { preserveFocus: true });
  });
}
