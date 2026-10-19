// The workbench page: a 35 px bar (views and the viewer's tools), then a
// row with the palette rail, the canvas and the properties drawer, then one
// status line. Everything shown comes from the last snapshot the host sent;
// every change goes back to the host as `arclang edit` requests.
import { connectTargetKinds, connectToolFor, EditRequest, RelationshipRecord, fragmentRequest, namedDiagramOf, nextRequirementId, requirementRequests, Tool, neighbours, showsLiteral, topLevelIds, VIEWPOINTS, ElementRecord, Finding, graphContainerOf, lifelineElement, Metamodel, Snapshot, ToWebview } from '../src/pure/workbench';
import { SemanticBrowser } from './browser';
import { button, clear, el, inlinePrompt } from './dom';
import type { ViewerApi, ViewerPosition, ViewerSelection } from './globals';
import { Palette } from './palette';
import { Properties } from './properties';
import { createFind } from './find';
import { createMinimap } from './minimap';
import { Editing } from './editing';
import { createHelp } from './help';
import { modelIdOf, relationshipOfEdge } from '../src/pure/editing';

const vscode = acquireVsCodeApi();

interface Layout {
  paletteOpen: boolean;
  drawerOpen: boolean;
  drawerWidth: number;
  rightTab: number;
}

interface State {
  snapshot: Snapshot | null;
  metamodel: Metamodel | null;
  viewer: ViewerApi | null;
  selected: ElementRecord | null;
  /** The drawing id of what the viewer has selected: a box, a port or an exchange. */
  selectedDrawing: string | null;
  lastPosition: ViewerPosition | null;
  byId: Map<string, ElementRecord>;
  byName: Map<string, ElementRecord>;
  byUuid: Map<string, ElementRecord>;
  layout: Layout;
}

const savedLayout = (vscode.getState() as { layout?: Partial<Layout> } | undefined)?.layout ?? {};
const wide = window.innerWidth >= 1000;
const state: State = {
  snapshot: null,
  metamodel: null,
  viewer: null,
  selected: null,
  selectedDrawing: null,
  lastPosition: null,
  byId: new Map(),
  byName: new Map(),
  byUuid: new Map(),
  layout: {
    paletteOpen: savedLayout.paletteOpen ?? wide,
    drawerOpen: savedLayout.drawerOpen ?? window.innerWidth >= 760,
    drawerWidth: Math.min(480, Math.max(240, savedLayout.drawerWidth ?? 280)),
    rightTab: savedLayout.rightTab ?? 0,
  },
};

function saveLayout(): void {
  vscode.setState({ layout: state.layout });
}

// --- Layout -----------------------------------------------------------------

const root = document.getElementById('workbench')!;
const bar = el('header', { class: 'wb-bar' });
const layerBar = el('nav', { class: 'wb-layers', 'aria-label': 'Arcadia layers' });
const toolRow = el('div', { class: 'wb-toolrow' });
const tabs = el('div', { class: 'wb-tabs wb-hidden' });
const tools = el('div', { class: 'wb-viewer-tools' });
const paletteToggle = button('', () => setPalette(!state.layout.paletteOpen), { class: 'wb-icon-button', title: 'Palette' });
const drawerToggle = button('', () => setDrawer(!state.layout.drawerOpen), { class: 'wb-icon-button', title: 'Properties' });
const minimapToggle = button('', () => toggleMinimap(), { class: 'wb-icon-button wb-minimap-toggle', title: 'Minimap (Shift+M)' });
toolRow.append(paletteToggle, tools, minimapToggle, drawerToggle);
bar.append(layerBar, toolRow);

const body = el('div', { class: 'wb-body' });
const left = el('aside', { class: 'wb-left' });
const sheet = el('main', { class: 'wb-sheet' });
const host = el('div', { class: 'wb-host' });
const modeChip = el('div', { class: 'wb-mode', hidden: true });
const toast = el('div', { class: 'wb-toast', hidden: true, role: 'status', 'aria-live': 'polite' });
const progress = el('div', { class: 'wb-progress', hidden: true });
const layoutNotice = el('div', { class: 'wb-notice', hidden: true, role: 'status' });
sheet.append(progress, host, modeChip, toast, layoutNotice);
const splitter = el('div', { class: 'wb-splitter', role: 'separator', 'aria-orientation': 'vertical', title: 'Drag to resize' });
const right = el('aside', { class: 'wb-right' });
const bottom = el('footer', { class: 'wb-bottom' });
const hiddenTree = el('div', { class: 'wb-hidden' });
const hiddenFoot = el('div', { class: 'wb-hidden' });
const detailsSlot = el('div', { class: 'wb-details' });
const relatedSlot = el('div', { class: 'wb-related' });

const rightTabs = el('div', { class: 'wb-right-tabs', role: 'tablist' });
const propertiesPane = el('div', { class: 'wb-pane' });
const browserPane = el('div', { class: 'wb-pane' });
const detailsPane = el('div', { class: 'wb-pane' }, [detailsSlot, relatedSlot]);
const panes: [string, HTMLElement][] = [
  ['Properties', propertiesPane],
  ['Relations', browserPane],
  ['Details', detailsPane],
];
panes.forEach(([label, pane], index) => {
  const tab = button(label, () => showRightTab(index), { class: 'wb-right-tab', role: 'tab' });
  rightTabs.appendChild(tab);
  pane.setAttribute('role', 'tabpanel');
});
function showRightTab(index: number): void {
  state.layout.rightTab = index;
  panes.forEach(([, pane], i) => pane.classList.toggle('is-active', i === index));
  [...rightTabs.children].forEach((tab, i) => {
    tab.classList.toggle('is-active', i === index);
    tab.setAttribute('aria-selected', String(i === index));
  });
  saveLayout();
}
right.append(rightTabs, propertiesPane, browserPane, detailsPane);
body.append(left, sheet, splitter, right);
root.append(bar, body, bottom, hiddenTree, hiddenFoot);
showRightTab(state.layout.rightTab);

