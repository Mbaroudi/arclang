// The table view: a live table or matrix of the model, as the compiler
// computes it. Filters, sorting and copying happen here; every change of
// the model (an edit in a cell, a trace created from the matrix) is a
// request to VS Code, which runs `arclang edit` and sends the table again.
import {
  cellLiteral,
  FromTableView,
  gridOf,
  isMatrix,
  TableData,
  TableRow,
  toHtmlTable,
  toTsv,
  ToTableView,
  ViewState,
  visibleRows,
} from '../../src/pure/tables';
import { renderGrid, GridHost } from './grid';
import { renderMatrix, MatrixHost } from './matrix';
import { el, clear } from './dom';

const vscode = acquireVsCodeApi() as unknown as { postMessage: (m: FromTableView) => void; getState: () => unknown; setState: (s: unknown) => void };
const post = (message: FromTableView) => vscode.postMessage(message);

const SCOPES: [string, string][] = [['', 'Whole model'], ['OA', 'Operational'], ['SA', 'System'], ['LA', 'Logical'], ['PA', 'Physical'], ['EPBS', 'EPBS']];
const ROW_TYPES = ['Requirement', 'Capability', 'SystemFunction', 'LogicalComponent', 'LogicalFunction', 'ComponentPort', 'PhysicalNode', 'BehaviorComponent', 'PhysicalFunction', 'ExchangeItem', 'TestCase', 'Hazard', 'Constraint', 'Class'];

interface Saved {
  view: ViewState;
  traceKind: string | null;
}

const saved = (vscode.getState() as Saved | undefined) ?? null;
let view: ViewState = saved?.view ?? { filter: '', columnFilters: {}, gapsOnly: false, sort: null };
let traceKind: string | null = saved?.traceKind ?? null;
let table: TableData | null = null;
let file = '';
let computedMs = 0;
let message: { text: string; error: boolean } | null = null;
let history: { undo: string | null; redo: string | null } = { undo: null, redo: null };

const root = document.getElementById('tables') as HTMLElement;

function persist(): void {
  vscode.setState({ view, traceKind } satisfies Saved);
}

function say(text: string, error = false): void {
  message = { text, error };
  renderStatus();
}

// --- Header -------------------------------------------------------------------

const KIND_LABELS: Record<string, string> = {
  requirements_coverage: 'Requirement coverage',
  traceability_matrix: 'Traceability matrix',
  allocation_matrix: 'Allocation matrix',
  interface_n2: 'N² interfaces',
  icd: 'Interface control document',
  elements: 'Elements',
};

const header = el('header', { class: 'tb-head' });
const tools = el('div', { class: 'tb-tools', role: 'toolbar', 'aria-label': 'Table tools' });
const body = el('main', { class: 'tb-body' });
const status = el('footer', { class: 'tb-status', 'aria-live': 'polite' });
root.append(header, tools, body, status);

function renderHeader(): void {
  clear(header);
  if (!table) {
    header.append(el('h1', { class: 'tb-title' }, 'Table'));
    return;
  }
  const s = table.summary;
  const title = el('div', { class: 'tb-titles' }, [
    el('span', { class: 'tb-kind' }, KIND_LABELS[table.kind] ?? table.kind),
    el('h1', { class: 'tb-title' }, table.name),
    el('span', { class: 'tb-chip', title: 'What the rows are taken from' }, table.spec.scope ? `scope ${table.spec.scope}` : 'whole model'),
    table.declared ? el('span', { class: 'tb-chip tb-chip-declared', title: `Declared in ${file}: versioned with the model (${table.uuid ?? ''})` }, 'declared') : el('span', { class: 'tb-chip tb-chip-adhoc', title: 'Not in the model: Save as table declaration keeps it' }, 'not saved'),
    el('span', { class: 'tb-live', title: 'Computed by the compiler on every change of the model' }, [el('span', { class: 'tb-dot' }), `live · ${computedMs} ms`]),
  ]);
  const total = Math.max(1, s.ok + s.warn + s.gap + s.fail);
  const bar = el('div', { class: 'tb-bar', role: 'img', 'aria-label': `${s.ok} ok, ${s.warn} pending, ${s.gap} with a gap, ${s.fail} failing` });
  for (const [key, n] of [['ok', s.ok], ['warn', s.warn], ['gap', s.gap], ['fail', s.fail]] as const) {
    if (n === 0) continue;
    // Set through the CSSOM: the page's policy refuses style attributes.
    const segment = el('span', { class: `tb-seg tb-${key}` });
    segment.style.flexGrow = String(n / total);
    bar.append(segment);
  }
  const legend = el('div', { class: 'tb-legend' }, [
    legendItem('ok', s.ok, table.kind === 'requirements_coverage' ? 'covered' : 'ok'),
    s.warn ? legendItem('warn', s.warn, 'pending') : null,
    legendItem('gap', s.gap, isMatrix(table.kind) ? 'empty' : s.gap === 1 ? 'gap' : 'gaps'),
    s.fail ? legendItem('fail', s.fail, 'failing') : null,
  ]);
  const gate = el(
    'span',
    { class: `tb-gate ${s.gate_passed ? 'tb-gate-pass' : 'tb-gate-block'}`, title: s.gate_passed ? `The production gate (${s.gate_standard}) passes` : `The production gate (${s.gate_standard}) is blocked; ${s.gate_blockers} of its blockers are about these rows` },
    s.gate_passed ? `Gate ${s.gate_standard} · pass` : s.gate_blockers ? `Gate ${s.gate_standard} · ${s.gate_blockers} blocker${s.gate_blockers === 1 ? '' : 's'} here` : `Gate ${s.gate_standard} · blocked elsewhere`,
  );
  header.append(title, el('div', { class: 'tb-health' }, [bar, legend, gate]));
}

