import assert from 'node:assert/strict';
import { test } from 'node:test';
import { filterPaths, globToRegExp } from '../../src/pure/glob';

test('glob patterns match as VS Code reads them', () => {
  const r = globToRegExp('**/*.arc');
  for (const ok of ['starter.arc', 'examples/aerospace/x.arc']) assert.ok(r.test(ok), ok);
  for (const no of ['examples', 'battery.mo', 'x.arc.bak', 'examples/aerospace']) assert.equal(r.test(no), false, no);
  assert.ok(globToRegExp('**/*.{arc,mo}').test('a/b.mo'));
  assert.ok(globToRegExp('Systems/*.mo').test('Systems/Car.mo'));
  assert.equal(globToRegExp('Systems/*.mo').test('Systems/x/Car.mo'), false);
  assert.ok(globToRegExp('**/{node_modules,.git,target}/**').test('a/node_modules/b/c.arc'));
});

test('the browser\'s answer of every path is filtered down to what the pattern names', () => {
  const all = ['starter.arc', 'battery.mo', 'examples', 'examples/aerospace', 'examples/aerospace/x.arc', 'node_modules/p/y.arc'];
  assert.deepEqual(filterPaths(all, '**/*.arc', '**/{node_modules,.git,target}/**'), ['starter.arc', 'examples/aerospace/x.arc']);
});
