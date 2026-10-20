// The rows of a table: a sticky header with sortable columns and a filter
// per column, the rows drawn only where the view is (thousands of rows
// scroll as fast as ten), cells linking to elements, cells edited in place.
import { CellEdit, TableCell, TableColumn, TableData, TableRow } from '../../src/pure/tables';
import { el, clear } from './dom';

export interface GridHost {
  sort: () => { key: string; descending: boolean } | null;
  setSort: (sort: { key: string; descending: boolean } | null) => void;
  columnFilter: (key: string) => string;
  setColumnFilter: (key: string, text: string) => void;
  reveal: (id: string, name: string, open: boolean) => void;
  edit: (edit: CellEdit, raw: string, label: string) => void;
  typing: (on: boolean) => void;
}

export const ROW_HEIGHT = 30;
const OVERSCAN = 12;

/** The width of a column, from what it holds. */
function width(column: TableColumn): string {
  if (column.key === 'id') return '128px';
  if (column.key === 'text' || column.key === 'description') return 'minmax(260px, 3fr)';
  if (column.key === 'name') return 'minmax(170px, 1.4fr)';
  if (column.type === 'links') return 'minmax(170px, 1.6fr)';
  if (column.type === 'status') return '132px';
  if (column.type === 'number') return '108px';
  return 'minmax(110px, 1fr)';
}

const VERDICT_CLASS: Record<string, string> = { pass: 'ok', fail: 'fail', pending: 'warn' };

function linkChip(link: NonNullable<TableCell['links']>[number], host: GridHost): HTMLElement {
  const chip = el('button', { class: 'tb-link', title: `${link.kind} ${link.id}\nClick: show in the source and the diagram · Double-click: open its diagram` }, [
    el('span', { class: `tb-link-dot tb-k-${kindFamily(link.kind)}${link.verdict ? ` tb-${VERDICT_CLASS[link.verdict] ?? 'warn'}` : ''}` }),
    link.name,
    link.verdict ? el('span', { class: `tb-verdict tb-${VERDICT_CLASS[link.verdict] ?? 'warn'}-text` }, link.verdict) : null,
  ]);
  chip.addEventListener('click', (event) => {
    event.stopPropagation();
    host.reveal(link.id, link.name, false);
  });
  chip.addEventListener('dblclick', (event) => {
    event.stopPropagation();
    host.reveal(link.id, link.name, true);
  });
  return chip;
}

/** The family a kind's colour comes from. */
export function kindFamily(kind: string): string {
  if (kind.endsWith('Function') || kind === 'OperationalActivity') return 'function';
  if (kind.endsWith('Component') || kind === 'PhysicalNode' || kind === 'OperationalEntity') return 'component';
  if (kind.endsWith('Port')) return 'port';
  if (kind === 'TestCase') return 'test';
  if (kind === 'Requirement') return 'requirement';
  if (kind.includes('Actor') || kind === 'Actor') return 'actor';
  if (kind === 'ExchangeItem' || kind === 'Class' || kind === 'DataType' || kind === 'Enumeration') return 'data';
  return 'other';
}

function cellNode(row: TableRow, column: TableColumn, host: GridHost): HTMLElement {
  const cell = row.cells[column.key] ?? { text: '' };
  const node = el('div', { class: `tb-cell tb-col-${column.type}${column.key === 'id' ? ' tb-mono' : ''}${cell.edit ? ' tb-editable' : ''}`, role: 'gridcell', 'data-key': column.key, tabindex: '-1' });
  if (cell.links && cell.links.length) {
    for (const link of cell.links) node.append(linkChip(link, host));
  } else if (cell.status) {
    node.append(el('span', { class: `tb-pill tb-${cell.status}` }, cell.text || (cell.status === 'gap' ? 'none' : '—')));
  } else if (column.key === 'id') {
    const id = el('button', { class: 'tb-id', title: `Show ${row.id} in the source and the diagram` }, cell.text);
    id.addEventListener('click', (event) => {
      event.stopPropagation();
      host.reveal(row.id, row.name, false);
    });
    id.addEventListener('dblclick', (event) => {
      event.stopPropagation();
      host.reveal(row.id, row.name, true);
    });
    node.append(id);
  } else {
    node.append(el('span', { class: 'tb-text' }, cell.text));
  }
  const tip = cell.links?.length ? cell.links.map((l) => `${l.name} (${l.id}${l.verdict ? `, ${l.verdict}` : ''})`).join('\n') : cell.text;
  if (tip) node.title = cell.edit ? `${tip}\n\nDouble-click or Enter edits` : tip;
  else if (cell.edit) node.title = 'Empty · double-click or Enter edits';
  if (cell.edit) {
    node.addEventListener('dblclick', () => startEdit(node, row, column, cell, host));
    node.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' || event.key === 'F2') {
        event.preventDefault();
        startEdit(node, row, column, cell, host);
      }
    });
  }
  return node;
}

