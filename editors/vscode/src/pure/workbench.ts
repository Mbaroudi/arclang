// The workbench, without VS Code: the shapes `arclang workbench` and
// `arclang edit` exchange, the model tree, the palette of gestures and the
// edits each gesture writes. Pure functions, tested with node:test; the
// extension host and the webview both import from here.
import { tableLayer } from './tables'; // tables hook
import type { ArrangeCommand } from './arrange'; // zorder hook

/** One element of the compiled model, as `arclang workbench` lists it. */
export interface ElementRecord {
  uuid: string;
  /** The id the compiler gives it: written, or derived from the name. */
  id: string;
  kind: string;
  name: string;
  /** The uuid of the owning element, null at the top. */
  owner: string | null;
  attributes: Record<string, unknown>;
  /** A port's direction: `in`, `out`, `inout`. */
  direction?: string;
}

export interface RelationshipRecord {
  uuid: string;
  kind: string;
  name: string;
  /** uuids of the two ends */
  source: string;
  target: string;
  attributes?: Record<string, unknown>;
}

export interface Finding {
  rule: string;
  severity: string;
  message: string;
  element?: string | null;
  fix?: string | null;
}

export interface GateReport {
  passed: boolean;
  requirements_total: number;
  requirements_verified: number;
  findings: { check: string; severity: string; message: string }[];
}

export interface ViewerNode {
  id: string;
  arc: { kind: string; uuid?: string; properties?: Record<string, unknown> };
  children?: ViewerNode[];
  ports?: { id: string; arc?: { direction?: string } }[];
}

export interface ViewerEdge {
  id: string;
  arc: { kind: string; source_node?: string; target_node?: string };
  sources: string[];
  targets: string[];
}

export interface ViewerGraph {
  id: string;
  arc: { view: string; title: string; chains?: unknown[] };
  children?: ViewerNode[];
  edges?: ViewerEdge[];
}

/** The `payload` argument of `ArcViz.mount`. */
export interface ViewerPayload {
  title: string;
  graphs: ViewerGraph[];
  diagnostics: unknown[];
  links: { kind: string; source: string; target: string }[];
  layout?: unknown;
  layout_name?: string;
}

export interface Snapshot {
  success: boolean;
  errors: string[];
  warnings: string[];
  stats: Record<string, number> | null;
  diagrams: ViewerPayload | null;
  document: unknown;
  elements: { elements: ElementRecord[]; relationships: RelationshipRecord[] } | null;
  findings: Finding[];
  gate: GateReport | null;
}

export interface AttributeType {
  type: string;
  of?: string;
  values?: string[];
}

export interface MetamodelAttribute {
  key: string;
  type: AttributeType;
  doc?: string;
}

export interface MetamodelKind {
  name: string;
  layer: string;
  arcadia?: string;
  sysml?: string;
  doc?: string;
  attributes: MetamodelAttribute[];
}

export interface Metamodel {
  version?: string;
  kinds: MetamodelKind[];
  enums?: Record<string, string[]> | unknown;
  trace_rules?: unknown;
  units?: unknown;
}

/** One edit of the model text, as `arclang edit --request` takes it. */
export type EditRequest =
  | { op: 'set'; element: string; key: string; value: string }
  | { op: 'unset'; element: string; key: string }
  | { op: 'rename'; element: string; name: string }
  | {
      op: 'add';
      keyword: string;
      name: string;
      attributes?: [string, string][];
      container?: string;
      layer?: string;
      layer_name?: string;
      top_level?: boolean;
    }
  | { op: 'add_raw'; declaration: string; container?: string; layer?: string; layer_name?: string; top_level?: boolean }
  | { op: 'remove'; element: string }
  | { op: 'move'; element: string; container?: string; layer?: string; layer_name?: string }
  | { op: 'add_trace'; from: string; kind: string; to: string }
  | { op: 'remove_trace'; from: string; kind: string; to: string }
  | { op: 'reconnect'; kind: string; name?: string; from: string; to: string; new_from?: string; new_to?: string }
  | { op: 'duplicate'; element: string; container?: string; layer?: string; layer_name?: string }
  | { op: 'duplicate_many'; elements: string[]; container?: string; layer?: string; layer_name?: string }
  | {
      op: 'wrap_fragment';
      scenario: string;
      kind: string;
      label?: string;
      first: string;
      last: string;
      operands?: [string, string][];
    };

