// The AI's side that is not the network: the context of a selection, the
// reading of answers, the shape of edit requests, the diff of a proposal,
// the consequences of an edit. Snapshots come from the real compiler when
// one is found (ARCLANG_BINARY, the repository's build, or the PATH).
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { test } from 'node:test';
import { parseAssistAnswer, parseSuggestions } from '../../src/ai/core/answer';
import { buildSelectionContext, declarationSpan, elementsInRange } from '../../src/ai/core/context';
import { lineDiff } from '../../src/ai/core/diff';
import { consequencesOf, diffSnapshots, touchedIds } from '../../src/ai/core/impact';
import { describeEdit, validateRequests } from '../../src/ai/core/requests';
import { isFromPageAi } from '../../src/ai/core/protocol';
import { Metamodel, parseSnapshot, Snapshot } from '../../src/pure/workbench';

const REFERENCE = fs.readFileSync(path.join(__dirname, '..', '..', 'media', 'reference.arc'), 'utf8');

function compiler(): string | null {
  const candidates = [process.env.ARCLANG_BINARY, path.resolve(__dirname, '../../../../target/release/arclang'), 'arclang'].filter((c): c is string => !!c);
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ['--version'], { encoding: 'utf8' });
    if (probe.status === 0) return candidate;
  }
  return null;
}
const ARCLANG = compiler();

