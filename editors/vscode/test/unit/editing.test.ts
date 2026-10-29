import assert from 'node:assert/strict';
import { test } from 'node:test';
import { writtenRef, alignMoves, distributeMoves, edgeEndIds, freshIdLike, modelIdOf, moveTargets, pasteRequests, quickConnectPlan, reconnectKinds, reconnectPlan, relationshipOfEdge, reversePlan } from '../../src/pure/editing';
import { ElementRecord, RelationshipRecord, Snapshot, ViewerGraph } from '../../src/pure/workbench';

const element = (uuid: string, id: string, kind: string, owner: string | null = 'm', name = id): ElementRecord => ({ uuid, id, kind, name, owner, attributes: {} });
const elements: ElementRecord[] = [
  element('m', 'Model', 'Model', null),
  element('a', 'LC-A', 'LogicalComponent'),
  { ...element('ap', 'LC-A.p', 'ComponentPort', 'a', 'p'), direction: 'out' },
  element('b', 'LC-B', 'LogicalComponent'),
  { ...element('bq', 'LC-B.q', 'ComponentPort', 'b', 'q'), direction: 'in' },
  element('c', 'LC-C', 'LogicalComponent'),
  element('f', 'LF-F', 'LogicalFunction', 'a'),
  element('s', 'M', 'StateMachine'),
  element('off', 'M.Off', 'State', 's', 'Off'),
  element('on', 'M.On', 'State', 's', 'On'),
];
const flow: RelationshipRecord = { uuid: 'r1', kind: 'ComponentExchange', name: 'Flow', source: 'ap', target: 'bq' };
const switchOn: RelationshipRecord = { uuid: 'r2', kind: 'Transition', name: '', source: 'off', target: 'on' };
const snapshot = { elements: { elements, relationships: [flow, switchOn] } } as unknown as Snapshot;

test('drawing ids are model ids, and an edge finds its relationship by kind and ends', () => {
  assert.equal(modelIdOf('LC-A::p'), 'LC-A.p');
  assert.equal(modelIdOf('SF-A::out.p'), 'SF-A.p');
  const lab: ViewerGraph = { id: 'lab', arc: { view: 'lab', title: 'L' }, edges: [{ id: 'ce:Flow', arc: { kind: 'component_exchange' }, sources: ['LC-A::p'], targets: ['LC-B::q'] }] };
  assert.deepEqual(edgeEndIds(lab.edges![0], lab), { source: 'LC-A.p', target: 'LC-B.q' });
  assert.equal(relationshipOfEdge('ce:Flow', lab, snapshot)?.uuid, 'r1');
  const scenario: ViewerGraph = { id: 'es:S', arc: { view: 'es', title: 'S' }, edges: [{ id: 'msg:1', arc: { kind: 'message' }, sources: ['S.LC-A'], targets: ['S.LC-B'] }] };
  assert.deepEqual(edgeEndIds(scenario.edges![0], scenario), { source: 'LC-A', target: 'LC-B' });
  assert.equal(relationshipOfEdge('nope', lab, snapshot), null);
});

test('an exchange reconnected to a box gets a port there, to a port goes straight to it', () => {
  assert.deepEqual(reconnectPlan(flow, 'target', elements.find((e) => e.id === 'LC-C')!, elements), [
    { op: 'add', keyword: 'port in', name: 'q', container: 'LC-C' },
    { op: 'reconnect', kind: 'ComponentExchange', from: 'LC-A.p', to: 'LC-B.q', name: 'Flow', new_to: 'LC-C.q' },
  ]);
  assert.deepEqual(reconnectPlan(flow, 'target', elements.find((e) => e.id === 'LC-A')!, elements), { error: 'an exchange does not connect an element to itself' });
  assert.deepEqual(reconnectPlan(switchOn, 'target', elements.find((e) => e.id === 'M.Off')!, elements), [{ op: 'reconnect', kind: 'Transition', from: 'M.Off', to: 'M.On', new_to: 'M.Off' }]);
  assert.ok('error' in (reconnectPlan(flow, 'target', elements.find((e) => e.id === 'LC-B.q')!, elements) as object));
  assert.deepEqual(reversePlan(flow, elements, [flow]), [
    { op: 'set', element: 'LC-A.p', key: 'direction', value: 'in' },
    { op: 'set', element: 'LC-B.q', key: 'direction', value: 'out' },
    { op: 'reconnect', kind: 'ComponentExchange', from: 'LC-A.p', to: 'LC-B.q', name: 'Flow', new_from: 'LC-B.q', new_to: 'LC-A.p' },
  ]);
  assert.ok('error' in (reversePlan(flow, elements, [flow, { ...flow, uuid: 'r9', name: 'Other' }]) as object), 'a port carrying another exchange does not turn');
  assert.deepEqual(reversePlan(switchOn, elements), [{ op: 'reconnect', kind: 'Transition', from: 'M.Off', to: 'M.On', new_from: 'M.On', new_to: 'M.Off' }]);
  assert.ok('error' in (reversePlan({ uuid: 't', kind: 'Trace', name: '', source: 'a', target: 'b' }, elements) as object));
  assert.ok('error' in (reconnectPlan(flow, 'source', elements.find((e) => e.id === 'LC-B.q')!, elements) as object), 'an in port is no source');
  assert.deepEqual(reconnectKinds('ComponentExchange', 'lab'), ['LogicalComponent', 'ComponentPort']);
  assert.equal(reconnectKinds('Message', 'es'), null);
});

