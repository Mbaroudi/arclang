import * as fs from 'node:fs';
import * as path from 'node:path';
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { familyOf, iconFamilies, iconSvg, kindIcon } from '../../src/pure/kind-icons';
import { childTools, countsBadge, diagramsByLayer, dropPlan, filterNodes, findingCounts, idOfPayload, dropPayload, isPanesMessage, modelName, subtreeCounts, tabTitle } from '../../src/pure/panes';
import { attributeGroups, checkValue, commonAttributes, displayValue, fieldLiteral, findingsOf, inheritedOf, listItems, markdownBlocks, quantityOf, quickFixes, referenceCandidates, relationsOf, sharedValue, tracesOf, unitsOf, valueShape } from '../../src/pure/properties';
import { ElementRecord, Metamodel, Snapshot, TreeNode, ViewerGraph } from '../../src/pure/workbench';

const metamodel: Metamodel = {
  kinds: [
    {
      name: 'SystemFunction',
      layer: 'System',
      attributes: [
        { key: 'id', type: { type: 'Identifier' } },
        { key: 'name', type: { type: 'Text' } },
        { key: 'description', type: { type: 'Text' } },
        { key: 'is', type: { type: 'Reference', of: 'Type' } },
        { key: 'tags', type: { type: 'List', of: { type: 'Text' } as never } },
        { key: 'latency', type: { type: 'Quantity', of: 'Time' }, doc: 'Latency.' },
        { key: 'period', type: { type: 'Quantity', of: 'Time' } },
        { key: 'ram', type: { type: 'Quantity', of: 'Information' } },
        { key: 'safety_level', type: { type: 'Enum', of: 'SafetyLevel' } },
        { key: 'variant', type: { type: 'Boolean' } },
        { key: 'multiplicity', type: { type: 'Multiplicity' } },
        { key: 'involves', type: { type: 'List', of: { type: 'Reference', of: 'Element' } as never } },
        { key: 'category', type: { type: 'Text' } },
      ],
    },
    { name: 'Type', layer: 'Transverse', attributes: [] },
    { name: 'LogicalComponent', layer: 'Logical', attributes: [{ key: 'safety_level', type: { type: 'Enum', of: 'SafetyLevel' } }, { key: 'ram', type: { type: 'Quantity', of: 'Information' } }] },
  ],
  enums: [{ name: 'SafetyLevel', values: ['QM', 'ASIL-A', 'ASIL-B'] }],
  units: [
    { symbol: 'ms', dimension: 'Time', factor: 0.001 },
    { symbol: 's', dimension: 'Time', factor: 1 },
    { symbol: 'kg', dimension: 'Mass', factor: 1 },
    { symbol: 'MB', dimension: 'Information', factor: 8e6 },
  ],
};

const q = (value: number, unit: string) => ({ '@type': 'Quantity', value, unit, dimension: 'time' });
const type: ElementRecord = { uuid: 't', id: 'Periodic task', kind: 'Type', name: 'Periodic task', owner: 'm', attributes: { period: q(20, 'ms'), required: ['wcet'] } };
const fn: ElementRecord = { uuid: 'f', id: 'SF-DET', kind: 'SystemFunction', name: 'Detect', owner: 'm', attributes: { id: 'SF-DET', is: 'Periodic task', period: q(20, 'ms'), latency: q(30, 'ms') } };
const fn2: ElementRecord = { ...fn, uuid: 'f2', id: 'SF-EST', name: 'Estimate', attributes: { is: 'Periodic task', period: q(10, 'ms') } };
const comp: ElementRecord = { uuid: 'c', id: 'LC-CAM', kind: 'LogicalComponent', name: 'Camera', owner: 'm', attributes: { safety_level: 'ASIL-B' } };
const port: ElementRecord = { uuid: 'p', id: 'LC-CAM.lane', kind: 'ComponentPort', name: 'lane', owner: 'c', attributes: {} };
const inner: ElementRecord = { uuid: 'lf', id: 'LF-CAP', kind: 'LogicalFunction', name: 'Capture', owner: 'c', attributes: {} };
const ctl: ElementRecord = { uuid: 'k', id: 'LC-CTL', kind: 'LogicalComponent', name: 'Controller', owner: 'm', attributes: {} };
const req: ElementRecord = { uuid: 'r', id: 'REQ-1', kind: 'Requirement', name: 'REQ-1', owner: 'm', attributes: {} };
const tc: ElementRecord = { uuid: 'tc', id: 'TC-1', kind: 'TestCase', name: 'TC-1', owner: 'm', attributes: { verdict: 'pass' } };
const model: ElementRecord = { uuid: 'm', id: 'LKA', kind: 'Model', name: 'Lane Keeping Assist', owner: null, attributes: {} };
const records = [model, type, fn, fn2, comp, port, inner, ctl, req, tc];