function setPalette(open: boolean): void {
  state.layout.paletteOpen = open;
  left.classList.toggle('is-collapsed', !open);
  paletteToggle.setAttribute('aria-pressed', String(open));
  palette.setCompact(!open);
  saveLayout();
  fitStage();
}

function setDrawer(open: boolean): void {
  state.layout.drawerOpen = open;
  right.hidden = !open;
  splitter.hidden = !open;
  right.style.width = `${state.layout.drawerWidth}px`;
  drawerToggle.setAttribute('aria-pressed', String(open));
  saveLayout();
  fitStage();
}

// The drawer is resized by dragging the splitter; the width is remembered.
splitter.addEventListener('pointerdown', (down) => {
  down.preventDefault();
  splitter.setPointerCapture(down.pointerId);
  const startX = down.clientX;
  const startWidth = state.layout.drawerWidth;
  const move = (event: PointerEvent) => {
    state.layout.drawerWidth = Math.min(480, Math.max(240, startWidth - (event.clientX - startX)));
    right.style.width = `${state.layout.drawerWidth}px`;
    fitStage();
  };
  const up = () => {
    splitter.removeEventListener('pointermove', move);
    splitter.removeEventListener('pointerup', up);
    saveLayout();
  };
  splitter.addEventListener('pointermove', move);
  splitter.addEventListener('pointerup', up);
});

/** The viewer sizes its stage from a variable: it gets what the sheet has left under its own chrome. */
function fitStage(): void {
  const stage = host.querySelector<HTMLElement>('.arcviz-stage');
  if (!stage) return;
  const above = stage.getBoundingClientRect().top - sheet.getBoundingClientRect().top;
  let below = 0;
  let after = stage.nextElementSibling as HTMLElement | null;
  while (after) {
    if (!after.hidden) below += after.getBoundingClientRect().height;
    after = after.nextElementSibling as HTMLElement | null;
  }
  const height = Math.max(160, sheet.clientHeight - above - below);
  root.style.setProperty('--av-stage-height', `${Math.floor(height)}px`);
}
new ResizeObserver(() => fitStage()).observe(sheet);

const palette = new Palette(left, {
  selected: () => (state.selected ? { id: state.selected.id, kind: state.selected.kind } : null),
  submit,
  hint: (text) => showMode(text),
  anchor: () => sheet,
  special: (tool) => specialTool(tool),
});

/** A requirement satisfied by the selection; a fragment around the selected message. */
function specialTool(tool: Tool): void {
  const records = state.snapshot?.elements?.elements ?? [];
  if (tool.special === 'requirement') {
    const id = nextRequirementId(records.filter((e) => e.kind === 'Requirement').map((e) => e.id));
    const by = state.selected && state.selected.kind !== 'Requirement' ? state.selected : null;
    void inlinePrompt(sheet, `New requirement ${id}${by ? `, satisfied by ${by.name || by.id}` : ''}`, '', 'The system shall…').then((text) => {
      if (text === null || !text.trim()) return;
      const hasBlock = records.some((e) => e.kind === 'Requirement');
      submit(requirementRequests(id, text.trim(), by?.id ?? null, hasBlock), `requirement ${id}`);
    });
    return;
  }
  if (tool.special === 'fragment') {
    const graph = currentGraph();
    const scenario = graph ? graphContainerOf(graph.id, graph.arc.view) : null;
    const message = state.selected && state.selected.kind === 'Message' ? state.selected.name : null;
    if (!scenario || !message) return showMode('Fragment: select a message of the scenario first');
    void inlinePrompt(sheet, `Fragment around "${message}"`, 'loop', 'loop | opt | alt | par | critical, then a label').then((answer) => {
      if (answer === null) return;
      const request = fragmentRequest(scenario, message, answer);
      if ('error' in request) return showToast(request.error, 'error');
      submit([request], `${answer.trim().split(/\s+/)[0]} around ${message}`);
    });
  }
}
const properties = new Properties(propertiesPane, {
  submit,
  reveal: (id) => vscode.postMessage({ type: 'reveal', id }),
  ask: (element) => vscode.postMessage({ type: 'ask', id: element.id, name: element.name, kind: element.kind }),
  diagram: (op, element) => (op === 'hide' ? hideFromDiagram(element) : op === 'related' ? showRelated(element) : newDiagramAround(element)),
  requirements: () => (state.snapshot?.elements?.elements ?? []).filter((e) => e.kind === 'Requirement'),
  ends: (element) => editing.endsOf(element),
  reverse: (element) => editing.reverse(element),
});
const browser = new SemanticBrowser(browserPane, (element) => selectElement(element, true), (element, other, outgoing) => editing.retargetTrace(element, other, outgoing));
setPalette(state.layout.paletteOpen);
setDrawer(state.layout.drawerOpen);

// --- Feedback on the canvas --------------------------------------------------

function showMode(text: string): void {
  modeChip.textContent = text;
  modeChip.hidden = !text;
}

let toastTimer: ReturnType<typeof setTimeout> | null = null;

/** A message anchored at the bottom of the canvas, with optional actions. */
function showToast(text: string, kind: 'info' | 'error', actions: [string, () => void][] = [], ms = kind === 'error' ? 0 : 6000): void {
  clear(toast);
  toast.className = `wb-toast is-${kind}`;
  toast.appendChild(el('span', { class: 'wb-toast-text', text }));
  for (const [label, run] of actions) toast.appendChild(button(label, run, { class: 'wb-link' }));
  toast.appendChild(button('✕', () => (toast.hidden = true), { class: 'wb-icon-button', title: 'Dismiss' }));
  toast.hidden = false;
  if (toastTimer) clearTimeout(toastTimer);
  if (ms > 0) toastTimer = setTimeout(() => (toast.hidden = true), ms);
}

// --- Snapshot and viewer ----------------------------------------------------

function submit(requests: EditRequest[], label: string): void {
  vscode.postMessage({ type: 'edit', requests, label });
}

const modelId = modelIdOf;

