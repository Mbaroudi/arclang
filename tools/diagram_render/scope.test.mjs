// Unit tests of what part of each view is shown (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const collapse = require(join(viewer, 'arcviz-collapse.js'));
const { create } = require(join(viewer, 'arcviz-scope.js'));

const leaf = (id) => ({ id, width: 100, height: 40, labels: [{ text: id }], ports: [], arc: { kind: 'function' } });
const box = (id, children) => ({ id, labels: [{ text: id }], ports: [], children, arc: { kind: 'logical_component' }, layoutOptions: {} });
const graphs = () => [
  { id: 'lab', children: [box('A', [leaf('A1'), box('AA', [leaf('AA1')])]), box('B', [leaf('B1')]), leaf('C')], edges: [], arc: { view: 'lab' } },
  { id: 'cdb', children: [leaf('K')], edges: [], arc: { view: 'cdb' } },
];
const memory = (initial = {}) => {
  const data = { ...initial };
  return { data, getItem: (key) => (key in data ? data[key] : null), setItem: (key, value) => { data[key] = value; } };
};
const scopeOf = (storage) => create({ graphs: graphs(), collapse, storage, storageKey: 'arcviz:test' });

test('a view starts whole, and its key names what is folded and focused', () => {
  const scope = scopeOf(memory());
  assert.deepEqual([...scope.foldedOf('lab')], []);
  assert.equal(scope.focusOf('lab'), null);
  const whole = scope.key('lab');
  scope.setFolded('lab', new Set(['B']));
  assert.notEqual(scope.key('lab'), whole);
  assert.equal(scope.key('lab', scope.WHOLE), whole);
});

test('the shape to lay out applies the focus, then the folds', () => {
  const scope = scopeOf(memory());
  scope.setFocus('lab', 'A');
  scope.setFolded('lab', new Set(['AA']));
  const shape = scope.shape(graphs()[0]);
  assert.deepEqual(shape.children.map((node) => node.id), ['A']);
  assert.equal(shape.children[0].children[1].arc.folded, 1);
  assert.deepEqual(scope.shape(graphs()[0], scope.WHOLE).children.map((node) => node.id), ['A', 'B', 'C']);
});

test('only containers can be focused, and not the one already open', () => {
  const scope = scopeOf(memory());
  assert.equal(scope.canFocus('lab', 'A'), true);
  assert.equal(scope.canFocus('lab', 'C'), false);
  scope.setFocus('lab', 'C');
  assert.equal(scope.focusOf('lab'), null, 'a leaf is not a diagram');
  scope.setFocus('lab', 'AA');
  assert.equal(scope.canFocus('lab', 'AA'), false);
  assert.deepEqual(scope.path('lab'), ['A', 'AA']);
});

test('an element outside the focus is not shown by it', () => {
  const scope = scopeOf(memory());
  scope.setFocus('lab', 'A');
  assert.equal(scope.shows('lab', 'AA1'), true);
  assert.equal(scope.shows('lab', 'A'), true);
  assert.equal(scope.shows('lab', 'B1'), false);
  scope.setFocus('lab', null);
  assert.equal(scope.shows('lab', 'B1'), true);
});

test('folds and focus come back after a reload', () => {
  const storage = memory();
  const first = scopeOf(storage);
  first.setFolded('lab', new Set(['B']));
  first.setFocus('lab', 'A');
  const second = scopeOf(storage);
  assert.deepEqual([...second.foldedOf('lab')], ['B']);
  assert.equal(second.focusOf('lab'), 'A');
});

test('what was saved for another model, or is not valid, is ignored', () => {
  const slot = Object.keys((() => { const m = memory(); scopeOf(m).setFolded('lab', new Set(['A'])); return m.data; })())[0];
  for (const saved of ['not json', '{"views":{"lab":{"folded":["Gone","B"],"open":"Gone"}}}', '{"views":{"lab":{"folded":"B"}}}', '[]', '{"views":{"__proto__":{"folded":["B"]}}}']) {
    const scope = scopeOf(memory({ [slot]: saved }));
    assert.deepEqual([...scope.foldedOf('lab')].filter((id) => id !== 'B'), [], saved);
    assert.equal(scope.focusOf('lab'), null, saved);
  }
});