const snapshot = (extra: Partial<Snapshot> = {}): Snapshot => ({
  success: true,
  errors: [],
  warnings: [],
  stats: null,
  document: null,
  gate: null,
  findings: [],
  elements: {
    elements: records,
    relationships: [
      { uuid: 'x1', kind: 'ComponentExchange', name: 'LaneFlow', source: 'p', target: 'k' },
      { uuid: 'x2', kind: 'Verification', name: '', source: 'tc', target: 'r' },
      { uuid: 'x3', kind: 'Typing', name: '', source: 'f', target: 't' },
    ],
  },
  diagrams: { title: '', graphs: [], diagnostics: [], links: [{ kind: 'satisfies', source: 'LC-CAM', target: 'REQ-1' }] },
  ...extra,
});

test('every kind has an icon of its family, written as a valid SVG', () => {
  assert.equal(familyOf('LogicalFunction'), 'function');
  assert.equal(familyOf('PhysicalNode'), 'node');
  assert.equal(familyOf('Unheard'), 'element');
  assert.equal(kindIcon('LogicalComponent').fill, '#D3E4F8');
  for (const family of iconFamilies()) assert.match(iconSvg(family), /^<svg [^>]*viewBox="0 0 16 16"[^>]*>(<(circle|rect|path|ellipse|line) [^>]*\/>)+<\/svg>$/);
});

test('the icon files of the tree are the icons of the source (npm run icons)', () => {
  const dir = path.resolve(__dirname, '../../media/kinds');
  for (const family of iconFamilies()) assert.equal(fs.readFileSync(path.join(dir, `${family}.svg`), 'utf8'), `${iconSvg(family)}\n`, family);
});

test('attributes are grouped by what they are: identity, safety, timing, resources, description', () => {
  const groups = attributeGroups(fn, metamodel).map((g) => [g.id, g.attributes.map((a) => a.key)]);
  assert.deepEqual(Object.fromEntries(groups), {
    identity: ['id', 'name', 'is', 'tags'],
    general: ['category'],
    safety: ['safety_level'],
    timing: ['latency', 'period'],
    resources: ['ram'],
    links: ['involves'],
    structure: ['variant', 'multiplicity'],
    description: ['description'],
  });
});

test('a list is read through to its items', () => {
  assert.deepEqual(valueShape({ type: 'List', of: { type: 'Reference', of: 'Element' } as never }), { list: true, item: 'Reference', of: 'Element' });
  assert.deepEqual(listItems(['a', 'b']), ['a', 'b']);
  assert.deepEqual(listItems('["a", "b"]'), ['a', 'b']);
});

test('what a control writes: enumeration members with a dash quoted, quantities bare, references quoted', () => {
  assert.equal(fieldLiteral({ type: 'Enum', of: 'SafetyLevel' }, 'ASIL-B'), '"ASIL-B"');
  assert.equal(fieldLiteral({ type: 'Enum', of: 'SafetyLevel' }, 'QM'), 'QM');
  assert.equal(fieldLiteral({ type: 'Quantity', of: 'Time' }, '25  ms'), '25 ms');
  assert.equal(fieldLiteral({ type: 'Reference', of: 'Type' }, 'Periodic task'), '"Periodic task"');
  assert.equal(fieldLiteral({ type: 'Multiplicity' }, '0..*'), '"0..*"');
  assert.equal(fieldLiteral({ type: 'Boolean' }, 'true'), 'true');
  assert.equal(fieldLiteral({ type: 'List', of: { type: 'Reference' } as never }, ['SF-DET', 'SF-EST']), '["SF-DET", "SF-EST"]');
  assert.equal(fieldLiteral({ type: 'Text' }, 'a "quoted" word'), '"a \\"quoted\\" word"');
});

test('quantities: read from the snapshot or the text, units of a dimension', () => {
  assert.deepEqual(quantityOf(q(20, 'ms')), { value: '20', unit: 'ms' });
  assert.deepEqual(quantityOf('2.5 MB'), { value: '2.5', unit: 'MB' });
  assert.equal(quantityOf('abc'), null);
  assert.deepEqual(unitsOf(metamodel, 'time').map((u) => u.symbol), ['ms', 's']);
  assert.equal(displayValue(q(20, 'ms')), '20 ms');
});

