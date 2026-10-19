// The model tree in the side bar: the diagrams of the active model, then its
// elements nested by ownership and grouped by Arcadia layer, from the
// compiler's snapshot. A click selects the element in the views and the
// text; a double-click opens the diagram that draws it; the context menu
// renames, deletes, goes to the source or asks the assistant.
import * as vscode from 'vscode';
import { ElementRecord, elementLabel, Metamodel, Snapshot, TreeNode, treeByLayer, ViewerGraph } from '../pure/workbench';
import { metamodelJson, onDidChangeSnapshot, snapshotOf } from './snapshot';
import { isModelDocument } from './source';

export type ModelItem =
  | { type: 'diagrams'; uri: vscode.Uri; graphs: ViewerGraph[] }
  | { type: 'diagram'; uri: vscode.Uri; graph: ViewerGraph }
  | { type: 'layer'; layer: string; label: string; roots: TreeNode[]; uri: vscode.Uri }
  | { type: 'element'; node: TreeNode; uri: vscode.Uri; parent: ModelItem }
  | { type: 'message'; text: string };

const ICONS: Record<string, string> = {
  Model: 'file',
  Operational: 'organization',
  System: 'symbol-method',
  Logical: 'symbol-class',
  Physical: 'server',
  Epbs: 'package',
  Data: 'database',
  Transverse: 'references',
};

const KIND_ICONS: Record<string, string> = {
  Actor: 'person',
  SystemActor: 'person',
  OperationalEntity: 'organization',
  OperationalActivity: 'run',
  OperationalCapability: 'star',
  Requirement: 'checklist',
  Mission: 'target',
  Capability: 'star',
  SystemFunction: 'symbol-method',
  LogicalFunction: 'symbol-method',
  PhysicalFunction: 'symbol-method',
  FunctionPort: 'plug',
  ComponentPort: 'plug',
  PhysicalPort: 'plug',
  SystemComponent: 'symbol-class',
  LogicalComponent: 'symbol-class',
  BehaviorComponent: 'symbol-class',
  HardwareComponent: 'circuit-board',
  PhysicalNode: 'server',
  LogicalInterface: 'symbol-interface',
  Class: 'symbol-structure',
  Enumeration: 'symbol-enum',
  DataType: 'symbol-field',
  ExchangeItem: 'symbol-field',
  StateMachine: 'circuit-board',
  State: 'circle-outline',
  Scenario: 'list-ordered',
  Diagram: 'graph',
  Hazard: 'warning',
  Threat: 'shield',
  Model: 'file',
  Type: 'symbol-ruler',
};

/** The viewpoints in method order, with their names. */
const VIEW_NAMES: Record<string, string> = {
  oab: 'Operational',
  sab: 'System',
  lab: 'Logical',
  pab: 'Physical',
  cap: 'Capabilities',
  cdb: 'Data',
  pbs: 'Product breakdown',
  msm: 'Modes and states',
  es: 'Scenarios',
};

export class ModelTreeProvider implements vscode.TreeDataProvider<ModelItem> {
  private readonly changed = new vscode.EventEmitter<ModelItem | undefined>();
  readonly onDidChangeTreeData = this.changed.event;
  private document: vscode.TextDocument | null = null;
  private metamodel: Metamodel | null = null;
  /** The items of the last build, by element uuid, so the tree can reveal one. */
  private items = new Map<string, Extract<ModelItem, { type: 'element' }>>();

