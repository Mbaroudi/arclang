// The workbench page: a 35 px bar (views and the viewer's tools), then a
// row with the palette rail, the canvas and the properties drawer, then one
// status line. Everything shown comes from the last snapshot the host sent;
// every change goes back to the host as `arclang edit` requests.
import { connectTargetKinds, connectToolFor, EditRequest, RelationshipRecord, fragmentRequest, nextRequirementId, requirementRequests, Tool, VIEWPOINTS, ElementRecord, graphContainerOf, lifelineElement, Metamodel, Snapshot, ToWebview } from '../src/pure/workbench';
import { SemanticBrowser } from './browser';
import { button, clear, el, inlinePrompt } from './dom';
import type { ViewerApi, ViewerPosition, ViewerSelection } from './globals';
import { Palette } from './palette';
import { Properties } from './properties';
import { createFind } from './find';
import { createMinimap } from './minimap';
import { Editing } from './editing';
import { createHelp } from './help';
import { createShell } from './shell';
import { DiagramContents } from './diagram-contents';
import { createStatusLine } from './status-line';
import { changesOf, markFindings as markFindings_, pulse, renderLayers as renderLayers_ } from './marks';
import { Alignment, modelIdOf, relationshipOfEdge } from '../src/pure/editing';
import { installAi } from './ask-ai'; // ai-design hook
import { installKeys } from './keys';
import { MultiSelection } from './multi-select';
import { installPanes } from './panes'; // panes hook
import { installImpact } from './impact-marks'; // impact hook
import { DiagramExports } from './diagram-export';
import { ThemeEditor } from './theme-editor';

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
    // Under 900 px the drawer floats over the canvas: closed until asked for.
    drawerOpen: savedLayout.drawerOpen ?? window.innerWidth >= 900,
    drawerWidth: Math.min(480, Math.max(240, savedLayout.drawerWidth ?? 280)),
    rightTab: savedLayout.rightTab ?? 0,
  },
};

function saveLayout(): void {
  vscode.setState({ layout: state.layout });
}

// --- Layout -----------------------------------------------------------------

const root = document.getElementById('workbench')!;
const shell = createShell(root, state.layout, { save: saveLayout, palette: (open) => palette.setCompact(!open), minimap: () => toggleMinimap() });
const { layerBar, tabs, tools, left, sheet, host, modeChip, toast, progress, layoutNotice, bottom, hiddenTree, hiddenFoot, detailsSlot, relatedSlot, propertiesPane, browserPane, showRightTab, setPalette, setDrawer, fitStage } = shell;

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
  // panes hook: the model, the links and the host's answers of the properties pane.
  snapshot: () => state.snapshot,
  select: (element) => selectElement(element, true),
  post: (message) => vscode.postMessage(message as never),
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

// --- Multi-selection (Shift+click, multi-select.ts) ---------------------------

const multiSelection = new MultiSelection({
  host,
  sheet,
  drawingId,
  byUuid: () => state.byUuid,
  refreshTools: () => state.viewer?.refreshTools?.(),
  showMany: (elements) => properties.showMany(elements, state.metamodel),
  submit,
  hide: (elements) => contents.hideMany(elements),
});
const markMulti = () => multiSelection.mark();
const outermost = (elements: ElementRecord[]) => multiSelection.outermost(elements);
const deleteMany = (elements: ElementRecord[]) => multiSelection.deleteMany(elements);
const hideMany = (elements: ElementRecord[]) => multiSelection.hideMany(elements);

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

