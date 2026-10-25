import assert from 'node:assert/strict';
import { test } from 'node:test';
import { declarationSpan, freshId, looksLikeArcLang, pasteTextRequests, selectionSource, splitDeclarations } from '../../src/pure/clipboard';
import { ElementRecord } from '../../src/pure/workbench';

const MODEL = `logical_architecture "L" {
  component "Camera" {
    id: "LC-CAM" // the eye
    port out "lane"
    function "Detect { lines }" { id: "LF-DET" }
  }
  component "Controller" {
    id: "LC-CTL"
    port in "lane"
  }
  component "Spare" {}
  component_exchange "LaneFlow" {
    from_port: "LC-CAM.lane"
    to_port: "LC-CTL.lane"
  }
}
`;
const record = (id: string, name: string): ElementRecord => ({ uuid: id, id, name, kind: 'LogicalComponent', owner: null, attributes: {} });

test('the declaration of an element is found by its id, else by its name, braces in strings aside', () => {
  const cam = declarationSpan(MODEL, { id: 'LC-CAM', name: 'Camera' })!;
  assert.equal(MODEL.slice(cam.start, cam.end), '  component "Camera" {\n    id: "LC-CAM" // the eye\n    port out "lane"\n    function "Detect { lines }" { id: "LF-DET" }\n  }');
  const spare = declarationSpan(MODEL, { id: 'LC-Spare', name: 'Spare' })!;
  assert.equal(MODEL.slice(spare.start, spare.end), '  component "Spare" {}');
  assert.equal(declarationSpan(MODEL, { id: 'LC-NOPE', name: 'Nope' }), null);
  assert.equal(declarationSpan(MODEL, { id: 'X', name: 'LaneFlow' }), null, 'an exchange is no element');
});

test('a selection is copied as source: its declarations, then the exchanges named between them', () => {
  const { text, missing } = selectionSource(MODEL, [{ id: 'LC-CAM', name: 'Camera' }, { id: 'LC-CTL', name: 'Controller' }, { id: 'LF-DET', name: 'Detect' }, { id: 'Q', name: 'Q' }], ['LaneFlow']);
  assert.deepEqual(missing, ['Q']);
  assert.equal(text, 'component "Camera" {\n  id: "LC-CAM" // the eye\n  port out "lane"\n  function "Detect { lines }" { id: "LF-DET" }\n}\ncomponent "Controller" {\n  id: "LC-CTL"\n  port in "lane"\n}\ncomponent_exchange "LaneFlow" {\n  from_port: "LC-CAM.lane"\n  to_port: "LC-CTL.lane"\n}\n');
  assert.deepEqual(splitDeclarations(text).map((d) => d.split('\n')[0]), ['component "Camera" {', 'component "Controller" {', 'component_exchange "LaneFlow" {']);
  assert.ok(looksLikeArcLang(text));
  // Lines selected in a text editor, indented as they are inside their layer.
  assert.deepEqual(splitDeclarations('  component "A" {\n    id: "LC-A"\n  }\n'), ['component "A" {\n  id: "LC-A"\n}']);
  assert.ok(!looksLikeArcLang('hello, world'));
});

test('pasted text takes fresh ids where they are taken, its references follow, a taken name is set apart', () => {
  const copied = selectionSource(MODEL, [{ id: 'LC-CAM', name: 'Camera' }, { id: 'LC-CTL', name: 'Controller' }], ['LaneFlow']).text;
  // Into another model that has none of them: kept as they are.
  const elsewhere = pasteTextRequests(copied, [record('LC-X', 'X')], { layer: 'logical' });
  assert.ok(!('error' in elsewhere));
  if ('error' in elsewhere) return;
  assert.equal(elsewhere.requests.length, 3);
  assert.deepEqual(elsewhere.requests[0], { op: 'add_raw', declaration: 'component "Camera" {\n  id: "LC-CAM" // the eye\n  port out "lane"\n  function "Detect { lines }" { id: "LF-DET" }\n}', layer: 'logical' });
  // Back into the same model: everything renamed, the exchange joined to the copies.
  const here = pasteTextRequests(copied, [record('LC-CAM', 'Camera'), record('LC-CTL', 'Controller'), record('LF-DET', 'Detect { lines }'), record('LaneFlow', 'LaneFlow')], { container: 'LC-SYS' });
  if ('error' in here) return assert.fail(here.error);
  const [cam, , flow] = here.requests as { declaration: string; container: string }[];
  assert.ok(cam.declaration.startsWith('component "Camera 2" {\n  id: "LC-CAM-2"'), cam.declaration);
  assert.ok(cam.declaration.includes('{ id: "LF-DET-2" }'), cam.declaration);
  assert.equal(flow.declaration, 'component_exchange "LaneFlow 2" {\n  from_port: "LC-CAM-2.lane"\n  to_port: "LC-CTL-2.lane"\n}');
  assert.equal(cam.container, 'LC-SYS');
  assert.deepEqual(here.ids, ['LC-CAM-2', 'LC-CTL-2']);
  assert.deepEqual(pasteTextRequests('   \n', [], { layer: 'logical' }), { error: 'the clipboard holds no ArcLang declaration' });
});

test('a fresh id counts on from the number it ends with, keeping its width', () => {
  assert.equal(freshId('LC-X', new Set()), 'LC-X-2');
  assert.equal(freshId('LC-X', new Set(['LC-X-2'])), 'LC-X-3');
  assert.equal(freshId('REQ-007', new Set()), 'REQ-008');
});
