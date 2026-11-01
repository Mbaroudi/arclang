// Semantic git on the workbench page: the "Compare ▾" menu, the compare mode
// (the diagrams drawn with the changes marked: added green, modified amber,
// removed ghosted dashed red at their old place, moved and renamed with a
// badge; counts on the layer tabs; a side panel of the changes), and the
// history of an element with the model at a past revision beside the current
// one. What is compared and what is read from git is the host's.
import type { CompareMark, CompareState, ElementHistory, HistoryEntry, MarkStatus, RelationChange } from '../src/pure/semgit';
import { entryDetails } from '../src/pure/semgit';
import { VIEWPOINTS } from '../src/pure/workbench';
import type { ElementRecord, ViewerPayload } from '../src/pure/workbench';
import { button, el } from './dom';
import type { MenuAction, MenuItem } from './menu';
import { ContextMenu } from './menu';

export interface CompareHooks {
  slot: HTMLElement;
  sheet: HTMLElement;
  host: HTMLElement;
  layerBar: HTMLElement;
  post: (message: unknown) => void;
  drawingId: (id: string) => string;
  /** Select an element of the model now (null when it is not in it: a ghost). */
  select: (id: string) => boolean;
  selection: () => ElementRecord | null;
  viewer: () => { fit?: () => void; select?: (id: string | null) => void; current?: () => { view?: string | null } | undefined } | null;
  /** Mount a second viewer, read-only, in an element. */
  mountRevision: (into: HTMLElement, diagrams: ViewerPayload, view: string | null) => void;
  toast: (text: string, kind: 'info' | 'error') => void;
}

const STATUS_CLASSES: MarkStatus[] = ['added', 'removed', 'modified', 'moved', 'renamed'];
const GLYPH: Record<MarkStatus, string> = { added: '+', removed: '×', modified: '~', moved: '⇄', renamed: 'Aa' };
const WORD: Record<MarkStatus, string> = { added: 'Added', removed: 'Removed', modified: 'Modified', moved: 'Moved', renamed: 'Renamed' };
const SVG = 'http://www.w3.org/2000/svg';
const LAYER_NAMES: Record<string, string> = { OA: 'Operational Analysis', SA: 'System Analysis', LA: 'Logical Architecture', PA: 'Physical Architecture', EPBS: 'EPBS', Data: 'Data model', Transverse: 'Requirements, safety, tests' };

let hooks: CompareHooks | null = null;
let state: CompareState | null = null;
let painting = false;
let panel: HTMLElement | null = null;
let selectedRow: string | null = null;

/** Whether the page shows a comparison: the editing gestures are refused. */
export function comparing(): boolean {
  return state !== null;
}

export function installCompare(h: CompareHooks): void {
  hooks = h;
  loadStyles();
  const menu = new ContextMenu(h.sheet);
  const open = el('button', { type: 'button', class: 'wb-icon-button wb-bar-toggle wb-compare-button', id: 'wb-compare-menu', 'aria-haspopup': 'menu', 'aria-label': 'Compare ▾: with a revision, history, baselines', title: 'Compare the model with main, a branch, a tag, a baseline or HEAD; history; baselines' }, [glyph()]);
  h.slot.appendChild(open);
  open.addEventListener('click', () => {
    if (menu.isOpen) return menu.close();
    const box = open.getBoundingClientRect();
    menu.open(box.left, box.bottom + 2, menuItems());
  });

  window.addEventListener('message', (event: MessageEvent) => {
    const message = event.data as { type?: string; state?: CompareState | null; history?: ElementHistory; error?: string; hash?: string; diagrams?: ViewerPayload; id?: string; name?: string } | undefined;
    if (!message) return;
    if (message.type === 'compare') {
      const was = state?.ref ?? null;
      state = message.state ?? null;
      if (was !== (state?.ref ?? null)) selectedRow = null;
      document.body.classList.toggle('wb-comparing', state !== null);
      renderPanel();
      paint();
      // The panel takes room: the drawing is fitted to what is left (only when the comparison starts or ends, never on each edit).
      if (was !== (state?.ref ?? null)) setTimeout(() => h.viewer()?.fit?.(), 450);
    } else if (message.type === 'history') {
      if (message.error) h.toast(`History: ${message.error}`, 'error');
      else if (message.history) showHistory(message.history);
    } else if (message.type === 'history-revision') {
      if (message.error) h.toast(`That revision could not be drawn: ${message.error}`, 'error');
      else if (message.diagrams) showRevision(message.hash ?? '', message.diagrams);
    } else if (message.type === 'baseline-created') {
      h.toast(`Baseline ${message.name} created`, 'info');
    }
  });
  // The drawing and the layer tabs are built again by the page: paint again on what was built.
  let timer: ReturnType<typeof setTimeout> | undefined;
  const again = () => {
    if (painting || !state) return;
    clearTimeout(timer);
    timer = setTimeout(paint, 120);
  };
  const observer = new MutationObserver(again);
  observer.observe(h.host, { childList: true, subtree: true });
  observer.observe(h.layerBar, { childList: true });
}