/** What the extension host sends to the webview. */
export type ToWebview =
  | { type: 'snapshot'; snapshot: Snapshot; metamodel: Metamodel | null; file: string; selected: string | null }
  | { type: 'select'; id: string | null }
  | { type: 'show'; view: string; select: string | null }
  | { type: 'layer'; step: number }
  | { type: 'diagram-op'; op: 'hide' | 'related' | 'new' | 'rename'; id: string }
  | { type: 'edit-result'; ok: boolean; label: string; error?: string }
  | { type: 'busy'; busy: boolean; label?: string }
  /** A key of the canvas, bound in VS Code (so it follows the keyboard layout) and run by the page. */
  | { type: 'key'; command: CanvasCommand; text?: string }
  /** What ⌘Z and ⌘⇧Z would take back and bring back; `done` names the step just undone or redone. */
  | { type: 'history'; undo: string | null; redo: string | null; done?: string }
  /** The colour themes on offer and the one drawn (`problem`: why the one asked for is not). */
  | { type: 'themes'; entries: ThemeEntry[]; active: string; problem: string | null; dark: boolean; copyBackground: 'white' | 'transparent' }
  /** A command of the diagram, run by the page (copy as image, PowerPoint, the theme editor). */
  | { type: 'diagram-action'; action: DiagramAction }
  /** Draw these views for the slides of a PowerPoint file; `target`, where the host writes it. */
  | { type: 'pptx-render'; ids: string[]; target?: string }
  /** A theme saved by the host, or one read from a file the user chose. */
  | { type: 'theme-saved'; entry: ThemeEntry }
  | { type: 'theme-imported'; theme: unknown; errors: string[]; name: string };

/** Where a colour theme comes from. */
export type ThemeSource = 'preset' | 'team' | 'user' | 'file';

/** A colour theme on offer, as the host read and checked it. */
export interface ThemeEntry {
  /** `preset:<id>`, `team:<id>`, `user:<id>` or `file:<id>`. */
  key: string;
  id: string;
  name: string;
  source: ThemeSource;
  path?: string;
  description?: string;
  theme: Record<string, unknown> | null;
  ok: boolean;
  errors: string[];
  warnings: string[];
}

/** The diagram commands the host asks the page to run. */
export type DiagramAction = 'copy-image' | 'copy-svg' | 'export-pptx' | 'choose-theme' | 'edit-theme';

/** One slide of a PowerPoint export, as the page draws it: the PNG in base64. */
export interface SlideData {
  title: string;
  data: string;
  width: number;
  height: number;
  notes: string[];
}

/** The canvas commands VS Code's keybindings run on the workbench. */
export type CanvasCommand = 'copy' | 'cut' | 'paste' | 'paste-text' | 'duplicate' | 'select-all' | 'delete' | 'zoom-in' | 'zoom-out' | 'fit' | ArrangeCommand; // zorder hook: ArrangeCommand

/** What the webview sends to the extension host. */
export type FromWebview =
  | { type: 'ready' }
  | { type: 'edit'; requests: EditRequest[]; label: string }
  | { type: 'reveal'; id: string }
  | { type: 'ask'; id: string; name: string; kind: string }
  | { type: 'selected'; id: string | null; kind?: string; name?: string }
  | { type: 'open-text' }
  | { type: 'command'; command: string }
  | { type: 'save'; name: string; mime: string; data: string }
  /** The arrangement changed; `join` makes it part of the step before it (a pasted copy placed). */
  | { type: 'layout'; file: unknown; join?: boolean }
  | { type: 'history'; op: 'undo' | 'redo' }
  /** What a copy took, for the host to put its source on the system clipboard. */
  | { type: 'copied'; elements: { id: string; name: string }[]; relationships: string[] }
  /** Text selected on the page, copied by ⌘C. */
  | { type: 'clipboard-text'; text: string }
  /** A field of the page has the keyboard: VS Code's keys are its own. */
  | { type: 'typing'; on: boolean }
  | { type: 'focus-mode'; on: boolean }
  /** A PNG (base64) the page could not put on the clipboard itself: the host does. */
  | { type: 'clipboard-image'; data: string; width: number; height: number; cropped: boolean }
  | { type: 'clipboard-svg'; text: string }
  /** A line for the status bar (an export done), or an error to show. */
  | { type: 'notify'; text: string; error?: boolean }
  /** The views a PowerPoint export may take, for the host to ask which. */
  | { type: 'pptx-request'; views: { id: string; title: string; view: string }[]; current: string | null }
  | { type: 'pptx'; slides: SlideData[]; title: string; target?: string }
  | { type: 'theme-choose'; key: string }
  | { type: 'theme-save'; scope: 'team' | 'user'; theme: Record<string, unknown> }
  | { type: 'theme-export'; theme: Record<string, unknown> }
  | { type: 'theme-import' };