test('a value is checked before the compiler: dimension, unit, enumeration, reference kind, number shapes', () => {
  const at = (key: string) => metamodel.kinds[0].attributes.find((a) => a.key === key)!;
  const ctx = { metamodel, records };
  assert.match(checkValue(at('latency'), '5 kg', ctx)!, /kg measures mass; latency is a time/);
  assert.match(checkValue(at('latency'), '5', ctx)!, /needs a unit/);
  assert.match(checkValue(at('latency'), '5 parsecs', ctx)!, /not a unit/);
  assert.equal(checkValue(at('latency'), '5 ms', ctx), null);
  assert.match(checkValue(at('safety_level'), 'ASIL-Z', ctx)!, /not a SafetyLevel/);
  assert.equal(checkValue(at('safety_level'), 'ASIL-A', ctx), null);
  assert.match(checkValue(at('is'), 'LC-CAM', ctx)!, /is a logical component; Is points at a type/);
  assert.equal(checkValue(at('is'), 'Periodic task', ctx), null);
  assert.match(checkValue(at('involves'), 'SF-DET, NOPE', ctx)!, /no element "NOPE"/);
  assert.match(checkValue(at('multiplicity'), '2..x', ctx)!, /multiplicity/);
  assert.equal(checkValue(at('multiplicity'), '1..*', ctx), null);
});

test('a reference picker offers the kind it points at, else every element', () => {
  assert.deepEqual(referenceCandidates('Type', records).map((r) => r.id), ['Periodic task']);
  assert.ok(referenceCandidates('Element', records, fn).every((r) => r.uuid !== 'f' && r.kind !== 'Model' && !/Port$/.test(r.kind)));
});

test('inherited values: the type\'s, redefined when the element writes another', () => {
  assert.deepEqual(inheritedOf(fn, records).map((i) => [i.key, i.redefined, i.from.id]), [['period', false, 'Periodic task']]);
  assert.deepEqual(inheritedOf(fn2, records).map((i) => [i.key, i.redefined]), [['period', true]]);
  assert.deepEqual(inheritedOf(comp, records), []);
});

test('relationships: owned, and the exchanges of the element\'s ports as its own', () => {
  const r = relationsOf(comp, snapshot());
  assert.deepEqual(r.owned.map((o) => o.other?.id), ['LC-CAM.lane', 'LF-CAP']);
  assert.deepEqual(r.outgoing.map((o) => [o.label, o.other?.id, o.remove?.[0]]), [['Component exchange LaneFlow', 'LC-CTL', { op: 'remove', element: 'LaneFlow' }]]);
  assert.equal(relationsOf(ctl, snapshot()).incoming[0].other?.id, 'LC-CAM');
});

test('traces: satisfies with its removal, verified by with the verdict, allocated functions', () => {
  const ofComp = tracesOf(comp, snapshot());
  assert.deepEqual(ofComp.map((g) => g.label), ['satisfies', 'allocated functions']);
  assert.deepEqual(ofComp[0].rows[0].remove, [{ op: 'remove_trace', from: 'LC-CAM', kind: 'satisfies', to: 'REQ-1' }]);
  const ofReq = tracesOf(req, snapshot());
  assert.deepEqual(ofReq.map((g) => [g.label, g.rows.map((r) => r.other?.id), g.rows[0].verdict]), [['satisfied by', ['LC-CAM'], undefined], ['verified by', ['TC-1'], 'pass']]);
});

test('findings of an element and the fixes the compiler names', () => {
  const findings = [
    { rule: 'metamodel.unit', severity: 'warning', message: 'no unit', element: 'SF-DET', fix: { kind: 'set', element: 'SF-DET', key: 'latency' } as never },
    { rule: 'trace.requirement', severity: 'warning', message: 'untraced', element: 'REQ-1', fix: { kind: 'trace', element: 'REQ-1' } as never },
    { rule: 'x', severity: 'error', message: 'by name', element: 'Detect' },
  ];
  assert.equal(findingsOf(fn, findings).length, 2);
  assert.deepEqual(quickFixes(findings[0]), [{ label: 'Edit latency', action: 'focus', key: 'latency' }]);
  assert.deepEqual(quickFixes(findings[1]), [{ label: 'Add a trace…', action: 'trace' }]);
  assert.deepEqual(quickFixes(findings[2]), []);
});

