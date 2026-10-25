// The host side of the panes: the properties pane's questions (would this
// value be accepted, where is the element declared), a diagram in an editor
// tab of its own ("<model> — <diagram>", beside the others or in a new
// window by VS Code's own command), the selection kept the same in every
// tab and in the text, and the model tree's filter, drag and context menu.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { childTools, isPanesMessage, modelName, PanesToHost, PanesToPage, tabTitle } from '../pure/panes';
import { elementLabel, ElementRecord } from '../pure/workbench';
import { applyRequests, editText, refusalMessage } from './edits';
import { onDidSelectElement, WorkbenchPanel } from './panel';
import { snapshotOf } from './snapshot';
import { arcTextOf, documentDir } from './source';
import { locateElement, ModelItem, ModelTreeProvider } from './tree';
import { clearHighlight, highlightElement } from './go-to-source'; // nav hook

export { isPanesMessage };

/** The view type of a diagram's own tab (a webview panel, titled by the diagram). */
export const DIAGRAM_TAB = 'arclang.workbench.diagram';

let extension: vscode.ExtensionContext | null = null;
/** The diagram each tab of its own shows. */
const pins = new WeakMap<WorkbenchPanel, { view: string; title: string }>();

/** A question of a workbench page's panes, answered on that page. */
export async function handlePanesMessage(panel: WorkbenchPanel, message: PanesToHost): Promise<void> {
  const document = panel.documentOf;
  const reply = (answer: PanesToPage) => panel.postPanes(answer);
  switch (message.type) {
    case 'panes-hello': {
      const pin = pins.get(panel);
      if (pin) reply({ type: 'panes-pin', view: pin.view, title: pin.title });
      return;
    }
    case 'panes-validate': {
      const source = await arcTextOf(document);
      if (!source.ok) return reply({ type: 'panes-validated', token: message.token, ok: false, error: source.error });
      const name = path.basename(document.uri.fsPath).replace(/\.sysml$/i, '.arc') || 'model.arc';
      const outcome = await editText(source.text, message.requests, name, documentDir(document));
      return reply({ type: 'panes-validated', token: message.token, ok: outcome.ok, error: outcome.ok ? undefined : outcome.error });
    }
    case 'panes-locate': {
      const element = await elementOf(document, message.id);
      const range = element ? await locateElement(document, element) : null;
      return reply({ type: 'panes-located', id: message.id, line: range ? range.start.line + 1 : null, text: range ? document.lineAt(range.start.line).text.trim() : undefined });
    }
    case 'panes-open-tab':
      await openDiagramTab(document, message.view);
      return;
    case 'panes-references': {
      const element = await elementOf(document, message.id);
      if (element) await showReferences(document, element);
      return;
    }
    case 'panes-tree':
      await vscode.commands.executeCommand('arclang.canvas.tree', { elementId: message.id });
      return;
    default:
      return;
  }
}

async function elementOf(document: vscode.TextDocument, id: string): Promise<ElementRecord | null> {
  const records = (await snapshotOf(document)).elements?.elements ?? [];
  return records.find((e) => e.id === id) ?? records.find((e) => e.name === id) ?? null;
}

/** A diagram in an editor tab of its own: the same workbench, showing that diagram only. */
export async function openDiagramTab(document: vscode.TextDocument, graphId: string, column: vscode.ViewColumn = vscode.ViewColumn.Beside): Promise<WorkbenchPanel | null> {
  if (!extension) return null;
  const snapshot = await snapshotOf(document);
  const graph = (snapshot.diagrams?.graphs ?? []).find((g) => g.id === graphId);
  if (!graph) {
    vscode.window.showInformationMessage(`ArcLang: no diagram ${graphId} in this model.`);
    return null;
  }
  const title = tabTitle(modelName(snapshot, path.basename(document.uri.fsPath)), graph.arc.title);
  const webview = vscode.window.createWebviewPanel(DIAGRAM_TAB, title, { viewColumn: column, preserveFocus: false }, { enableScripts: true, retainContextWhenHidden: true, enableFindWidget: false });
  const panel = WorkbenchPanel.adopt(extension, document, webview);
  pins.set(panel, { view: graph.id, title });
  return panel;
}

/** The places that name an element, in the editor's references view. */
async function showReferences(document: vscode.TextDocument, element: ElementRecord): Promise<void> {
  const declaration = await locateElement(document, element);
  const text = document.getText();
  const locations: vscode.Location[] = [];
  for (const needle of new Set([`"${element.id}"`, element.name && element.name !== element.id ? `"${element.name}"` : ''])) {
    if (!needle) continue;
    for (let at = text.indexOf(needle); at >= 0; at = text.indexOf(needle, at + needle.length)) {
      const range = new vscode.Range(document.positionAt(at + 1), document.positionAt(at + needle.length - 1));
      if (declaration && declaration.contains(range.start) && declaration.start.line === range.start.line) continue;
      locations.push(new vscode.Location(document.uri, range));
    }
  }
  if (locations.length === 0) {
    vscode.window.showInformationMessage(`ArcLang: nothing in this file refers to ${elementLabel(element)}.`);
    return;
  }
  const at = declaration?.start ?? locations[0].range.start;
  await vscode.commands.executeCommand('editor.action.showReferences', document.uri, at, locations);
}