/** `arclang workbench` output, read leniently: a broken answer is a failed snapshot. */
export function parseSnapshot(text: string): Snapshot {
  const failed = (message: string): Snapshot => ({
    success: false,
    errors: [message],
    warnings: [],
    stats: null,
    diagrams: null,
    document: null,
    elements: null,
    findings: [],
    gate: null,
  });
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return failed(text.trim() || 'the compiler printed nothing');
  }
  if (typeof raw !== 'object' || raw === null) return failed('the compiler printed no snapshot');
  const value = raw as Record<string, unknown>;
  const strings = (key: string): string[] => (Array.isArray(value[key]) ? (value[key] as unknown[]).map(String) : []);
  const elements = value.elements as Snapshot['elements'] | undefined;
  return {
    success: value.success === true,
    errors: strings('errors'),
    warnings: strings('warnings'),
    stats: (value.stats as Record<string, number> | null) ?? null,
    diagrams: (value.diagrams as ViewerPayload | null) ?? null,
    document: value.document ?? null,
    elements: elements && Array.isArray(elements.elements) ? { elements: elements.elements, relationships: elements.relationships ?? [] } : null,
    findings: Array.isArray(value.findings) ? (value.findings as Finding[]) : [],
    gate: (value.gate as GateReport | null) ?? null,
  };
}

/** `arclang edit` output: the edited text, or why it was refused. */
export function parseEditAnswer(text: string): { ok: true; source: string } | { ok: false; error: string } {
  try {
    const value = JSON.parse(text) as { success?: boolean; source?: string; error?: string };
    if (value.success === true && typeof value.source === 'string') return { ok: true, source: value.source };
    return { ok: false, error: value.error ?? 'the edit was refused' };
  } catch {
    return { ok: false, error: text.trim() || 'the compiler printed nothing' };
  }
}

// --- The model tree -------------------------------------------------------

export interface TreeNode {
  element: ElementRecord;
  children: TreeNode[];
}

/** The Arcadia layers in method order, as the tree groups elements. */
export const LAYERS = ['Operational', 'System', 'Logical', 'Physical', 'Epbs', 'Data', 'Transverse'] as const;

export const LAYER_LABELS: Record<string, string> = {
  Operational: 'Operational Analysis',
  System: 'System Analysis',
  Logical: 'Logical Architecture',
  Physical: 'Physical Architecture',
  Epbs: 'EPBS',
  Data: 'Data',
  Transverse: 'Transverse',
};

/** The layer of a kind, from the metamodel; unknown kinds are transverse. */
export function layerOfKind(kind: string, metamodel: Metamodel | null): string {
  const found = metamodel?.kinds.find((k) => k.name === kind);
  return found?.layer ?? 'Transverse';
}

/** Elements nested by ownership, in the order the compiler lists them. */
export function elementTree(elements: ElementRecord[]): TreeNode[] {
  const nodes = new Map<string, TreeNode>();
  for (const element of elements) nodes.set(element.uuid, { element, children: [] });
  const roots: TreeNode[] = [];
  for (const node of nodes.values()) {
    const parent = node.element.owner ? nodes.get(node.element.owner) : undefined;
    if (parent && parent !== node) parent.children.push(node);
    else roots.push(node);
  }
  return roots;
}

/**
 * The tree grouped by layer, layers in method order, empty ones left out.
 * The compiler makes the model element own every top-level declaration:
 * the groups are its children, and the model itself heads the list.
 */
export function treeByLayer(elements: ElementRecord[], metamodel: Metamodel | null): { layer: string; label: string; roots: TreeNode[] }[] {
  const roots = elementTree(elements);
  const models = roots.filter((r) => r.element.kind === 'Model');
  const tops = roots.flatMap((r) => (r.element.kind === 'Model' ? r.children : [r]));
  const groups = new Map<string, TreeNode[]>();
  for (const top of tops) {
    const layer = top.element.kind === 'Table' ? tableLayer(top.element.attributes) : layerOfKind(top.element.kind, metamodel); // tables hook
    groups.set(layer, [...(groups.get(layer) ?? []), top]);
  }
  const order = [...LAYERS, ...[...groups.keys()].filter((l) => !(LAYERS as readonly string[]).includes(l))];
  const layers = order
    .filter((layer) => (groups.get(layer) ?? []).length > 0)
    .map((layer) => ({ layer, label: LAYER_LABELS[layer] ?? layer, roots: groups.get(layer) ?? [] }));
  const head = models.map((model) => ({ layer: 'Model', label: model.element.name || model.element.id, roots: [{ element: model.element, children: [] }] }));
  return [...head, ...layers];
}

// --- Attribute values -----------------------------------------------------

/** An ArcLang string literal. */
export function stringLiteral(text: string): string {
  return JSON.stringify(text);
}

/**
 * What the user typed in a properties field, as ArcLang source for `set`:
 * texts are quoted, numbers, quantities, enumeration members and booleans
 * are written bare, references are quoted, lists are `[a, b]`.
 */
