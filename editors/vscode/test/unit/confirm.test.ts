import assert from 'node:assert/strict';
import { test } from 'node:test';
import { confirmEdits, confirmRequest, mdFence, mdText } from '../../src/pure/confirm';

test('a value shows as text: no link, no emphasis, no code span it can close', () => {
  const sneaky = '`x` [click](https://evil.example) **bold** <img src=x> _i_';
  const out = mdText(sneaky);
  assert.ok(!/(^|[^\\])`/.test(out), out);
  assert.ok(!/(^|[^\\])\[/.test(out) && !/(^|[^\\])\(/.test(out), out);
  assert.ok(!/(^|[^\\])\*/.test(out) && !/(^|[^\\])</.test(out) && !/(^|[^\\])_/.test(out), out);
  assert.equal(mdText('a\nb'), 'a ⏎ b', 'a value stays on its line');
});

test('a raw declaration is shown whole, in a fence its backticks cannot close', () => {
  const declaration = 'component_exchange "A" {\n  from_port: "X.out"\n  // ``` not the end\n  to_port: "Y.in"\n}';
  const item = confirmRequest({ op: 'add_raw', declaration, layer: 'logical_architecture' });
  for (const line of declaration.split('\n')) assert.ok(item.includes(line), line);
  assert.ok(item.includes('````arclang'), item);
  assert.equal(mdFence('plain'), '```arclang\nplain\n```');
});

test('the confirmation of an edit names the file in full and every request', () => {
  const out = confirmEdits('models/sub/car.arc', [
    { op: 'set', element: 'LC-1', key: 'description', value: '"`rm -rf` [x](y)"' },
    { op: 'rename', element: 'LC-1', name: 'Brain' },
    { op: 'add_raw', declaration: 'port in "p"', container: 'LC-1' },
  ]);
  assert.match(out, /models\/sub\/car\\\.arc/);
  assert.match(out, /rename LC\\-1 to \\"Brain\\"/);
  assert.match(out, /port in "p"/);
  assert.ok(!out.includes('[x](y)'), out);
});