function legendItem(key: string, n: number, label: string): HTMLElement {
  const item = el('button', { class: `tb-legend-item tb-${key}-text`, title: `Show only these rows (is:${key})` }, [el('span', { class: `tb-swatch tb-${key}` }), `${n} ${label}`]);
  item.addEventListener('click', () => {
    const term = `is:${key}`;
    view = { ...view, filter: view.filter.includes(term) ? view.filter.replace(term, '').trim() : `${view.filter} ${term}`.trim() };
    filterInput.value = view.filter;
    persist();
    renderBody();
  });
  return item;
}

// --- Tools ----------------------------------------------------------------------

const filterInput = el('input', { class: 'tb-filter', type: 'search', placeholder: 'Filter: words · priority:high · status=covered · is:gap · -term', 'aria-label': 'Filter the rows', spellcheck: 'false' }) as HTMLInputElement;
filterInput.value = view.filter;
let filterTimer: number | undefined;
filterInput.addEventListener('input', () => {
  window.clearTimeout(filterTimer);
  filterTimer = window.setTimeout(() => {
    view = { ...view, filter: filterInput.value };
    persist();
    renderBody(true);
  }, 80);
});
for (const input of [filterInput]) {
  input.addEventListener('focus', () => post({ type: 'typing', on: true }));
  input.addEventListener('blur', () => post({ type: 'typing', on: false }));
}

function button(label: string, title: string, onClick: () => void, extra = ''): HTMLButtonElement {
  const b = el('button', { class: `tb-button ${extra}`.trim(), title }, label) as HTMLButtonElement;
  b.addEventListener('click', onClick);
  return b;
}

function select(options: [string, string][], value: string, title: string, onChange: (v: string) => void): HTMLSelectElement {
  const s = el('select', { class: 'tb-select', title, 'aria-label': title }) as HTMLSelectElement;
  for (const [v, l] of options) s.append(el('option', { value: v }, l));
  s.value = value;
  s.addEventListener('change', () => onChange(s.value));
  return s;
}

function changeSpec(change: (spec: TableData['spec']) => TableData['spec']): void {
  if (!table) return;
  post({ type: 'spec', spec: change({ ...table.spec }) });
}

