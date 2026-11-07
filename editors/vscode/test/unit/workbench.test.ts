import { debugView, firePicks, nothingToRun, runPicks } from '../../src/pure/debug';
import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  attributeDisplay,
  attributeLiteral,
  connectPlan,
  connectTargetKinds,
  connectToolFor,
  createRequest,
  elementTree,
  ElementRecord,
  graphContainerOf,
  lifelineElement,
  Metamodel,
  neighbors,
  ownerOfPort,
  parseEditAnswer,
  parseSnapshot,
  portNameFrom,
  Snapshot,
  toolsFor,
  VIEWPOINTS,
  treeByLayer, nextRequirementId, requirementRequests, fragmentRequest, noteRequest, invariantRequest } from '../../src/pure/workbench';

const metamodel: Metamodel = {
  kinds: [
    { name: 'LogicalComponent', layer: 'Logical', attributes: [] },
    { name: 'LogicalFunction', layer: 'Logical', attributes: [] },
    { name: 'Requirement', layer: 'System', attributes: [] },
    { name: 'Model', layer: 'Transverse', attributes: [] },
  ],
};

const element = (uuid: string, id: string, kind: string, owner: string | null = null, name = id): ElementRecord => ({
  uuid,
  id,
  kind,
  name,
  owner,
  attributes: {},
});

test('a snapshot is read leniently and a broken answer is a failed one', () => {
  const ok = parseSnapshot('{"success":true,"errors":[],"warnings":["w"],"findings":[],"elements":{"elements":[],"relationships":[]},"diagrams":{"title":"t","graphs":[],"diagnostics":[],"links":[]}}');
  assert.equal(ok.success, true);
  assert.deepEqual(ok.warnings, ['w']);
  assert.ok(ok.elements && ok.diagrams);
  const broken = parseSnapshot('not json');
  assert.equal(broken.success, false);
  assert.equal(broken.errors[0], 'not json');
  const refused = parseEditAnswer('{"success":false,"error":"no element"}');
  assert.deepEqual(refused, { ok: false, error: 'no element' });
  assert.deepEqual(parseEditAnswer('{"success":true,"source":"model M {}"}'), { ok: true, source: 'model M {}' });
});

test('elements nest by ownership and group by the layer of their kind, under the model', () => {
  // The compiler makes the model own every top-level declaration.
  const elements = [
    element('m', 'M', 'Model'),
    element('c', 'LC-1', 'LogicalComponent', 'm'),
    element('f', 'LF-1', 'LogicalFunction', 'c'),
    element('r', 'REQ-1', 'Requirement', 'm'),
    element('x', 'X-1', 'Unknown'),
  ];
  const roots = elementTree(elements);
  assert.deepEqual(
    roots.map((n) => [n.element.id, n.children.map((c) => c.element.id)]),
    [
      ['M', ['LC-1', 'REQ-1']],
      ['X-1', []],
    ],
  );
  const groups = treeByLayer(elements, metamodel);
  assert.deepEqual(
    groups.map((g) => [g.layer, g.roots.map((r) => r.element.id)]),
    [
      ['Model', ['M']],
      ['System', ['REQ-1']],
      ['Logical', ['LC-1']],
      ['Transverse', ['X-1']],
    ],
  );
  assert.equal(groups[0].label, 'M');
});

test('typed fields become ArcLang literals', () => {
  assert.equal(attributeLiteral({ type: 'Text' }, 'Brake controller'), '"Brake controller"');
  assert.equal(attributeLiteral({ type: 'Text' }, '"already quoted"'), '"already quoted"');
  assert.equal(attributeLiteral({ type: 'Quantity' }, '20 ms'), '20 ms');
  assert.equal(attributeLiteral({ type: 'Number' }, '3.5'), '3.5');
  assert.equal(attributeLiteral({ type: 'Enum' }, 'ASIL-D'), 'ASIL-D');
  assert.equal(attributeLiteral({ type: 'Boolean' }, 'true'), 'true');
  assert.equal(attributeLiteral({ type: 'Reference' }, 'LC-001'), '"LC-001"');
  assert.equal(attributeLiteral({ type: 'Reference' }, 'Alpha'), 'Alpha');
  assert.equal(attributeLiteral({ type: 'List' }, 'LC-1, Beta, "x y"'), '["LC-1", Beta, "x y"]');
  assert.equal(attributeLiteral(undefined, 'free text'), '"free text"');
  assert.equal(attributeLiteral(undefined, '42'), '42');
});

