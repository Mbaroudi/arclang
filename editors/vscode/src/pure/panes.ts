// The model tree, the diagram tabs and the messages of the panes, without
// VS Code: where diagrams go in the tree, what a filter keeps, the findings
// counted on each node, what a node dropped on a diagram writes, the kinds a
// node may be given as children, the title of a diagram's own tab.
import { EditRequest, ElementRecord, Finding, namedDiagramOf, PALETTE, showsLiteral, Snapshot, TreeNode, ViewerGraph } from './workbench';

// --- Messages between the page and the host -------------------------------------

/** What the panes of the page ask of the host. */
export type PanesToHost =
  /** The page is up: a diagram's own tab is told which diagram it shows. */
  | { type: 'panes-hello' }
  /** Would these edits be accepted? Nothing is written. */
  | { type: 'panes-validate'; token: number; requests: EditRequest[] }
  /** Where an element is declared. */
  | { type: 'panes-locate'; id: string }
  /** Open a diagram in a tab of its own. */
  | { type: 'panes-open-tab'; view: string }
  /** The references to an element, in the editor's references view. */
  | { type: 'panes-references'; id: string }
  /** Show an element in the side bar's model tree. */
  | { type: 'panes-tree'; id: string };

/** What the host answers. */
export type PanesToPage =
  | { type: 'panes-validated'; token: number; ok: boolean; error?: string }
  | { type: 'panes-located'; id: string; line: number | null; text?: string }
  /** This page shows one diagram only (a diagram's own tab). */
  | { type: 'panes-pin'; view: string; title: string }
  /** An element of the tree dropped on the page: show it on the diagram on screen. */
  | { type: 'panes-drop'; id: string };

export function isPanesMessage(message: unknown): message is PanesToHost {
  const type = (message as { type?: unknown } | null)?.type;
  return typeof type === 'string' && type.startsWith('panes-');
}

// --- Tabs ---------------------------------------------------------------------------

/** The model's name: its `model` element, else the file without its extension. */
export function modelName(snapshot: Snapshot | null, fileName: string): string {
  const model = snapshot?.elements?.elements.find((e) => e.kind === 'Model');
  return model?.name || model?.id || fileName.replace(/\.(arc|sysml)$/i, '');
}

/** A diagram's own tab: "<model> — <diagram>". */
export function tabTitle(model: string, diagram: string): string {
  return `${model} — ${diagram}`;
}

// --- The tree -------------------------------------------------------------------------

/** The layer a diagram belongs to, as the tree groups it; machines and scenarios go with the transverse elements. */
export function layerOfView(view: string): string {
  const layers: Record<string, string> = { oab: 'Operational', sab: 'System', cap: 'System', lab: 'Logical', pab: 'Physical', pbs: 'Epbs', cdb: 'Data' };
  return layers[view] ?? 'Transverse';
}

/** The diagrams of the model by the layer they belong to, in the order the compiler lists them. */
export function diagramsByLayer(graphs: ViewerGraph[]): Map<string, ViewerGraph[]> {
  const out = new Map<string, ViewerGraph[]>();
  for (const graph of graphs) {
    const layer = layerOfView(graph.arc.view);
    out.set(layer, [...(out.get(layer) ?? []), graph]);
  }
  return out;
}

/** Whether an element answers a filter: its name, id or kind contains every word, any case. */
export function matchesFilter(element: Pick<ElementRecord, 'id' | 'name' | 'kind'>, query: string): boolean {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  const text = `${element.name} ${element.id} ${element.kind}`.toLowerCase();
  return words.every((w) => text.includes(w));
}

/** The nodes that match, with the ancestors that lead to them (their other children left out). */
export function filterNodes(nodes: TreeNode[], query: string): TreeNode[] {
  if (!query.trim()) return nodes;
  const out: TreeNode[] = [];
  for (const node of nodes) {
    const children = filterNodes(node.children, query);
    if (matchesFilter(node.element, query)) out.push({ element: node.element, children: children.length ? children : node.children.filter((c) => matchesFilter(c.element, query)) });
    else if (children.length) out.push({ element: node.element, children });
  }
  return out;
}

export interface Counts {
  errors: number;
  warnings: number;
  infos: number;
}

const ZERO: Counts = { errors: 0, warnings: 0, infos: 0 };

/** The findings on each element (by uuid), the element's own. */
export function findingCounts(elements: ElementRecord[], findings: Finding[]): Map<string, Counts> {
  const byKey = new Map<string, ElementRecord>();
  for (const e of elements) {
    byKey.set(e.name, e);
    byKey.set(e.id, e);
  }
  const out = new Map<string, Counts>();
  for (const finding of findings) {
    const element = finding.element ? byKey.get(finding.element) : undefined;
    if (!element) continue;
    const counts = out.get(element.uuid) ?? { ...ZERO };
    const severity = finding.severity.toLowerCase();
    out.set(element.uuid, {
      errors: counts.errors + (severity === 'error' ? 1 : 0),
      warnings: counts.warnings + (severity === 'warning' ? 1 : 0),
      infos: counts.infos + (severity === 'error' || severity === 'warning' ? 0 : 1),
    });
  }
  return out;
}