test('several elements: their common attributes and whether they share a value', () => {
  assert.deepEqual(commonAttributes([fn, comp], metamodel).map((a) => a.key), ['ram', 'safety_level']);
  assert.deepEqual(sharedValue([comp, { ...comp, uuid: 'c2' }], 'safety_level'), { value: 'ASIL-B', mixed: false });
  assert.deepEqual(sharedValue([comp, ctl], 'safety_level'), { value: '', mixed: true });
});

test('a little Markdown as runs of text, never markup', () => {
  const blocks = markdownBlocks('# Title\nPlans the **steering** with `80 ms`\n\n- one\n- *two*\n```\ncode <b>\n```');
  assert.deepEqual(blocks.map((b) => b.type), ['h', 'p', 'li', 'li', 'code']);
  assert.deepEqual(blocks[1].inline, [{ text: 'Plans the ' }, { text: 'steering', bold: true }, { text: ' with ' }, { text: '80 ms', code: true }]);
  assert.equal(blocks[4].inline[0].text, 'code <b>');
});

test('the tree: diagrams under their layer, filtered nodes with their path, findings counted up', () => {
  const graphs = [{ id: 'lab', arc: { view: 'lab', title: 'Logical' } }, { id: 'msm:M', arc: { view: 'msm', title: 'M' } }, { id: 'cap', arc: { view: 'cap', title: 'C' } }] as ViewerGraph[];
  assert.deepEqual([...diagramsByLayer(graphs).entries()].map(([k, v]) => [k, v.map((g) => g.id)]), [['Logical', ['lab']], ['Transverse', ['msm:M']], ['System', ['cap']]]);
  const tree: TreeNode[] = [{ element: comp, children: [{ element: inner, children: [] }, { element: port, children: [] }] }, { element: ctl, children: [] }];
  assert.deepEqual(filterNodes(tree, 'capture').map((n) => [n.element.id, n.children.map((c) => c.element.id)]), [['LC-CAM', ['LF-CAP']]]);
  assert.deepEqual(filterNodes(tree, 'camera').map((n) => n.element.id), ['LC-CAM']);
  assert.equal(filterNodes(tree, '').length, 2);
  const counts = findingCounts(records, [{ rule: 'a', severity: 'error', message: '', element: 'LF-CAP' }, { rule: 'b', severity: 'warning', message: '', element: 'Camera' }]);
  assert.deepEqual(subtreeCounts(tree[0], counts), { errors: 1, warnings: 1, infos: 0 });
  assert.deepEqual(countsBadge({ errors: 0, warnings: 3, infos: 0 }), { text: '3', severity: 'warning' });
  assert.deepEqual(countsBadge({ errors: 0, warnings: 0, infos: 0 }), { text: '', severity: null });
});

test('a new child offers what the palette may declare inside the kind', () => {
  assert.deepEqual(childTools('LogicalComponent').map((t) => t.keyword), ['port in', 'port out', 'component', 'function']);
  assert.deepEqual(childTools('Requirement'), []);
});

test('a node dropped on a diagram: revealed when drawn, added to a declared diagram, else refused', () => {
  const named = { id: 'lab:Chain', arc: { view: 'lab', title: 'Chain' }, children: [{ id: 'LC-CAM', arc: { kind: 'x' }, children: [{ id: 'LF-CAP', arc: { kind: 'f' } }] }] } as ViewerGraph;
  assert.deepEqual(dropPlan(inner, named, ['LC-CAM']), { kind: 'reveal', drawingId: 'LF-CAP' });
  assert.deepEqual(dropPlan(ctl, named, ['LC-CAM']), { kind: 'edit', requests: [{ op: 'set', element: 'Chain', key: 'shows', value: '["LC-CAM", "LC-CTL"]' }], label: 'show LC-CTL on Chain' });
  assert.equal(dropPlan(ctl, { ...named, id: 'lab' }, []).kind, 'refuse');
});

test('drag payloads and messages', () => {
  assert.equal(idOfPayload(dropPayload('file:///m.arc', 'Periodic task')), 'Periodic task');
  assert.equal(idOfPayload(JSON.stringify({ id: 'LC-CAM' })), 'LC-CAM');
  assert.equal(idOfPayload('file:///other'), null);
  assert.ok(isPanesMessage({ type: 'panes-validate' }));
  assert.ok(!isPanesMessage({ type: 'edit' }));
});

test('a diagram\'s own tab is titled "<model> — <diagram>"', () => {
  assert.equal(tabTitle(modelName(snapshot(), 'x.arc'), 'Lane keeping chain'), 'Lane Keeping Assist — Lane keeping chain');
  assert.equal(modelName(null, 'braking.sysml'), 'braking');
});
