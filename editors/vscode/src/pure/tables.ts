// The live tables of a model, as `arclang table -f json` computes them, and
// what the table view does with them without VS Code: the filter (the same
// rules as the compiler's `filter:`), sorting, the text copied to the
// clipboard, the command line of a table and the edit of a matrix click.

export interface TableLink {
  id: string;
  uuid: string;
  name: string;
  kind: string;
  verdict?: string;
}

export interface CellEdit {
  op: 'set' | 'rename';
  element: string;
  key?: string;
  value_type?: string;
  values?: string[];
}

export interface TableCell {
  text: string;
  links?: TableLink[];
  status?: string;
  edit?: CellEdit;
}

export interface TableColumn {
  key: string;
  label: string;
  type: string;
}

export interface TableRow {
  id: string;
  uuid: string;
  kind: string;
  name: string;
  status: string;
  issues?: string[];
  cells: Record<string, TableCell>;
}

export interface MatrixLink {
  kind: string;
  source: string;
  from: string;
  to: string;
  uuid?: string;
}

export interface MatrixCell {
  row: number;
  column: number;
  kinds: string[];
  items: string[];
  links: MatrixLink[];
}

export interface Matrix {
  rows: TableLink[];
  columns: TableLink[];
  cells: MatrixCell[];
  creatable: string[];
  direction: Record<string, 'row_to_column' | 'column_to_row'>;
  empty_rows: number[];
  empty_columns: number[];
  diagonal: boolean;
}

export interface TableSpec {
  name?: string | null;
  id?: string | null;
  kind: string;
  scope?: string | null;
  row_type?: string | null;
  columns: string[];
  rows: string[];
  matrix_columns: string[];
  kinds: string[];
  filter?: string | null;
  gaps_only: boolean;
}

export interface TableSummary {
  rows: number;
  ok: number;
  warn: number;
  gap: number;
  fail: number;
  gate_standard: string;
  gate_passed: boolean;
  gate_blockers: number;
}

export interface TableData {
  name: string;
  id: string;
  uuid: string | null;
  kind: string;
  declared: boolean;
  spec: TableSpec;
  columns: TableColumn[];
  available_columns: TableColumn[];
  rows: TableRow[];
  matrix: Matrix | null;
  summary: TableSummary;
}

export const TABLE_KINDS: { kind: string; label: string; detail: string }[] = [
  { kind: 'requirements_coverage', label: 'Requirement coverage', detail: 'per requirement: text, satisfied by, verified by with verdicts, gate status' },
  { kind: 'traceability_matrix', label: 'Traceability matrix', detail: 'rows x columns of elements, the trace kinds in the cells' },
  { kind: 'allocation_matrix', label: 'Allocation matrix', detail: 'functions x components or nodes' },
  { kind: 'interface_n2', label: 'N² interfaces', detail: 'components x components, the exchanges in the cells' },
  { kind: 'icd', label: 'Interface control document', detail: 'one row per exchange: ends, ports, items, protocol, bandwidth, latency' },
  { kind: 'elements', label: 'Elements', detail: 'any kind, the columns picked among its attributes' },
];

export function isMatrix(kind: string): boolean {
  return kind === 'traceability_matrix' || kind === 'allocation_matrix' || kind === 'interface_n2';
}

/** The answer of `arclang table -f json`: one table, or a reason. */
export function parseTable(text: string): { ok: true; table: TableData } | { ok: false; error: string } {
  try {
    const value = JSON.parse(text) as TableData;
    if (!value || typeof value !== 'object' || !Array.isArray(value.rows) || !Array.isArray(value.columns)) return { ok: false, error: 'the compiler did not answer a table' };
    return { ok: true, table: value };
  } catch {
    return { ok: false, error: (text.trim().split('\n').pop() ?? '').replace(/^Error:\s*/, '') || 'the compiler did not answer a table' };
  }
}