/** A 16 × 16 outline in the toolbar's style: two branches and an arrow between them (the label is the tooltip, the toolbar is tight). */
function glyph(): SVGSVGElement {
  const svg = document.createElementNS(SVG, 'svg');
  for (const [key, value] of Object.entries({ viewBox: '0 0 16 16', width: '16', height: '16', fill: 'none', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'aria-hidden': 'true', focusable: 'false', class: 'arcviz-icon wb-compare-icon' })) svg.setAttribute(key, value);
  for (const d of ['M4 5.5v5', 'M12 10.5v-5', 'M10 7.5l2-2 2 2', 'M2.5 4a1.5 1.5 0 103 0 1.5 1.5 0 10-3 0z', 'M10.5 12a1.5 1.5 0 103 0 1.5 1.5 0 10-3 0z']) {
    const path = document.createElementNS(SVG, 'path');
    path.setAttribute('d', d);
    svg.appendChild(path);
  }
  return svg;
}

function loadStyles(): void {
  const base = document.querySelector<HTMLLinkElement>('link[href*="workbench.css"]');
  if (!base || document.querySelector('link[data-compare-styles]')) return;
  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = base.href.replace(/workbench\.css(\?.*)?$/, 'compare.css');
  link.dataset.compareStyles = '';
  base.after(link);
}

function menuItems(): MenuItem[] {
  const post = hooks!.post;
  const selected = hooks!.selection();
  const items: MenuItem[] = [
    { label: 'Timeline: travel in the history…', run: () => post({ type: 'semgit-timeline', op: 'open' }) },
    'separator',
    { label: 'Compare with HEAD', run: () => post({ type: 'semgit-compare', op: 'with', ref: 'HEAD' }) },
    { label: 'Compare with a branch, a tag or a revision…', run: () => post({ type: 'semgit-compare', op: 'with' }) },
    { label: 'Compare with a baseline…', run: () => post({ type: 'semgit-compare', op: 'baseline' }) },
  ];
  if (state) items.push({ label: 'Exit the comparison', run: () => post({ type: 'semgit-compare', op: 'exit' }) });
  items.push('separator', { label: selected ? `History of ${selected.name || selected.id}…` : 'History of the selection…', disabled: !selected || selected.kind === 'Model', run: () => selected && post({ type: 'semgit-history', id: selected.id }) });
  items.push('separator', { label: 'Create baseline…', run: () => post({ type: 'semgit-baseline', op: 'create' }) }, { label: 'Install the semantic merge driver…', run: () => post({ type: 'semgit-merge-install' }) });
  return items;
}

/** "History…" in the context menu of one element (semgit hook in editing.ts). */
export function withHistory(items: MenuItem[]): MenuItem[] {
  const element = hooks?.selection() ?? null;
  if (!hooks || !element || element.kind === 'Model') return items;
  const item: MenuAction = { label: 'History…', run: () => hooks?.post({ type: 'semgit-history', id: element.id }) };
  return [item, ...items];
}

/** Ask for the history of an element (the Properties pane's button). */
export function openHistory(id: string): void {
  hooks?.post({ type: 'semgit-history', id });
}

// --- Marks -------------------------------------------------------------------

