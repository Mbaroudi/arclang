// Unit tests of the themes: the colour science, the validator, the presets
// and a theme applied by the renderer (run with `node --test`). The fixtures
// of tests/fixtures/themes are shared with the Rust validator
// (src/compiler/diagram/theme.rs): both must reach the same verdicts.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const here = dirname(fileURLToPath(import.meta.url));
const viewer = resolve(here, '../../src/compiler/diagram/viewer');
const fixtures = resolve(here, '../../tests/fixtures/themes');
const Color = require(join(viewer, 'arcviz-color.js'));
const Theme = require(join(viewer, 'arcviz-theme.js'));
const Render = require(join(viewer, 'arcviz-render.js'));

test('CIEDE2000 matches the reference pairs of Sharma, Wu and Dalal (2005)', () => {
  const pairs = [
    [[50, 2.6772, -79.7751], [50, 0, -82.7485], 2.0425],
    [[50, -1.3802, -84.2814], [50, 0, -82.7485], 1.0],
    [[60.2574, -34.0099, 36.2677], [60.4626, -34.1751, 39.4387], 1.2644],
    [[2.0776, 0.0795, -1.135], [0.9033, -0.0636, -0.5514], 0.9082],
  ];
  for (const [a, b, expected] of pairs) assert.ok(Math.abs(Color.deltaELab(a, b) - expected) < 1e-4, `${a} / ${b}`);
});

test('WCAG contrast and L* are those of the standard', () => {
  assert.equal(Math.round(Color.contrast(Color.parse('#777777'), Color.parse('#FFFFFF')) * 100) / 100, 4.48);
  assert.equal(Math.round(Color.contrast(Color.parse('#000000'), Color.parse('#FFFFFF'))), 21);
  assert.deepEqual(Color.lab(Color.parse('#FF0000')).map((v) => Math.round(v * 100) / 100), [53.24, 80.09, 67.2]);
});

test('red and green collapse for a deuteranope, blue and yellow do not', () => {
  const close = (a, b) => Color.deltaE(Color.simulate(Color.parse(a), 'deuteranopia'), Color.simulate(Color.parse(b), 'deuteranopia'));
  assert.ok(close('#B8442F', '#4E7F2E') < 5);
  assert.ok(close('#0072B2', '#F0E442') > 40);
});

test('every built-in preset passes every rule, its dark variant too', () => {
  const ids = Theme.presets().map((p) => p.id);
  assert.deepEqual(ids, ['arclang-default', 'capella-classic', 'colorblind-safe', 'print-grayscale', 'high-contrast', 'monochrome-document']);
  for (const id of ids) {
    const result = Theme.validate(Theme.preset(id));
    assert.equal(result.ok, true, `${id}: ${result.errors.map((e) => e.message).join('; ')}`);
  }
  assert.ok(Theme.darkOf(Theme.preset('arclang-default')), 'the default theme has a dark variant');
});

test('every preset colours every concept and every exchange', () => {
  for (const { id } of Theme.presets()) {
    const preset = Theme.preset(id);
    for (const concept of Theme.CONCEPTS) assert.ok(preset.concepts[concept.key]?.fill, `${id} ${concept.key}`);
    for (const exchange of Theme.EXCHANGES) assert.ok(preset.exchanges[exchange.key]?.line, `${id} ${exchange.key}`);
  }
});

test('Capella classic keeps Capella colours where its code is', () => {
  const capella = Theme.preset('capella-classic');
  assert.equal(capella.concepts.logical_component.fill, '#96B1DA'); // _CAP_Component_Blue
  assert.equal(capella.concepts.physical_node.fill, '#FFFCB7'); // _CAP_Node_Yellow
  // Functions are one step darker than _CAP_xAB_Function_Green, to stay apart from nodes for a deuteranope.
  assert.ok(Color.deltaE(Color.parse(capella.concepts.system_function.fill), Color.parse('#C5FFA6')) < 6);
  assert.equal(capella.concepts.operational_activity.fill, '#F7DA74'); // _CAP_Activity_Orange
  assert.equal(capella.exchanges.physical_link.line, '#EF2929'); // Sirius red
  assert.equal(capella.exchanges.functional_exchange.line, '#095C2E'); // _CAP_xAB_Function_Border_Green
  assert.match(capella.source, /common\.odesign/);
});

for (const file of readdirSync(fixtures).filter((name) => name.endsWith('.json')).sort()) {
  const fixture = JSON.parse(readFileSync(join(fixtures, file), 'utf8'));
  test(`fixture ${file}: ${fixture.description}`, () => {
    const result = Theme.validate(fixture.theme);
    assert.equal(result.ok, fixture.expect.ok, result.errors.map((e) => e.message).join('; '));
    for (const rule of fixture.expect.rules) assert.ok(result.errors.some((e) => e.rule === rule), `rule ${rule} in ${result.errors.map((e) => e.rule)}`);
    for (const text of fixture.expect.errors) assert.ok(result.errors.some((e) => e.message.includes(text)), `"${text}" in\n${result.errors.map((e) => e.message).join('\n')}`);
  });
}

test('a file that is not JSON is refused with the parser\'s reason', () => {
  const result = Theme.parse('{ "version": 1, ');
  assert.equal(result.ok, false);
  assert.match(result.errors[0].message, /^not JSON/);
});

