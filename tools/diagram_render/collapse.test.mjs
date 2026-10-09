// Unit tests of container folding (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const ELK = require(join(viewer, 'vendor/elk.bundled.js'));
const { fold, containersOf, ancestorsOf } = require(join(viewer, 'arcviz-collapse.js'));
const { renderSvg } = require(join(viewer, 'arcviz-render.js'));

const leaf = (id, ports = []) => ({ id, width: 120, height: 44, labels: [{ text: id }], ports: ports.map((port) => ({ id: port, width: 8, height: 8, arc: { name: port } })), arc: { kind: 'function', realizes: [], properties: {} }, layoutOptions: {} });
const box = (id, children, ports = []) => ({ id, labels: [{ text: id }], ports: ports.map((port) => ({ id: port, width: 12, height: 12, arc: { name: port } })), children, arc: { kind: 'logical_component', realizes: [], properties: {} }, layoutOptions: { 'elk.padding': '[top=34,left=18,bottom=20,right=18]' } });
const edge = (id, source, target, sourceNode, targetNode) => ({ id, sources: [source], targets: [target], labels: [{ text: id, width: 20, height: 12 }], arc: { kind: 'functional_exchange', source_node: sourceNode, target_node: targetNode, properties: {} } });
const model = () => ({
  id: 'lab',
  layoutOptions: { 'elk.algorithm': 'layered', 'elk.direction': 'RIGHT', 'elk.hierarchyHandling': 'INCLUDE_CHILDREN' },
  children: [
    box('A', [leaf('A1', ['A1::out']), leaf('A2', ['A2::in']), box('AA', [leaf('AA1')])], ['A::p']),
    box('B', [leaf('B1', ['B1::in'])]),
    leaf('C'),
  ],
  edges: [
    edge('inside', 'A1::out', 'A2::in', 'A1', 'A2'),
    edge('across', 'A1::out', 'B1::in', 'A1', 'B1'),
    edge('deep', 'AA1', 'C', 'AA1', 'C'),
    edge('own', 'A::p', 'C', 'A', 'C'),
  ],
  arc: { view: 'lab', title: 'Logical', chains: [] },
});

test('folding nothing returns the graph unchanged and never touches the input', () => {
  const graph = model();
  const before = JSON.stringify(graph);
  assert.deepEqual(fold(graph, new Set()), graph);
  fold(graph, new Set(['A']));
  assert.equal(JSON.stringify(graph), before);
});

test('a folded container hides what it holds and says how much', () => {
  const folded = fold(model(), new Set(['A']));
  const a = folded.children.find((node) => node.id === 'A');
  assert.equal(a.children, undefined);
  assert.equal(a.arc.folded, 4, 'A1, A2, AA and AA1');
  assert.ok(a.width >= 112 && a.height >= 42, 'a folded box has a size of its own');
  assert.deepEqual(a.ports.map((port) => port.id), ['A::p'], 'its own ports stay');
});

test('exchanges of hidden elements end on the folded container, inner ones disappear', () => {
  const folded = fold(model(), new Set(['A']));
  const ends = Object.fromEntries(folded.edges.map((e) => [e.id, [e.sources[0], e.targets[0], e.arc.source_node, e.arc.target_node]]));
  assert.deepEqual(ends, {
    across: ['A', 'B1::in', 'A', 'B1'],
    deep: ['A', 'C', 'A', 'C'],
    own: ['A::p', 'C', 'A', 'C'],
  });
});

test('folding both ends keeps the exchange between the two folded boxes', () => {
  const folded = fold(model(), new Set(['A', 'B']));
  const across = folded.edges.find((e) => e.id === 'across');
  assert.deepEqual([across.sources[0], across.targets[0]], ['A', 'B']);
});

test('a folded container inside a folded container is simply hidden', () => {
  const folded = fold(model(), new Set(['A', 'AA']));
  assert.equal(folded.children.find((node) => node.id === 'A').arc.folded, 4);
});

test('only containers can fold, and an element knows what must unfold to show it', () => {
  const graph = model();
  assert.deepEqual(containersOf(graph), ['A', 'AA', 'B']);
  assert.deepEqual(ancestorsOf(graph, 'AA1'), ['A', 'AA']);
  assert.deepEqual(ancestorsOf(graph, 'C'), []);
});

test('a folded graph lays out and draws every remaining element and exchange', async () => {
  const laidOut = await new ELK().layout(fold(model(), new Set(['A'])));
  const { svg } = renderSvg(laidOut);
  for (const id of ['A', 'B', 'B1', 'C']) assert.match(svg, new RegExp(`class="av-node[^"]*" data-id="${id}"`));
  assert.doesNotMatch(svg, /data-id="A1"/);
  for (const id of ['across', 'deep', 'own']) assert.match(svg, new RegExp(`class="av-edge" data-id="${id}"`));
  assert.match(svg, /4 inside/, 'the folded box says what it hides');
});

