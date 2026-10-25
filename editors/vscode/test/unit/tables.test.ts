import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  cellLiteral,
  emptySpec,
  gridOf,
  Matrix,
  parseTable,
  removalRequests,
  rowMatches,
  TableData,
  TableRow,
  tableArgs,
  tableLayer,
  toHtmlTable,
  toTsv,
  traceRequest,
  visibleRows,
} from '../../src/pure/tables';
import { treeByLayer } from '../../src/pure/workbench';

function row(id: string, status: string, priority: string, satisfied: string[] = []): TableRow {
  return {
    id,
    uuid: `u-${id}`,
    kind: 'Requirement',
    name: `Name ${id}`,
    status,
    cells: {
      id: { text: id },
      priority: { text: priority, edit: { op: 'set', element: id, key: 'priority', value_type: 'Enum', values: ['High', 'Low'] } },
      satisfied_by: { text: satisfied.join(', '), links: satisfied.map((s) => ({ id: s, uuid: `u-${s}`, name: s.toLowerCase(), kind: 'LogicalComponent' })) },
      text: { text: `The ${id}\tshall\nwork` },
    },
  };
}

const TABLE: TableData = {
  name: 'Coverage',
  id: 'Coverage',
  uuid: null,
  kind: 'requirements_coverage',
  declared: false,
  spec: emptySpec('requirements_coverage', 'SA'),
  columns: [
    { key: 'id', label: 'Req id', type: 'text' },
    { key: 'priority', label: 'Priority', type: 'text' },
    { key: 'satisfied_by', label: 'Satisfied by', type: 'links' },
    { key: 'text', label: 'Text', type: 'text' },
  ],
  available_columns: [],
  rows: [row('REQ-10', 'ok', 'Low', ['LC-A']), row('REQ-2', 'gap', 'High'), row('REQ-1', 'fail', 'High', ['LC-B', 'LC-A'])],
  matrix: null,
  summary: { rows: 3, ok: 1, warn: 0, gap: 1, fail: 1, gate_standard: 'ISO26262', gate_passed: false, gate_blockers: 2 },
};

test('the filter follows the compiler: words, column:text, =, !=, is:gap, negation, quotes', () => {
  const [ok, gap, fail] = TABLE.rows;
  assert.ok(rowMatches(ok, 'req-10'));
  assert.ok(rowMatches(gap, 'priority=high') && !rowMatches(ok, 'priority=high'));
  assert.ok(rowMatches(ok, 'priority!=high'));
  assert.ok(rowMatches(fail, 'satisfied_by:lc-b'), 'a link matches by its id');
  assert.ok(rowMatches(gap, 'is:gap') && rowMatches(fail, 'is:gap') && !rowMatches(ok, 'is:gap'));
  assert.ok(rowMatches(ok, '-is:gap'));
  assert.ok(rowMatches(ok, '"the req-10"'));
  assert.ok(!rowMatches(ok, 'nowhere:x') && rowMatches(ok, 'nowhere!=x'));
});

test('visible rows: gaps only, column filters and a natural sort', () => {
  const base = { filter: '', columnFilters: {}, gapsOnly: false, sort: null };
  assert.deepEqual(visibleRows(TABLE, { ...base, gapsOnly: true }).map((r) => r.id), ['REQ-2', 'REQ-1']);
  assert.deepEqual(visibleRows(TABLE, { ...base, columnFilters: { satisfied_by: 'lc-a' } }).map((r) => r.id), ['REQ-10', 'REQ-1']);
  assert.deepEqual(visibleRows(TABLE, { ...base, sort: { key: 'id', descending: false } }).map((r) => r.id), ['REQ-1', 'REQ-2', 'REQ-10']);
  assert.deepEqual(visibleRows(TABLE, { ...base, sort: { key: 'id', descending: true } }).map((r) => r.id), ['REQ-10', 'REQ-2', 'REQ-1']);
});

test('copy: TSV with one line per row, tabs and breaks in a cell made spaces; HTML with its header', () => {
  const grid = gridOf(TABLE, TABLE.rows.slice(0, 1));
  const tsv = toTsv(grid);
  assert.equal(tsv, 'Req id\tPriority\tSatisfied by\tText\nREQ-10\tLow\tlc-a\tThe REQ-10 shall work\n');
  const html = toHtmlTable(grid);
  assert.ok(html.startsWith('<table') && html.includes('<th') && html.includes('>lc-a</td>'));
  assert.ok(!toHtmlTable([['<b>']]).includes('<b>'), 'escaped');
});

