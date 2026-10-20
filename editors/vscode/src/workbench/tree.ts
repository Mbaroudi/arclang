// The model tree in the side bar: the model, then its Arcadia layers, each
// with its diagrams first and then its elements nested by ownership, from
// the compiler's snapshot. Every node has the icon of its kind (the
// diagram's colours), the findings on it as a badge, and can be filtered,
// dragged onto a diagram, renamed, deleted, given a child; a click selects
// the element in the views and the text, a diagram opens on click.
import * as fs from 'node:fs';
import * as vscode from 'vscode';
import { familyOf } from '../pure/kind-icons';
import { Counts, countsBadge, diagramsByLayer, dropPayload, filterNodes, findingCounts, subtreeCounts, TREE_MIME } from '../pure/panes';
import { ElementRecord, elementLabel, LAYER_LABELS, LAYERS, Metamodel, Snapshot, TreeNode, treeByLayer, ViewerGraph } from '../pure/workbench';
import { metamodelJson, onDidChangeSnapshot, snapshotOf } from './snapshot';
import { isModelDocument } from './source';

export type ModelItem =
  | { type: 'diagrams'; uri: vscode.Uri; graphs: ViewerGraph[] }
  | { type: 'diagram'; uri: vscode.Uri; graph: ViewerGraph; parent?: ModelItem }
  | { type: 'layer'; layer: string; label: string; roots: TreeNode[]; uri: vscode.Uri; graphs?: ViewerGraph[] }
  | { type: 'element'; node: TreeNode; uri: vscode.Uri; parent: ModelItem }
  | { type: 'message'; text: string };

/** The viewpoints with their names, as a diagram node describes itself. */
const VIEW_NAMES: Record<string, string> = {
  oab: 'Operational',
  sab: 'System',
  lab: 'Logical',
  pab: 'Physical',
  cap: 'Capabilities',
  cdb: 'Data',
  pbs: 'Product breakdown',
  msm: 'Modes and states',
  es: 'Scenario',
};

/** The scheme of the uris that carry a node's findings to the decoration provider. */
const DECORATION_SCHEME = 'arclang-node';

/** The kind icons, files of the extension (media/kinds, written by `npm run icons`). */
class IconFiles {
  private readonly dir: vscode.Uri | null;

  constructor(context: vscode.ExtensionContext) {
    const dir = context.extensionUri ? vscode.Uri.joinPath(context.extensionUri, 'media', 'kinds') : null;
    this.dir = dir && fs.existsSync(dir.fsPath) ? dir : null;
  }

  of(kind: string): vscode.Uri | vscode.ThemeIcon {
    return this.dir ? vscode.Uri.joinPath(this.dir, `${familyOf(kind)}.svg`) : new vscode.ThemeIcon('symbol-misc');
  }
}

/** The findings of a node, shown as a badge and a colour at the end of its line. */
class FindingDecorations implements vscode.FileDecorationProvider {
  provideFileDecoration(uri: vscode.Uri): vscode.FileDecoration | undefined {
    if (uri.scheme !== DECORATION_SCHEME) return undefined;
    const query = new URLSearchParams(uri.query);
    const counts: Counts = { errors: Number(query.get('e') ?? 0), warnings: Number(query.get('w') ?? 0), infos: Number(query.get('i') ?? 0) };
    const own = query.get('own') === '1';
    const badge = countsBadge(counts);
    if (!badge.severity) return undefined;
    const tooltip = [`${counts.errors} error${counts.errors === 1 ? '' : 's'}`, `${counts.warnings} warning${counts.warnings === 1 ? '' : 's'}`, counts.infos ? `${counts.infos} advisory` : ''].filter(Boolean).join(', ') + (own ? '' : ' below');
    const color = badge.severity === 'error' ? new vscode.ThemeColor('list.errorForeground') : badge.severity === 'warning' ? new vscode.ThemeColor('list.warningForeground') : undefined;
    return { badge: own ? badge.text || undefined : undefined, color, tooltip };
  }
}

export class ModelTreeProvider implements vscode.TreeDataProvider<ModelItem>, vscode.TreeDragAndDropController<ModelItem> {
  private readonly changed = new vscode.EventEmitter<ModelItem | undefined>();
  readonly onDidChangeTreeData = this.changed.event;
  readonly dragMimeTypes = [TREE_MIME, 'text/uri-list', 'text/plain'];
  readonly dropMimeTypes: string[] = [];
  private document: vscode.TextDocument | null = null;
  private metamodel: Metamodel | null = null;
  /** The items of the last build, by element uuid, so the tree can reveal one. */
  private items = new Map<string, Extract<ModelItem, { type: 'element' }>>();
  private children = new Map<ModelItem, ModelItem[]>();
  private roots: ModelItem[] | null = null;
  private counts = new Map<string, Counts>();
  private query = '';
  private readonly icons: IconFiles;

