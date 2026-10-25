// Unit tests of the export helpers: the box around a selection and the
// drawing cut to it, the sheet dropped for a transparent image (node --test).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
require(join(viewer, 'arcviz-export.js'));
const { crop, boundsOf, withoutPaper } = globalThis.ArcVizExport;
const { renderSvg } = require(join(viewer, 'arcviz-render.js'));

const graph = {
  id: 'lab', width: 500, height: 300, arc: { view: 'lab', title: 'Logical' },
  children: [
    { id: 'P', x: 20, y: 20, width: 300, height: 200, labels: [{ text: 'P' }], arc: { kind: 'logical_component', properties: {} }, children: [
      { id: 'A', x: 10, y: 30, width: 100, height: 40, labels: [{ text: 'A' }], arc: { kind: 'logical_component', properties: {} } },
    ] },
    { id: 'B', x: 380, y: 100, width: 100, height: 40, labels: [{ text: 'B' }], arc: { kind: 'logical_component', properties: {} } },
  ],
  edges: [{ id: 'E', container: 'lab', sections: [{ startPoint: { x: 130, y: 70 }, bendPoints: [{ x: 250, y: 260 }], endPoint: { x: 380, y: 120 } }], arc: { kind: 'component_exchange', properties: {} } }],
};

test('the box around a nested box counts its containers\' offsets; a line counts its route', () => {
  assert.deepEqual(boundsOf(graph, ['A']), { x: 30, y: 50, width: 100, height: 40 });
  assert.deepEqual(boundsOf(graph, ['A', 'B']), { x: 30, y: 50, width: 450, height: 90 });
  assert.deepEqual(boundsOf(graph, ['E']), { x: 130, y: 70, width: 250, height: 190 });
  assert.equal(boundsOf(graph, ['nothing']), null);
});

test('a drawing cut to a box keeps its markup, with a view box and a sheet of that box and its margin', () => {
  const drawn = renderSvg(graph);
  const cut = crop(drawn, { x: 30, y: 50, width: 100, height: 40 }, 10);
  assert.equal(cut.width, 120);
  assert.equal(cut.height, 60);
  assert.match(cut.svg, /^<svg[^>]* width="120" height="60" viewBox="20 40 120 60"/);
  assert.match(cut.svg, /<rect class="av-paper" x="20" y="40" width="120" height="60"/);
  assert.ok(cut.svg.includes('data-id="B"'), 'what lies outside is still in the file, out of view');
});

test('a transparent drawing has no sheet of paper', () => {
  const drawn = renderSvg(graph);
  assert.match(drawn.svg, /class="av-paper"/);
  assert.doesNotMatch(withoutPaper(drawn.svg), /class="av-paper"/);
});