function renderTools(): void {
  clear(tools);
  if (!table) return;
  const t = table;
  const left = el('div', { class: 'tb-tools-left' }, [filterInput]);
  left.append(select(SCOPES, t.spec.scope ?? '', 'Scope: the layer (or element) the rows are taken from', (v) => changeSpec((s) => ({ ...s, scope: v || null }))));
  if (t.kind === 'elements' || t.kind === 'requirements_coverage') {
    const current = t.spec.row_type ?? (t.kind === 'requirements_coverage' ? 'Requirement' : '');
    const types = [...new Set([current, ...ROW_TYPES].filter(Boolean))];
    left.append(select([...(t.kind === 'elements' ? [['', 'Any kind'] as [string, string]] : []), ...types.map((k) => [k, k] as [string, string])], current, 'Row type: the kind of the rows', (v) => changeSpec((s) => ({ ...s, row_type: v || null, columns: [] }))));
  }
  if (!t.matrix) left.append(columnPicker(t));
  const gaps = el('label', { class: 'tb-toggle', title: 'Show only the rows with a gap or a failure' }, [el('input', { type: 'checkbox' }), 'Only gaps']);
  const box = gaps.querySelector('input') as HTMLInputElement;
  box.checked = view.gapsOnly;
  box.addEventListener('change', () => {
    view = { ...view, gapsOnly: box.checked };
    persist();
    renderBody();
  });
  left.append(gaps);
  if (t.matrix && t.matrix.creatable.length) {
    if (!traceKind || !t.matrix.creatable.includes(traceKind)) traceKind = t.matrix.creatable[0];
    left.append(el('span', { class: 'tb-sep' }), el('span', { class: 'tb-label' }, 'Click creates'), select(t.matrix.creatable.map((k) => [k, k]), traceKind, 'The kind of trace a click on an empty cell creates', (v) => { traceKind = v; persist(); }));
  }
  const right = el('div', { class: 'tb-tools-right' }, [
    button('Copy', 'Copy the rows shown as a table (pastes into Excel, PowerPoint, Word)', copyTable),
    button('CSV', 'Export the table as CSV', () => post({ type: 'export', format: 'csv' })),
    button('Excel', 'Export the table as an Excel workbook (.xlsx)', () => post({ type: 'export', format: 'xlsx' })),
    t.declared ? null : button('Save as table…', 'Add this table to the model as a `table` declaration: versioned with it, exported to SysML v2', () => post({ type: 'save-declaration', spec: { ...t.spec, filter: view.filter || t.spec.filter || null, gaps_only: view.gapsOnly || t.spec.gaps_only } }), 'tb-primary'),
  ]);
  tools.append(left, right);
}

function columnPicker(t: TableData): HTMLElement {
  const details = el('details', { class: 'tb-columns' });
  details.append(el('summary', { class: 'tb-button', title: 'Pick the columns' }, `Columns (${t.columns.length})`));
  const list = el('div', { class: 'tb-columns-list', role: 'menu' });
  const shown = new Set(t.columns.map((c) => c.key));
  for (const column of t.available_columns) {
    const item = el('label', { class: 'tb-columns-item' }, [el('input', { type: 'checkbox' }), column.label, el('code', {}, column.key)]);
    const input = item.querySelector('input') as HTMLInputElement;
    input.checked = shown.has(column.key);
    input.addEventListener('change', () => {
      const keys = t.available_columns.map((c) => c.key).filter((k) => (k === column.key ? input.checked : shown.has(k)));
      const ordered = [...t.columns.map((c) => c.key).filter((k) => keys.includes(k)), ...keys.filter((k) => !t.columns.some((c) => c.key === k))];
      changeSpec((s) => ({ ...s, columns: ordered }));
    });
    list.append(item);
  }
  details.append(list);
  return details;
}

// --- Copy -----------------------------------------------------------------------

function shownRows(): TableRow[] {
  return table ? visibleRows(table, view) : [];
}

/** The rows shown as TSV and as an HTML table. */
function clipboardPayload(): { tsv: string; html: string; rows: number } | null {
  if (!table) return null;
  const rows = shownRows();
  const grid = gridOf(table, rows);
  return { tsv: toTsv(grid), html: toHtmlTable(grid), rows: rows.length };
}

let copiedHere = false;

// A copy outside a field (⌘C, Copy) puts the rows shown on the clipboard,
// as text (TSV: Excel) and as HTML (a table in PowerPoint and Word).
document.addEventListener('copy', (event) => {
  const active = document.activeElement as HTMLElement | null;
  const inField = !!active && (active.tagName === 'INPUT' || active.tagName === 'SELECT' || active.tagName === 'TEXTAREA');
  if (inField || window.getSelection()?.toString()) return;
  const payload = clipboardPayload();
  if (!payload || !event.clipboardData) return;
  event.clipboardData.setData('text/plain', payload.tsv);
  event.clipboardData.setData('text/html', payload.html);
  event.preventDefault();
  copiedHere = true;
  post({ type: 'copied', rows: payload.rows, ok: true, tsv: payload.tsv });
  say(`Copied ${payload.rows} row${payload.rows === 1 ? '' : 's'} as a table (TSV and HTML)`);
});

function copyTable(): void {
  const payload = clipboardPayload();
  if (!payload) return;
  copiedHere = false;
  try {
    document.execCommand('copy');
  } catch {
    copiedHere = false;
  }
  // Where the page may not write the clipboard, VS Code writes the text.
  if (!copiedHere) {
    post({ type: 'copied', rows: payload.rows, ok: false, tsv: payload.tsv });
    say(`Copied ${payload.rows} row${payload.rows === 1 ? '' : 's'} as TSV`);
  }
}