const MATRIX: Matrix = {
  rows: [{ id: 'REQ-1', uuid: 'r1', name: 'One', kind: 'Requirement' }],
  columns: [{ id: 'LC-A', uuid: 'c1', name: 'A', kind: 'LogicalComponent' }, { id: 'LC-B', uuid: 'c2', name: 'B', kind: 'LogicalComponent' }],
  cells: [{ row: 0, column: 1, kinds: ['satisfies', 'verifies'], items: ['satisfies', 'verifies'], links: [{ kind: 'satisfies', source: 'trace', from: 'LC-B', to: 'REQ-1' }, { kind: 'verifies', source: 'verifies', from: 'LC-B', to: 'REQ-1' }] }],
  creatable: ['satisfies', 'refines'],
  direction: { satisfies: 'column_to_row', refines: 'row_to_column' },
  empty_rows: [],
  empty_columns: [0],
  diagonal: false,
};

test('a click on an empty matrix cell creates the trace the way it goes', () => {
  assert.deepEqual(traceRequest(MATRIX, 0, 0, 'satisfies'), { op: 'add_trace', from: 'LC-A', kind: 'satisfies', to: 'REQ-1' });
  assert.deepEqual(traceRequest(MATRIX, 0, 0, 'refines'), { op: 'add_trace', from: 'REQ-1', kind: 'refines', to: 'LC-A' });
  assert.equal(traceRequest(MATRIX, 0, 0, 'allocates'), null, 'only the kinds the matrix creates');
  assert.equal(traceRequest(MATRIX, 3, 0, 'satisfies'), null);
});

test('a click on a filled cell removes its trace declarations, and keeps what is written elsewhere', () => {
  const { requests, kept } = removalRequests(MATRIX.cells[0]);
  assert.deepEqual(requests, [{ op: 'remove_trace', from: 'LC-B', kind: 'satisfies', to: 'REQ-1' }]);
  assert.deepEqual(kept.map((k) => k.source), ['verifies']);
});

test('a matrix copies as its grid', () => {
  const table: TableData = { ...TABLE, kind: 'traceability_matrix', matrix: MATRIX, rows: [{ ...TABLE.rows[0], uuid: 'r1' }] };
  assert.deepEqual(gridOf(table, table.rows), [['', 'A', 'B'], ['One', '', 'satisfies; verifies']]);
});

test('a typed cell becomes ArcLang source', () => {
  assert.equal(cellLiteral({ op: 'set', element: 'X', value_type: 'Text' }, 'say "hi"'), '"say \\"hi\\""');
  assert.equal(cellLiteral({ op: 'set', element: 'X', value_type: 'Quantity' }, ' 25 ms '), '25 ms');
  assert.equal(cellLiteral({ op: 'set', element: 'X', value_type: 'Enum' }, 'High'), 'High');
  assert.equal(cellLiteral({ op: 'set', element: 'X', value_type: 'Enum' }, 'ASIL-D'), '"ASIL-D"');
  assert.equal(cellLiteral({ op: 'set', element: 'X', value_type: 'List' }, 'a, b'), '["a", "b"]');
  assert.equal(cellLiteral({ op: 'rename', element: 'X' }, 'New name'), '"New name"');
});

test('the command line of a table: declared by name, else its spec', () => {
  assert.deepEqual(tableArgs('m.arc', emptySpec('icd'), 'ICD', 'iso26262'), ['table', 'm.arc', '-f', 'json', '--gate-standard', 'iso26262', '--name', 'ICD']);
  const spec = { ...emptySpec('traceability_matrix', 'LA'), rows: ['Requirement'], matrix_columns: ['LogicalComponent', 'PhysicalNode'], kinds: ['satisfies'], filter: '-is:ok', gaps_only: true };
  assert.deepEqual(tableArgs('m.arc', spec, null, 'do178c').slice(6), ['--kind', 'traceability_matrix', '--scope', 'LA', '--columns', 'LogicalComponent,PhysicalNode', '--rows', 'Requirement', '--kinds', 'satisfies', '--filter', '-is:ok', '--gaps']);
});

test('the answer of the compiler is a table or a reason', () => {
  assert.equal(parseTable(JSON.stringify(TABLE)).ok, true);
  const refused = parseTable('Error: Configuration error: the model declares no table \'X\'');
  assert.ok(!refused.ok && refused.error.includes('declares no table'));
});

test('a table sits in the model tree under the layer of its scope', () => {
  assert.equal(tableLayer({ scope: 'SA' }), 'System');
  assert.equal(tableLayer({ scope: 'Logical Architecture' }), 'Logical');
  assert.equal(tableLayer({ kind: 'icd' }), 'Logical');
  assert.equal(tableLayer({ kind: 'elements' }), 'Transverse');
  const model = { uuid: 'm', id: 'M', kind: 'Model', name: 'M', owner: null, attributes: {} };
  const table = { uuid: 't', id: 'T', kind: 'Table', name: 'Coverage', owner: 'm', attributes: { kind: 'requirements_coverage', scope: 'SA' } };
  const groups = treeByLayer([model, table] as never, null);
  assert.deepEqual(groups.map((g) => g.layer), ['Model', 'System']);
});
