// Unit tests of routing around boxes and of draw.io's arrangement gestures
// as the placement applies them (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const ELK = require(join(viewer, 'vendor/elk.bundled.js'));
const { route, crosses } = require(join(viewer, 'arcviz-route.js'));
const { place } = require(join(viewer, 'arcviz-place.js'));
const { renderSvg } = require(join(viewer, 'arcviz-render.js'));
const collapse = require(join(viewer, 'arcviz-collapse.js'));
const scopes = require(join(viewer, 'arcviz-scope.js'));

const rectilinear = (points) => points.every((p, i) => i === 0 || p.x === points[i - 1].x || p.y === points[i - 1].y);
const clear = (points, rects) => rects.every((r) => !crosses(points, r));

test('a route goes around a box in its way, at right angles, square to both ends', () => {
  const wall = { x: 100, y: -50, width: 60, height: 200 };
  const points = route({ from: { x: 0, y: 50, side: 'east' }, to: { x: 300, y: 50, side: 'west' }, obstacles: [wall] });
  assert.ok(points, 'a route is found');
  assert.ok(rectilinear(points), JSON.stringify(points));
  assert.ok(clear(points, [wall]), `it does not cross the box: ${JSON.stringify(points)}`);
  assert.deepEqual(points[0], { x: 0, y: 50 });
  assert.deepEqual(points.at(-1), { x: 300, y: 50 });
  assert.equal(points[1].y, 50, 'it leaves eastwards');
  assert.equal(points.at(-2).y, 50, 'it comes in from the west');
});

test('with nothing in the way a route runs straight', () => {
  const points = route({ from: { x: 0, y: 50, side: 'east' }, to: { x: 300, y: 50, side: 'west' }, obstacles: [{ x: 100, y: 200, width: 40, height: 40 }] });
  assert.deepEqual(points, [{ x: 0, y: 50 }, { x: 300, y: 50 }]);
});

test('a route passes through the points the reader gave it, in order', () => {
  const via = [{ x: 150, y: -80 }];
  const points = route({ from: { x: 0, y: 50, side: 'east' }, to: { x: 300, y: 50, side: 'west' }, via, obstacles: [] });
  assert.ok(points.some((p) => p.x === 150 && p.y === -80), JSON.stringify(points));
  assert.ok(rectilinear(points));
});

test('routing is the same for the same input, and fast enough for a hundred boxes', () => {
  const boxes = [];
  for (let i = 0; i < 10; i += 1) for (let j = 0; j < 10; j += 1) boxes.push({ x: i * 120, y: j * 90, width: 80, height: 50 });
  const spec = { from: { x: -40, y: 25, side: 'west' }, to: { x: 1200, y: 865, side: 'east' }, obstacles: boxes };
  const started = performance.now();
  const one = route(spec);
  const elapsed = performance.now() - started;
  assert.ok(one && rectilinear(one) && clear(one, boxes), JSON.stringify(one));
  assert.deepEqual(route(spec), one);
  assert.ok(elapsed < 1500, `${Math.round(elapsed)} ms`);
});

// --- the placement, with routing around boxes (an editor's) ---------------

const leaf = (id, ports = [], size = [120, 44]) => ({ id, width: size[0], height: size[1], labels: [{ text: id }], ports: ports.map((port) => ({ id: port, width: 8, height: 8, arc: { name: port } })), arc: { kind: 'logical_component', realizes: [], properties: {} }, layoutOptions: { 'elk.portConstraints': 'FREE' } });
const edge = (id, source, target, sourceNode, targetNode) => ({ id, sources: [source], targets: [target], labels: [{ text: id, width: 30, height: 12 }], arc: { kind: 'component_exchange', source_node: sourceNode, target_node: targetNode, properties: {} } });
// Camera -> Controller -> Actuator, Camera over the actuator: reference.arc's logical view in small.
const laidOut = () => new ELK().layout({
  id: 'lab',
  layoutOptions: { 'elk.algorithm': 'layered', 'elk.direction': 'RIGHT', 'elk.edgeRouting': 'ORTHOGONAL' },
  children: [leaf('CAM', ['CAM::lane']), leaf('CTL', ['CTL::lane', 'CTL::steer']), leaf('ACT', ['ACT::steer']), leaf('HMI', [])],
  edges: [edge('lane', 'CAM::lane', 'CTL::lane', 'CAM', 'CTL'), edge('steer', 'CTL::steer', 'ACT::steer', 'CTL', 'ACT'), edge('alert', 'CTL', 'HMI', 'CTL', 'HMI')],
  arc: { view: 'lab', title: 'Logical', chains: [] },
});
function absolute(graph) {
  const rects = new Map();
  const walk = (node, ox, oy) => {
    rects.set(node.id, { x: ox + node.x, y: oy + node.y, width: node.width, height: node.height, ports: node.ports });
    for (const child of node.children || []) walk(child, ox + node.x, oy + node.y);
  };
  for (const child of graph.children) walk(child, 0, 0);
  const edges = new Map((graph.edges || []).map((e) => [e.id, e.sections.flatMap((s) => [s.startPoint, ...(s.bendPoints || []), s.endPoint])]));
  return { rects, edges };
}

