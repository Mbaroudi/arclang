import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  compareCounts,
  compareMarks,
  compareState,
  diffArgs,
  entryDetails,
  markStatus,
  parseConflicts,
  parseDiff,
  parseHistory,
  parseRefs,
  resolveConflict,
  RichDiff,
  validBaselineName,
  withGhosts,
} from '../../src/pure/semgit';
import type { ViewerPayload } from '../../src/pure/workbench';

function diff(): RichDiff {
  return {
    model: 'm.arc',
    from: 'HEAD',
    to: 'WORKTREE',
    model_changed: true,
    layout_only: false,
    summary: {},
    layers: [
      {
        layer: 'LA',
        label: 'Logical Architecture',
        counts: { added: 1, removed: 1, modified: 2 },
        kinds: [
          {
            kind: 'LogicalComponent',
            elements: [
              { id: 'LC-1', uuid: 'u1', name: 'New', kind: 'LogicalComponent', status: 'added', changes: [] },
              { id: 'LC-2', uuid: 'u2', name: 'Gone', kind: 'LogicalComponent', status: 'removed', changes: [] },
              { id: 'LC-3', uuid: 'u3', name: 'Ren', kind: 'LogicalComponent', status: 'modified', renamed: { from: 'Old', to: 'Ren' }, changes: [] },
              { id: 'LC-4', uuid: 'u4', name: 'Mod', kind: 'LogicalComponent', status: 'modified', changes: [{ field: 'latency', old: '20 ms', new: '30 ms' }] },
            ],
          },
        ],
      },
    ],
    relationships: [],
    consequences: {
      gate_before: { standard: 'ISO26262', passed: true, blockers: [] },
      gate_after: { standard: 'ISO26262', passed: false, blockers: ['x'] },
      new_blockers: ['x'],
      removed_blockers: [],
      newly_unsatisfied: [],
      newly_unverified: [],
      coverage_before: { requirements: 1, satisfied: 1, verified: 1 },
      coverage_after: { requirements: 1, satisfied: 1, verified: 0 },
      constraints: [],
      budgets: [],
      impacted: 3,
      impacted_blockers: 1,
    },
    layout: [],
  };
}

test('diffArgs asks for the JSON of a revision against the working tree or another one', () => {
  assert.deepEqual(diffArgs('m.arc', 'main', null, 'iso26262'), ['diff', 'm.arc', '--from', 'main', '--gate-standard', 'iso26262', '-f', 'json']);
  assert.deepEqual(diffArgs('m.arc', 'v1', 'v2', 'do178c').slice(0, 6), ['diff', 'm.arc', '--from', 'v1', '--to', 'v2']);
});

test('parseDiff reads the report and names a failure', () => {
  assert.equal(parseDiff(JSON.stringify(diff())).ok, true);
  const bad = parseDiff('', 'Error: Compilation error: unknown revision \'x\'');
  assert.deepEqual(bad, { ok: false, error: "unknown revision 'x'" });
});

test('the marks say added, removed, renamed and modified, with what changed', () => {
  const marks = compareMarks(diff());
  assert.deepEqual(marks.map((m) => [m.id, m.status]), [['LC-1', 'added'], ['LC-2', 'removed'], ['LC-3', 'renamed'], ['LC-4', 'modified']]);
  assert.deepEqual(marks[2].details, ['renamed: "Old" → "Ren"']);
  assert.deepEqual(marks[3].details, ['latency: "20 ms" → "30 ms"']);
  assert.equal(marks[0].layer, 'LA');
});

test('a moved element is marked moved only when nothing else changed', () => {
  const base = { id: 'a', uuid: 'u', name: 'a', kind: 'k', status: 'modified' as const };
  assert.equal(markStatus({ ...base, moved: { from: 'A', to: 'B' }, changes: [] }), 'moved');
  assert.equal(markStatus({ ...base, moved: { from: 'A', to: 'B' }, changes: [{ field: 'x', old: '1', new: '2' }] }), 'modified');
});

test('the layer tabs count the changes of their layer', () => {
  assert.deepEqual(compareCounts(diff()), { lab: { added: 1, removed: 1, modified: 2 } });
  const state = compareState(diff(), 'HEAD', 'HEAD');
  assert.deepEqual(state.ghosts, ['LC-2']);
  assert.equal(state.changed, true);
});

function payload(children: unknown[], edges: unknown[] = [], layout?: unknown): ViewerPayload {
  return { title: 't', graphs: [{ id: 'lab', arc: { view: 'lab', title: 'L' }, children, edges } as never], diagnostics: [], links: [], layout };
}

const node = (id: string, uuid: string, children: unknown[] = []) => ({ id, arc: { kind: 'logical_component', uuid }, children });