export function attributeLiteral(type: AttributeType | undefined, raw: string): string {
  const text = raw.trim();
  // Without a type, the shape of the text decides (numbers, booleans and lists stay bare).
  const kind = type?.type ?? 'Unknown';
  const quoteUnlessLiteral = (item: string): string => {
    const t = item.trim();
    if (/^".*"$/.test(t)) return t;
    if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(t)) return t;
    return stringLiteral(t);
  };
  switch (kind) {
    case 'Text':
    case 'Identifier':
      return /^".*"$/.test(text) ? text : stringLiteral(text);
    case 'Number':
    case 'Integer':
    case 'Quantity':
    case 'Boolean':
    case 'Enum':
    case 'Enumeration':
      return text;
    case 'Reference':
      return quoteUnlessLiteral(text);
    case 'List':
    case 'References':
    case 'TextList': {
      const inner = text.replace(/^\[/, '').replace(/\]$/, '');
      const items = inner
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean);
      return `[${items.map(quoteUnlessLiteral).join(', ')}]`;
    }
    default:
      return /^".*"$/.test(text) || /^[-+]?\d/.test(text) || /^(true|false)$/.test(text) || /^\[.*\]$/.test(text) ? text : stringLiteral(text);
  }
}

/** The attribute as the properties view shows it: lists joined, texts bare. */
export function attributeDisplay(value: unknown): string {
  if (value === null || value === undefined) return '';
  if (Array.isArray(value)) return value.map((v) => attributeDisplay(v)).join(', ');
  if (typeof value === 'object') {
    const record = value as Record<string, unknown>;
    if ('value' in record && 'unit' in record) return `${record.value} ${record.unit}`;
    return JSON.stringify(value);
  }
  return String(value);
}

// --- The palette ----------------------------------------------------------

/** The viewpoints in method order, with the name the layer bar shows. */
export const VIEWPOINTS: { view: string; label: string; short: string }[] = [
  { view: 'oab', label: 'Operational analysis', short: 'Operational' },
  { view: 'sab', label: 'System analysis', short: 'System' },
  { view: 'lab', label: 'Logical architecture', short: 'Logical' },
  { view: 'pab', label: 'Physical architecture', short: 'Physical' },
  { view: 'pbs', label: 'Product breakdown (EPBS)', short: 'EPBS' },
  { view: 'cap', label: 'Missions and capabilities', short: 'Capabilities' },
  { view: 'cdb', label: 'Data model', short: 'Data' },
  { view: 'msm', label: 'Modes and states', short: 'Modes' },
  { view: 'es', label: 'Scenarios', short: 'Scenarios' },
];

/** The viewpoint the layer of the model tree opens. */
export const VIEW_OF_LAYER: Record<string, string> = { Operational: 'oab', System: 'sab', Logical: 'lab', Physical: 'pab', Epbs: 'pbs', Data: 'cdb', Transverse: 'cap' };

/** The layer block a view's elements are declared in. */
export const LAYER_OF_VIEW: Record<string, string> = {
  oab: 'operational_analysis',
  sab: 'system_analysis',
  lab: 'logical_architecture',
  pab: 'physical_architecture',
  cap: 'system_analysis',
};

export type ToolKind = 'create' | 'connect';

export interface Tool {
  id: string;
  label: string;
  kind: ToolKind;
  /** create: the declaration keyword (`component`, `port out`). */
  keyword?: string;
  /** create: declared inside the selected element when its kind is one of these, else in the layer (or at the top level). */
  containerKinds?: string[];
  /** create: must be declared inside one of `containerKinds` (never in the layer). */
  needsContainer?: boolean;
  /** create: declared at the top level of the file (data model). */
  topLevel?: boolean;
  /** create: declared inside the element the diagram is about (a state machine, a scenario). */
  inGraph?: boolean;
  /** special gestures handled by the page: a requirement (with a satisfies trace), a fragment around a message. */
  special?: 'requirement' | 'fragment';
  /** connect: the keyword of the relationship and how its ends are written. */
  connect?: 'interaction' | 'functional_exchange' | 'component_exchange' | 'physical_link' | 'physical_exchange' | 'association' | 'transition' | 'message' | 'trace';
  /** connect: the kinds the two ends may have. */
  endKinds?: string[];
}

const PORTED = ['SystemFunction', 'LogicalFunction', 'PhysicalFunction', 'SystemComponent', 'SystemActor', 'LogicalComponent', 'BehaviorComponent', 'PhysicalNode', 'Actor'];
const FUNCTION_KINDS = ['SystemFunction', 'LogicalFunction', 'PhysicalFunction', 'OperationalActivity'];

