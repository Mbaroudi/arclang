// Unit tests of the SVG renderer (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const ELK = require(join(viewer, 'vendor/elk.bundled.js'));
const { renderSvg } = require(join(viewer, 'arcviz-render.js'));

const node = (id, extra = {}) => ({
  id,
  width: 120,
  height: 44,
  labels: [{ text: id }],
  ports: [],
  arc: { kind: 'logical_component', realizes: [], properties: {} },
  layoutOptions: {},
  ...extra,
});
const edge = (id, source, target) => ({
  id,
  sources: [source],
  targets: [target],
  labels: [{ text: id, width: 20, height: 12 }],
  arc: { kind: 'component_exchange', source_node: source, target_node: target, properties: {} },
});
const graph = (id, children, edges) => ({
  id,
  layoutOptions: { 'elk.algorithm': 'layered', 'elk.direction': 'RIGHT', 'elk.hierarchyHandling': 'INCLUDE_CHILDREN' },
  children,
  edges,
  arc: { view: id, title: id, chains: [] },
});
const pathOf = (svg, id) => {
  const group = svg.slice(svg.indexOf(`class="av-edge" data-id="${id}"`));
  return /class="av-line" d="([^"]+)"/.exec(group)[1];
};

test('a node named like the view does not shift root-level edges', async () => {
  const laidOut = async (firstId) =>
    new ELK().layout(graph('lab', [node(firstId), node('A'), node('B')], [edge('e', 'A', 'B')]));
  const colliding = renderSvg(await laidOut('lab')).svg;
  const plain = renderSvg(await laidOut('other')).svg;
  assert.equal(pathOf(colliding, 'e'), pathOf(plain, 'e'));
});

test('an edge inside a container is drawn where its ports are', async () => {
  const inner = [node('A'), node('B')];
  const box = node('Box', { children: inner, layoutOptions: { 'elk.padding': '[top=34,left=18,bottom=20,right=18]' } });
  delete box.width;
  delete box.height;
  const laidOut = await new ELK().layout(graph('sab', [box], [edge('e', 'A', 'B')]));
  const a = laidOut.children[0].children.find((child) => child.id === 'A');
  const startX = laidOut.children[0].x + a.x + a.width;
  const startY = laidOut.children[0].y + a.y + a.height / 2;
  const [, x, y] = /^M([\d.]+) ([\d.]+)/.exec(pathOf(renderSvg(laidOut).svg, 'e'));
  assert.ok(Math.abs(Number(x) - startX) < 1 && Math.abs(Number(y) - startY) < 1, `edge starts at ${x},${y}, port side at ${startX},${startY}`);
});

test('a self-loop is routed', async () => {
  const laidOut = await new ELK().layout(graph('lab', [node('A')], [edge('loop', 'A', 'A')]));
  assert.ok(laidOut.edges[0].sections.length > 0);
  assert.match(renderSvg(laidOut).svg, /data-id="loop"/);
});

test('model text is escaped everywhere it is drawn', async () => {
  const hostile = '"><script>alert(1)</script><img src=x onerror=alert(1)>';
  const evil = node(hostile, {
    labels: [{ text: hostile }],
    ports: [{ id: `${hostile}::p`, width: 12, height: 12, arc: { name: hostile, direction: 'out', interface: hostile, protocol: hostile } }],
    arc: { kind: 'logical_component', realizes: [hostile], properties: { description: hostile, safety_level: hostile } },
  });
  const link = edge(hostile + 'e', hostile, 'B');
  link.labels[0].text = hostile;
  link.arc.exchange_item = hostile;
  const { svg } = renderSvg(await new ELK().layout(graph('lab', [evil, node('B')], [link])));
  assert.ok(!svg.includes('<script'), 'no script element');
  assert.ok(!svg.includes('<img'), 'no injected element');
  assert.ok(!svg.includes(hostile), 'the raw string never appears');
  assert.ok(svg.includes('&quot;&gt;&lt;script&gt;'), 'it appears escaped');
});