function nodeOf(mark: CompareMark): Element | null {
  const h = hooks!;
  for (const candidate of [h.drawingId(mark.id), mark.id, mark.name]) {
    if (!candidate) continue;
    const found = h.host.querySelector(`[data-id="${CSS.escape(candidate)}"]`);
    if (found) return found;
  }
  return null;
}

function addBadge(node: Element, status: MarkStatus): void {
  if (!(node instanceof SVGGraphicsElement)) return;
  let box: DOMRect;
  try {
    box = node.getBBox();
  } catch {
    return;
  }
  if (!box.width && !box.height) return;
  const group = document.createElementNS(SVG, 'g');
  group.setAttribute('class', `wb-cmp-badge is-${status}`);
  group.setAttribute('transform', `translate(${box.x + box.width},${box.y})`);
  const circle = document.createElementNS(SVG, 'circle');
  circle.setAttribute('r', status === 'renamed' ? '10' : '9');
  const text = document.createElementNS(SVG, 'text');
  text.setAttribute('text-anchor', 'middle');
  text.setAttribute('dominant-baseline', 'central');
  text.textContent = GLYPH[status];
  const title = document.createElementNS(SVG, 'title');
  title.textContent = WORD[status];
  group.append(title, circle, text);
  node.appendChild(group);
}

function paint(): void {
  const h = hooks;
  if (!h) return;
  painting = true;
  try {
    for (const node of h.host.querySelectorAll('.wb-cmp')) node.classList.remove('wb-cmp', ...STATUS_CLASSES.map((s) => `wb-cmp-${s}`));
    for (const old of h.host.querySelectorAll('.wb-cmp-badge, title[data-cmp]')) old.remove();
    if (state) {
      const drawnGhosts = new Set(state.ghostDrawn ?? []);
      for (const mark of state.marks) {
        const node = nodeOf(mark);
        if (!node) continue;
        node.classList.add('wb-cmp', `wb-cmp-${mark.status}`);
        const title = document.createElementNS(SVG, 'title');
        title.dataset.cmp = '';
        title.textContent = [`${WORD[mark.status]}: ${mark.name || mark.id}`, ...mark.details].join('\n');
        node.prepend(title);
        addBadge(node, mark.status);
      }
      // Exchanges the old drawing had, ghosted.
      for (const id of drawnGhosts) {
        const node = h.host.querySelector(`[data-id="${CSS.escape(id)}"]`);
        if (node && !node.classList.contains('wb-cmp-removed')) node.classList.add('wb-cmp', 'wb-cmp-removed');
      }
      markRelationships(h.host, state.relationships);
    }
    badges(h.layerBar);
    h.host.classList.toggle('has-compare', state !== null);
  } finally {
    painting = false;
  }
}

/** Added and changed exchanges and traces, by the name they are drawn under. */
function markRelationships(host: HTMLElement, relations: RelationChange[]): void {
  for (const r of relations) {
    if (!r.name || r.status === 'removed') continue;
    const candidates = host.querySelectorAll(`[data-id$=":${CSS.escape(r.name)}"], [data-id="${CSS.escape(r.name)}"]`);
    for (const node of candidates) {
      if (node.classList.contains('wb-cmp')) continue;
      const status: MarkStatus = r.status === 'added' ? 'added' : 'modified';
      node.classList.add('wb-cmp', `wb-cmp-${status}`);
    }
  }
}

/** "+1 −2 ~3" on each layer tab that holds changes. */
function badges(bar: HTMLElement): void {
  for (const old of bar.querySelectorAll('.wb-cmp-count')) old.remove();
  if (!state) return;
  for (const tab of bar.querySelectorAll<HTMLElement>('.wb-layer')) {
    const name = tab.querySelector('.wb-layer-name')?.textContent ?? '';
    const view = VIEWPOINTS.find((v) => v.short === name)?.view;
    const count = view ? state.counts[view] : undefined;
    if (!count) continue;
    const total = count.added + count.removed + count.modified;
    const badge = el('span', { class: 'wb-cmp-count', 'data-view': view ?? '' });
    if (count.added) badge.appendChild(el('span', { class: 'is-added', text: `+${count.added}` }));
    if (count.removed) badge.appendChild(el('span', { class: 'is-removed', text: `−${count.removed}` }));
    if (count.modified) badge.appendChild(el('span', { class: 'is-modified', text: `~${count.modified}` }));
    badge.title = `${total} change${total === 1 ? '' : 's'} in this layer: ${count.added} added, ${count.removed} removed, ${count.modified} modified`;
    tab.appendChild(badge);
  }
}

