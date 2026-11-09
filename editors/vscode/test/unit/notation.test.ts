import assert from 'node:assert/strict';
import { test } from 'node:test';
import { defaultNotation, graphsOfNotation, NOTATION_VIEWS, parseSnapshot, type CesamSnapshot, type ViewerGraph } from '../../src/pure/workbench';

const graph = (id: string, view: string): ViewerGraph => ({ id, arc: { view, title: id } });
const graphs = [graph('sab', 'sab'), graph('lab', 'lab'), graph('gv', 'gv'), graph('iv', 'iv'), graph('msm:Modes', 'msm'), graph('req', 'req'), graph('oab', 'oab')];
const cesam: CesamSnapshot = {
  declared: true,
  table: { columns: [], rows: [] },
  diagrams: [
    { id: 'req', vision: 'operational', aspect: 'requirements', view: 'Stakeholder requirements' },
    { id: 'oab', vision: 'operational', aspect: 'interactions', view: 'Environment diagram' },
    { id: 'msm:Modes', vision: 'functional', aspect: 'states', view: 'Functional modes' },
    { id: 'lab', vision: 'constructional', aspect: 'interactions', view: 'Constructional interactions' },
  ],
};

test('a model opens in its notation: CESAM when it declares it, SysML v2 for a .sysml file, else Arcadia', () => {
  assert.equal(defaultNotation({ cesam }, 'lock.arc'), 'cesam');
  assert.equal(defaultNotation({ cesam: { ...cesam, declared: false } }, 'drone.sysml'), 'sysml');
  assert.equal(defaultNotation({ cesam: { ...cesam, declared: false } }, 'drone.arc'), 'arcadia');
  assert.equal(defaultNotation(null, undefined), 'arcadia');
});

test('each notation shows only its own diagrams: no SysML view among the Arcadia ones, nor the reverse', () => {
  assert.deepEqual(graphsOfNotation('arcadia', graphs, cesam).map((g) => g.id), ['oab', 'sab', 'lab', 'msm:Modes', 'req']);
  assert.deepEqual(graphsOfNotation('sysml', graphs, cesam).map((g) => g.id), ['gv', 'iv', 'msm:Modes', 'req']);
  // CESAM: the diagrams its grid names, in the grid's order.
  assert.deepEqual(graphsOfNotation('cesam', graphs, cesam).map((g) => g.id), ['req', 'oab', 'msm:Modes', 'lab']);
  assert.deepEqual(graphsOfNotation('cesam', graphs, undefined), []);
  for (const view of ['gv', 'iv', 'afv']) assert.ok(!NOTATION_VIEWS.arcadia.includes(view), view);
  for (const view of ['oab', 'sab', 'lab', 'pab']) assert.ok(!NOTATION_VIEWS.sysml.includes(view), view);
});

test('the snapshot keeps the CESAM grid the compiler sends, and drops a malformed one', () => {
  assert.deepEqual(parseSnapshot(JSON.stringify({ success: true, cesam })).cesam, cesam);
  assert.equal(parseSnapshot(JSON.stringify({ success: true, cesam: { table: {} } })).cesam, undefined);
  assert.equal(parseSnapshot(JSON.stringify({ success: true })).cesam, undefined);
});