document.addEventListener('keydown', (event) => {
  const target = event.target as HTMLElement | null;
  const inField = !!target && (target.tagName === 'INPUT' || target.tagName === 'SELECT' || target.tagName === 'TEXTAREA');
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'f' && !inField) {
    event.preventDefault();
    filterInput.focus();
    filterInput.select();
  }
});

// --- Body -------------------------------------------------------------------------

const gridHost: GridHost = {
  sort: () => view.sort,
  setSort: (sort) => {
    view = { ...view, sort };
    persist();
    renderBody();
  },
  columnFilter: (key) => view.columnFilters[key] ?? '',
  setColumnFilter: (key, text) => {
    view = { ...view, columnFilters: { ...view.columnFilters, [key]: text } };
    persist();
    renderBody(true);
  },
  reveal: (id, name, open) => post({ type: 'reveal', id, name, open }),
  edit: (edit, raw, label) => {
    post({ type: 'edit', edit, value: cellLiteral(edit, raw), label });
    say(`Applying ${label}…`);
  },
  typing: (on) => post({ type: 'typing', on }),
};

const matrixHost: MatrixHost = {
  reveal: (id, name, open) => post({ type: 'reveal', id, name, open }),
  create: (row, column) => {
    if (!traceKind) return;
    post({ type: 'add-trace', row, column, kind: traceKind });
    say(`Creating a ${traceKind} trace…`);
  },
  remove: (row, column) => post({ type: 'remove-trace', row, column }),
  kind: () => traceKind,
};

let renderedGrid: { update: (rows: TableRow[]) => void } | null = null;

/** Draw the table again; `rowsOnly` keeps the header (and the field being typed in). */
function renderBody(rowsOnly = false): void {
  if (!table) return;
  const rows = shownRows();
  if (rowsOnly && renderedGrid) renderedGrid.update(rows);
  else {
    const scroll = body.querySelector('.tb-scroll');
    const at = scroll ? { top: scroll.scrollTop, left: scroll.scrollLeft } : null;
    clear(body);
    renderedGrid = table.matrix ? renderMatrix(body, table, rows, matrixHost) : renderGrid(body, table, rows, gridHost);
    const next = body.querySelector('.tb-scroll');
    if (next && at) {
      next.scrollTop = at.top;
      next.scrollLeft = at.left;
    }
  }
  renderStatus(rows.length);
}

function renderStatus(shown?: number): void {
  clear(status);
  if (!table) return;
  const count = shown ?? shownRows().length;
  status.append(el('span', {}, `${count} of ${table.rows.length} row${table.rows.length === 1 ? '' : 's'}`));
  if (history.undo) status.append(el('span', { class: 'tb-muted' }, `⌘Z undoes ${history.undo}`));
  if (message) status.append(el('span', { class: message.error ? 'tb-error' : 'tb-note' }, message.text));
  status.append(el('span', { class: 'tb-muted tb-right' }, file));
}

window.addEventListener('message', (event: MessageEvent<ToTableView>) => {
  const data = event.data;
  switch (data.type) {
    case 'table': {
      const kindChanged = !table || table.kind !== data.table.kind;
      table = data.table;
      file = data.file;
      computedMs = data.computedMs;
      history = data.history;
      if (kindChanged) traceKind = null;
      document.body.classList.remove('tb-stale');
      renderHeader();
      renderTools();
      renderBody();
      break;
    }
    case 'error':
      file = data.file;
      document.body.classList.add('tb-stale');
      say(`The model does not compile: the table shows its last state. ${data.error}`, true);
      if (!table) {
        clear(body);
        body.append(el('div', { class: 'tb-empty' }, [el('strong', {}, 'No table yet'), el('p', {}, data.error)]));
      }
      break;
    case 'busy':
      document.body.classList.add('tb-busy');
      window.setTimeout(() => document.body.classList.remove('tb-busy'), 600);
      break;
    case 'result':
      if (data.message) say(data.message, !data.ok);
      else if (data.ok) message = null;
      renderStatus();
      break;
    case 'history':
      history = { undo: data.undo, redo: data.redo };
      renderStatus();
      break;
    case 'copy-request':
      copyTable();
      break;
  }
});

renderHeader();
body.append(el('div', { class: 'tb-empty' }, [el('strong', {}, 'Computing the table…'), el('p', {}, 'The compiler computes it from the model, and again on every change.')]));
post({ type: 'ready' });