export const PALETTE: Record<string, Tool[]> = {
  oab: [
    { id: 'actor', label: 'Actor', kind: 'create', keyword: 'actor' },
    { id: 'entity', label: 'Entity', kind: 'create', keyword: 'entity' },
    { id: 'activity', label: 'Activity', kind: 'create', keyword: 'activity', containerKinds: ['OperationalEntity', 'Actor'] },
    { id: 'operational_capability', label: 'Capability', kind: 'create', keyword: 'operational_capability' },
    { id: 'interaction', label: 'Interaction', kind: 'connect', connect: 'interaction', endKinds: ['Actor', 'OperationalEntity', 'OperationalActivity'] },
    { id: 'trace', label: 'Trace', kind: 'connect', connect: 'trace' },
  ],
  sab: [
    { id: 'actor', label: 'Actor', kind: 'create', keyword: 'actor' },
    { id: 'function', label: 'Function', kind: 'create', keyword: 'function', containerKinds: ['SystemFunction', 'SystemComponent', 'SystemActor'] },
    { id: 'component', label: 'Component', kind: 'create', keyword: 'system_component' },
    { id: 'port_in', label: 'Port in', kind: 'create', keyword: 'port in', containerKinds: PORTED, needsContainer: true },
    { id: 'port_out', label: 'Port out', kind: 'create', keyword: 'port out', containerKinds: PORTED, needsContainer: true },
    { id: 'mission', label: 'Mission', kind: 'create', keyword: 'mission' },
    { id: 'capability', label: 'Capability', kind: 'create', keyword: 'capability' },
    { id: 'functional_chain', label: 'Functional chain', kind: 'create', keyword: 'functional_chain' },
    { id: 'requirement', label: 'Requirement', kind: 'create', special: 'requirement' },
    { id: 'functional_exchange', label: 'Functional exchange', kind: 'connect', connect: 'functional_exchange', endKinds: ['SystemFunction', 'FunctionPort'] },
    { id: 'component_exchange', label: 'Component exchange', kind: 'connect', connect: 'component_exchange', endKinds: ['SystemComponent', 'SystemActor', 'ComponentPort'] },
    { id: 'trace', label: 'Trace', kind: 'connect', connect: 'trace' },
  ],
  lab: [
    { id: 'component', label: 'Component', kind: 'create', keyword: 'component', containerKinds: ['LogicalComponent'] },
    { id: 'function', label: 'Function', kind: 'create', keyword: 'function', containerKinds: ['LogicalComponent', 'LogicalFunction'] },
    { id: 'port_in', label: 'Port in', kind: 'create', keyword: 'port in', containerKinds: PORTED, needsContainer: true },
    { id: 'port_out', label: 'Port out', kind: 'create', keyword: 'port out', containerKinds: PORTED, needsContainer: true },
    { id: 'interface', label: 'Interface', kind: 'create', keyword: 'interface' },
    { id: 'component_exchange', label: 'Component exchange', kind: 'connect', connect: 'component_exchange', endKinds: ['LogicalComponent', 'ComponentPort'] },
    { id: 'functional_exchange', label: 'Functional exchange', kind: 'connect', connect: 'functional_exchange', endKinds: ['LogicalFunction', 'FunctionPort'] },
    { id: 'trace', label: 'Trace', kind: 'connect', connect: 'trace' },
  ],
  pab: [
    { id: 'node', label: 'Node', kind: 'create', keyword: 'node', containerKinds: ['PhysicalNode'] },
    { id: 'behavior_component', label: 'Behavior component', kind: 'create', keyword: 'behavior_component', containerKinds: ['PhysicalNode'], needsContainer: true },
    { id: 'hardware_component', label: 'Hardware component', kind: 'create', keyword: 'hardware_component', containerKinds: ['PhysicalNode'], needsContainer: true },
    { id: 'function', label: 'Function', kind: 'create', keyword: 'function', containerKinds: ['BehaviorComponent', 'PhysicalNode', 'PhysicalFunction'] },
    { id: 'port_in', label: 'Port in', kind: 'create', keyword: 'port in', containerKinds: PORTED, needsContainer: true },
    { id: 'port_out', label: 'Port out', kind: 'create', keyword: 'port out', containerKinds: PORTED, needsContainer: true },
    { id: 'physical_link', label: 'Physical link', kind: 'connect', connect: 'physical_link', endKinds: ['PhysicalNode'] },
    { id: 'physical_exchange', label: 'Physical exchange', kind: 'connect', connect: 'physical_exchange', endKinds: ['PhysicalNode', 'BehaviorComponent', 'ComponentPort', 'PhysicalPort'] },
    { id: 'functional_exchange', label: 'Functional exchange', kind: 'connect', connect: 'functional_exchange', endKinds: ['PhysicalFunction', 'FunctionPort'] },
    { id: 'trace', label: 'Trace', kind: 'connect', connect: 'trace' },
  ],
  cap: [
    { id: 'mission', label: 'Mission', kind: 'create', keyword: 'mission' },
    { id: 'capability', label: 'Capability', kind: 'create', keyword: 'capability' },
    { id: 'trace', label: 'Trace', kind: 'connect', connect: 'trace' },
  ],
  msm: [
    { id: 'state', label: 'State', kind: 'create', keyword: 'state', inGraph: true, containerKinds: ['State'] },
    { id: 'mode', label: 'Mode', kind: 'create', keyword: 'mode', inGraph: true, containerKinds: ['State'] },
    { id: 'transition', label: 'Transition', kind: 'connect', connect: 'transition', endKinds: ['State'] },
  ],
  es: [
    { id: 'message', label: 'Message', kind: 'connect', connect: 'message' },
    { id: 'fragment', label: 'Fragment', kind: 'create', special: 'fragment' },
  ],
  cdb: [
    { id: 'class', label: 'Class', kind: 'create', keyword: 'class', topLevel: true },
    { id: 'enumeration', label: 'Enumeration', kind: 'create', keyword: 'enumeration', topLevel: true },
    { id: 'data_type', label: 'Data type', kind: 'create', keyword: 'data_type', topLevel: true },
    { id: 'exchange_item', label: 'Exchange item', kind: 'create', keyword: 'exchange_item', topLevel: true },
    { id: 'association', label: 'Association', kind: 'connect', connect: 'association', endKinds: ['Class'] },
  ],
};

