// The impact analysis page: what a change reaches, as the compiler computes
// it. A header (the seed, the counts by severity, the gate), a filter row,
// the graph (rings by distance, severity colours), the table with the "why"
// path of each element, and, for a what-if, the panel of what the change
// does to the model. The page only shows: every computation, every write
// and every export is the host's.
import {
  CATEGORIES,
  Category,
  countsOf,
  DEFAULT_DEPTH,
  emptyFilters,
  FromImpactView,
  ImpactFilters,
  ImpactQuery,
  ImpactReport,
  LAYERS,
  LINK_KINDS,
  LINK_LABELS,
  LinkKind,
  MAX_DEPTH,
  queryTitle,
  Severity,
  toHtmlTable,
  toTsv,
  ToImpactView,
  visibleItems,
} from '../../src/pure/impact';
import { button, clear, el } from '../dom';
import { GraphView, renderGraph } from './graph';
import { renderTable, Sort } from './table';
import { renderWhatIf } from './whatif';

const vscode = acquireVsCodeApi() as unknown as { postMessage: (m: FromImpactView) => void; getState: () => unknown; setState: (s: unknown) => void };
const post = (message: FromImpactView) => vscode.postMessage(message);

interface Saved {
  filters: ImpactFilters;
  sort: Sort;
  highlight: boolean;
  graph: boolean;
}

const saved = (vscode.getState() as Saved | undefined) ?? null;
let filters: ImpactFilters = saved?.filters ?? emptyFilters();
let sort: Sort = saved?.sort ?? { key: 'severity', descending: false };
let highlight = saved?.highlight ?? true;
let showGraph = saved?.graph ?? true;
let report: ImpactReport | null = null;
let query: ImpactQuery | null = null;
let file = '';
let computedMs = 0;
let aiAvailable = false;
let applied = false;
let stale = false;
let error: string | null = null;
let selected: string | null = null;
let notice: { text: string; error: boolean } | null = null;
let graphView: GraphView | null = null;

const root = document.getElementById('impact') as HTMLElement;
const sections = {
  header: el('header', { class: 'ip-header' }),
  whatif: el('div', { class: 'ip-whatif-slot' }),
  filters: el('div', { class: 'ip-filters', role: 'toolbar', 'aria-label': 'Filters' }),
  body: el('div', { class: 'ip-body' }),
  status: el('footer', { class: 'ip-status', role: 'status', 'aria-live': 'polite' }),
};
root.append(sections.header, sections.whatif, sections.filters, sections.body, sections.status);

function persist(): void {
  vscode.setState({ filters, sort, highlight, graph: showGraph } satisfies Saved);
}

function say(text: string, isError = false): void {
  notice = { text, error: isError };
  renderStatus();
  setTimeout(() => {
    if (notice?.text === text) {
      notice = null;
      renderStatus();
    }
  }, 6000);
}

/** The query changed from the filters: the host computes it again. */
function ask(next: Partial<ImpactQuery>): void {
  if (!query) return;
  query = { ...query, ...next };
  post({ type: 'query', query });
}

// --- Header --------------------------------------------------------------------

function chip(text: string, tone: string, title?: string): HTMLElement {
  return el('span', { class: `ip-chip tone-${tone}`, text, title });
}

