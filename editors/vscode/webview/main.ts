// The workbench page: the viewer in the middle, the palette on the left,
// properties and the semantic browser on the right, problems at the bottom.
// Everything shown comes from the last snapshot the host sent; every change
// goes back to the host as `arclang edit` requests.
import { EditRequest, ElementRecord, Finding, graphContainerOf, lifelineElement, Metamodel, Snapshot, ToWebview } from '../src/pure/workbench';
import { SemanticBrowser } from './browser';
import { button, clear, el } from './dom';
import type { ViewerApi, ViewerPosition, ViewerSelection } from './globals';
import { Palette } from './palette';
import { Properties } from './properties';

const vscode = acquireVsCodeApi();

interface State {
  snapshot: Snapshot | null;
  metamodel: Metamodel | null;
  viewer: ViewerApi | null;
  selected: ElementRecord | null;
  lastPosition: ViewerPosition | null;
}

const state: State = { snapshot: null, metamodel: null, viewer: null, selected: null, lastPosition: null };

// --- Layout -----------------------------------------------------------------

const root = document.getElementById('workbench')!;
const bar = el('header', { class: 'wb-bar' });
const title = el('span', { class: 'wb-title', text: root.dataset.file ?? '' });
const tabs = el('div', { class: 'wb-tabs' });
const tools = el('div', { class: 'wb-viewer-tools' });
const hint = el('span', { class: 'wb-hint' });
const busy = el('span', { class: 'wb-busy', text: '' });
bar.append(title, tabs, tools, hint, busy);

const left = el('aside', { class: 'wb-left' });
const sheet = el('main', { class: 'wb-sheet' });
const host = el('div', { class: 'wb-host' });
sheet.appendChild(host);
const right = el('aside', { class: 'wb-right' });
const bottom = el('footer', { class: 'wb-bottom' });
const hiddenTree = el('div', { class: 'wb-hidden' });
const detailsSlot = el('div', { class: 'wb-details' });
const relatedSlot = el('div', { class: 'wb-related' });

const rightTabs = el('div', { class: 'wb-right-tabs' });
const propertiesPane = el('div', { class: 'wb-pane is-active' });
const browserPane = el('div', { class: 'wb-pane' });
const detailsPane = el('div', { class: 'wb-pane' }, [detailsSlot, relatedSlot]);
const panes: [string, HTMLElement][] = [
  ['Properties', propertiesPane],
  ['Semantic browser', browserPane],
  ['Details', detailsPane],
];
for (const [label, pane] of panes) {
  const tab = button(label, () => {
    for (const [, other] of panes) other.classList.remove('is-active');
    for (const node of rightTabs.children) node.classList.remove('is-active');
    pane.classList.add('is-active');
    tab.classList.add('is-active');
  }, { class: `wb-right-tab${pane === propertiesPane ? ' is-active' : ''}` });
  rightTabs.appendChild(tab);
}
right.append(rightTabs, propertiesPane, browserPane, detailsPane);
root.append(bar, left, sheet, right, bottom, hiddenTree);

const palette = new Palette(left, {
  selected: () => (state.selected ? { id: state.selected.id, kind: state.selected.kind } : null),
  submit,
  hint: (text) => (hint.textContent = text),
});
const properties = new Properties(propertiesPane, {
  submit,
  reveal: (id) => vscode.postMessage({ type: 'reveal', id }),
  ask: (element) => vscode.postMessage({ type: 'ask', id: element.id, name: element.name, kind: element.kind }),
});
const browser = new SemanticBrowser(browserPane, (element) => selectElement(element, true));

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
  const records = state.snapshot?.elements?.elements ?? [];
  const element = records.find((e) => e.id === id);
  if (element && /Port$/.test(element.kind)) {
    const at = id.lastIndexOf('.');
    if (at > 0) return `${id.slice(0, at)}::${id.slice(at + 1)}`;
  }
  return id;
}