test('after a move, an exchange goes around the boxes now in its way', async () => {
  const graph = await laidOut();
  const was = absolute(graph);
  // Put the actuator right between the camera and the controller.
  const cam = was.rects.get('CAM');
  const ctl = was.rects.get('CTL');
  const act = was.rects.get('ACT');
  // The controller well to the right first, so the actuator fits between them.
  const gap = 400;
  const ctlDx = cam.x + cam.width + gap - ctl.x;
  const target = { x: cam.x + cam.width + gap / 2 - act.width / 2, y: cam.y + cam.height / 2 - act.height / 2 };
  const offsets = { CTL: { dx: ctlDx, dy: 0 }, ACT: { dx: target.x - act.x, dy: target.y - act.y } };
  const plain = absolute(place(graph, offsets, {}));
  assert.ok(crosses(plain.edges.get('lane'), plain.rects.get('ACT')), 'without routing the line is left across the box');
  const avoided = absolute(place(graph, offsets, {}, { avoid: true }));
  const lane = avoided.edges.get('lane');
  assert.ok(rectilinear(lane), JSON.stringify(lane));
  assert.ok(!crosses(lane, avoided.rects.get('ACT')), `it goes around: ${JSON.stringify(lane)}`);
  for (const id of ['steer', 'alert']) assert.ok(!crosses(avoided.edges.get(id), avoided.rects.get('CAM')), id);
});

test('box-to-box lines float: their ends sit level where the boxes face each other', async () => {
  const graph = await laidOut();
  const was = absolute(graph);
  const ctl = was.rects.get('CTL');
  const hmi = was.rects.get('HMI');
  // HMI straight to the right of the controller, a little lower.
  const placed = absolute(place(graph, { HMI: { dx: ctl.x + ctl.width + 140 - hmi.x, dy: ctl.y + 12 - hmi.y } }, {}, { avoid: true }));
  const alert = placed.edges.get('alert');
  assert.equal(alert.length, 2, `one straight run: ${JSON.stringify(alert)}`);
  assert.equal(alert[0].x, ctl.x + ctl.width);
  assert.equal(alert[0].y, alert[1].y);
});

test('a port dragged along its box stays on the border, follows a resize, and its exchange follows it', async () => {
  const graph = await laidOut();
  const was = absolute(graph);
  const ctl = was.rects.get('CTL');
  const port = ctl.ports.find((p) => p.id === 'CTL::steer');
  // Down along its side, then far off the box: it is put back on the nearest side.
  const placed = place(graph, {}, {}, { ports: { 'CTL::steer': { dx: -60, dy: 500 } } });
  const now = absolute(placed);
  const moved = now.rects.get('CTL').ports.find((p) => p.id === 'CTL::steer');
  assert.ok(moved.y + moved.height >= ctl.height - 0.5 && moved.x > 0 && moved.x + moved.width < ctl.width, `on the bottom side: ${JSON.stringify(moved)} (was ${JSON.stringify(port)})`);
  const steer = now.edges.get('steer');
  const centre = { x: now.rects.get('CTL').x + moved.x + moved.width / 2, y: now.rects.get('CTL').y + moved.y + moved.height / 2 };
  assert.ok(Math.abs(steer[0].x - centre.x) < 1 && Math.abs(steer[0].y - (now.rects.get('CTL').y + ctl.height)) < 1, `the exchange leaves from the moved port: ${JSON.stringify(steer[0])} ${JSON.stringify(centre)}`);
  const bigger = absolute(place(graph, { CTL: { dx: 0, dy: 0, dh: 60 } }, {}, { ports: { 'CTL::steer': { dx: -60, dy: 500 } } }));
  const after = bigger.rects.get('CTL');
  const seated = after.ports.find((p) => p.id === 'CTL::steer');
  assert.ok(seated.y + seated.height >= after.height - 0.5, 'still on the bottom once the box is taller');
  // Slid down its side by 10, then the box made taller: it stays 10 lower than it was, as dragged.
  const slid = absolute(place(graph, { CTL: { dx: 0, dy: 0, dh: 60 } }, {}, { ports: { 'CTL::steer': { dx: 0, dy: 10 } } })).rects.get('CTL').ports.find((p) => p.id === 'CTL::steer');
  assert.equal(slid.y, port.y + 10, 'a resize keeps a slid port where it was put');
  assert.equal(slid.x, port.x, 'on the same side');
});