/** A node's findings with those of everything under it. */
export function subtreeCounts(node: TreeNode, own: Map<string, Counts>): Counts {
  return node.children.reduce((sum, child) => {
    const c = subtreeCounts(child, own);
    return { errors: sum.errors + c.errors, warnings: sum.warnings + c.warnings, infos: sum.infos + c.infos };
  }, own.get(node.element.uuid) ?? { ...ZERO });
}

/** The badge of a node: errors first, then warnings; empty when clean. */
export function countsBadge(counts: Counts): { text: string; severity: 'error' | 'warning' | 'info' | null } {
  if (counts.errors) return { text: String(Math.min(99, counts.errors)), severity: 'error' };
  if (counts.warnings) return { text: String(Math.min(99, counts.warnings)), severity: 'warning' };
  if (counts.infos) return { text: '', severity: 'info' };
  return { text: '', severity: null };
}

/** What the tree's "New child" offers on an element: the palette's tools that may be declared inside its kind. */
export function childTools(kind: string): { label: string; keyword: string }[] {
  const seen = new Set<string>();
  const out: { label: string; keyword: string }[] = [];
  for (const tools of Object.values(PALETTE)) {
    for (const tool of tools) {
      if (tool.kind !== 'create' || !tool.keyword || !tool.containerKinds?.includes(kind) || seen.has(tool.keyword)) continue;
      seen.add(tool.keyword);
      out.push({ label: tool.label, keyword: tool.keyword });
    }
  }
  return out;
}

// --- A node dropped on a diagram ----------------------------------------------------------

export type DropPlan =
  /** The element is drawn there already: reveal it. */
  | { kind: 'reveal'; drawingId: string }
  /** A declared diagram: add it to what the diagram shows. */
  | { kind: 'edit'; requests: EditRequest[]; label: string }
  | { kind: 'refuse'; reason: string };

function drawnId(graph: ViewerGraph, element: ElementRecord): string | null {
  const wanted = new Set([element.id, element.name, element.id.replace(/\.([^.]+)$/, '::$1')]);
  let found: string | null = null;
  const walk = (nodes: ViewerGraph['children']): void => {
    for (const node of nodes ?? []) {
      if (found) return;
      if (wanted.has(node.id) || node.id.endsWith(`.${element.id}`)) found = node.id;
      for (const port of node.ports ?? []) if (!found && wanted.has(port.id)) found = port.id;
      walk(node.children);
    }
  };
  walk(graph.children);
  return found;
}

/**
 * An element of the tree dropped on a diagram: revealed when it is drawn
 * there, added to the `shows` of a declared diagram, else refused with what
 * to do (a viewpoint draws the whole layer: the element is not of it).
 */
export function dropPlan(element: ElementRecord, graph: ViewerGraph, topIds: string[]): DropPlan {
  const drawn = drawnId(graph, element);
  if (drawn) return { kind: 'reveal', drawingId: drawn };
  const named = namedDiagramOf(graph.id, graph.arc.view);
  if (!named) return { kind: 'refuse', reason: `${element.name || element.id} is not drawn on ${graph.arc.title}: drop it on a declared diagram, or open the diagram of its layer` };
  if (/Port$/.test(element.kind)) return { kind: 'refuse', reason: 'a port is shown with its owner: drop the owner' };
  const shows = [...new Set([...topIds, element.id])];
  return { kind: 'edit', requests: [{ op: 'set', element: named, key: 'shows', value: showsLiteral(shows) }], label: `show ${element.id} on ${named}` };
}

/** The tree's drag payload: the element's id, typed for the page to recognize it. */
export const TREE_MIME = 'application/vnd.code.tree.arclangmodel';

export function dropPayload(uri: string, id: string): string {
  return `${uri}#${ELEMENT_FRAGMENT}${encodeURIComponent(id)}`;
}

/** How a tree node reaches a diagram: VS Code drops into an editor or a webview only with Shift held. */
export const DROP_HINT = 'Hold Shift while dragging it onto a diagram (VS Code drops into an editor only with Shift), or right-click › Show in the diagram.';

/** The fragment that names an element in a drag payload's URI. */
export const ELEMENT_FRAGMENT = 'arclang-element=';

/**
 * The element id of a drag payload: a `text/uri-list` line, VS Code's list of
 * resources (a JSON array) or the tree's own JSON. VS Code writes a tree
 * item's `resourceUri` into the drag itself (the extension's own data stays
 * in the extension host), with its fragment encoded: `#arclang-element%3D…`.
 * VS Code's tree payload `{ id: <view>, itemHandles }` names the view, not an element.
 */
export function idOfPayload(text: string): string | null {
  const match = /#arclang-element(?:=|%3D)([^&\s"',\]]+)/i.exec(text);
  if (match) return decodeFully(match[1]);
  try {
    const parsed = JSON.parse(text) as { id?: unknown; itemHandles?: unknown };
    if (typeof parsed.id === 'string' && parsed.itemHandles === undefined) return parsed.id;
  } catch {
    // not JSON
  }
  return null;
}

/** Undo every layer of percent-encoding (a URI's toString encodes the `%` of an encoded id again). */
function decodeFully(text: string): string {
  let current = text;
  for (let i = 0; i < 4; i++) {
    let next: string;
    try {
      next = decodeURIComponent(current);
    } catch {
      return current;
    }
    if (next === current) return current;
    current = next;
  }
  return current;
}