function onSelect(picked: ViewerSelection | null): void {
  if (picked === null && Date.now() < programmaticUntil && state.selected) return;
  programmaticUntil = 0;
  state.selectedDrawing = picked?.id ?? null;
  const element = elementFor(picked);
  if (multiSelection.shiftClick(element, state.selected, !!picked?.isEdge)) {
    selectElement(element, false);
    return;
  }
  // A box of the selection dragged with the others: the selection stays.
  if (editing.placement.holds(element)) {
    selectElement(element, false);
    setTimeout(markMulti, 0);
    return;
  }
  if (multiSelection.elements.length) {
    multiSelection.elements = [];
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

// --- Diagram contents ---------------------------------------------------------

/** The diagram to show once the snapshot that creates it arrives. */
let pendingView: string | null = null;
const contents = new DiagramContents({
  graph: () => currentGraph(),
  snapshot: () => state.snapshot,
  idOf: (id) => elementFor({ view: '', id, name: '', kind: '', isEdge: false })?.id ?? null,
  submit,
  toast: (text, kind) => showToast(text, kind),
  showLater: (view) => (pendingView = view),
});
const hideFromDiagram = (element: ElementRecord) => contents.hide(element);
const showRelated = (element: ElementRecord) => contents.related(element);
const newDiagramAround = (element: ElementRecord) => contents.around(element);

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
const toggleMinimap = (force?: boolean) => shell.setMinimapShown(minimap.toggle(force));
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
  multi: () => multiSelection.elements,
  setMulti: (elements) => {
    multiSelection.elements = elements;
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
  copied: (elements, relationships) => vscode.postMessage({ type: 'copied', elements: elements.map((e) => ({ id: e.id, name: e.name })), relationships }),
  joinLayout,
  diagramOf: (elements) => contents.fromSelection(elements),
  history: {
    undo: () => vscode.postMessage({ type: 'history', op: 'undo' }),
    redo: () => vscode.postMessage({ type: 'history', op: 'redo' }),
    labels: () => historyLabels,
  },
  remark: () => markMulti(),
  extraMenu: () => exportsUi.menuItems(),
});

// --- Exports and colour themes (diagram-export.ts, theme-editor.ts) --------------

const exportsUi = new DiagramExports({
  post: (message) => vscode.postMessage(message as never),
  viewer: () => state.viewer,
  selection: () => (multiSelection.elements.length > 1 ? multiSelection.elements.map((e) => drawingId(e.id)) : state.selectedDrawing ? [state.selectedDrawing] : []),
  toast: (text, kind) => showToast(text, kind),
  file: () => root.dataset.file ?? 'diagrams',
  openEditor: () => themeEditor.open(),
});
const themeEditor = new ThemeEditor({
  host: sheet,
  post: (message) => vscode.postMessage(message as never),
  viewer: () => state.viewer,
  entries: () => exportsUi.entries,
  active: () => exportsUi.activeEntry(),
  dark: (theme) => document.body.classList.contains('vscode-dark') && typeof theme.dark === 'object' && theme.dark !== null,
  restore: () => exportsUi.reapply(),
});

/** What ⌘Z and ⌘⇧Z would undo and redo, as the host keeps the history. */
let historyLabels: { undo: string | null; redo: string | null } = { undo: null, redo: null };

/** Changes of the arrangement made while this is above zero belong to the step before them. */
let joining = 0;
async function joinLayout(run: () => Promise<void> | undefined): Promise<void> {
  joining += 1;
  try {
    await run();
  } finally {
    joining -= 1;
  }
}

// ai-design hook: Ask AI on the selection (⌘I) and the Suggestions pane (webview/ask-ai.ts).
installAi({ post: (m) => vscode.postMessage(m as never), sheet, host, selection: () => (multiSelection.elements.length > 1 ? multiSelection.elements : state.selected ? [state.selected] : []), drawingId, toast: (text, kind) => showToast(text, kind), openDrawer: () => setDrawer(true), snapshot: () => state.snapshot });

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
  // A remount while the last one is still restoring the view: the view the reader had is the one it kept.
  const previous = remounting && state.lastPosition ? state.lastPosition : (state.viewer?.current?.() ?? state.lastPosition);
  state.lastPosition = previous;
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
    theme: window.ArcVizTheme,
  };
  // A host that keeps the arrangement next to the model (VS Code) is its memory.
  if (snapshot.diagrams.layout_name) forgetStoredArrangement();
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
    onLayout: (file: unknown) => vscode.postMessage({ type: 'layout', file, join: joining > 0 }),
    // Editing from the drawing: double-click renames, ends reconnect, bands select, arrows connect.
    edit: editing.viewerOptions(),
    // Undo and redo from the host's history, align on the multi-selection; boxes are always placed by dragging.
    toolbar: {
      history: { undo: () => vscode.postMessage({ type: 'history', op: 'undo' }), redo: () => vscode.postMessage({ type: 'history', op: 'redo' }), labels: () => historyLabels },
      align: { count: () => multiSelection.elements.length, align: (how: string) => editing.align(how as Alignment), distribute: (axis: 'horizontal' | 'vertical') => editing.distribute(axis) },
      arrangeToggle: false,
    },
    // Copy as image, as SVG, PowerPoint in Export; the theme chooser and editor in Theme.
    exportItems: () => exportsUi.exportItems(),
    themeItems: () => exportsUi.themeItems(),
    theme: exportsUi.mountTheme(),
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
    editing.mounted();
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

