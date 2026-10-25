import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  askPrompt,
  countsOf,
  defaultQuery,
  emptyFilters,
  ImpactedItem,
  ImpactReport,
  impactArgs,
  LINK_KINDS,
  parseReport,
  pathSegments,
  queryTitle,
  ringLayout,
  toHtmlTable,
  toTsv,
  valueLiteral,
  visibleItems,
  whatIfQuery,
  workbenchMarks,
} from '../../src/pure/impact';

function item(id: string, severity: ImpactedItem['severity'], distance: number, layer = 'LA', category: ImpactedItem['category'] = 'architecture', from = 'SEED'): ImpactedItem {
  return { id, uuid: `u-${id}`, name: `Name ${id}`, kind: 'LogicalComponent', layer, category, severity, reason: 'to review', distance, why: `${from} --links--> ${id}`, path: [{ from, relation: 'links', kind: 'reference', to: id }] };
}

const items = [item('A', 'blocker', 1, 'SA', 'requirement'), item('B', 'major', 1, 'LA', 'test'), item('C', 'minor', 2, 'PA', 'view', 'B'), item('D', 'major', 3, 'LA')];

function report(): ImpactReport {
  return {
    success: true,
    model: 'm.arc',
    seeds: [{ id: 'SEED', uuid: 'u-seed', name: 'Seed', kind: 'LogicalComponent', layer: 'LA' }],
    depth: 4,
    kinds: [...LINK_KINDS],
    direction: 'both',
    impacted: items,
    summary: { total: 4, blockers: 1, majors: 2, minors: 1, by_layer: {}, by_category: {}, max_distance: 3 },
    gate: { standard: 'ISO26262', passed: true, blockers: [] },
    links: {},
  };
}

test('the command line of a query names seeds, depth, kinds, direction and the gate', () => {
  const query = { ...defaultQuery(['LC-A', 'LC-B']), depth: 3, kinds: ['satisfies' as const, 'verifies' as const], direction: 'downstream' as const };
  assert.deepEqual(impactArgs('/tmp/m.arc', query, 'iso26262'), ['impact', '/tmp/m.arc', '--element', 'LC-A', '--element', 'LC-B', '--depth', '3', '--kinds', 'satisfies,verifies', '--direction', 'downstream', '--gate-standard', 'iso26262', '-f', 'json']);
  assert.ok(!impactArgs('/tmp/m.arc', defaultQuery(['X']), 'iso26262').includes('--kinds'), 'all kinds is the default');
});

test('a what-if sends its change as JSON and never the seeds; a since query has no change', () => {
  const query = whatIfQuery({ op: 'set', element: 'SF-A', key: 'latency', value: '80 ms' }, defaultQuery(['LC-X']));
  assert.deepEqual(query.elements, []);
  assert.equal(queryTitle(query), 'What if SF-A.latency = 80 ms');
  const args = impactArgs('m.arc', query, 'iso26262');
  assert.equal(args[args.indexOf('--change') + 1], '[{"op":"set","element":"SF-A","key":"latency","value":"80 ms"}]');
  assert.ok(!args.includes('--element'));
  const del = impactArgs('m.arc', whatIfQuery({ op: 'delete', element: 'LC-LOG' }, defaultQuery([])), 'iso26262');
  assert.equal(del[del.indexOf('--delete') + 1], 'LC-LOG');
  const since = impactArgs('m.arc', { ...defaultQuery([]), since: 'HEAD' }, 'iso26262');
  assert.equal(since[since.indexOf('--since') + 1], 'HEAD');
  assert.equal(queryTitle({ ...defaultQuery([]), since: 'main' }), 'Impact of my changes since main');
});

test('values of the what-if form are written as ArcLang', () => {
  assert.equal(valueLiteral('80 ms'), '80 ms');
  assert.equal(valueLiteral('12.5'), '12.5');
  assert.equal(valueLiteral('ASIL-B'), '"ASIL-B"');
  assert.equal(valueLiteral('"ASIL-B"'), '"ASIL-B"');
  assert.equal(valueLiteral('["A", "B"]'), '["A", "B"]');
  assert.deepEqual(whatIfQuery({ op: 'retype', element: 'LC-1', type: 'Sensor unit' }, defaultQuery([])).change?.[0], { op: 'set', element: 'LC-1', key: 'is', value: '"Sensor unit"' });
});