function snapshotOf(text: string): Snapshot {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcai-'));
  try {
    fs.writeFileSync(path.join(dir, 'model.arc'), text);
    const out = spawnSync(ARCLANG!, ['workbench', path.join(dir, 'model.arc')], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
    return parseSnapshot(out.stdout);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

function metamodel(): Metamodel {
  const out = spawnSync(ARCLANG!, ['metamodel', '--format', 'json'], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  return JSON.parse(out.stdout) as Metamodel;
}

test('the declaration of an element: its block by id, a port by its owner, a name-only element by its line', () => {
  const camera = declarationSpan(REFERENCE, { id: 'LC-CAM', name: 'Camera', kind: 'LogicalComponent' });
  assert.ok(camera);
  assert.match(camera.text, /^  component "Camera" \{/);
  assert.match(camera.text, /function "Capture frame"/);
  assert.match(camera.text, /\n  \}$/);
  const port = declarationSpan(REFERENCE, { id: 'LC-CTL.steer', name: 'steer', kind: 'ComponentPort' });
  assert.equal(port?.text.trim(), 'port out "steer"');
  const exchange = declarationSpan(REFERENCE, { id: 'LaneFlow', name: 'LaneFlow', kind: 'ComponentExchange' });
  assert.match(exchange?.text ?? '', /component_exchange "LaneFlow" \{[\s\S]*to_port: "LC-CTL.lane"\n  \}/);
  const requirement = declarationSpan(REFERENCE, { id: 'REQ-LKA-001', name: 'REQ-LKA-001', kind: 'Requirement' });
  assert.match(requirement?.text ?? '', /requirement "REQ-LKA-001" \{[\s\S]*ASIL-B"\n  \}/);
  assert.equal(declarationSpan(REFERENCE, { id: 'NOPE', name: 'Nope', kind: 'X' }), null);
  // A brace in a string or a comment does not open a block.
  const tricky = 'logical_architecture "L" {\n  // a comment with { brace\n  component "A" {\n    id: "A"\n    description: "has } inside"\n  }\n}\n';
  assert.equal(declarationSpan(tricky, { id: 'A', name: 'A', kind: 'LogicalComponent' })?.text, '  component "A" {\n    id: "A"\n    description: "has } inside"\n  }');
});

test('the context of a selection holds its source, relationships, findings, metamodel and gate', { skip: !ARCLANG && 'no arclang compiler' }, () => {
  const snapshot = snapshotOf(REFERENCE);
  const built = buildSelectionContext({ snapshot, metamodel: metamodel(), text: REFERENCE, fileName: 'reference.arc', ids: ['LC-CAM', 'LC-CTL'], scope: 'selectionAndNeighbours' });
  assert.deepEqual(built.elements.map((e) => e.id), ['LC-CAM', 'LC-CTL']);
  assert.match(built.text, /### LogicalComponent LC-CAM "Camera", in `logical_architecture "Logical Architecture"`, lines 96-104/);
  assert.match(built.text, /## Top-level blocks[\s\S]*- logical_architecture "Logical Architecture"\n[\s\S]*- safety_analysis\n/);
  assert.match(built.text, /```arc\n  component "Camera" \{/);
  assert.match(built.text, /## Relationships of the selection/);
  assert.match(built.text, /LC-CTL satisfies REQ-LKA-001/);
  assert.match(built.text, /## Neighbours/);
  assert.match(built.text, /## Production gate: PASS, 2\/2/);
  assert.match(built.text, /## Metamodel of these kinds[\s\S]*LogicalComponent \(Logical/);
  const narrow = buildSelectionContext({ snapshot, metamodel: null, text: REFERENCE, fileName: 'reference.arc', ids: ['LC-CAM'], scope: 'selection' });
  assert.ok(narrow.text.length < built.text.length);
  assert.ok(!narrow.text.includes('## The whole model'));
  const whole = buildSelectionContext({ snapshot, metamodel: null, text: REFERENCE, fileName: 'reference.arc', ids: ['LC-CAM'], scope: 'wholeModel' });
  assert.match(whole.text, /## The whole model[\s\S]*diagram "Lane keeping chain"/);
  const clipped = buildSelectionContext({ snapshot, metamodel: null, text: REFERENCE, fileName: 'reference.arc', ids: ['LC-CAM'], scope: 'wholeModel', maxChars: 2000 });
  assert.ok(clipped.text.length < 2100);
});

test('a text selection designates the elements it declares, else the innermost one around it', { skip: !ARCLANG && 'no arclang compiler' }, () => {
  const records = snapshotOf(REFERENCE).elements!.elements;
  const start = REFERENCE.indexOf('  component "Camera"');
  const end = REFERENCE.indexOf('  component_exchange "LaneFlow"');
  assert.deepEqual(elementsInRange(REFERENCE, records, start, end).map((e) => e.id), ['LC-CAM', 'LC-CTL', 'LC-ACT']);
  const inside = REFERENCE.indexOf('safety_level: "ASIL-B"', REFERENCE.indexOf('"LC-CTL"'));
  assert.deepEqual(elementsInRange(REFERENCE, records, inside, inside + 5).map((e) => e.id), ['LC-CTL']);
});

test('answers: a propose_edits call, a json block, a whole model, or prose only', () => {
  const requests = [{ op: 'set', element: 'LC-CAM', key: 'safety_level', value: '"ASIL-D"' }];
  const fromTool = parseAssistAnswer('ignored prose', [{ id: '1', name: 'propose_edits', input: { explanation: 'Raises it.', requests } }]);
  assert.equal(fromTool.explanation, 'Raises it.');
  assert.deepEqual(fromTool.requests, requests);
  const fromJson = parseAssistAnswer('Here:\n```json\n{"explanation": "Raises it.", "requests": [{"op": "set", "element": "LC-CAM", "key": "safety_level", "value": "\\"ASIL-D\\""}]}\n```', []);
  assert.deepEqual(fromJson.requests, requests);
  const model = parseAssistAnswer('The whole model:\n```arc\nmodel M {}\nlogical_architecture "L" {}\n```\nDone.', []);
  assert.match(model.source ?? '', /logical_architecture/);
  assert.equal(model.explanation, 'The whole model:\n\nDone.');
  const prose = parseAssistAnswer('The camera captures frames.', []);
  assert.equal(prose.requests.length, 0);
  assert.equal(prose.source, null);
  assert.equal(prose.explanation, 'The camera captures frames.');
  const bad = parseAssistAnswer('', [{ id: '1', name: 'propose_edits', input: { explanation: 'x', requests: [{ op: 'rm -rf', element: '/' }] } }]);
  assert.match(bad.problem ?? '', /unknown op/);
});

test('edit requests are checked for shape; anything else of an answer is dropped', () => {
  assert.equal(validateRequests('nope').ok, false);
  assert.match((validateRequests([{ op: 'set', element: 'A', key: 'k' }]) as { error: string }).error, /no value/);
  assert.match((validateRequests([{ op: 'run_command', command: 'ls' }]) as { error: string }).error, /unknown op/);
  const ok = validateRequests([{ op: 'add', keyword: 'component', name: 'M', layer: 'logical_architecture', attributes: [['id', '"M"']], extra: 'dropped', __proto__: { polluted: true } }, { op: 'set', element: 'A', key: 'latency', value: 20 }]);
  assert.ok(ok.ok);
  assert.deepEqual(ok.requests[0], { op: 'add', keyword: 'component', name: 'M', layer: 'logical_architecture', attributes: [['id', '"M"']] });
  assert.equal((ok.requests[1] as { value: string }).value, '20');
  assert.equal(validateRequests(Array(41).fill({ op: 'remove', element: 'A' })).ok, false);
  assert.equal(describeEdit({ op: 'add_trace', from: 'A', kind: 'satisfies', to: 'R' }), 'trace A satisfies R');
  assert.equal(describeEdit({ op: 'add_raw', declaration: 'port in "x"', container: 'LC-MON' }), 'add port in "x" inside LC-MON');
});

test('suggestions: one to three proposals, the bad ones dropped with their reason', () => {
  const { suggestions, dropped } = parseSuggestions('', [{ id: '1', name: 'propose_changes', input: { proposals: [
    { title: 'Trace back', rationale: 'r', resolves: 'c', requests: [{ op: 'add_trace', from: 'A', kind: 'satisfies', to: 'R' }] },
    { title: 'Empty', rationale: 'r', resolves: 'c', requests: [] },
    { title: 'Bad', requests: [{ op: 'nope' }] },
    { title: 'Fourth', requests: [{ op: 'remove', element: 'X' }] },
  ] } }]);
  assert.deepEqual(suggestions.map((s) => s.title), ['Trace back']);
  assert.equal(dropped.length, 2);
  assert.equal(parseSuggestions('```json\n{"proposals": [{"title": "T", "requests": [{"op": "remove", "element": "X"}]}]}\n```', []).suggestions.length, 1);
});

test('the diff of a proposal: hunks with context, counts of lines', () => {
  const before = 'a\nb\nc\nd\ne\nf\ng\nh';
  const after = 'a\nb\nc\nD\ne\nf\ng\nh\ni';
  const diff = lineDiff(before, after, 1);
  assert.deepEqual(diff.lines, ['@@ -3,3 +3,3 @@', ' c', '-d', '+D', ' e', '@@ -8,1 +8,2 @@', ' h', '+i']);
  assert.equal(diff.added, 2);
  assert.equal(diff.removed, 1);
  assert.deepEqual(lineDiff('same', 'same').lines, []);
});

test('the consequences of an edit that removes a trace: the trace, the new finding, the gate blocker', { skip: !ARCLANG && 'no arclang compiler' }, () => {
  const before = snapshotOf(REFERENCE);
  const broken = REFERENCE.replace('trace "LC-CAM" satisfies "REQ-LKA-001" { rationale: "Delivers the lane model within its frame period" }\n', '').replace('trace "LC-CTL" satisfies "REQ-LKA-001" { rationale: "Plans the correction within the chain budget" }\n', '');
  const after = snapshotOf(broken);
  const change = diffSnapshots(before, after);
  assert.deepEqual(change.removedTraces.sort(), ['LC-CAM satisfies REQ-LKA-001', 'LC-CTL satisfies REQ-LKA-001']);
  const found = consequencesOf(before, after, change);
  const kinds = new Set(found.map((c) => c.kind));
  assert.ok(kinds.has('broken-trace'));
  assert.ok(kinds.has('new-finding'));
  assert.ok(kinds.has('unsatisfied-requirement'));
  assert.ok(kinds.has('gate'));
  assert.ok(found.some((c) => /REQ-LKA-001 is satisfied by no element/.test(c.message)));
  assert.ok(touchedIds(change).includes('LC-CAM'));
  // An edit that breaks nothing has no consequence.
  const renamed = snapshotOf(REFERENCE.replace('description: "Drives the vehicle"', 'description: "Drives the car"'));
  assert.deepEqual(consequencesOf(before, renamed), []);
});

test('only ai- messages of the page are the AI\'s', () => {
  assert.equal(isFromPageAi({ type: 'ai-ask', session: 's', ids: [], prompt: '' }), true);
  assert.equal(isFromPageAi({ type: 'edit' }), false);
  assert.equal(isFromPageAi(null), false);
  assert.equal(isFromPageAi('ai-ask'), false);
});