const sequence = (messages, lifelines = ['A', 'B']) => ({
  id: 'es:S',
  children: lifelines.map((id) => ({ id, width: 112, height: 42, labels: [{ text: id }], ports: [], arc: { kind: 'lifeline', realizes: [], properties: {} } })),
  edges: messages.map(([from, to, text, properties = { type: 'sync' }], index) => ({
    id: `msg:${index + 1}`,
    sources: [from],
    targets: [to],
    labels: [{ text }],
    arc: { kind: 'message', source_node: from, target_node: to, properties },
  })),
  arc: { view: 'es', layout: 'sequence', title: 'S', chains: [] },
});
const rowOf = (svg, id) => Number(/class="av-line" d="M[\d.]+ ([\d.]+)/.exec(svg.slice(svg.indexOf(`data-id="${id}"`)))[1]);

test('a scenario is drawn without a layout engine, messages top to bottom in order', () => {
  const { svg, width, height } = renderSvg(sequence([['A', 'B', 'first'], ['B', 'A', 'second', { type: 'async', timing: '5 ms' }], ['B', 'B', 'third']]));
  assert.ok(width > 0 && height > 0);
  assert.ok(rowOf(svg, 'msg:1') < rowOf(svg, 'msg:2') && rowOf(svg, 'msg:2') < rowOf(svg, 'msg:3'));
  assert.match(svg, />1: first</);
  assert.match(svg, />2: second \{5 ms\}</);
  assert.match(svg, /data-id="A"[^>]*data-kind="lifeline"/);
});

test('a long message label widens the columns instead of overlapping the next lifeline', () => {
  const short = renderSvg(sequence([['A', 'B', 'x']])).width;
  const long = renderSvg(sequence([['A', 'B', 'a very long message label that needs room to be read in full']])).width;
  assert.ok(long > short);
});

test('scenario text is escaped', () => {
  const hostile = '"><script>alert(1)</script>';
  const { svg } = renderSvg(sequence([[hostile, 'B', hostile, { type: 'sync', timing: hostile }]], [hostile, 'B']));
  assert.ok(!svg.includes('<script') && !svg.includes(hostile));
});

test('a box listing lines draws each line under its name', async () => {
  const klass = node('Frame', { height: 34 + 2 * 14 + 10 });
  klass.arc = { kind: 'class', realizes: [], properties: {}, compartment: ['range: float', '<b>level</b>: Level'] };
  const { svg } = renderSvg(await new ELK().layout(graph('cdb', [klass], [])));
  assert.match(svg, />range: float</);
  assert.ok(svg.includes('&lt;b&gt;level&lt;/b&gt;: Level') && !svg.includes('<b>'), 'lines are escaped');
});

test('a generalization ends in a hollow triangle, other links in a filled arrow', async () => {
  const special = { ...edge('g', 'A', 'B'), arc: { kind: 'generalization', source_node: 'A', target_node: 'B', properties: {} } };
  const laidOut = await new ELK().layout(graph('cap', [node('A'), node('B'), node('C')], [special, edge('e', 'B', 'C')]));
  const { svg } = renderSvg(laidOut);
  const marker = (kind) => new RegExp(`<marker id="av-arrow-${kind}"[^>]*>(.*?)</marker>`).exec(svg)[1];
  assert.match(marker('generalization'), /fill="#FFFFFF"/);
  assert.match(marker('generalization'), /stroke="#/);
  assert.doesNotMatch(marker('component_exchange'), /fill="#FFFFFF"/);
});

test('an operational process has its own style, not the fallback', async () => {
  const process = node('P', { arc: { kind: 'operational_process', realizes: [], properties: {} } });
  const laidOut = await new ELK().layout(graph('cap', [process], []));
  const { svg } = renderSvg(laidOut);
  assert.match(svg, /Operational process/);
});