// --- The panel of changes ------------------------------------------------------

function gateText(s: CompareState): string {
  const c = s.consequences;
  const word = (g: { passed: boolean }) => (g.passed ? 'PASS' : 'FAIL');
  return `${word(c.gate_before)} → ${word(c.gate_after)}`;
}

function renderPanel(): void {
  const h = hooks!;
  panel?.remove();
  panel = null;
  if (!state) return;
  const s = state;
  const c = s.consequences;
  const root = el('aside', { class: 'wb-cmp-panel', 'aria-label': `Changes against ${s.label}` });
  const past = s.timeTravel === true;
  const head = el('div', { class: 'wb-cmp-head' }, [
    el('div', { class: 'wb-cmp-title' }, [el('strong', { text: past ? 'At ' : 'Comparing with ' }), el('code', { text: s.label })]),
    past
      ? button('Back to now', () => h.post({ type: 'semgit-timeline', op: 'now' }), { class: 'wb-cmp-exit', title: 'The model as it is in the editor' })
      : button('Exit', () => h.post({ type: 'semgit-compare', op: 'exit' }), { class: 'wb-cmp-exit', title: 'Leave the comparison and edit again' }),
  ]);
  root.appendChild(head);
  root.appendChild(el('p', { class: 'wb-cmp-note', text: past ? 'A past moment of the model, as git holds it: read-only. The changes marked are those since the moment compared with.' : 'Read-only: leave the comparison to edit the model.' }));

  const summary = el('div', { class: 'wb-cmp-summary' });
  const total = s.marks.length;
  summary.appendChild(el('span', { class: `wb-cmp-gate ${c.gate_after.passed ? 'is-pass' : 'is-fail'}`, text: `Gate ${gateText(s)}` }));
  summary.appendChild(el('span', { class: 'wb-cmp-chip', text: `${total} element${total === 1 ? '' : 's'}` }));
  if (c.new_blockers.length) summary.appendChild(el('span', { class: 'wb-cmp-chip is-fail', text: `+${c.new_blockers.length} blocker${c.new_blockers.length === 1 ? '' : 's'}` }));
  if (c.removed_blockers.length) summary.appendChild(el('span', { class: 'wb-cmp-chip is-pass', text: `−${c.removed_blockers.length} blocker${c.removed_blockers.length === 1 ? '' : 's'}` }));
  if (c.newly_unsatisfied.length) summary.appendChild(el('span', { class: 'wb-cmp-chip is-fail', text: `${c.newly_unsatisfied.length} unsatisfied` }));
  if (c.newly_unverified.length) summary.appendChild(el('span', { class: 'wb-cmp-chip is-warn', text: `${c.newly_unverified.length} unverified` }));
  root.appendChild(summary);
  if (s.layoutChanges) root.appendChild(el('p', { class: 'wb-cmp-note', text: `${s.layoutChanges} layout entr${s.layoutChanges === 1 ? 'y' : 'ies'} differ${s.layoutOnly ? ': layout only, the model is unchanged' : ' (not a model change)'}.` }));

  const list = el('div', { class: 'wb-cmp-list', role: 'list' });
  if (!s.changed) list.appendChild(el('p', { class: 'wb-cmp-empty', text: s.layoutOnly ? 'The model is unchanged.' : 'No semantic change.' }));
  const byLayer = new Map<string, CompareMark[]>();
  for (const mark of s.marks) byLayer.set(mark.layer, [...(byLayer.get(mark.layer) ?? []), mark]);
  for (const [layer, marks] of byLayer) {
    list.appendChild(el('h4', { class: 'wb-cmp-layer', text: `${LAYER_NAMES[layer] ?? layer} · ${marks.length}` }));
    for (const mark of marks) list.appendChild(row(mark));
  }
  if (s.relationships.length) {
    list.appendChild(el('h4', { class: 'wb-cmp-layer', text: `Relationships · ${s.relationships.length}` }));
    for (const r of s.relationships) list.appendChild(relationRow(r));
  }
  root.appendChild(list);
  h.sheet.appendChild(root);
  panel = root;
}