/**
 * The drawing id of a model element: its id when the drawing uses it, else
 * its name (an element that writes no id is drawn under its name), ports as
 * `Owner::port`.
 */
function drawingId(id: string): string {
  const element = state.byId.get(id);
  if (element && /Port$/.test(element.kind)) {
    const at = id.lastIndexOf('.');
    if (at > 0) return `${drawingId(id.slice(0, at))}::${id.slice(at + 1)}`;
  }
  const drawn = (candidate: string) => !!host.querySelector(`[data-id="${CSS.escape(candidate)}"]`);
  if (drawn(id) || !element) return id;
  if (element.name && drawn(element.name)) return element.name;
  return id;
}

function currentGraph() {
  const current = state.viewer?.current?.();
  return state.snapshot?.diagrams?.graphs.find((g) => g.id === current?.view) ?? null;
}

function elementFor(selection: ViewerSelection | null): ElementRecord | null {
  if (!selection || !state.snapshot?.elements) return null;
  // An edge is the relationship of its kind between its two ends (a message, a transition, an exchange).
  const edgeGraph = currentGraph();
  const drawn = edgeGraph && (edgeGraph.edges ?? []).some((e) => e.id === selection.id) ? relationshipOfEdge(selection.id, edgeGraph, state.snapshot, selection.name || undefined) : null;
  if (drawn) return relationshipElement(drawn);
  const id = modelId(selection.id);
  const found = state.byId.get(id) ?? (selection.name ? state.byName.get(selection.name) : undefined);
  if (found) return found;
  // A lifeline of a scenario stands for its participant.
  const graph = currentGraph();
  if (graph) {
    const participant = state.byId.get(lifelineElement(id, graphContainerOf(graph.id, graph.arc.view)));
    if (participant) return participant;
  }
  // An edge is drawn for a relationship (`ce:LaneFlow`): the model lists it by name.
  const bare = selection.id.replace(/^[a-z]+:/, '');
  const named = state.byName.get(bare) ?? state.byId.get(bare);
  if (named) return named;
  if (!selection.isEdge) return null;
  // Exchanges, links and traces are relationships: selectable by their name.
  const relationship = (state.snapshot.elements.relationships ?? []).find((r) => r.name === bare || r.name === selection.name);
  return relationship ? relationshipElement(relationship) : null;
}

/** A relationship as the views show an element: designated by its written id, else its name. */
function relationshipElement(r: RelationshipRecord): ElementRecord {
  const attributes = r.attributes ?? {};
  const id = typeof attributes.id === 'string' ? attributes.id : r.name || r.kind;
  return { uuid: r.uuid, id, kind: r.kind, name: r.name ?? '', owner: null, attributes };
}

function selectElement(element: ElementRecord | null, inViewer: boolean): void {
  state.selected = element;
  properties.show(element, state.metamodel);
  browser.show(element, state.snapshot);
  vscode.postMessage({ type: 'selected', id: element?.id ?? null, kind: element?.kind, name: element?.name });
  if (inViewer && element) selectInViewer(element.id);
}

// --- Multi-selection (Shift+click) ------------------------------------------

let shiftDown = false;
host.addEventListener('pointerdown', (event) => (shiftDown = event.shiftKey), true);
/** The elements selected together, the primary one included; empty when one or none is selected. */
let multi: ElementRecord[] = [];

function markMulti(): void {
  for (const node of host.querySelectorAll('.wb-multi')) node.classList.remove('wb-multi');
  for (const element of multi) host.querySelector(`[data-id="${CSS.escape(drawingId(element.id))}"]`)?.classList.add('wb-multi');
  multiBar.hidden = multi.length < 2;
  multiCount.textContent = `${multi.length} selected`;
  if (multi.length > 1) properties.showMany(multi, state.metamodel);
}

/** The selected elements without those another selected element owns (removing the owner removes them). */
function outermost(elements: ElementRecord[]): ElementRecord[] {
  const uuids = new Set(elements.map((e) => e.uuid));
  const ownedBySelection = (element: ElementRecord): boolean => {
    let owner = element.owner ? state.byUuid.get(element.owner) : undefined;
    while (owner) {
      if (uuids.has(owner.uuid)) return true;
      owner = owner.owner ? state.byUuid.get(owner.owner) : undefined;
    }
    return false;
  };
  return elements.filter((e) => !ownedBySelection(e));
}

function deleteMany(elements: ElementRecord[]): void {
  const targets = outermost(elements);
  submit(targets.map((e) => ({ op: 'remove', element: e.id })), `delete ${targets.length} elements`);
  multi = [];
  markMulti();
}

function hideMany(elements: ElementRecord[]): void {
  const graph = currentGraph();
  if (!graph) return;
  const ids = new Set(elements.map((e) => e.id));
  const drawnAtTop = topLevelIds(graph).map((id) => elementFor({ view: '', id, name: '', kind: '', isEdge: false })?.id).filter((id): id is string => !!id);
  const rest = drawnAtTop.filter((id) => !ids.has(id));
  const named = namedDiagramOf(graph.id, graph.arc.view);
  if (named) submit([{ op: 'set', element: named, key: 'shows', value: showsLiteral(rest) }], `hide ${ids.size} elements from ${named}`);
  else {
    const name = uniqueDiagramName(`${graph.arc.title} (custom)`);
    pendingView = `${graph.arc.view}:${name}`;
    submit([{ op: 'add_raw', declaration: `diagram ${JSON.stringify(name)} {\n  view: ${graph.arc.view}\n  shows: ${showsLiteral(rest)}\n}`, top_level: true }], `new diagram ${name} without ${ids.size} elements`);
  }
  multi = [];
  markMulti();
}

const multiBar = el('div', { class: 'wb-multibar', hidden: true, role: 'toolbar', 'aria-label': 'Selection' });
const multiCount = el('strong', { text: '' });
multiBar.append(
  multiCount,
  button('Delete all', () => deleteMany(multi), { class: 'wb-danger', title: 'Delete every selected element (one undo)' }),
  button('Hide all', () => hideMany(multi), { title: 'Hide every selected element from this diagram' }),
  button('Clear', () => {
    multi = [];
    markMulti();
  }),
);
sheet.appendChild(multiBar);