test('a dragged name keeps its offset; a styled line is drawn straight or curved', async () => {
  const graph = await laidOut();
  const label = (g) => g.edges.find((e) => e.id === 'lane').labels[0];
  const shifted = place(graph, {}, {}, { labels: { lane: { dx: 30, dy: -20 } } });
  assert.equal(label(shifted).x, label(graph).x + 30);
  assert.equal(label(shifted).y, label(graph).y - 20);
  const straight = place(graph, {}, {}, { styles: { steer: 'straight' } });
  const run = absolute(straight).edges.get('steer');
  assert.equal(run.length, 2, `end to end: ${JSON.stringify(run)}`);
  const curved = place(graph, {}, {}, { styles: { steer: 'curved' } });
  const { svg } = renderSvg(curved);
  assert.match(svg, /data-id="steer" data-style="curved"/);
  assert.match(svg.split('data-id="steer"')[1], /d="M[^"]*Q/);
  assert.doesNotMatch(renderSvg(place(graph, {}, {}, { styles: { steer: 'zigzag' } })).svg, /data-style/);
});

test('a box brought to the front is drawn after the others, one sent to the back before', async () => {
  const graph = await laidOut();
  const ids = (g) => g.children.map((n) => n.id);
  assert.equal(ids(place(graph, {}, {}, { order: { CAM: 1 } })).at(-1), 'CAM');
  assert.equal(ids(place(graph, {}, {}, { order: { HMI: -1 } }))[0], 'HMI');
  assert.equal(place(graph, {}, {}, { order: {} }), graph, 'nothing restacked, nothing changed');
});

// --- the layout file keeps all of it ----------------------------------------

test('styles, names, ports and stacking are written to the layout file and read back', async () => {
  const graphs = [await laidOut()];
  const memory = () => ({ data: {}, getItem() { return null; }, setItem() {} });
  const scope = scopes.create({ graphs, collapse, storage: memory(), storageKey: 'k' });
  scope.setStyle('lab', 'steer', 'curved');
  scope.moveLabelBy('lab', 'lane', 10, 5);
  scope.movePortBy('lab', 'CTL::steer', 0, 20);
  scope.restack('lab', 'CAM', true);
  scope.restack('lab', 'HMI', false);
  const file = scope.toFile();
  const entry = file.arrangements[0];
  assert.deepEqual(entry.edges, { lane: { label: { dx: 10, dy: 5 } }, steer: { style: 'curved' } });
  assert.deepEqual(entry.ports, { 'CTL::steer': { dx: 0, dy: 20 } });
  assert.deepEqual(entry.order, { CAM: 1, HMI: -1 });
  const again = scopes.create({ graphs, collapse, storage: memory(), storageKey: 'k', initial: JSON.parse(JSON.stringify(file)) });
  assert.deepEqual(again.toFile(), file);
  assert.equal(again.styleOf('lab', 'steer'), 'curved');
  assert.deepEqual(again.extrasOf('lab').ports, { 'CTL::steer': { dx: 0, dy: 20 } });
  // Unknown styles, stale ids and absurd values are left out.
  const odd = scopes.create({ graphs, collapse, storage: memory(), storageKey: 'k', initial: { arclang_layout: '1', views: {}, arrangements: [{ view: 'lab', open: null, folded: [], places: {}, edges: { steer: { style: 'zigzag' }, gone: { style: 'curved' } }, ports: { 'CTL::nope': { dx: 1, dy: 1 }, 'CTL::steer': { dx: 'far', dy: 0 } }, order: { CAM: 1.5, Gone: 1 } }] } });
  assert.equal(odd.arranged('lab'), false);
  scope.resetPlaces('lab');
  assert.equal(scope.arranged('lab'), false, 'reset layout forgets all of it');
  assert.equal(scope.styleOf('lab', 'steer'), 'orthogonal');
});
