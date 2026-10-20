// A matrix: the column elements across the top (sticky, turned), the row
// elements down the side (sticky), a mark per link in the cells, the
// crossing of the pointer lit. A click on an empty cell creates a trace of
// the chosen kind, a click on a filled one removes it (VS Code asks first).
import { MatrixCell, TableData, TableRow } from '../../src/pure/tables';
import { el, clear } from './dom';
import { kindFamily } from './grid';

export interface MatrixHost {
  reveal: (id: string, name: string, open: boolean) => void;
  create: (row: number, column: number) => void;
  remove: (row: number, column: number) => void;
  kind: () => string | null;
}

const CELL = 30;
/** The height of a row of an N² chart: its cells hold the names of the exchanges. */
const N2_ROW = 44;
const OVERSCAN = 10;

/** The mark of a link kind in a cell. */
const MARKS: Record<string, string> = {
  satisfies: 'S',
  verifies: 'V',
  implements: 'I',
  realizes: 'R',
  refines: 'Rf',
  allocates: 'A',
  validates: 'Va',
  allocated: '●',
  contains: '○',
  deployed: '◆',
};

function mark(cell: MatrixCell, diagonal: boolean): string {
  if (diagonal) return cell.items.join(', ');
  return cell.kinds.map((k) => MARKS[k] ?? k.slice(0, 2)).join('·');
}

export function renderMatrix(container: HTMLElement, table: TableData, rows: TableRow[], host: MatrixHost): { update: (rows: TableRow[]) => void } {
  const matrix = table.matrix!;
  const byCell = new Map<string, MatrixCell>(matrix.cells.map((c) => [`${c.row}:${c.column}`, c]));
  const rowIndex = new Map(matrix.rows.map((r, i) => [r.uuid, i]));
  const emptyRows = new Set(matrix.empty_rows);
  const emptyColumns = new Set(matrix.empty_columns);
  const creatable = matrix.creatable.length > 0;

  const scroll = el('div', { class: `tb-scroll tb-matrix${matrix.diagonal ? ' tb-n2' : ''}`, role: 'grid', 'aria-label': table.name });
  scroll.style.setProperty('--tb-mcols', String(matrix.columns.length));
  const kinds = [...new Set(matrix.cells.flatMap((c) => c.kinds))];
  const legend = matrix.diagonal ? null : el('span', { class: 'tb-mlegend' }, kinds.map((k) => el('span', { class: `tb-mkey tb-m-${k}` }, [el('b', {}, MARKS[k] ?? k.slice(0, 2)), k])));
  const head = el('div', { class: 'tb-mhead', role: 'row' }, [el('div', { class: 'tb-mcorner' }, [el('span', {}, `${matrix.rows.length} × ${matrix.columns.length}`), legend, el('span', { class: 'tb-muted' }, creatable ? 'Click an empty cell to create, a mark to remove' : matrix.diagonal ? 'Row sends to column · hover for the exchanges' : 'Hover a mark for its links')])]);
  matrix.columns.forEach((column, c) => {
    const th = el('button', { class: `tb-mcol${emptyColumns.has(c) ? ' tb-mempty' : ''} tb-k-${kindFamily(column.kind)}-edge`, role: 'columnheader', 'data-col': String(c), title: `${column.kind} ${column.id}${emptyColumns.has(c) ? '\nNothing in this column' : ''}` }, [el('span', {}, column.name)]);
    th.addEventListener('click', () => host.reveal(column.id, column.name, false));
    th.addEventListener('dblclick', () => host.reveal(column.id, column.name, true));
    head.append(th);
  });
  const sheet = el('div', { class: 'tb-mrows', role: 'rowgroup' });
  scroll.append(head, sheet);
  container.append(scroll);

  const lit = (r: number | null, c: number | null) => {
    scroll.querySelectorAll('.tb-lit').forEach((n) => n.classList.remove('tb-lit'));
    if (c !== null) scroll.querySelectorAll(`[data-col="${c}"]`).forEach((n) => n.classList.add('tb-lit'));
    if (r !== null) scroll.querySelector(`.tb-mrow[data-row="${r}"]`)?.classList.add('tb-lit');
  };

  let current = rows;
  let drawn = '';
  const draw = () => {
    const height = matrix.diagonal ? N2_ROW : CELL;
    const top = Math.max(0, scroll.scrollTop - head.offsetHeight);
    const first = Math.max(0, Math.floor(top / height) - OVERSCAN);
    const last = Math.min(current.length, Math.ceil((top + scroll.clientHeight) / height) + OVERSCAN);
    const key = `${first}:${last}:${current.length}`;
    if (key === drawn) return;
    drawn = key;
    clear(sheet);
    sheet.style.height = `${current.length * height}px`;
    for (let i = first; i < last; i++) {
      const r = rowIndex.get(current[i].uuid);
      if (r === undefined) continue;
      const element = matrix.rows[r];
      const line = el('div', { class: `tb-mrow${emptyRows.has(r) ? ' tb-mempty' : ''}`, role: 'row', 'data-row': String(r) });
      line.style.transform = `translateY(${i * height}px)`;
      const name = el('button', { class: `tb-mname tb-k-${kindFamily(element.kind)}-edge`, role: 'rowheader', title: `${element.kind} ${element.id}${emptyRows.has(r) ? `\n${current[i].issues?.join('; ') ?? 'nothing in this row'}` : ''}` }, [el('span', { class: 'tb-mono tb-mid' }, element.id === element.name ? '' : element.id), el('span', {}, element.name)]);
      name.addEventListener('click', () => host.reveal(element.id, element.name, false));
      name.addEventListener('dblclick', () => host.reveal(element.id, element.name, true));
      line.append(name);
      matrix.columns.forEach((column, c) => {
        const found = byCell.get(`${r}:${c}`);
        const self = matrix.diagonal && column.uuid === element.uuid;
        const text = found ? (matrix.diagonal ? found.items.join(', ') : mark(found, false)) : self ? element.name : '';
        const cell = el('button', { class: `tb-mcell${found ? ' tb-mfilled' : ''}${self ? ' tb-mself' : ''}${found ? ` tb-m-${found.kinds[0]}` : ''}`, role: 'gridcell', 'data-col': String(c), 'data-row': String(r) }, matrix.diagonal ? el('span', {}, text) : text);
        if (found) {
          cell.title = `${element.name} × ${column.name}\n${found.items.join('\n')}${found.links.some((l) => l.source === 'trace') ? '\n\nClick removes (asks first)' : ''}`;
        } else if (!self && creatable) {
          cell.title = `${element.name} × ${column.name}\nClick creates a ${host.kind() ?? ''} trace`;
        }
        cell.addEventListener('mouseenter', () => lit(r, c));
        cell.addEventListener('click', () => {
          if (self) return;
          if (found) host.remove(r, c);
          else if (creatable) host.create(r, c);
        });
        line.append(cell);
      });
      sheet.append(line);
    }
    if (current.length === 0) sheet.append(el('div', { class: 'tb-none' }, matrix.rows.length ? 'No row matches the filter.' : 'Nothing to show in this scope.'));
  };
  scroll.addEventListener('mouseleave', () => lit(null, null));
  let frame = 0;
  scroll.addEventListener('scroll', () => {
    if (!frame) frame = requestAnimationFrame(() => { frame = 0; draw(); });
  });
  new ResizeObserver(() => { drawn = ''; draw(); }).observe(scroll);
  draw();
  return {
    update: (rows) => {
      current = rows;
      drawn = '';
      draw();
    },
  };
}