/** A selection the page asks of the viewer: its empty answer while the drawing settles is not the reader's. */
let programmaticUntil = 0;

// The reader's own clicks and keys always count.
host.addEventListener('pointerdown', () => (programmaticUntil = 0), true);
host.addEventListener('keydown', () => (programmaticUntil = 0), true);

function selectInViewer(id: string): void {
  programmaticUntil = Date.now() + 2000;
  // A relationship is drawn under its edge id, kept from the reader's click.
  const edge = state.selected?.id === id && !state.byId.has(id) ? state.selectedDrawing : null;
  state.viewer?.select?.(edge ?? drawingId(id));
}

function onSelect(selection: ViewerSelection | null): void {
  if (selection === null && Date.now() < programmaticUntil && state.selected) return;
  programmaticUntil = 0;
  state.selectedDrawing = selection?.id ?? null;
  const element = elementFor(selection);
  if (shiftDown && element && !selection?.isEdge) {
    const primary = state.selected;
    const base = multi.length ? multi : primary && primary.uuid !== element.uuid ? [primary] : [];
    multi = base.some((e) => e.uuid === element.uuid) ? base.filter((e) => e.uuid !== element.uuid) : [...base, element];
    shiftDown = false;
    setTimeout(markMulti, 0);
    selectElement(element, false);
    return;
  }
  if (multi.length) {
    multi = [];
    setTimeout(markMulti, 0);
  }
  // A connect tool takes the click as one of its ends.
  if (palette.pick(element ? { id: element.id, kind: element.kind } : null)) return;
  selectElement(element, false);
}

/** A box let go on another box: it goes into it (the viewer places it otherwise). */
function onDrop({ from: dragged, to: target }: { view: string; from: string; to: string | null }): void {
  const element = elementFor({ view: '', id: dragged, name: '', kind: '', isEdge: false });
  if (!element) return;
  const container = target ? elementFor({ view: '', id: target, name: '', kind: '', isEdge: false }) : null;
  if (!container || container.uuid === element.owner) return;
  submit([{ op: 'move', element: element.id, container: container.id }], `move ${element.id} into ${container.id}`);
}

function countNodes(nodes: NonNullable<ReturnType<typeof currentGraph>>['children'] & unknown[]): number {
  return nodes.reduce((sum, node) => sum + 1 + countNodes(node.children ?? []), 0);
}

// --- Diagram contents (Capella's Insert/Remove, SysON's Manage visibility) ---

/** Hide an element from the diagram on screen: out of a named diagram's `shows:`, or a new named diagram without it. */
function hideFromDiagram(element: ElementRecord): void {
  const graph = currentGraph();
  if (!graph) return;
  const drawnAtTop = topLevelIds(graph).map((id) => elementFor({ view: '', id, name: '', kind: '', isEdge: false })?.id).filter((id): id is string => !!id);
  if (!drawnAtTop.includes(element.id)) {
    showToast(`${element.name || element.id} is drawn inside another box: hide that box, or open it as a diagram.`, 'info');
    return;
  }
  const rest = drawnAtTop.filter((id) => id !== element.id);
  const named = namedDiagramOf(graph.id, graph.arc.view);
  if (named) {
    submit([{ op: 'set', element: named, key: 'shows', value: showsLiteral(rest) }], `hide ${element.id} from ${named}`);
    return;
  }
  const name = uniqueDiagramName(`${graph.arc.title} (custom)`);
  pendingView = `${graph.arc.view}:${name}`;
  submit([{ op: 'add_raw', declaration: `diagram ${JSON.stringify(name)} {\n  view: ${graph.arc.view}\n  shows: ${showsLiteral(rest)}\n}`, top_level: true }], `new diagram ${name} without ${element.id}`);
}

/** Add what an element is connected to into the named diagram on screen (or a new one). */
function showRelated(element: ElementRecord): void {
  const graph = currentGraph();
  if (!graph || !state.snapshot) return;
  const around = neighbours(element, state.snapshot);
  const related = [...around.incoming, ...around.outgoing].map((n) => n.other?.id).filter((id): id is string => !!id);
  const named = namedDiagramOf(graph.id, graph.arc.view);
  const current = named ? topLevelIds(graph).map((id) => elementFor({ view: '', id, name: '', kind: '', isEdge: false })?.id).filter((id): id is string => !!id) : [];
  const shows = [...new Set([...current, element.id, ...related])];
  if (named) {
    if (shows.length === current.length) return showToast('Everything connected to it is already on this diagram.', 'info');
    submit([{ op: 'set', element: named, key: 'shows', value: showsLiteral(shows) }], `show what ${element.id} is connected to`);
    return;
  }
  newDiagramAround(element);
}

/** A new named diagram with the element and what it is connected to (Capella's contextual diagram). */
function newDiagramAround(element: ElementRecord): void {
  const graph = currentGraph();
  if (!graph || !state.snapshot) return;
  const around = neighbours(element, state.snapshot);
  const related = [...around.incoming, ...around.outgoing].map((n) => n.other?.id).filter((id): id is string => !!id);
  const view = ['oab', 'sab', 'lab', 'pab', 'cap', 'cdb'].includes(graph.arc.view) ? graph.arc.view : 'lab';
  const name = uniqueDiagramName(`${element.name || element.id} context`);
  pendingView = `${view}:${name}`;
  submit([{ op: 'add_raw', declaration: `diagram ${JSON.stringify(name)} {\n  view: ${view}\n  shows: ${showsLiteral([...new Set([element.id, ...related])])}\n  context: true\n}`, top_level: true }], `new diagram ${name}`);
}

function uniqueDiagramName(base: string): string {
  const taken = new Set((state.snapshot?.diagrams?.graphs ?? []).map((g) => g.arc.title));
  let name = base;
  for (let n = 2; taken.has(name); n++) name = `${base} ${n}`;
  return name;
}