/**
 * The arrangement is the host's (the layout file, and the history that
 * restores it): the viewer's own memory of it in the page's storage would
 * put back a state the history undid.
 */
function forgetStoredArrangement(): void {
  try {
    const keys = Array.from({ length: localStorage.length }, (_, i) => localStorage.key(i)).filter((key): key is string => !!key && key.startsWith('arcviz:'));
    for (const key of keys) localStorage.removeItem(key);
  } catch {
    // No storage in this page: nothing remembered.
  }
}

/** Nodes with findings are marked on the drawing, with the findings as their tooltip. */
function markFindings(): void {
  markFindings_(host, state.snapshot?.findings ?? [], (id) => {
    const element = state.byId.get(id) ?? state.byName.get(id);
    return element ? drawingId(element.id) : id;
  });
}


function renderLayers(): void {
  renderLayers_(layerBar, state.snapshot?.diagrams?.graphs ?? [], state.viewer?.current?.()?.view ?? null, showGraph);
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

const showProblems = createStatusLine(bottom, {
  post: (message) => vscode.postMessage(message),
  select: (id) => {
    const element = state.byId.get(id) ?? state.byName.get(id) ?? null;
    if (element) selectElement(element, true);
  },
  resized: fitStage,
});

// --- Messages from the host -------------------------------------------------

window.addEventListener('message', (event: MessageEvent<ToWebview>) => {
  const message = event.data;
  if (exportsUi.handle(message) || themeEditor.handle(message)) return;
  switch (message.type) {
    case 'snapshot': {
      const before = state.snapshot;
      state.snapshot = message.snapshot;
      state.metamodel = message.metamodel;
      index(message.snapshot);
      editing.snapshotArrived();
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
      editing.editResult(message.label, message.ok);
      if (message.ok) showToast(`Applied: ${message.label}. ⌘Z undoes it.`, 'info');
      else showToast(`Refused: ${message.error ?? ''}`, 'error', [['Show source', () => vscode.postMessage({ type: 'open-text' })]]);
      return;
    case 'busy':
      progress.hidden = !message.busy;
      return;
    case 'key':
      canvasCommand(message.command, message.text);
      return;
    case 'history':
      historyLabels = { undo: message.undo, redo: message.redo };
      state.viewer?.refreshTools?.();
      if (message.done) showToast(`${message.done[0].toUpperCase()}${message.done.slice(1)}.`, 'info');
      return;
    default:
      return;
  }
});

/** Elements added or changed by the last edit pulse once; one created is selected and centred, as Capella does. */
function flashChanges(before: Snapshot, after: Snapshot): void {
  const { created, changed } = changesOf(before, after);
  void state.viewer?.ready.then(() => {
    if (created) {
      setTimeout(() => {
        const view = state.viewer?.current?.()?.view;
        if (!view) return;
        programmaticUntil = Date.now() + 2000;
        void state.viewer?.goTo?.(view, drawingId(created.id));
        selectElement(created, false);
      }, 200);
    }
    if (changed.length) setTimeout(() => pulse(host, changed.map((e) => drawingId(e.id))), 300);
  });
}

// --- Keyboard (keys.ts) ---------------------------------------------------------

const keys = installKeys({
  post: (message) => vscode.postMessage(message as never), editing, palette, tools, toast,
  multi: () => multiSelection.elements, selected: () => state.selected, viewer: () => state.viewer, snapshot: () => state.snapshot,
  deleteMany, hideMany, hideFromDiagram, setFocusMode, toggleMinimap: () => toggleMinimap(), stepLayer, openFind, showToast, submit, copyImage: () => void exportsUi.copyImage(),
});
const canvasCommand = keys.canvasCommand;
const deleteOne = keys.deleteOne;

// panes hook: tree drops, a diagram's own tab, "Open in a new tab" on the layers (webview/panes.ts).
installPanes({ sheet, layerBar, viewer: () => state.viewer, graph: () => currentGraph(), snapshot: () => state.snapshot, idOf: (id) => elementFor({ view: '', id, name: '', kind: '', isEdge: false })?.id ?? null, select: (element) => selectElement(element, false), submit, toast: (text, kind) => showToast(text, kind), post: (message) => vscode.postMessage(message as never), showGraph });

installImpact({ post: (m) => vscode.postMessage(m as never), host, layerBar, drawingId, selection: () => (multiSelection.elements.length > 1 ? multiSelection.elements : state.selected ? [state.selected] : []) }); // impact hook

vscode.postMessage({ type: 'ready' });