function elementFor(selection: ViewerSelection | null): ElementRecord | null {
  if (!selection || !state.snapshot?.elements) return null;
  const records = state.snapshot.elements.elements;
  const id = modelId(selection.id);
  const found = records.find((e) => e.id === id) ?? records.find((e) => e.name === selection.name);
  if (found) return found;
  // A lifeline of a scenario stands for its participant.
  const current = state.viewer?.current?.();
  const graph = state.snapshot.diagrams?.graphs.find((g) => g.id === current?.view);
  if (graph) {
    const participant = lifelineElement(id, graphContainerOf(graph.id, graph.arc.view));
    const byParticipant = records.find((e) => e.id === participant);
    if (byParticipant) return byParticipant;
  }
  // An edge is drawn for a relationship (`ce:LaneFlow`): the model lists it by name.
  const bare = selection.id.replace(/^[a-z]+:/, '');
  return records.find((e) => e.name === bare || e.id === bare) ?? null;
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

function mount(snapshot: Snapshot): void {
  const previous = state.viewer?.current?.() ?? state.lastPosition;
  clear(tabs);
  clear(tools);
  clear(hiddenTree);
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
  const viewer = window.ArcViz.mount(host, snapshot.diagrams, deps, {
    slots: { tabs, tools, tree: hiddenTree, details: detailsSlot, related: relatedSlot },
    onSelect,
    onDrop,
  });
  state.viewer = viewer;
  void viewer.ready.then(() => {
    if (previous?.view && viewer.show) {
      const known = viewer.views?.().some((v) => v.id === previous.view);
      if (known) viewer.show(previous.view);
    }
    if (previous?.position && viewer.restore) viewer.restore(previous.position);
    if (state.selected && viewer.select) viewer.select(drawingId(state.selected.id));
    refreshPalette();
    void viewer.settled?.()?.then(() => refreshPalette());
  });
}

let shownGraph: string | null = null;

/** The palette follows the view on screen; the viewer changes it on tabs, the diagram list and navigation. */
function refreshPalette(): void {
  const current = state.viewer?.current?.();
  const graph = state.snapshot?.diagrams?.graphs.find((g) => g.id === current?.view);
  const key = graph ? `${graph.id}\u0000${state.snapshot?.diagrams?.title ?? ''}` : null;
  if (key === shownGraph && graph) return;
  shownGraph = key;
  palette.show(graph?.arc.view ?? '', graph ? graphContainerOf(graph.id, graph.arc.view) : null);
}

// The viewer has no event for a change of view: the palette checks the view on
// screen after any interaction with the viewer's chrome, and once a second.
for (const node of [tabs, tools, sheet]) node.addEventListener('click', () => setTimeout(refreshPalette, 0));
sheet.addEventListener('change', () => setTimeout(refreshPalette, 0));
setInterval(refreshPalette, 1000);

// --- Problems ---------------------------------------------------------------

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
      const element = state.snapshot?.elements?.elements.find((e) => e.id === id || e.name === id) ?? null;
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
      state.snapshot = message.snapshot;
      state.metamodel = message.metamodel;
      title.textContent = message.file;
      if (message.snapshot.success) mount(message.snapshot);
      else if (!state.viewer) mount(message.snapshot);
      // A model that stopped compiling keeps its last drawing, marked as such.
      sheet.classList.toggle('is-stale', !message.snapshot.success);
      showProblems(message.snapshot);
      const stillThere = state.selected ? (message.snapshot.elements?.elements.find((e) => e.uuid === state.selected!.uuid) ?? null) : null;
      selectElement(stillThere, false);
      if (message.selected && !stillThere) {
        const wanted = message.snapshot.elements?.elements.find((e) => e.id === message.selected) ?? null;
        if (wanted) selectElement(wanted, true);
      }
      return;
    }
    case 'select': {
      const element = message.id ? (state.snapshot?.elements?.elements.find((e) => e.id === message.id) ?? null) : null;
      selectElement(element, true);
      return;
    }
    case 'edit-result':
      hint.textContent = message.ok ? '' : `Refused: ${message.error ?? ''}`;
      hint.classList.toggle('is-error', !message.ok);
      return;
    case 'busy':
      busy.textContent = message.busy ? `${message.label ?? 'working'}…` : '';
      return;
    default:
      return;
  }
});

document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') palette.disarm();
  if ((event.key === 'Delete' || event.key === 'Backspace') && state.selected && !(event.target instanceof HTMLInputElement) && !(event.target instanceof HTMLSelectElement)) {
    // Deletion asks for the id in the properties view, never on a bare key press.
    for (const node of rightTabs.children) node.classList.remove('is-active');
    (rightTabs.children[0] as HTMLElement | undefined)?.classList.add('is-active');
    for (const [, pane] of panes) pane.classList.remove('is-active');
    propertiesPane.classList.add('is-active');
  }
});

vscode.postMessage({ type: 'ready' });