/** The diagram to show once the snapshot that creates it arrives. */
let pendingView: string | null = null;

// --- Direct edit and find -----------------------------------------------------


const finder = createFind(sheet, host, {
  graph: () => currentGraph(),
  elementFor: (id) => elementFor({ view: '', id, name: '', kind: '', isEdge: false }),
  select: (id, element) => {
    state.viewer?.select?.(id);
    if (element) selectElement(element, false);
  },
});
const minimap = createMinimap(sheet, host, { viewer: () => state.viewer, key: () => `${currentGraph()?.id}:${state.snapshot?.elements?.elements.length}` });
const toggleMinimap = (force?: boolean) => minimap.toggle(force);
const openFind = () => finder.open();
const help = createHelp(sheet);
const editing = new Editing({
  host,
  sheet,
  snapshot: () => state.snapshot,
  viewer: () => state.viewer,
  graph: () => currentGraph(),
  selected: () => state.selected,
  selectedDrawing: () => state.selectedDrawing,
  multi: () => multi,
  setMulti: (elements) => {
    multi = elements;
    if (elements.length) selectElement(elements[elements.length - 1], false);
    markMulti();
  },
  elementFor: (id, isEdge = false, name = '') => elementFor({ view: '', id, name, kind: '', isEdge }),
  drawingId,
  outermost,
  submit,
  toast: (text, kind) => showToast(text, kind),
  select: selectElement,
  remove: (elements) => (elements.length > 1 ? deleteMany(elements) : elements[0] && deleteOne(elements[0])),
  hide: (elements) => (elements.length > 1 ? hideMany(elements) : elements[0] && hideFromDiagram(elements[0])),
  related: showRelated,
  newDiagram: newDiagramAround,
  reveal: (element) => vscode.postMessage({ type: 'reveal', id: element.id }),
  help: () => help.toggle(),
});

/** Exports go to the host, which asks where to save them (a webview cannot download). */
function onDownload(blob: Blob, filename: string): void {
  const reader = new FileReader();
  reader.onload = () => {
    const url = String(reader.result);
    const data = url.slice(url.indexOf(',') + 1);
    vscode.postMessage({ type: 'save', name: filename, mime: blob.type, data });
  };
  reader.onerror = () => showToast(`The export ${filename} could not be read.`, 'error');
  reader.readAsDataURL(blob);
}

/**
 * Full screen in an editor: the canvas takes the page (palette, drawer and
 * status line hidden) and the host maximizes the editor; Esc or the button leaves.
 */
function setFocusMode(on: boolean): void {
  document.body.classList.toggle('wb-focus', on);
  vscode.postMessage({ type: 'focus-mode', on });
  fitStage();
  setTimeout(() => {
    fitStage();
    state.viewer?.fit?.();
  }, 250);
}

/** The drawn element ids a connection from `fromId` may land on (null: any). */
function connectTargets(fromId: string): string[] | null {
  const from = elementFor({ view: '', id: fromId, name: '', kind: '', isEdge: false });
  const graph = currentGraph();
  if (!from || !graph) return [];
  // An armed Trace tool connects any two elements.
  if (palette.armedConnectTool()?.connect === 'trace') return null;
  const kinds = connectTargetKinds(graph.arc.view, from.kind);
  if (kinds === null) return null;
  const wanted = new Set(kinds);
  const ids: string[] = [];
  const walk = (nodes: NonNullable<typeof graph.children>) => {
    for (const node of nodes) {
      const element = elementFor({ view: '', id: node.id, name: '', kind: '', isEdge: false });
      if (element && wanted.has(element.kind) && element.id !== from.id) ids.push(node.id);
      for (const port of node.ports ?? []) {
        const portElement = elementFor({ view: '', id: port.id, name: '', kind: '', isEdge: false });
        if (portElement && wanted.has(portElement.kind)) ids.push(port.id);
      }
      walk(node.children ?? []);
    }
  };
  walk(graph.children ?? []);
  return ids;
}

/** A drag from a handle to another element: the armed connect tool, else the one these kinds mean. */
function onConnect(fromId: string, toId: string): void {
  const from = elementFor({ view: '', id: fromId, name: '', kind: '', isEdge: false });
  const to = elementFor({ view: '', id: toId, name: '', kind: '', isEdge: false });
  const graph = currentGraph();
  if (!from || !to || !graph) return;
  const tool = palette.armedConnectTool() ?? connectToolFor(graph.arc.view, from.kind, to.kind);
  if (!tool) {
    showToast(`Nothing connects ${from.kind} to ${to.kind} on this view.`, 'info');
    return;
  }
  palette.connectWith(tool, { id: from.id, kind: from.kind }, { id: to.id, kind: to.kind });
}

/** Double-click opens the element: its properties, and its declaration in the source. */
function onOpen(id: string): void {
  const element = elementFor({ view: '', id, name: '', kind: '', isEdge: false });
  if (!element) return;
  selectElement(element, false);
  if (!state.layout.drawerOpen) setDrawer(true);
  showRightTab(0);
  vscode.postMessage({ type: 'reveal', id: element.id });
}

function index(snapshot: Snapshot): void {
  const records = snapshot.elements?.elements ?? [];
  state.byId = new Map(records.map((e) => [e.id, e]));
  state.byName = new Map(records.map((e) => [e.name, e]));
  state.byUuid = new Map(records.map((e) => [e.uuid, e]));
}

