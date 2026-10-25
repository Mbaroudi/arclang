// Unit tests of the stacking order, the layers, groups and locks (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const layers = require(join(viewer, 'arcviz-layers.js'));
const collapse = require(join(viewer, 'arcviz-collapse.js'));
const scopes = require(join(viewer, 'arcviz-scope.js'));

const unit = (id, x, y, layer = 0) => ({ id, rect: { x, y, width: 100, height: 40 }, layer });
// A, B and C overlap in a row; D stands apart.
const row = () => [unit('A', 0, 0), unit('B', 50, 10), unit('C', 90, 20), unit('D', 500, 0)];

test('to the front and to the back keep the order of what moves', () => {
  assert.deepEqual(layers.restacked(row(), ['A', 'B'], 'front'), ['C', 'D', 'A', 'B']);
  assert.deepEqual(layers.restacked(row(), ['C', 'D'], 'back'), ['C', 'D', 'A', 'B']);
});

test('forward takes one step past the nearest box it overlaps', () => {
  assert.deepEqual(layers.restacked(row(), ['A'], 'forward'), ['B', 'A', 'C', 'D']);
  assert.deepEqual(layers.restacked(['B', 'A', 'C', 'D'].map((id) => row().find((u) => u.id === id)), ['A'], 'forward'), ['B', 'C', 'A', 'D']);
});

test('backward takes one step under the nearest box it overlaps', () => {
  assert.deepEqual(layers.restacked(row(), ['C'], 'backward'), ['A', 'C', 'B', 'D']);
});

test('a box that overlaps nothing above it does not move forward', () => {
  assert.deepEqual(layers.restacked(row(), ['D'], 'forward'), ['A', 'B', 'C', 'D']);
  assert.deepEqual(layers.restacked(row(), ['D'], 'backward'), ['A', 'B', 'C', 'D']);
});

test('several selected move forward together and keep their order', () => {
  assert.deepEqual(layers.restacked(row(), ['A', 'B'], 'forward'), ['C', 'A', 'B', 'D']);
});

test('forward only passes boxes of the same layer', () => {
  const units = [unit('A', 0, 0, 0), unit('B', 50, 10, 1), unit('C', 60, 10, 0)];
  assert.deepEqual(layers.restacked(units, ['A'], 'forward'), ['B', 'C', 'A']);
});

test('the stacking is written 1, 2, 3 from the bottom', () => {
  assert.deepEqual(layers.stackValues(['C', 'A', 'B']), { C: 1, A: 2, B: 3 });
});

test('same size, width and height follow the reference box', () => {
  const boxes = [{ id: 'A', width: 100, height: 40 }, { id: 'B', width: 160, height: 60 }, { id: 'C', width: 160, height: 60 }];
  assert.deepEqual(layers.sameSize(boxes, 'B', 'both'), [{ id: 'A', dw: 60, dh: 20 }]);
  assert.deepEqual(layers.sameSize(boxes, 'A', 'width'), [{ id: 'B', dw: -60, dh: 0 }, { id: 'C', dw: -60, dh: 0 }]);
  assert.deepEqual(layers.sameSize(boxes, 'A', 'height').map((c) => c.dh), [-20, -20]);
});

const box = (id, x, y, children = []) => ({ id, x, y, width: 120, height: 80, labels: [{ text: id }], ports: [{ id: `${id}::p`, x: 116, y: 30, width: 8, height: 8 }], children, arc: { kind: 'logical_component' } });
const graph = () => ({
  id: 'lab',
  arc: { view: 'lab' },
  children: [box('A', 20, 20, [box('A1', 10, 30)]), box('B', 200, 20), box('C', 380, 20)],
  edges: [
    { id: 'ce:AB', sources: ['A::p'], targets: ['B::p'], container: 'lab', sections: [{ id: 's', startPoint: { x: 140, y: 54 }, endPoint: { x: 200, y: 54 } }], labels: [{ text: 'AB', x: 160, y: 40 }], arc: { kind: 'component_exchange' } },
    { id: 'ce:BC', sources: ['B::p'], targets: ['C::p'], container: 'lab', sections: [{ id: 's', startPoint: { x: 320, y: 54 }, endPoint: { x: 380, y: 54 } }], labels: [], arc: { kind: 'component_exchange' } },
  ],
});
const tags = { A: ['Safety'], 'ce:BC': ['Comms'] };
const storeOf = (file) => {
  const store = layers.create({ tagsOf: (view, id) => tags[id] || [] });
  store.load(file);
  return store;
};