test('an exchange from a hidden element to its container\'s own port is internal and disappears', () => {
  const graph = model();
  graph.edges.push(edge('delegation', 'A1::out', 'A::p', 'A1', 'A'));
  const folded = fold(graph, new Set(['A']));
  assert.equal(folded.edges.find((e) => e.id === 'delegation'), undefined);
  assert.ok(folded.edges.find((e) => e.id === 'own'), 'the port still carries what leaves the container');
});

test('every end of an exchange is moved, not only the first', () => {
  const graph = model();
  graph.edges.push({ ...edge('fan', 'C', 'B1::in', 'C', 'B1'), sources: ['C', 'A1::out'] });
  const folded = fold(graph, new Set(['A']));
  assert.deepEqual(folded.edges.find((e) => e.id === 'fan').sources, ['C', 'A']);
});

test('a folded container keeps the sides its ports sit on', () => {
  const graph = model();
  graph.children[0].layoutOptions = { 'elk.padding': '[top=34]', 'elk.portConstraints': 'FIXED_SIDE', 'elk.spacing.portPort': '8' };
  const a = fold(graph, new Set(['A'])).children[0];
  assert.deepEqual(a.layoutOptions, { 'elk.portConstraints': 'FIXED_SIDE', 'elk.spacing.portPort': '8' });
});

// --- opening a container as a diagram of its own ---------------------------
const { focus } = require(join(viewer, 'arcviz-collapse.js'));

test('a focused container is drawn alone with what it exchanges with', () => {
  const focused = focus(model(), 'A');
  assert.deepEqual(focused.children.map((node) => [node.id, Boolean(node.arc.context)]), [['A', false], ['B', true], ['C', true]]);
  const a = focused.children[0];
  assert.deepEqual(a.children.map((node) => node.id), ['A1', 'A2', 'AA'], 'its content is whole');
});

test('context elements are folded: the diagram is about the focused container', () => {
  const b = focus(model(), 'A').children.find((node) => node.id === 'B');
  assert.equal(b.children, undefined);
  assert.equal(b.arc.folded, 1);
});

test('exchanges inside stay, crossing ones end on the context, foreign ones go', () => {
  const graph = model();
  graph.edges.push(edge('elsewhere', 'B1::in', 'C', 'B1', 'C'));
  const ends = Object.fromEntries(focus(graph, 'A').edges.map((e) => [e.id, [e.sources[0], e.targets[0]]]));
  assert.deepEqual(ends, {
    inside: ['A1::out', 'A2::in'],
    across: ['A1::out', 'B'],
    deep: ['AA1', 'C'],
    own: ['A::p', 'C'],
  });
});

test('a nested container is lifted out of its parent, which becomes context when it exchanges', () => {
  const graph = model();
  graph.edges.push(edge('sibling', 'A1::out', 'AA1', 'A1', 'AA1'));
  const focused = focus(graph, 'AA');
  assert.deepEqual(focused.children.map((node) => node.id), ['AA', 'A1', 'C']);
  assert.deepEqual(focused.edges.map((e) => e.id).sort(), ['deep', 'sibling']);
  assert.equal(focused.children[1].arc.context, true);
});

test('focusing something that is not a container changes nothing', () => {
  const graph = model();
  assert.equal(focus(graph, 'C'), graph);
  assert.equal(focus(graph, 'nowhere'), graph);
  assert.equal(focus(graph, null), graph);
});

test('a focused graph lays out, and context elements are drawn apart', async () => {
  const laidOut = await new ELK().layout(JSON.parse(JSON.stringify(focus(model(), 'A'))));
  const { svg } = renderSvg(laidOut);
  for (const id of ['A', 'A1', 'AA1', 'B', 'C']) assert.match(svg, new RegExp(`data-id="${id}"`));
  assert.match(svg, /class="av-node is-context" data-id="B"/);
  assert.doesNotMatch(svg, /class="av-node is-context" data-id="A"/);
});

test('exchanges with the containers around the open one are named, not silently lost', () => {
  const graph = model();
  graph.edges.push(edge('toParent', 'AA1', 'A::p', 'AA1', 'A'));
  const focused = focus(graph, 'AA');
  assert.equal(focused.edges.find((e) => e.id === 'toParent'), undefined);
  assert.deepEqual(focused.arc.left_out, [{ id: 'toParent', label: 'toParent' }]);
  assert.deepEqual(focus(model(), 'A').arc.left_out, []);
});