function mount(snapshot: Snapshot): void {
  const previous = state.viewer?.current?.() ?? state.lastPosition;
  clear(tabs);
  clear(tools);
  clear(hiddenTree);
  clear(hiddenFoot);
  clear(detailsSlot);
  clear(relatedSlot);
  if (!snapshot.diagrams) {
    state.viewer = null;
    clear(host).appendChild(el('p', { class: 'wb-empty', text: 'Nothing to draw: the model does not compile yet.' }));
    return;
  }
  const deps = {
    ELK: window.ELK,
    render: window.ArcVizRender,
    links: window.ArcVizLinks,
    navigation: window.ArcVizNavigation,
    collapse: window.ArcVizCollapse,
    scope: window.ArcVizScope,
    exporter: window.ArcVizExport,
    stage: window.ArcVizStage,
    place: window.ArcVizPlace,
    edit: window.ArcVizEdit,
  };
  progress.hidden = false;
  remounting = true;
  const generation = ++mountGeneration;
  // Past three seconds (NN/g's limit of attention), the page says what it is doing.
  const graphs = snapshot.diagrams.graphs;
  const boxes = graphs.reduce((sum, g) => sum + countNodes(g.children ?? []), 0);
  const slow = setTimeout(() => {
    layoutNotice.textContent = `Laying out ${graphs.length} diagram${graphs.length > 1 ? 's' : ''}, ${boxes} boxes…`;
    layoutNotice.hidden = false;
  }, 3000);
  const options = {
    slots: { tabs, tools, tree: hiddenTree, details: detailsSlot, related: relatedSlot, foot: hiddenFoot },
    onSelect,
    onDrop,
    onOpen,
    onConnect,
    connectTargets,
    onDownload,
    onFullscreen: () => setFocusMode(!document.body.classList.contains('wb-focus')),
    // Boxes go where they are dragged (into another box when let go on it), kept next to the model.
    arrange: true,
    onLayout: (file: unknown) => vscode.postMessage({ type: 'layout', file }),
    // Editing from the drawing: double-click renames, ends reconnect, bands select, arrows connect.
    edit: editing.viewerOptions(),
  };
  let viewer: ReturnType<typeof window.ArcViz.mount>;
  try {
    viewer = window.ArcViz.mount(host, snapshot.diagrams, deps, options);
  } catch (error) {
    if (!snapshot.diagrams.layout) throw error;
    // A layout file this page cannot use: the automatic layout, and the reason.
    viewer = window.ArcViz.mount(host, { ...snapshot.diagrams, layout: undefined }, deps, options);
    showToast(`The layout file next to the model could not be used (${error instanceof Error ? error.message : String(error)}): the automatic layout is shown.`, 'error');
  }
  state.viewer = viewer;
  fitStage();
  void viewer.ready.then(() => {
    // A later snapshot remounted the viewer meanwhile: that mount restores the view.
    if (generation !== mountGeneration) return;
    markFindings();
    if (previous?.view && viewer.show) {
      const known = viewer.views?.().some((v) => v.id === previous.view);
      if (known) viewer.show(previous.view);
    }
    if (previous?.position && viewer.restore) viewer.restore(previous.position);
    if (state.selected) selectInViewer(state.selected.id);
    remounting = false;
    shownGraph = null;
    followView();
    fitStage();
    progress.hidden = true;
    clearTimeout(slow);
    layoutNotice.hidden = true;
    void viewer.settled?.()?.then(() => {
      followView();
      fitStage();
    });
  });
}

/** Nodes with findings are marked on the drawing, with the findings as their tooltip. */
function markFindings(): void {
  const findings = state.snapshot?.findings ?? [];
  if (findings.length === 0) return;
  const byElement = new Map<string, string[]>();
  for (const finding of findings) {
    if (!finding.element) continue;
    const element = state.byId.get(finding.element) ?? state.byName.get(finding.element);
    const key = element ? drawingId(element.id) : finding.element;
    byElement.set(key, [...(byElement.get(key) ?? []), `${finding.severity}: ${finding.rule}: ${finding.message}`]);
  }
  for (const [id, lines] of byElement) {
    const node = host.querySelector<SVGGElement>(`[data-id="${CSS.escape(id)}"]`);
    if (!node || node.classList.contains('wb-has-finding')) continue;
    node.classList.add('wb-has-finding');
    if (lines.some((l) => l.startsWith('error'))) node.classList.add('is-error');
    const title = document.createElementNS('http://www.w3.org/2000/svg', 'title');
    title.textContent = lines.join('\n');
    node.prepend(title);
  }
}


/** The layers of the model, one big button each, the one on screen pressed; several diagrams of a layer unfold in a list. */
function renderLayers(): void {
  clear(layerBar);
  const graphs = state.snapshot?.diagrams?.graphs ?? [];
  if (graphs.length === 0) return;
  const current = state.viewer?.current?.()?.view ?? null;
  for (const viewpoint of VIEWPOINTS) {
    const own = graphs.filter((g) => g.arc.view === viewpoint.view);
    if (own.length === 0) continue;
    const active = own.some((g) => g.id === current);
    const item = button('', () => showGraph(own[0].id), { class: `wb-layer${active ? ' is-active' : ''}`, 'aria-pressed': String(active), title: `${viewpoint.label}${own.length > 1 ? ` (${own.length} diagrams)` : ''}` });
    item.append(el('span', { class: 'wb-layer-name', text: viewpoint.short }));
    if (own.length > 1) item.appendChild(el('span', { class: 'wb-layer-count', text: String(own.length) }));
    layerBar.appendChild(item);
  }
  const extra = graphs.filter((g) => !VIEWPOINTS.some((v) => v.view === g.arc.view));
  if (extra.length > 0) {
    const active = extra.some((g) => g.id === current);
    layerBar.appendChild(button(`Other (${extra.length})`, () => showGraph(extra[0].id), { class: `wb-layer${active ? ' is-active' : ''}`, 'aria-pressed': String(active) }));
  }
}

function showGraph(id: string): void {
  const viewer = state.viewer;
  if (!viewer?.show) return;
  viewer.show(id);
  setTimeout(() => {
    followView();
    viewer.fit?.();
  }, 0);
}

/** The next or previous layer that has a diagram (⌘⌥] and ⌘⌥[ in the editor). */
function stepLayer(step: number): void {
  const graphs = state.snapshot?.diagrams?.graphs ?? [];
  const views = VIEWPOINTS.map((v) => v.view).filter((view) => graphs.some((g) => g.arc.view === view));
  if (views.length === 0) return;
  const current = currentGraph()?.arc.view ?? views[0];
  const at = Math.max(0, views.indexOf(current));
  const next = views[(at + step + views.length) % views.length];
  const graph = graphs.find((g) => g.arc.view === next);
  if (graph) showGraph(graph.id);
}