function startEdit(node: HTMLElement, row: TableRow, column: TableColumn, cell: TableCell, host: GridHost): void {
  const edit = cell.edit;
  if (!edit || node.querySelector('.tb-editor')) return;
  const original = cell.text;
  const field = (edit.values && edit.values.length
    ? el('select', { class: 'tb-editor', 'aria-label': `${column.label} of ${row.id}` }, edit.values.map((v) => el('option', { value: v }, v)))
    : el('input', { class: 'tb-editor', type: 'text', 'aria-label': `${column.label} of ${row.id}`, spellcheck: 'false' })) as HTMLInputElement | HTMLSelectElement;
  if (field instanceof HTMLSelectElement && original && !edit.values?.includes(original)) field.prepend(el('option', { value: original }, original));
  field.value = original;
  node.classList.add('tb-editing');
  node.append(field);
  host.typing(true);
  field.focus();
  if (field instanceof HTMLInputElement) field.select();
  let done = false;
  const finish = (commit: boolean) => {
    if (done) return;
    done = true;
    host.typing(false);
    const value = field.value;
    field.remove();
    node.classList.remove('tb-editing');
    if (commit && value !== original) {
      node.classList.add('tb-saving');
      const label = edit.op === 'rename' ? `rename ${row.id}` : `${edit.key} of ${edit.element}`;
      host.edit(edit, value, label);
    }
    node.focus();
  };
  field.addEventListener('keydown', (event) => {
    const key = (event as KeyboardEvent).key;
    if (key === 'Enter') {
      event.preventDefault();
      finish(true);
    } else if (key === 'Escape') {
      event.preventDefault();
      finish(false);
    }
    event.stopPropagation();
  });
  field.addEventListener('blur', () => finish(true));
  if (field instanceof HTMLSelectElement) field.addEventListener('change', () => finish(true));
}

