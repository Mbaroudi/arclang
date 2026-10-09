// Unit tests of manual placement (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const ELK = require(join(viewer, 'vendor/elk.bundled.js'));
const { place } = require(join(viewer, 'arcviz-place.js'));
const { renderSvg } = require(join(viewer, 'arcviz-render.js'));

const leaf = (id, ports = []) => ({ id, width: 120, height: 44, labels: [{ text: id }], ports: ports.map((port) => ({ id: port, width: 8, height: 8, arc: { name: port } })), arc: { kind: 'function', realizes: [], properties: {} }, layoutOptions: {} });
const box = (id, children) => ({ id, labels: [{ text: id }], ports: [], children, arc: { kind: 'logical_component', realizes: [], properties: {} }, layoutOptions: { 'elk.padding': '[top=34,left=18,bottom=20,right=18]' } });
const edge = (id, source, target, sourceNode, targetNode) => ({ id, sources: [source], targets: [target], labels: [{ text: id, width: 30, height: 12 }], arc: { kind: 'functional_exchange', source_node: sourceNode, target_node: targetNode, properties: {} } });
const laidOut = () => new ELK().layout({
  id: 'lab',
  layoutOptions: { 'elk.algorithm': 'layered', 'elk.direction': 'RIGHT', 'elk.hierarchyHandling': 'INCLUDE_CHILDREN', 'elk.edgeRouting': 'ORTHOGONAL' },
  children: [box('A', [leaf('A1', ['A1::out']), leaf('A2')]), box('B', [leaf('B1', ['B1::in'])]), leaf('C'), leaf('D')],
  edges: [
    edge('inner', 'A1::out', 'A2', 'A1', 'A2'),
    edge('across', 'A1::out', 'B1::in', 'A1', 'B1'),
    edge('plain', 'C', 'D', 'C', 'D'),
    edge('far', 'A2', 'D', 'A2', 'D'),
  ],
  arc: { view: 'lab', title: 'Logical', chains: [] },
});

// Absolute geometry of a placed graph.
function absolute(graph) {
  const rects = new Map();
  const walk = (node, ox, oy) => {
    const rect = { x: ox + node.x, y: oy + node.y, width: node.width, height: node.height };
    rects.set(node.id, rect);
    for (const child of node.children || []) walk(child, rect.x, rect.y);
  };
  for (const child of graph.children) walk(child, 0, 0);
  const edges = new Map();
  const collect = (holder) => {
    for (const e of holder.edges || []) {
      const origin = e.container && e.container !== graph.id ? rects.get(e.container) : { x: 0, y: 0 };
      edges.set(e.id, e.sections.flatMap((s) => [s.startPoint, ...(s.bendPoints || []), s.endPoint]).map((p) => ({ x: p.x + origin.x, y: p.y + origin.y })));
    }
    for (const child of holder.children || []) collect(child);
  };
  collect(graph);
  return { rects, edges };
}
const onBorder = (point, rect) => {
  const near = (a, b) => Math.abs(a - b) < 9; // a port sticks out by its size
  const insideX = point.x > rect.x - 9 && point.x < rect.x + rect.width + 9;
  const insideY = point.y > rect.y - 9 && point.y < rect.y + rect.height + 9;
  return (insideY && (near(point.x, rect.x) || near(point.x, rect.x + rect.width))) || (insideX && (near(point.y, rect.y) || near(point.y, rect.y + rect.height)));
};
const rectilinear = (points) => points.every((p, i) => i === 0 || p.x === points[i - 1].x || p.y === points[i - 1].y);

test('placing nothing returns the layout itself, and the layout is never modified', async () => {
  const graph = await laidOut();
  const before = JSON.stringify(graph);
  assert.equal(place(graph, {}), graph);
  place(graph, { C: { dx: 40, dy: 90 } });
  assert.equal(JSON.stringify(graph), before);
});

test('a moved box goes exactly where it was dropped, the others stay', async () => {
  const graph = await laidOut();
  const was = absolute(graph);
  const now = absolute(place(graph, { D: { dx: 60, dy: 150 } }));
  assert.deepEqual([now.rects.get('D').x, now.rects.get('D').y], [was.rects.get('D').x + 60, was.rects.get('D').y + 150]);
  for (const id of ['A', 'A1', 'B', 'B1', 'C']) assert.deepEqual(now.rects.get(id), was.rects.get(id), id);
});

test('exchanges of a moved box are redrawn onto it, at right angles; the others keep their route', async () => {
  const graph = await laidOut();
  const was = absolute(graph);
  const now = absolute(place(graph, { D: { dx: 60, dy: 150 } }));
  for (const id of ['plain', 'far']) {
    const points = now.edges.get(id);
    assert.ok(rectilinear(points), `${id} is drawn at right angles: ${JSON.stringify(points)}`);
    assert.ok(onBorder(points[points.length - 1], now.rects.get('D')), `${id} ends on D`);
  }
  assert.ok(onBorder(now.edges.get('plain')[0], now.rects.get('C')), 'plain still starts on C');
  for (const id of ['inner', 'across']) assert.deepEqual(now.edges.get(id), was.edges.get(id), id);
});