let shownGraph: string | null = null;
/** While the viewer is remounted on a new snapshot, the view on screen is not the reader's yet. */
let remounting = false;
let mountGeneration = 0;

/** What follows the view on screen: the palette, and a selection that is not drawn there. */
function followView(): void {
  const graph = currentGraph();
  const key = graph ? graph.id : null;
  if (remounting || key === shownGraph) return;
  shownGraph = key;
  renderLayers();
  palette.show(graph?.arc.view ?? '', graph ? graphContainerOf(graph.id, graph.arc.view) : null);
  if (state.selected && graph && !drawn(graph, state.selected)) selectElement(null, false);
  fitStage();
  setTimeout(() => {
    markFindings();
  }, 200);
}

/** Whether an element has a node, a port or a lifeline on the graph. */
function drawn(graph: NonNullable<ReturnType<typeof currentGraph>>, element: ElementRecord): boolean {
  if (!state.byUuid.has(element.uuid)) return (graph.edges ?? []).some((e) => e.id === state.selectedDrawing);
  const wanted = new Set([element.id, drawingId(element.id)]);
  const container = graphContainerOf(graph.id, graph.arc.view);
  if (container) wanted.add(`${container}.${element.id}`);
  const walk = (nodes: NonNullable<typeof graph.children>): boolean =>
    nodes.some((node) => wanted.has(node.id) || (node.ports ?? []).some((p) => wanted.has(p.id)) || walk(node.children ?? []));
  return walk(graph.children ?? []);
}

// The viewer has no event for a change of view: the page checks the view on
// screen after any interaction with the viewer's chrome, and once a second.
for (const node of [tabs, tools, sheet]) node.addEventListener('click', () => setTimeout(followView, 0));
sheet.addEventListener('change', () => setTimeout(followView, 0));
sheet.addEventListener('keyup', () => setTimeout(followView, 0));
setInterval(followView, 1000);

// --- Status line --------------------------------------------------------------

let problemsOpen = false;

/** One line of status; the list of findings unfolds on demand (errors unfold by themselves). */
function showProblems(snapshot: Snapshot): void {
  clear(bottom);
  const errors = snapshot.errors.length;
  const findings = snapshot.findings;
  const gate = snapshot.gate;
  const bySeverity = new Map<string, number>();
  for (const finding of findings) bySeverity.set(finding.severity, (bySeverity.get(finding.severity) ?? 0) + 1);
  const counts = [...bySeverity.entries()].map(([severity, n]) => `${n} ${severity}${n > 1 ? 's' : ''}`).join(', ');
  const open = problemsOpen || errors > 0;
  const toggle = button(open ? '▾' : '▸', () => {
    problemsOpen = !problemsOpen;
    showProblems(snapshot);
  }, { class: 'wb-toggle', title: open ? 'Hide the findings' : 'Show the findings' });
  const summary = el('div', { class: 'wb-problems-head' }, [
    toggle,
    el('strong', { class: errors ? 'is-error' : '', text: errors ? `${errors} error${errors > 1 ? 's' : ''}: the model does not compile` : findings.length ? counts : 'no findings' }),
    gate ? el('span', { class: `wb-gate ${gate.passed ? 'is-pass' : 'is-fail'}`, text: gate.passed ? 'gate PASS' : `gate FAIL (${gate.requirements_verified}/${gate.requirements_total} requirements verified)` }) : null,
    el('span', { class: 'wb-spacer' }),
    button('Check', () => vscode.postMessage({ type: 'command', command: 'arclang.check' })),
    button('Gate', () => vscode.postMessage({ type: 'command', command: 'arclang.runGate' })),
    button('Transition plan', () => vscode.postMessage({ type: 'command', command: 'arclang.transitionPlan' })),
    button('Source', () => vscode.postMessage({ type: 'open-text' })),
  ]);
  bottom.appendChild(summary);
  bottom.classList.toggle('is-open', open);
  fitStage();
  if (!open) return;
  const list = el('ul', { class: 'wb-problems' });
  for (const error of snapshot.errors) list.appendChild(el('li', { class: 'is-error', text: error }));
  for (const finding of findings.slice(0, 200)) list.appendChild(problemItem(finding));
  if (findings.length > 200) list.appendChild(el('li', { class: 'wb-muted', text: `… ${findings.length - 200} more in the Problems view` }));
  bottom.appendChild(list);
}

function problemItem(finding: Finding): HTMLElement {
  const item = el('li', { class: `is-${finding.severity}` }, [el('code', { text: finding.rule }), ' ', finding.message]);
  if (finding.element) {
    const id = finding.element;
    item.appendChild(button(id, () => {
      const element = state.byId.get(id) ?? state.byName.get(id) ?? null;
      if (element) selectElement(element, true);
      vscode.postMessage({ type: 'reveal', id });
    }, { class: 'wb-link' }));
  }
  return item;
}

// --- Messages from the host -------------------------------------------------