/** The tools of a view; unknown views get what their diagrams can take. */
export function toolsFor(view: string): Tool[] {
  return PALETTE[view] ?? [];
}

export interface CreateContext {
  view: string;
  name: string;
  /** The element clicked, if any, with its kind. */
  selected?: { id: string; kind: string } | null;
  /** The element the diagram is about (the state machine of an msm view, the scenario of an es view). */
  graphContainer?: string | null;
}

/** Where a created element goes: inside the selected element, in the layer, or at the top. */
export function createRequest(tool: Tool, context: CreateContext): EditRequest | { error: string } {
  if (!tool.keyword) return { error: `${tool.label} creates nothing` };
  const name = context.name.trim();
  if (!name) return { error: 'a name is needed' };
  const container = context.selected && tool.containerKinds?.includes(context.selected.kind) ? context.selected.id : undefined;
  if (tool.needsContainer && !container) {
    return { error: `${tool.label}: select ${(tool.containerKinds ?? []).join(', ')} first` };
  }
  if (container) return { op: 'add', keyword: tool.keyword, name, container };
  if (tool.inGraph) {
    if (!context.graphContainer) return { error: `${tool.label}: this diagram is not about one state machine or scenario` };
    return { op: 'add', keyword: tool.keyword, name, container: context.graphContainer };
  }
  if (tool.topLevel) return { op: 'add', keyword: tool.keyword, name, top_level: true };
  const layer = LAYER_OF_VIEW[context.view];
  if (!layer) return { error: `${tool.label}: no layer for the ${context.view} view` };
  return { op: 'add', keyword: tool.keyword, name, layer };
}

export interface ConnectEnd {
  id: string;
  kind: string;
}

/** A port id is `Owner.port`; its owner is the part before the last dot. */
export function ownerOfPort(portId: string): string {
  const at = portId.lastIndexOf('.');
  return at > 0 ? portId.slice(0, at) : portId;
}

/** A name that can serve as a port name: letters, digits and underscores. */
export function portNameFrom(name: string): string {
  const cleaned = name
    .trim()
    .replace(/[^A-Za-z0-9_]+/g, '_')
    .replace(/^_+|_+$/g, '');
  return cleaned || 'flow';
}

/**
 * The edits that connect two ends: exchanges between functions or components
 * go through ports, created when the ends are not ports already; links and
 * interactions name the elements directly. Applied in order on one text.
 */
