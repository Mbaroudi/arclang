// Editing what is already drawn, without VS Code or a page: which
// relationship an edge stands for, the requests that reconnect, reverse,
// copy, paste or move it, and the moves that align or distribute boxes.
// Pure functions, tested with node:test; the webview imports them.
import { connectPlan, EditRequest, ElementRecord, LAYER_OF_VIEW, lifelineElement, graphContainerOf, RelationshipRecord, Snapshot, Tool, toolsFor, ViewerEdge, ViewerGraph } from './workbench';

/** The relationship kind an edge of the viewer draws, by its `arc.kind`. */
export const RELATIONSHIP_OF_EDGE: Record<string, string> = {
  component_exchange: 'ComponentExchange',
  functional_exchange: 'FunctionalExchange',
  physical_link: 'PhysicalLink',
  physical_exchange: 'PhysicalExchange',
  interaction: 'OperationalExchange',
  operational_exchange: 'OperationalExchange',
  transition: 'Transition',
  message: 'Message',
  association: 'Association',
};

/** The connect tool whose ends a relationship's ends may be, by relationship kind. */
const TOOL_OF_RELATIONSHIP: Record<string, string> = {
  ComponentExchange: 'component_exchange',
  FunctionalExchange: 'functional_exchange',
  PhysicalLink: 'physical_link',
  PhysicalExchange: 'physical_exchange',
  OperationalExchange: 'interaction',
  Transition: 'transition',
  Message: 'message',
  Association: 'association',
  Trace: 'trace',
};

/** Relationships whose ends are ports: reconnected to a box, they get a port there. */
const THROUGH_PORTS = new Set(['FunctionalExchange', 'ComponentExchange', 'PhysicalExchange']);

export const isPortKind = (kind: string): boolean => /Port$/.test(kind);

/**
 * How a declaration written in the source designates an element: its written
 * id, else its name (a port: its owner's reference, a dot, its name). The id
 * the compiler derives for an element that writes none (`LC-Batterie`) is for
 * the tools: in `from_port: "LC-Batterie.out"` it does not resolve.
 */
export function writtenRef(element: ElementRecord, byUuid: Map<string, ElementRecord>): string {
  if (isPortKind(element.kind)) {
    const owner = element.owner ? byUuid.get(element.owner) : undefined;
    return owner ? `${writtenRef(owner, byUuid)}.${element.name}` : element.id;
  }
  const written = element.attributes?.id;
  return typeof written === 'string' && written ? element.id : element.name || element.id;
}

/**
 * The model id of a node or port of the drawing: `LC-A::p` (a component port)
 * and `SF-A::in.p` (a function port, with its direction) are `LC-A.p` and `SF-A.p`.
 */
export function modelIdOf(drawingId: string): string {
  return drawingId.replace(/::(?:in|out|inout)\./, '.').replace('::', '.');
}

/** The model ids of the two ends of an edge (a lifeline stands for its participant). */
export function edgeEndIds(edge: ViewerEdge, graph: ViewerGraph): { source: string; target: string } {
  const container = graphContainerOf(graph.id, graph.arc.view);
  const end = (id: string | undefined) => (graph.arc.view === 'es' ? lifelineElement(id ?? '', container) : modelIdOf(id ?? ''));
  return { source: end(edge.sources[0]), target: end(edge.targets[0]) };
}

/** The relationship an edge of the graph draws: same kind, same ends, the same name when several match. */
export function relationshipOfEdge(edgeId: string, graph: ViewerGraph, snapshot: Snapshot, label?: string): RelationshipRecord | null {
  const edge = (graph.edges ?? []).find((e) => e.id === edgeId);
  if (!edge || !snapshot.elements) return null;
  const { source, target } = edgeEndIds(edge, graph);
  const byId = new Map(snapshot.elements.elements.map((e) => [e.id, e]));
  const from = byId.get(source);
  const to = byId.get(target);
  if (!from || !to) return null;
  const kind = RELATIONSHIP_OF_EDGE[edge.arc.kind];
  const candidates = (snapshot.elements.relationships ?? []).filter((r) => r.source === from.uuid && r.target === to.uuid && (!kind || r.kind === kind));
  if (candidates.length <= 1) return candidates[0] ?? null;
  const name = label ?? edgeId.replace(/^[a-z]+:/, '');
  return candidates.find((r) => r.name === name) ?? candidates[0];
}