/** The arguments of `arclang table` for a spec (the file first). */
export function tableArgs(file: string, spec: TableSpec, declaredName: string | null, gateStandard: string): string[] {
  const args = ['table', file, '-f', 'json', '--gate-standard', gateStandard];
  if (declaredName) return [...args, '--name', declaredName];
  args.push('--kind', spec.kind);
  if (spec.name) args.push('--name', spec.name);
  if (spec.scope) args.push('--scope', spec.scope);
  if (spec.row_type) args.push('--row-type', spec.row_type);
  const columns = isMatrix(spec.kind) ? spec.matrix_columns : spec.columns;
  if (columns.length) args.push('--columns', columns.join(','));
  if (spec.rows.length) args.push('--rows', spec.rows.join(','));
  if (spec.kinds.length) args.push('--kinds', spec.kinds.join(','));
  if (spec.filter) args.push('--filter', spec.filter);
  if (spec.gaps_only) args.push('--gaps');
  return args;
}

/** A spec with nothing but its kind. */
export function emptySpec(kind: string, scope?: string): TableSpec {
  return { kind, scope: scope ?? null, columns: [], rows: [], matrix_columns: [], kinds: [], gaps_only: false };
}

// --- Filter -----------------------------------------------------------------

type Op = 'contains' | 'equals' | 'not_equals';
interface Term {
  negated: boolean;
  column: string | null;
  op: Op;
  value: string;
}

function split(text: string): string[] {
  const out: string[] = [];
  let current = '';
  let quoted = false;
  for (const c of text) {
    if (c === '"') quoted = !quoted;
    else if (/\s/.test(c) && !quoted) {
      if (current) out.push(current);
      current = '';
    } else current += c;
  }
  if (current) out.push(current);
  return out;
}

export function parseFilter(text: string): Term[] {
  return split(text).map((raw) => {
    const negated = raw.startsWith('-') && raw.length > 1;
    const body = (negated ? raw.slice(1) : raw).toLowerCase();
    for (const [token, op] of [['!=', 'not_equals'], ['=', 'equals'], [':', 'contains']] as const) {
      const at = body.indexOf(token);
      if (at > 0 && /^[\p{L}\p{N}_]+$/u.test(body.slice(0, at))) return { negated, column: body.slice(0, at), op, value: body.slice(at + token.length) };
    }
    return { negated, column: null, op: 'contains' as Op, value: body };
  });
}

function compare(term: Term, text: string): boolean {
  const t = text.toLowerCase();
  if (term.op === 'equals') return t === term.value;
  if (term.op === 'not_equals') return t !== term.value;
  return t.includes(term.value);
}

function termMatches(term: Term, row: TableRow): boolean {
  const cellMatches = (cell: TableCell) => compare(term, cell.text) || (cell.links ?? []).some((l) => compare(term, l.id));
  switch (term.column) {
    case 'is':
      return term.value === 'gap' || term.value === 'gaps' ? row.status === 'gap' || row.status === 'fail' : row.status === term.value;
    case 'id':
      return compare(term, row.id);
    case null:
      return [row.id, row.name, row.kind].some((t) => compare(term, t)) || Object.values(row.cells).some(cellMatches);
    default: {
      if (term.column === 'name' && !row.cells.name) return compare(term, row.name);
      if (term.column === 'kind' && !row.cells.kind) return compare(term, row.kind);
      const cell = row.cells[term.column];
      return cell ? cellMatches(cell) : term.op === 'not_equals';
    }
  }
}

/** Whether a row passes the filter text: every term must match. */
export function rowMatches(row: TableRow, filter: string): boolean {
  return parseFilter(filter).every((term) => termMatches(term, row) !== term.negated);
}

export interface ViewState {
  filter: string;
  /** Per column: text the column must contain. */
  columnFilters: Record<string, string>;
  gapsOnly: boolean;
  sort: { key: string; descending: boolean } | null;
}

const STATUS_ORDER: Record<string, number> = { fail: 0, gap: 1, warn: 2, ok: 3 };