test('a refused theme is never turned into a palette', () => {
  const bad = JSON.parse(readFileSync(join(fixtures, 'reserved-error-red.json'), 'utf8')).theme;
  assert.throws(() => Theme.palette(bad), /theme refused: actor stroke #E51400/);
});

test('a duplicated preset is a complete theme of its own, with a new id', () => {
  const copy = Theme.duplicate(Theme.preset('capella-classic'), 'Programme Astra');
  assert.equal(copy.id, 'programme-astra');
  assert.equal(copy.extends, undefined);
  assert.equal(Theme.validate(copy).ok, true);
});

// --- the renderer --------------------------------------------------------
const box = (id, kind, extra = {}) => ({ id, x: 10, y: 10, width: 120, height: 44, labels: [{ text: id }], ports: [], arc: { kind, realizes: [], properties: {} }, ...extra });
const view = (viewId, children, edges = []) => ({ id: viewId, width: 400, height: 200, children, edges, arc: { view: viewId, title: viewId, chains: [] } });
const fillOf = (svg, id) => new RegExp(`data-id="${id}"[^]*?class="av-shape"[^>]*fill="([^"]+)"`).exec(svg)[1];
const strokeOf = (svg, id) => new RegExp(`data-id="${id}"[^]*?class="av-shape"[^>]*stroke="([^"]+)"`).exec(svg)[1];

test('a theme colours the boxes, the lines, the text and the sheet', () => {
  const palette = Theme.palette(Theme.preset('capella-classic'));
  const edge = { id: 'E1', sources: ['A'], targets: ['B'], container: 'lab', labels: [{ text: 'data', x: 40, y: 40 }], sections: [{ startPoint: { x: 130, y: 30 }, endPoint: { x: 200, y: 30 } }], arc: { kind: 'functional_exchange', source_node: 'A', target_node: 'B', properties: {} } };
  const { svg } = Render.renderSvg(view('lab', [box('A', 'function'), box('B', 'logical_component', { x: 200 })], [edge]), { theme: palette });
  assert.equal(fillOf(svg, 'A'), '#B8F396');
  assert.equal(strokeOf(svg, 'A'), '#095C2E');
  assert.equal(fillOf(svg, 'B'), '#96B1DA');
  assert.match(svg, /class="av-line"[^>]*stroke="#095C2E"/);
  assert.match(svg, /class="av-paper"[^>]*fill="#FFFFFF"/);
  assert.match(svg, />A<\/text>/);
  assert.match(svg, /fill="#000000">A</);
});

test('one theme colours every layer: a function has one colour in System, Logical and Physical', () => {
  const palette = Theme.palette(Theme.preset('colorblind-safe'));
  const fills = ['sab', 'lab', 'pab'].map((id) => fillOf(Render.renderSvg(view(id, [box('F', 'function')]), { theme: palette }).svg, 'F'));
  assert.equal(new Set(fills).size, 1);
  assert.equal(fills[0], Theme.preset('colorblind-safe').concepts.logical_function.fill);
  const activity = fillOf(Render.renderSvg(view('oab', [box('F', 'operational_activity')]), { theme: palette }).svg, 'F');
  assert.notEqual(activity, fills[0]);
});

test('setTheme colours later drawings, the legend\'s styles too, and null puts the built-in colours back', () => {
  const graph = view('sab', [box('C', 'system_component')]);
  const before = fillOf(Render.renderSvg(graph).svg, 'C');
  Render.setTheme(Theme.palette(Theme.preset('high-contrast')));
  try {
    assert.equal(fillOf(Render.renderSvg(graph).svg, 'C'), Theme.preset('high-contrast').concepts.system_component.fill);
    assert.equal(Render.nodeStyle('system_component').fill, Theme.preset('high-contrast').concepts.system_component.fill);
    assert.equal(Render.edgeStyle('physical_link').stroke, Theme.preset('high-contrast').exchanges.physical_link.line);
    // An export asks for its own colours whatever the screen shows.
    assert.equal(fillOf(Render.renderSvg(graph, { theme: null }).svg, 'C'), before);
  } finally {
    Render.setTheme(null);
  }
  assert.equal(fillOf(Render.renderSvg(graph).svg, 'C'), before);
});

test('the dark variant draws on its dark sheet with light text', () => {
  const palette = Theme.palette(Theme.preset('arclang-default'), { dark: true });
  assert.equal(palette.dark, true);
  const { svg } = Render.renderSvg(view('lab', [box('L', 'logical_component')]), { theme: palette });
  assert.match(svg, /class="av-paper"[^>]*fill="#1E1F22"/);
  assert.equal(fillOf(svg, 'L'), Theme.darkOf(Theme.preset('arclang-default')).concepts.logical_component.fill);
  // A theme without a dark variant draws its light one.
  assert.equal(Theme.palette(Theme.preset('capella-classic'), { dark: true }).dark, false);
});

test('a variant is drawn with the theme\'s marking style', () => {
  const palette = Theme.palette(Theme.preset('print-grayscale'));
  const { svg } = Render.renderSvg(view('lab', [box('V', 'logical_component', { arc: { kind: 'logical_component', realizes: [], properties: { variant: 'true' } } })]), { theme: palette });
  assert.match(svg, /class="av-shape"[^>]*stroke-dasharray="6 3"/);
});