export function renderGrid(container: HTMLElement, table: TableData, rows: TableRow[], host: GridHost): { update: (rows: TableRow[]) => void } {
  const template = table.columns.map(width).join(' ');
  const scroll = el('div', { class: 'tb-scroll', role: 'grid', 'aria-rowcount': String(rows.length + 1), 'aria-colcount': String(table.columns.length) });
  scroll.style.setProperty('--tb-cols', `6px ${template}`);
  const minimum = 6 + table.columns.map(width).reduce((sum, w) => sum + Number((/(\d+)px/.exec(w) ?? ['', '100'])[1]), 0);
  scroll.style.setProperty('--tb-minw', `${minimum}px`);
  const head = el('div', { class: 'tb-header', role: 'row' }, [el('div', { class: 'tb-stripe-head' })]);
  const filters = el('div', { class: 'tb-header tb-filters', role: 'row' }, [el('div', { class: 'tb-stripe-head' })]);
  const sort = host.sort();
  for (const column of table.columns) {
    const sorted = sort?.key === column.key ? (sort.descending ? 'descending' : 'ascending') : 'none';
    const th = el('button', { class: `tb-th${sorted !== 'none' ? ' tb-sorted' : ''}`, role: 'columnheader', 'aria-sort': sorted, title: `Sort by ${column.label}` }, [
      el('span', {}, column.label),
      el('span', { class: 'tb-sort' }, sorted === 'ascending' ? '▲' : sorted === 'descending' ? '▼' : ''),
    ]);
    th.addEventListener('click', () => {
      const next = sorted === 'none' ? { key: column.key, descending: false } : sorted === 'ascending' ? { key: column.key, descending: true } : null;
      host.setSort(next);
    });
    head.append(th);
    const input = el('input', { class: 'tb-col-filter', type: 'search', placeholder: '…', 'aria-label': `Filter ${column.label}`, spellcheck: 'false' }) as HTMLInputElement;
    input.value = host.columnFilter(column.key);
    let timer: number | undefined;
    input.addEventListener('input', () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => host.setColumnFilter(column.key, input.value), 80);
    });
    input.addEventListener('focus', () => host.typing(true));
    input.addEventListener('blur', () => host.typing(false));
    filters.append(el('div', { class: 'tb-filter-cell' }, [input]));
  }
  const sheet = el('div', { class: 'tb-rows', role: 'rowgroup' });
  scroll.append(head, filters, sheet);
  container.append(scroll);

  let current = rows;
  let drawn = '';
  const draw = () => {
    const top = Math.max(0, scroll.scrollTop - head.offsetHeight - filters.offsetHeight);
    const first = Math.max(0, Math.floor(top / ROW_HEIGHT) - OVERSCAN);
    const last = Math.min(current.length, Math.ceil((top + scroll.clientHeight) / ROW_HEIGHT) + OVERSCAN);
    const key = `${first}:${last}:${current.length}`;
    if (key === drawn) return;
    drawn = key;
    clear(sheet);
    sheet.style.height = `${current.length * ROW_HEIGHT}px`;
    for (let i = first; i < last; i++) {
      const row = current[i];
      const line = el('div', { class: `tb-row tb-row-${row.status}`, role: 'row', 'aria-rowindex': String(i + 2), 'data-id': row.id, 'data-uuid': row.uuid });
      line.style.transform = `translateY(${i * ROW_HEIGHT}px)`;
      line.append(el('div', { class: `tb-stripe tb-${row.status}`, title: row.issues?.length ? row.issues.join('\n') : row.status }));
      for (const column of table.columns) line.append(cellNode(row, column, host));
      sheet.append(line);
    }
    if (current.length === 0) sheet.append(el('div', { class: 'tb-none' }, table.rows.length ? 'No row matches the filter.' : 'This table has no row in this scope.'));
  };
  let frame = 0;
  scroll.addEventListener('scroll', () => {
    if (!frame) frame = requestAnimationFrame(() => { frame = 0; draw(); });
  });
  new ResizeObserver(() => { drawn = ''; draw(); }).observe(scroll);
  // Arrows move between cells, as in a spreadsheet.
  scroll.addEventListener('keydown', (event) => {
    const cell = (event.target as HTMLElement).closest('.tb-cell') as HTMLElement | null;
    if (!cell || (event.target as HTMLElement).classList.contains('tb-editor')) return;
    const row = cell.parentElement as HTMLElement;
    let target: Element | null = null;
    if (event.key === 'ArrowRight') target = cell.nextElementSibling;
    if (event.key === 'ArrowLeft') target = cell.previousElementSibling?.classList.contains('tb-cell') ? cell.previousElementSibling : null;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      const index = [...row.children].indexOf(cell);
      const rows = [...sheet.children];
      const next = rows[rows.indexOf(row) + (event.key === 'ArrowDown' ? 1 : -1)];
      target = next?.children[index] ?? null;
    }
    if (target instanceof HTMLElement) {
      event.preventDefault();
      target.focus();
    }
  });
  sheet.addEventListener('click', (event) => {
    const cell = (event.target as HTMLElement).closest('.tb-cell') as HTMLElement | null;
    if (cell) cell.focus();
  });
  draw();
  return {
    update: (rows) => {
      current = rows;
      drawn = '';
      scroll.setAttribute('aria-rowcount', String(rows.length + 1));
      draw();
    },
  };
}