/** The text a cell sorts and copies as. */
export function cellText(cell: TableCell | undefined): string {
  if (!cell) return '';
  if (cell.links && cell.links.length) return cell.links.map((l) => (l.verdict ? `${l.name} (${l.verdict})` : l.name)).join(', ');
  return cell.text;
}

/** Natural order: `REQ-2` before `REQ-10`. */
export function naturalCompare(a: string, b: string): number {
  return a.localeCompare(b, undefined, { numeric: true, sensitivity: 'base' });
}

/** The rows the view shows, in its order. */
export function visibleRows(table: TableData, state: ViewState): TableRow[] {
  let rows = table.rows.filter((row) => (!state.gapsOnly || row.status === 'gap' || row.status === 'fail') && rowMatches(row, state.filter));
  for (const [key, text] of Object.entries(state.columnFilters)) {
    const wanted = text.trim().toLowerCase();
    if (wanted) rows = rows.filter((row) => cellText(row.cells[key]).toLowerCase().includes(wanted) || (row.cells[key]?.links ?? []).some((l) => l.id.toLowerCase().includes(wanted)));
  }
  if (state.sort) {
    const { key, descending } = state.sort;
    const column = table.columns.find((c) => c.key === key);
    const value = (row: TableRow) => (key === 'status' || column?.type === 'status' ? String(STATUS_ORDER[row.cells[key]?.status ?? row.status] ?? 9) : cellText(row.cells[key]));
    rows = [...rows].sort((a, b) => {
      const order = naturalCompare(value(a), value(b)) || naturalCompare(a.id, b.id);
      return descending ? -order : order;
    });
  }
  return rows;
}

// --- Clipboard ----------------------------------------------------------------

function tsvField(text: string): string {
  return text.replace(/[\t\r\n]+/g, ' ');
}

/** The grid the view shows: the header, then the rows (a matrix as its grid). */
export function gridOf(table: TableData, rows: TableRow[]): string[][] {
  if (table.matrix) {
    const m = table.matrix;
    const kept = new Set(rows.map((r) => r.uuid));
    const grid: string[][] = [['', ...m.columns.map((c) => c.name)]];
    m.rows.forEach((row, r) => {
      if (!kept.has(row.uuid)) return;
      const line = [row.name];
      m.columns.forEach((column, c) => {
        if (m.diagonal && column.uuid === row.uuid) line.push('■');
        else {
          const cell = m.cells.find((x) => x.row === r && x.column === c);
          line.push(cell ? (m.diagonal ? cell.items.join('; ') : cell.kinds.join('; ')) : '');
        }
      });
      grid.push(line);
    });
    return grid;
  }
  return [table.columns.map((c) => c.label), ...rows.map((row) => table.columns.map((c) => cellText(row.cells[c.key])))];
}

/** Tab separated, what Excel and PowerPoint paste as a table. */
export function toTsv(grid: string[][]): string {
  return grid.map((line) => line.map(tsvField).join('\t')).join('\n') + '\n';
}

function escapeHtml(text: string): string {
  return text.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c);
}

/** The same grid as an HTML table: PowerPoint and Word paste it as a table with its header. */
export function toHtmlTable(grid: string[][]): string {
  const [header, ...body] = grid;
  const head = `<tr>${(header ?? []).map((h) => `<th style="text-align:left;background:#efece5;border:1px solid #b9b3a6;padding:2px 6px">${escapeHtml(h)}</th>`).join('')}</tr>`;
  const rows = body.map((line) => `<tr>${line.map((t) => `<td style="border:1px solid #dcd8cf;padding:2px 6px">${escapeHtml(t)}</td>`).join('')}</tr>`).join('');
  return `<table style="border-collapse:collapse;font-family:Calibri,sans-serif;font-size:11pt">${head}${rows}</table>`;
}

// --- Matrix clicks -------------------------------------------------------------