export function connectPlan(tool: Tool, from: ConnectEnd, to: ConnectEnd, name: string, view: string, graphContainer?: string | null): EditRequest[] | { error: string } {
  const label = name.trim();
  if (!label && tool.connect !== 'transition') return { error: 'a name is needed' };
  if (tool.endKinds && (!tool.endKinds.includes(from.kind) || !tool.endKinds.includes(to.kind))) {
    return { error: `${tool.label} connects ${tool.endKinds.join(', ')}; not ${from.kind} to ${to.kind}` };
  }
  if (from.id === to.id) return { error: 'an element is not connected to itself' };
  const layer = LAYER_OF_VIEW[view];
  const quoted = stringLiteral(label);
  const throughPorts = (fromKey: string, toKey: string, keyword: string): EditRequest[] => {
    const port = portNameFrom(label);
    const edits: EditRequest[] = [];
    const isPort = (end: ConnectEnd) => end.kind === 'FunctionPort' || end.kind === 'ComponentPort' || end.kind === 'PhysicalPort';
    const fromPort = isPort(from) ? from.id : `${from.id}.${port}`;
    const toPort = isPort(to) ? to.id : `${to.id}.${port}`;
    if (!isPort(from)) edits.push({ op: 'add', keyword: 'port out', name: port, container: from.id });
    if (!isPort(to)) edits.push({ op: 'add', keyword: 'port in', name: port, container: to.id });
    const declaration = `${keyword} ${quoted} {\n  ${fromKey}: ${stringLiteral(fromPort)}\n  ${toKey}: ${stringLiteral(toPort)}\n}`;
    edits.push(layer ? { op: 'add_raw', declaration, layer } : { op: 'add_raw', declaration, top_level: true });
    return edits;
  };
  const direct = (keyword: string, fromKey = 'from', toKey = 'to'): EditRequest[] => {
    const declaration = `${keyword} ${quoted} {\n  ${fromKey}: ${stringLiteral(from.id)}\n  ${toKey}: ${stringLiteral(to.id)}\n}`;
    return [layer ? { op: 'add_raw', declaration, layer } : { op: 'add_raw', declaration, top_level: true }];
  };
  switch (tool.connect) {
    case 'interaction':
      return direct('interaction');
    case 'functional_exchange':
      return throughPorts('from', 'to', 'functional_exchange');
    case 'component_exchange':
      return throughPorts('from_port', 'to_port', 'component_exchange');
    case 'physical_exchange':
      return throughPorts('from', 'to', 'physical_exchange');
    case 'physical_link':
      return direct('physical_link');
    case 'association':
      return [{ op: 'add_raw', declaration: `association ${quoted} {\n  from: ${stringLiteral(from.id)}\n  to: ${stringLiteral(to.id)}\n}`, top_level: true }];
    case 'transition': {
      // States are declared inside their machine: `transition Off -> Armed { guard: "..." }`.
      if (!graphContainer) return { error: 'a transition is drawn in the diagram of one state machine' };
      const guard = label ? ` guard: ${quoted} ` : ' ';
      const declaration = `transition ${bareOrQuoted(localName(from.id, graphContainer))} -> ${bareOrQuoted(localName(to.id, graphContainer))} {${guard}}`;
      return [{ op: 'add_raw', declaration, container: graphContainer }];
    }
    case 'trace': {
      // The name asked is the kind of the trace.
      const kind = label.toLowerCase();
      if (!TRACE_KINDS.includes(kind)) return { error: `a trace is one of ${TRACE_KINDS.join(', ')}` };
      return [{ op: 'add_trace', from: from.id, kind, to: to.id }];
    }
    case 'message': {
      if (!graphContainer) return { error: 'a message is drawn in the diagram of one scenario' };
      const declaration = `message ${stringLiteral(from.id)} -> ${stringLiteral(to.id)} ${quoted}`;
      return [{ op: 'add_raw', declaration, container: graphContainer }];
    }
    default:
      return { error: `${tool.label} connects nothing` };
  }
}

/** The kinds of traces the language knows (TraceKind of the metamodel). */
export const TRACE_KINDS = ['realizes', 'satisfies', 'implements', 'allocates', 'verifies', 'refines', 'validates'];

/** The name of a named diagram from its graph id (`lab:Lane keeping chain`), null for a viewpoint, a machine or a scenario. */
export function namedDiagramOf(graphId: string, view: string): string | null {
  if (view === 'msm' || view === 'es') return null;
  const at = graphId.indexOf(':');
  return at > 0 ? graphId.slice(at + 1) : null;
}

/** The ids drawn at the top of a graph (what a diagram's `shows:` lists). */
export function topLevelIds(graph: ViewerGraph): string[] {
  return (graph.children ?? []).map((node) => node.id);
}

/** A `shows:` list as ArcLang source. */
export function showsLiteral(ids: string[]): string {
  return `[${ids.map((id) => stringLiteral(id)).join(', ')}]`;
}

/** The part of an id after its owner's prefix (`LKA modes.Off` in `LKA modes` is `Off`). */
export function localName(id: string, owner: string): string {
  return id.startsWith(`${owner}.`) ? id.slice(owner.length + 1) : id;
}

/** A name written bare when it is an identifier, quoted otherwise. */
export function bareOrQuoted(name: string): string {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(name) ? name : stringLiteral(name);
}

/** The element a lifeline of a scenario stands for (`Lane departure.LC-CAM` is `LC-CAM`). */
export function lifelineElement(nodeId: string, scenario: string | null): string {
  return scenario ? localName(nodeId, scenario) : nodeId;
}

/** The element a diagram is about, from its graph id (`msm:LKA modes`, `es:Lane departure`). */
export function graphContainerOf(graphId: string, view: string): string | null {
  if (view !== 'msm' && view !== 'es') return null;
  const at = graphId.indexOf(':');
  return at > 0 ? graphId.slice(at + 1) : null;
}

/**
 * The connect tool a drag from one element to another means, on a view: the
 * first tool of the view whose ends accept both kinds (an exchange between
 * functions, a component exchange between components, a transition between
 * states...). Null when nothing connects these two.
 */
export function connectToolFor(view: string, fromKind: string, toKind: string): Tool | null {
  const accepts = (tool: Tool, kind: string) => !tool.endKinds || tool.endKinds.includes(kind);
  // A trace is chosen, never guessed: it is left to the armed Trace tool.
  return toolsFor(view).find((tool) => tool.kind === 'connect' && tool.connect !== 'trace' && accepts(tool, fromKind) && accepts(tool, toKind)) ?? null;
}