test('moving a container carries what it holds and the exchanges inside it', async () => {
  const graph = await laidOut();
  const was = absolute(graph);
  const now = absolute(place(graph, { A: { dx: 0, dy: 200 } }));
  assert.equal(now.rects.get('A1').y, was.rects.get('A1').y + 200);
  assert.deepEqual(now.edges.get('inner'), was.edges.get('inner').map((p) => ({ x: p.x, y: p.y + 200 })));
  const across = now.edges.get('across');
  assert.ok(onBorder(across[0], now.rects.get('A1')), 'across starts on the moved function');
  assert.ok(onBorder(across[across.length - 1], now.rects.get('B1')));
});

test('a box inside a container cannot be dragged out of it', async () => {
  const graph = await laidOut();
  const now = absolute(place(graph, { A1: { dx: 5000, dy: -5000 } }));
  const [inner, outer] = [now.rects.get('A1'), now.rects.get('A')];
  assert.ok(inner.x + inner.width <= outer.x + outer.width && inner.y >= outer.y, JSON.stringify([inner, outer]));
});

test('the sheet grows with what is placed, and nothing leaves it on the top or left', async () => {
  const graph = await laidOut();
  const moved = place(graph, { D: { dx: 900, dy: 700 }, C: { dx: -900, dy: -700 } });
  const now = absolute(moved);
  for (const [id, rect] of now.rects) {
    assert.ok(rect.x >= 0 && rect.y >= 0, `${id} is on the sheet`);
    assert.ok(rect.x + rect.width <= moved.width && rect.y + rect.height <= moved.height, `${id} fits`);
  }
  for (const [id, points] of now.edges) for (const p of points) assert.ok(p.x >= 0 && p.y >= 0 && p.x <= moved.width && p.y <= moved.height, `${id} fits`);
});

test('an offset for an element the view does not draw is ignored', async () => {
  const graph = await laidOut();
  assert.deepEqual(absolute(place(graph, { Gone: { dx: 10, dy: 10 } })).rects, absolute(graph).rects);
});

test('a placed layout draws every element and exchange', async () => {
  const { svg } = renderSvg(place(await laidOut(), { D: { dx: 60, dy: 150 }, A: { dx: 0, dy: 120 } }));
  for (const id of ['A', 'A1', 'A2', 'B', 'B1', 'C', 'D']) assert.match(svg, new RegExp(`data-id="${id}"`));
  for (const id of ['inner', 'across', 'plain', 'far']) assert.match(svg, new RegExp(`class="av-edge" data-id="${id}"`));
  assert.doesNotMatch(svg, /NaN/);
});

test('a port flush inside the border keeps its side when its box moves', async () => {
  const graph = await laidOut();
  const a1 = graph.children[0].children.find((node) => node.id === 'A1');
  a1.ports[0].x = a1.width - a1.ports[0].width; // east port drawn inside the box
  const now = absolute(place(graph, { A: { dx: 0, dy: 200 } }));
  const start = now.edges.get('across')[0];
  assert.equal(start.x, now.rects.get('A1').x + now.rects.get('A1').width, 'it leaves by the east side');
});

test('a loop on a moved box moves with it', async () => {
  const layout = await new ELK().layout({
    id: 'msm',
    layoutOptions: { 'elk.algorithm': 'layered', 'elk.direction': 'RIGHT', 'elk.edgeRouting': 'ORTHOGONAL' },
    children: [leaf('S'), leaf('T')],
    edges: [edge('loop', 'S', 'S', 'S', 'S'), edge('next', 'S', 'T', 'S', 'T')],
    arc: { view: 'msm', title: 'Modes', chains: [] },
  });
  const was = absolute(layout).edges.get('loop');
  const now = absolute(place(layout, { S: { dx: 0, dy: 90 } })).edges.get('loop');
  assert.ok(was.length > 1, 'the loop has a route');
  assert.deepEqual(now, was.map((p) => ({ x: p.x, y: p.y + 90 })));
});

test('a placed layout says where each box really went', async () => {
  const graph = await laidOut();
  const placed = place(graph, { D: { dx: 60, dy: 150 }, A1: { dx: 5000, dy: 0 } }).arc.placed;
  assert.deepEqual(placed.D, { dx: 60, dy: 150 });
  assert.ok(placed.A1.dx < 200 && placed.A1.dx >= 0, `held back by its container: ${JSON.stringify(placed.A1)}`);
});

test('an absurd offset is ignored rather than drawn', async () => {
  const graph = await laidOut();
  for (const offset of [{ dx: 1e308, dy: 0 }, { dx: NaN, dy: 3 }, { dx: 'far', dy: 0 }, null]) {
    assert.equal(place(graph, { D: offset }), graph, JSON.stringify(offset));
  }
});
