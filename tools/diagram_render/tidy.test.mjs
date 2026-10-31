// Unit tests of tidying a drawing up (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const ELK = require(join(viewer, 'vendor/elk.bundled.js'));
const { place } = require(join(viewer, 'arcviz-place.js'));
const { measure, worse, describe, seeded, placements } = require(join(viewer, 'arcviz-tidy.js'));

const leaf = (id) => ({ id, width: 120, height: 44, labels: [{ text: id }], ports: [], arc: { kind: 'logical_component', realizes: [], properties: {} }, layoutOptions: {} });
const box = (id, children) => ({ id, labels: [{ text: id }], ports: [], children, arc: { kind: 'logical_component', realizes: [], properties: {} }, layoutOptions: { 'elk.padding': '[top=34,left=18,bottom=20,right=18]' } });
const edge = (id, source, target) => ({ id, sources: [source], targets: [target], labels: [], arc: { kind: 'component_exchange', source_node: source, target_node: target, properties: {} } });
const shape = () => ({
  id: 'lab',
  layoutOptions: { 'elk.algorithm': 'layered', 'elk.direction': 'RIGHT', 'elk.hierarchyHandling': 'INCLUDE_CHILDREN', 'elk.edgeRouting': 'ORTHOGONAL' },
  children: [box('A', [leaf('A1'), leaf('A2')]), leaf('C'), leaf('D'), leaf('E')],
  edges: [edge('a', 'A1', 'A2'), edge('b', 'A2', 'D'), edge('c', 'C', 'D'), edge('d', 'C', 'E')],
  arc: { view: 'lab', title: 'Logical', chains: [] },
});
const absolute = (graph) => {
  const out = {};
  const walk = (holder, bx, by) => {
    for (const n of holder.children || []) {
      out[n.id] = { x: bx + n.x, y: by + n.y, width: n.width, height: n.height };
      walk(n, bx + n.x, by + n.y);
    }
  };
  walk(graph, 0, 0);
  return out;
};

test('the automatic layout is orderly', async () => {
  const auto = await new ELK().layout(shape());
  assert.deepEqual(measure(auto), { overlaps: 0, through: 0, crossings: 0 });
});

test('a box dragged onto another is counted, and the change is worse', async () => {
  const auto = await new ELK().layout(shape());
  const at = absolute(auto);
  const onto = { E: { dx: at.D.x - at.E.x + 20, dy: at.D.y - at.E.y + 10 } };
  const moved = place(auto, onto, {}, { avoid: true });
  const after = measure(moved);
  assert.equal(after.overlaps, 1, JSON.stringify(after));
  assert.equal(worse(measure(auto), after), true);
  assert.equal(worse(after, after), false, 'no change, nothing worse');
  assert.match(describe(after), /^1 box overlaps another/);
});

test('a line running across a box is counted; a line inside its container is not', async () => {
  const graph = { id: 'g', children: [
    { id: 'P', x: 0, y: 0, width: 300, height: 200, children: [{ id: 'P1', x: 20, y: 40, width: 60, height: 30 }, { id: 'P2', x: 200, y: 40, width: 60, height: 30 }] },
    { id: 'Q', x: 400, y: 0, width: 60, height: 30 }, { id: 'R', x: 600, y: 0, width: 60, height: 30 }, { id: 'S', x: 500, y: -20, width: 40, height: 80 }],
    edges: [
      { id: 'in', sources: ['P1'], targets: ['P2'], container: 'P', sections: [{ startPoint: { x: 80, y: 55 }, endPoint: { x: 200, y: 55 } }] },
      { id: 'over', sources: ['Q'], targets: ['R'], sections: [{ startPoint: { x: 460, y: 15 }, endPoint: { x: 600, y: 15 } }] },
    ] };
  assert.deepEqual(measure(graph), { overlaps: 0, through: 1, crossings: 0 });
});

test('two lines that cross are counted once; lines meeting at an end are not', () => {
  const graph = { id: 'g', children: [], edges: [
    { id: 'h', sources: ['x'], targets: ['y'], sections: [{ startPoint: { x: 0, y: 50 }, endPoint: { x: 100, y: 50 } }] },
    { id: 'v', sources: ['u'], targets: ['w'], sections: [{ startPoint: { x: 50, y: 0 }, endPoint: { x: 50, y: 100 } }] },
    { id: 't', sources: ['y'], targets: ['z'], sections: [{ startPoint: { x: 100, y: 50 }, endPoint: { x: 100, y: 120 } }] },
  ] };
  assert.equal(measure(graph).crossings, 1);
});

test('tidying up keeps the order the reader chose and removes the overlap', async () => {
  const auto = await new ELK().layout(shape());
  const at = absolute(auto);
  // E dragged far to the left of everything, D dragged onto C.
  const hand = { E: { dx: -at.E.x - 400, dy: 0 }, D: { dx: at.C.x - at.D.x + 30, dy: at.C.y - at.D.y + 10 } };
  const messy = place(auto, hand, {}, { avoid: true });
  assert.ok(measure(messy).overlaps > 0);
  const tidy = await new ELK().layout(seeded(shape(), messy));
  const moves = placements(auto, tidy);
  const again = place(auto, moves, {}, { avoid: true });
  const now = absolute(again);
  assert.equal(measure(again).overlaps, 0, describe(measure(again)));
  assert.ok(now.E.x < now.C.x && now.E.x < now.A.x, `E stays leftmost: ${JSON.stringify(now)}`);
  // What the layout file keeps reproduces the tidy layout.
  const tidyAt = absolute(tidy);
  for (const id of ['A', 'C', 'D', 'E', 'A1', 'A2']) {
    assert.ok(Math.abs(now[id].x - tidyAt[id].x) < 1.5 && Math.abs(now[id].y - tidyAt[id].y) < 1.5, `${id}: ${JSON.stringify(now[id])} vs ${JSON.stringify(tidyAt[id])}`);
  }
});

test('a resized box keeps its size through a tidy', async () => {
  const auto = await new ELK().layout(shape());
  const messy = place(auto, { C: { dx: 0, dy: 0, dw: 80, dh: 20 } }, {}, {});
  const tidy = await new ELK().layout(seeded(shape(), messy));
  const moves = placements(auto, tidy);
  assert.equal(moves.C.dw, 80);
  assert.equal(moves.C.dh, 20);
});
