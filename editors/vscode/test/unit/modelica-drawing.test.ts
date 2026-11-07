import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { componentsOf, drawDiagram, escapeXml, fillText, placementMatrix, recordFields } from '../../src/pure/modelica-drawing';

const at = (m: number[], x: number, y: number) => [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]].map((v) => Math.round(v * 1000) / 1000);
const icon: [[number, number], [number, number]] = [[-100, -100], [100, 100]];

test('an icon is mapped onto its placement: its extent, then the origin', () => {
  const m = placementMatrix(icon, { origin: [50, 20], extent: [[-10, -5], [10, 5]], rotation: 0 });
  assert.deepEqual(at(m, 100, 100), [60, 25]);
  assert.deepEqual(at(m, -100, -100), [40, 15]);
});

test('a reversed extent flips the icon, a rotation turns it about the origin', () => {
  const flipped = placementMatrix(icon, { origin: [0, 0], extent: [[10, -10], [-10, 10]], rotation: 0 });
  assert.deepEqual(at(flipped, 100, 0), [-10, 0]);
  const turned = placementMatrix(icon, { origin: [0, 0], extent: [[-10, -10], [10, 10]], rotation: 90 });
  assert.deepEqual(at(turned, 100, 0), [0, 10]);
});

test("a graphic's positional record is read by the specification's field order", () => {
  const rectangle = recordFields('Rectangle', { $kind: 'record', name: 'Rectangle', elements: [true, [0, 0], 0, [0, 0, 255], [255, 255, 255], null, null, 0.25, null, [[-10, -10], [10, 10]], 2] });
  assert.deepEqual(rectangle.extent, [[-10, -10], [10, 10]]);
  assert.equal(rectangle.radius, 2);
  assert.deepEqual(rectangle.lineColor, [0, 0, 255]);
  // A connection's Line comes by name.
  assert.deepEqual(recordFields('Line', { points: [[0, 0], [1, 1]] }).points, [[0, 0], [1, 1]]);
});

test('a text fills %name, %class and a known parameter, and leaves the rest', () => {
  assert.equal(fillText('%name: R=%R, %%, %unknown', { name: 'r1', className: 'Resistor', parameters: { R: '10' } }), 'r1: R=10, %, %unknown');
});

test('a placed part, its connector and a connection are drawn; a part without a place is listed', () => {
  const connector = {
    name: 'P.Pin', restriction: 'connector',
    annotation: { Icon: { graphics: [{ $kind: 'record', name: 'Rectangle', elements: [true, [0, 0], 0, [0, 0, 255], [0, 0, 255], null, { $kind: 'enum', name: 'FillPattern.Solid' }, 0.25, null, [[-100, -100], [100, 100]], 0] }] } },
    elements: [],
  };
  const box = {
    name: 'P.Box', restriction: 'model',
    annotation: { Icon: { graphics: [{ $kind: 'record', name: 'Text', elements: [true, [0, 0], 0, [0, 0, 0], [0, 0, 0], null, null, 0.25, [[-100, 110], [100, 140]], '%name', 0, [0, 0, 255], '', [], null] }] } },
    elements: [{ $kind: 'component', name: 'p', type: connector, annotation: { Placement: { transformation: { extent: [[90, -10], [110, 10]] } } } }],
  };
  const instance = {
    name: 'P.System', restriction: 'model',
    elements: [
      { $kind: 'component', name: 'a', type: box, annotation: { Placement: { transformation: { origin: [0, 0], extent: [[-10, -10], [10, 10]] } } } },
      // A replaceable part's place is written after constrainedby.
      { $kind: 'component', name: 'b', type: box, prefixes: { replaceable: { annotation: { Placement: { transformation: { origin: [40, 0], extent: [[-10, -10], [10, 10]] } } } } } },
      { $kind: 'component', name: 'lost', type: box },
      { $kind: 'component', name: 'k', type: 'Real' },
    ],
    connections: [{ lhs: { $kind: 'cref', parts: [{ name: 'a' }, { name: 'p' }] }, rhs: { $kind: 'cref', parts: [{ name: 'b' }, { name: 'p' }] }, annotation: { Line: { points: [[11, 0], [29, 0]], color: [255, 0, 0] } } }],
  };
  const drawing = drawDiagram(instance);
  assert.deepEqual(drawing.components.map((c) => c.name), ['a', 'b']);
  assert.deepEqual(drawing.unplaced, ['lost']);
  assert.equal(drawing.connections, 1);
  assert.match(drawing.svg, /<text[^>]*>a<\/text>/);
  assert.match(drawing.svg, /stroke="rgb\(255,0,0\)"/);
  assert.equal((drawing.svg.match(/<path[^>]*fill="rgb\(0,0,255\)"/g) ?? []).length, 2, 'each part shows its connector');
  assert.ok(drawing.components[0].opens);
  assert.ok(componentsOf(instance).find((c) => c.name === 'p') === undefined);
});

test('the names and texts of the model are escaped in the SVG', () => {
  assert.equal(escapeXml('<a href="x">&\'</a>'), '&lt;a href=&quot;x&quot;&gt;&amp;&#39;&lt;/a&gt;');
  const instance = { name: 'X', elements: [], annotation: { Diagram: { graphics: [{ $kind: 'record', name: 'Text', elements: [true, [0, 0], 0, [0, 0, 0], [0, 0, 0], null, null, 0.25, [[-50, -10], [50, 10]], '<script>x</script>', 0, [0, 0, 0], '', [], null] }] } } };
  const svg = drawDiagram(instance).svg;
  assert.ok(!svg.includes('<script>'));
  assert.ok(svg.includes('&lt;script&gt;'));
});