function elementItem(item: unknown): Extract<ModelItem, { type: 'element' }> | null {
  return typeof item === 'object' && item !== null && (item as ModelItem).type === 'element' ? (item as Extract<ModelItem, { type: 'element' }>) : null;
}

/** The workbench page a tree command acts on: the one in front, else the first of the document. */
function pageOf(uri: vscode.Uri): WorkbenchPanel | undefined {
  const active = WorkbenchPanel.activePanel();
  return active && active.documentOf.uri.toString() === uri.toString() ? active : WorkbenchPanel.for(uri);
}

/** The commands and the behaviour of the panes; `tree` and `view` are the side bar's model tree. */
export function registerPanes(context: vscode.ExtensionContext, tree: ModelTreeProvider, view: vscode.TreeView<ModelItem>): void {
  extension = context;
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));

  // --- Diagram tabs -------------------------------------------------------------
  register('arclang.diagram.openInNewTab', async (item?: unknown) => {
    const picked = item as Extract<ModelItem, { type: 'diagram' }> | undefined;
    if (picked?.type === 'diagram') {
      const document = await vscode.workspace.openTextDocument(picked.uri);
      await openDiagramTab(document, picked.graph.id);
      return;
    }
    const document = WorkbenchPanel.activeDocument() ?? tree.shown ?? vscode.window.activeTextEditor?.document;
    if (!document) return;
    const graphs = (await snapshotOf(document)).diagrams?.graphs ?? [];
    const choice = await vscode.window.showQuickPick(
      graphs.map((graph) => ({ label: `$(graph) ${graph.arc.title}`, description: graph.arc.view.toUpperCase(), graph })),
      { placeHolder: 'Open a diagram in a tab of its own' },
    );
    if (choice) await openDiagramTab(document, choice.graph.id);
  });

  // --- The selection, the same in every tab and in the text -----------------------
  context.subscriptions.push(
    onDidSelectElement(async ({ uri, id }) => {
      const panels = WorkbenchPanel.all(uri);
      // The page the selection came from already holds it; the others follow.
      for (const other of panels) if (other.selectedId !== id) other.select(id);
      if (!id || panels.length === 0) {
        clearHighlight(); // nav hook: the highlight goes with the selection
        return;
      }
      // In full screen the text is hidden behind the maximized group: showing it there would
      // bring the other groups back and take the full screen away from under the pointer.
      if (panels.some((panel) => panel.fullScreen)) return;
      // A selection made in the text (the cursor) stays the reader's: the text is followed only from a page.
      if (vscode.window.activeTextEditor?.document.uri.toString() === uri.toString()) return;
      // nav hook: a click only selects. The text follows by a highlight when it is on screen, and is never opened, focused or selected.
      await highlightElement(panels[0].documentOf, await elementOf(panels[0].documentOf, id));
    }),
  );

  // --- The tree's filter ----------------------------------------------------------
  const setFiltered = (query: string) => {
    tree.setFilter(query);
    void vscode.commands.executeCommand('setContext', 'arclang.treeFiltered', !!query.trim());
    view.message = query.trim() ? `Filtered by “${query.trim()}”` : undefined;
  };
  register('arclang.model.filter', async () => {
    const input = vscode.window.createInputBox();
    input.title = 'Filter the model tree';
    input.placeholder = 'name, id or kind (e.g. camera, LC-, function)';
    input.value = tree.filter;
    input.onDidChangeValue((value) => setFiltered(value));
    input.onDidAccept(() => input.hide());
    input.onDidHide(() => input.dispose());
    input.show();
  });
  register('arclang.model.clearFilter', () => setFiltered(''));

  // --- The tree's context menu ----------------------------------------------------
  register('arclang.model.showInDiagram', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    let page = pageOf(element.uri);
    if (!page) {
      await WorkbenchPanel.open(document);
      page = pageOf(element.uri);
    }
    page?.postPanes({ type: 'panes-drop', id: element.node.element.id });
  });
  register('arclang.model.references', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    await showReferences(document, element.node.element);
  });
  register('arclang.model.newChild', async (item?: unknown) => {
    const element = elementItem(item);
    if (!element) return;
    const record = element.node.element;
    const tools = childTools(record.kind);
    if (tools.length === 0) {
      vscode.window.showInformationMessage(`ArcLang: nothing is declared inside a ${record.kind}.`);
      return;
    }
    const tool = tools.length === 1 ? tools[0] : await vscode.window.showQuickPick(tools.map((t) => ({ ...t, description: t.keyword })), { placeHolder: `New element in ${elementLabel(record)}` });
    if (!tool) return;
    const name = await vscode.window.showInputBox({ prompt: `Name of the new ${tool.label.toLowerCase()} in ${elementLabel(record)}`, validateInput: (v) => (v.trim() ? null : 'a name is needed') });
    if (!name) return;
    const document = await vscode.workspace.openTextDocument(element.uri);
    const page = pageOf(element.uri);
    const request = { op: 'add' as const, keyword: tool.keyword, name: name.trim(), container: record.id };
    // Through the page when one is open: one step of its history (⌘Z on the canvas).
    if (page) return page.submitEdit([request], `new ${tool.keyword} ${name.trim()}`);
    const outcome = await applyRequests(document, [request]);
    if (!outcome.ok) vscode.window.showWarningMessage(refusalMessage(`new ${tool.keyword}`, outcome));
  });
}