test('a created element goes inside the selection, else in the layer or at the top', () => {
  const component = toolsFor('lab').find((t) => t.id === 'component')!;
  assert.deepEqual(createRequest(component, { view: 'lab', name: 'Radar', selected: null }), { op: 'add', keyword: 'component', name: 'Radar', layer: 'logical_architecture' });
  assert.deepEqual(createRequest(component, { view: 'lab', name: 'Radar', selected: { id: 'LC-1', kind: 'LogicalComponent' } }), {
    op: 'add',
    keyword: 'component',
    name: 'Radar',
    container: 'LC-1',
  });
  const port = toolsFor('lab').find((t) => t.id === 'port_in')!;
  assert.deepEqual(createRequest(port, { view: 'lab', name: 'lane', selected: null }), { error: `Port in: select ${port.containerKinds!.join(', ')} first` });
  const klass = toolsFor('cdb').find((t) => t.id === 'class')!;
  assert.deepEqual(createRequest(klass, { view: 'cdb', name: 'Frame', selected: { id: 'X', kind: 'Class' } }), { op: 'add', keyword: 'class', name: 'Frame', top_level: true });
  assert.deepEqual(createRequest(component, { view: 'lab', name: '  ', selected: null }), { error: 'a name is needed' });
});

test('connecting two components declares the ports and the exchange; links name the ends', () => {
  const exchange = toolsFor('lab').find((t) => t.id === 'component_exchange')!;
  const plan = connectPlan(exchange, { id: 'LC-A', kind: 'LogicalComponent' }, { id: 'LC-B', kind: 'LogicalComponent' }, 'Lane flow', 'lab');
  assert.ok(Array.isArray(plan));
  assert.deepEqual(plan[0], { op: 'add', keyword: 'port out', name: 'Lane_flow', container: 'LC-A' });
  assert.deepEqual(plan[1], { op: 'add', keyword: 'port in', name: 'Lane_flow', container: 'LC-B' });
  assert.equal(plan[2].op, 'add_raw');
  assert.match((plan[2] as { declaration: string }).declaration, /component_exchange "Lane flow" \{\n {2}from_port: "LC-A.Lane_flow"\n {2}to_port: "LC-B.Lane_flow"\n\}/);
  assert.equal((plan[2] as { layer?: string }).layer, 'logical_architecture');
  // Existing ports are used as they are.
  const viaPorts = connectPlan(exchange, { id: 'LC-A.out1', kind: 'ComponentPort' }, { id: 'LC-B.in1', kind: 'ComponentPort' }, 'x', 'lab');
  assert.ok(Array.isArray(viaPorts) && viaPorts.length === 1);
  const link = toolsFor('pab').find((t) => t.id === 'physical_link')!;
  const linkPlan = connectPlan(link, { id: 'PN-1', kind: 'PhysicalNode' }, { id: 'PN-2', kind: 'PhysicalNode' }, 'CAN', 'pab');
  assert.ok(Array.isArray(linkPlan));
  assert.match((linkPlan[0] as { declaration: string }).declaration, /physical_link "CAN" \{\n {2}from: "PN-1"\n {2}to: "PN-2"\n\}/);
  assert.deepEqual(connectPlan(link, { id: 'PN-1', kind: 'PhysicalNode' }, { id: 'LC-1', kind: 'LogicalComponent' }, 'x', 'pab'), {
    error: 'Physical link connects PhysicalNode; not PhysicalNode to LogicalComponent',
  });
  assert.deepEqual(connectPlan(link, { id: 'PN-1', kind: 'PhysicalNode' }, { id: 'PN-1', kind: 'PhysicalNode' }, 'x', 'pab'), { error: 'an element is not connected to itself' });
  assert.equal(ownerOfPort('LC-A.lane'), 'LC-A');
  assert.equal(portNameFrom('steering correction!'), 'steering_correction');
});