  constructor(context: vscode.ExtensionContext) {
    context.subscriptions.push(
      vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor && isModelDocument(editor.document)) this.follow(editor.document);
      }),
      onDidChangeSnapshot((uri) => {
        if (this.document && uri.toString() === this.document.uri.toString()) this.changed.fire(undefined);
      }),
      vscode.workspace.onDidCloseTextDocument((document) => {
        if (this.document && document.uri.toString() === this.document.uri.toString()) {
          this.document = null;
          this.changed.fire(undefined);
        }
      }),
    );
    const active = vscode.window.activeTextEditor;
    if (active && isModelDocument(active.document)) this.follow(active.document);
    void metamodelJson().then((m) => {
      this.metamodel = m;
      this.changed.fire(undefined);
    });
  }

  /** The document the tree shows. */
  get shown(): vscode.TextDocument | null {
    return this.document;
  }

  follow(document: vscode.TextDocument): void {
    if (this.document && this.document.uri.toString() === document.uri.toString()) return;
    this.document = document;
    this.items.clear();
    this.changed.fire(undefined);
  }

  refresh(): void {
    this.changed.fire(undefined);
  }

  /** The tree item of an element, once the tree has built it. */
  itemOf(uuid: string): Extract<ModelItem, { type: 'element' }> | undefined {
    return this.items.get(uuid);
  }

  getParent(item: ModelItem): ModelItem | undefined {
    return item.type === 'element' ? item.parent : undefined;
  }

  getTreeItem(item: ModelItem): vscode.TreeItem {
    switch (item.type) {
      case 'message': {
        const tree = new vscode.TreeItem(item.text, vscode.TreeItemCollapsibleState.None);
        tree.iconPath = new vscode.ThemeIcon('info');
        return tree;
      }
      case 'diagrams': {
        const tree = new vscode.TreeItem('Diagrams', vscode.TreeItemCollapsibleState.Collapsed);
        tree.iconPath = new vscode.ThemeIcon('type-hierarchy');
        tree.description = `${item.graphs.length}`;
        tree.contextValue = 'arclang.diagrams';
        return tree;
      }
      case 'diagram': {
        const view = item.graph.arc.view;
        const tree = new vscode.TreeItem(item.graph.arc.title, vscode.TreeItemCollapsibleState.None);
        tree.description = VIEW_NAMES[view] ?? view;
        tree.iconPath = new vscode.ThemeIcon('graph');
        tree.contextValue = 'arclang.diagram';
        tree.command = { command: 'arclang.openDiagram', title: 'Open the diagram', arguments: [item] };
        return tree;
      }
      case 'layer': {
        const tree = new vscode.TreeItem(item.label, item.layer === 'Model' ? vscode.TreeItemCollapsibleState.None : vscode.TreeItemCollapsibleState.Expanded);
        tree.iconPath = new vscode.ThemeIcon(ICONS[item.layer] ?? 'folder');
        tree.contextValue = 'arclang.layer';
        tree.description = item.layer === 'Model' ? '' : `${countNodes(item.roots)}`;
        tree.tooltip = item.layer === 'Model' ? 'The model' : `${item.label}: click opens its diagram`;
        if (item.layer === 'Model' && item.roots[0]) {
          tree.command = { command: 'arclang.model.select', title: 'Select', arguments: [{ type: 'element', node: item.roots[0], uri: item.uri, parent: item }] };
        } else {
          tree.command = { command: 'arclang.openLayer', title: 'Open the diagram of this layer', arguments: [item] };
        }
        return tree;
      }
      case 'element': {
        const element = item.node.element;
        const tree = new vscode.TreeItem(
          element.name || element.id,
          item.node.children.length > 0 ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None,
        );
        tree.id = `${item.uri.toString()}#${element.uuid}`;
        tree.description = element.name && element.name !== element.id ? element.id : element.kind;
        tree.tooltip = new vscode.MarkdownString(`**${element.kind}** \`${element.id}\`\n\n${String(element.attributes.description ?? '')}`);
        tree.iconPath = new vscode.ThemeIcon(KIND_ICONS[element.kind] ?? 'symbol-misc');
        tree.contextValue = 'arclang.element';
        tree.command = { command: 'arclang.model.select', title: 'Select', arguments: [item] };
        return tree;
      }
    }
  }

  async getChildren(item?: ModelItem): Promise<ModelItem[]> {
    if (!this.document) return [{ type: 'message', text: 'Open a .arc or .sysml file to see its model.' }];
    if (!item) {
      if (!this.metamodel) this.metamodel = await metamodelJson();
      const snapshot = await snapshotOf(this.document);
      if (!snapshot.elements) {
        const reason = snapshot.errors[0] ?? 'the model does not compile';
        return [{ type: 'message', text: `No model: ${firstLine(reason)}` }];
      }
      const uri = this.document.uri;
      this.items.clear();
      const graphs = snapshot.diagrams?.graphs ?? [];
      const layers: ModelItem[] = treeByLayer(snapshot.elements.elements, this.metamodel).map((group) => ({
        type: 'layer' as const,
        layer: group.layer,
        label: group.label,
        roots: group.roots,
        uri,
      }));
      return graphs.length ? [{ type: 'diagrams', uri, graphs }, ...layers] : layers;
    }
    if (item.type === 'diagrams') return item.graphs.map((graph) => ({ type: 'diagram' as const, uri: item.uri, graph }));
    if (item.type === 'layer') return item.roots.map((node) => this.remember({ type: 'element' as const, node, uri: item.uri, parent: item }));
    if (item.type === 'element') return item.node.children.map((node) => this.remember({ type: 'element' as const, node, uri: item.uri, parent: item }));
    return [];
  }

  private remember(item: Extract<ModelItem, { type: 'element' }>): Extract<ModelItem, { type: 'element' }> {
    this.items.set(item.node.element.uuid, item);
    return item;
  }

  /** The element records of the shown model, for commands. */
  async elements(): Promise<ElementRecord[]> {
    if (!this.document) return [];
    const snapshot: Snapshot = await snapshotOf(this.document);
    return snapshot.elements?.elements ?? [];
  }
}

function countNodes(nodes: TreeNode[]): number {
  return nodes.reduce((sum, node) => sum + 1 + countNodes(node.children), 0);
}

function firstLine(text: string): string {
  return text.split('\n')[0].slice(0, 120);
}

