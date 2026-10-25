import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { arrangeCommandOf, arrangeKeybindings, dropLayer, groupIntoKind, groupIntoPlan, layerNameProblem, layerRows, moveToLayerEdit, stepLayer, tagsOf } from '../../src/pure/arrange';
import { ElementRecord } from '../../src/pure/workbench';

const element = (uuid: string, id: string, kind: string, owner: string | null = 'm', attributes: Record<string, unknown> = {}): ElementRecord => ({ uuid, id, kind, name: id, owner, attributes });

test('tags are read as the workbench lists them', () => {
  assert.deepEqual(tagsOf({ tags: ['Safety', 'Draft'] }), ['Safety', 'Draft']);
  assert.deepEqual(tagsOf({ tags: 'Safety' }), ['Safety']);
  assert.deepEqual(tagsOf({ tags: '["Safety", "Draft"]' }), ['Safety', 'Draft']);
  assert.deepEqual(tagsOf({}), []);
  assert.deepEqual(tagsOf(undefined), []);
});

test('moving to a layer replaces the first tag and keeps the others', () => {
  const tagged = element('a', 'LC-A', 'LogicalComponent', 'm', { tags: ['Safety', 'Draft'] });
  assert.deepEqual(moveToLayerEdit(tagged, 'Comms'), { op: 'set', element: 'LC-A', key: 'tags', value: '["Comms", "Draft"]' });
  assert.deepEqual(moveToLayerEdit(tagged, ''), { op: 'set', element: 'LC-A', key: 'tags', value: '["Draft"]' });
  assert.equal(moveToLayerEdit(tagged, 'Safety'), null, 'already there');
  const plain = element('b', 'LC-B', 'LogicalComponent');
  assert.deepEqual(moveToLayerEdit(plain, 'Safety'), { op: 'set', element: 'LC-B', key: 'tags', value: '["Safety"]' });
  const one = element('c', 'LC-C', 'LogicalComponent', 'm', { tags: ['Safety'] });
  assert.deepEqual(moveToLayerEdit(one, ''), { op: 'unset', element: 'LC-C', key: 'tags' });
  assert.deepEqual(moveToLayerEdit(element('d', 'LC-D', 'LogicalComponent', 'm', { tags: ['Draft', 'Safety'] }), 'Safety'), { op: 'set', element: 'LC-D', key: 'tags', value: '["Safety"]' }, 'no tag twice');
});

test('the layers of a model: new ones on top, Default at the bottom, counts by first tag', () => {
  const elements = [{ attributes: { tags: ['Safety'] } }, { attributes: { tags: ['Safety', 'Draft'] } }, { attributes: {} }, { attributes: { tags: ['Comms'] } }];
  assert.deepEqual(layerRows(elements, []).map((r) => [r.label, r.count]), [['Comms', 1], ['Draft', 0], ['Safety', 2], ['Default', 1]]);
  assert.deepEqual(layerRows(elements, ['', 'Safety']).map((r) => r.label), ['Comms', 'Draft', 'Default', 'Safety']);
  assert.deepEqual(layerRows(elements, [], ['Zone']).map((r) => r.label), ['Comms', 'Draft', 'Safety', 'Zone', 'Default'], 'a layer made on the panel');
});

test('layers move a step, or are dropped before another', () => {
  assert.deepEqual(stepLayer(['a', 'b', 'c'], 'b', -1), ['b', 'a', 'c']);
  assert.deepEqual(stepLayer(['a', 'b', 'c'], 'c', 1), ['a', 'b', 'c']);
  assert.deepEqual(dropLayer(['a', 'b', 'c'], 'c', 'a'), ['c', 'a', 'b']);
  assert.deepEqual(dropLayer(['a', 'b', 'c'], 'a', null), ['b', 'c', 'a']);
});

test('a new layer name is checked', () => {
  assert.equal(layerNameProblem('Safety', []), null);
  assert.match(layerNameProblem('', []) ?? '', /needs a name/);
  assert.match(layerNameProblem('Default', []) ?? '', /without a tag/);
  assert.match(layerNameProblem('Safety', ['Safety']) ?? '', /exists/);
  assert.match(layerNameProblem('a"b', []) ?? '', /quote/);
});