test('parseReport reads the answer, or the reason there is none', () => {
  assert.equal(parseReport(JSON.stringify(report())).ok, true);
  const refused = parseReport(JSON.stringify({ success: false, error: 'edit 1 of 1 refused' }));
  assert.deepEqual(refused, { ok: false, error: 'edit 1 of 1 refused' });
  assert.deepEqual(parseReport('', 'Error: Compilation error: element NOPE not found'), { ok: false, error: 'element NOPE not found' });
});

test('the filters keep what every group chosen allows', () => {
  assert.equal(visibleItems(items, emptyFilters()).length, 4);
  assert.deepEqual(visibleItems(items, { ...emptyFilters(), severities: ['major'] }).map((i) => i.id), ['B', 'D']);
  assert.deepEqual(visibleItems(items, { ...emptyFilters(), severities: ['major'], layers: ['LA'], maxDistance: 1 }).map((i) => i.id), ['B']);
  assert.deepEqual(visibleItems(items, { ...emptyFilters(), categories: ['requirement', 'view'] }).map((i) => i.id), ['A', 'C']);
  assert.deepEqual(visibleItems(items, { ...emptyFilters(), text: 'name d' }).map((i) => i.id), ['D']);
  assert.deepEqual(countsOf(items), { blocker: 1, major: 2, minor: 1 });
});

test('a path reads as clickable elements between its relations', () => {
  const two: ImpactedItem = { ...item('TC', 'major', 2), path: [{ from: 'LC', relation: 'satisfies', kind: 'satisfies', to: 'REQ' }, { from: 'REQ', relation: 'verified by', kind: 'verifies', to: 'TC' }] };
  assert.deepEqual(pathSegments(two), [{ element: 'LC' }, { relation: 'satisfies' }, { element: 'REQ' }, { relation: 'verified by' }, { element: 'TC' }]);
  assert.deepEqual(pathSegments({ ...item('X', 'minor', 1), path: [], why: 'the change itself' }), [{ relation: 'the change itself' }]);
});

test('the rings put a distance on a circle and are deterministic', () => {
  const layout = ringLayout(report().seeds, items);
  const again = ringLayout(report().seeds, [...items].reverse());
  assert.deepEqual(layout.nodes.map((n) => [n.id, Math.round(n.x), Math.round(n.y)]), again.nodes.map((n) => [n.id, Math.round(n.x), Math.round(n.y)]));
  const radius = (id: string) => { const n = layout.nodes.find((x) => x.id === id)!; return Math.round(Math.hypot(n.x, n.y)); };
  assert.equal(radius('SEED'), 0);
  assert.equal(radius('A'), 120);
  assert.equal(radius('C'), 240);
  assert.equal(radius('D'), 360);
  assert.deepEqual(layout.edges.map((e) => `${e.from}>${e.to}`), ['SEED>A', 'SEED>B', 'B>C', 'SEED>D']);
});

test('the marks of the diagrams: the seed apart, the removed left out, the counts by layer view', () => {
  const removed = { ...item('R', 'major', 1, 'LA'), removed: true };
  const { marks, counts } = workbenchMarks(report(), [...items, removed]);
  assert.deepEqual(marks.map((m) => `${m.id}:${m.severity}`), ['SEED:seed', 'A:blocker', 'B:major', 'C:minor', 'D:major']);
  assert.deepEqual(counts.sab, { blocker: 1, major: 0, minor: 0 });
  assert.deepEqual(counts.lab, { blocker: 0, major: 2, minor: 0 });
  // A view is no element of a layer.
  assert.equal(counts.pab, undefined);
});

test('the copied table has a header, a line per row and no tab or newline inside a cell', () => {
  const tricky = { ...item('T', 'minor', 1), reason: 'a\tb\nc' };
  const lines = toTsv([tricky]).trimEnd().split('\n');
  assert.equal(lines.length, 2);
  assert.equal(lines[0].split('\t').length, 9);
  assert.equal(lines[1].split('\t').length, 9);
  assert.match(toHtmlTable([{ ...tricky, name: '<b>' }]), /&lt;b&gt;/);
});

test('the question to the assistant carries the counts, the gate and the worst items first', () => {
  const prompt = askPrompt(report(), defaultQuery(['SEED']), items);
  assert.match(prompt, /^@arclang Explain this change impact/);
  assert.match(prompt, /1 blockers, 2 majors, 1 minors\. Gate PASS\./);
  assert.ok(prompt.indexOf('- [blocker] A') < prompt.indexOf('- [major] B'));
});