test('a removed box is put back inside the box that held it, with its exchanges', () => {
  const before = payload([node('P', 'up', [node('GONE', 'u2'), node('STAY', 's')]), node('Q', 'uq')], [{ id: 'ce:Flow', arc: { kind: 'x' }, sources: ['GONE'], targets: ['Q'] }], {
    arclang_layout: '1',
    views: {},
    arrangements: [{ view: 'lab', open: null, folded: [], places: { GONE: { dx: 40, dy: 5 }, STAY: { dx: 1, dy: 1 } } }],
  });
  const now = payload([node('P', 'up', [node('STAY', 's')]), node('Q', 'uq')]);
  const { diagrams, drawn } = withGhosts(now, before, ['LC-2'], ['u2']);
  const graph = diagrams.graphs[0];
  assert.deepEqual(graph.children![0].children!.map((c) => c.id), ['STAY', 'GONE']);
  assert.deepEqual(drawn.sort(), ['GONE', 'ce:Flow']);
  assert.equal(graph.edges!.length, 1);
  const places = (diagrams.layout as { arrangements: { places: Record<string, unknown> }[] }).arrangements[0].places;
  assert.deepEqual(places, { GONE: { dx: 40, dy: 5 } }, 'the old place comes along, only for the ghost');
  assert.equal(now.graphs[0].children![0].children!.length, 1, 'the model now is not touched');
});

test('a removed box whose container is gone too is drawn at the top of the diagram', () => {
  const before = payload([node('P', 'up', [node('GONE', 'u2')])]);
  const now = payload([]);
  const { diagrams } = withGhosts(now, before, [], ['up']);
  assert.deepEqual(diagrams.graphs[0].children!.map((c) => c.id), ['P']);
});

test('no ghost without an old drawing', () => {
  const now = payload([node('A', 'a')]);
  assert.deepEqual(withGhosts(now, null, ['x'], ['y']).drawn, []);
});

test('git refs are listed HEAD, branches, baselines, tags, remotes', () => {
  const out = ['refs/heads/main\tabc1234\tinitial', 'refs/heads/feature\tdef5678\twork', 'refs/tags/v1\t1111111\trelease', 'refs/tags/baseline/v1\t2222222\tbaseline', 'refs/remotes/origin/main\tabc1234\tinitial', 'refs/remotes/origin/HEAD\tabc1234\tx'].join('\n');
  const refs = parseRefs(out, 'main');
  assert.deepEqual(refs.map((r) => `${r.kind}:${r.name}`), ['head:HEAD', 'branch:feature', 'branch:main', 'baseline:baseline/v1', 'tag:v1', 'remote:origin/main']);
  assert.match(refs.find((r) => r.name === 'main' && r.kind === 'branch')!.detail, /current branch/);
});

const CONFLICT = ['model {', '<<<<<<< ours component \'Controller\' [LC-001]: description: "A" (ours) vs description: "B" (theirs)', '    component "C" {', '        description: "A"', '    }', '=======', '    component "C" {', '        description: "B"', '    }', '>>>>>>> theirs component \'Controller\' [LC-001]', '}'].join('\n');

test('conflict markers are read by element, with both sides', () => {
  const [c] = parseConflicts(CONFLICT);
  assert.equal(c.element, 'LC-001');
  assert.equal(c.label, "component 'Controller' [LC-001]");
  assert.match(c.reason, /description: "A" \(ours\) vs description: "B" \(theirs\)/);
  assert.equal(c.ours.length, 3);
  assert.equal(c.theirs[1].trim(), 'description: "B"');
  assert.equal(c.start, 1);
  assert.equal(c.end, 9);
});

test('accepting ours, theirs or both settles the conflict and keeps the rest', () => {
  const [c] = parseConflicts(CONFLICT);
  assert.equal(resolveConflict(CONFLICT, c, 'ours'), ['model {', '    component "C" {', '        description: "A"', '    }', '}'].join('\n'));
  assert.ok(resolveConflict(CONFLICT, c, 'theirs').includes('"B"') && !resolveConflict(CONFLICT, c, 'theirs').includes('"A"'));
  assert.equal(parseConflicts(resolveConflict(CONFLICT, c, 'both')).length, 0);
  assert.ok(resolveConflict(CONFLICT.replace(/\n/g, '\r\n'), parseConflicts(CONFLICT.replace(/\n/g, '\r\n'))[0], 'ours').includes('\r\n'));
});

test('two conflicts are resolved one after the other', () => {
  const text = `${CONFLICT}\n${CONFLICT}`;
  const first = parseConflicts(text);
  assert.equal(first.length, 2);
  const once = resolveConflict(text, first[1], 'theirs');
  assert.equal(parseConflicts(once).length, 1);
});

test('a half marker is not a conflict', () => {
  assert.deepEqual(parseConflicts('<<<<<<< ours x: y\nstuff\n'), []);
});

test('the history of an element is read and its changes spelled', () => {
  const json = JSON.stringify({ model: 'm', element: 'LC-1', id: 'LC-1', name: 'N', skipped: [], entries: [{ hash: 'h', short: 's', author: 'Ada', date: '2026-10-25T10:00:00Z', subject: 'x', status: 'modified', renamed: { from: 'a', to: 'b' }, changes: [{ field: 'p', old: '1', new: '2' }], relationships: [] }] });
  const parsed = parseHistory(json);
  assert.equal(parsed.ok, true);
  if (parsed.ok) assert.deepEqual(entryDetails(parsed.history.entries[0]), ['renamed: "a" → "b"', 'p: "1" → "2"']);
  assert.equal(parseHistory('', 'Error: nope').ok, false);
});

test('baseline names are checked before the compiler is asked', () => {
  assert.equal(validBaselineName('v1.0-rc1'), null);
  assert.ok(validBaselineName('bad name'));
  assert.ok(validBaselineName('../x'));
  assert.ok(validBaselineName(''));
});