test('a browser that refuses storage still folds for the session', () => {
  const refusing = { getItem: () => { throw new Error('denied'); }, setItem: () => { throw new Error('denied'); } };
  const scope = scopeOf(refusing);
  scope.setFolded('lab', new Set(['A']));
  assert.deepEqual([...scope.foldedOf('lab')], ['A']);
  assert.doesNotThrow(() => create({ graphs: graphs(), collapse, storage: null, storageKey: 'k' }).setFocus('lab', 'A'));
});

test('the open container itself never folds', () => {
  const scope = scopeOf(memory());
  scope.setFolded('lab', new Set(['A', 'AA']));
  scope.setFocus('lab', 'A');
  assert.deepEqual(scope.foldable('lab'), ['AA']);
  const shape = scope.shape(graphs()[0]);
  assert.ok(Array.isArray(shape.children[0].children), 'A is open, not a single box');
  assert.equal(shape.children[0].children[1].arc.folded, 1, 'what is inside it can still be folded');
});

test('two models with the same title do not share what was folded', () => {
  const storage = memory();
  const first = create({ graphs: graphs(), collapse, storage, storageKey: 'arcviz:Same' });
  first.setFolded('lab', new Set(['A']));
  const other = graphs();
  other[0].children[0].id = 'A'; // same id ...
  other[0].children.push(box('Z', [leaf('Z1')])); // ... in a different model
  const second = create({ graphs: other, collapse, storage, storageKey: 'arcviz:Same' });
  assert.deepEqual([...second.foldedOf('lab')], []);
  assert.deepEqual([...create({ graphs: graphs(), collapse, storage, storageKey: 'arcviz:Same' }).foldedOf('lab')], ['A']);
});

test('the whole-view scope cannot be changed through a view that uses it', () => {
  const scope = scopeOf(memory());
  scope.setFocus('lab', 'A');
  scope.foldedOf('lab').add('B');
  assert.deepEqual([...scope.WHOLE.folded], []);
});

test('a box moved twice ends where both moves take it, and only in the scope it was moved in', () => {
  const scope = scopeOf(memory());
  scope.moveBy('lab', 'C', 30, 10);
  scope.moveBy('lab', 'C', -10, 5);
  assert.deepEqual(scope.placedOf('lab'), { C: { dx: 20, dy: 15 } });
  scope.setFocus('lab', 'A');
  assert.deepEqual(scope.placedOf('lab'), {}, 'another diagram, another arrangement');
  assert.deepEqual(scope.placedOf('lab', scope.WHOLE), { C: { dx: 20, dy: 15 } });
});

test('a box moved back to where it was is no longer placed', () => {
  const scope = scopeOf(memory());
  scope.moveBy('lab', 'C', 30, 0);
  scope.moveBy('lab', 'C', -30, 0);
  assert.deepEqual(scope.placedOf('lab'), {});
});

test('resetting a view forgets its placement and nothing else', () => {
  const scope = scopeOf(memory());
  scope.setFolded('lab', new Set(['B']));
  scope.moveBy('lab', 'C', 30, 10);
  scope.moveBy('cdb', 'K', 5, 5);
  scope.resetPlaces('lab');
  assert.deepEqual(scope.placedOf('lab'), {});
  assert.deepEqual([...scope.foldedOf('lab')], ['B']);
  assert.deepEqual(scope.placedOf('cdb'), { K: { dx: 5, dy: 5 } });
});

test('placement comes back after a reload, and what is not a placement is ignored', () => {
  const storage = memory();
  scopeOf(storage).moveBy('lab', 'C', 30, 10);
  assert.deepEqual(scopeOf(storage).placedOf('lab'), { C: { dx: 30, dy: 10 } });

  const slot = Object.keys(storage.data)[0];
  const saved = JSON.parse(storage.data[slot]);
  saved.arrangements[0].places = { C: { dx: 'far', dy: 1 }, Gone: { dx: 1, dy: 1 }, B: { dx: 4, dy: Infinity }, A: { dx: 7, dy: 8 } };
  saved.arrangements.push({ view: 'not a view', open: null, folded: [], places: { C: { dx: 1, dy: 1 } } });
  storage.data[slot] = JSON.stringify(saved);
  assert.deepEqual(scopeOf(storage).placedOf('lab'), { A: { dx: 7, dy: 8 } });
});