/** Flattened document symbols of a document, from the language server. */
async function documentSymbols(uri: vscode.Uri): Promise<vscode.DocumentSymbol[]> {
  const symbols = (await vscode.commands.executeCommand<vscode.DocumentSymbol[]>('vscode.executeDocumentSymbolProvider', uri)) ?? [];
  const flat: vscode.DocumentSymbol[] = [];
  const walk = (list: vscode.DocumentSymbol[]) => {
    for (const symbol of list) {
      flat.push(symbol);
      if (symbol.children?.length) walk(symbol.children);
    }
  };
  walk(symbols);
  return flat;
}

/** The id a symbol's detail carries (`component LC-001`), when it writes one. */
function symbolId(symbol: vscode.DocumentSymbol): string | null {
  const at = symbol.detail.indexOf(' ');
  return at > 0 ? symbol.detail.slice(at + 1) : null;
}

/** The range of an element's declaration in its document, when the server or the text knows it. */
export async function locateElement(document: vscode.TextDocument, element: Pick<ElementRecord, 'id' | 'name'>): Promise<vscode.Range | null> {
  const symbols = await documentSymbols(document.uri);
  // The server names a symbol by the written name and details it `keyword id`.
  const matches = (symbol: vscode.DocumentSymbol) => symbol.name === element.id || symbol.name === element.name || symbolId(symbol) === element.id;
  const found = symbols.find(matches);
  if (found) return found.selectionRange ?? found.range;
  // Without a symbol: the first line that spells the id or the name in quotes.
  // ArcLang quotes with ", the SysML v2 export writes <'id'> and 'name'.
  const text = document.getText();
  for (const needle of [`"${element.id}"`, `<'${element.id}'>`, `"${element.name}"`, `'${element.name}'`, element.id]) {
    const at = needle ? text.indexOf(needle) : -1;
    if (at >= 0) return new vscode.Range(document.positionAt(at), document.positionAt(at + needle.length));
  }
  return null;
}

/** Show the element's declaration in a text editor beside the views. */
export async function revealElement(
  document: vscode.TextDocument,
  element: Pick<ElementRecord, 'id' | 'name'>,
  options: { preserveFocus?: boolean } = {},
): Promise<boolean> {
  const range = await locateElement(document, element);
  if (!range) {
    vscode.window.setStatusBarMessage(`ArcLang: ${elementLabel({ ...element, kind: '' })} is not declared in this file (it may come from an import)`, 4000);
    return false;
  }
  const column = vscode.window.visibleTextEditors.find((e) => e.document.uri.toString() === document.uri.toString())?.viewColumn ?? vscode.ViewColumn.One;
  const editor = await vscode.window.showTextDocument(document, { viewColumn: column, preserveFocus: options.preserveFocus ?? false, preview: false });
  editor.selection = new vscode.Selection(range.start, range.end);
  editor.revealRange(range, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
  return true;
}

/** The element whose declaration contains the position, by the server's symbols. */
export async function elementAt(document: vscode.TextDocument, position: vscode.Position, elements: ElementRecord[]): Promise<ElementRecord | null> {
  const symbols = await documentSymbols(document.uri);
  const containing = symbols.filter((s) => s.range.contains(position));
  // The innermost declaration wins.
  containing.sort((a, b) => (a.range.contains(b.range) ? 1 : -1));
  for (const symbol of containing) {
    const id = symbolId(symbol);
    const found = elements.find((e) => e.id === symbol.name || e.name === symbol.name || (id !== null && e.id === id));
    if (found) return found;
  }
  if (symbols.length > 0) return null;
  // Without a language server (SysML v2 text): the nearest declaration above
  // the cursor that spells an id `<'ID'>` or a name `'Name'` of the model.
  const byId = new Map(elements.map((e) => [e.id, e]));
  const byName = new Map(elements.map((e) => [e.name, e]));
  for (let line = position.line; line >= 0 && line > position.line - 400; line--) {
    const text = document.lineAt(line).text;
    for (const match of text.matchAll(/<'([^']+)'>/g)) {
      const found = byId.get(match[1]);
      if (found) return found;
    }
    for (const match of text.matchAll(/\b(?:part|port|action|item|attribute|requirement|state|package|connection|interface|flow|allocation|use case|occurrence)\s+(?:def\s+)?(?:<'[^']+'>\s+)?(?:'([^']+)'|([A-Za-z_][A-Za-z0-9_]*))/g)) {
      const found = byName.get(match[1] ?? match[2]) ?? byId.get(match[1] ?? match[2]);
      if (found) return found;
    }
  }
  return null;
}

/** The graphs that draw an element, by its id (nodes, ports, lifelines). */
export function graphsDrawing(graphs: ViewerGraph[], element: Pick<ElementRecord, 'id'>): ViewerGraph[] {
  const wanted = new Set([element.id, element.id.replace(/\.([^.]+)$/, '::$1')]);
  const walk = (nodes: ViewerGraph['children']): boolean =>
    (nodes ?? []).some((node) => wanted.has(node.id) || node.id.endsWith(`.${element.id}`) || (node.ports ?? []).some((p) => wanted.has(p.id)) || walk(node.children));
  return graphs.filter((graph) => walk(graph.children));
}
