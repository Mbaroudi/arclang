// The workbench page: a 35 px bar (views and the viewer's tools), then a
// row with the palette rail, the canvas and the properties drawer, then one
// status line. Everything shown comes from the last snapshot the host sent;
// every change goes back to the host as `arclang edit` requests.
import { connectTargetKinds, connectToolFor, EditRequest, ElementRecord, Finding, graphContainerOf, lifelineElement, Metamodel, Snapshot, ToWebview } from '../src/pure/workbench';
import { SemanticBrowser } from './browser';
import { button, clear, el } from './dom';
import type { ViewerApi, ViewerPosition, ViewerSelection } from './globals';
import { Palette } from './palette';
import { Properties } from './properties';

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
const tabs = el('div', { class: 'wb-tabs' });
const tools = el('div', { class: 'wb-viewer-tools' });
const paletteToggle = button('', () => setPalette(!state.layout.paletteOpen), { class: 'wb-icon-button', title: 'Palette' });
const drawerToggle = button('', () => setDrawer(!state.layout.drawerOpen), { class: 'wb-icon-button', title: 'Properties' });
bar.append(paletteToggle, tabs, tools, drawerToggle);

const body = el('div', { class: 'wb-body' });
const left = el('aside', { class: 'wb-left' });
const sheet = el('main', { class: 'wb-sheet' });
const host = el('div', { class: 'wb-host' });
const modeChip = el('div', { class: 'wb-mode', hidden: true });
const toast = el('div', { class: 'wb-toast', hidden: true, role: 'status', 'aria-live': 'polite' });
const progress = el('div', { class: 'wb-progress', hidden: true });
sheet.append(progress, host, modeChip, toast);
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
});
const properties = new Properties(propertiesPane, {
  submit,
  reveal: (id) => vscode.postMessage({ type: 'reveal', id }),
  ask: (element) => vscode.postMessage({ type: 'ask', id: element.id, name: element.name, kind: element.kind }),
});
const browser = new SemanticBrowser(browserPane, (element) => selectElement(element, true));
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

/** The model id of a node or port of the drawing (`LC-A::p` is the port `LC-A.p`). */
function modelId(drawingId: string): string {
  return drawingId.replace('::', '.');
}

/** The drawing id of a model element, when it has a node. */
function drawingId(id: string): string {
  const element = state.byId.get(id);
  if (element && /Port$/.test(element.kind)) {
    const at = id.lastIndexOf('.');
    if (at > 0) return `${id.slice(0, at)}::${id.slice(at + 1)}`;
  }
  return id;
}

function currentGraph() {
  const current = state.viewer?.current?.();
  return state.snapshot?.diagrams?.graphs.find((g) => g.id === current?.view) ?? null;
}

function elementFor(selection: ViewerSelection | null): ElementRecord | null {
  if (!selection || !state.snapshot?.elements) return null;
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
  return state.byName.get(bare) ?? state.byId.get(bare) ?? null;
}

function selectElement(element: ElementRecord | null, inViewer: boolean): void {
  state.selected = element;
  properties.show(element, state.metamodel);
  browser.show(element, state.snapshot);
  vscode.postMessage({ type: 'selected', id: element?.id ?? null, kind: element?.kind, name: element?.name });
  if (inViewer && element && state.viewer?.select) state.viewer.select(drawingId(element.id));
}

function onSelect(selection: ViewerSelection | null): void {
  const element = elementFor(selection);
  // A connect tool takes the click as one of its ends.
  if (palette.pick(element ? { id: element.id, kind: element.kind } : null)) return;
  selectElement(element, false);
}

function onDrop(dragged: string, target: string | null): void {
  const element = elementFor({ view: '', id: dragged, name: '', kind: '', isEdge: false });
  if (!element) return;
  const container = target ? elementFor({ view: '', id: target, name: '', kind: '', isEdge: false }) : null;
  if (!container || container.uuid === element.owner) return;
  submit([{ op: 'move', element: element.id, container: container.id }], `move ${element.id} into ${container.id}`);
}