window.addEventListener('message', (event: MessageEvent<ToWebview>) => {
  const message = event.data;
  switch (message.type) {
    case 'snapshot': {
      const before = state.snapshot;
      state.snapshot = message.snapshot;
      state.metamodel = message.metamodel;
      index(message.snapshot);
      if (message.snapshot.success) mount(message.snapshot);
      else if (!state.viewer) mount(message.snapshot);
      // A model that stopped compiling keeps its last drawing, marked as such.
      sheet.classList.toggle('is-stale', !message.snapshot.success);
      showProblems(message.snapshot);
      const relationship = (message.snapshot.elements?.relationships ?? []).find((r) => r.uuid === state.selected?.uuid);
      const stillThere = state.selected ? (state.byUuid.get(state.selected.uuid) ?? (relationship ? relationshipElement(relationship) : null)) : null;
      selectElement(stillThere, false);
      if (message.selected && !stillThere) {
        const wanted = state.byId.get(message.selected) ?? null;
        if (wanted) selectElement(wanted, true);
      }
      if (before && message.snapshot.success) flashChanges(before, message.snapshot);
      if (pendingView && message.snapshot.success) {
        const wanted = pendingView;
        pendingView = null;
        void state.viewer?.ready.then(() => {
          if (state.viewer?.views?.().some((v) => v.id === wanted)) showGraph(wanted);
        });
      }
      return;
    }
    case 'select': {
      const element = message.id ? (state.byId.get(message.id) ?? null) : null;
      selectElement(element, true);
      return;
    }
    case 'layer':
      stepLayer(message.step);
      return;
    case 'diagram-op': {
      const element = state.byId.get(message.id) ?? null;
      if (!element) return;
      if (message.op === 'hide') hideFromDiagram(element);
      if (message.op === 'related') showRelated(element);
      if (message.op === 'new') newDiagramAround(element);
      if (message.op === 'rename') editing.rename(element, drawingId(element.id));
      return;
    }
    case 'show': {
      const viewer = state.viewer;
      if (!viewer) return;
      void viewer.ready.then(() => {
        if (viewer.views?.().some((v) => v.id === message.view)) viewer.show?.(message.view);
        followView();
        const element = message.select ? (state.byId.get(message.select) ?? null) : null;
        if (element) selectElement(element, true);
        viewer.fit?.();
      });
      return;
    }
    case 'edit-result':
      if (message.ok) showToast(`Applied: ${message.label}. Undo with ⌘Z in the editor.`, 'info');
      else showToast(`Refused: ${message.error ?? ''}`, 'error', [['Show source', () => vscode.postMessage({ type: 'open-text' })]]);
      return;
    case 'busy':
      progress.hidden = !message.busy;
      return;
    default:
      return;
  }
});

/** Elements added or changed by the last edit pulse once, so the eye finds them. */
function flashChanges(before: Snapshot, after: Snapshot): void {
  // One element created: selected and brought to the middle, as Capella does.
  const known = new Set((before.elements?.elements ?? []).map((e) => e.uuid));
  const created = (after.elements?.elements ?? []).filter((e) => !known.has(e.uuid) && !/Port$/.test(e.kind));
  if (created.length === 1) {
    const element = created[0];
    void state.viewer?.ready.then(() =>
      setTimeout(() => {
        const view = state.viewer?.current?.()?.view;
        if (!view) return;
        programmaticUntil = Date.now() + 2000;
        void state.viewer?.goTo?.(view, drawingId(element.id));
        selectElement(element, false);
      }, 200),
    );
  }
  const old = new Map((before.elements?.elements ?? []).map((e) => [e.uuid, JSON.stringify(e.attributes) + e.name + e.owner]));
  const changed = (after.elements?.elements ?? []).filter((e) => old.get(e.uuid) !== JSON.stringify(e.attributes) + e.name + e.owner);
  if (changed.length === 0 || changed.length > 50) return;
  void state.viewer?.ready.then(() => {
    setTimeout(() => {
      for (const element of changed) {
        const node = host.querySelector<HTMLElement>(`[data-id="${CSS.escape(drawingId(element.id))}"]`);
        if (!node) continue;
        node.classList.add('wb-flash');
        setTimeout(() => node.classList.remove('wb-flash'), 900);
      }
    }, 300);
  });
}

document.addEventListener('keydown', (event) => {
  const typing = event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement || event.target instanceof HTMLTextAreaElement;
  if (event.key === 'Escape') {
    // Esc first cancels what is under way (a tool, a selection); then it leaves full screen.
    const busy = palette.isArmed() || !!state.selected || multi.length > 0;
    if (!busy && document.body.classList.contains('wb-focus')) setFocusMode(false);
    palette.disarm();
    toast.hidden = true;
    return;
  }
  if (typing) return;
  if (editing.key(event)) return;
  if (event.shiftKey && !event.metaKey && !event.ctrlKey && event.key === 'M') {
    event.preventDefault();
    toggleMinimap();
    return;
  }
  if (!event.metaKey && !event.ctrlKey && !event.altKey && (event.key === '[' || event.key === ']')) {
    event.preventDefault();
    stepLayer(event.key === ']' ? 1 : -1);
    return;
  }
  if (!event.metaKey && !event.ctrlKey && !event.altKey && event.key.length === 1 && palette.shortcut(event.key)) {
    event.preventDefault();
    return;
  }
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'f') {
    event.preventDefault();
    openFind();
    return;
  }
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'h' && multi.length > 1) {
    event.preventDefault();
    hideMany(multi);
    return;
  }
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'h' && state.selected) {
    event.preventDefault();
    hideFromDiagram(state.selected);
    return;
  }
  if ((event.key === 'Delete' || event.key === 'Backspace') && multi.length > 1) {
    event.preventDefault();
    deleteMany(multi);
    return;
  }

  if (event.key === 'F2' && state.selected) {
    event.preventDefault();
    editing.rename(state.selected);
    return;
  }
  if ((event.key === 'Delete' || event.key === 'Backspace') && state.selected) {
    event.preventDefault();
    deleteOne(state.selected);
  }
});

/** Delete one element; one that owns others says so first. */
function deleteOne(element: ElementRecord): void {
  const owned = (state.snapshot?.elements?.elements ?? []).filter((e) => e.owner === element.uuid).length;
  if (owned > 0) {
    showToast(`${element.name || element.id} owns ${owned} element${owned > 1 ? 's' : ''}.`, 'info', [
      [`Delete with ${owned === 1 ? 'it' : `all ${owned}`}`, () => {
        toast.hidden = true;
        submit([{ op: 'remove', element: element.id }], `delete ${element.id} and what it owns`);
      }],
    ], 10000);
    return;
  }
  submit([{ op: 'remove', element: element.id }], `delete ${element.id}`);
}

vscode.postMessage({ type: 'ready' });