test('a hidden layer is not drawn, nor what it holds, nor the lines that touch it', () => {
  const drawn = storeOf({ layers: { hidden: ['Safety'] } }).apply(graph(), 'lab');
  assert.deepEqual(drawn.children.map((n) => n.id), ['B', 'C']);
  assert.deepEqual(drawn.edges.map((e) => e.id), ['ce:BC']);
  const comms = storeOf({ layers: { hidden: ['Comms'] } }).apply(graph(), 'lab');
  assert.deepEqual(comms.edges.map((e) => e.id), ['ce:AB'], 'a tagged line is on its layer');
});

test('nothing to arrange: the graph is the one laid out', () => {
  const laid = graph();
  assert.equal(storeOf({}).apply(laid, 'lab'), laid);
});

test('a layer above another draws its boxes later among their siblings', () => {
  const drawn = storeOf({ layers: { order: ['Safety', ''] } }).apply(graph(), 'lab');
  assert.deepEqual(drawn.children.map((n) => n.id), ['B', 'C', 'A']);
  const below = storeOf({ layers: { order: ['', 'Safety'] } }).apply(graph(), 'lab');
  assert.deepEqual(below.children.map((n) => n.id), ['A', 'B', 'C']);
});

test('a group frames its members where its topmost member is, the members right after it', () => {
  const drawn = storeOf({ groups: { lab: [{ id: 'g1', members: ['A', 'C'], label: 'Pair' }] } }).apply(graph(), 'lab');
  assert.deepEqual(drawn.children.map((n) => n.id), ['B', 'group:g1', 'A', 'C']);
  const frame = drawn.children[1];
  assert.equal(frame.arc.kind, 'diagram_group');
  assert.equal(frame.labels[0].text, 'Pair');
  assert.ok(frame.x <= 20 && frame.x + frame.width >= 500 && frame.y <= 20 && frame.y + frame.height >= 100);
});

test('a group of boxes in different containers is not drawn', () => {
  const drawn = storeOf({ groups: { lab: [{ id: 'g1', members: ['A1', 'B'] }] } }).apply(graph(), 'lab');
  assert.ok(!drawn.children.some((n) => n.id === 'group:g1'));
});

test('groups nest: the outer frame holds the inner one', () => {
  const drawn = storeOf({ groups: { lab: [{ id: 'g2', members: ['group:g1', 'C'] }, { id: 'g1', members: ['A', 'B'] }] } }).apply(graph(), 'lab');
  assert.deepEqual(drawn.children.map((n) => n.id), ['group:g2', 'group:g1', 'A', 'B', 'C']);
});

test('a collapsed group is one box, the lines to its members end on its border', () => {
  const drawn = storeOf({ groups: { lab: [{ id: 'g1', members: ['A', 'B'], collapsed: true }] } }).apply(graph(), 'lab');
  assert.deepEqual(drawn.children.map((n) => n.id), ['group:g1', 'C']);
  const frame = drawn.children[0];
  assert.equal(frame.arc.collapsed, true);
  assert.equal(frame.arc.folded, 2);
  assert.deepEqual(drawn.edges.map((e) => e.id), ['ce:BC'], 'the line between two members is inside it');
  const start = drawn.edges[0].sections[0].startPoint;
  assert.ok(Math.abs(start.x - (frame.x + frame.width)) < 0.5 || Math.abs(start.y - frame.y) < 0.5 || Math.abs(start.y - (frame.y + frame.height)) < 0.5, JSON.stringify(start));
});

