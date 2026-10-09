// Unit tests of cross-view navigation data (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const { index } = require(join(viewer, 'arcviz-links.js'));

const node = (id, name, kind, extra = {}) => ({ id, labels: [{ text: name }], arc: { kind, realizes: [], ...extra.arc }, children: extra.children || [] });
const graph = (id, view, title, children) => ({ id, children, edges: [], arc: { view, title } });

const graphs = [
  graph('oab', 'oab', 'Operational', [node('OE-1', 'Vehicle', 'operational_entity', { children: [node('OA-1', 'Monitor Road', 'operational_activity')] })]),
  graph('sab', 'sab', 'System', [node('SF-1', 'Detect Obstacle', 'function')]),
  graph('lab', 'lab', 'Logical', [node('LC-1', 'Perception', 'logical_component', { children: [node('SF-1', 'Detect Obstacle', 'function')] })]),
  graph('pab', 'pab', 'Physical', [node('PN-1', 'ECU', 'physical_node', { children: [node('PN-1/Perception', 'Perception', 'deployed_component', { arc: { realizes: ['LC-1'] } })] })]),
];
const links = [
  { kind: 'realizes', source: 'SF-1', target: 'OA-1' },
  { kind: 'satisfies', source: 'SF-1', target: 'REQ-1' },
];

test('an element knows every view that draws it, in view order', () => {
  const model = index(graphs, links);
  assert.deepEqual(model.placesOf('SF-1').map((place) => place.graph), ['sab', 'lab']);
  assert.deepEqual(model.placesOf('REQ-1'), []);
});

test('a trace is a relation on both of its ends', () => {
  const model = index(graphs, links);
  const out = model.relationsOf('SF-1');
  assert.deepEqual(out.map((r) => [r.label, r.name, r.places.map((p) => p.graph)]), [
    ['Realizes', 'Monitor Road', ['oab']],
    ['Satisfies', 'REQ-1', []],
  ]);
  assert.deepEqual(model.relationsOf('OA-1').map((r) => [r.label, r.id, r.name]), [['Realized by', 'SF-1', 'Detect Obstacle']]);
});

test('a deployed component is the way from a logical component to the node hosting it', () => {
  const model = index(graphs, links);
  assert.deepEqual(model.relationsOf('LC-1').map((r) => [r.label, r.id, r.places[0].graph]), [['Realized by', 'PN-1/Perception', 'pab']]);
  assert.deepEqual(model.relationsOf('PN-1/Perception').map((r) => [r.label, r.id]), [['Realizes', 'LC-1']]);
});

test('an unknown trace type still reads in both directions', () => {
  const model = index(graphs, [{ kind: 'constrains', source: 'SF-1', target: 'OA-1' }]);
  assert.equal(model.relationsOf('SF-1')[0].label, 'Constrains');
  assert.equal(model.relationsOf('OA-1')[0].label, 'Constrains (from)');
});

test('the outline keeps the nesting of every view', () => {
  const model = index(graphs, links);
  const lab = model.outline().find((entry) => entry.graph === 'lab');
  assert.equal(lab.title, 'Logical');
  assert.deepEqual(lab.nodes.map((n) => [n.name, n.children.map((c) => c.name)]), [['Perception', ['Detect Obstacle']]]);
});

test('an element with no relation and one view has nothing to navigate to', () => {
  const model = index(graphs, links);
  assert.deepEqual(model.relationsOf('PN-1'), []);
  assert.equal(model.placesOf('PN-1').length, 1);
});

test('a trace type named like a built-in property is still plain text', () => {
  for (const kind of ['constructor', 'toString', '__proto__']) {
    const model = index(graphs, [{ kind, source: 'SF-1', target: 'OA-1' }]);
    const label = model.relationsOf('OA-1')[0].label;
    assert.equal(typeof label, 'string');
    assert.match(label, /\(from\)$/);
  }
});