test('boxes align on the outermost edge or the centre, and distribute with equal gaps', () => {
  const boxes = [
    { id: 'a', x: 0, y: 0, width: 100, height: 40 },
    { id: 'b', x: 150, y: 30, width: 50, height: 40 },
    { id: 'c', x: 400, y: 100, width: 100, height: 40 },
  ];
  assert.deepEqual(alignMoves(boxes, 'left'), [{ id: 'b', dx: -150, dy: 0 }, { id: 'c', dx: -400, dy: 0 }]);
  assert.deepEqual(alignMoves(boxes, 'top'), [{ id: 'b', dx: 0, dy: -30 }, { id: 'c', dx: 0, dy: -100 }]);
  assert.deepEqual(alignMoves(boxes, 'bottom').map((m) => m.dy), [100, 70]);
  assert.deepEqual(alignMoves(boxes, 'center').map((m) => [m.id, m.dx]), [['a', 200], ['b', 75], ['c', -200]]);
  assert.deepEqual(alignMoves([boxes[0]], 'left'), []);
  // Span 0..500 holds 250 of boxes: two gaps of 125.
  assert.deepEqual(distributeMoves(boxes, 'horizontal'), [{ id: 'b', dx: 75, dy: 0 }]);
  assert.deepEqual(distributeMoves(boxes.slice(0, 2), 'vertical'), []);
});

test('a cut pastes as a move into the selected container, a copy as duplicates', () => {
  const c = elements.find((e) => e.id === 'LC-C')!;
  assert.deepEqual(pasteRequests({ ids: ['LF-F'], cut: true }, c), [{ op: 'move', element: 'LF-F', container: 'LC-C' }]);
  assert.deepEqual(pasteRequests({ ids: ['LF-F'], cut: false }, c), [{ op: 'duplicate', element: 'LF-F', container: 'LC-C' }]);
  assert.deepEqual(pasteRequests({ ids: ['LC-C'], cut: false }, c), [{ op: 'duplicate', element: 'LC-C' }]);
  assert.ok('error' in (pasteRequests({ ids: ['LF-F'], cut: true }, null) as object));
  assert.ok('error' in (pasteRequests({ ids: [], cut: false }, c) as object));
});

test('move targets: a port goes to another owner of ports, a function to another component', () => {
  const port = elements.find((e) => e.id === 'LC-B.q')!;
  assert.deepEqual(moveTargets(port, elements).map((e) => e.id), ['LC-A', 'LC-C', 'LF-F']);
  assert.deepEqual(moveTargets(port, [...elements, element('cq', 'LC-C.q', 'ComponentPort', 'c', 'q')]).map((e) => e.id), ['LC-A', 'LF-F'], 'an owner with a port of that name is not offered');
  const fn = elements.find((e) => e.id === 'LF-F')!;
  assert.deepEqual(moveTargets(fn, elements).map((e) => e.id), ['LC-B', 'LC-C']);
});

test('quick connect creates an element like the box, with a fresh id, and connects it', () => {
  assert.equal(freshIdLike('LC-CAM', new Set(['LC-1'])), 'LC-2');
  assert.equal(freshIdLike('Camera', new Set()), null);
  const plan = quickConnectPlan('lab', elements.find((e) => e.id === 'LC-A')!, 'Radar', elements, null);
  assert.ok(!('error' in plan));
  if ('error' in plan) return;
  assert.equal(plan.created, 'LC-1');
  assert.deepEqual(plan.requests[0], { op: 'add', keyword: 'component', name: 'Radar', attributes: [['id', '"LC-1"']], layer: 'logical_architecture' });
  assert.ok(plan.requests.some((r) => r.op === 'add_raw' && /component_exchange "LC-A to Radar"/.test(r.declaration)));
  const state = quickConnectPlan('msm', elements.find((e) => e.id === 'M.Off')!, 'Standby', elements, 'M');
  assert.ok(!('error' in state) && state.requests.some((r) => r.op === 'add_raw' && /transition Off -> Standby/.test(r.declaration)));
  assert.ok('error' in (quickConnectPlan('cdb', elements.find((e) => e.id === 'LC-A')!, 'X', elements, null) as object));
});

test('a declaration designates an element by its written id, else by its name', () => {
  const byUuid = new Map<string, ElementRecord>();
  const add = (e: ElementRecord) => byUuid.set(e.uuid, e) && e;
  const written = add({ uuid: 'w', id: 'LC-W', kind: 'LogicalComponent', name: 'Written', owner: null, attributes: { id: 'LC-W' } });
  const derived = add({ uuid: 'd', id: 'LC-Batterie', kind: 'LogicalComponent', name: 'Batterie', owner: null, attributes: {} });
  const port = add({ uuid: 'p', id: 'LC-Batterie.out', kind: 'ComponentPort', name: 'out', owner: 'd', attributes: {} });
  const writtenPort = add({ uuid: 'q', id: 'LC-W.out', kind: 'ComponentPort', name: 'out', owner: 'w', attributes: {} });
  assert.equal(writtenRef(written, byUuid), 'LC-W');
  assert.equal(writtenRef(derived, byUuid), 'Batterie');
  assert.equal(writtenRef(port, byUuid), 'Batterie.out');
  assert.equal(writtenRef(writtenPort, byUuid), 'LC-W.out');
});