test('the state goes to the layout file and back, checked', () => {
  const file = { layers: { order: ['Safety', ''], hidden: ['Comms'], locked: ['Safety'] }, groups: { lab: [{ id: 'g1', members: ['A', 'B'], label: 'Pair', collapsed: true }, { id: 'g1', members: ['C'] }, { id: 'bad', members: [] }] }, locked: ['C', 'C', 42] };
  const store = storeOf(file);
  assert.deepEqual(store.toFile(), { layers: { order: ['Safety', ''], hidden: ['Comms'], locked: ['Safety'] }, groups: { lab: [{ id: 'g1', members: ['A', 'B'], label: 'Pair', collapsed: true }] }, locked: ['C'] });
  assert.equal(store.isLocked('lab', 'A', false), true, 'on a locked layer');
  assert.equal(store.isLocked('lab', 'C', false), true, 'locked by itself');
  assert.equal(store.isLocked('lab', 'B', false), false);
  assert.deepEqual(storeOf({}).toFile(), {});
});

test('a member of a group stands for it until the group is entered', () => {
  const store = storeOf({ groups: { lab: [{ id: 'g2', members: ['group:g1', 'C'] }, { id: 'g1', members: ['A', 'B'] }] } });
  assert.equal(layers.redirect(store, 'lab', 'A'), 'group:g2');
  store.enter('lab', 'group:g2');
  assert.equal(layers.redirect(store, 'lab', 'A'), 'group:g1');
  assert.equal(layers.redirect(store, 'lab', 'C'), 'C');
  store.enter('lab', 'group:g1');
  assert.equal(layers.redirect(store, 'lab', 'A'), 'A');
});

test('the scope writes the sections and several stackings at once', () => {
  const store = storeOf({});
  const scope = scopes.create({ graphs: [graph()], collapse, storage: null, storageKey: 'k', initial: { arclang_layout: '1', locked: ['B'] }, sections: store });
  assert.equal(store.lockedOne('B'), true, 'loaded through the scope');
  scope.setStacking('lab', { A: 3, B: 1, C: 2, nope: 4 });
  const file = scope.toFile();
  assert.deepEqual(file.arrangements[0].order, { A: 3, B: 1, C: 2 });
  assert.deepEqual(file.locked, ['B']);
  scope.setStacking('lab', { A: 0, B: 0, C: 0 });
  assert.equal(scope.toFile().arrangements.length, 0);
});

test('the commands: restack, group, ungroup, lock, each one change', async () => {
  const store = storeOf({});
  const scope = scopes.create({ graphs: [graph()], collapse, storage: null, storageKey: 'k', initial: null, sections: store });
  let changes = 0;
  const drawnNow = () => ({ graph: store.apply(graph(), 'lab') });
  const api = layers.attach({ store, scope, replace: async (change) => { change(); changes += 1; }, view: () => 'lab', shown: async () => drawnNow(), announce: () => {} });
  assert.equal((await api.sendToBack(['C'])).ok, true);
  assert.deepEqual(scope.toFile().arrangements[0].order, { C: 1, A: 2, B: 3 });
  assert.equal((await api.bringForward(['A'])).ok, false, 'A overlaps nothing above it');
  const grouped = await api.group(['A', 'B']);
  assert.equal(grouped.ok, true);
  assert.deepEqual(store.toFile().groups.lab[0].members, ['A', 'B']);
  assert.equal((await api.group(['A1', 'C'])).ok, false, 'not siblings');
  assert.equal((await api.group(['A'])).reason, 'Select two or more boxes to group them.');
  await api.lock(['C'], true);
  assert.equal(api.isLocked('C', false), true);
  assert.equal((await api.ungroup(grouped.id)).ok, true);
  assert.equal(store.toFile().groups, undefined);
  assert.equal(changes, 4);
});
