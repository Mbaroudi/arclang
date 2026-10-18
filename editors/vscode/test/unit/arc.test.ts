import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  condenseMetamodel,
  exitLine,
  extractArcFence,
  importFormatFor,
  parseCheckReport,
  renderTransitionPlan,
  splitCommand,
  summarizeCheck,
} from '../../src/pure/arc';

test('the last arc fence of an answer is the model, with a final newline', () => {
  const answer = 'Assumptions:\n- one\n```arc\nmodel A {}\n```\ntext\n```arc\nmodel B {\n  version: "1"\n}\n```\nRefine next:';
  assert.equal(extractArcFence(answer), 'model B {\n  version: "1"\n}\n');
});

test('an answer without a model gives null, a bare model is accepted', () => {
  assert.equal(extractArcFence('No model here.'), null);
  assert.equal(extractArcFence('model X {\n}\n'), 'model X {\n}\n');
  assert.equal(extractArcFence('```json\n{}\n```'), null);
});

test('the import format follows the extension, and is asked for ambiguous ones', () => {
  assert.equal(importFormatFor('vehicle.sysml'), 'sysml');
  assert.equal(importFormatFor('Project.aird'), 'capella');
  assert.equal(importFormatFor('reqs.reqif'), 'req-if');
  assert.equal(importFormatFor('flow.mmd'), 'mermaid');
  assert.equal(importFormatFor('records.json'), null);
  assert.equal(importFormatFor('model.xml'), null);
});

test('a check report is read leniently and summarized by severity', () => {
  const report = parseCheckReport(JSON.stringify({
    file: 'm.arc',
    errors: [],
    findings: [
      { rule: 'trace.component', severity: 'warning', message: 'Component A has no trace', element: 'A' },
      { rule: 'metamodel.type', severity: 'error', message: 'latency: expected a time', element: 'F' },
      { rule: 'gate.info', severity: 'info', message: 'ok' },
    ],
  }));
  assert.ok(report);
  const summary = summarizeCheck(report);
  assert.match(summary, /^1 error:\n  - \[metamodel.type\]/m);
  assert.match(summary, /1 warning:\n  - \[trace.component\] Component A has no trace/);
  assert.equal(parseCheckReport('not json'), null);
  assert.equal(summarizeCheck({ errors: [], findings: [] }), 'The model compiles with no finding.');
  const broken = parseCheckReport(JSON.stringify({ errors: [{ message: 'Parser error' }], findings: [] }));
  assert.deepEqual(broken?.errors, ['{"message":"Parser error"}']);
});

test('a transition plan is rendered with what is done and what is to create', () => {
  const text = renderTransitionPlan(
    [
      { step: 'oa-to-sa', label: 'Operational activities to system functions', candidates: [
        { id: 'OA-1', name: 'Monitor', kind: 'operational_activity', realized_by: ['Acquire'] },
        { id: 'OA-2', name: 'Alert', kind: 'operational_activity', realized_by: [] },
      ] },
      { step: 'sa-to-la', label: 'System functions to logical components', candidates: [], blocked: 'no logical architecture' },
    ],
    'm.arc',
  );
  assert.match(text, /^Operational activities to system functions \(oa-to-sa\): 1 realized, 1 to create$/m);
  assert.match(text, /^  done  operational_activity Monitor realized by Acquire$/m);
  assert.match(text, /^  todo  operational_activity Alert$/m);
  assert.match(text, /next: arclang transition m.arc --step oa-to-sa --write/);
  assert.match(text, /\(sa-to-la\): 0 realized, 0 to create; no logical architecture/);
});

test('the metamodel is condensed by layer with typed attributes, keywords, enums, traces and units', () => {
  const json = JSON.stringify({
    version: '4.0.0',
    kinds: [
      { name: 'Actor', layer: 'Operational', doc: 'Human or external system.', attributes: [
        { key: 'id', type: { type: 'Identifier' } }, { key: 'name', type: { type: 'Text' } },
        { key: 'safety_level', type: { type: 'Enum', of: 'SafetyLevel' } },
      ] },
      { name: 'FunctionalChain', layer: 'System', doc: 'A path.', attributes: [
        { key: 'involves', type: { type: 'List', of: { type: 'Reference', of: 'Element' } } },
        { key: 'latency_budget', type: { type: 'Quantity', of: 'Time' } },
      ] },
    ],
    enums: [{ name: 'SafetyLevel', values: ['QM', 'ASIL-A'] }],
    trace_rules: [{ kind: 'satisfies', from: 'component', to: 'Requirement', doc: 'Counted by the gate.' }],
    units: [{ symbol: 'ms', dimension: 'Time' }, { symbol: 's', dimension: 'Time' }, { symbol: 'Mbps', dimension: 'DataRate' }],
  });
  const text = condenseMetamodel(json);
  assert.match(text, /## Operational\n- Actor \(keyword `actor`\): Human or external system\. Attributes: id, name, description, safety_level: enum<SafetyLevel>\./);
  assert.match(text, /FunctionalChain \(keyword `functional_chain`\).*involves: list<reference<Element>>, latency_budget: quantity<Time>/);
  assert.match(text, /- SafetyLevel: QM, ASIL-A/);
  assert.match(text, /- satisfies: from component to Requirement\. Counted by the gate\./);
  assert.match(text, /- Time: ms s\n- DataRate: Mbps/);
});

test('exit lines and command splitting', () => {
  assert.equal(exitLine('check', 0), '`arclang check` succeeded (exit 0).');
  assert.match(exitLine('check', 127), /not found/);
  assert.match(exitLine('gate', 2), /failed \(exit 2\)/);
  assert.deepEqual(splitCommand('uv run --directory "/p/with space/mcp-server" arclang-mcp'), ['uv', 'run', '--directory', '/p/with space/mcp-server', 'arclang-mcp']);
  assert.deepEqual(splitCommand(''), []);
});