function row(mark: CompareMark): HTMLElement {
  const h = hooks!;
  const item = el('div', { class: `wb-cmp-row is-${mark.status}`, role: 'listitem', tabindex: '0', 'data-id': mark.id });
  item.appendChild(el('span', { class: `wb-cmp-glyph is-${mark.status}`, 'aria-label': WORD[mark.status], text: GLYPH[mark.status] }));
  const body = el('div', { class: 'wb-cmp-body' }, [el('div', { class: 'wb-cmp-name' }, [el('span', { text: mark.name || mark.id }), el('code', { text: mark.id })]), el('div', { class: 'wb-cmp-kind', text: mark.kind })]);
  for (const line of mark.details) body.appendChild(el('div', { class: 'wb-cmp-detail', text: line }));
  item.appendChild(body);
  const actions = el('div', { class: 'wb-cmp-actions' });
  if (mark.status !== 'removed') actions.appendChild(button('Source', () => h.post({ type: 'reveal', id: mark.id }), { class: 'wb-link', title: 'Show it in the source' }));
  actions.appendChild(button('Diff', () => h.post({ type: 'semgit-open-diff', id: mark.id }), { class: 'wb-link', title: "Open VS Code's diff of the file against the revision" }));
  if (mark.status !== 'added') actions.appendChild(button('History', () => h.post({ type: 'semgit-history', id: mark.id }), { class: 'wb-link', title: 'The commits that changed it' }));
  item.appendChild(actions);
  const choose = () => {
    selectedRow = mark.id;
    panel?.querySelectorAll('.wb-cmp-row.is-selected').forEach((n) => n.classList.remove('is-selected'));
    item.classList.add('is-selected');
    if (!h.select(mark.id)) h.viewer()?.select?.(h.drawingId(mark.id));
  };
  item.addEventListener('click', (event) => {
    if ((event.target as HTMLElement).closest('.wb-cmp-actions')) return;
    choose();
  });
  item.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      choose();
    }
  });
  if (selectedRow === mark.id) item.classList.add('is-selected');
  return item;
}

function relationRow(r: RelationChange): HTMLElement {
  const mark: MarkStatus = r.status === 'added' ? 'added' : r.status === 'removed' ? 'removed' : 'modified';
  const item = el('div', { class: `wb-cmp-row is-${mark}`, role: 'listitem' });
  item.appendChild(el('span', { class: `wb-cmp-glyph is-${mark}`, text: GLYPH[mark] }));
  const was = r.was ? ` (was ${r.was.from} → ${r.was.to})` : '';
  item.appendChild(el('div', { class: 'wb-cmp-body' }, [el('div', { class: 'wb-cmp-name' }, [el('span', { text: `${r.kind || r.category}${r.name ? ` ${r.name}` : ''}` }), el('code', { text: r.status })]), el('div', { class: 'wb-cmp-detail', text: `${r.source} → ${r.target}${was}` }), ...r.changes.map((c) => el('div', { class: 'wb-cmp-detail', text: `${c.field}: "${c.old}" → "${c.new}"` }))]));
  return item;
}

// --- History of an element ---------------------------------------------------------

let historyPanel: HTMLElement | null = null;
let revisionPane: HTMLElement | null = null;

function closeHistory(): void {
  historyPanel?.remove();
  revisionPane?.remove();
  historyPanel = null;
  revisionPane = null;
  document.body.classList.remove('wb-history');
}