function renderHeader(): void {
  const head = sections.header;
  clear(head);
  const title = query ? queryTitle(query) : 'Impact analysis';
  const top = el('div', { class: 'ip-title-row' }, [el('h1', { class: 'ip-title', text: title }), el('span', { class: `ip-fresh${stale ? ' is-stale' : ''}`, text: stale ? 'recomputing…' : report ? `${file} · ${computedMs} ms` : '' })]);
  head.appendChild(top);
  if (!report) return;
  const seeds = el('div', { class: 'ip-seeds' });
  if (report.since) seeds.appendChild(el('span', { class: 'ip-muted', text: `since ${report.since}:` }));
  for (const seed of report.seeds) {
    const link = el('button', { type: 'button', class: 'ip-seed', title: `${seed.kind} · ${seed.layer}: show in its diagram` }, [el('span', { class: 'ip-seed-layer', text: seed.layer }), seed.name || seed.id]);
    link.addEventListener('click', () => post({ type: 'reveal', id: seed.id, open: true }));
    seeds.appendChild(link);
  }
  const counts = countsOf(report.impacted);
  const w = report.what_if;
  const gate = w ? el('span', { class: `ip-chip ip-gate-chip ${w.gate_after.passed ? 'tone-ok' : 'tone-blocker'}`, title: `Production gate ${w.gate_after.standard}` }, [`gate ${w.gate_before.passed ? 'PASS' : 'FAIL'}`, el('span', { 'aria-hidden': 'true', text: ' → ' }), el('b', { text: w.gate_after.passed ? 'PASS' : 'FAIL' })]) : chip(`gate ${report.gate.passed ? 'PASS' : 'FAIL'}`, report.gate.passed ? 'ok' : 'blocker', `Production gate ${report.gate.standard}`);
  const chips = el('div', { class: 'ip-chips' }, [
    chip(`${counts.blocker} blocker${counts.blocker === 1 ? '' : 's'}`, 'blocker', 'A requirement newly unsatisfied, an ASIL element, a gate blocker'),
    chip(`${counts.major} major`, 'major', 'An interface change, a test to redo'),
    chip(`${counts.minor} minor`, 'minor', 'A view or a plain element to review'),
    gate,
    chip(`${report.impacted.length} reached · depth ${report.depth}`, 'plain'),
  ]);
  const tools = el('div', { class: 'ip-tools' });
  const seed = report.seeds[0];
  if (seed && !w && !report.since) tools.appendChild(button('What if…', () => post({ type: 'open-what-if', id: seed.id }), { class: 'ip-btn', title: 'Delete it, change an attribute, retype or move it: computed on a copy' }));
  tools.append(
    button(highlight ? 'Highlight on diagrams: on' : 'Highlight on diagrams: off', () => {
      highlight = !highlight;
      persist();
      post({ type: 'highlight', on: highlight });
      renderHeader();
    }, { class: `ip-btn${highlight ? ' is-on' : ''}`, 'aria-pressed': String(highlight), title: 'Outline the impacted elements on the workbench diagrams, with the counts on the layer tabs' }),
    button(showGraph ? 'Graph: on' : 'Graph: off', () => {
      showGraph = !showGraph;
      persist();
      renderBody();
      renderHeader();
    }, { class: `ip-btn${showGraph ? ' is-on' : ''}`, 'aria-pressed': String(showGraph) }),
    button('Copy as table', copyTable, { class: 'ip-btn', title: 'Copy the rows shown, ready to paste in a spreadsheet' }),
    button('Markdown', () => post({ type: 'export', format: 'md' }), { class: 'ip-btn', title: 'Change-request report as Markdown' }),
    button('HTML', () => post({ type: 'export', format: 'html' }), { class: 'ip-btn', title: 'Change-request report as a web page' }),
    button('Excel', () => post({ type: 'export', format: 'xlsx' }), { class: 'ip-btn', title: 'Change-request report as a workbook: summary, impact, gate, coverage…' }),
  );
  if (aiAvailable) tools.appendChild(button('✦ Ask AI', () => post({ type: 'ask' }), { class: 'ip-btn ip-ai', title: 'Explain this impact and propose mitigations' }));
  head.append(seeds, chips, tools);
}

function copyTable(): void {
  if (!report) return;
  const items = visibleItems(report.impacted, filters);
  const tsv = toTsv(items);
  const done = (ok: boolean) => post({ type: 'copied', ok, tsv, rows: items.length });
  // Rich: an HTML table for documents, plain tab-separated text for spreadsheets.
  const html = toHtmlTable(items);
  if (navigator.clipboard && 'write' in navigator.clipboard && typeof ClipboardItem !== 'undefined') {
    navigator.clipboard
      .write([new ClipboardItem({ 'text/plain': new Blob([tsv], { type: 'text/plain' }), 'text/html': new Blob([html], { type: 'text/html' }) })])
      .then(() => done(true), () => done(false));
  } else {
    done(false);
  }
}

// --- Filters ---------------------------------------------------------------------