/** The kinds an end of a relationship may be reconnected to, on a view (null: any element drawn). */
export function reconnectKinds(kind: string, view: string): string[] | null {
  const toolId = TOOL_OF_RELATIONSHIP[kind];
  const tool = toolsFor(view).find((t) => t.kind === 'connect' && t.connect === toolId);
  if (!tool || !tool.endKinds) return null;
  return tool.endKinds;
}

function ends(relationship: RelationshipRecord, elements: ElementRecord[]): { from: ElementRecord; to: ElementRecord } | { error: string } {
  const from = elements.find((e) => e.uuid === relationship.source);
  const to = elements.find((e) => e.uuid === relationship.target);
  if (!from || !to) return { error: `an end of ${relationship.name ?? relationship.kind} is not an element` };
  return { from, to };
}

const portName = (portId: string): string => portId.slice(portId.lastIndexOf('.') + 1);

/**
 * The requests that move one end of a relationship to `target`: an exchange
 * between ports landed on a box gets a port there first (named like the one
 * it leaves, created unless the box has it); anything else is reconnected
 * to the element itself. One gesture: the requests go together.
 */
export function reconnectPlan(relationship: RelationshipRecord, end: 'source' | 'target', target: ElementRecord, elements: ElementRecord[]): EditRequest[] | { error: string } {
  const both = ends(relationship, elements);
  if ('error' in both) return both;
  const old = end === 'source' ? both.from : both.to;
  const other = end === 'source' ? both.to : both.from;
  if (target.uuid === old.uuid) return { error: 'it is already connected there' };
  const requests: EditRequest[] = [];
  let newEnd = target.id;
  if (isPortKind(target.kind) && target.direction === (end === 'source' ? 'in' : 'out')) {
    return { error: `${target.id} is an ${target.direction} port: the ${end} of an exchange ${end === 'source' ? 'leaves by an out' : 'arrives by an in'} or inout port` };
  }
  if (THROUGH_PORTS.has(relationship.kind) && !isPortKind(target.kind)) {
    const name = portName(old.id);
    newEnd = `${target.id}.${name}`;
    if (!elements.some((e) => e.id === newEnd)) requests.push({ op: 'add', keyword: end === 'source' ? 'port out' : 'port in', name, container: target.id });
  }
  // A state may loop on itself, a lifeline send to itself; an exchange joins two ends.
  const ownerOf = (id: string) => (THROUGH_PORTS.has(relationship.kind) ? id.slice(0, Math.max(0, id.lastIndexOf('.'))) || id : id);
  if ((newEnd === other.id || ownerOf(newEnd) === ownerOf(other.id)) && !['Transition', 'Message'].includes(relationship.kind)) return { error: 'an exchange does not connect an element to itself' };
  const reconnect: EditRequest = {
    op: 'reconnect',
    kind: relationship.kind,
    from: both.from.id,
    to: both.to.id,
    ...(relationship.name ? { name: relationship.name } : {}),
    ...(end === 'source' ? { new_from: newEnd } : { new_to: newEnd }),
  };
  return [...requests, reconnect];
}

/**
 * The requests that turn a relationship around: its source becomes its
 * target. An exchange between an out and an in port turns its ports around
 * too, when they carry nothing else; otherwise it is not reversed.
 */
export function reversePlan(relationship: RelationshipRecord, elements: ElementRecord[], relationships: RelationshipRecord[] = []): EditRequest[] | { error: string } {
  const both = ends(relationship, elements);
  if ('error' in both) return both;
  if (relationship.kind === 'Trace') return { error: 'a trace goes from the element to what it satisfies, realizes or verifies: retarget it instead' };
  const flips: EditRequest[] = [];
  for (const [port, now, next] of [[both.from, 'out', 'in'], [both.to, 'in', 'out']] as const) {
    if (!isPortKind(port.kind) || port.direction !== now) continue;
    const carried = relationships.filter((r) => r.source === port.uuid || r.target === port.uuid).length;
    if (carried > 1) return { error: `${port.id} carries other exchanges: it cannot turn around for this one` };
    flips.push({ op: 'set', element: port.id, key: 'direction', value: next });
  }
  return [...flips, { op: 'reconnect', kind: relationship.kind, from: both.from.id, to: both.to.id, ...(relationship.name ? { name: relationship.name } : {}), new_from: both.to.id, new_to: both.from.id }];
}

// --- Align and distribute ---------------------------------------------------