/** The kinds a drag from `fromKind` may land on, on a view (the union over the connect tools that accept the source). */
export function connectTargetKinds(view: string, fromKind: string): string[] | null {
  const tools = toolsFor(view).filter((tool) => tool.kind === 'connect' && tool.connect !== 'trace' && (!tool.endKinds || tool.endKinds.includes(fromKind)));
  if (tools.length === 0) return [];
  if (tools.some((tool) => !tool.endKinds)) return null;
  return [...new Set(tools.flatMap((tool) => tool.endKinds ?? []))];
}

/** The kinds that are functions, for the palette's own checks. */
export function isFunctionKind(kind: string): boolean {
  return FUNCTION_KINDS.includes(kind);
}

/** A short label for an element in lists and messages. */
export function elementLabel(element: Pick<ElementRecord, 'id' | 'name' | 'kind'>): string {
  return element.name && element.name !== element.id ? `${element.name} (${element.id})` : element.id;
}

/** The relationships and traces an element takes part in, for the semantic browser. */
export function neighbours(
  element: ElementRecord,
  snapshot: Snapshot,
): { incoming: { kind: string; name: string; other: ElementRecord | null }[]; outgoing: { kind: string; name: string; other: ElementRecord | null }[] } {
  const records = snapshot.elements?.elements ?? [];
  const byUuid = new Map(records.map((e) => [e.uuid, e]));
  const byId = new Map(records.map((e) => [e.id, e]));
  const incoming: { kind: string; name: string; other: ElementRecord | null }[] = [];
  const outgoing: { kind: string; name: string; other: ElementRecord | null }[] = [];
  for (const relationship of snapshot.elements?.relationships ?? []) {
    if (relationship.source === element.uuid) outgoing.push({ kind: relationship.kind, name: relationship.name, other: byUuid.get(relationship.target) ?? null });
    if (relationship.target === element.uuid) incoming.push({ kind: relationship.kind, name: relationship.name, other: byUuid.get(relationship.source) ?? null });
  }
  for (const link of snapshot.diagrams?.links ?? []) {
    if (link.source === element.id) outgoing.push({ kind: link.kind, name: '', other: byId.get(link.target) ?? null });
    if (link.target === element.id) incoming.push({ kind: link.kind, name: '', other: byId.get(link.source) ?? null });
  }
  return { incoming, outgoing };
}

/** The id after the highest `REQ-…-n` of the model: `REQ-LKA-003` after `REQ-LKA-002`. */
export function nextRequirementId(ids: string[]): string {
  let prefix = 'REQ-';
  let highest = 0;
  for (const id of ids) {
    const match = /^(.*?)(\d+)$/.exec(id);
    if (!match || !/^REQ/i.test(id)) continue;
    const n = Number(match[2]);
    if (n >= highest) {
      highest = n;
      prefix = match[1];
    }
  }
  const width = ids.find((id) => /^REQ/i.test(id))?.match(/(\d+)$/)?.[1].length ?? 3;
  return `${prefix}${String(highest + 1).padStart(width, '0')}`;
}

/** The requests that create a requirement and, when given, trace an element to it. */
export function requirementRequests(id: string, text: string, satisfiedBy: string | null, hasRequirementsBlock: boolean): EditRequest[] {
  const declaration = `requirement ${stringLiteral(id)} {\n  description: ${stringLiteral(text)}\n}`;
  const create: EditRequest = hasRequirementsBlock
    ? { op: 'add_raw', declaration, layer: 'requirements' }
    : { op: 'add_raw', declaration: `requirements {\n  ${declaration.replace(/\n/g, '\n  ')}\n}`, top_level: true };
  return satisfiedBy ? [create, { op: 'add_trace', from: satisfiedBy, kind: 'satisfies', to: id }] : [create];
}

/** A combined fragment around one message: `loop "every cycle"`, `opt`, `alt`, `par`, `critical`. */
export function fragmentRequest(scenario: string, message: string, answer: string): EditRequest | { error: string } {
  const [kind, ...rest] = answer.trim().split(/\s+/);
  const kinds = ['loop', 'opt', 'alt', 'par', 'seq', 'critical'];
  if (!kinds.includes((kind ?? '').toLowerCase())) return { error: `a fragment is one of ${kinds.join(', ')}, then an optional label` };
  return { op: 'wrap_fragment', scenario, kind: kind.toLowerCase(), label: rest.join(' '), first: message, last: message };
}

/** Where the arrangement of a model is kept, as the compiler reads it: `braking.arc` -> `braking.layout.json`. */
export function layoutPathOf(modelPath: string): string {
  const dot = modelPath.lastIndexOf('.');
  const slash = Math.max(modelPath.lastIndexOf('/'), modelPath.lastIndexOf('\\'));
  return `${dot > slash ? modelPath.slice(0, dot) : modelPath}.layout.json`;
}

/** A layout file as the viewer writes it: an object carrying `arclang_layout`. */
export function isLayoutFile(value: unknown): value is { arclang_layout: string | number } {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) return false;
  const version = (value as { arclang_layout?: unknown }).arclang_layout;
  return typeof version === 'string' || typeof version === 'number';
}