/** The drawn element ids a connection from `fromId` may land on (null: any). */
function connectTargets(fromId: string): string[] | null {
  const from = elementFor({ view: '', id: fromId, name: '', kind: '', isEdge: false });
  const graph = currentGraph();
  if (!from || !graph) return [];
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
  };
  progress.hidden = false;
  const viewer = window.ArcViz.mount(host, snapshot.diagrams, deps, {
    slots: { tabs, tools, tree: hiddenTree, details: detailsSlot, related: relatedSlot, foot: hiddenFoot },
    onSelect,
    onDrop,
    onOpen,
    onConnect,
    connectTargets,
  });
  state.viewer = viewer;
  fitStage();
  void viewer.ready.then(() => {
    tagContexts();
    markFindings();
    if (previous?.view && viewer.show) {
      const known = viewer.views?.().some((v) => v.id === previous.view);
      if (known) viewer.show(previous.view);
    }
    if (previous?.position && viewer.restore) viewer.restore(previous.position);
    if (state.selected && viewer.select) viewer.select(drawingId(state.selected.id));
    shownGraph = null;
    followView();
    fitStage();
    progress.hidden = true;
    void viewer.settled?.()?.then(() => {
      followView();
      fitStage();
    });
  });
}

/**
 * Every drawn element carries its context for VS Code's native context menu
 * (`webview/context` in the manifest): right-click gives Rename, Delete,
 * Show in source... with the element's id as the command argument.
 */
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

function tagContexts(): void {
  for (const node of host.querySelectorAll<HTMLElement>('[data-id]')) {
    if (node.dataset.vscodeContext) continue;
    const element = elementFor({ view: '', id: node.dataset.id ?? '', name: '', kind: '', isEdge: false });
    if (!element) continue;
    node.dataset.vscodeContext = JSON.stringify({ webviewSection: 'element', elementId: element.id, elementName: element.name, elementKind: element.kind, preventDefaultContextMenuItems: true });
  }
}
host.addEventListener('contextmenu', (event) => {
  // The right-clicked element becomes the selection, so menu commands and the views agree.
  const target = (event.target as HTMLElement).closest<HTMLElement>('[data-id]');
  const element = target ? elementFor({ view: '', id: target.dataset.id ?? '', name: '', kind: '', isEdge: false }) : null;
  if (element) selectElement(element, false);
  tagContexts();
});

let shownGraph: string | null = null;

/** What follows the view on screen: the palette, and a selection that is not drawn there. */
function followView(): void {
  const graph = currentGraph();
  const key = graph ? graph.id : null;
  if (key === shownGraph) return;
  shownGraph = key;
  palette.show(graph?.arc.view ?? '', graph ? graphContainerOf(graph.id, graph.arc.view) : null);
  if (state.selected && graph && !drawn(graph, state.selected)) selectElement(null, false);
  fitStage();
  setTimeout(() => {
    tagContexts();
    markFindings();
  }, 200);
}

/** Whether an element has a node, a port or a lifeline on the graph. */
function drawn(graph: NonNullable<ReturnType<typeof currentGraph>>, element: ElementRecord): boolean {
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
      const stillThere = state.selected ? (state.byUuid.get(state.selected.uuid) ?? null) : null;
      selectElement(stillThere, false);
      if (message.selected && !stillThere) {
        const wanted = state.byId.get(message.selected) ?? null;
        if (wanted) selectElement(wanted, true);
      }
      if (before && message.snapshot.success) flashChanges(before, message.snapshot);
      return;
    }
    case 'select': {
      const element = message.id ? (state.byId.get(message.id) ?? null) : null;
      selectElement(element, true);
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
  const editing = event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement || event.target instanceof HTMLTextAreaElement;
  if (event.key === 'Escape') {
    palette.disarm();
    toast.hidden = true;
    return;
  }
  if (editing) return;
  if (!event.metaKey && !event.ctrlKey && !event.altKey && event.key.length === 1 && palette.shortcut(event.key)) {
    event.preventDefault();
    return;
  }
  if (event.key === 'F2' && state.selected) {
    event.preventDefault();
    properties.rename();
    return;
  }
  if ((event.key === 'Delete' || event.key === 'Backspace') && state.selected) {
    event.preventDefault();
    const element = state.selected;
    const owned = (state.snapshot?.elements?.elements ?? []).filter((e) => e.owner === element.uuid).length;
    if (owned > 0) {
      showToast(`${element.name || element.id} owns ${owned} element${owned > 1 ? 's' : ''}: delete it from the Properties view.`, 'info');
      return;
    }
    submit([{ op: 'remove', element: element.id }], `delete ${element.id}`);
  }
});

vscode.postMessage({ type: 'ready' });