function toggle<T>(list: T[], value: T): T[] {
  return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

function group(label: string, children: HTMLElement[]): HTMLElement {
  return el('div', { class: 'ip-group', role: 'group', 'aria-label': label }, [el('span', { class: 'ip-group-label', text: label }), ...children]);
}

function toggleChip(label: string, on: boolean, change: () => void, tone = 'plain', title?: string): HTMLElement {
  return button(label, change, { class: `ip-toggle tone-${tone}${on ? ' is-on' : ''}`, 'aria-pressed': String(on), title });
}

function kindsMenu(): HTMLElement {
  const active = query?.kinds ?? [...LINK_KINDS];
  const details = el('details', { class: 'ip-menu' }, [el('summary', { class: 'ip-toggle', text: `Links ${active.length === LINK_KINDS.length ? '(all)' : `(${active.length}/${LINK_KINDS.length})`}` })]);
  const list = el('div', { class: 'ip-menu-list' });
  for (const kind of LINK_KINDS) {
    const count = report?.links[kind] ?? 0;
    const box = el('input', { type: 'checkbox', id: `ip-kind-${kind}` });
    box.checked = active.includes(kind);
    box.addEventListener('change', () => {
      const next: LinkKind[] = box.checked ? [...active, kind] : active.filter((k) => k !== kind);
      ask({ kinds: next.length ? LINK_KINDS.filter((k) => next.includes(k)) : [...LINK_KINDS] });
    });
    list.appendChild(el('label', { class: 'ip-menu-item', for: `ip-kind-${kind}` }, [box, ` ${LINK_LABELS[kind]}`, el('span', { class: 'ip-muted', text: ` ${count}` })]));
  }
  list.appendChild(button('All links', () => ask({ kinds: [...LINK_KINDS] }), { class: 'ip-link-button' }));
  details.appendChild(list);
  return details;
}

function renderFilters(): void {
  const bar = sections.filters;
  clear(bar);
  if (!report || !query) return;
  const counts = countsOf(report.impacted);
  const severities: [Severity, string, number][] = [['blocker', 'Blockers', counts.blocker], ['major', 'Majors', counts.major], ['minor', 'Minors', counts.minor]];
  bar.appendChild(group('Severity', severities.map(([s, label, n]) => toggleChip(`${label} ${n}`, filters.severities.includes(s), () => { filters = { ...filters, severities: toggle(filters.severities, s) }; persist(); renderFilters(); renderBody(); }, s))));
  const present = new Set(report.impacted.map((i) => i.layer));
  bar.appendChild(group('Layer', LAYERS.filter((l) => present.has(l)).map((l) => toggleChip(`${l} ${report!.summary.by_layer[l] ?? 0}`, filters.layers.includes(l), () => { filters = { ...filters, layers: toggle(filters.layers, l) }; persist(); renderFilters(); renderBody(); }))));
  const categories = CATEGORIES.filter((c) => (report!.summary.by_category[c.id] ?? 0) > 0);
  bar.appendChild(group('Category', categories.map((c) => toggleChip(`${c.label} ${report!.summary.by_category[c.id]}`, filters.categories.includes(c.id as Category), () => { filters = { ...filters, categories: toggle(filters.categories, c.id as Category) }; persist(); renderFilters(); renderBody(); }))));

  const depth = el('input', { type: 'range', min: '1', max: String(MAX_DEPTH), step: '1', class: 'ip-depth', 'aria-label': 'Depth' });
  depth.value = String(query.depth || DEFAULT_DEPTH);
  const depthValue = el('output', { class: 'ip-depth-value', text: depth.value });
  let timer: ReturnType<typeof setTimeout> | undefined;
  depth.addEventListener('input', () => {
    depthValue.textContent = depth.value;
    clearTimeout(timer);
    timer = setTimeout(() => ask({ depth: Number(depth.value) }), 250);
  });
  const direction = el('select', { class: 'ip-select', 'aria-label': 'Direction' }, [
    el('option', { value: 'both', text: 'Both ways' }),
    el('option', { value: 'downstream', text: 'What depends on it' }),
    el('option', { value: 'upstream', text: 'What it depends on' }),
  ]);
  direction.value = query.direction;
  direction.addEventListener('change', () => ask({ direction: direction.value as ImpactQuery['direction'] }));
  const search = el('input', { type: 'search', class: 'ip-search', placeholder: 'Filter: id, name, why…', 'aria-label': 'Filter the impacted elements' });
  search.value = filters.text;
  search.addEventListener('input', () => {
    filters = { ...filters, text: search.value };
    persist();
    renderBody();
  });
  bar.append(group('Depth', [depth, depthValue]), group('Direction', [direction]), kindsMenu(), search);
}

// --- Body --------------------------------------------------------------------------

function select(id: string | null): void {
  selected = id;
  graphView?.select(id);
  for (const row of sections.body.querySelectorAll<HTMLElement>('.ip-row')) row.classList.toggle('is-selected', row.dataset.id === id);
  if (id) sections.body.querySelector<HTMLElement>(`.ip-row[data-id="${CSS.escape(id)}"]`)?.scrollIntoView({ block: 'nearest' });
}

function renderBody(): void {
  const body = sections.body;
  clear(body);
  graphView = null;
  if (error) {
    body.appendChild(el('div', { class: 'ip-error', role: 'alert' }, [el('h2', { text: 'No analysis' }), el('p', { text: error })]));
    return;
  }
  if (!report) {
    body.appendChild(el('p', { class: 'ip-muted', text: 'Computing…' }));
    return;
  }
  const items = visibleItems(report.impacted, filters);
  body.classList.toggle('has-graph', showGraph);
  if (showGraph) {
    graphView = renderGraph(report.seeds, items, selected, {
      select: (id) => select(id),
      reveal: (id) => post({ type: 'reveal', id, open: true }),
    });
    body.appendChild(graphView.root);
  }
  const caption = el('div', { class: 'ip-caption', text: items.length === report.impacted.length ? `${items.length} impacted element${items.length === 1 ? '' : 's'}` : `${items.length} of ${report.impacted.length} shown` });
  body.appendChild(
    el('div', { class: 'ip-list' }, [
      caption,
      renderTable(items, sort, selected, { select, reveal: (id, open) => post({ type: 'reveal', id, open }), analyze: (id) => post({ type: 'analyze', id }) }, (next) => {
        sort = next;
        persist();
        renderBody();
      }),
    ]),
  );
}

function renderWhatIfSlot(): void {
  const slot = sections.whatif;
  clear(slot);
  if (!report?.what_if) return;
  slot.appendChild(renderWhatIf(report.what_if, applied, { apply: () => post({ type: 'apply' }), discard: () => post({ type: 'discard' }), reveal: (id) => post({ type: 'reveal', id, open: true }) }));
}

function renderStatus(): void {
  const status = sections.status;
  clear(status);
  if (notice) status.appendChild(el('span', { class: notice.error ? 'is-error' : '', text: notice.text }));
  else if (report) status.appendChild(el('span', { class: 'ip-muted', text: `${report.kinds.length} kinds of link followed · ${Object.values(report.links).reduce((a, b) => a + b, 0)} links in the model` }));
}

function renderAll(): void {
  root.classList.toggle('is-stale', stale);
  renderHeader();
  renderWhatIfSlot();
  renderFilters();
  renderBody();
  renderStatus();
}

window.addEventListener('message', (event: MessageEvent<ToImpactView>) => {
  const message = event.data;
  switch (message.type) {
    case 'busy':
      stale = message.stale;
      root.classList.toggle('is-stale', stale);
      renderHeader();
      return;
    case 'report':
      report = message.report;
      query = message.query;
      file = message.file;
      computedMs = message.computedMs;
      aiAvailable = message.aiAvailable;
      applied = message.applied;
      stale = false;
      error = null;
      if (selected && !report.impacted.some((i) => i.id === selected)) selected = null;
      renderAll();
      return;
    case 'error':
      error = message.error;
      query = message.query;
      file = message.file;
      report = null;
      stale = false;
      renderAll();
      return;
    case 'result':
      say(message.message, !message.ok);
      return;
  }
});

renderAll();
post({ type: 'ready' });