function showHistory(history: ElementHistory): void {
  const h = hooks!;
  closeHistory();
  document.body.classList.add('wb-history');
  setTimeout(() => h.viewer()?.fit?.(), 450);
  const root = el('aside', { class: 'wb-hist-panel', 'aria-label': `History of ${history.name || history.id}` });
  root.appendChild(el('div', { class: 'wb-cmp-head' }, [el('div', { class: 'wb-cmp-title' }, [el('strong', { text: 'History of ' }), el('code', { text: history.name && history.name !== history.id ? `${history.name} (${history.id})` : history.id })]), button('Close', closeHistory, { class: 'wb-cmp-exit' })]));
  const list = el('div', { class: 'wb-cmp-list', role: 'list' });
  if (history.entries.length === 0) list.appendChild(el('p', { class: 'wb-cmp-empty', text: 'No commit changed this element.' }));
  for (const entry of history.entries) list.appendChild(historyRow(entry, history.id));
  root.appendChild(list);
  if (history.skipped.length) root.appendChild(el('p', { class: 'wb-cmp-note', text: `${history.skipped.length} revision(s) skipped: the model did not compile there.` }));
  h.sheet.appendChild(root);
  historyPanel = root;
}

function historyRow(entry: HistoryEntry, id: string): HTMLElement {
  const h = hooks!;
  const status: MarkStatus = entry.status === 'modified' ? 'modified' : entry.status;
  const item = el('div', { class: `wb-cmp-row is-${status}`, role: 'listitem', tabindex: '0', 'data-hash': entry.hash });
  item.appendChild(el('span', { class: `wb-cmp-glyph is-${status}`, text: GLYPH[status] }));
  const body = el('div', { class: 'wb-cmp-body' }, [
    el('div', { class: 'wb-cmp-name' }, [el('span', { text: entry.subject }), el('code', { text: entry.short })]),
    el('div', { class: 'wb-cmp-kind', text: `${entry.author} · ${entry.date.slice(0, 10)}` }),
  ]);
  for (const line of entryDetails(entry)) body.appendChild(el('div', { class: 'wb-cmp-detail', text: line }));
  for (const r of entry.relationships) body.appendChild(el('div', { class: 'wb-cmp-detail', text: `${r.status} ${r.kind || r.category}: ${r.source} → ${r.target}` }));
  item.appendChild(body);
  const actions = el('div', { class: 'wb-cmp-actions' }, [button('Diff', () => h.post({ type: 'semgit-history-diff', hash: entry.hash }), { class: 'wb-link', title: "VS Code's diff of the file at this commit" })]);
  item.appendChild(actions);
  const choose = () => {
    historyPanel?.querySelectorAll('.wb-cmp-row.is-selected').forEach((n) => n.classList.remove('is-selected'));
    item.classList.add('is-selected');
    h.post({ type: 'semgit-history-show', hash: entry.hash });
    pendingElement = id;
  };
  item.addEventListener('click', (event) => {
    if (!(event.target as HTMLElement).closest('.wb-cmp-actions')) choose();
  });
  item.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') choose();
  });
  return item;
}

let pendingElement: string | null = null;

function showRevision(hash: string, diagrams: ViewerPayload): void {
  const h = hooks!;
  revisionPane?.remove();
  const view = h.viewer()?.current?.()?.view ?? null;
  const pane = el('section', { class: 'wb-rev-pane', 'aria-label': `The model at ${hash.slice(0, 7)}` });
  pane.appendChild(el('div', { class: 'wb-cmp-head' }, [el('div', { class: 'wb-cmp-title' }, [el('strong', { text: 'At ' }), el('code', { text: hash.slice(0, 7) })]), button('✕', () => {
    pane.remove();
    revisionPane = null;
    document.body.classList.remove('wb-rev-open');
  }, { class: 'wb-cmp-exit', title: 'Close' })]));
  const stage = el('div', { class: 'wb-rev-stage' });
  pane.appendChild(stage);
  h.sheet.appendChild(pane);
  revisionPane = pane;
  document.body.classList.add('wb-rev-open');
  setTimeout(() => h.viewer()?.fit?.(), 450);
  h.mountRevision(stage, diagrams, view);
  if (pendingElement) {
    const id = pendingElement;
    setTimeout(() => {
      const node = stage.querySelector(`[data-id="${CSS.escape(id)}"]`);
      node?.classList.add('wb-cmp', 'wb-cmp-modified');
    }, 800);
  }
}