export interface Box {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export type Alignment = 'left' | 'right' | 'top' | 'bottom' | 'center' | 'middle';

export interface Move {
  id: string;
  dx: number;
  dy: number;
}

const moved = (moves: Move[]): Move[] => moves.filter((m) => Math.abs(m.dx) > 0.01 || Math.abs(m.dy) > 0.01);

/** The moves that line boxes up on the outermost edge (or the mean centre) of the selection. */
export function alignMoves(boxes: Box[], how: Alignment): Move[] {
  if (boxes.length < 2) return [];
  const left = Math.min(...boxes.map((b) => b.x));
  const right = Math.max(...boxes.map((b) => b.x + b.width));
  const top = Math.min(...boxes.map((b) => b.y));
  const bottom = Math.max(...boxes.map((b) => b.y + b.height));
  const centre = (left + right) / 2;
  const middle = (top + bottom) / 2;
  return moved(
    boxes.map((b) => {
      switch (how) {
        case 'left':
          return { id: b.id, dx: left - b.x, dy: 0 };
        case 'right':
          return { id: b.id, dx: right - b.x - b.width, dy: 0 };
        case 'top':
          return { id: b.id, dx: 0, dy: top - b.y };
        case 'bottom':
          return { id: b.id, dx: 0, dy: bottom - b.y - b.height };
        case 'center':
          return { id: b.id, dx: centre - b.x - b.width / 2, dy: 0 };
        default:
          return { id: b.id, dx: 0, dy: middle - b.y - b.height / 2 };
      }
    }),
  );
}

/** The moves that leave equal gaps between boxes, the two outermost staying put. */
export function distributeMoves(boxes: Box[], axis: 'horizontal' | 'vertical'): Move[] {
  if (boxes.length < 3) return [];
  const along = axis === 'horizontal' ? (b: Box) => b.x : (b: Box) => b.y;
  const size = axis === 'horizontal' ? (b: Box) => b.width : (b: Box) => b.height;
  const sorted = [...boxes].sort((a, b) => along(a) - along(b));
  const first = sorted[0];
  const last = sorted[sorted.length - 1];
  const span = along(last) + size(last) - along(first);
  const gap = (span - sorted.reduce((sum, b) => sum + size(b), 0)) / (sorted.length - 1);
  let at = along(first);
  const moves: Move[] = [];
  for (const box of sorted) {
    const shift = at - along(box);
    moves.push(axis === 'horizontal' ? { id: box.id, dx: shift, dy: 0 } : { id: box.id, dx: 0, dy: shift });
    at += size(box) + gap;
  }
  return moved(moves);
}

// --- Copy, cut, paste, duplicate, move to -----------------------------------

export interface Clip {
  ids: string[];
  cut: boolean;
}

/** What a paste writes: a cut moves the elements into the container, a copy duplicates them there (beside the originals without one). */
export function pasteRequests(clip: Clip, container: ElementRecord | null): EditRequest[] | { error: string } {
  if (clip.ids.length === 0) return { error: 'nothing was copied' };
  const into = container && !clip.ids.includes(container.id) ? container : null;
  if (clip.cut) {
    if (!into) return { error: 'select the element to paste into, then paste' };
    return clip.ids.map((id) => ({ op: 'move', element: id, container: into.id }));
  }
  return clip.ids.map((id) => (into ? { op: 'duplicate', element: id, container: into.id } : { op: 'duplicate', element: id }));
}

const PORT_OWNERS = ['SystemFunction', 'LogicalFunction', 'PhysicalFunction', 'SystemComponent', 'SystemActor', 'LogicalComponent', 'BehaviorComponent', 'PhysicalNode', 'Actor', 'OperationalEntity'];

/**
 * Where an element may be moved to, for "Move to…": a port goes to another
 * owner of ports of the same layer, anything else into an element of the
 * kind of its owner or of its own kind (never into itself or what it holds).
 */
export function moveTargets(element: ElementRecord, elements: ElementRecord[]): ElementRecord[] {
  const byUuid = new Map(elements.map((e) => [e.uuid, e]));
  const owner = element.owner ? byUuid.get(element.owner) : undefined;
  const sameLayer = (other: ElementRecord) => layerFamily(other.kind) === layerFamily(element.kind);
  const inside = (other: ElementRecord): boolean => {
    for (let at: ElementRecord | undefined = other; at; at = at.owner ? byUuid.get(at.owner) : undefined) if (at.uuid === element.uuid) return true;
    return false;
  };
  if (isPortKind(element.kind)) {
    const kinds = new Set(PORT_OWNERS.filter((k) => !owner || layerFamily(k) === layerFamily(owner.kind)));
    // An owner that already has a port of that name cannot take another.
    const taken = (other: ElementRecord) => elements.some((e) => e.id === `${other.id}.${element.name}`);
    return elements.filter((e) => kinds.has(e.kind) && e.uuid !== owner?.uuid && !taken(e));
  }
  const kinds = new Set([element.kind, ...(owner && owner.kind !== 'Model' ? [owner.kind] : [])]);
  return elements.filter((e) => kinds.has(e.kind) && e.uuid !== owner?.uuid && !inside(e) && sameLayer(e));
}

/** The family of a kind for ports: what layer its name says it is in. */
function layerFamily(kind: string): string {
  if (/^System|^Actor$|^SystemActor$/.test(kind)) return 'system';
  if (/^Logical/.test(kind)) return 'logical';
  if (/^Physical|^Behavior/.test(kind)) return 'physical';
  return 'operational';
}

/** The request that moves an element out of its container into the layer of the view. */
export function moveToLayerRequest(element: ElementRecord, view: string): EditRequest | { error: string } {
  const layer = LAYER_OF_VIEW[view];
  if (!layer) return { error: `no layer for the ${view} view` };
  return { op: 'move', element: element.id, layer };
}

// --- Quick connect: a new element connected to a box --------------------------

/** The create tool a quick connect from a box of `kind` uses: the tool making that kind on the view. */
const QUICK: Record<string, string> = {
  LogicalComponent: 'component',
  SystemComponent: 'component',
  SystemActor: 'actor',
  SystemFunction: 'function',
  LogicalFunction: 'function',
  PhysicalFunction: 'function',
  PhysicalNode: 'node',
  Actor: 'entity',
  OperationalEntity: 'entity',
  OperationalActivity: 'activity',
  State: 'state',
  Class: 'class',
};

/** A fresh id with the prefix of `like` (`LC-CAM` -> `LC-1`, the first number free), or null without a prefix. */
export function freshIdLike(like: string, taken: Set<string>): string | null {
  const prefix = /^[A-Z][A-Z0-9]*-/.exec(like)?.[0];
  if (!prefix) return null;
  for (let n = 1; ; n++) if (!taken.has(`${prefix}${n}`)) return `${prefix}${n}`;
}

/**
 * The requests that create an element like `from` beside it and connect it:
 * the create tool of the view for that kind, then the connect tool between
 * the two kinds. Null when the view has neither.
 */
export function quickConnectPlan(view: string, from: ElementRecord, name: string, elements: ElementRecord[], graphContainer: string | null): { requests: EditRequest[]; created: string } | { error: string } {
  const tools = toolsFor(view);
  const create = tools.find((t) => t.kind === 'create' && t.id === QUICK[from.kind]);
  if (!create || !create.keyword) return { error: `the ${view} view has no tool to create a ${from.kind} beside it` };
  const connect = tools.find((t) => t.kind === 'connect' && t.connect !== 'trace' && (!t.endKinds || t.endKinds.includes(from.kind)));
  if (!connect) return { error: `nothing connects a ${from.kind} on this view` };
  const label = name.trim();
  if (!label) return { error: 'a name is needed' };
  const taken = new Set(elements.flatMap((e) => [e.id, e.name]));
  const id = create.inGraph ? null : freshIdLike(from.id, taken);
  const byUuid = new Map(elements.map((e) => [e.uuid, e]));
  const owner = from.owner ? byUuid.get(from.owner) : undefined;
  const container = owner && create.containerKinds?.includes(owner.kind) ? owner.id : undefined;
  const add = addRequest(create, label, id, container, view, graphContainer);
  if ('error' in add) return add;
  const createdId = id ?? (create.inGraph && graphContainer ? `${graphContainer}.${label}` : label);
  const plan = connectPlan(connect, { id: from.id, kind: from.kind }, { id: id ?? label, kind: from.kind }, connect.connect === 'transition' ? '' : `${from.name} to ${label}`, view, graphContainer);
  if ('error' in plan) return plan;
  return { requests: [add, ...plan], created: createdId };
}

function addRequest(tool: Tool, name: string, id: string | null, container: string | undefined, view: string, graphContainer: string | null): EditRequest | { error: string } {
  const attributes: [string, string][] = id ? [['id', JSON.stringify(id)]] : [];
  if (container) return { op: 'add', keyword: tool.keyword!, name, attributes, container };
  if (tool.inGraph) return graphContainer ? { op: 'add', keyword: tool.keyword!, name, attributes, container: graphContainer } : { error: 'this diagram is not about one state machine' };
  if (tool.topLevel) return { op: 'add', keyword: tool.keyword!, name, attributes, top_level: true };
  const layer = LAYER_OF_VIEW[view];
  return layer ? { op: 'add', keyword: tool.keyword!, name, attributes, layer } : { error: `no layer for the ${view} view` };
}