/** The edit a click on an empty matrix cell makes: a trace of `kind`, the way it goes. */
export function traceRequest(matrix: Matrix, row: number, column: number, kind: string): { op: 'add_trace'; from: string; kind: string; to: string } | null {
  const r = matrix.rows[row];
  const c = matrix.columns[column];
  if (!r || !c || !matrix.creatable.includes(kind)) return null;
  const rowToColumn = (matrix.direction[kind] ?? 'column_to_row') === 'row_to_column';
  return rowToColumn ? { op: 'add_trace', from: r.id, kind, to: c.id } : { op: 'add_trace', from: c.id, kind, to: r.id };
}

/** The edits that remove what a cell shows; what a `trace` declaration does not hold is left, and said. */
export function removalRequests(cell: MatrixCell): { requests: { op: 'remove_trace'; from: string; kind: string; to: string }[]; kept: MatrixLink[] } {
  const requests = cell.links.filter((l) => l.source === 'trace').map((l) => ({ op: 'remove_trace' as const, from: l.from, kind: l.kind, to: l.to }));
  return { requests, kept: cell.links.filter((l) => l.source !== 'trace') };
}

/** The value typed in a cell as ArcLang source for `set`. */
export function cellLiteral(edit: CellEdit, raw: string): string {
  const text = raw.trim();
  switch (edit.value_type) {
    case 'Number':
    case 'Quantity':
    case 'Boolean':
      return text;
    case 'Enum':
      return /^[A-Za-z_][A-Za-z0-9_]*$/.test(text) ? text : JSON.stringify(text);
    case 'List':
    case 'Reference': {
      if (edit.value_type === 'Reference') return JSON.stringify(text);
      const items = text.replace(/^\[|\]$/g, '').split(',').map((s) => s.trim()).filter(Boolean);
      return `[${items.map((i) => JSON.stringify(i.replace(/^"|"$/g, ''))).join(', ')}]`;
    }
    default:
      return JSON.stringify(text);
  }
}

/** The layer a table belongs to in the model tree: its scope's. */
export function tableLayer(attributes: Record<string, unknown>): string {
  const scope = String(attributes.scope ?? '').toLowerCase().replace(/[^a-z0-9]/g, '');
  const layers: Record<string, string> = {
    oa: 'Operational', operational: 'Operational', operationalanalysis: 'Operational',
    sa: 'System', system: 'System', systemanalysis: 'System',
    la: 'Logical', logical: 'Logical', logicalarchitecture: 'Logical',
    pa: 'Physical', physical: 'Physical', physicalarchitecture: 'Physical',
    epbs: 'Epbs', data: 'Data',
  };
  if (layers[scope]) return layers[scope];
  const kind = String(attributes.kind ?? '');
  if (kind === 'requirements_coverage') return 'System';
  if (kind === 'allocation_matrix' || kind === 'interface_n2' || kind === 'icd') return 'Logical';
  return 'Transverse';
}

// --- Messages between the table view and VS Code ------------------------------

export type ToTableView =
  | { type: 'table'; table: TableData; computedMs: number; file: string; history: { undo: string | null; redo: string | null } }
  | { type: 'error'; error: string; file: string }
  | { type: 'busy' }
  | { type: 'result'; ok: boolean; message?: string }
  | { type: 'history'; undo: string | null; redo: string | null }
  | { type: 'copy-request' };

export type FromTableView =
  | { type: 'ready' }
  | { type: 'reveal'; id: string; name: string; open: boolean }
  | { type: 'edit'; edit: CellEdit; value: string; label: string }
  | { type: 'add-trace'; row: number; column: number; kind: string }
  | { type: 'remove-trace'; row: number; column: number }
  | { type: 'export'; format: 'csv' | 'xlsx' | 'html' | 'md' }
  | { type: 'copied'; rows: number; ok: boolean; tsv: string }
  | { type: 'save-declaration'; spec: TableSpec }
  | { type: 'spec'; spec: TableSpec }
  | { type: 'typing'; on: boolean }
  | { type: 'undo' }
  | { type: 'redo' };