const all = [element('m', 'Model', 'Model', null), element('a', 'LC-A', 'LogicalComponent'), element('b', 'LC-B', 'LogicalComponent'), element('p', 'LC-P', 'LogicalComponent'), element('a1', 'LC-A1', 'LogicalComponent', 'p'), element('b1', 'LC-B1', 'LogicalComponent', 'p'), element('f', 'LF-1', 'LogicalFunction', 'a'), element('g', 'LF-2', 'LogicalFunction', 'a'), element('e', 'OE-1', 'OperationalActivity')];
const pick = (...ids: string[]) => ids.map((id) => all.find((e) => e.id === id)!);

test('group into a new component: added where the elements are, each moved in, one request list', () => {
  const plan = groupIntoPlan('lab', pick('LC-A', 'LC-B'), all, 'Sensing');
  assert.ok(!('error' in plan));
  assert.equal(plan.id, 'LC-1');
  assert.deepEqual(plan.requests, [
    { op: 'add', keyword: 'component', name: 'Sensing', attributes: [['id', '"LC-1"']], layer: 'logical_architecture' },
    { op: 'move', element: 'LC-A', container: 'LC-1' },
    { op: 'move', element: 'LC-B', container: 'LC-1' },
  ]);
  const inner = groupIntoPlan('lab', pick('LC-A1', 'LC-B1'), all, 'Inner');
  assert.ok(!('error' in inner) && inner.requests[0].op === 'add' && inner.requests[0].container === 'LC-P', 'inside their component');
  const functions = groupIntoPlan('lab', pick('LF-1', 'LF-2'), all, 'Plan');
  assert.ok(!('error' in functions) && functions.keyword === 'function');
});

test('group into a new one is refused where the method allows no such container', () => {
  assert.match((groupIntoKind('lab', pick('LC-A')) as { error: string }).error, /two or more/);
  assert.match((groupIntoKind('lab', pick('LC-A', 'LF-1')) as { error: string }).error, /mixes/);
  assert.match((groupIntoKind('lab', pick('LC-A', 'LC-A1')) as { error: string }).error, /different containers/);
  assert.ok('error' in groupIntoKind('oab', pick('OE-1', 'OE-1')));
  assert.match((groupIntoPlan('lab', pick('LC-A', 'LC-B'), all, '  ') as { error: string }).error, /needs a name/);
});

test('the Arrange keys, as the page reads them and VS Code binds them', () => {
  const press = (key: string, mods: { meta?: boolean; shift?: boolean; alt?: boolean }, code?: string) => arrangeCommandOf({ key, code, metaKey: !!mods.meta, ctrlKey: false, shiftKey: !!mods.shift, altKey: !!mods.alt });
  assert.equal(press('F', { meta: true, shift: true }), 'bring-front');
  assert.equal(press('B', { meta: true, shift: true }), 'send-back');
  assert.equal(press('Ï', { meta: true, shift: true, alt: true }, 'KeyF'), 'bring-forward', 'Alt types another letter: the code says which key');
  assert.equal(press('g', { meta: true }), 'group');
  assert.equal(press('G', { meta: true, shift: true }), 'ungroup');
  assert.equal(press('l', { meta: true }), 'lock');
  assert.equal(press('L', { meta: true, shift: true }), 'layers');
  assert.equal(press('E', { meta: true, shift: true }), 'select-edges');
  assert.equal(press('f', { meta: true }), null, '⌘F stays Find');
  assert.equal(press('g', {}), null);
  const pkg = JSON.parse(readFileSync(join(__dirname, '../../package.json'), 'utf8')) as { contributes: { keybindings: { command: string; key: string; mac: string; when: string }[]; commands: { command: string }[] } };
  for (const wanted of arrangeKeybindings('')) {
    const bound = pkg.contributes.keybindings.filter((k) => k.command === wanted.command);
    assert.equal(bound.length, 2, `${wanted.command}: in the workbench and in a diagram's tab`);
    for (const k of bound) {
      assert.equal(k.mac, wanted.mac);
      assert.equal(k.key, wanted.key);
      assert.match(k.when, /focusedView == '' && !inputFocus && !arclang.workbenchTyping/, 'only on the canvas');
    }
    assert.ok(pkg.contributes.commands.some((c) => c.command === wanted.command));
  }
  const canvas = pkg.contributes.keybindings.filter((k) => /arclang\.workbench\b/.test(k.when) || /arclang\.workbench\.diagram/.test(k.when));
  for (const wanted of arrangeKeybindings('')) {
    const clash = canvas.filter((k) => k.mac === wanted.mac && k.command !== wanted.command);
    assert.deepEqual(clash.map((k) => k.command), [], `${wanted.mac} is taken`);
  }
});