test('a placement is bounded, and settles on where the boxes really went', () => {
  const scope = scopeOf(memory());
  scope.moveBy('lab', 'C', 1e308, 0);
  assert.deepEqual(scope.placedOf('lab'), {}, 'an absurd move is refused');
  scope.moveBy('lab', 'A1', 5000, 0);
  scope.settle('lab', { A1: { dx: 40, dy: 0 }, Gone: { dx: 1, dy: 1 } });
  assert.deepEqual(scope.placedOf('lab'), { A1: { dx: 40, dy: 0 } });
  scope.moveBy('lab', 'A1', -10, 0);
  assert.deepEqual(scope.placedOf('lab'), { A1: { dx: 30, dy: 0 } }, 'a small move back shows at once');
});

// --- the layout file ---------------------------------------------------------
const file = () => ({
  arclang_layout: '1',
  views: { lab: { folded: ['B'], open: null } },
  arrangements: [{ view: 'lab', open: null, folded: ['B'], places: { C: { dx: 12, dy: -4 } } }],
});
const withFile = (storage, initial) => create({ graphs: graphs(), collapse, storage, storageKey: 'arcviz:test', initial });

test('the layout file is what a reader sees first', () => {
  const scope = withFile(memory(), file());
  assert.deepEqual([...scope.foldedOf('lab')], ['B']);
  assert.deepEqual(scope.placedOf('lab'), { C: { dx: 12, dy: -4 } });
  assert.equal(scope.dirty(), false);
});

test('what the reader arranges is written back in the same form', () => {
  const scope = withFile(memory(), file());
  assert.deepEqual(scope.toFile(), file());
  scope.setFocus('lab', 'A');
  scope.moveBy('lab', 'A1', 5, 5);
  assert.equal(scope.dirty(), true);
  assert.deepEqual(scope.toFile(), {
    arclang_layout: '1',
    views: { lab: { folded: ['B'], open: 'A' } },
    arrangements: [
      { view: 'lab', open: 'A', folded: ['B'], places: { A1: { dx: 5, dy: 5 } } },
      { view: 'lab', open: null, folded: ['B'], places: { C: { dx: 12, dy: -4 } } },
    ],
  });
});

test('a view left whole is not written, and an untouched model writes an empty layout', () => {
  assert.deepEqual(scopeOf(memory()).toFile(), { arclang_layout: '1', views: {}, arrangements: [] });
  const scope = scopeOf(memory());
  scope.setFolded('lab', new Set(['A']));
  scope.setFolded('lab', new Set());
  assert.deepEqual(scope.toFile().views, {});
});

test('the reader\'s own changes hold after a reload, until the file itself changes', () => {
  const storage = memory();
  const first = withFile(storage, file());
  first.setFolded('lab', new Set(['A']));
  assert.deepEqual([...withFile(storage, file()).foldedOf('lab')], ['A'], 'same file: the working copy');
  const updated = file();
  updated.views.lab.folded = ['A', 'B'];
  assert.deepEqual([...withFile(storage, updated).foldedOf('lab')].sort(), ['A', 'B'], 'new file: the file');
});

test('a layout file naming what the model does not draw is ignored piece by piece', () => {
  const stale = { arclang_layout: '1', views: { lab: { folded: ['Gone', 'B'], open: 'C' }, nope: { folded: ['A'], open: null } }, arrangements: [{ view: 'lab', open: null, folded: ['B'], places: { Gone: { dx: 1, dy: 1 }, C: { dx: 2, dy: 2 } } }, 'junk', { view: 'lab', places: 'junk' }] };
  const scope = withFile(memory(), stale);
  assert.deepEqual([...scope.foldedOf('lab')], ['B']);
  assert.equal(scope.focusOf('lab'), null);
  assert.deepEqual(scope.placedOf('lab'), { C: { dx: 2, dy: 2 } });
  assert.doesNotThrow(() => withFile(memory(), 'junk'));
});

test('going back to the file discards the working copy', () => {
  const storage = memory();
  const scope = withFile(storage, file());
  scope.setFolded('lab', new Set(['A']));
  scope.revert();
  assert.deepEqual([...scope.foldedOf('lab')], ['B']);
  assert.equal(scope.dirty(), false);
  assert.deepEqual([...withFile(storage, file()).foldedOf('lab')], ['B']);
});

test('placement made in an open container is found whether or not that container is also folded', () => {
  const scope = scopeOf(memory());
  scope.setFolded('lab', new Set(['A']));
  scope.setFocus('lab', 'A');
  scope.moveBy('lab', 'A1', 5, 5);
  scope.setFolded('lab', new Set());
  assert.deepEqual(scope.placedOf('lab'), { A1: { dx: 5, dy: 5 } });
});