  constructor(context: vscode.ExtensionContext) {
    this.icons = new IconFiles(context);
    context.subscriptions.push(
      vscode.window.registerFileDecorationProvider(new FindingDecorations()),
      vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor && isModelDocument(editor.document)) this.follow(editor.document);
      }),
      onDidChangeSnapshot((uri) => {
        if (this.document && uri.toString() === this.document.uri.toString()) this.refresh();
      }),
      vscode.workspace.onDidCloseTextDocument((document) => {
        if (this.document && document.uri.toString() === this.document.uri.toString()) {
          this.document = null;
          this.refresh();
        }
      }),
    );
    const active = vscode.window.activeTextEditor;
    if (active && isModelDocument(active.document)) this.follow(active.document);
    void metamodelJson().then((m) => {
      this.metamodel = m;
      this.refresh();
    });
  }

  /** The document the tree shows. */
  get shown(): vscode.TextDocument | null {
    return this.document;
  }

  /** What the tree is filtered by ('' for nothing). */
  get filter(): string {
    return this.query;
  }

  follow(document: vscode.TextDocument): void {
    if (this.document && this.document.uri.toString() === document.uri.toString()) return;
    this.document = document;
    this.refresh();
  }

  refresh(): void {
    this.roots = null;
    this.changed.fire(undefined);
  }

  /** Keep the elements whose name, id or kind holds every word (and the path to them). */
  setFilter(query: string): void {
    this.query = query.trim();
    this.refresh();
  }

  /** The number of elements the filter keeps, once built. */
  matchCount(): number {
    return this.items.size;
  }

  /** The tree item of an element, once the tree has built it. */
  itemOf(uuid: string): Extract<ModelItem, { type: 'element' }> | undefined {
    return this.items.get(uuid);
  }

  getParent(item: ModelItem): ModelItem | undefined {
    if (item.type === 'element') return item.parent;
    if (item.type === 'diagram') return item.parent;
    return undefined;
  }

  handleDrag(source: readonly ModelItem[], data: vscode.DataTransfer): void {
    const elements = source.filter((item): item is Extract<ModelItem, { type: 'element' }> => item.type === 'element');
    if (elements.length === 0) return;
    const first = elements[0];
    data.set(TREE_MIME, new vscode.DataTransferItem(JSON.stringify({ id: first.node.element.id, uri: first.uri.toString(), ids: elements.map((e) => e.node.element.id) })));
    data.set('text/uri-list', new vscode.DataTransferItem(elements.map((e) => dropPayload(e.uri.toString(), e.node.element.id)).join('\r\n')));
    data.set('text/plain', new vscode.DataTransferItem(dropPayload(first.uri.toString(), first.node.element.id)));
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
        tree.iconPath = this.icons.of('Diagram');
        tree.description = `${item.graphs.length}`;
        tree.contextValue = 'arclang.diagrams';
        return tree;
      }
      case 'diagram': {
        const view = item.graph.arc.view;
        const tree = new vscode.TreeItem(item.graph.arc.title, vscode.TreeItemCollapsibleState.None);
        tree.id = `${item.uri.toString()}#diagram:${item.graph.id}`;
        tree.description = VIEW_NAMES[view] ?? view;
        tree.tooltip = `${VIEW_NAMES[view] ?? view} diagram: click opens it; right-click opens it in a tab of its own`;
        tree.iconPath = this.icons.of('Diagram');
        tree.contextValue = 'arclang.diagram';
        tree.command = { command: 'arclang.openDiagram', title: 'Open the diagram', arguments: [item] };
        return tree;
      }
      case 'layer': {
        const collapsed = item.layer === 'Model' ? vscode.TreeItemCollapsibleState.None : this.query ? vscode.TreeItemCollapsibleState.Expanded : vscode.TreeItemCollapsibleState.Expanded;
        const tree = new vscode.TreeItem(item.label, collapsed);
        tree.id = `${item.uri.toString()}#layer:${item.layer}${this.query ? `?${this.query}` : ''}`;
        tree.iconPath = this.icons.of(item.layer === 'Model' ? 'Model' : 'Package');
        tree.contextValue = 'arclang.layer';
        const elements = countNodes(item.roots);
        const diagrams = item.graphs?.length ?? 0;
        tree.description = item.layer === 'Model' ? '' : [elements ? `${elements}` : '', diagrams ? `${diagrams} diagram${diagrams > 1 ? 's' : ''}` : ''].filter(Boolean).join(' · ');
        tree.tooltip = item.layer === 'Model' ? 'The model' : `${item.label}: click opens its diagram`;
        const counts = item.roots.reduce((sum, node) => add(sum, subtreeCounts(node, this.counts)), { errors: 0, warnings: 0, infos: 0 });
        tree.resourceUri = decorationUri(`layer-${item.layer}`, counts, false);
        if (item.layer === 'Model' && item.roots[0]) {
          tree.command = { command: 'arclang.model.select', title: 'Select', arguments: [{ type: 'element', node: item.roots[0], uri: item.uri, parent: item }] };
          tree.resourceUri = decorationUri('model', this.counts.get(item.roots[0].element.uuid) ?? { errors: 0, warnings: 0, infos: 0 }, true);
        } else {
          tree.command = { command: 'arclang.openLayer', title: 'Open the diagram of this layer', arguments: [item] };
        }
        return tree;
      }
      case 'element': {
        const element = item.node.element;
        const open = this.query && item.node.children.length > 0 ? vscode.TreeItemCollapsibleState.Expanded : item.node.children.length > 0 ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None;
        const tree = new vscode.TreeItem(element.name || element.id, open);
        tree.id = `${item.uri.toString()}#${element.uuid}${this.query ? `?${this.query}` : ''}`;
        tree.description = element.name && element.name !== element.id ? element.id : '';
        const own = this.counts.get(element.uuid) ?? { errors: 0, warnings: 0, infos: 0 };
        const below = subtreeCounts(item.node, this.counts);
        const findings = own.errors + own.warnings ? `\n\n$(warning) ${own.errors} error(s), ${own.warnings} warning(s)` : '';
        tree.tooltip = new vscode.MarkdownString(`**${element.kind}** \`${element.id}\`\n\n${String(element.attributes.description ?? '')}${findings}`, true);
        tree.iconPath = this.icons.of(element.kind);
        tree.resourceUri = decorationUri(element.uuid, own.errors + own.warnings ? own : below, own.errors + own.warnings > 0);
        tree.contextValue = 'arclang.element';
        tree.command = { command: 'arclang.model.select', title: 'Select', arguments: [item] };
        return tree;
      }
    }
  }

  async getChildren(item?: ModelItem): Promise<ModelItem[]> {
    if (!this.document) return [{ type: 'message', text: 'Open a .arc or .sysml file to see its model.' }];
    if (!item) {
      if (!this.roots) this.roots = await this.build(this.document);
      return this.roots;
    }
    return this.children.get(item) ?? [];
  }

  /** The whole tree at once: the reveal of a selection needs every node, built or not. */
  private async build(document: vscode.TextDocument): Promise<ModelItem[]> {
    if (!this.metamodel) this.metamodel = await metamodelJson();
    const snapshot = await snapshotOf(document);
    this.items = new Map();
    this.children = new Map();
    if (!snapshot.elements) {
      const reason = snapshot.errors[0] ?? 'the model does not compile';
      return [{ type: 'message', text: `No model: ${firstLine(reason)}` }];
    }
    const uri = document.uri;
    this.counts = findingCounts(snapshot.elements.elements, snapshot.findings ?? []);
    const graphs = this.query ? (snapshot.diagrams?.graphs ?? []).filter((g) => g.arc.title.toLowerCase().includes(this.query.toLowerCase())) : (snapshot.diagrams?.graphs ?? []);
    const byLayer = diagramsByLayer(graphs);
    const groups = treeByLayer(snapshot.elements.elements, this.metamodel);
    const known = new Set(groups.map((g) => g.layer));
    // A layer drawn but holding no element of its own still lists its diagrams.
    for (const layer of byLayer.keys()) if (!known.has(layer)) groups.push({ layer, label: LAYER_LABELS[layer] ?? layer, roots: [] });
    const order = (layer: string) => (layer === 'Model' ? -1 : (LAYERS as readonly string[]).indexOf(layer) < 0 ? LAYERS.length : (LAYERS as readonly string[]).indexOf(layer));
    groups.sort((a, b) => order(a.layer) - order(b.layer));
    const roots: ModelItem[] = [];
    for (const group of groups) {
      const nodes = group.layer === 'Model' ? group.roots : filterNodes(group.roots, this.query);
      const own = group.layer === 'Model' ? [] : (byLayer.get(group.layer) ?? []);
      if (this.query && group.layer !== 'Model' && nodes.length === 0 && own.length === 0) continue;
      const layer: Extract<ModelItem, { type: 'layer' }> = { type: 'layer', layer: group.layer, label: group.label, roots: nodes, uri, graphs: own };
      const diagrams: ModelItem[] = own.map((graph) => ({ type: 'diagram' as const, uri, graph, parent: layer }));
      this.children.set(layer, [...diagrams, ...nodes.map((node) => this.element(node, uri, layer))]);
      roots.push(layer);
    }
    return roots;
  }

  private element(node: TreeNode, uri: vscode.Uri, parent: ModelItem): Extract<ModelItem, { type: 'element' }> {
    const item = { type: 'element' as const, node, uri, parent };
    this.items.set(node.element.uuid, item);
    this.children.set(item, node.children.map((child) => this.element(child, uri, item)));
    return item;
  }

  /** The element records of the shown model, for commands. */
  async elements(): Promise<ElementRecord[]> {
    if (!this.document) return [];
    const snapshot: Snapshot = await snapshotOf(this.document);
    return snapshot.elements?.elements ?? [];
  }
}

function add(a: Counts, b: Counts): Counts {
  return { errors: a.errors + b.errors, warnings: a.warnings + b.warnings, infos: a.infos + b.infos };
}

function decorationUri(key: string, counts: Counts, own: boolean): vscode.Uri {
  return vscode.Uri.from({ scheme: DECORATION_SCHEME, path: `/${key}`, query: `e=${counts.errors}&w=${counts.warnings}&i=${counts.infos}&own=${own ? 1 : 0}` });
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