test('states, transitions and messages are declared inside the machine or the scenario of the diagram', () => {
  assert.equal(graphContainerOf('msm:LKA modes', 'msm'), 'LKA modes');
  assert.equal(graphContainerOf('es:Lane departure', 'es'), 'Lane departure');
  assert.equal(graphContainerOf('lab', 'lab'), null);
  assert.equal(lifelineElement('Lane departure.LC-CAM', 'Lane departure'), 'LC-CAM');
  const state = toolsFor('msm').find((t) => t.id === 'state')!;
  assert.deepEqual(createRequest(state, { view: 'msm', name: 'Fault', selected: null, graphContainer: 'LKA modes' }), { op: 'add', keyword: 'state', name: 'Fault', container: 'LKA modes' });
  assert.deepEqual(createRequest(state, { view: 'msm', name: 'Sub', selected: { id: 'LKA modes.Off', kind: 'State' }, graphContainer: 'LKA modes' }), {
    op: 'add',
    keyword: 'state',
    name: 'Sub',
    container: 'LKA modes.Off',
  });
  assert.deepEqual(createRequest(state, { view: 'msm', name: 'X', selected: null, graphContainer: null }), { error: 'State: this diagram is not about one state machine or scenario' });
  const transition = toolsFor('msm').find((t) => t.id === 'transition')!;
  const plan = connectPlan(transition, { id: 'LKA modes.Off', kind: 'State' }, { id: 'LKA modes.Armed', kind: 'State' }, '', 'msm', 'LKA modes');
  assert.deepEqual(plan, [{ op: 'add_raw', declaration: 'transition Off -> Armed { }', container: 'LKA modes' }]);
  const guarded = connectPlan(transition, { id: 'LKA modes.Off', kind: 'State' }, { id: 'LKA modes.Armed', kind: 'State' }, 'lane lost', 'msm', 'LKA modes');
  assert.deepEqual(guarded, [{ op: 'add_raw', declaration: 'transition Off -> Armed { guard: "lane lost" }', container: 'LKA modes' }]);
  const message = toolsFor('es').find((t) => t.id === 'message')!;
  assert.deepEqual(connectPlan(message, { id: 'LC-CAM', kind: 'LogicalComponent' }, { id: 'LC-CTL', kind: 'LogicalComponent' }, 'frame ready', 'es', 'Lane departure'), [
    { op: 'add_raw', declaration: 'message "LC-CAM" -> "LC-CTL" "frame ready"', container: 'Lane departure' },
  ]);
  assert.deepEqual(connectPlan(message, { id: 'A', kind: 'X' }, { id: 'B', kind: 'Y' }, '', 'es', 'S'), { error: 'a name is needed' });
});

test('a drag between two elements means the connect tool their kinds accept on the view', () => {
  assert.equal(connectToolFor('lab', 'LogicalComponent', 'LogicalComponent')?.id, 'component_exchange');
  assert.equal(connectToolFor('lab', 'LogicalFunction', 'LogicalFunction')?.id, 'functional_exchange');
  assert.equal(connectToolFor('lab', 'LogicalComponent', 'LogicalFunction'), null);
  assert.equal(connectToolFor('msm', 'State', 'State')?.id, 'transition');
  assert.equal(connectToolFor('es', 'LogicalComponent', 'Actor')?.id, 'message');
  assert.deepEqual(connectTargetKinds('lab', 'LogicalComponent'), ['LogicalComponent', 'ComponentPort']);
  assert.deepEqual(connectTargetKinds('lab', 'Requirement'), []);
  assert.equal(connectTargetKinds('es', 'Anything'), null);
});

test('the semantic browser lists relationships and traces around an element', () => {
  const a = element('a', 'LC-A', 'LogicalComponent');
  const b = element('b', 'LC-B', 'LogicalComponent');
  const r = element('r', 'REQ-1', 'Requirement');
  const snapshot: Snapshot = {
    success: true,
    errors: [],
    warnings: [],
    stats: null,
    document: null,
    findings: [],
    gate: null,
    elements: { elements: [a, b, r], relationships: [{ uuid: 'x', kind: 'ComponentExchange', name: 'flow', source: 'a', target: 'b' }] },
    diagrams: { title: 't', graphs: [], diagnostics: [], links: [{ kind: 'satisfies', source: 'LC-A', target: 'REQ-1' }] },
  };
  const around = neighbors(a, snapshot);
  assert.deepEqual(
    around.outgoing.map((e) => [e.kind, e.other?.id]),
    [
      ['ComponentExchange', 'LC-B'],
      ['satisfies', 'REQ-1'],
    ],
  );
  assert.deepEqual(neighbors(b, snapshot).incoming.map((e) => [e.kind, e.other?.id]), [['ComponentExchange', 'LC-A']]);
});

