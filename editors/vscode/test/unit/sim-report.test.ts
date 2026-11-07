import assert from 'node:assert/strict';
import { test } from 'node:test';
import { inflateRawSync } from 'node:zlib';
import { buildReport, buildReportPdf, buildReportPptx, chartsOf, parseExpectation, plot, ticks, type Replay } from '../../src/pure/sim-report';
import type { Outcome } from '../../src/pure/simulation';
import { unzip } from '../../src/pure/zip';

const outcome = (over: Partial<Outcome>): Outcome => ({
  test_case: 'TC-1', name: 'Pump failure', verifies: ['REQ-N1'], configuration_name: 'Two pumps', model: 'P.PumpFailure', verdict: 'fail', reason: '',
  values: [{ value: 'max(racks.gpu) = 356.7', expect: 'max("PN-RACKS".gpu) <= 85 °C', holds: true, margin: '1.48 K to spare' }, { value: 'min(racks.cap) = 0.6869', expect: 'min("PN-RACKS".cap) >= 0.99', holds: false, margin: 'short by 0.3031' }],
  tool: 'docker', omc_version: 'OpenModelica 1.27.1', log: '', seconds: 2, points: [], probability: null, estimate: null, sensitivity: [], ...over,
});

const replay: Replay = {
  time: [0, 3600, 7200, 10800, 14400],
  elements: [{ id: 'PN-RACKS', values: [{ name: 'gpu', series: [310, 340, 350, 355, 356.7] }, { name: 'cap', series: [1, 1, 1, 0.69, 0.7] }] }],
  expectations: [
    { text: 'max("PN-RACKS".gpu) <= 85 °C', holds: true, element: 'PN-RACKS' },
    { text: 'REQ-N1: min("PN-RACKS".cap) >= 0.99', holds: false, element: 'PN-RACKS', first_failure: 10800 },
    { text: 'final(ppue) <= 1.1', holds: true },
  ],
};

test('an expectation names its quantity, its bound and its unit', () => {
  assert.deepEqual(parseExpectation('max("PN-RACKS".gpu_temperature) <= 85 °C'), { element: 'PN-RACKS', name: 'gpu_temperature', bound: 85, unit: '°C' });
  assert.deepEqual(parseExpectation('REQ-PUE: final(ppue) <= 1.1'), { element: null, name: 'ppue', bound: 1.1, unit: '' });
  assert.deepEqual(parseExpectation('final(vehicleBody.distance) >= 300 km'), { element: null, name: 'vehicleBody.distance', bound: 300, unit: 'km' });
  assert.equal(parseExpectation('nothing to read'), null);
});

test('the curves come from the replay, in the unit the expectation is written in', () => {
  const charts = chartsOf(replay);
  assert.equal(charts.length, 2, 'a quantity the replay does not hold has no curve');
  assert.equal(charts[0].unit, '°C');
  assert.ok(Math.abs(charts[0].values[4] - 83.55) < 1e-9, 'kelvin shown in °C');
  assert.equal(charts[0].timeUnit, 'h');
  assert.deepEqual(charts[0].times, [0, 1, 2, 3, 4]);
  assert.equal(charts[1].failureAt, 3, 'the first failure, in hours');
  assert.equal(charts[1].bound, 0.99);
});

test('ticks are round and cover the range; a plot keeps its curve in its frame', () => {
  assert.deepEqual(ticks(0, 24), [0, 5, 10, 15, 20]);
  assert.deepEqual(ticks(0.7, 1), [0.7, 0.8, 0.9, 1]);
  assert.deepEqual(ticks(-12.5, 87), [0, 20, 40, 60, 80]);
  const p = plot(chartsOf(replay)[1], { x: 36, y: 40, w: 400, h: 200 });
  assert.ok(p.polyline.every(([x, y]) => x >= p.frame.x - 1e-9 && x <= p.frame.x + p.frame.w + 1e-9 && y >= p.frame.y && y <= p.frame.y + p.frame.h));
  assert.ok(p.bound !== null && p.failure !== null);
});

test('the report: a summary, then a page per test case with its verdict, values and curves', () => {
  const outcomes = [outcome({}), outcome({ test_case: 'TC-2', name: 'Envelope', verdict: 'pass', points: [{ label: 'ambient = 30 °C', values: [], holds: true, error: '', inputs: [] }] })];
  const report = buildReport('cooling', outcomes, new Map([['TC-1', replay], ['TC-2', replay]]), new Date('2026-11-02T10:00:00Z'));
  assert.equal(report.tool, 'OpenModelica 1.27.1');
  assert.equal(report.cases[0].charts.length, 2);
  assert.equal(report.cases[1].charts.length, 0, 'a sweep shows its points, not one point\'s curve');
  assert.match(report.cases[1].extra[0], /Sweep: 1 points, 1 hold/);
  assert.equal(report.cases[0].context, 'verifies REQ-N1 · configuration Two pumps · model P.PumpFailure');

  const pdf = Buffer.from(buildReportPdf(report)).toString('latin1');
  assert.match(pdf, /\/Type \/Pages \/Count 3 /, 'a summary and two cases');
  assert.ok(pdf.includes('[4 3] 0 d'), 'the bound, dashed');
  const xref = Number(/startxref\n(\d+)\n%%EOF\n$/.exec(pdf)?.[1]);
  for (const [i, m] of [...pdf.slice(xref).matchAll(/^(\d{10}) 00000 n $/gm)].entries()) assert.ok(pdf.slice(Number(m[1])).startsWith(`${i + 1} 0 obj\n`));

  const deck = unzip(buildReportPptx(report), (d) => new Uint8Array(inflateRawSync(d)));
  const names = deck.map((e) => e.name);
  assert.ok(names.includes('ppt/slides/slide3.xml') && !names.some((n) => n.startsWith('ppt/media/')), 'three slides of shapes, no picture');
  const slide2 = new TextDecoder().decode(deck.find((e) => e.name === 'ppt/slides/slide2.xml')!.data);
  assert.ok(slide2.includes('<a:t>TC-1 — Pump failure</a:t>'), 'the title');
  assert.ok(slide2.includes('<a:t>FAIL</a:t>') && slide2.includes('<a:custGeom>'), 'the verdict and a curve');
  const ids = [...slide2.matchAll(/<p:cNvPr id="(\d+)"/g)].map((m) => m[1]);
  assert.equal(new Set(ids).size, ids.length, 'every shape has its own id');
});