test('the next requirement id follows the highest one, keeping prefix and width', () => {
  assert.equal(nextRequirementId(['REQ-LKA-001', 'REQ-LKA-002']), 'REQ-LKA-003');
  assert.equal(nextRequirementId(['REQ-9', 'LC-CAM']), 'REQ-10');
  assert.equal(nextRequirementId([]), 'REQ-001');
});

test('a requirement goes into the requirements block, or a new one, and is traced when asked', () => {
  const inBlock = requirementRequests('REQ-1', 'Shall "stop"', 'LC-CTL', true);
  assert.deepEqual(inBlock[0], { op: 'add_raw', declaration: 'requirement "REQ-1" {\n  description: "Shall \\"stop\\""\n}', layer: 'requirements' });
  assert.deepEqual(inBlock[1], { op: 'add_trace', from: 'LC-CTL', kind: 'satisfies', to: 'REQ-1' });
  const alone = requirementRequests('REQ-1', 'x', null, false);
  assert.equal(alone.length, 1);
  assert.match((alone[0] as { declaration: string }).declaration, /^requirements \{\n  requirement "REQ-1" \{/);
  // Created with a requirement selected: the new one derives from it.
  const derived = requirementRequests('REQ-2', 'Cells stay under 55 °C', null, true, 'REQ-1');
  assert.deepEqual(derived[1], { op: 'add_trace', from: 'REQ-2', kind: 'derives', to: 'REQ-1' });
});

test('the requirements view connects by the kind of its tool, without asking', () => {
  const derive = toolsFor('req').find((t) => t.id === 'derive')!;
  assert.deepEqual(connectPlan(derive, { id: 'REQ-2', kind: 'Requirement' }, { id: 'REQ-1', kind: 'Requirement' }, 'derives', 'req'), [{ op: 'add_trace', from: 'REQ-2', kind: 'derives', to: 'REQ-1' }]);
  const refused = connectPlan(derive, { id: 'PN-1', kind: 'PhysicalNode' }, { id: 'REQ-1', kind: 'Requirement' }, 'derives', 'req');
  assert.ok('error' in refused && /connects Requirement/.test(refused.error));
  const satisfy = toolsFor('req').find((t) => t.id === 'satisfy')!;
  assert.deepEqual(connectPlan(satisfy, { id: 'PN-1', kind: 'PhysicalNode' }, { id: 'REQ-1', kind: 'Requirement' }, 'satisfies', 'req'), [{ op: 'add_trace', from: 'PN-1', kind: 'satisfies', to: 'REQ-1' }]);
  assert.ok(VIEWPOINTS.some((v) => v.view === 'req' && v.short === 'Requirements'));
});

test('a fragment request names its kind and label, and refuses an unknown kind', () => {
  assert.deepEqual(fragmentRequest('Lane departure', 'lane model', 'loop every cycle'), { op: 'wrap_fragment', scenario: 'Lane departure', kind: 'loop', label: 'every cycle', first: 'lane model', last: 'lane model' });
  assert.ok('error' in fragmentRequest('S', 'm', 'repeat'));
});

test('spatial blocks show and edit as ArcLang text', () => {
  const location = { lat: { value: 47.795, unit: 'deg' }, alt: { value: 180, unit: 'm' }, lon: { value: 3.661, unit: 'deg' } };
  assert.equal(attributeDisplay(location), '{ lat: 47.795 deg lon: 3.661 deg alt: 180 m }');
  assert.equal(attributeDisplay({ box: [{ value: 6, unit: 'm' }, { value: 2.5, unit: 'm' }, { value: 2.6, unit: 'm' }] }), '{ box: [6 m, 2.5 m, 2.6 m] }');
  const route = [{ lat: { value: 1, unit: 'deg' }, lon: { value: 2, unit: 'deg' } }, { lat: { value: 3, unit: 'deg' }, lon: { value: 4, unit: 'deg' } }];
  assert.equal(attributeDisplay(route), '[{ lat: 1 deg lon: 2 deg }, { lat: 3 deg lon: 4 deg }]');
  assert.equal(attributeLiteral({ type: 'Location' }, '{ lat: 1 deg lon: 2 deg }'), '{ lat: 1 deg lon: 2 deg }');
  assert.equal(attributeLiteral({ type: 'List', of: 'Location' }, attributeDisplay(route)), attributeDisplay(route), 'a route is not split at its commas');
  assert.equal(attributeDisplay(['a', 'b']), 'a, b', 'plain lists stay joined');
});

test('a note is written at the top of the file about the selection, and asks for both', () => {
  assert.deepEqual(noteRequest('  Runs at "30" Hz ', ['LC-CAM', 'LC-CTL']), { op: 'add_raw', declaration: 'note "Runs at \\"30\\" Hz" { about: ["LC-CAM", "LC-CTL"] }', top_level: true });
  assert.ok('error' in (noteRequest('x', []) as object));
  assert.ok('error' in (noteRequest('  ', ['LC-CAM']) as object));
});

test('a state is asserted on the lifeline offered, or on the one named, as a state, a mode or a function', () => {
  const lines = ['LC-A', 'LC-B'];
  assert.deepEqual(invariantRequest('Run', 'tick', 'ack', 'LC-B', lines, ' Armed '), { op: 'wrap_invariant', scenario: 'Run', keyword: 'state', lifeline: 'LC-B', name: 'Armed', first: 'tick', last: 'ack' });
  assert.deepEqual(invariantRequest('Run', 'tick', 'tick', 'LC-B', lines, 'mode Lane keeping'), { op: 'wrap_invariant', scenario: 'Run', keyword: 'mode', lifeline: 'LC-B', name: 'Lane keeping', first: 'tick', last: 'tick' });
  assert.deepEqual(invariantRequest('Run', 'tick', 'tick', 'LC-B', lines, 'LC-A: function Plan'), { op: 'wrap_invariant', scenario: 'Run', keyword: 'function', lifeline: 'LC-A', name: 'Plan', first: 'tick', last: 'tick' });
  assert.deepEqual(invariantRequest('Run', 'tick', 'tick', 'LC-B', lines, 'state'), { op: 'wrap_invariant', scenario: 'Run', keyword: 'state', lifeline: 'LC-B', name: 'state', first: 'tick', last: 'tick' });
  assert.ok('error' in (invariantRequest('Run', 'tick', 'tick', 'LC-B', lines, 'Driver: Armed') as object));
  assert.ok('error' in (invariantRequest('Run', 'tick', 'tick', 'LC-B', lines, '  ') as object));
});

test('the debugger shows the diagram of what runs, and offers what the model runs', () => {
  assert.equal(debugView({ type: 'debug-state', kind: 'machine', name: 'LKA modes', nodes: [], edges: [] }), 'msm:LKA modes');
  assert.equal(debugView({ type: 'debug-state', kind: 'scenario', name: 'Lane departure', nodes: [], edges: ['msg:1'] }), 'es:Lane departure');
  assert.equal(debugView({ type: 'debug-state', kind: 'chain', name: 'Reaction', nodes: ['SF-A'], edges: [] }), null);
  const picks = runPicks({ machines: ['LKA modes'], scenarios: ['Lane departure'], chains: ['Reaction'] });
  assert.deepEqual(picks.map((p) => [p.key, p.name]), [['machine', 'LKA modes'], ['scenario', 'Lane departure'], ['chain', 'Reaction']]);
  assert.deepEqual(firePicks(['SF-DET']).map((p) => p.event), ['SF-DET', null, '?']);
});

test('a model with nothing to run step by step is told so, not that it does not compile', () => {
  assert.deepEqual(runPicks({ machines: [], scenarios: [], chains: [] }), []);
  const simulated = nothingToRun(true);
  assert.match(simulated.text, /nothing to run step by step/);
  assert.doesNotMatch(simulated.text, /compile/);
  assert.equal(simulated.action, 'Run simulations', 'a model verified by simulation is offered its simulations');
  assert.equal(nothingToRun(false).action, null);
});
